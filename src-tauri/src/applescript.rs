use crate::errors::AppError;

/// Approved AppleScript snippets only — no arbitrary script execution.
/// Each template is a single, validated expression.
pub enum AppleScriptTemplate {
    MuteVolume,
    UnmuteVolume,
    SetOutputVolume(u8),
    GetOutputVolume,
    DndEnable,
    DndDisable,
    SetDisplayBrightness(u8),
}

impl AppleScriptTemplate {
    fn script(&self) -> String {
        match self {
            AppleScriptTemplate::MuteVolume => "set volume with output muted".to_string(),
            AppleScriptTemplate::UnmuteVolume => "set volume without output muted".to_string(),
            AppleScriptTemplate::SetOutputVolume(level) => {
                format!("set volume output volume {level}")
            }
            AppleScriptTemplate::GetOutputVolume => {
                "output volume of (get volume settings)".to_string()
            }
            AppleScriptTemplate::DndEnable => {
                // Enable first Focus mode (requires macOS Monterey+)
                // Activates the most recent or default focus mode
                "tell application \"System Events\" to key code 20 using {shift down, option down, command down}".to_string()
            }
            AppleScriptTemplate::DndDisable => {
                // Disable Focus mode by pressing the same shortcut again
                "tell application \"System Events\" to key code 20 using {shift down, option down, command down}".to_string()
            }
            AppleScriptTemplate::SetDisplayBrightness(level) => {
                // Set display brightness 0-100 via AppleScript (requires permissions)
                let brightness = (*level as f32 / 100.0).min(1.0).max(0.0);
                format!(
                    "tell application \"System Events\" to tell display preferences to set brightness to {:.2}",
                    brightness
                )
            }
        }
    }
}

/// Run a validated AppleScript template. Requires macOS.
pub fn run_template(template: AppleScriptTemplate) -> Result<String, AppError> {
    run_script(&template.script())
}

/// Run a pre-validated script string via `osascript -e`.
/// Caller is responsible for ensuring the script is from an approved template.
pub fn run_validated_script(script: &str) -> Result<String, AppError> {
    run_script(script)
}

/// Get current system output volume as a 0–100 integer.
/// Returns None if unavailable (non-macOS or osascript failure).
pub fn get_volume() -> Option<u8> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("osascript")
            .args(["-e", "output volume of (get volume settings)"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&output.stdout);
        s.trim().parse::<u8>().ok()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Enable Do Not Disturb / first Focus mode.
/// Requires macOS Monterey+ and Focus mode to be configured.
pub fn enable_dnd() -> Result<String, AppError> {
    run_template(AppleScriptTemplate::DndEnable)
}

/// Disable Do Not Disturb / active Focus mode.
/// Requires macOS Monterey+ and Focus mode to be configured.
pub fn disable_dnd() -> Result<String, AppError> {
    run_template(AppleScriptTemplate::DndDisable)
}

/// Set display brightness as a 0–100 percentage.
/// Requires macOS and Accessibility permissions.
pub fn set_brightness(level: u8) -> Result<String, AppError> {
    run_template(AppleScriptTemplate::SetDisplayBrightness(level))
}

/// Return true when the osascript stderr indicates a permission denial.
/// macOS uses error code -1743 ("Not authorized to send Apple events") and
/// similar messages for both Apple Events and Accessibility refusals.
#[cfg(any(target_os = "macos", test))]
fn is_permission_denied(stderr: &str) -> bool {
    stderr.contains("Not authorized")
        || stderr.contains("-1743")
        || stderr.contains("not allowed to send")
        || stderr.contains("access for assistive devices")
        || stderr.contains("is not allowed assistive access")
}

#[cfg(target_os = "macos")]
fn run_script(script: &str) -> Result<String, AppError> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| AppError::ExecutionError(format!("osascript launch failed: {e}")))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if is_permission_denied(&stderr) {
            Err(AppError::PermissionDenied(
                "Apple Events permission required. Grant access in System Settings → Privacy & Security → Automation.".to_string(),
            ))
        } else {
            Err(AppError::ExecutionError(format!(
                "osascript exited with error: {stderr}"
            )))
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn run_script(_script: &str) -> Result<String, AppError> {
    Err(AppError::PlatformNotSupported(
        "AppleScript requires macOS".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_denied_detection() {
        assert!(is_permission_denied(
            "execution error: Not authorized to send Apple events (-1743)"
        ));
        assert!(is_permission_denied(
            "osascript is not allowed to send keystrokes"
        ));
        assert!(is_permission_denied("-1743"));
        assert!(!is_permission_denied("syntax error near line 1"));
        assert!(!is_permission_denied(""));
    }

    #[test]
    fn test_dnd_enable_script() {
        let script = AppleScriptTemplate::DndEnable.script();
        assert!(script.contains("key code 20"));
        assert!(script.contains("shift down"));
        assert!(script.contains("option down"));
        assert!(script.contains("command down"));
    }

    #[test]
    fn test_dnd_disable_script() {
        let script = AppleScriptTemplate::DndDisable.script();
        assert!(script.contains("key code 20"));
    }

    #[test]
    fn test_brightness_script_generation() {
        let script_50 = AppleScriptTemplate::SetDisplayBrightness(50).script();
        assert!(script_50.contains("0.50")); // 50/100 = 0.5

        let script_100 = AppleScriptTemplate::SetDisplayBrightness(100).script();
        assert!(script_100.contains("1.00")); // 100/100 = 1.0

        let script_0 = AppleScriptTemplate::SetDisplayBrightness(0).script();
        assert!(script_0.contains("0.00")); // 0/100 = 0.0
    }

    #[test]
    fn test_brightness_clamping() {
        // Over 100 should clamp to 1.0
        let script = AppleScriptTemplate::SetDisplayBrightness(255).script();
        assert!(script.contains("1.00"));
    }
}
