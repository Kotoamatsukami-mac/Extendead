use crate::models::{
    ApprovalStatus, InterpretationPreview, MachineInfo, ParsedCommand, PermState, PreviewStatus,
    PreviewToken, PreviewTokenKind, ResolvedAction, RiskLevel, UnresolvedCode,
};
use crate::{native_lexicon, parser, permissions, resolver, risk, validator, AppState};

#[tauri::command]
pub async fn interpret_preview(
    input: String,
    state: tauri::State<'_, AppState>,
) -> Result<InterpretationPreview, String> {
    let normalized = parser::normalize(&input);
    if normalized.trim().is_empty() {
        return Ok(InterpretationPreview {
            status: PreviewStatus::Empty,
            normalized,
            canonical: Some("COMMAND YOUR MAC IN ONE SENTENCE".to_string()),
            tokens: vec![],
            headline: "COMMAND YOUR MAC IN ONE SENTENCE".to_string(),
            detail: Some("Try open Safari or create folder called work in Desktop.".to_string()),
            suggestion: None,
            choices: vec![],
            risk: None,
            can_submit: false,
        });
    }

    let machine = machine_snapshot(&state)?;
    let lexicon = native_lexicon::build(&machine);
    let intent = parser::parse_intent(&input);
    let (kind, routes, unresolved_code, unresolved_message) = resolver::resolve(&intent, &machine);
    let mut command = ParsedCommand {
        id: "preview".to_string(),
        raw_input: input.clone(),
        normalized: normalized.clone(),
        kind,
        routes,
        risk: RiskLevel::R0,
        requires_approval: false,
        approval_status: ApprovalStatus::NotRequired,
        unresolved_code,
        unresolved_message,
        interpretation_decision: None,
        clarification_message: None,
        clarification_slots: vec![],
        choices: vec![],
    };

    command = risk::annotate(command);
    let tokens = tokenize_preview(&input, &lexicon);

    if command.routes.is_empty() {
        return Ok(unresolved_preview(normalized, tokens, &command));
    }

    let valid_routes = command
        .routes
        .iter()
        .filter(|route| validator::validate_action(&route.action).is_ok())
        .count();

    if valid_routes == 0 {
        return Ok(InterpretationPreview {
            status: PreviewStatus::Blocked,
            normalized,
            canonical: None,
            tokens,
            headline: "Blocked".to_string(),
            detail: Some("The route resolved but failed local validation.".to_string()),
            suggestion: None,
            choices: vec![],
            risk: Some(command.risk),
            can_submit: false,
        });
    }

    if let Some(permission_message) = permission_needed(&command) {
        return Ok(InterpretationPreview {
            status: PreviewStatus::PermissionNeeded,
            normalized,
            canonical: command.routes.first().map(|route| route.label.clone()),
            tokens,
            headline: "Permission needed".to_string(),
            detail: Some(permission_message),
            suggestion: None,
            choices: vec![],
            risk: Some(command.risk),
            can_submit: false,
        });
    }

    if command.routes.len() > 1 {
        return Ok(InterpretationPreview {
            status: PreviewStatus::ChooseOne,
            normalized,
            canonical: None,
            tokens,
            headline: "Choose one".to_string(),
            detail: Some("Multiple valid routes match this command.".to_string()),
            suggestion: None,
            choices: command
                .routes
                .iter()
                .take(4)
                .map(|route| route.label.clone())
                .collect(),
            risk: Some(command.risk),
            can_submit: true,
        });
    }

    if command.requires_approval {
        return Ok(InterpretationPreview {
            status: PreviewStatus::ApprovalNeeded,
            normalized,
            canonical: command.routes.first().map(|route| route.label.clone()),
            tokens,
            headline: "Approval needed".to_string(),
            detail: command.routes.first().map(|route| route.description.clone()),
            suggestion: None,
            choices: vec![],
            risk: Some(command.risk),
            can_submit: true,
        });
    }

    Ok(InterpretationPreview {
        status: PreviewStatus::Valid,
        normalized,
        canonical: command.routes.first().map(|route| route.label.clone()),
        tokens,
        headline: "Ready".to_string(),
        detail: command.routes.first().map(|route| route.description.clone()),
        suggestion: None,
        choices: vec![],
        risk: Some(command.risk),
        can_submit: true,
    })
}

fn machine_snapshot(state: &tauri::State<'_, AppState>) -> Result<MachineInfo, String> {
    let current = {
        let inner = state.inner.lock().map_err(|_| "state lock error")?;
        inner.machine_info.clone()
    };

    let needs_scan = current
        .as_ref()
        .map(crate::machine::app_cache_is_stale)
        .unwrap_or(true);
    if needs_scan {
        let info = crate::machine::scan_machine();
        let mut inner = state.inner.lock().map_err(|_| "state lock error")?;
        inner.machine_info = Some(info.clone());
        return Ok(info);
    }

    current.ok_or_else(|| "machine info not yet scanned".to_string())
}

fn unresolved_preview(
    normalized: String,
    tokens: Vec<PreviewToken>,
    command: &ParsedCommand,
) -> InterpretationPreview {
    let status = match command.unresolved_code {
        Some(UnresolvedCode::AmbiguousTarget) => PreviewStatus::ChooseOne,
        Some(UnresolvedCode::PermanentDeleteBlocked) => PreviewStatus::Blocked,
        Some(UnresolvedCode::ProviderConfigurationRequired) => PreviewStatus::UnsupportedYet,
        Some(UnresolvedCode::PathNotFound)
        | Some(UnresolvedCode::SourcePathNotFound)
        | Some(UnresolvedCode::BasePathUnresolved)
        | Some(UnresolvedCode::DestinationPathUnresolved)
        | Some(UnresolvedCode::DestinationParentMissing)
        | Some(UnresolvedCode::TargetAlreadyExists) => PreviewStatus::NeedsMore,
        _ => PreviewStatus::UnsupportedYet,
    };

    let headline = match &status {
        PreviewStatus::ChooseOne => "Choose one",
        PreviewStatus::Blocked => "Blocked",
        PreviewStatus::NeedsMore => "Needs more",
        _ => "Unsupported yet",
    }
    .to_string();

    InterpretationPreview {
        status,
        normalized,
        canonical: None,
        tokens,
        headline,
        detail: Some(
            command
                .unresolved_message
                .clone()
                .unwrap_or_else(|| "That command is outside current local coverage.".to_string()),
        ),
        suggestion: suggestion_for_unresolved(command),
        choices: command.choices.clone(),
        risk: Some(command.risk.clone()),
        can_submit: false,
    }
}

fn suggestion_for_unresolved(command: &ParsedCommand) -> Option<String> {
    match command.unresolved_code {
        Some(UnresolvedCode::PermanentDeleteBlocked) => Some("Use trash with a safe home path instead.".to_string()),
        Some(UnresolvedCode::ProviderConfigurationRequired) => {
            Some("Try a supported local command like open Safari or downloads.".to_string())
        }
        Some(UnresolvedCode::UnsupportedCommand) | None => {
            Some("Try open app create folder move file trash file volume or mode.".to_string())
        }
        _ => None,
    }
}

fn permission_needed(command: &ParsedCommand) -> Option<String> {
    let status = permissions::get_permission_status();
    let needs_automation = command.routes.iter().any(|route| {
        matches!(
            &route.action,
            ResolvedAction::AppleScriptTemplate { .. }
                | ResolvedAction::QuitApp { .. }
                | ResolvedAction::HideApp { .. }
                | ResolvedAction::ForceQuitApp { .. }
        )
    });

    if needs_automation && matches!(status.accessibility, PermState::Denied) {
        return Some("Accessibility permission is required for this Mac action.".to_string());
    }
    if needs_automation && matches!(status.apple_events, PermState::Denied) {
        return Some("Apple Events permission is required for this Mac action.".to_string());
    }
    None
}

fn tokenize_preview(input: &str, lexicon: &native_lexicon::NativeLexicon) -> Vec<PreviewToken> {
    input
        .split_whitespace()
        .map(|word| {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '~');
            let kind = classify_token(clean, lexicon);
            PreviewToken {
                text: word.to_string(),
                kind,
                resolved: None,
                confidence: if clean.len() <= 1 { 0.55 } else { 0.82 },
            }
        })
        .collect()
}

fn classify_token(word: &str, lexicon: &native_lexicon::NativeLexicon) -> PreviewTokenKind {
    let normalized = parser::normalize(word);
    if normalized.is_empty() {
        return PreviewTokenKind::Unknown;
    }
    if matches!(
        normalized.as_str(),
        "open" | "close" | "hide" | "quit" | "make" | "create" | "move" | "run" | "set" | "mute"
    ) {
        return PreviewTokenKind::Verb;
    }
    if matches!(normalized.as_str(), "delete" | "force" | "kill" | "trash") {
        return PreviewTokenKind::Risk;
    }
    if matches!(normalized.as_str(), "in" | "to" | "into" | "called" | "named") {
        return PreviewTokenKind::Connector;
    }
    if normalized.ends_with("mode") || matches!(normalized.as_str(), "study" | "focus" | "break") {
        return PreviewTokenKind::Mode;
    }
    if word.starts_with("~/") || word.starts_with('/') || word.contains('/') {
        return PreviewTokenKind::Path;
    }
    if native_lexicon::contains_word(lexicon, &normalized, "service") {
        return PreviewTokenKind::Service;
    }
    if native_lexicon::contains_word(lexicon, &normalized, "browser") {
        return PreviewTokenKind::Browser;
    }
    if native_lexicon::contains_word(lexicon, &normalized, "folder") {
        return PreviewTokenKind::Path;
    }
    if native_lexicon::contains_word(lexicon, &normalized, "app") {
        return PreviewTokenKind::Target;
    }
    PreviewTokenKind::Unknown
}
