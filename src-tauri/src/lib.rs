use std::collections::HashMap;
use std::sync::Mutex;

use models::{HistoryEntry, MachineInfo, ParsedCommand};

pub mod applescript;
pub mod arbiter;
pub mod commands;
pub mod config;
pub mod errors;
pub mod events;
pub mod executor;
pub mod history;
pub mod intent_ontology;
pub mod interpret_commands;
pub mod interpret_local;
pub mod machine;
pub mod models;
pub mod parser;
pub mod permissions;
pub mod planner;
pub mod provider_keys;
pub mod resolver;
pub mod risk;
pub mod ui_automation;
pub mod validator;

pub const APP_CONFIG_MAX_HISTORY: usize = 500;

pub struct AppState {
    pub inner: Mutex<AppStateInner>,
}

pub struct AppStateInner {
    pub machine_info: Option<MachineInfo>,
    pub pending_commands: HashMap<String, ParsedCommand>,
    pub history: Vec<HistoryEntry>,
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

    tauri::Builder::default()
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
                let _ = window.set_always_on_top(cfg.always_on_top);
            }

            app.global_shortcut()
                .register("CommandOrControl+Shift+Space")
                .unwrap_or_else(|e| log::warn!("Could not register global shortcut: {e}"));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::parse_command,
            commands::execute_command,
            commands::approve_command,
            commands::deny_command,
            commands::get_machine_info,
            commands::get_permission_status,
            commands::get_history,
            commands::undo_last,
            commands::set_window_mode,
            commands::toggle_always_on_top,
            commands::refresh_machine_info,
            commands::get_provider_key_status,
            commands::set_provider_key,
            commands::delete_provider_key,
            interpret_commands::debug_interpret_local,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
