use std::fs;
use std::path::PathBuf;

use crate::errors::AppError;
use crate::models::HistoryEntry;

fn history_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("extendead").join("history.json"))
}

/// Load history from disk. Returns empty vec on error or missing file.
pub fn load_history() -> Vec<HistoryEntry> {
    let Some(path) = history_path() else {
        return vec![];
    };
    let Ok(bytes) = fs::read(&path) else {
        return vec![];
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Append a new history entry and persist to disk.
pub fn append_and_save(
    entries: &mut Vec<HistoryEntry>,
    entry: HistoryEntry,
    max: usize,
) -> Result<(), AppError> {
    entries.push(entry);
    if entries.len() > max {
        let drain_count = entries.len() - max;
        entries.drain(0..drain_count);
    }
    save_history(entries)
}

/// Persist the current history vector to disk.
pub fn save_history(entries: &[HistoryEntry]) -> Result<(), AppError> {
    let path = history_path().ok_or_else(|| AppError::IoError("no data dir".to_string()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(entries)?;
    fs::write(&path, json)?;
    Ok(())
}
