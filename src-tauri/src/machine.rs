use std::path::Path;

use crate::models::{AppInfo, BrowserInfo, MachineInfo};

/// Known browsers to scan for on macOS.
static BROWSER_CANDIDATES: &[(&str, &str, &str)] = &[
    (
        "Google Chrome",
        "com.google.Chrome",
        "/Applications/Google Chrome.app",
    ),
    ("Safari", "com.apple.Safari", "/Applications/Safari.app"),
    (
        "Firefox",
        "org.mozilla.firefox",
        "/Applications/Firefox.app",
    ),
    (
        "Brave",
        "com.brave.Browser",
        "/Applications/Brave Browser.app",
    ),
    ("Arc", "company.thebrowser.Browser", "/Applications/Arc.app"),
];

/// Known applications (non-browser) to scan for on macOS.
static APP_CANDIDATES: &[(&str, &str, &str)] = &[
    (
        "Slack",
        "com.tinyspeck.slackmacgap",
        "/Applications/Slack.app",
    ),
    ("Zoom", "us.zoom.xos", "/Applications/zoom.us.app"),
    (
        "Visual Studio Code",
        "com.microsoft.VSCode",
        "/Applications/Visual Studio Code.app",
    ),
    ("iTerm2", "com.googlecode.iterm2", "/Applications/iTerm.app"),
    ("Spotify", "com.spotify.client", "/Applications/Spotify.app"),
    ("Discord", "com.hnc.Discord", "/Applications/Discord.app"),
    ("Notion", "notion.id", "/Applications/Notion.app"),
    ("Figma", "com.figma.Desktop", "/Applications/Figma.app"),
    (
        "Terminal",
        "com.apple.Terminal",
        "/System/Applications/Utilities/Terminal.app",
    ),
    (
        "Finder",
        "com.apple.finder",
        "/System/Library/CoreServices/Finder.app",
    ),
];

/// Return true when a bundle id is part of Extendead's controlled app catalog.
pub fn is_supported_bundle_id(bundle_id: &str) -> bool {
    BROWSER_CANDIDATES
        .iter()
        .any(|(_, candidate_bundle_id, _)| *candidate_bundle_id == bundle_id)
        || APP_CANDIDATES
            .iter()
            .any(|(_, candidate_bundle_id, _)| *candidate_bundle_id == bundle_id)
}

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

/// Scan for installed applications (non-browser) by checking well-known paths.
pub fn scan_apps() -> Vec<AppInfo> {
    APP_CANDIDATES
        .iter()
        .filter_map(|(name, bundle_id, path)| {
            if Path::new(path).exists() {
                Some(AppInfo {
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

/// Check if an application with the given bundle ID is detected as installed.
pub fn is_app_installed(info: &MachineInfo, bundle_id: &str) -> bool {
    info.installed_browsers
        .iter()
        .any(|b| b.bundle_id == bundle_id)
        || info.installed_apps.iter().any(|a| a.bundle_id == bundle_id)
}

/// Collect hostname, username, home dir, OS version, architecture,
/// installed browsers, and installed applications.
pub fn scan_machine() -> MachineInfo {
    let hostname = hostname();
    let username = username();
    let home_dir = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~".to_string());
    let installed_browsers = scan_browsers();
    let installed_apps = scan_apps();
    let os_version = os_version();
    let architecture = architecture();

    MachineInfo {
        hostname,
        username,
        os_version,
        architecture,
        installed_browsers,
        installed_apps,
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

fn os_version() -> String {
    #[cfg(target_os = "macos")]
    {
        read_command("sw_vers", &["-productVersion"])
    }
    #[cfg(not(target_os = "macos"))]
    {
        "unknown".to_string()
    }
}

fn architecture() -> String {
    #[cfg(target_os = "macos")]
    {
        read_command("uname", &["-m"])
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::consts::ARCH.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_machine_returns_populated_info() {
        let info = scan_machine();
        // hostname and username should never be empty.
        assert!(!info.hostname.is_empty());
        assert!(!info.username.is_empty());
        assert!(!info.home_dir.is_empty());
        assert!(!info.architecture.is_empty());
    }

    #[test]
    fn is_app_installed_finds_browser() {
        let info = MachineInfo {
            hostname: "test".to_string(),
            username: "test".to_string(),
            os_version: "14.0".to_string(),
            architecture: "x86_64".to_string(),
            installed_browsers: vec![BrowserInfo {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
                path: "/Applications/Safari.app".to_string(),
            }],
            installed_apps: vec![],
            home_dir: "/Users/test".to_string(),
        };
        assert!(is_app_installed(&info, "com.apple.Safari"));
        assert!(!is_app_installed(&info, "com.google.Chrome"));
    }

    #[test]
    fn is_app_installed_finds_app() {
        let info = MachineInfo {
            hostname: "test".to_string(),
            username: "test".to_string(),
            os_version: "14.0".to_string(),
            architecture: "x86_64".to_string(),
            installed_browsers: vec![],
            installed_apps: vec![AppInfo {
                name: "Slack".to_string(),
                bundle_id: "com.tinyspeck.slackmacgap".to_string(),
                path: "/Applications/Slack.app".to_string(),
            }],
            home_dir: "/Users/test".to_string(),
        };
        assert!(is_app_installed(&info, "com.tinyspeck.slackmacgap"));
        assert!(!is_app_installed(&info, "com.apple.Safari"));
    }
}
