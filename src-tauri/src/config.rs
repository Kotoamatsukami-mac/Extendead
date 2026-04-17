use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Whether the window starts as always-on-top.
    pub always_on_top: bool,
    /// Maximum history entries to retain.
    pub max_history: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            always_on_top: true,
            max_history: 500,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("extendead").join("config.json"))
}

pub fn load_config() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig::default();
    };
    let Ok(bytes) = fs::read(&path) else {
        return AppConfig::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save_config(config: &AppConfig) -> Result<(), AppError> {
    let path = config_path().ok_or_else(|| AppError::IoError("no config dir".to_string()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;
    fs::write(&path, json)?;
    Ok(())
}
