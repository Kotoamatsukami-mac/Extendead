use chrono::Utc;
use tauri::{AppHandle, Emitter};

use crate::intent_language::{CandidateIntent, CanonicalAction};
use crate::models::{
    ApprovalStatus, ExecutionEvent, ExecutionEventKind, ExecutionOutcome, ExecutionResult,
    HistoryEntry, MachineInfo, ParsedCommand, PermissionStatus,
};
use crate::provider_keys::ProviderKeyStatus;
use crate::{
    arbiter, executor, history, interpret_local, machine, parser, permissions, provider_keys,
    resolver, risk,
};
use crate::{service_catalog, AppState, APP_CONFIG_MAX_HISTORY};

fn interpreted_intent(input: &str) -> Option<parser::Intent> {
    let candidates = interpret_local::interpret(input);
    let arbitration = arbiter::decide(&candidates);
    if arbitration.decision != arbiter::ArbitrationDecision::Execute {
        return None;
    }
    let chosen = arbitration.chosen_index?;
    let candidate = candidates.get(chosen)?;
    intent_from_candidate(candidate)
}

fn intent_from_candidate(candidate: &CandidateIntent) -> Option<parser::Intent> {
    let slot = |name: &str| {
        candidate
            .slots
            .get(name)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
    };

    match candidate.canonical_action {
        CanonicalAction::OpenApp => slot("app").map(|app| parser::Intent::OpenAppNamed(app.to_string())),
        CanonicalAction::QuitApp => {
            slot("app").map(|app| parser::Intent::CloseAppNamed(app.to_string()))
        }
        CanonicalAction::OpenPath => slot("path").map(|path| parser::Intent::OpenPath(path.to_string())),
        CanonicalAction::OpenService => {
            let service = slot("service")?;
            match slot("browser") {
                Some(browser) => Some(parser::Intent::OpenServiceInBrowser {
                    service_id: service.to_string(),
                    browser: browser.to_string(),
                }),
                None => Some(parser::Intent::OpenService(service.to_string())),
            }
        }
        CanonicalAction::CreateFolder => {
            let name = slot("name")?;
            let base = slot("base").or_else(|| slot("base_path")).map(|value| value.to_string());
            Some(parser::Intent::CreateFolder {
                name: name.to_string(),
                base,
            })
        }
        CanonicalAction::MovePath => {
            let source = slot("source")?;
            let destination = slot("destination")?;
            Some(parser::Intent::MovePath {
                source: source.to_string(),
                destination: destination.to_string(),
            })
        }
        CanonicalAction::Unknown => None,
    }
}

// ── parse_command ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn parse_command(
    input: String,
    state: tauri::State<'_, AppState>,
) -> Result<ParsedCommand, String> {
    let normalized = parser::normalize(&input);
    let intent = interpreted_intent(&input).unwrap_or_else(|| parser::parse_intent(&input));

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

    let (kind, routes, unresolved_code, unresolved_message) =
        resolver::resolve(&intent, &machine_info);

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

    let run = executor::execute(&command, route_index, &app).map_err(|e| e.to_string())?;
    let result = run.result;

    // Build history entry.
    let inverse = result.inverse_action.clone();
    let entry = HistoryEntry {
        command: command.clone(),
        outcome: result.outcome.clone(),
        execution_events: run.events,
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

// ── get_app_config ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_app_config() -> Result<crate::config::AppConfig, String> {
    Ok(crate::config::load_config())
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

    let run = executor::execute(&undo_cmd, 0, &app).map_err(|e| e.to_string())?;
    let result = run.result;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::intent_language::{ExecutorFamily, IntentFamily, InterpretationSource};
    use crate::models::RiskLevel;

    fn candidate(action: CanonicalAction, slots: &[(&str, &str)]) -> CandidateIntent {
        let slot_map = slots
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<BTreeMap<_, _>>();

        CandidateIntent {
            family: IntentFamily::Unknown,
            canonical_action: action,
            slots: slot_map,
            missing_slots: vec![],
            confidence: 0.95,
            clarification_needed: false,
            risk_baseline: RiskLevel::R0,
            executor_family: ExecutorFamily::Unknown,
            source: InterpretationSource::LocalPattern,
        }
    }

    #[test]
    fn maps_open_app_candidate_to_intent() {
        let c = candidate(CanonicalAction::OpenApp, &[("app", "Slack")]);
        assert_eq!(
            intent_from_candidate(&c),
            Some(parser::Intent::OpenAppNamed("Slack".to_string()))
        );
    }

    #[test]
    fn maps_open_service_candidate_with_browser_to_intent() {
        let c = candidate(
            CanonicalAction::OpenService,
            &[("service", "youtube"), ("browser", "safari")],
        );
        assert_eq!(
            intent_from_candidate(&c),
            Some(parser::Intent::OpenServiceInBrowser {
                service_id: "youtube".to_string(),
                browser: "safari".to_string(),
            })
        );
    }

    #[test]
    fn returns_none_when_required_slots_missing() {
        let c = candidate(CanonicalAction::MovePath, &[("source", "~/Desktop/a.txt")]);
        assert_eq!(intent_from_candidate(&c), None);
    }
}
