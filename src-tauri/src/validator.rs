use crate::errors::AppError;
use crate::models::{ParsedCommand, ResolvedAction};

/// Approved AppleScript template IDs.
static APPROVED_TEMPLATE_IDS: &[&str] =
    &["mute_volume", "unmute_volume", "set_volume", "get_volume"];

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
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = without_scheme.split('/').next()?;
    Some(host.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL validation ────────────────────────────────────────────────────────

    #[test]
    fn approved_youtube_url_passes() {
        let action = ResolvedAction::OpenUrl {
            url: "https://www.youtube.com".to_string(),
            browser_bundle: String::new(),
            browser_name: "Safari".to_string(),
        };
        assert!(validate_action(&action).is_ok());
    }

    #[test]
    fn unapproved_url_host_is_rejected() {
        let action = ResolvedAction::OpenUrl {
            url: "https://example.com".to_string(),
            browser_bundle: String::new(),
            browser_name: "Safari".to_string(),
        };
        let err = validate_action(&action).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[test]
    fn malformed_url_without_scheme_is_rejected() {
        let action = ResolvedAction::OpenUrl {
            url: "not-a-url".to_string(),
            browser_bundle: String::new(),
            browser_name: "Safari".to_string(),
        };
        let err = validate_action(&action).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    // ── Bundle ID validation ──────────────────────────────────────────────────

    #[test]
    fn approved_bundle_id_passes() {
        let action = ResolvedAction::OpenApp {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
        };
        assert!(validate_action(&action).is_ok());
    }

    #[test]
    fn all_approved_browsers_pass() {
        let bundles = [
            "com.google.Chrome",
            "com.apple.Safari",
            "org.mozilla.firefox",
            "com.brave.Browser",
            "company.thebrowser.Browser",
        ];
        for bundle in bundles {
            let action = ResolvedAction::OpenApp {
                bundle_id: bundle.to_string(),
                app_name: "Browser".to_string(),
            };
            assert!(
                validate_action(&action).is_ok(),
                "bundle {bundle} should be approved"
            );
        }
    }

    #[test]
    fn unapproved_bundle_id_is_rejected() {
        let action = ResolvedAction::OpenApp {
            bundle_id: "com.evil.app".to_string(),
            app_name: "Evil".to_string(),
        };
        let err = validate_action(&action).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    // ── AppleScript template validation ───────────────────────────────────────

    #[test]
    fn approved_templates_pass() {
        let templates = ["mute_volume", "unmute_volume", "set_volume", "get_volume"];
        for tmpl in templates {
            let action = ResolvedAction::AppleScriptTemplate {
                script: "set volume with output muted".to_string(),
                template_id: tmpl.to_string(),
            };
            assert!(
                validate_action(&action).is_ok(),
                "template {tmpl} should be approved"
            );
        }
    }

    #[test]
    fn unapproved_template_is_rejected() {
        let action = ResolvedAction::AppleScriptTemplate {
            script: "do shell script \"rm -rf /\"".to_string(),
            template_id: "dangerous_script".to_string(),
        };
        let err = validate_action(&action).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }

    // ── System preferences ────────────────────────────────────────────────────

    #[test]
    fn system_preferences_always_passes() {
        let action = ResolvedAction::OpenSystemPreferences {
            pane_url: "x-apple.systempreferences:com.apple.preference.displays".to_string(),
        };
        assert!(validate_action(&action).is_ok());
    }

    // ── Path validation ───────────────────────────────────────────────────────

    #[test]
    fn path_outside_home_is_rejected() {
        // Only meaningful if home_dir is available; skip on CI if not.
        if let Some(home) = dirs::home_dir() {
            let action = ResolvedAction::OpenPath {
                path: "/etc/passwd".to_string(),
            };
            // /etc is outside home, so it should fail if home is non-empty.
            if !home.display().to_string().is_empty() {
                let err = validate_action(&action).unwrap_err();
                assert!(matches!(err, AppError::ValidationError(_)));
            }
        }
    }

    #[test]
    fn path_inside_home_passes() {
        if let Some(home) = dirs::home_dir() {
            let path = format!("{}/Downloads", home.display());
            let action = ResolvedAction::OpenPath { path };
            assert!(validate_action(&action).is_ok());
        }
    }

    // ── Route index out of range ──────────────────────────────────────────────

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
        };
        let err = validate(&cmd, 99).unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
    }
}
