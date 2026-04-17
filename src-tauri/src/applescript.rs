use crate::errors::AppError;

/// Approved AppleScript snippets only — no arbitrary script execution.
/// Each template is a single, validated expression.
pub enum AppleScriptTemplate {
    MuteVolume,
    UnmuteVolume,
    SetOutputVolume(u8),
    GetOutputVolume,
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
        Err(AppError::ExecutionError(format!(
            "osascript exited with error: {stderr}"
        )))
    }
}

#[cfg(not(target_os = "macos"))]
fn run_script(_script: &str) -> Result<String, AppError> {
    Err(AppError::PlatformNotSupported(
        "AppleScript requires macOS".to_string(),
    ))
}
