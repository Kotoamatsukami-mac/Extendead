use crate::models::{PermState, PermissionStatus};

/// Check permission state for accessibility and Apple Events.
/// On macOS we perform lightweight probes. On other platforms we report Unknown.
pub fn get_permission_status() -> PermissionStatus {
    PermissionStatus {
        accessibility: check_accessibility(),
        apple_events: check_apple_events(),
    }
}

/// Accessibility check.
/// AXIsProcessTrusted() is the authoritative API but requires unsafe FFI.
/// For Phase 2 we report Unknown and surface the banner so the user can grant
/// access proactively. Phase 3 will add the native binding.
#[cfg(target_os = "macos")]
fn check_accessibility() -> PermState {
    PermState::Unknown
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
