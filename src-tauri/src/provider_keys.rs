/// Provider key service — Phase 1 scaffold.
///
/// On macOS, provider credentials (e.g. OpenAI API keys) must be stored in the
/// system keychain and never passed back to the frontend in plain text.
///
/// Phase 1 implements the interface using the macOS `security` CLI. A later
/// phase will replace this with a direct Security framework binding via unsafe
/// FFI to eliminate the CLI subprocess.
///
/// The frontend always receives a `ProviderKeyStatus` (masked) — never the raw key.
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// The keychain service name used for all Extendead entries.
/// Referenced only by macOS-specific functions; suppress the dead_code warning
/// on non-macOS targets where those functions are compiled out.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const KEYCHAIN_SERVICE: &str = "com.extendead.app";

/// Masked status returned to the frontend for a given provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    /// A key is stored and accessible.
    Set,
    /// No key is stored for this provider.
    NotSet,
    /// A key exists but keychain access was denied.
    AccessDenied,
}

/// Masked representation of a stored provider key — safe to send to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderKeyStatus {
    pub provider: String,
    pub status: KeyStatus,
}

/// Store a provider key in the system keychain.
/// The key string must never leave the Rust layer.
pub fn store_key(provider: &str, key: &str) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        store_key_macos(provider, key)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (provider, key);
        Err(AppError::PlatformNotSupported(
            "Keychain storage requires macOS".to_string(),
        ))
    }
}

/// Delete a provider key from the system keychain.
pub fn delete_key(provider: &str) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        delete_key_macos(provider)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = provider;
        Err(AppError::PlatformNotSupported(
            "Keychain storage requires macOS".to_string(),
        ))
    }
}

/// Return the masked status for a provider key. Safe to expose to the frontend.
pub fn key_status(provider: &str) -> ProviderKeyStatus {
    #[cfg(target_os = "macos")]
    {
        key_status_macos(provider)
    }
    #[cfg(not(target_os = "macos"))]
    {
        ProviderKeyStatus {
            provider: provider.to_string(),
            status: KeyStatus::NotSet,
        }
    }
}

/// True when a usable key is already linked for the provider.
pub fn is_provider_configured(provider: &str) -> bool {
    matches!(key_status(provider).status, KeyStatus::Set)
}

/// Retrieve a provider key for internal Rust use only.
///
/// This function must **never** be called from a Tauri command handler.
/// It is `pub(crate)` so the compiler enforces this boundary.
/// Marked `allow(dead_code)` because callers live in later phases (Phase 2
/// planner will call this when making AI API requests).
#[allow(dead_code)]
pub(crate) fn retrieve_key(provider: &str) -> Result<String, AppError> {
    #[cfg(target_os = "macos")]
    {
        retrieve_key_macos(provider)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = provider;
        Err(AppError::PlatformNotSupported(
            "Keychain storage requires macOS".to_string(),
        ))
    }
}

// ── macOS keychain implementation via `security` CLI ─────────────────────────
//
// The `security` binary is Apple-provided. It is used here as a safe Phase 1
// bootstrap. A Phase 2 upgrade will use Security.framework directly via FFI.
// The CLI is executed with only the minimum required arguments; no user-
// supplied data is passed through shell interpolation.

#[cfg(target_os = "macos")]
fn store_key_macos(provider: &str, key: &str) -> Result<(), AppError> {
    // `security add-generic-password -U` upserts (adds or updates).
    let status = std::process::Command::new("security")
        .args([
            "add-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            provider,
            "-w",
            key,
            "-U",
        ])
        .status()
        .map_err(|e| AppError::ExecutionError(format!("security CLI error: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::ExecutionError(
            "Keychain write denied — check system keychain access".to_string(),
        ))
    }
}

#[cfg(target_os = "macos")]
fn delete_key_macos(provider: &str) -> Result<(), AppError> {
    let status = std::process::Command::new("security")
        .args([
            "delete-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            provider,
        ])
        .status()
        .map_err(|e| AppError::ExecutionError(format!("security CLI error: {e}")))?;

    // Exit code 44 means "item not found" — treat as already deleted.
    if status.success() || status.code() == Some(44) {
        Ok(())
    } else {
        Err(AppError::ExecutionError(
            "Keychain delete denied — check system keychain access".to_string(),
        ))
    }
}

#[cfg(target_os = "macos")]
fn key_status_macos(provider: &str) -> ProviderKeyStatus {
    let result = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            provider,
        ])
        .output();

    let status = match result {
        Ok(o) if o.status.success() => KeyStatus::Set,
        Ok(o) if o.status.code() == Some(44) => KeyStatus::NotSet,
        Ok(_) => KeyStatus::AccessDenied,
        Err(_) => KeyStatus::AccessDenied,
    };

    ProviderKeyStatus {
        provider: provider.to_string(),
        status,
    }
}

#[cfg(target_os = "macos")]
fn retrieve_key_macos(provider: &str) -> Result<String, AppError> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            provider,
            "-w", // output only the password value
        ])
        .output()
        .map_err(|e| AppError::ExecutionError(format!("security CLI error: {e}")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else if output.status.code() == Some(44) {
        Err(AppError::ProviderNotConfigured(provider.to_string()))
    } else {
        Err(AppError::ExecutionError(
            "Keychain read denied — check system keychain access".to_string(),
        ))
    }
}
