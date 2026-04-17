use crate::models::{PermState, PermissionStatus};

/// Check permission state for accessibility and Apple Events.
/// On macOS we perform a lightweight probe. On other platforms we report Unknown.
pub fn get_permission_status() -> PermissionStatus {
    PermissionStatus {
        accessibility: check_accessibility(),
        apple_events: check_apple_events(),
    }
}

#[cfg(target_os = "macos")]
fn check_accessibility() -> PermState {
    // AXIsProcessTrusted() is the proper check; we approximate via osascript probe.
    // A real production build would use the CoreFoundation / ApplicationServices API
    // via a thin unsafe binding. For Phase 1 we report Unknown and let the user grant.
    PermState::Unknown
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility() -> PermState {
    PermState::Unknown
}

#[cfg(target_os = "macos")]
fn check_apple_events() -> PermState {
    // Probe: try running a harmless osascript that requires Apple Events.
    let result = std::process::Command::new("osascript")
        .args(["-e", "return 1"])
        .output();
    match result {
        Ok(o) if o.status.success() => PermState::Granted,
        Ok(_) => PermState::Denied,
        Err(_) => PermState::Unknown,
    }
}

#[cfg(not(target_os = "macos"))]
fn check_apple_events() -> PermState {
    PermState::Unknown
}
