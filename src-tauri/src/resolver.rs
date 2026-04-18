use crate::models::{BrowserInfo, CommandKind, ResolvedAction, ResolvedRoute};
use crate::parser::{BrowserTarget, Intent};

pub fn resolve(intent: &Intent, browsers: &[BrowserInfo]) -> (CommandKind, Vec<ResolvedRoute>) {
    match intent {
        Intent::OpenYoutube => resolve_youtube(browsers),
        Intent::OpenYoutubeInBrowser(target) => resolve_youtube_in_browser(target, browsers),
        Intent::OpenBrowserApp(target) => resolve_browser_app(target, browsers),
        Intent::OpenFinder => resolve_finder(),
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
        browsers.iter().map(|b| youtube_route(url, b)).collect()
    };
    (CommandKind::MixedWorkflow, routes)
}

fn resolve_youtube_in_browser(
    target: &BrowserTarget,
    browsers: &[BrowserInfo],
) -> (CommandKind, Vec<ResolvedRoute>) {
    let browser = find_browser(target, browsers).unwrap_or_else(|| fallback_browser(target));
    (
        CommandKind::MixedWorkflow,
        vec![youtube_route("https://www.youtube.com", &browser)],
    )
}

fn resolve_browser_app(target: &BrowserTarget, browsers: &[BrowserInfo]) -> (CommandKind, Vec<ResolvedRoute>) {
    let browser = find_browser(target, browsers).unwrap_or_else(|| fallback_browser(target));
    (
        CommandKind::AppControl,
        vec![ResolvedRoute {
            label: format!("Open {}", browser.name),
            description: format!("Launch {}", browser.name),
            action: ResolvedAction::OpenApp {
                bundle_id: browser.bundle_id,
                app_name: browser.name,
            },
        }],
    )
}

fn resolve_finder() -> (CommandKind, Vec<ResolvedRoute>) {
    (
        CommandKind::AppControl,
        vec![ResolvedRoute {
            label: "Open Finder".to_string(),
            description: "Launch Finder.app".to_string(),
            action: ResolvedAction::OpenApp {
                bundle_id: "com.apple.finder".to_string(),
                app_name: "Finder".to_string(),
            },
        }],
    )
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
        action: ResolvedAction::OpenPath { path: downloads_path },
    }];
    (CommandKind::Filesystem, routes)
}

fn youtube_route(url: &str, browser: &BrowserInfo) -> ResolvedRoute {
    ResolvedRoute {
        label: format!("Open in {}", browser.name),
        description: format!("Open youtube.com in {}", browser.name),
        action: ResolvedAction::OpenUrl {
            url: url.to_string(),
            browser_bundle: browser.bundle_id.clone(),
            browser_name: browser.name.clone(),
        },
    }
}

fn find_browser(target: &BrowserTarget, browsers: &[BrowserInfo]) -> Option<BrowserInfo> {
    let bundle = fallback_browser(target).bundle_id;
    browsers.iter().find(|b| b.bundle_id == bundle).cloned()
}

fn fallback_browser(target: &BrowserTarget) -> BrowserInfo {
    match target {
        BrowserTarget::Safari => BrowserInfo {
            name: "Safari".to_string(),
            bundle_id: "com.apple.Safari".to_string(),
            path: "/Applications/Safari.app".to_string(),
        },
        BrowserTarget::Chrome => BrowserInfo {
            name: "Google Chrome".to_string(),
            bundle_id: "com.google.Chrome".to_string(),
            path: "/Applications/Google Chrome.app".to_string(),
        },
        BrowserTarget::Firefox => BrowserInfo {
            name: "Firefox".to_string(),
            bundle_id: "org.mozilla.firefox".to_string(),
            path: "/Applications/Firefox.app".to_string(),
        },
        BrowserTarget::Brave => BrowserInfo {
            name: "Brave".to_string(),
            bundle_id: "com.brave.Browser".to_string(),
            path: "/Applications/Brave Browser.app".to_string(),
        },
        BrowserTarget::Arc => BrowserInfo {
            name: "Arc".to_string(),
            bundle_id: "company.thebrowser.Browser".to_string(),
            path: "/Applications/Arc.app".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn targeted_youtube_uses_selected_browser() {
        let (_, routes) = resolve(&Intent::OpenYoutubeInBrowser(BrowserTarget::Safari), &one_browser());
        match &routes[0].action {
            ResolvedAction::OpenUrl { browser_bundle, .. } => {
                assert_eq!(browser_bundle, "com.apple.Safari");
            }
            _ => panic!("expected OpenUrl"),
        }
    }

    #[test]
    fn targeted_browser_open_is_app_control() {
        let (kind, routes) = resolve(&Intent::OpenBrowserApp(BrowserTarget::Chrome), &no_browsers());
        assert_eq!(kind, CommandKind::AppControl);
        match &routes[0].action {
            ResolvedAction::OpenApp { bundle_id, .. } => {
                assert_eq!(bundle_id, "com.google.Chrome");
            }
            _ => panic!("expected OpenApp"),
        }
    }

    #[test]
    fn finder_resolves_to_open_app() {
        let (kind, routes) = resolve(&Intent::OpenFinder, &no_browsers());
        assert_eq!(kind, CommandKind::AppControl);
        match &routes[0].action {
            ResolvedAction::OpenApp { bundle_id, app_name } => {
                assert_eq!(bundle_id, "com.apple.finder");
                assert_eq!(app_name, "Finder");
            }
            _ => panic!("expected OpenApp"),
        }
    }
}
