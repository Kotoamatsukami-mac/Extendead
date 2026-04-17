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
