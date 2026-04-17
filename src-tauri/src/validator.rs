use crate::errors::AppError;
use crate::models::{ParsedCommand, ResolvedAction};

/// Approved AppleScript template IDs.
static APPROVED_TEMPLATE_IDS: &[&str] = &["mute_volume", "unmute_volume", "set_volume", "get_volume"];

/// Approved URL hostnames for OpenUrl actions.
static APPROVED_URL_HOSTS: &[&str] = &["www.youtube.com", "youtube.com"];

/// Approved bundle IDs for OpenApp actions.
static APPROVED_BUNDLE_IDS: &[&str] = &[
    "com.tinyspeck.slackmacgap",
    "com.google.Chrome",
    "com.apple.Safari",
    "org.mozilla.firefox",
    "com.brave.Browser",
    "company.thebrowser.Browser",
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
        ResolvedAction::OpenUrl { url, .. } => {
            let host = extract_host(url).ok_or_else(|| {
                AppError::ValidationError(format!("Cannot extract host from URL: {url}"))
            })?;
            if !APPROVED_URL_HOSTS.contains(&host.as_str()) {
                return Err(AppError::ValidationError(format!(
                    "URL host '{host}' is not on the approved list"
                )));
            }
            Ok(())
        }
        ResolvedAction::OpenApp { bundle_id, .. } => {
            if !APPROVED_BUNDLE_IDS.contains(&bundle_id.as_str()) {
                return Err(AppError::ValidationError(format!(
                    "Bundle ID '{bundle_id}' is not on the approved list"
                )));
            }
            Ok(())
        }
        ResolvedAction::AppleScriptTemplate { template_id, .. } => {
            if !APPROVED_TEMPLATE_IDS.contains(&template_id.as_str()) {
                return Err(AppError::ValidationError(format!(
                    "AppleScript template '{template_id}' is not approved"
                )));
            }
            Ok(())
        }
        // System preferences and path opens are low-risk and pre-approved in Phase 1.
        ResolvedAction::OpenSystemPreferences { .. } => Ok(()),
        ResolvedAction::OpenPath { path } => {
            // Only allow paths under home directory (resolver always expands ~).
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
    }
}

fn extract_host(url: &str) -> Option<String> {
    // Minimal host extraction without pulling in a URL parsing crate.
    let without_scheme = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let host = without_scheme.split('/').next()?;
    Some(host.to_string())
}
