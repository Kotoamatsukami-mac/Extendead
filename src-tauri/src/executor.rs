use std::time::Instant;

use tauri::{AppHandle, Emitter, Runtime};

use crate::applescript;
use crate::errors::AppError;
use crate::events::{ExecutionEventPayload, EXECUTION_EVENT_NAME};
use crate::models::{
    ExecutionEvent, ExecutionEventKind, ExecutionOutcome, ExecutionResult, ParsedCommand,
    ResolvedAction,
};
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

fn emit<R: Runtime>(app: &AppHandle<R>, event: ExecutionEvent) {
    let _ = app.emit(EXECUTION_EVENT_NAME, ExecutionEventPayload { event });
}

/// Map an AppError to the appropriate ExecutionOutcome.
/// PermissionDenied must become Blocked — never silently downgraded.
fn outcome_for_error(e: &AppError) -> ExecutionOutcome {
    match e {
        AppError::PermissionDenied(_) => ExecutionOutcome::Blocked,
        AppError::ValidationError(_) | AppError::ShellPolicyViolation(_) => {
            ExecutionOutcome::Blocked
        }
        _ => ExecutionOutcome::RecoverableFailure,
    }
}

/// Build a concise, operator-readable failure message.
fn human_message_for_error(e: &AppError) -> String {
    match e {
        AppError::PermissionDenied(detail) => {
            format!("✗ Permission required — {detail}")
        }
        AppError::ValidationError(detail) => format!("✗ Blocked: {detail}"),
        _ => format!("✗ {e}"),
    }
}

/// Execute a specific route on an approved command.
pub fn execute<R: Runtime>(
    command: &ParsedCommand,
    route_index: usize,
    app: &AppHandle<R>,
) -> Result<ExecutionResult, AppError> {
    let start = Instant::now();

    // Validate before execution — never skip this.
    validator::validate(command, route_index)?;

    let route = command.routes.get(route_index).expect("already validated");
    let command_id = command.id.as_str();

    // For set_volume, capture the current volume before executing so the
    // inverse action can restore it precisely rather than keeping it None.
    let pre_volume: Option<u8> = match &route.action {
        ResolvedAction::AppleScriptTemplate { template_id, .. } if template_id == "set_volume" => {
            applescript::get_volume()
        }
        _ => None,
    };

    emit(
        app,
        new_event(
            command_id,
            ExecutionEventKind::Started,
            format!("Executing: {}", route.label),
        ),
    );

    let result = dispatch_action(command_id, &route.action, app);

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(message) => {
            // Build inverse: for set_volume, use the captured pre-volume level.
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
                new_event(command_id, ExecutionEventKind::Completed, message.clone()),
            );
            Ok(ExecutionResult {
                command_id: command_id.to_string(),
                outcome: ExecutionOutcome::Success,
                message,
                human_message: format!("✓ {}", route.label),
                duration_ms,
                inverse_action: inverse,
            })
        }
        Err(e) => {
            let outcome = outcome_for_error(&e);
            let human_message = human_message_for_error(&e);
            let msg = e.to_string();
            emit(
                app,
                new_event(command_id, ExecutionEventKind::Failed, msg.clone()),
            );
            Ok(ExecutionResult {
                command_id: command_id.to_string(),
                outcome,
                message: msg,
                human_message,
                duration_ms,
                inverse_action: None,
            })
        }
    }
}

fn dispatch_action<R: Runtime>(
    command_id: &str,
    action: &ResolvedAction,
    app: &AppHandle<R>,
) -> Result<String, AppError> {
    match action {
        ResolvedAction::OpenUrl {
            url,
            browser_bundle,
            browser_name,
        } => {
            emit(
                app,
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
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Launching {app_name}"),
                ),
            );
            open_app(app_name, bundle_id)
        }
        ResolvedAction::AppleScriptTemplate {
            script,
            template_id,
        } => {
            let progress_msg = match template_id.as_str() {
                "mute_volume" => "Muting system audio output".to_string(),
                "unmute_volume" => "Unmuting system audio output".to_string(),
                "set_volume" => "Setting output volume".to_string(),
                _ => "Running AppleScript".to_string(),
            };
            emit(
                app,
                new_event(command_id, ExecutionEventKind::Progress, progress_msg),
            );
            applescript::run_validated_script(script).map(|_| match template_id.as_str() {
                "mute_volume" => "System audio muted".to_string(),
                "unmute_volume" => "System audio unmuted".to_string(),
                "set_volume" => "Output volume set".to_string(),
                _ => "Done".to_string(),
            })
        }
        ResolvedAction::OpenSystemPreferences { pane_url } => {
            emit(
                app,
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
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Revealing {path} in Finder"),
                ),
            );
            open_path(path)
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
    // Prefer bundle ID (-b flag) for reliability; fall back to app name (-a flag).
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
