use std::path::{Path, PathBuf};

use crate::machine;
use crate::models::{
    BrowserInfo, CommandKind, MachineInfo, ResolvedAction, ResolvedRoute, UnresolvedCode,
};
use crate::parser::Intent;
use crate::service_catalog;

type ResolveResult = (
    CommandKind,
    Vec<ResolvedRoute>,
    Option<UnresolvedCode>,
    Option<String>,
);

pub fn resolve(intent: &Intent, machine: &MachineInfo) -> ResolveResult {
    match intent {
        Intent::OpenSafari => resolve_open_app(machine, "Safari", "com.apple.Safari"),
        Intent::OpenChrome => resolve_open_app(machine, "Google Chrome", "com.google.Chrome"),
        Intent::OpenFirefox => resolve_open_app(machine, "Firefox", "org.mozilla.firefox"),
        Intent::OpenBrave => resolve_open_app(machine, "Brave", "com.brave.Browser"),
        Intent::OpenArc => resolve_open_app(machine, "Arc", "company.thebrowser.Browser"),
        Intent::OpenFinder => resolve_open_app(machine, "Finder", "com.apple.finder"),
        Intent::OpenSlack => resolve_open_app(machine, "Slack", "com.tinyspeck.slackmacgap"),
        Intent::OpenService(service_id) => resolve_service(machine, service_id),
        Intent::OpenServiceInBrowser { service_id, browser } => {
            resolve_service_in_named_browser(machine, service_id, browser)
        }
        Intent::OpenAppNamed(name) => resolve_app_named(machine, name, false),
        Intent::CloseAppNamed(name) => resolve_app_named(machine, name, true),
        Intent::OpenPath(path) => resolve_open_path(path),
        Intent::CreateFolder { name, base } => resolve_create_folder(machine, name, base.as_deref()),
        Intent::MovePath { source, destination } => resolve_move_path(machine, source, destination),
        Intent::MuteVolume => {
            let (kind, routes) = resolve_mute();
            (kind, routes, None, None)
        }
        Intent::SetVolume(level) => {
            let (kind, routes) = resolve_set_volume(*level);
            (kind, routes, None, None)
        }
        Intent::OpenDisplaySettings => {
            let (kind, routes) = resolve_display_settings();
            (kind, routes, None, None)
        }
        Intent::RevealDownloads => {
            let (kind, routes) = resolve_downloads();
            (kind, routes, None, None)
        }
        Intent::Unknown(_) => (
            CommandKind::Unknown,
            vec![],
            Some(UnresolvedCode::UnsupportedCommand),
            Some("That command is outside current local coverage.".to_string()),
        ),
    }
}

fn resolve_service(machine: &MachineInfo, service_id: &str) -> ResolveResult {
    if service_catalog::service_by_id(service_id).is_none() {
        return (
            CommandKind::MixedWorkflow,
            vec![],
            Some(UnresolvedCode::UnsupportedService),
            Some("That service is outside current local coverage.".to_string()),
        );
    }

    let (kind, routes) = resolve_service_id(&machine.installed_browsers, service_id);
    (kind, routes, None, None)
}

fn resolve_service_id(browsers: &[BrowserInfo], service_id: &str) -> (CommandKind, Vec<ResolvedRoute>) {
    let Some(service) = service_catalog::service_by_id(service_id) else {
        return (CommandKind::MixedWorkflow, vec![]);
    };

    let routes: Vec<ResolvedRoute> = if browsers.is_empty() {
        vec![ResolvedRoute {
            label: format!("Open {}", service.display_name),
            description: format!("Open {} in default browser", service.display_name),
            action: ResolvedAction::OpenUrl {
                url: service.url.to_string(),
                browser_bundle: String::new(),
                browser_name: "Default Browser".to_string(),
            },
        }]
    } else {
        browsers
            .iter()
            .map(|b| ResolvedRoute {
                label: format!("Open in {}", b.name),
                description: format!("Open {} in {}", service.display_name, b.name),
                action: ResolvedAction::OpenUrl {
                    url: service.url.to_string(),
                    browser_bundle: b.bundle_id.clone(),
                    browser_name: b.name.clone(),
                },
            })
            .collect()
    };

    (CommandKind::MixedWorkflow, routes)
}

fn resolve_service_in_named_browser(
    machine: &MachineInfo,
    service_id: &str,
    browser_query: &str,
) -> ResolveResult {
    let Some(service) = service_catalog::service_by_id(service_id) else {
        return (
            CommandKind::MixedWorkflow,
            vec![],
            Some(UnresolvedCode::UnsupportedService),
            Some("That service is outside current local coverage.".to_string()),
        );
    };

    let Some((browser_name, bundle_id)) = find_installed_browser(machine, browser_query) else {
        return (
            CommandKind::MixedWorkflow,
            vec![],
            Some(UnresolvedCode::BrowserNotInstalled),
            Some(format!("{} is not installed on this Mac.", display_name(browser_query))),
        );
    };

    (
        CommandKind::MixedWorkflow,
        vec![ResolvedRoute {
            label: format!("Open {} in {}", service.display_name, browser_name),
            description: format!("Open {} in {}", service.display_name, browser_name),
            action: ResolvedAction::OpenUrl {
                url: service.url.to_string(),
                browser_bundle: bundle_id,
                browser_name,
            },
        }],
        None,
        None,
    )
}

fn resolve_open_app(machine: &MachineInfo, app_name: &str, bundle_id: &str) -> ResolveResult {
    if !machine::is_app_installed(machine, bundle_id) {
        return (
            CommandKind::AppControl,
            vec![],
            Some(UnresolvedCode::AppNotInstalled),
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
        None,
    )
}

fn resolve_app_named(machine: &MachineInfo, app_name: &str, should_quit: bool) -> ResolveResult {
    let query = canonical_app_name(app_name);
    let app = find_installed_app(machine, &query);
    let Some((resolved_name, bundle_id)) = app else {
        return (
            CommandKind::AppControl,
            vec![],
            Some(UnresolvedCode::AppNotInstalled),
            Some(format!("{} is not installed on this Mac.", display_name(&query))),
        );
    };

    let route = if should_quit {
        ResolvedRoute {
            label: format!("Close {resolved_name}"),
            description: format!("Quit {resolved_name}"),
            action: ResolvedAction::QuitApp {
                bundle_id,
                app_name: resolved_name,
            },
        }
    } else {
        ResolvedRoute {
            label: format!("Open {resolved_name}"),
            description: format!("Launch {resolved_name}.app"),
            action: ResolvedAction::OpenApp {
                bundle_id,
                app_name: resolved_name,
            },
        }
    };

    (CommandKind::AppControl, vec![route], None, None)
}

fn resolve_open_path(path: &str) -> ResolveResult {
    let expanded = expand_user_path(path);
    if !Path::new(&expanded).exists() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::PathNotFound),
            Some(format!("{} does not exist.", expanded)),
        );
    }

    (
        CommandKind::Filesystem,
        vec![ResolvedRoute {
            label: format!("Open {}", expanded),
            description: format!("Open {} in Finder", expanded),
            action: ResolvedAction::OpenPath { path: expanded },
        }],
        None,
        None,
    )
}

fn resolve_create_folder(
    machine: &MachineInfo,
    name: &str,
    base: Option<&str>,
) -> ResolveResult {
    let base_path = resolve_base_path(machine, base.unwrap_or("home"));
    let Some(base_path) = base_path else {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::BasePathUnresolved),
            Some("I could not resolve where to create that folder.".to_string()),
        );
    };

    let target = Path::new(&base_path).join(name);
    let target_path = target.display().to_string();
    if target.exists() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::TargetAlreadyExists),
            Some(format!("{} already exists.", target_path)),
        );
    }

    (
        CommandKind::Filesystem,
        vec![ResolvedRoute {
            label: format!("Create folder {}", name),
            description: format!("Create {}", target_path),
            action: ResolvedAction::CreateFolder { path: target_path },
        }],
        None,
        None,
    )
}

fn resolve_move_path(machine: &MachineInfo, source: &str, destination: &str) -> ResolveResult {
    let source_path = expand_user_path(source);
    let source_pb = PathBuf::from(&source_path);
    if !source_pb.exists() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::PathNotFound),
            Some(format!("{} does not exist.", source_path)),
        );
    }

    let destination_base = expand_user_path_with_machine(machine, destination);
    let destination_pb = PathBuf::from(&destination_base);
    let final_destination = if destination_pb.is_dir() {
        match source_pb.file_name() {
            Some(name) => destination_pb.join(name),
            None => destination_pb.clone(),
        }
    } else {
        destination_pb.clone()
    };

    let Some(parent) = final_destination.parent() else {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::DestinationPathUnresolved),
            Some("I could not resolve the destination path.".to_string()),
        );
    };

    if !parent.exists() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::DestinationParentMissing),
            Some(format!("{} does not exist.", parent.display())),
        );
    }

    let destination_path = final_destination.display().to_string();
    (
        CommandKind::Filesystem,
        vec![ResolvedRoute {
            label: format!("Move {}", source_pb.display()),
            description: format!("Move {} to {}", source_pb.display(), destination_path),
            action: ResolvedAction::MovePath {
                source_path,
                destination_path,
            },
        }],
        None,
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

fn find_installed_app(machine: &MachineInfo, query: &str) -> Option<(String, String)> {
    let canonical = canonical_app_name(query);

    for app in &machine.installed_apps {
        if canonical_app_name(&app.name) == canonical {
            return Some((app.name.clone(), app.bundle_id.clone()));
        }
    }

    for browser in &machine.installed_browsers {
        if canonical_app_name(&browser.name) == canonical {
            return Some((browser.name.clone(), browser.bundle_id.clone()));
        }
    }

    None
}

fn find_installed_browser(machine: &MachineInfo, query: &str) -> Option<(String, String)> {
    let canonical = canonical_app_name(query);
    machine
        .installed_browsers
        .iter()
        .find(|browser| canonical_app_name(&browser.name) == canonical)
        .map(|browser| (browser.name.clone(), browser.bundle_id.clone()))
}

fn canonical_app_name(name: &str) -> String {
    match name.trim().to_lowercase().as_str() {
        "chrome" | "google chrome" => "google chrome".to_string(),
        "safari" => "safari".to_string(),
        "firefox" => "firefox".to_string(),
        "brave" | "brave browser" => "brave".to_string(),
        "arc" => "arc".to_string(),
        "finder" => "finder".to_string(),
        "slack" => "slack".to_string(),
        other => other.to_string(),
    }
}

fn display_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn resolve_base_path(machine: &MachineInfo, base: &str) -> Option<String> {
    let home = if machine.home_dir.is_empty() {
        dirs::home_dir().map(|p| p.display().to_string())?
    } else {
        machine.home_dir.clone()
    };

    match base.trim().to_lowercase().as_str() {
        "home" | "~" => Some(home),
        "desktop" => Some(format!("{}/Desktop", home)),
        "downloads" => Some(format!("{}/Downloads", home)),
        "documents" => Some(format!("{}/Documents", home)),
        other => Some(expand_user_path(other)),
    }
}

fn expand_user_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home = home.display().to_string();
        match path.trim() {
            "home" | "~" => home,
            "desktop" => format!("{}/Desktop", home),
            "downloads" => format!("{}/Downloads", home),
            "documents" => format!("{}/Documents", home),
            other if other.starts_with("~/") => format!("{}{}", home, &other[1..]),
            other => other.to_string(),
        }
    } else {
        path.trim().to_string()
    }
}

fn expand_user_path_with_machine(machine: &MachineInfo, path: &str) -> String {
    if machine.home_dir.is_empty() {
        expand_user_path(path)
    } else {
        match path.trim() {
            "home" | "~" => machine.home_dir.clone(),
            "desktop" => format!("{}/Desktop", machine.home_dir),
            "downloads" => format!("{}/Downloads", machine.home_dir),
            "documents" => format!("{}/Documents", machine.home_dir),
            other if other.starts_with("~/") => format!("{}{}", machine.home_dir, &other[1..]),
            other => other.to_string(),
        }
    }
}
