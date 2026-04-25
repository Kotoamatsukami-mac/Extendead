use crate::models::{PermState, PermissionStatus};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

const PERMISSION_CACHE_TTL_SECS: u64 = 30;
const PERMISSION_CACHE_TTL: Duration = Duration::from_secs(PERMISSION_CACHE_TTL_SECS);

#[derive(Clone)]
struct CachedPermissionStatus {
    captured_at: Instant,
    status: PermissionStatus,
}

static PERMISSION_STATUS_CACHE: LazyLock<Mutex<Option<CachedPermissionStatus>>> =
    LazyLock::new(|| Mutex::new(None));

/// Check permission state for accessibility and Apple Events.
/// On macOS we perform lightweight probes. On other platforms we report Unknown.
pub fn get_permission_status() -> PermissionStatus {
    get_permission_status_with_cache(
        &PERMISSION_STATUS_CACHE,
        Instant::now(),
        compute_permission_status,
    )
}

/// Drop cached permission state so the next read re-probes the system.
pub fn invalidate_permission_cache() {
    invalidate_permission_cache_for(&PERMISSION_STATUS_CACHE);
}

fn compute_permission_status() -> PermissionStatus {
    PermissionStatus {
        accessibility: check_accessibility(),
        apple_events: check_apple_events(),
    }
}

fn get_permission_status_with_cache(
    cache: &Mutex<Option<CachedPermissionStatus>>,
    now: Instant,
    compute: impl FnOnce() -> PermissionStatus,
) -> PermissionStatus {
    let mut guard = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(cached) = guard.as_ref() {
        let fresh = now
            .checked_duration_since(cached.captured_at)
            .is_some_and(|age| age < PERMISSION_CACHE_TTL);
        if fresh {
            return cached.status.clone();
        }
    }

    let status = compute();
    *guard = Some(CachedPermissionStatus {
        captured_at: now,
        status: status.clone(),
    });
    status
}

fn invalidate_permission_cache_for(cache: &Mutex<Option<CachedPermissionStatus>>) {
    let mut guard = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = None;
}

/// Accessibility check.
/// Use the native ApplicationServices AXIsProcessTrusted() API so the shell can
/// report the real trust state instead of a placeholder `unknown` value.
#[cfg(target_os = "macos")]
fn check_accessibility() -> PermState {
    if unsafe { ax_is_process_trusted() } {
        PermState::Granted
    } else {
        PermState::Denied
    }
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility() -> PermState {
    PermState::Unknown
}

/// Apple Events check.
/// Probe by reading the output volume — a harmless osascript call that exercises
/// the osascript runtime without targeting another application (so it does not
/// trigger an Apple Events permission dialog on its own). If osascript is
/// blocked at the process level, this will fail with an error.
#[cfg(target_os = "macos")]
fn check_apple_events() -> PermState {
    let result = std::process::Command::new("osascript")
        .args(["-e", "output volume of (get volume settings)"])
        .output();
    match result {
        Ok(o) if o.status.success() => PermState::Granted,
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            // If the error mentions authorization, the user explicitly denied.
            if stderr.contains("Not authorized") || stderr.contains("-1743") {
                PermState::Denied
            } else {
                // osascript ran but returned a non-zero exit for another reason.
                PermState::Unknown
            }
        }
        Err(_) => PermState::Unknown,
    }
}

#[cfg(not(target_os = "macos"))]
fn check_apple_events() -> PermState {
    PermState::Unknown
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> std::os::raw::c_uchar;
}

#[cfg(target_os = "macos")]
unsafe fn ax_is_process_trusted() -> bool {
    AXIsProcessTrusted() != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(state: PermState) -> PermissionStatus {
        PermissionStatus {
            accessibility: state.clone(),
            apple_events: state,
        }
    }

    #[test]
    fn first_call_computes_permission_status() {
        let cache = Mutex::new(None);
        let now = Instant::now();
        let mut compute_calls = 0;

        let value = get_permission_status_with_cache(&cache, now, || {
            compute_calls += 1;
            status(PermState::Granted)
        });

        assert_eq!(compute_calls, 1);
        assert!(matches!(value.accessibility, PermState::Granted));
        assert!(matches!(value.apple_events, PermState::Granted));
    }

    #[test]
    fn second_call_inside_ttl_uses_cache() {
        let cache = Mutex::new(None);
        let now = Instant::now();
        let mut compute_calls = 0;

        let _first = get_permission_status_with_cache(&cache, now, || {
            compute_calls += 1;
            status(PermState::Granted)
        });
        let second = get_permission_status_with_cache(&cache, now + Duration::from_secs(5), || {
            compute_calls += 1;
            status(PermState::Denied)
        });

        assert_eq!(compute_calls, 1);
        assert!(matches!(second.accessibility, PermState::Granted));
        assert!(matches!(second.apple_events, PermState::Granted));
    }

    #[test]
    fn invalidation_forces_recompute() {
        let cache = Mutex::new(None);
        let now = Instant::now();
        let mut compute_calls = 0;

        let _first = get_permission_status_with_cache(&cache, now, || {
            compute_calls += 1;
            status(PermState::Granted)
        });
        invalidate_permission_cache_for(&cache);
        let second = get_permission_status_with_cache(&cache, now + Duration::from_secs(1), || {
            compute_calls += 1;
            status(PermState::Denied)
        });

        assert_eq!(compute_calls, 2);
        assert!(matches!(second.accessibility, PermState::Denied));
        assert!(matches!(second.apple_events, PermState::Denied));
    }

    #[test]
    fn expired_ttl_recomputes_permission_status() {
        let cache = Mutex::new(None);
        let now = Instant::now();
        let mut compute_calls = 0;

        let _first = get_permission_status_with_cache(&cache, now, || {
            compute_calls += 1;
            status(PermState::Granted)
        });
        let second = get_permission_status_with_cache(
            &cache,
            now + Duration::from_secs(PERMISSION_CACHE_TTL_SECS + 1),
            || {
                compute_calls += 1;
                status(PermState::Denied)
            },
        );

        assert_eq!(compute_calls, 2);
        assert!(matches!(second.accessibility, PermState::Denied));
        assert!(matches!(second.apple_events, PermState::Denied));
    }
}
