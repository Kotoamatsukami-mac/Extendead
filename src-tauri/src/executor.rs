use std::time::Instant;

use tauri::{AppHandle, Emitter, Runtime};

use crate::applescript;
use crate::errors::AppError;
use crate::events::{ExecutionEventPayload, EXECUTION_EVENT_NAME};
use crate::models::{
    ExecutionEvent, ExecutionEventKind, ExecutionOutcome, ExecutionResult, ParsedCommand,
    PermState, PermissionStatus, ResolvedAction,
};
use crate::path_policy;
use crate::permissions;
use crate::risk;
use crate::validator;

fn new_event(command_id: &str, kind: ExecutionEventKind, message: String) -> ExecutionEvent {
    ExecutionEvent {
        id: uuid::Uuid::new_v4().to_string(),
        command_id: command_id.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        kind,
        message,
    }
}

fn emit<R: Runtime>(app: &AppHandle<R>, timeline: &mut Vec<ExecutionEvent>, event: ExecutionEvent) {
    let _ = app.emit(
        EXECUTION_EVENT_NAME,
        ExecutionEventPayload {
            event: event.clone(),
        },
    );
    timeline.push(event);
}

pub struct ExecutionRun {
    pub result: ExecutionResult,
    pub events: Vec<ExecutionEvent>,
}

enum AppleScriptPermissionProfile {
    Volume,
    BrowserTab,
    Brightness,
    Other,
}

fn outcome_for_error(e: &AppError) -> ExecutionOutcome {
    match e {
        AppError::PermissionDenied(_) => ExecutionOutcome::Blocked,
        AppError::ValidationError(_) | AppError::ShellPolicyViolation(_) => {
            ExecutionOutcome::Blocked
        }
        _ => ExecutionOutcome::RecoverableFailure,
    }
}

fn human_message_for_error(e: &AppError) -> String {
    match e {
        AppError::PermissionDenied(detail) => {
            format!("✗ Permission required — {detail}")
        }
        AppError::ValidationError(detail) => format!("✗ Blocked: {detail}"),
        _ => format!("✗ {e}"),
    }
}

fn applescript_permission_profile(template_id: &str) -> AppleScriptPermissionProfile {
    match template_id {
        "mute_volume" | "unmute_volume" | "set_volume" | "get_volume" => {
            AppleScriptPermissionProfile::Volume
        }
        "browser_new_tab" | "browser_close_tab" | "browser_reopen_closed_tab" => {
            AppleScriptPermissionProfile::BrowserTab
        }
        "brightness_up" | "brightness_down" => AppleScriptPermissionProfile::Brightness,
        _ => AppleScriptPermissionProfile::Other,
    }
}

fn preflight_applescript_template(template_id: &str) -> Result<(), AppError> {
    let status = permissions::get_permission_status();
    preflight_applescript_template_for_status(template_id, &status)
}

fn preflight_applescript_template_for_status(
    template_id: &str,
    status: &PermissionStatus,
) -> Result<(), AppError> {
    match applescript_permission_profile(template_id) {
        AppleScriptPermissionProfile::Volume => {
            if status.apple_events == PermState::Denied {
                return Err(AppError::PermissionDenied(
                    "Automation permission is required for volume control. Grant Extendead access in System Settings -> Privacy & Security -> Automation.".to_string(),
                ));
            }
        }
        AppleScriptPermissionProfile::BrowserTab => {
            if status.accessibility == PermState::Denied {
                return Err(AppError::PermissionDenied(
                    "Accessibility permission is required for browser tab shortcuts. Grant Extendead access in System Settings -> Privacy & Security -> Accessibility.".to_string(),
                ));
            }
            if status.apple_events == PermState::Denied {
                return Err(AppError::PermissionDenied(
                    "Automation permission is required to control browser tabs. Grant Extendead access in System Settings -> Privacy & Security -> Automation.".to_string(),
                ));
            }
        }
        AppleScriptPermissionProfile::Brightness => {
            if status.accessibility == PermState::Denied {
                return Err(AppError::PermissionDenied(
                    "Accessibility permission is required for brightness shortcuts. Grant Extendead access in System Settings -> Privacy & Security -> Accessibility.".to_string(),
                ));
            }
        }
        AppleScriptPermissionProfile::Other => {}
    }
    Ok(())
}

pub fn execute<R: Runtime>(
    command: &ParsedCommand,
    route_index: usize,
    app: &AppHandle<R>,
) -> Result<ExecutionRun, AppError> {
    let start = Instant::now();
    let mut timeline = Vec::new();

    validator::validate(command, route_index)?;

    let route = command.routes.get(route_index).expect("already validated");
    let command_id = command.id.as_str();

    let pre_volume: Option<u8> = match &route.action {
        ResolvedAction::AppleScriptTemplate { template_id, .. } if template_id == "set_volume" => {
            applescript::get_volume()
        }
        _ => None,
    };

    emit(
        app,
        &mut timeline,
        new_event(
            command_id,
            ExecutionEventKind::Started,
            format!("Executing: {}", route.label),
        ),
    );

    let result = dispatch_action(command_id, &route.action, app, &mut timeline);

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(message) => {
            let inverse = if let Some(level) = pre_volume {
                Some(ResolvedAction::AppleScriptTemplate {
                    script: format!("set volume output volume {level}"),
                    template_id: "set_volume".to_string(),
                })
            } else {
                risk::inverse_action(&route.action)
            };

            emit(
                app,
                &mut timeline,
                new_event(command_id, ExecutionEventKind::Completed, message.clone()),
            );
            Ok(ExecutionRun {
                result: ExecutionResult {
                    command_id: command_id.to_string(),
                    outcome: ExecutionOutcome::Success,
                    message,
                    human_message: format!("✓ {}", route.label),
                    duration_ms,
                    inverse_action: inverse,
                },
                events: timeline,
            })
        }
        Err(e) => {
            let outcome = outcome_for_error(&e);
            let human_message = human_message_for_error(&e);
            let msg = e.to_string();
            emit(
                app,
                &mut timeline,
                new_event(command_id, ExecutionEventKind::Failed, msg.clone()),
            );
            Ok(ExecutionRun {
                result: ExecutionResult {
                    command_id: command_id.to_string(),
                    outcome,
                    message: msg,
                    human_message,
                    duration_ms,
                    inverse_action: None,
                },
                events: timeline,
            })
        }
    }
}

fn dispatch_action<R: Runtime>(
    command_id: &str,
    action: &ResolvedAction,
    app: &AppHandle<R>,
    timeline: &mut Vec<ExecutionEvent>,
) -> Result<String, AppError> {
    match action {
        ResolvedAction::OpenUrl {
            url,
            browser_bundle,
            browser_name,
        } => {
            emit(
                app,
                timeline,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Opening {url} in {browser_name}"),
                ),
            );
            open_url(url, browser_bundle, browser_name)
        }
        ResolvedAction::OpenApp {
            bundle_id,
            app_name,
        } => {
            emit(
                app,
                timeline,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Launching {app_name}"),
                ),
            );
            open_app(app_name, bundle_id)
        }
        ResolvedAction::QuitApp {
            bundle_id,
            app_name,
        } => {
            emit(
                app,
                timeline,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Closing {app_name}"),
                ),
            );
            quit_app(app_name, bundle_id)
        }
        ResolvedAction::AppleScriptTemplate {
            script,
            template_id,
        } => {
            preflight_applescript_template(template_id)?;
            let progress_msg = match template_id.as_str() {
                "mute_volume" => "Muting system audio output".to_string(),
                "unmute_volume" => "Unmuting system audio output".to_string(),
                "set_volume" => "Setting output volume".to_string(),
                "browser_new_tab" => "Opening a new browser tab".to_string(),
                "browser_close_tab" => "Closing browser tab".to_string(),
                "browser_reopen_closed_tab" => "Reopening closed browser tab".to_string(),
                "brightness_up" => "Increasing display brightness".to_string(),
                "brightness_down" => "Decreasing display brightness".to_string(),
                _ => "Running AppleScript".to_string(),
            };
            emit(
                app,
                timeline,
                new_event(command_id, ExecutionEventKind::Progress, progress_msg),
            );
            applescript::run_validated_script(script).map(|_| match template_id.as_str() {
                "mute_volume" => "System audio muted".to_string(),
                "unmute_volume" => "System audio unmuted".to_string(),
                "set_volume" => "Output volume set".to_string(),
                "browser_new_tab" => "New tab opened".to_string(),
                "browser_close_tab" => "Tab closed".to_string(),
                "browser_reopen_closed_tab" => "Closed tab reopened".to_string(),
                "brightness_up" => "Brightness increased".to_string(),
                "brightness_down" => "Brightness decreased".to_string(),
                _ => "Done".to_string(),
            })
        }
        ResolvedAction::OpenSystemPreferences { pane_url } => {
            emit(
                app,
                timeline,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    "Opening System Settings".to_string(),
                ),
            );
            open_url(pane_url, "", "System Settings")
        }
        ResolvedAction::OpenPath { path } => {
            emit(
                app,
                timeline,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Revealing {path} in Finder"),
                ),
            );
            open_path(path)
        }
        ResolvedAction::CreateFolder { path } => {
            emit(
                app,
                timeline,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Creating folder {path}"),
                ),
            );
            create_folder(path)
        }
        ResolvedAction::MovePath {
            source_path,
            destination_path,
        } => {
            emit(
                app,
                timeline,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Moving {source_path} to {destination_path}"),
                ),
            );
            move_path(source_path, destination_path)
        }
    }
}

#[cfg(target_os = "macos")]
fn open_url(url: &str, browser_bundle: &str, browser_name: &str) -> Result<String, AppError> {
    let mut cmd = std::process::Command::new("open");
    if !browser_bundle.is_empty() {
        cmd.args(["-b", browser_bundle]);
    }
    cmd.arg(url);
    let status = cmd
        .status()
        .map_err(|e| AppError::ExecutionError(format!("open failed: {e}")))?;
    if status.success() {
        Ok(format!("Opened {url} in {browser_name}"))
    } else {
        Err(AppError::ExecutionError(format!(
            "open exited with status {status}"
        )))
    }
}

#[cfg(not(target_os = "macos"))]
fn open_url(_url: &str, _browser_bundle: &str, _browser_name: &str) -> Result<String, AppError> {
    Err(AppError::PlatformNotSupported(
        "URL opening requires macOS".to_string(),
    ))
}

#[cfg(target_os = "macos")]
fn open_app(app_name: &str, bundle_id: &str) -> Result<String, AppError> {
    let mut cmd = std::process::Command::new("open");
    if !bundle_id.is_empty() {
        cmd.args(["-b", bundle_id]);
    } else {
        cmd.args(["-a", app_name]);
    }
    let status = cmd
        .status()
        .map_err(|e| AppError::ExecutionError(format!("open failed: {e}")))?;
    if status.success() {
        Ok(format!("{app_name} launched"))
    } else {
        Err(AppError::ExecutionError(format!(
            "open {app_name} exited with status {status}"
        )))
    }
}

#[cfg(not(target_os = "macos"))]
fn open_app(_app_name: &str, _bundle_id: &str) -> Result<String, AppError> {
    Err(AppError::PlatformNotSupported(
        "App launching requires macOS".to_string(),
    ))
}

#[cfg(target_os = "macos")]
fn quit_app(app_name: &str, bundle_id: &str) -> Result<String, AppError> {
    let script = if !bundle_id.is_empty() {
        format!("tell application id \"{bundle_id}\" to quit")
    } else {
        format!("tell application \"{app_name}\" to quit")
    };
    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map_err(|e| AppError::ExecutionError(format!("quit failed: {e}")))?;
    if status.success() {
        Ok(format!("{app_name} closed"))
    } else {
        Err(AppError::ExecutionError(format!(
            "quit {app_name} exited with status {status}"
        )))
    }
}

#[cfg(not(target_os = "macos"))]
fn quit_app(_app_name: &str, _bundle_id: &str) -> Result<String, AppError> {
    Err(AppError::PlatformNotSupported(
        "App quitting requires macOS".to_string(),
    ))
}

#[cfg(target_os = "macos")]
fn open_path(path: &str) -> Result<String, AppError> {
    let status = std::process::Command::new("open")
        .arg(path)
        .status()
        .map_err(|e| AppError::ExecutionError(format!("open path failed: {e}")))?;
    if status.success() {
        Ok(format!("Opened {path} in Finder"))
    } else {
        Err(AppError::ExecutionError(format!(
            "open {path} exited with status {status}"
        )))
    }
}

#[cfg(not(target_os = "macos"))]
fn open_path(_path: &str) -> Result<String, AppError> {
    Err(AppError::PlatformNotSupported(
        "Path opening requires macOS".to_string(),
    ))
}

fn create_folder(path: &str) -> Result<String, AppError> {
    std::fs::create_dir_all(path)
        .map_err(|e| AppError::ExecutionError(format!("create folder failed: {e}")))?;
    Ok(format!("Created folder {path}"))
}

fn move_path(source_path: &str, destination_path: &str) -> Result<String, AppError> {
    let destination_is_trash = path_policy::destination_is_home_trash(destination_path)
        .map_err(|e| AppError::ExecutionError(format!("trash boundary check failed: {e}")))?;

    if destination_is_trash {
        if let Some(parent) = std::path::Path::new(destination_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::ExecutionError(format!("prepare trash destination failed: {e}"))
            })?;
        }
    }

    std::fs::rename(source_path, destination_path)
        .map_err(|e| AppError::ExecutionError(format!("move failed: {e}")))?;
    Ok(format!("Moved {source_path} to {destination_path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn denied_permissions() -> PermissionStatus {
        PermissionStatus {
            accessibility: PermState::Denied,
            apple_events: PermState::Denied,
        }
    }

    fn granted_permissions() -> PermissionStatus {
        PermissionStatus {
            accessibility: PermState::Granted,
            apple_events: PermState::Granted,
        }
    }

    #[test]
    fn blocks_browser_tab_when_accessibility_is_denied() {
        let status = PermissionStatus {
            accessibility: PermState::Denied,
            apple_events: PermState::Granted,
        };
        let err =
            preflight_applescript_template_for_status("browser_new_tab", &status).unwrap_err();
        assert!(matches!(err, AppError::PermissionDenied(_)));
    }

    #[test]
    fn blocks_brightness_when_accessibility_is_denied() {
        let status = PermissionStatus {
            accessibility: PermState::Denied,
            apple_events: PermState::Granted,
        };
        let err = preflight_applescript_template_for_status("brightness_up", &status).unwrap_err();
        assert!(matches!(err, AppError::PermissionDenied(_)));
    }

    #[test]
    fn blocks_volume_when_automation_is_denied() {
        let status = PermissionStatus {
            accessibility: PermState::Granted,
            apple_events: PermState::Denied,
        };
        let err = preflight_applescript_template_for_status("set_volume", &status).unwrap_err();
        assert!(matches!(err, AppError::PermissionDenied(_)));
    }

    #[test]
    fn allows_templates_when_required_permissions_are_granted() {
        let status = granted_permissions();
        assert!(preflight_applescript_template_for_status("browser_close_tab", &status).is_ok());
        assert!(preflight_applescript_template_for_status("brightness_down", &status).is_ok());
        assert!(preflight_applescript_template_for_status("mute_volume", &status).is_ok());
    }

    #[test]
    fn other_templates_do_not_require_permission_preflight() {
        assert!(preflight_applescript_template_for_status(
            "future_template",
            &denied_permissions()
        )
        .is_ok());
    }
}
