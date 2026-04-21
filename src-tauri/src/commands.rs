use chrono::Utc;
use tauri::{AppHandle, Emitter};

use crate::intent_language::{CandidateIntent, CanonicalAction};
use crate::models::{
    ApprovalStatus, CommandSuggestion, ExecutionEvent, ExecutionEventKind, ExecutionOutcome,
    ExecutionResult, HistoryEntry, MachineInfo, ParsedCommand, PermissionStatus,
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

fn interpretation_clarification_message(input: &str) -> Option<String> {
    let candidates = interpret_local::interpret(input);
    let arbitration = arbiter::decide(&candidates);

    match arbitration.decision {
        arbiter::ArbitrationDecision::Clarify => {
            let chosen = arbitration.chosen_index?;
            let candidate = candidates.get(chosen)?;
            if !candidate.missing_slots.is_empty() {
                return Some(format!(
                    "Need more detail: {}.",
                    candidate
                        .missing_slots
                        .iter()
                        .map(|slot| slot.replace('_', " "))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            Some(arbitration.explanation)
        }
        arbiter::ArbitrationDecision::OfferChoices => {
            Some("Multiple actions look plausible. Add a bit more detail.".to_string())
        }
        _ => None,
    }
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
        CanonicalAction::OpenApp => {
            slot("app").map(|app| parser::Intent::OpenAppNamed(app.to_string()))
        }
        CanonicalAction::QuitApp => {
            slot("app").map(|app| parser::Intent::CloseAppNamed(app.to_string()))
        }
        CanonicalAction::OpenPath => {
            slot("path").map(|path| parser::Intent::OpenPath(path.to_string()))
        }
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
            let base = slot("base")
                .or_else(|| slot("base_path"))
                .map(|value| value.to_string());
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
        CanonicalAction::BrowserNewTab => Some(parser::Intent::BrowserNewTab {
            browser: slot("browser").map(|browser| browser.to_string()),
        }),
        CanonicalAction::BrowserCloseTab => Some(parser::Intent::BrowserCloseTab {
            browser: slot("browser").map(|browser| browser.to_string()),
        }),
        CanonicalAction::BrowserReopenClosedTab => Some(parser::Intent::BrowserReopenClosedTab {
            browser: slot("browser").map(|browser| browser.to_string()),
        }),
        CanonicalAction::BrightnessUp => Some(parser::Intent::IncreaseBrightness),
        CanonicalAction::BrightnessDown => Some(parser::Intent::DecreaseBrightness),
        CanonicalAction::TrashPath => {
            slot("path").map(|path| parser::Intent::TrashPath(path.to_string()))
        }
        CanonicalAction::Unknown => None,
    }
}

fn machine_snapshot(state: &tauri::State<'_, AppState>) -> Result<MachineInfo, String> {
    let inner = state.inner.lock().map_err(|_| "state lock error")?;
    Ok(inner
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
        }))
}

fn intent_key(intent: &parser::Intent) -> String {
    match intent {
        parser::Intent::OpenService(service_id) => format!("open_service:{service_id}"),
        parser::Intent::OpenServiceInBrowser {
            service_id,
            browser,
        } => {
            format!(
                "open_service_in_browser:{service_id}:{}",
                parser::normalize(browser)
            )
        }
        parser::Intent::OpenAppNamed(app) => format!("open_app:{}", parser::normalize(app)),
        parser::Intent::CloseAppNamed(app) => format!("close_app:{}", parser::normalize(app)),
        parser::Intent::OpenPath(path) => format!("open_path:{}", parser::normalize(path)),
        parser::Intent::CreateFolder { name, base } => {
            format!(
                "create_folder:{}:{}",
                parser::normalize(name),
                base.as_ref()
                    .map(|b| parser::normalize(b))
                    .unwrap_or_else(|| "home".to_string())
            )
        }
        parser::Intent::MovePath {
            source,
            destination,
        } => {
            format!(
                "move_path:{}:{}",
                parser::normalize(source),
                parser::normalize(destination)
            )
        }
        parser::Intent::BrowserNewTab { browser } => format!(
            "browser_new_tab:{}",
            browser
                .as_ref()
                .map(|b| parser::normalize(b))
                .unwrap_or_else(|| "default".to_string())
        ),
        parser::Intent::BrowserCloseTab { browser } => format!(
            "browser_close_tab:{}",
            browser
                .as_ref()
                .map(|b| parser::normalize(b))
                .unwrap_or_else(|| "default".to_string())
        ),
        parser::Intent::BrowserReopenClosedTab { browser } => format!(
            "browser_reopen_closed_tab:{}",
            browser
                .as_ref()
                .map(|b| parser::normalize(b))
                .unwrap_or_else(|| "default".to_string())
        ),
        parser::Intent::TrashPath(path) => format!("trash_path:{}", parser::normalize(path)),
        parser::Intent::DeletePathPermanently(path) => {
            format!("delete_permanent:{}", parser::normalize(path))
        }
        parser::Intent::OpenSafari => "open_safari".to_string(),
        parser::Intent::OpenChrome => "open_chrome".to_string(),
        parser::Intent::OpenFirefox => "open_firefox".to_string(),
        parser::Intent::OpenBrave => "open_brave".to_string(),
        parser::Intent::OpenArc => "open_arc".to_string(),
        parser::Intent::OpenFinder => "open_finder".to_string(),
        parser::Intent::OpenSlack => "open_slack".to_string(),
        parser::Intent::MuteVolume => "mute_volume".to_string(),
        parser::Intent::SetVolume(level) => format!("set_volume:{level}"),
        parser::Intent::OpenDisplaySettings => "open_display_settings".to_string(),
        parser::Intent::RevealDownloads => "reveal_downloads".to_string(),
        parser::Intent::IncreaseBrightness => "brightness_up".to_string(),
        parser::Intent::DecreaseBrightness => "brightness_down".to_string(),
        parser::Intent::Unknown(raw) => format!("unknown:{}", parser::normalize(raw)),
    }
}

fn intent_family_label(intent: &parser::Intent) -> &'static str {
    match intent {
        parser::Intent::OpenService(_) | parser::Intent::OpenServiceInBrowser { .. } => {
            "open service"
        }
        parser::Intent::OpenAppNamed(_)
        | parser::Intent::OpenSafari
        | parser::Intent::OpenChrome
        | parser::Intent::OpenFirefox
        | parser::Intent::OpenBrave
        | parser::Intent::OpenArc
        | parser::Intent::OpenFinder
        | parser::Intent::OpenSlack => "open app",
        parser::Intent::CloseAppNamed(_) => "close app",
        parser::Intent::BrowserNewTab { .. }
        | parser::Intent::BrowserCloseTab { .. }
        | parser::Intent::BrowserReopenClosedTab { .. } => "browser tab",
        parser::Intent::OpenPath(_) | parser::Intent::RevealDownloads => "open path",
        parser::Intent::CreateFolder { .. } => "create folder",
        parser::Intent::MovePath { .. } => "move path",
        parser::Intent::TrashPath(_) => "trash path",
        parser::Intent::DeletePathPermanently(_) => "delete path",
        parser::Intent::MuteVolume | parser::Intent::SetVolume(_) => "sound",
        parser::Intent::IncreaseBrightness | parser::Intent::DecreaseBrightness => "brightness",
        parser::Intent::OpenDisplaySettings => "settings",
        parser::Intent::Unknown(_) => "unknown",
    }
}

fn intent_canonical(intent: &parser::Intent) -> String {
    match intent {
        parser::Intent::OpenService(service_id) => service_catalog::service_by_id(service_id)
            .map(|s| format!("open {}", s.display_name.to_lowercase()))
            .unwrap_or_else(|| format!("open {service_id}")),
        parser::Intent::OpenServiceInBrowser {
            service_id,
            browser,
        } => {
            let service = service_catalog::service_by_id(service_id)
                .map(|s| s.display_name.to_lowercase())
                .unwrap_or_else(|| service_id.to_string());
            format!("open {service} in {}", parser::normalize(browser))
        }
        parser::Intent::OpenAppNamed(app) => format!("open {}", parser::normalize(app)),
        parser::Intent::CloseAppNamed(app) => format!("close {}", parser::normalize(app)),
        parser::Intent::OpenPath(path) => format!("open {path}"),
        parser::Intent::CreateFolder { name, base } => match base {
            Some(base) => format!("create folder called {name} in {base}"),
            None => format!("create folder called {name} in home"),
        },
        parser::Intent::MovePath {
            source,
            destination,
        } => {
            format!("move {source} to {destination}")
        }
        parser::Intent::TrashPath(path) => format!("trash {path}"),
        parser::Intent::DeletePathPermanently(path) => format!("delete permanently {path}"),
        parser::Intent::BrowserNewTab { browser } => match browser {
            Some(browser) => format!("open new tab in {}", parser::normalize(browser)),
            None => "open new tab".to_string(),
        },
        parser::Intent::BrowserCloseTab { browser } => match browser {
            Some(browser) => format!("close tab in {}", parser::normalize(browser)),
            None => "close tab".to_string(),
        },
        parser::Intent::BrowserReopenClosedTab { browser } => match browser {
            Some(browser) => format!("reopen closed tab in {}", parser::normalize(browser)),
            None => "reopen closed tab".to_string(),
        },
        parser::Intent::IncreaseBrightness => "increase brightness".to_string(),
        parser::Intent::DecreaseBrightness => "decrease brightness".to_string(),
        parser::Intent::OpenSafari => "open safari".to_string(),
        parser::Intent::OpenChrome => "open chrome".to_string(),
        parser::Intent::OpenFirefox => "open firefox".to_string(),
        parser::Intent::OpenBrave => "open brave".to_string(),
        parser::Intent::OpenArc => "open arc".to_string(),
        parser::Intent::OpenFinder => "open finder".to_string(),
        parser::Intent::OpenSlack => "open slack".to_string(),
        parser::Intent::MuteVolume => "mute".to_string(),
        parser::Intent::SetVolume(level) => format!("set volume to {level}"),
        parser::Intent::OpenDisplaySettings => "display settings".to_string(),
        parser::Intent::RevealDownloads => "downloads".to_string(),
        parser::Intent::Unknown(raw) => raw.to_string(),
    }
}

fn push_intent_if_new(intents: &mut Vec<parser::Intent>, intent: parser::Intent) {
    if matches!(intent, parser::Intent::Unknown(_)) {
        return;
    }
    let key = intent_key(&intent);
    if intents.iter().any(|existing| intent_key(existing) == key) {
        return;
    }
    intents.push(intent);
}

fn path_like(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("~/")
        || trimmed.starts_with('/')
        || trimmed.contains('/')
        || matches!(
            trimmed.to_lowercase().as_str(),
            "desktop" | "downloads" | "documents" | "home"
        )
}

fn suggestion_intents(input: &str, machine: &MachineInfo) -> Vec<parser::Intent> {
    let mut intents = Vec::new();
    let normalized = parser::normalize(input);
    if normalized.len() < 2 {
        return intents;
    }

    if let Some(intent) = interpreted_intent(input) {
        push_intent_if_new(&mut intents, intent);
    }

    push_intent_if_new(&mut intents, parser::parse_intent(input));

    if let Some(query) = normalized
        .strip_prefix("open ")
        .or_else(|| normalized.strip_prefix("launch "))
        .or_else(|| normalized.strip_prefix("start "))
        .or_else(|| normalized.strip_prefix("run "))
    {
        let query = query.trim();
        if !query.is_empty() && !path_like(query) {
            for app in &machine.installed_apps {
                if parser::normalize(&app.name).contains(query) {
                    push_intent_if_new(
                        &mut intents,
                        parser::Intent::OpenAppNamed(app.name.clone()),
                    );
                }
            }
            for browser in &machine.installed_browsers {
                if parser::normalize(&browser.name).contains(query) {
                    push_intent_if_new(
                        &mut intents,
                        parser::Intent::OpenAppNamed(browser.name.clone()),
                    );
                }
            }
        }
    }

    if let Some(query) = normalized
        .strip_prefix("close ")
        .or_else(|| normalized.strip_prefix("quit "))
        .or_else(|| normalized.strip_prefix("exit "))
    {
        let query = query.trim();
        if !query.is_empty() {
            for app in &machine.installed_apps {
                if parser::normalize(&app.name).contains(query) {
                    push_intent_if_new(
                        &mut intents,
                        parser::Intent::CloseAppNamed(app.name.clone()),
                    );
                }
            }
            for browser in &machine.installed_browsers {
                if parser::normalize(&browser.name).contains(query) {
                    push_intent_if_new(
                        &mut intents,
                        parser::Intent::CloseAppNamed(browser.name.clone()),
                    );
                }
            }
        }
    }

    if let Some(query) = normalized
        .strip_prefix("open ")
        .or_else(|| normalized.strip_prefix("watch "))
        .or_else(|| normalized.strip_prefix("browse "))
        .or_else(|| normalized.strip_prefix("visit "))
        .or_else(|| normalized.strip_prefix("go to "))
    {
        let query = query.split(" in ").next().unwrap_or("").trim();
        if !query.is_empty() {
            for service in service_catalog::search_services(query, 4) {
                push_intent_if_new(
                    &mut intents,
                    parser::Intent::OpenService(service.id.to_string()),
                );
            }
        }
    }

    intents
}

// ── suggest_commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn suggest_commands(
    input: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<CommandSuggestion>, String> {
    let machine_info = machine_snapshot(&state)?;
    let intents = suggestion_intents(&input, &machine_info);
    let mut suggestions = Vec::new();

    for intent in intents {
        let (kind, routes, unresolved_code, _unresolved_message) =
            resolver::resolve(&intent, &machine_info);
        if unresolved_code.is_some() || routes.is_empty() {
            continue;
        }

        let valid_routes: Vec<_> = routes
            .into_iter()
            .filter(|route| crate::validator::validate_action(&route.action).is_ok())
            .collect();
        if valid_routes.is_empty() {
            continue;
        }

        let annotated = risk::annotate(ParsedCommand {
            id: "suggestion".to_string(),
            raw_input: intent_canonical(&intent),
            normalized: parser::normalize(&input),
            kind,
            routes: valid_routes.clone(),
            risk: crate::models::RiskLevel::R0,
            requires_approval: false,
            approval_status: ApprovalStatus::NotRequired,
            unresolved_code: None,
            unresolved_message: None,
        });

        let detail = if annotated.requires_approval {
            format!("{} (requires approval)", valid_routes[0].description)
        } else {
            valid_routes[0].description.clone()
        };
        let canonical = intent_canonical(&intent);
        let id = format!(
            "{}-{}",
            intent_family_label(&intent).replace(' ', "-"),
            canonical
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .to_lowercase()
        );
        suggestions.push(CommandSuggestion {
            id,
            family: intent_family_label(&intent).to_string(),
            canonical,
            detail,
        });

        if suggestions.len() >= 4 {
            break;
        }
    }

    Ok(suggestions)
}

// ── parse_command ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn parse_command(
    input: String,
    state: tauri::State<'_, AppState>,
) -> Result<ParsedCommand, String> {
    let normalized = parser::normalize(&input);
    let parsed_intent = parser::parse_intent(&input);
    let intent = interpreted_intent(&input).unwrap_or_else(|| parsed_intent.clone());

    let machine_info = machine_snapshot(&state)?;

    let (kind, routes, unresolved_code, unresolved_message) =
        resolver::resolve(&intent, &machine_info);

    let mut cmd = ParsedCommand {
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

    if cmd.routes.is_empty() && matches!(parsed_intent, parser::Intent::Unknown(_)) {
        if let Some(clarify) = interpretation_clarification_message(&cmd.raw_input) {
            cmd.unresolved_message = Some(clarify);
        }
    }

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

    #[test]
    fn maps_browser_new_tab_candidate_with_browser_slot() {
        let c = candidate(CanonicalAction::BrowserNewTab, &[("browser", "safari")]);
        assert_eq!(
            intent_from_candidate(&c),
            Some(parser::Intent::BrowserNewTab {
                browser: Some("safari".to_string()),
            })
        );
    }

    #[test]
    fn maps_trash_path_candidate() {
        let c = candidate(
            CanonicalAction::TrashPath,
            &[("path", "~/Desktop/test.txt")],
        );
        assert_eq!(
            intent_from_candidate(&c),
            Some(parser::Intent::TrashPath("~/Desktop/test.txt".to_string()))
        );
    }
}
