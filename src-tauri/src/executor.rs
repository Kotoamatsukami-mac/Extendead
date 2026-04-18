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
            let msg = e.to_string();
            emit(
                app,
                new_event(command_id, ExecutionEventKind::Failed, msg.clone()),
            );
            Ok(ExecutionResult {
                command_id: command_id.to_string(),
                outcome: ExecutionOutcome::RecoverableFailure,
                message: msg.clone(),
                human_message: format!("✗ {msg}"),
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
            open_url(url, browser_bundle)
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
        ResolvedAction::AppleScriptTemplate { script, .. } => {
            emit(
                app,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    "Running AppleScript template".to_string(),
                ),
            );
            applescript::run_validated_script(script)
        }
        ResolvedAction::OpenSystemPreferences { pane_url } => {
            emit(
                app,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Opening system preferences: {pane_url}"),
                ),
            );
            open_url(pane_url, "")
        }
        ResolvedAction::OpenPath { path } => {
            emit(
                app,
                new_event(
                    command_id,
                    ExecutionEventKind::Progress,
                    format!("Revealing path: {path}"),
                ),
            );
            open_path(path)
        }
    }
}

#[cfg(target_os = "macos")]
fn open_url(url: &str, browser_bundle: &str) -> Result<String, AppError> {
    let mut cmd = std::process::Command::new("open");
    if !browser_bundle.is_empty() {
        cmd.args(["-b", browser_bundle]);
    }
    cmd.arg(url);
    let status = cmd
        .status()
        .map_err(|e| AppError::ExecutionError(format!("open failed: {e}")))?;
    if status.success() {
        Ok(format!("Opened {url}"))
    } else {
        Err(AppError::ExecutionError(format!(
            "open exited with status {status}"
        )))
    }
}

#[cfg(not(target_os = "macos"))]
fn open_url(_url: &str, _browser_bundle: &str) -> Result<String, AppError> {
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
        Ok(format!("Launched {app_name}"))
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
        Ok(format!("Opened {path}"))
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
