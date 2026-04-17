use std::path::Path;

use crate::models::{BrowserInfo, MachineInfo};

/// Known browsers to scan for on macOS.
static BROWSER_CANDIDATES: &[(&str, &str, &str)] = &[
    ("Google Chrome", "com.google.Chrome", "/Applications/Google Chrome.app"),
    ("Safari", "com.apple.Safari", "/Applications/Safari.app"),
    ("Firefox", "org.mozilla.firefox", "/Applications/Firefox.app"),
    ("Brave", "com.brave.Browser", "/Applications/Brave Browser.app"),
    ("Arc", "company.thebrowser.Browser", "/Applications/Arc.app"),
];

/// Scan for installed browsers by checking well-known application paths.
pub fn scan_browsers() -> Vec<BrowserInfo> {
    BROWSER_CANDIDATES
        .iter()
        .filter_map(|(name, bundle_id, path)| {
            if Path::new(path).exists() {
                Some(BrowserInfo {
                    name: name.to_string(),
                    bundle_id: bundle_id.to_string(),
                    path: path.to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Collect hostname, username, home dir, and installed browsers.
pub fn scan_machine() -> MachineInfo {
    let hostname = hostname();
    let username = username();
    let home_dir = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~".to_string());
    let installed_browsers = scan_browsers();

    MachineInfo {
        hostname,
        username,
        installed_browsers,
        home_dir,
    }
}

fn hostname() -> String {
    #[cfg(target_os = "macos")]
    {
        read_command("hostname", &[])
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
    }
}

fn username() -> String {
    #[cfg(target_os = "macos")]
    {
        read_command("whoami", &[])
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

/// Run a trusted read-only command and return trimmed stdout.
#[cfg(target_os = "macos")]
fn read_command(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
