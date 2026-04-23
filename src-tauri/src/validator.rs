use crate::errors::AppError;
use crate::machine;
use crate::models::{ParsedCommand, ResolvedAction};
use crate::path_policy;
use crate::service_catalog;
use std::path::Path;

/// Approved AppleScript template IDs.
static APPROVED_TEMPLATE_IDS: &[&str] = &[
    "mute_volume",
    "unmute_volume",
    "set_volume",
    "get_volume",
    "browser_new_tab",
    "browser_close_tab",
    "browser_reopen_closed_tab",
    "brightness_up",
    "brightness_down",
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
        } => validate_move_path(source_path, destination_path),
    }
}

fn validate_url(url: &str) -> Result<(), AppError> {
    let host = extract_host(url)
        .ok_or_else(|| AppError::ValidationError(format!("Cannot extract host from URL: {url}")))?;
    if !service_catalog::is_approved_service_host(&host) {
        return Err(AppError::ValidationError(format!(
            "URL host '{host}' is not on the approved list"
        )));
    }
    Ok(())
}

fn validate_bundle_id(bundle_id: &str) -> Result<(), AppError> {
    if !machine::is_supported_bundle_id(bundle_id) {
        return Err(AppError::ValidationError(format!(
            "Bundle ID '{bundle_id}' is outside the controlled app catalog"
        )));
    }
    Ok(())
}

fn validate_home_path(path: &str) -> Result<(), AppError> {
    let boundary = path_policy::canonical_home_and_trash()?;

    let requested = Path::new(path);
    let requested_canonical =
        path_policy::canonicalize_path_for_boundary(requested).map_err(|e| {
            AppError::ValidationError(format!("Path '{path}' cannot be resolved safely: {e}"))
        })?;

    if !requested_canonical.starts_with(&boundary.home_canonical) {
        return Err(AppError::ValidationError(format!(
            "Path '{path}' is outside the home directory"
        )));
    }
    Ok(())
}

fn validate_move_path(source_path: &str, destination_path: &str) -> Result<(), AppError> {
    let boundary = path_policy::canonical_home_and_trash()?;

    let source = path_policy::canonicalize_existing_path_for_boundary(Path::new(source_path))
        .map_err(|e| {
            AppError::ValidationError(format!(
                "Source path '{source_path}' cannot be resolved safely: {e}"
            ))
        })?;
    if !source.starts_with(&boundary.home_canonical) {
        return Err(AppError::ValidationError(format!(
            "Source path '{source_path}' is outside the home directory"
        )));
    }

    let destination = path_policy::canonicalize_path_for_boundary(Path::new(destination_path))
        .map_err(|e| {
            AppError::ValidationError(format!(
                "Destination path '{destination_path}' cannot be resolved safely: {e}"
            ))
        })?;
    if !destination.starts_with(&boundary.home_canonical) {
        return Err(AppError::ValidationError(format!(
            "Destination path '{destination_path}' is outside the home directory"
        )));
    }

    if destination.starts_with(&boundary.trash_canonical) {
        if source == boundary.home_canonical {
            return Err(AppError::ValidationError(
                "Cannot move the entire home directory to Trash".to_string(),
            ));
        }
        if source == boundary.trash_canonical || source.starts_with(&boundary.trash_canonical) {
            return Err(AppError::ValidationError(
                "Path is already in Trash".to_string(),
            ));
        }
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
    fn create_folder_with_home_prefix_trick_is_rejected() {
        if let Some(home) = dirs::home_dir() {
            let action = ResolvedAction::CreateFolder {
                path: format!("{}-evil/tmp", home.display()),
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
            unresolved_code: None,
            unresolved_message: None,
            interpretation_decision: None,
            clarification_message: None,
            clarification_slots: vec![],
            choices: vec![],
        };
        let err = validate(&cmd, 99).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn browser_tab_template_is_allowed() {
        let action = ResolvedAction::AppleScriptTemplate {
            script: "tell application \"System Events\" to keystroke \"t\" using {command down}"
                .to_string(),
            template_id: "browser_new_tab".to_string(),
        };
        assert!(validate_action(&action).is_ok());
    }

    #[test]
    fn trashing_home_directory_is_rejected() {
        if let Some(home) = dirs::home_dir() {
            let action = ResolvedAction::MovePath {
                source_path: home.display().to_string(),
                destination_path: format!("{}/.Trash/home", home.display()),
            };
            let err = validate_action(&action).unwrap_err();
            assert!(matches!(err, AppError::ValidationError(_)));
        }
    }
}
