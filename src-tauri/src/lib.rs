use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use models::{HistoryEntry, MachineInfo, ParsedCommand};

pub mod applescript;
pub mod arbiter;
pub mod commands;
pub mod config;
pub mod errors;
pub mod events;
pub mod executor;
pub mod history;
pub mod intent_language;
pub mod intent_ontology;
pub mod interpret_commands;
pub mod interpret_local;
pub mod machine;
pub mod models;
pub mod parser;
pub mod path_policy;
pub mod permissions;
pub mod planner;
pub mod provider_interpreter;
pub mod provider_keys;
pub mod resolver;
pub mod risk;
pub mod semantic;
pub mod service_catalog;
// Future-facing stub. Keep isolated from active execution paths.
pub mod ui_automation;
pub mod validator;

pub const APP_CONFIG_MAX_HISTORY: usize = 500;

pub struct AppState {
    pub inner: Mutex<AppStateInner>,
}

pub struct PendingCommandEntry {
    pub command: ParsedCommand,
    pub created_at: DateTime<Utc>,
}

pub struct AppStateInner {
    pub machine_info: Option<MachineInfo>,
    pub pending_commands: HashMap<String, PendingCommandEntry>,
    pub history: Vec<HistoryEntry>,
}

#[cfg(all(test, debug_assertions))]
fn invoke_command_names() -> &'static [&'static str] {
    &[
        "parse_command",
        "suggest_commands",
        "execute_command",
        "approve_command",
        "deny_command",
        "get_machine_info",
        "get_permission_status",
        "get_app_config",
        "get_history",
        "get_service_catalog",
        "undo_last",
        "set_window_mode",
        "toggle_always_on_top",
        "refresh_machine_info",
        "get_provider_key_status",
        "set_provider_key",
        "delete_provider_key",
        "debug_interpret_local",
    ]
}

#[cfg(all(test, not(debug_assertions)))]
fn invoke_command_names() -> &'static [&'static str] {
    &[
        "parse_command",
        "suggest_commands",
        "execute_command",
        "approve_command",
        "deny_command",
        "get_machine_info",
        "get_permission_status",
        "get_app_config",
        "get_history",
        "get_service_catalog",
        "undo_last",
        "set_window_mode",
        "toggle_always_on_top",
        "refresh_machine_info",
        "get_provider_key_status",
        "set_provider_key",
        "delete_provider_key",
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let app_state = AppState {
        inner: Mutex::new(AppStateInner {
            machine_info: None,
            pending_commands: HashMap::new(),
            history: history::load_history(),
        }),
    };

    let builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(
                    |app: &tauri::AppHandle<tauri::Wry>,
                     _shortcut: &tauri_plugin_global_shortcut::Shortcut,
                     event: tauri_plugin_global_shortcut::ShortcutEvent| {
                        use tauri::Manager;
                        use tauri_plugin_global_shortcut::ShortcutState;
                        if event.state() == ShortcutState::Pressed {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    },
                )
                .build(),
        )
        .manage(app_state)
        .setup(|app| {
            use tauri::Manager;
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            let state = app.state::<AppState>();
            let info = machine::scan_machine();
            {
                let mut inner = state.inner.lock().expect("state lock on setup");
                inner.machine_info = Some(info);
            }

            let cfg = config::load_config();
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.set_decorations(false) {
                    log::warn!("Failed to disable decorations: {e}");
                }
                if let Err(e) = window.set_shadow(false) {
                    log::warn!("Failed to disable shadow: {e}");
                }
                if let Err(e) = window.set_always_on_top(cfg.always_on_top) {
                    log::warn!("Failed to apply always-on-top={}: {e}", cfg.always_on_top);
                }
            }

            app.global_shortcut()
                .register("CommandOrControl+Shift+Space")
                .unwrap_or_else(|e| log::warn!("Could not register global shortcut: {e}"));

            Ok(())
        });

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::parse_command,
        commands::suggest_commands,
        commands::execute_command,
        commands::approve_command,
        commands::deny_command,
        commands::get_machine_info,
        commands::get_permission_status,
        commands::get_app_config,
        commands::get_history,
        commands::get_service_catalog,
        commands::undo_last,
        commands::set_window_mode,
        commands::toggle_always_on_top,
        commands::refresh_machine_info,
        commands::get_provider_key_status,
        commands::set_provider_key,
        commands::delete_provider_key,
        interpret_commands::debug_interpret_local,
    ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::parse_command,
        commands::suggest_commands,
        commands::execute_command,
        commands::approve_command,
        commands::deny_command,
        commands::get_machine_info,
        commands::get_permission_status,
        commands::get_app_config,
        commands::get_history,
        commands::get_service_catalog,
        commands::undo_last,
        commands::set_window_mode,
        commands::toggle_always_on_top,
        commands::refresh_machine_info,
        commands::get_provider_key_status,
        commands::set_provider_key,
        commands::delete_provider_key,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::invoke_command_names;

    #[test]
    fn debug_interpret_exposure_matches_build_mode() {
        let has_debug_interpret = invoke_command_names().contains(&"debug_interpret_local");
        assert_eq!(has_debug_interpret, cfg!(debug_assertions));
    }
}
