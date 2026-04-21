use chrono::Utc;
use tauri::{AppHandle, Emitter};

use crate::models::{
    ApprovalStatus, ExecutionEvent, ExecutionEventKind, ExecutionOutcome, ExecutionResult,
    HistoryEntry, MachineInfo, ParsedCommand, PermissionStatus,
};
use crate::provider_keys::ProviderKeyStatus;
use crate::{executor, history, machine, parser, permissions, provider_keys, resolver, risk};
use crate::{service_catalog, AppState, APP_CONFIG_MAX_HISTORY};

// ── parse_command ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn parse_command(
    input: String,
    state: tauri::State<'_, AppState>,
) -> Result<ParsedCommand, String> {
    let normalized = parser::normalize(&input);
    let intent = parser::parse_intent(&input);

    let machine_info = {
        let inner = state.inner.lock().map_err(|_| "state lock error")?;
        inner
            .machine_info
            .clone()
            .unwrap_or_else(|| crate::models::MachineInfo {
                hostname: String::new(),
                username: String::new(),
                os_version: String::new(),
                architecture: String::new(),
                installed_browsers: vec![],
                installed_apps: vec![],
                home_dir: String::new(),
            })
    };

    let (kind, routes, unresolved_code, unresolved_message) = resolver::resolve(&intent, &machine_info);

    let cmd = ParsedCommand {
        id: uuid::Uuid::new_v4().to_string(),
        raw_input: input,
        normalized,
        kind,
        routes,
        risk: crate::models::RiskLevel::R0,
        requires_approval: false,
        approval_status: ApprovalStatus::NotRequired,
        unresolved_code,
        unresolved_message,
    };

    let cmd = risk::annotate(cmd);

    // Store in pending map.
    {
        let mut inner = state.inner.lock().map_err(|_| "state lock error")?;
        inner.pending_commands.insert(cmd.id.clone(), cmd.clone());
    }

    Ok(cmd)
}

// ── execute_command ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn execute_command(
    command_id: String,
    route_index: usize,
    state: tauri::State<'_, AppState>,
    app: AppHandle,
) -> Result<ExecutionResult, String> {
    // Look up the pending command.
    let command = {
        let inner = state.inner.lock().map_err(|_| "state lock error")?;
        inner
            .pending_commands
            .get(&command_id)
            .cloned()
            .ok_or_else(|| format!("command '{command_id}' not found in pending map"))?
    };

    // Verify approval — must be approved or not required.
    match &command.approval_status {
        ApprovalStatus::Approved | ApprovalStatus::NotRequired => {}
        ApprovalStatus::Pending => {
            return Err("command requires approval before execution".to_string())
        }
        ApprovalStatus::Denied => return Err("command was denied".to_string()),
    }

    let result = executor::execute(&command, route_index, &app).map_err(|e| e.to_string())?;

    // Build history entry.
    let inverse = result.inverse_action.clone();
    let entry = HistoryEntry {
        command: command.clone(),
        outcome: result.outcome.clone(),
        execution_events: vec![],
        duration_ms: result.duration_ms,
        inverse_action: inverse,
        timestamp: Utc::now().to_rfc3339(),
    };

    // Persist to history.
    {
        let mut inner = state.inner.lock().map_err(|_| "state lock error")?;
        inner.pending_commands.remove(&command_id);
        let _ = history::append_and_save(&mut inner.history, entry, APP_CONFIG_MAX_HISTORY);
    }

    Ok(result)
}

// ── approve_command ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn approve_command(
    command_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<ParsedCommand, String> {
    let mut inner = state.inner.lock().map_err(|_| "state lock error")?;
    let cmd = inner
        .pending_commands
        .get_mut(&command_id)
        .ok_or_else(|| format!("command '{command_id}' not found"))?;
    cmd.approval_status = ApprovalStatus::Approved;
    Ok(cmd.clone())
}

// ── deny_command ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn deny_command(
    command_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|_| "state lock error")?;
    if let Some(cmd) = inner.pending_commands.get_mut(&command_id) {
        cmd.approval_status = ApprovalStatus::Denied;
        inner.pending_commands.remove(&command_id);
    }
    Ok(())
}

// ── get_machine_info ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_machine_info(state: tauri::State<'_, AppState>) -> Result<MachineInfo, String> {
    let inner = state.inner.lock().map_err(|_| "state lock error")?;
    inner
        .machine_info
        .clone()
        .ok_or_else(|| "machine info not yet scanned".to_string())
}

// ── get_permission_status ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_permission_status() -> Result<PermissionStatus, String> {
    Ok(permissions::get_permission_status())
}

// ── get_history ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_history(state: tauri::State<'_, AppState>) -> Result<Vec<HistoryEntry>, String> {
    let inner = state.inner.lock().map_err(|_| "state lock error")?;
    Ok(inner.history.clone())
}

// ── get_service_catalog ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_service_catalog() -> Result<Vec<service_catalog::ServiceDefinition>, String> {
    Ok(service_catalog::all_services().to_vec())
}

// ── undo_last ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn undo_last(
    state: tauri::State<'_, AppState>,
    app: AppHandle,
) -> Result<ExecutionResult, String> {
    let inverse = {
        let inner = state.inner.lock().map_err(|_| "state lock error")?;
        inner
            .history
            .last()
            .and_then(|e| e.inverse_action.clone())
            .ok_or_else(|| "nothing to undo".to_string())?
    };

    // Validate the inverse action before running it.
    crate::validator::validate_action(&inverse).map_err(|e| e.to_string())?;

    // Build a synthetic command for the inverse execution.
    let undo_cmd = ParsedCommand {
        id: uuid::Uuid::new_v4().to_string(),
        raw_input: "undo".to_string(),
        normalized: "undo".to_string(),
        kind: crate::models::CommandKind::LocalSystem,
        routes: vec![crate::models::ResolvedRoute {
            label: "Undo".to_string(),
            description: "Reverse last action".to_string(),
            action: inverse,
        }],
        risk: crate::models::RiskLevel::R1,
        requires_approval: false,
        approval_status: ApprovalStatus::NotRequired,
        unresolved_code: None,
        unresolved_message: None,
    };

    let result = executor::execute(&undo_cmd, 0, &app).map_err(|e| e.to_string())?;

    if result.outcome == ExecutionOutcome::Success {
        let mut inner = state.inner.lock().map_err(|_| "state lock error")?;
        if let Some(last) = inner.history.last_mut() {
            last.inverse_action = None;
        }
        history::save_history(&inner.history).map_err(|e| e.to_string())?;
    }

    Ok(result)
}

// ── set_window_mode ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn set_window_mode(mode: String, window: tauri::WebviewWindow) -> Result<(), String> {
    match mode.as_str() {
        "lounge" => {
            window
                .set_size(tauri::LogicalSize::new(760.0_f64, 60.0_f64))
                .map_err(|e| e.to_string())?;
        }
        "expanded" => {
            window
                .set_size(tauri::LogicalSize::new(760.0_f64, 420.0_f64))
                .map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("unknown window mode: {mode}")),
    }
    Ok(())
}

// ── toggle_always_on_top ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn toggle_always_on_top(
    enabled: bool,
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    window
        .set_always_on_top(enabled)
        .map_err(|e| e.to_string())?;

    // Persist preference so it survives restarts.
    let mut config = crate::config::load_config();
    config.always_on_top = enabled;
    let _ = crate::config::save_config(&config);

    Ok(())
}

// ── refresh_machine_info ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn refresh_machine_info(
    state: tauri::State<'_, AppState>,
) -> Result<MachineInfo, String> {
    let info = machine::scan_machine();
    let mut inner = state.inner.lock().map_err(|_| "state lock error")?;
    inner.machine_info = Some(info.clone());
    Ok(info)
}

// ── emit_test_event (dev only) ─────────────────────────────────────────────────

#[cfg(debug_assertions)]
#[tauri::command]
pub async fn emit_test_event(app: AppHandle) -> Result<(), String> {
    use crate::events::{ExecutionEventPayload, EXECUTION_EVENT_NAME};
    let event = ExecutionEvent {
        id: uuid::Uuid::new_v4().to_string(),
        command_id: "test".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        kind: ExecutionEventKind::Progress,
        message: "Test event from Rust".to_string(),
    };
    app.emit(EXECUTION_EVENT_NAME, ExecutionEventPayload { event })
        .map_err(|e| e.to_string())
}

// ── Provider key commands ─────────────────────────────────────────────────────
// These commands manage AI provider credentials stored in the system keychain.
// The raw key value is NEVER returned to the frontend; only masked status is.

/// Return the masked status for a provider key (e.g. "openai").
#[tauri::command]
pub async fn get_provider_key_status(provider: String) -> Result<ProviderKeyStatus, String> {
    Ok(provider_keys::key_status(&provider))
}

/// Store a provider key in the system keychain.
/// The key must be supplied by the user via a secure input field and discarded
/// from frontend state immediately after this call returns.
#[tauri::command]
pub async fn set_provider_key(provider: String, key: String) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("Key must not be empty".to_string());
    }
    provider_keys::store_key(&provider, &key).map_err(|e| e.to_string())
}

/// Delete a provider key from the system keychain.
#[tauri::command]
pub async fn delete_provider_key(provider: String) -> Result<(), String> {
    provider_keys::delete_key(&provider).map_err(|e| e.to_string())
}
