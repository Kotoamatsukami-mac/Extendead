use crate::models::{BrowserInfo, CommandKind, ResolvedAction, ResolvedRoute};
use crate::parser::Intent;

pub fn resolve(intent: &Intent, browsers: &[BrowserInfo]) -> (CommandKind, Vec<ResolvedRoute>) {
    match intent {
        Intent::OpenYoutube => resolve_youtube(browsers),
        Intent::OpenYoutubeInSafari => resolve_youtube_in_browser("Safari", "com.apple.Safari"),
        Intent::OpenYoutubeInChrome => {
            resolve_youtube_in_browser("Google Chrome", "com.google.Chrome")
        }
        Intent::OpenYoutubeInFirefox => {
            resolve_youtube_in_browser("Firefox", "org.mozilla.firefox")
        }
        Intent::OpenYoutubeInBrave => {
            resolve_youtube_in_browser("Brave", "com.brave.Browser")
        }
        Intent::OpenYoutubeInArc => {
            resolve_youtube_in_browser("Arc", "company.thebrowser.Browser")
        }
        Intent::OpenSafari => resolve_open_app("Safari", "com.apple.Safari"),
        Intent::OpenChrome => resolve_open_app("Google Chrome", "com.google.Chrome"),
        Intent::OpenFirefox => resolve_open_app("Firefox", "org.mozilla.firefox"),
        Intent::OpenBrave => resolve_open_app("Brave", "com.brave.Browser"),
        Intent::OpenArc => resolve_open_app("Arc", "company.thebrowser.Browser"),
        Intent::OpenFinder => resolve_open_app("Finder", "com.apple.finder"),
        Intent::OpenSlack => resolve_open_app("Slack", "com.tinyspeck.slackmacgap"),
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

fn resolve_youtube_in_browser(browser_name: &str, bundle_id: &str) -> (CommandKind, Vec<ResolvedRoute>) {
    (
        CommandKind::MixedWorkflow,
        vec![ResolvedRoute {
            label: format!("Open YouTube in {browser_name}"),
            description: format!("Open youtube.com in {browser_name}"),
            action: ResolvedAction::OpenUrl {
                url: "https://www.youtube.com".to_string(),
                browser_bundle: bundle_id.to_string(),
                browser_name: browser_name.to_string(),
            },
        }],
    )
}

fn resolve_open_app(app_name: &str, bundle_id: &str) -> (CommandKind, Vec<ResolvedRoute>) {
    (
        CommandKind::AppControl,
        vec![ResolvedRoute {
            label: format!("Open {app_name}"),
            description: format!("Launch {app_name}.app"),
            action: ResolvedAction::OpenApp {
                bundle_id: bundle_id.to_string(),
                app_name: app_name.to_string(),
            },
        }],
    )
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

    #[test]
    fn youtube_no_browsers_resolves_default() {
        let (kind, routes) = resolve(&Intent::OpenYoutube, &no_browsers());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert_eq!(routes.len(), 1);
    }

    #[test]
    fn youtube_two_browsers_resolves_multiple_routes() {
        let (kind, routes) = resolve(&Intent::OpenYoutube, &two_browsers());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn youtube_in_specific_browser_resolves_single_target() {
        let (kind, routes) = resolve(&Intent::OpenYoutubeInSafari, &no_browsers());
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
    fn browser_app_intent_resolves_to_open_app() {
        let (kind, routes) = resolve(&Intent::OpenChrome, &no_browsers());
        assert_eq!(kind, CommandKind::AppControl);
        match &routes[0].action {
            ResolvedAction::OpenApp { bundle_id, app_name } => {
                assert_eq!(bundle_id, "com.google.Chrome");
                assert_eq!(app_name, "Google Chrome");
            }
            _ => panic!("expected OpenApp"),
        }
    }

    #[test]
    fn finder_resolves_to_open_app() {
        let (kind, routes) = resolve(&Intent::OpenFinder, &no_browsers());
        assert_eq!(kind, CommandKind::AppControl);
        match &routes[0].action {
            ResolvedAction::OpenApp { bundle_id, .. } => {
                assert_eq!(bundle_id, "com.apple.finder");
            }
            _ => panic!("expected OpenApp"),
        }
    }

    #[test]
    fn existing_commands_still_resolve() {
        assert_eq!(resolve(&Intent::OpenSlack, &no_browsers()).0, CommandKind::AppControl);
        assert_eq!(resolve(&Intent::MuteVolume, &no_browsers()).0, CommandKind::LocalSystem);
        assert_eq!(resolve(&Intent::SetVolume(42), &no_browsers()).0, CommandKind::LocalSystem);
        assert_eq!(resolve(&Intent::OpenDisplaySettings, &no_browsers()).0, CommandKind::LocalSystem);
        assert_eq!(resolve(&Intent::RevealDownloads, &no_browsers()).0, CommandKind::Filesystem);
    }
}
