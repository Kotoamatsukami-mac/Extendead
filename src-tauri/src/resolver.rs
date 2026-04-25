use std::path::{Path, PathBuf};

use crate::models::{
    AppInfo, BrowserInfo, CommandKind, MachineInfo, ResolvedAction, ResolvedPlanStep,
    ResolvedRoute, RiskLevel, UnresolvedCode,
};
use crate::parser::{Intent, VolumeDirection};
use crate::service_catalog;

type ResolveResult = (
    CommandKind,
    Vec<ResolvedRoute>,
    Option<UnresolvedCode>,
    Option<String>,
);

pub fn resolve(intent: &Intent, machine: &MachineInfo) -> ResolveResult {
    match intent {
        Intent::OpenTarget(name) => resolve_target_named(machine, name, TargetOperation::Open),
        Intent::OpenService(service_id) => resolve_service(machine, service_id),
        Intent::OpenServiceInBrowser {
            service_id,
            browser,
        } => resolve_service_in_named_browser(machine, service_id, browser),
        Intent::CloseTarget(name) => resolve_target_named(machine, name, TargetOperation::Close),
        Intent::HideTarget(name) => resolve_target_named(machine, name, TargetOperation::Hide),
        Intent::ForceQuitTarget(name) => {
            resolve_target_named(machine, name, TargetOperation::ForceQuit)
        }
        Intent::BrowserNewTab { browser } => resolve_browser_tab_shortcut(
            machine,
            browser.as_deref(),
            "browser_new_tab",
            "t",
            &["command down"],
            "Open new tab",
        ),
        Intent::BrowserCloseTab { browser } => resolve_browser_tab_shortcut(
            machine,
            browser.as_deref(),
            "browser_close_tab",
            "w",
            &["command down"],
            "Close tab",
        ),
        Intent::BrowserReopenClosedTab { browser } => resolve_browser_tab_shortcut(
            machine,
            browser.as_deref(),
            "browser_reopen_closed_tab",
            "t",
            &["command down", "shift down"],
            "Reopen closed tab",
        ),
        Intent::OpenPath(path) => resolve_open_path(path),
        Intent::CreateFolder { name, base } => {
            resolve_create_folder(machine, name, base.as_deref())
        }
        Intent::MovePath {
            source,
            destination,
        } => resolve_move_path(machine, source, destination),
        Intent::RunMode(mode) => resolve_mode(machine, mode),
        Intent::TrashPath(path) => resolve_trash_path(machine, path),
        Intent::DeletePathPermanently(_) => (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::PermanentDeleteBlocked),
            Some("Permanent delete is disabled. Use trash <path> instead.".to_string()),
        ),
        Intent::MuteVolume => {
            let (kind, routes) = resolve_mute();
            (kind, routes, None, None)
        }
        Intent::IncreaseBrightness => resolve_brightness("brightness_up", "Increase brightness"),
        Intent::DecreaseBrightness => resolve_brightness("brightness_down", "Decrease brightness"),
        Intent::SetVolume(level) => {
            let (kind, routes) = resolve_set_volume(*level);
            (kind, routes, None, None)
        }
        Intent::AdjustVolume {
            direction,
            intensity,
        } => {
            let (kind, routes) = resolve_adjust_volume(*direction, *intensity);
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

fn resolve_service_id(
    browsers: &[BrowserInfo],
    service_id: &str,
) -> (CommandKind, Vec<ResolvedRoute>) {
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
            Some(format!(
                "{} is not installed on this Mac.",
                display_name(browser_query)
            )),
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

fn resolve_browser_tab_shortcut(
    machine: &MachineInfo,
    browser_query: Option<&str>,
    template_id: &str,
    key: &str,
    modifiers: &[&str],
    label_prefix: &str,
) -> ResolveResult {
    if machine.installed_browsers.is_empty() {
        return (
            CommandKind::MixedWorkflow,
            vec![],
            Some(UnresolvedCode::BrowserNotInstalled),
            Some("No supported browser is installed on this Mac.".to_string()),
        );
    }

    if let Some(browser_query) = browser_query {
        let Some((browser_name, bundle_id)) = find_installed_browser(machine, browser_query) else {
            return (
                CommandKind::MixedWorkflow,
                vec![],
                Some(UnresolvedCode::BrowserNotInstalled),
                Some(format!(
                    "{} is not installed on this Mac.",
                    display_name(browser_query)
                )),
            );
        };

        return (
            CommandKind::MixedWorkflow,
            vec![ResolvedRoute {
                label: format!("{label_prefix} in {browser_name}"),
                description: format!("{label_prefix} in {browser_name}"),
                action: ResolvedAction::AppleScriptTemplate {
                    script: browser_shortcut_script(&bundle_id, key, modifiers),
                    template_id: template_id.to_string(),
                },
            }],
            None,
            None,
        );
    }

    let routes = machine
        .installed_browsers
        .iter()
        .map(|browser| ResolvedRoute {
            label: format!("{label_prefix} in {}", browser.name),
            description: format!("{label_prefix} in {}", browser.name),
            action: ResolvedAction::AppleScriptTemplate {
                script: browser_shortcut_script(&browser.bundle_id, key, modifiers),
                template_id: template_id.to_string(),
            },
        })
        .collect::<Vec<_>>();

    (CommandKind::MixedWorkflow, routes, None, None)
}

fn browser_shortcut_script(bundle_id: &str, key: &str, modifiers: &[&str]) -> String {
    let modifiers_clause = if modifiers.is_empty() {
        String::new()
    } else {
        format!(" using {{{}}}", modifiers.join(", "))
    };

    format!(
        "tell application id \"{bundle_id}\" to activate\ntell application \"System Events\" to keystroke \"{key}\"{modifiers_clause}"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetOperation {
    Open,
    Close,
    Hide,
    ForceQuit,
}

#[derive(Debug, Clone)]
struct AppMatch {
    name: String,
    bundle_id: String,
    score: f32,
}

fn resolve_target_named(
    machine: &MachineInfo,
    target_name: &str,
    operation: TargetOperation,
) -> ResolveResult {
    let query = target_name.trim();
    let matches = find_installed_app_matches(machine, query);

    if matches.is_empty() {
        return (
            CommandKind::AppControl,
            vec![],
            Some(UnresolvedCode::AppNotInstalled),
            Some(format!(
                "{} is not an installed app on this Mac.",
                display_name(query)
            )),
        );
    };

    let best = &matches[0];
    let ambiguous = matches
        .get(1)
        .map(|second| best.score < 0.86 || (best.score - second.score) < 0.08)
        .unwrap_or(best.score < 0.78);
    if ambiguous {
        let choices = matches
            .iter()
            .take(3)
            .map(|m| m.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        return (
            CommandKind::AppControl,
            vec![],
            Some(UnresolvedCode::AmbiguousTarget),
            Some(format!(
                "I found multiple possible apps for '{}': {}.",
                query, choices
            )),
        );
    }

    let route = route_for_app(best.name.clone(), best.bundle_id.clone(), operation);
    (CommandKind::AppControl, vec![route], None, None)
}

fn route_for_app(name: String, bundle_id: String, operation: TargetOperation) -> ResolvedRoute {
    match operation {
        TargetOperation::Open => ResolvedRoute {
            label: format!("Open {name}"),
            description: format!("Launch {name}.app"),
            action: ResolvedAction::OpenApp {
                bundle_id,
                app_name: name,
            },
        },
        TargetOperation::Close => ResolvedRoute {
            label: format!("Close {name}"),
            description: format!("Quit {name}"),
            action: ResolvedAction::QuitApp {
                bundle_id,
                app_name: name,
            },
        },
        TargetOperation::Hide => ResolvedRoute {
            label: format!("Hide {name}"),
            description: format!("Hide {name} without quitting"),
            action: ResolvedAction::HideApp {
                bundle_id,
                app_name: name,
            },
        },
        TargetOperation::ForceQuit => ResolvedRoute {
            label: format!("Force Quit {name}"),
            description: format!("Force quit {name}"),
            action: ResolvedAction::ForceQuitApp {
                bundle_id,
                app_name: name,
            },
        },
    }
}

fn resolve_open_path(path: &str) -> ResolveResult {
    let expanded = expand_user_path(path);
    if !Path::new(&expanded).is_absolute() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::PathNotFound),
            Some("Path must be absolute or use ~/ alias.".to_string()),
        );
    }
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

fn resolve_create_folder(machine: &MachineInfo, name: &str, base: Option<&str>) -> ResolveResult {
    let base_path = resolve_base_path(machine, base.unwrap_or("home"));
    let Some(base_path) = base_path else {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::BasePathUnresolved),
            Some("I could not resolve where to create that folder.".to_string()),
        );
    };
    if !Path::new(&base_path).is_absolute() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::BasePathUnresolved),
            Some("Base path must resolve inside your home directory.".to_string()),
        );
    }

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
    if !source_pb.is_absolute() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::SourcePathNotFound),
            Some("Source path must be absolute or use ~/ alias.".to_string()),
        );
    }
    if !source_pb.exists() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::SourcePathNotFound),
            Some(format!("{} does not exist.", source_path)),
        );
    }

    let destination_base = expand_user_path_with_machine(machine, destination);
    let destination_pb = PathBuf::from(&destination_base);
    if !destination_pb.is_absolute() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::DestinationPathUnresolved),
            Some("Destination path must be absolute or use ~/ alias.".to_string()),
        );
    }
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

fn resolve_trash_path(machine: &MachineInfo, path: &str) -> ResolveResult {
    let source_path = expand_user_path_with_machine(machine, path);
    let source = PathBuf::from(&source_path);
    if !source.is_absolute() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::SourcePathNotFound),
            Some("Source path must be absolute or use ~/ alias.".to_string()),
        );
    }
    if !source.exists() {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::SourcePathNotFound),
            Some(format!("{} does not exist.", source_path)),
        );
    }

    let Some(file_name) = source.file_name().map(|name| name.to_os_string()) else {
        return (
            CommandKind::Filesystem,
            vec![],
            Some(UnresolvedCode::DestinationPathUnresolved),
            Some("I could not resolve the Trash destination.".to_string()),
        );
    };

    let home = if machine.home_dir.is_empty() {
        dirs::home_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~".to_string())
    } else {
        machine.home_dir.clone()
    };
    let trash_dir = PathBuf::from(home).join(".Trash");
    let destination_path = next_available_destination(trash_dir.join(file_name));

    (
        CommandKind::Filesystem,
        vec![ResolvedRoute {
            label: format!("Move {} to Trash", source.display()),
            description: format!(
                "Move {} to {}",
                source.display(),
                destination_path.display()
            ),
            action: ResolvedAction::MovePath {
                source_path,
                destination_path: destination_path.display().to_string(),
            },
        }],
        None,
        None,
    )
}

fn next_available_destination(base: PathBuf) -> PathBuf {
    if !base.exists() {
        return base;
    }

    let stem = base
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("item")
        .to_string();
    let ext = base.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = base
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    for i in 1..=9999 {
        let candidate_name = if ext.is_empty() {
            format!("{stem} {i}")
        } else {
            format!("{stem} {i}.{ext}")
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    let ts = chrono::Utc::now().timestamp_millis();
    if ext.is_empty() {
        parent.join(format!("{stem}-{ts}"))
    } else {
        parent.join(format!("{stem}-{ts}.{ext}"))
    }
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

fn resolve_adjust_volume(
    direction: VolumeDirection,
    intensity: u8,
) -> (CommandKind, Vec<ResolvedRoute>) {
    let delta = intensity.clamp(1, 25);
    let script = match direction {
        VolumeDirection::Up => format!(
            "set currentVolume to output volume of (get volume settings)\nset volume output volume (currentVolume + {delta})"
        ),
        VolumeDirection::Down => format!(
            "set currentVolume to output volume of (get volume settings)\nset volume output volume (currentVolume - {delta})"
        ),
    };
    let label = match direction {
        VolumeDirection::Up => "Make Mac louder",
        VolumeDirection::Down => "Make Mac quieter",
    };
    let routes = vec![ResolvedRoute {
        label: label.to_string(),
        description: format!("Adjust system output volume by {delta}%"),
        action: ResolvedAction::AppleScriptTemplate {
            script,
            template_id: "adjust_volume".to_string(),
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

fn resolve_mode(machine: &MachineInfo, mode: &str) -> ResolveResult {
    let canonical = mode.trim().to_lowercase();
    let Some(mode_def) = mode_definition(&canonical) else {
        return (
            CommandKind::MixedWorkflow,
            vec![],
            Some(UnresolvedCode::UnsupportedCommand),
            Some(format!("No ready mode named '{}' exists yet.", mode.trim())),
        );
    };

    let mut steps = Vec::new();
    for app_query in mode_def.open_apps {
        if let Some(app) = find_installed_app_matches(machine, app_query).first() {
            if app.score >= 0.78 {
                steps.push(ResolvedPlanStep {
                    label: format!("Open {}", app.name),
                    description: format!("Launch {}", app.name),
                    action: Box::new(ResolvedAction::OpenApp {
                        bundle_id: app.bundle_id.clone(),
                        app_name: app.name.clone(),
                    }),
                    execution_group: "parallel:setup".to_string(),
                    risk: RiskLevel::R0,
                    requires_approval: false,
                });
            }
        }
    }

    if let Some(volume) = mode_def.volume {
        steps.push(ResolvedPlanStep {
            label: format!("Set volume to {volume}%"),
            description: "Adjust system audio for this mode".to_string(),
            action: Box::new(ResolvedAction::AppleScriptTemplate {
                script: format!("set volume output volume {volume}"),
                template_id: "set_volume".to_string(),
            }),
            execution_group: "parallel:setup".to_string(),
            risk: RiskLevel::R1,
            requires_approval: false,
        });
    }

    if steps.is_empty() {
        return (
            CommandKind::MixedWorkflow,
            vec![],
            Some(UnresolvedCode::AppNotInstalled),
            Some(format!(
                "{} mode has no available app steps on this Mac.",
                mode_def.display_name
            )),
        );
    }

    (
        CommandKind::MixedWorkflow,
        vec![ResolvedRoute {
            label: format!("Run {} Mode", mode_def.display_name),
            description: format!("Run {} coordinated steps", steps.len()),
            action: ResolvedAction::RunPlan {
                mode_name: mode_def.display_name.to_string(),
                steps,
            },
        }],
        None,
        None,
    )
}

struct ModeDefinition {
    display_name: &'static str,
    aliases: &'static [&'static str],
    open_apps: &'static [&'static str],
    volume: Option<u8>,
}

static MODE_DEFINITIONS: &[ModeDefinition] = &[
    ModeDefinition {
        display_name: "Study",
        aliases: &["study", "focus study"],
        open_apps: &["Notes", "Safari", "Visual Studio Code", "Finder"],
        volume: Some(25),
    },
    ModeDefinition {
        display_name: "Focus",
        aliases: &["focus", "work", "deep work"],
        open_apps: &["Visual Studio Code", "Notes", "Finder"],
        volume: Some(15),
    },
    ModeDefinition {
        display_name: "Break",
        aliases: &["break", "rest", "chill"],
        open_apps: &["Spotify", "Music", "Prime Video", "Safari"],
        volume: Some(40),
    },
];

fn mode_definition(mode: &str) -> Option<&'static ModeDefinition> {
    MODE_DEFINITIONS
        .iter()
        .find(|candidate| candidate.aliases.contains(&mode))
}

fn resolve_brightness(template_id: &str, label: &str) -> ResolveResult {
    (
        CommandKind::LocalSystem,
        vec![ResolvedRoute {
            label: label.to_string(),
            description: label.to_string(),
            action: ResolvedAction::AppleScriptTemplate {
                script: match template_id {
                    "brightness_up" => {
                        "tell application \"System Events\" to key code 144".to_string()
                    }
                    "brightness_down" => {
                        "tell application \"System Events\" to key code 145".to_string()
                    }
                    _ => String::new(),
                },
                template_id: template_id.to_string(),
            },
        }],
        None,
        None,
    )
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

fn find_installed_browser(machine: &MachineInfo, query: &str) -> Option<(String, String)> {
    let mut matches = machine
        .installed_browsers
        .iter()
        .flat_map(|browser| {
            app_aliases(&browser.name)
                .into_iter()
                .map(move |alias| (browser, fuzzy_score(query, &alias)))
        })
        .filter(|(_, score)| *score >= 0.78)
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let (browser, score) = matches.first()?;
    let ambiguous = matches
        .get(1)
        .map(|(_, second_score)| (*score - *second_score) < 0.08)
        .unwrap_or(false);
    if ambiguous {
        None
    } else {
        Some((browser.name.clone(), browser.bundle_id.clone()))
    }
}

fn find_installed_app_matches(machine: &MachineInfo, query: &str) -> Vec<AppMatch> {
    let mut matches = all_installed_apps(machine)
        .into_iter()
        .filter_map(|app| {
            let score = app_aliases(&app.name)
                .into_iter()
                .map(|alias| fuzzy_score(query, &alias))
                .fold(0.0_f32, f32::max);
            (score >= 0.70).then_some(AppMatch {
                name: app.name,
                bundle_id: app.bundle_id,
                score,
            })
        })
        .collect::<Vec<_>>();

    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    matches
}

fn all_installed_apps(machine: &MachineInfo) -> Vec<AppInfo> {
    let mut apps = machine.installed_apps.clone();
    apps.extend(machine.installed_browsers.iter().map(|browser| AppInfo {
        name: browser.name.clone(),
        bundle_id: browser.bundle_id.clone(),
        path: browser.path.clone(),
    }));
    apps
}

fn app_aliases(name: &str) -> Vec<String> {
    let normalized = parser_normalize_name(name);
    let mut aliases = vec![normalized.clone()];
    if let Some(stripped) = normalized.strip_suffix(" browser") {
        aliases.push(stripped.to_string());
    }
    if normalized == "google chrome" {
        aliases.push("chrome".to_string());
    }
    if normalized == "visual studio code" {
        aliases.push("vscode".to_string());
        aliases.push("code".to_string());
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

fn parser_normalize_name(name: &str) -> String {
    crate::parser::normalize(name)
        .trim_end_matches(".app")
        .trim()
        .to_string()
}

fn fuzzy_score(query: &str, candidate: &str) -> f32 {
    let q = parser_normalize_name(query);
    let c = parser_normalize_name(candidate);
    if q.is_empty() || c.is_empty() {
        return 0.0;
    }
    if q == c {
        return 1.0;
    }
    if c.starts_with(&q) {
        return 0.94 - ((c.len().saturating_sub(q.len())) as f32 * 0.01).min(0.12);
    }
    if c.contains(&q) {
        return 0.84;
    }

    let distance = edit_distance(&q, &c);
    let max_len = q.chars().count().max(c.chars().count()) as f32;
    (1.0 - (distance as f32 / max_len)).clamp(0.0, 1.0)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a = a.chars().collect::<Vec<_>>();
    let b = b.chars().collect::<Vec<_>>();
    let mut prev = (0..=b.len()).collect::<Vec<_>>();
    let mut curr = vec![0; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Intent;

    fn test_machine() -> MachineInfo {
        MachineInfo {
            hostname: "test".to_string(),
            username: "test".to_string(),
            os_version: "14".to_string(),
            architecture: "arm64".to_string(),
            installed_browsers: vec![BrowserInfo {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
                path: "/Applications/Safari.app".to_string(),
            }],
            installed_apps: vec![],
            home_dir: "/Users/test".to_string(),
        }
    }

    fn app_machine() -> MachineInfo {
        MachineInfo {
            hostname: "test".to_string(),
            username: "test".to_string(),
            os_version: "14".to_string(),
            architecture: "arm64".to_string(),
            installed_browsers: vec![BrowserInfo {
                name: "Example Browser".to_string(),
                bundle_id: "com.example.Browser".to_string(),
                path: "/Applications/Example Browser.app".to_string(),
            }],
            installed_apps: vec![
                AppInfo {
                    name: "Signal".to_string(),
                    bundle_id: "org.signal.Signal".to_string(),
                    path: "/Applications/Signal.app".to_string(),
                },
                AppInfo {
                    name: "Notes Plus".to_string(),
                    bundle_id: "com.example.NotesPlus".to_string(),
                    path: "/Applications/Notes Plus.app".to_string(),
                },
            ],
            home_dir: "/Users/test".to_string(),
        }
    }

    #[test]
    fn open_operator_family_resolves_to_same_app_action() {
        let machine = app_machine();
        for synonym in ["open", "launch", "start", "run"] {
            let intent = crate::parser::parse_intent(&format!("{synonym} Signal"));
            let (_kind, routes, unresolved_code, _msg) = resolve(&intent, &machine);
            assert!(unresolved_code.is_none(), "{synonym} should resolve");
            assert_eq!(routes.len(), 1);
            match &routes[0].action {
                ResolvedAction::OpenApp {
                    bundle_id,
                    app_name,
                } => {
                    assert_eq!(bundle_id, "org.signal.Signal");
                    assert_eq!(app_name, "Signal");
                }
                other => panic!("expected open app route, got {other:?}"),
            }
        }

        let (_kind, routes, unresolved_code, _msg) =
            resolve(&crate::parser::parse_intent("Signal"), &machine);
        assert!(unresolved_code.is_none());
        assert!(matches!(routes[0].action, ResolvedAction::OpenApp { .. }));
    }

    #[test]
    fn one_edit_app_typos_resolve_only_when_unique() {
        let machine = app_machine();
        for typo in one_edit_variants("Signal").into_iter().take(12) {
            let (_kind, routes, unresolved_code, _msg) =
                resolve(&Intent::OpenTarget(typo), &machine);
            assert!(unresolved_code.is_none());
            match &routes[0].action {
                ResolvedAction::OpenApp { bundle_id, .. } => {
                    assert_eq!(bundle_id, "org.signal.Signal");
                }
                other => panic!("expected open app route, got {other:?}"),
            }
        }
    }

    #[test]
    fn ambiguous_app_matches_request_clarification_instead_of_guessing() {
        let mut machine = app_machine();
        machine.installed_apps.push(AppInfo {
            name: "Signal Beta".to_string(),
            bundle_id: "org.signal.Beta".to_string(),
            path: "/Applications/Signal Beta.app".to_string(),
        });

        let (_kind, routes, unresolved_code, _msg) =
            resolve(&Intent::OpenTarget("Sig".to_string()), &machine);
        assert!(routes.is_empty());
        assert_eq!(unresolved_code, Some(UnresolvedCode::AmbiguousTarget));
    }

    #[test]
    fn volume_family_normalizes_set_and_relative_actions() {
        for n in [0, 1, 25, 50, 99, 100] {
            let intent = crate::parser::parse_intent(&format!("set volume to {n}"));
            let (_kind, routes, unresolved_code, _msg) = resolve(&intent, &app_machine());
            assert!(unresolved_code.is_none());
            assert!(matches!(
                routes[0].action,
                ResolvedAction::AppleScriptTemplate { ref template_id, .. } if template_id == "set_volume"
            ));
        }

        let (_kind, routes, unresolved_code, _msg) =
            resolve(&crate::parser::parse_intent("quieter"), &app_machine());
        assert!(unresolved_code.is_none());
        assert!(matches!(
            routes[0].action,
            ResolvedAction::AppleScriptTemplate { ref template_id, .. } if template_id == "adjust_volume"
        ));
    }

    #[test]
    fn mode_activation_is_one_route_with_step_plan() {
        let machine = app_machine();
        let (_kind, routes, unresolved_code, _msg) =
            resolve(&crate::parser::parse_intent("study mode"), &machine);
        assert!(unresolved_code.is_none());
        assert_eq!(routes.len(), 1);
        match &routes[0].action {
            ResolvedAction::RunPlan { steps, .. } => {
                assert!(!steps.is_empty());
                assert!(steps
                    .iter()
                    .any(|step| step.execution_group.starts_with("parallel")));
            }
            other => panic!("expected plan route, got {other:?}"),
        }
    }

    #[test]
    fn browser_new_tab_generates_route() {
        let machine = test_machine();
        let (_kind, routes, unresolved_code, _msg) =
            resolve(&Intent::BrowserNewTab { browser: None }, &machine);
        assert!(unresolved_code.is_none());
        assert_eq!(routes.len(), 1);
        assert!(matches!(
            routes[0].action,
            ResolvedAction::AppleScriptTemplate { .. }
        ));
    }

    #[test]
    fn permanent_delete_is_blocked() {
        let machine = test_machine();
        let (_kind, routes, unresolved_code, msg) = resolve(
            &Intent::DeletePathPermanently("~/Desktop/test.txt".to_string()),
            &machine,
        );
        assert!(routes.is_empty());
        assert_eq!(
            unresolved_code,
            Some(UnresolvedCode::PermanentDeleteBlocked)
        );
        assert!(msg.unwrap_or_default().contains("Permanent delete"));
    }

    fn one_edit_variants(input: &str) -> Vec<String> {
        let chars = input.chars().collect::<Vec<_>>();
        let mut variants = Vec::new();
        for i in 0..chars.len() {
            let mut delete = chars.clone();
            delete.remove(i);
            variants.push(delete.iter().collect());

            let mut substitute = chars.clone();
            substitute[i] = 'x';
            variants.push(substitute.iter().collect());
        }
        for i in 0..chars.len().saturating_sub(1) {
            let mut transpose = chars.clone();
            transpose.swap(i, i + 1);
            variants.push(transpose.iter().collect());
        }
        variants
    }
}
