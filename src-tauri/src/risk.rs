use crate::models::{ApprovalStatus, CommandKind, ParsedCommand, ResolvedAction, RiskLevel};

/// Assign a risk level to a parsed command based on its kind and resolved actions.
pub fn score(kind: &CommandKind, routes: &[crate::models::ResolvedRoute]) -> RiskLevel {
    // Use the highest risk level across all available routes.
    routes
        .iter()
        .map(|r| action_risk(&r.action, kind))
        .max()
        .unwrap_or(RiskLevel::R0)
}

fn action_risk(action: &ResolvedAction, kind: &CommandKind) -> RiskLevel {
    match action {
        // Opening a URL: medium-low — opens external content in a browser.
        ResolvedAction::OpenUrl { .. } => RiskLevel::R1,
        // Opening an app: low risk, straightforward launch.
        ResolvedAction::OpenApp { .. } => RiskLevel::R0,
        // AppleScript: risk depends on what the script does.
        ResolvedAction::AppleScriptTemplate { template_id, .. } => {
            applescript_template_risk(template_id)
        }
        // Opening system preferences: read-only navigation, R0.
        ResolvedAction::OpenSystemPreferences { .. } => RiskLevel::R0,
        // Opening a file path: R0 for reads, R1 if it could be a write.
        ResolvedAction::OpenPath { .. } => match kind {
            CommandKind::Filesystem => RiskLevel::R0,
            _ => RiskLevel::R1,
        },
    }
}

fn applescript_template_risk(template_id: &str) -> RiskLevel {
    match template_id {
        // Mute/unmute and volume change are reversible — R1.
        "mute_volume" | "unmute_volume" | "set_volume" | "get_volume" => RiskLevel::R1,
        // Unknown template — treat conservatively.
        _ => RiskLevel::R2,
    }
}

/// Determine if user approval is required before execution.
pub fn requires_approval(risk: &RiskLevel) -> bool {
    matches!(risk, RiskLevel::R1 | RiskLevel::R2 | RiskLevel::R3)
}

/// Build initial approval status for a command.
pub fn initial_approval_status(requires: bool) -> ApprovalStatus {
    if requires {
        ApprovalStatus::Pending
    } else {
        ApprovalStatus::NotRequired
    }
}

/// Produce the inverse action for a given action where possible.
pub fn inverse_action(action: &ResolvedAction) -> Option<ResolvedAction> {
    match action {
        ResolvedAction::AppleScriptTemplate { template_id, .. } => {
            match template_id.as_str() {
                "mute_volume" => Some(ResolvedAction::AppleScriptTemplate {
                    script: "set volume without output muted".to_string(),
                    template_id: "unmute_volume".to_string(),
                }),
                "set_volume" => {
                    // Inverse is captured at execution time (get current volume first).
                    None
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Fully annotate a ParsedCommand with risk and approval metadata.
pub fn annotate(mut cmd: ParsedCommand) -> ParsedCommand {
    let risk = score(&cmd.kind, &cmd.routes);
    let req = requires_approval(&risk);
    cmd.risk = risk;
    cmd.requires_approval = req;
    cmd.approval_status = initial_approval_status(req);
    cmd
}
