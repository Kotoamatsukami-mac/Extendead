use crate::machine;
use crate::models::{BrowserInfo, CommandKind, MachineInfo, ResolvedAction, ResolvedRoute};
use crate::parser::Intent;

pub fn resolve(
    intent: &Intent,
    machine: &MachineInfo,
) -> (CommandKind, Vec<ResolvedRoute>, Option<String>) {
    let browsers = &machine.installed_browsers;
    match intent {
        Intent::OpenYoutube => {
            let (kind, routes) = resolve_youtube(browsers);
            (kind, routes, None)
        }
        Intent::OpenYoutubeInSafari => {
            resolve_youtube_in_browser(machine, "Safari", "com.apple.Safari")
        }
        Intent::OpenYoutubeInChrome => {
            resolve_youtube_in_browser(machine, "Google Chrome", "com.google.Chrome")
        }
        Intent::OpenYoutubeInFirefox => {
            resolve_youtube_in_browser(machine, "Firefox", "org.mozilla.firefox")
        }
        Intent::OpenYoutubeInBrave => {
            resolve_youtube_in_browser(machine, "Brave", "com.brave.Browser")
        }
        Intent::OpenYoutubeInArc => {
            resolve_youtube_in_browser(machine, "Arc", "company.thebrowser.Browser")
        }
        Intent::OpenSafari => resolve_open_app(machine, "Safari", "com.apple.Safari"),
        Intent::OpenChrome => resolve_open_app(machine, "Google Chrome", "com.google.Chrome"),
        Intent::OpenFirefox => resolve_open_app(machine, "Firefox", "org.mozilla.firefox"),
        Intent::OpenBrave => resolve_open_app(machine, "Brave", "com.brave.Browser"),
        Intent::OpenArc => resolve_open_app(machine, "Arc", "company.thebrowser.Browser"),
        Intent::OpenFinder => resolve_open_app(machine, "Finder", "com.apple.finder"),
        Intent::OpenSlack => resolve_open_app(machine, "Slack", "com.tinyspeck.slackmacgap"),
        Intent::MuteVolume => {
            let (kind, routes) = resolve_mute();
            (kind, routes, None)
        }
        Intent::SetVolume(level) => {
            let (kind, routes) = resolve_set_volume(*level);
            (kind, routes, None)
        }
        Intent::OpenDisplaySettings => {
            let (kind, routes) = resolve_display_settings();
            (kind, routes, None)
        }
        Intent::RevealDownloads => {
            let (kind, routes) = resolve_downloads();
            (kind, routes, None)
        }
        Intent::Unknown(_) => (
            CommandKind::Unknown,
            vec![],
            Some("That command is outside current local coverage.".to_string()),
        ),
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

fn resolve_youtube_in_browser(
    machine: &MachineInfo,
    browser_name: &str,
    bundle_id: &str,
) -> (CommandKind, Vec<ResolvedRoute>, Option<String>) {
    if !machine::is_app_installed(machine, bundle_id) {
        return (
            CommandKind::MixedWorkflow,
            vec![],
            Some(format!("{browser_name} is not installed on this Mac.")),
        );
    }

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
        None,
    )
}

fn resolve_open_app(
    machine: &MachineInfo,
    app_name: &str,
    bundle_id: &str,
) -> (CommandKind, Vec<ResolvedRoute>, Option<String>) {
    if !machine::is_app_installed(machine, bundle_id) {
        return (
            CommandKind::AppControl,
            vec![],
            Some(format!("{app_name} is not installed on this Mac.")),
        );
    }

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
        None,
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
        action: ResolvedAction::OpenPath { path: downloads_path },
    }];
    (CommandKind::Filesystem, routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppInfo;
    use crate::parser::Intent;

    fn no_browsers_machine() -> MachineInfo {
        MachineInfo {
            hostname: "test".to_string(),
            username: "test".to_string(),
            os_version: "14.0".to_string(),
            architecture: "x86_64".to_string(),
            installed_browsers: vec![],
            installed_apps: vec![],
            home_dir: "/Users/test".to_string(),
        }
    }

    fn full_machine() -> MachineInfo {
        MachineInfo {
            hostname: "test".to_string(),
            username: "test".to_string(),
            os_version: "14.0".to_string(),
            architecture: "x86_64".to_string(),
            installed_browsers: vec![
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
            ],
            installed_apps: vec![
                AppInfo {
                    name: "Slack".to_string(),
                    bundle_id: "com.tinyspeck.slackmacgap".to_string(),
                    path: "/Applications/Slack.app".to_string(),
                },
                AppInfo {
                    name: "Finder".to_string(),
                    bundle_id: "com.apple.finder".to_string(),
                    path: "/System/Library/CoreServices/Finder.app".to_string(),
                },
            ],
            home_dir: "/Users/test".to_string(),
        }
    }

    #[test]
    fn youtube_no_browsers_resolves_default() {
        let (kind, routes, unresolved) = resolve(&Intent::OpenYoutube, &no_browsers_machine());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert_eq!(routes.len(), 1);
        assert!(unresolved.is_none());
    }

    #[test]
    fn youtube_known_browser_resolves_single_target() {
        let (kind, routes, unresolved) = resolve(&Intent::OpenYoutubeInSafari, &full_machine());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert_eq!(routes.len(), 1);
        assert!(unresolved.is_none());
        match &routes[0].action {
            ResolvedAction::OpenUrl { browser_bundle, .. } => {
                assert_eq!(browser_bundle, "com.apple.Safari");
            }
            _ => panic!("expected OpenUrl"),
        }
    }

    #[test]
    fn missing_specific_browser_returns_unresolved_message() {
        let (kind, routes, unresolved) = resolve(&Intent::OpenYoutubeInSafari, &no_browsers_machine());
        assert_eq!(kind, CommandKind::MixedWorkflow);
        assert!(routes.is_empty());
        assert_eq!(unresolved.as_deref(), Some("Safari is not installed on this Mac."));
    }

    #[test]
    fn browser_app_intent_resolves_to_open_app() {
        let (kind, routes, unresolved) = resolve(&Intent::OpenChrome, &full_machine());
        assert_eq!(kind, CommandKind::AppControl);
        assert!(unresolved.is_none());
        match &routes[0].action {
            ResolvedAction::OpenApp {
                bundle_id,
                app_name,
            } => {
                assert_eq!(bundle_id, "com.google.Chrome");
                assert_eq!(app_name, "Google Chrome");
            }
            _ => panic!("expected OpenApp"),
        }
    }

    #[test]
    fn missing_app_returns_unresolved_message() {
        let (kind, routes, unresolved) = resolve(&Intent::OpenSlack, &no_browsers_machine());
        assert_eq!(kind, CommandKind::AppControl);
        assert!(routes.is_empty());
        assert_eq!(unresolved.as_deref(), Some("Slack is not installed on this Mac."));
    }

    #[test]
    fn unknown_returns_local_coverage_message() {
        let (kind, routes, unresolved) = resolve(
            &Intent::Unknown("do something impossible".to_string()),
            &full_machine(),
        );
        assert_eq!(kind, CommandKind::Unknown);
        assert!(routes.is_empty());
        assert_eq!(
            unresolved.as_deref(),
            Some("That command is outside current local coverage."),
        );
    }

    #[test]
    fn existing_commands_still_resolve() {
        let m = full_machine();
        assert_eq!(resolve(&Intent::OpenSlack, &m).0, CommandKind::AppControl);
        assert_eq!(resolve(&Intent::MuteVolume, &m).0, CommandKind::LocalSystem);
        assert_eq!(resolve(&Intent::SetVolume(42), &m).0, CommandKind::LocalSystem);
        assert_eq!(resolve(&Intent::OpenDisplaySettings, &m).0, CommandKind::LocalSystem);
        assert_eq!(resolve(&Intent::RevealDownloads, &m).0, CommandKind::Filesystem);
    }
}
