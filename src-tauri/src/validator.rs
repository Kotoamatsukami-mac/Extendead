use crate::errors::AppError;
use crate::models::{ParsedCommand, ResolvedAction};
use crate::service_catalog;

/// Approved AppleScript template IDs.
static APPROVED_TEMPLATE_IDS: &[&str] =
    &["mute_volume", "unmute_volume", "set_volume", "get_volume"];

/// Approved bundle IDs for app actions.
static APPROVED_BUNDLE_IDS: &[&str] = &[
    "com.tinyspeck.slackmacgap",
    "com.google.Chrome",
    "com.apple.Safari",
    "org.mozilla.firefox",
    "com.brave.Browser",
    "company.thebrowser.Browser",
    "com.apple.finder",
];

/// Validate that a parsed command and its selected route are safe to execute.
pub fn validate(command: &ParsedCommand, route_index: usize) -> Result<(), AppError> {
    let route = command.routes.get(route_index).ok_or_else(|| {
        AppError::ValidationError(format!(
            "Route index {route_index} out of range (command has {} routes)",
            command.routes.len()
        ))
    })?;

    validate_action(&route.action)
}

pub fn validate_action(action: &ResolvedAction) -> Result<(), AppError> {
    match action {
        ResolvedAction::OpenUrl { url, .. } => validate_url(url),
        ResolvedAction::OpenApp { bundle_id, .. } | ResolvedAction::QuitApp { bundle_id, .. } => {
            validate_bundle_id(bundle_id)
        }
        ResolvedAction::AppleScriptTemplate { template_id, .. } => {
            if !APPROVED_TEMPLATE_IDS.contains(&template_id.as_str()) {
                return Err(AppError::ValidationError(format!(
                    "AppleScript template '{template_id}' is not approved"
                )));
            }
            Ok(())
        }
        ResolvedAction::OpenSystemPreferences { .. } => Ok(()),
        ResolvedAction::OpenPath { path } | ResolvedAction::CreateFolder { path } => {
            validate_home_path(path)
        }
        ResolvedAction::MovePath {
            source_path,
            destination_path,
        } => {
            validate_home_path(source_path)?;
            validate_home_path(destination_path)
        }
    }
}

fn validate_url(url: &str) -> Result<(), AppError> {
    let host = extract_host(url).ok_or_else(|| {
        AppError::ValidationError(format!("Cannot extract host from URL: {url}"))
    })?;
    if !service_catalog::is_approved_service_host(&host) {
        return Err(AppError::ValidationError(format!(
            "URL host '{host}' is not on the approved list"
        )));
    }
    Ok(())
}

fn validate_bundle_id(bundle_id: &str) -> Result<(), AppError> {
    if !APPROVED_BUNDLE_IDS.contains(&bundle_id) {
        return Err(AppError::ValidationError(format!(
            "Bundle ID '{bundle_id}' is not on the approved list"
        )));
    }
    Ok(())
}

fn validate_home_path(path: &str) -> Result<(), AppError> {
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    if !home.is_empty() && !path.starts_with(&home) {
        return Err(AppError::ValidationError(format!(
            "Path '{path}' is outside the home directory"
        )));
    }
    Ok(())
}

fn extract_host(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = without_scheme.split('/').next()?;
    Some(host.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_service_url_passes() {
        let action = ResolvedAction::OpenUrl {
            url: "https://www.youtube.com".to_string(),
            browser_bundle: String::new(),
            browser_name: "Safari".to_string(),
        };
        assert!(validate_action(&action).is_ok());
    }

    #[test]
    fn approved_open_app_bundle_passes() {
        let action = ResolvedAction::OpenApp {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
        };
        assert!(validate_action(&action).is_ok());
    }

    #[test]
    fn approved_quit_app_bundle_passes() {
        let action = ResolvedAction::QuitApp {
            bundle_id: "com.apple.Safari".to_string(),
            app_name: "Safari".to_string(),
        };
        assert!(validate_action(&action).is_ok());
    }

    #[test]
    fn unapproved_bundle_id_is_rejected() {
        let action = ResolvedAction::QuitApp {
            bundle_id: "com.evil.app".to_string(),
            app_name: "Evil".to_string(),
        };
        let err = validate_action(&action).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn create_folder_inside_home_passes() {
        if let Some(home) = dirs::home_dir() {
            let action = ResolvedAction::CreateFolder {
                path: format!("{}/Chat", home.display()),
            };
            assert!(validate_action(&action).is_ok());
        }
    }

    #[test]
    fn move_path_outside_home_is_rejected() {
        if dirs::home_dir().is_some() {
            let action = ResolvedAction::MovePath {
                source_path: "/etc/hosts".to_string(),
                destination_path: "/tmp/hosts".to_string(),
            };
            let err = validate_action(&action).unwrap_err();
            assert!(matches!(err, AppError::ValidationError(_)));
        }
    }

    #[test]
    fn out_of_range_route_index_is_rejected() {
        use crate::models::{ApprovalStatus, CommandKind, ResolvedRoute, RiskLevel};
        let cmd = ParsedCommand {
            id: "test".to_string(),
            raw_input: "mute".to_string(),
            normalized: "mute".to_string(),
            kind: CommandKind::LocalSystem,
            routes: vec![ResolvedRoute {
                label: "Mute".to_string(),
                description: "Mute audio".to_string(),
                action: ResolvedAction::AppleScriptTemplate {
                    script: "set volume with output muted".to_string(),
                    template_id: "mute_volume".to_string(),
                },
            }],
            risk: RiskLevel::R1,
            requires_approval: true,
            approval_status: ApprovalStatus::Approved,
            unresolved_message: None,
        };
        let err = validate(&cmd, 99).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }
}
