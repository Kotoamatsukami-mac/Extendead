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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ResolvedRoute;

    fn make_route(action: ResolvedAction) -> ResolvedRoute {
        ResolvedRoute {
            label: "Test".to_string(),
            description: "Test route".to_string(),
            action,
        }
    }

    // ── Risk scoring ──────────────────────────────────────────────────────────

    #[test]
    fn open_url_is_r1() {
        let action = ResolvedAction::OpenUrl {
            url: "https://www.youtube.com".to_string(),
            browser_bundle: String::new(),
            browser_name: "Safari".to_string(),
        };
        let risk = score(&CommandKind::MixedWorkflow, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R1);
    }

    #[test]
    fn open_app_is_r0() {
        let action = ResolvedAction::OpenApp {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
        };
        let risk = score(&CommandKind::AppControl, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R0);
    }

    #[test]
    fn mute_volume_template_is_r1() {
        let action = ResolvedAction::AppleScriptTemplate {
            script: "set volume with output muted".to_string(),
            template_id: "mute_volume".to_string(),
        };
        let risk = score(&CommandKind::LocalSystem, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R1);
    }

    #[test]
    fn set_volume_template_is_r1() {
        let action = ResolvedAction::AppleScriptTemplate {
            script: "set volume output volume 50".to_string(),
            template_id: "set_volume".to_string(),
        };
        let risk = score(&CommandKind::LocalSystem, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R1);
    }

    #[test]
    fn unknown_applescript_template_is_r2() {
        let action = ResolvedAction::AppleScriptTemplate {
            script: "do some script".to_string(),
            template_id: "unknown_template".to_string(),
        };
        let risk = score(&CommandKind::LocalSystem, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R2);
    }

    #[test]
    fn open_system_preferences_is_r0() {
        let action = ResolvedAction::OpenSystemPreferences {
            pane_url: "x-apple.systempreferences:com.apple.preference.displays".to_string(),
        };
        let risk = score(&CommandKind::LocalSystem, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R0);
    }

    #[test]
    fn filesystem_open_path_is_r0() {
        let action = ResolvedAction::OpenPath {
            path: "/Users/user/Downloads".to_string(),
        };
        let risk = score(&CommandKind::Filesystem, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R0);
    }

    #[test]
    fn empty_routes_defaults_r0() {
        let risk = score(&CommandKind::Unknown, &[]);
        assert_eq!(risk, RiskLevel::R0);
    }

    #[test]
    fn highest_risk_wins_across_routes() {
        let r0_action = ResolvedAction::OpenApp {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
        };
        let r1_action = ResolvedAction::OpenUrl {
            url: "https://www.youtube.com".to_string(),
            browser_bundle: String::new(),
            browser_name: "Safari".to_string(),
        };
        let risk = score(
            &CommandKind::MixedWorkflow,
            &[make_route(r0_action), make_route(r1_action)],
        );
        assert_eq!(risk, RiskLevel::R1);
    }

    // ── Approval gate ─────────────────────────────────────────────────────────

    #[test]
    fn r0_does_not_require_approval() {
        assert!(!requires_approval(&RiskLevel::R0));
    }

    #[test]
    fn r1_requires_approval() {
        assert!(requires_approval(&RiskLevel::R1));
    }

    #[test]
    fn r2_requires_approval() {
        assert!(requires_approval(&RiskLevel::R2));
    }

    #[test]
    fn r3_requires_approval() {
        assert!(requires_approval(&RiskLevel::R3));
    }

    // ── Inverse actions ───────────────────────────────────────────────────────

    #[test]
    fn mute_has_unmute_inverse() {
        let action = ResolvedAction::AppleScriptTemplate {
            script: "set volume with output muted".to_string(),
            template_id: "mute_volume".to_string(),
        };
        let inv = inverse_action(&action);
        assert!(inv.is_some());
        match inv.unwrap() {
            ResolvedAction::AppleScriptTemplate { template_id, .. } => {
                assert_eq!(template_id, "unmute_volume");
            }
            _ => panic!("expected AppleScriptTemplate"),
        }
    }

    #[test]
    fn set_volume_has_no_static_inverse() {
        // Inverse is captured dynamically at execution time.
        let action = ResolvedAction::AppleScriptTemplate {
            script: "set volume output volume 30".to_string(),
            template_id: "set_volume".to_string(),
        };
        assert!(inverse_action(&action).is_none());
    }

    #[test]
    fn open_url_has_no_inverse() {
        let action = ResolvedAction::OpenUrl {
            url: "https://www.youtube.com".to_string(),
            browser_bundle: String::new(),
            browser_name: "Safari".to_string(),
        };
        assert!(inverse_action(&action).is_none());
    }

    #[test]
    fn open_app_has_no_inverse() {
        let action = ResolvedAction::OpenApp {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
        };
        assert!(inverse_action(&action).is_none());
    }
}
