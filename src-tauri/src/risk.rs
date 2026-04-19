use crate::models::{ApprovalStatus, CommandKind, ParsedCommand, ResolvedAction, RiskLevel};

pub fn score(kind: &CommandKind, routes: &[crate::models::ResolvedRoute]) -> RiskLevel {
    routes
        .iter()
        .map(|r| action_risk(&r.action, kind))
        .max()
        .unwrap_or(RiskLevel::R0)
}

fn action_risk(action: &ResolvedAction, kind: &CommandKind) -> RiskLevel {
    match action {
        ResolvedAction::OpenUrl { .. } => RiskLevel::R1,
        ResolvedAction::OpenApp { .. } => RiskLevel::R0,
        ResolvedAction::QuitApp { .. } => RiskLevel::R1,
        ResolvedAction::AppleScriptTemplate { template_id, .. } => {
            applescript_template_risk(template_id)
        }
        ResolvedAction::OpenSystemPreferences { .. } => RiskLevel::R0,
        ResolvedAction::OpenPath { .. } => match kind {
            CommandKind::Filesystem => RiskLevel::R0,
            _ => RiskLevel::R1,
        },
        ResolvedAction::CreateFolder { .. } => RiskLevel::R1,
        ResolvedAction::MovePath { .. } => RiskLevel::R2,
    }
}

fn applescript_template_risk(template_id: &str) -> RiskLevel {
    match template_id {
        "mute_volume" | "unmute_volume" | "set_volume" | "get_volume" => RiskLevel::R1,
        _ => RiskLevel::R2,
    }
}

pub fn requires_approval(risk: &RiskLevel) -> bool {
    matches!(risk, RiskLevel::R1 | RiskLevel::R2 | RiskLevel::R3)
}

pub fn initial_approval_status(requires: bool) -> ApprovalStatus {
    if requires {
        ApprovalStatus::Pending
    } else {
        ApprovalStatus::NotRequired
    }
}

pub fn inverse_action(action: &ResolvedAction) -> Option<ResolvedAction> {
    match action {
        ResolvedAction::AppleScriptTemplate { template_id, .. } => match template_id.as_str() {
            "mute_volume" => Some(ResolvedAction::AppleScriptTemplate {
                script: "set volume without output muted".to_string(),
                template_id: "unmute_volume".to_string(),
            }),
            "set_volume" => None,
            _ => None,
        },
        ResolvedAction::QuitApp {
            bundle_id,
            app_name,
        } => Some(ResolvedAction::OpenApp {
            bundle_id: bundle_id.clone(),
            app_name: app_name.clone(),
        }),
        ResolvedAction::MovePath {
            source_path,
            destination_path,
        } => Some(ResolvedAction::MovePath {
            source_path: destination_path.clone(),
            destination_path: source_path.clone(),
        }),
        _ => None,
    }
}

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
    fn quit_app_is_r1() {
        let action = ResolvedAction::QuitApp {
            bundle_id: "com.apple.Safari".to_string(),
            app_name: "Safari".to_string(),
        };
        let risk = score(&CommandKind::AppControl, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R1);
    }

    #[test]
    fn move_path_is_r2() {
        let action = ResolvedAction::MovePath {
            source_path: "/Users/test/Desktop/a.txt".to_string(),
            destination_path: "/Users/test/Documents/a.txt".to_string(),
        };
        let risk = score(&CommandKind::Filesystem, &[make_route(action)]);
        assert_eq!(risk, RiskLevel::R2);
    }

    #[test]
    fn quit_app_has_open_app_inverse() {
        let action = ResolvedAction::QuitApp {
            bundle_id: "com.apple.Safari".to_string(),
            app_name: "Safari".to_string(),
        };
        let inv = inverse_action(&action).unwrap();
        match inv {
            ResolvedAction::OpenApp { bundle_id, .. } => {
                assert_eq!(bundle_id, "com.apple.Safari");
            }
            _ => panic!("expected OpenApp inverse"),
        }
    }
}
