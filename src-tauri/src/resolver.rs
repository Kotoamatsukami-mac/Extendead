use crate::models::{BrowserInfo, CommandKind, ResolvedAction, ResolvedRoute};
use crate::parser::Intent;

/// Resolve a parsed intent into one or more concrete routes.
/// Machine info (installed browsers) is passed in so Rust owns the scan result.
pub fn resolve(intent: &Intent, browsers: &[BrowserInfo]) -> (CommandKind, Vec<ResolvedRoute>) {
    match intent {
        Intent::OpenYoutube => resolve_youtube(browsers),
        Intent::OpenSlack => resolve_slack(),
        Intent::MuteVolume => resolve_mute(),
        Intent::SetVolume(level) => resolve_set_volume(*level),
        Intent::OpenDisplaySettings => resolve_display_settings(),
        Intent::RevealDownloads => resolve_downloads(),
        Intent::Unknown(_) => (CommandKind::Unknown, vec![]),
    }
}

fn resolve_youtube(browsers: &[BrowserInfo]) -> (CommandKind, Vec<ResolvedRoute>) {
    let url = "https://www.youtube.com";
    let routes: Vec<ResolvedRoute> = if browsers.is_empty() {
        // Fall back to default browser via plain URL open
        vec![ResolvedRoute {
            label: "Open YouTube".to_string(),
            description: "Open youtube.com in default browser".to_string(),
            action: ResolvedAction::OpenUrl {
                url: url.to_string(),
                browser_bundle: String::new(),
                browser_name: "Default Browser".to_string(),
            },
        }]
    } else {
        browsers
            .iter()
            .map(|b| ResolvedRoute {
                label: format!("Open in {}", b.name),
                description: format!("Open youtube.com in {}", b.name),
                action: ResolvedAction::OpenUrl {
                    url: url.to_string(),
                    browser_bundle: b.bundle_id.clone(),
                    browser_name: b.name.clone(),
                },
            })
            .collect()
    };
    (CommandKind::MixedWorkflow, routes)
}

fn resolve_slack() -> (CommandKind, Vec<ResolvedRoute>) {
    let routes = vec![ResolvedRoute {
        label: "Open Slack".to_string(),
        description: "Launch Slack.app".to_string(),
        action: ResolvedAction::OpenApp {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
        },
    }];
    (CommandKind::AppControl, routes)
}

fn resolve_mute() -> (CommandKind, Vec<ResolvedRoute>) {
    let routes = vec![ResolvedRoute {
        label: "Mute Mac".to_string(),
        description: "Mute system audio output".to_string(),
        action: ResolvedAction::AppleScriptTemplate {
            script: "set volume with output muted".to_string(),
            template_id: "mute_volume".to_string(),
        },
    }];
    (CommandKind::LocalSystem, routes)
}

fn resolve_set_volume(level: u8) -> (CommandKind, Vec<ResolvedRoute>) {
    let level = level.min(100);
    let routes = vec![ResolvedRoute {
        label: format!("Set volume to {level}%"),
        description: format!("Set system output volume to {level}%"),
        action: ResolvedAction::AppleScriptTemplate {
            script: format!("set volume output volume {level}"),
            template_id: "set_volume".to_string(),
        },
    }];
    (CommandKind::LocalSystem, routes)
}

fn resolve_display_settings() -> (CommandKind, Vec<ResolvedRoute>) {
    let routes = vec![ResolvedRoute {
        label: "Open Display Settings".to_string(),
        description: "Open System Settings → Displays".to_string(),
        action: ResolvedAction::OpenSystemPreferences {
            pane_url: "x-apple.systempreferences:com.apple.preference.displays".to_string(),
        },
    }];
    (CommandKind::LocalSystem, routes)
}

fn resolve_downloads() -> (CommandKind, Vec<ResolvedRoute>) {
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~".to_string());
    let downloads_path = format!("{home}/Downloads");
    let routes = vec![ResolvedRoute {
        label: "Reveal Downloads".to_string(),
        description: "Open ~/Downloads in Finder".to_string(),
        action: ResolvedAction::OpenPath {
            path: downloads_path,
        },
    }];
    (CommandKind::Filesystem, routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Intent;

    fn no_browsers() -> Vec<BrowserInfo> {
        vec![]
    }

    fn one_browser() -> Vec<BrowserInfo> {
        vec![BrowserInfo {
            name: "Safari".to_string(),
            bundle_id: "com.apple.Safari".to_string(),
            path: "/Applications/Safari.app".to_string(),
        }]
    }

    fn two_browsers() -> Vec<BrowserInfo> {
        vec![
            BrowserInfo {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
                path: "/Applications/Safari.app".to_string(),
            },
            BrowserInfo {
                name: "Chrome".to_string(),
                bundle_id: "com.google.Chrome".to_string(),
                path: "/Applications/Google Chrome.app".to_string(),
            },
        ]
    }

    // ── YouTube ──────────────────────────────────────────────────────────────

    #[test]
    fn youtube_no_browsers_resolves_default() {
        let (kind, routes) = resolve(&Intent::OpenYoutube, &no_browsers());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert_eq!(routes.len(), 1);
        match &routes[0].action {
            ResolvedAction::OpenUrl {
                url,
                browser_bundle,
                browser_name,
            } => {
                assert_eq!(url, "https://www.youtube.com");
                assert_eq!(browser_bundle, "");
                assert_eq!(browser_name, "Default Browser");
            }
            _ => panic!("expected OpenUrl"),
        }
    }

    #[test]
    fn youtube_one_browser_resolves_single_route() {
        let (kind, routes) = resolve(&Intent::OpenYoutube, &one_browser());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert_eq!(routes.len(), 1);
        match &routes[0].action {
            ResolvedAction::OpenUrl { browser_bundle, .. } => {
                assert_eq!(browser_bundle, "com.apple.Safari");
            }
            _ => panic!("expected OpenUrl"),
        }
    }

    #[test]
    fn youtube_two_browsers_resolves_two_routes() {
        let (kind, routes) = resolve(&Intent::OpenYoutube, &two_browsers());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert_eq!(routes.len(), 2);
    }

    // ── Slack ─────────────────────────────────────────────────────────────────

    #[test]
    fn slack_resolves_to_app_control() {
        let (kind, routes) = resolve(&Intent::OpenSlack, &no_browsers());
        assert_eq!(kind, CommandKind::AppControl);
        assert_eq!(routes.len(), 1);
        match &routes[0].action {
            ResolvedAction::OpenApp {
                bundle_id,
                app_name,
            } => {
                assert_eq!(bundle_id, "com.tinyspeck.slackmacgap");
                assert_eq!(app_name, "Slack");
            }
            _ => panic!("expected OpenApp"),
        }
    }

    // ── Mute ─────────────────────────────────────────────────────────────────

    #[test]
    fn mute_resolves_to_applescript_template() {
        let (kind, routes) = resolve(&Intent::MuteVolume, &no_browsers());
        assert_eq!(kind, CommandKind::LocalSystem);
        assert_eq!(routes.len(), 1);
        match &routes[0].action {
            ResolvedAction::AppleScriptTemplate { template_id, .. } => {
                assert_eq!(template_id, "mute_volume");
            }
            _ => panic!("expected AppleScriptTemplate"),
        }
    }

    // ── Set volume ────────────────────────────────────────────────────────────

    #[test]
    fn set_volume_resolves_with_correct_level() {
        let (kind, routes) = resolve(&Intent::SetVolume(42), &no_browsers());
        assert_eq!(kind, CommandKind::LocalSystem);
        assert_eq!(routes.len(), 1);
        match &routes[0].action {
            ResolvedAction::AppleScriptTemplate {
                script,
                template_id,
            } => {
                assert_eq!(template_id, "set_volume");
                assert!(script.contains("42"), "script must contain the level");
            }
            _ => panic!("expected AppleScriptTemplate"),
        }
    }

    #[test]
    fn set_volume_clamps_to_100() {
        let (_, routes) = resolve(&Intent::SetVolume(200), &no_browsers());
        assert_eq!(routes[0].label, "Set volume to 100%");
    }

    // ── Display settings ──────────────────────────────────────────────────────

    #[test]
    fn display_settings_resolves_to_pref_pane() {
        let (kind, routes) = resolve(&Intent::OpenDisplaySettings, &no_browsers());
        assert_eq!(kind, CommandKind::LocalSystem);
        match &routes[0].action {
            ResolvedAction::OpenSystemPreferences { pane_url } => {
                assert!(pane_url.contains("displays"));
            }
            _ => panic!("expected OpenSystemPreferences"),
        }
    }

    // ── Downloads ─────────────────────────────────────────────────────────────

    #[test]
    fn downloads_resolves_to_open_path() {
        let (kind, routes) = resolve(&Intent::RevealDownloads, &no_browsers());
        assert_eq!(kind, CommandKind::Filesystem);
        match &routes[0].action {
            ResolvedAction::OpenPath { path } => {
                assert!(path.ends_with("/Downloads"));
            }
            _ => panic!("expected OpenPath"),
        }
    }

    // ── Unknown ───────────────────────────────────────────────────────────────

    #[test]
    fn unknown_resolves_to_empty_routes() {
        let (kind, routes) = resolve(
            &Intent::Unknown("gibberish command".to_string()),
            &no_browsers(),
        );
        assert_eq!(kind, CommandKind::Unknown);
        assert!(routes.is_empty());
    }
}
