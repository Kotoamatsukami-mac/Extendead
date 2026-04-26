use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{AppInfo, BrowserInfo, MachineInfo};

const APP_CACHE_TTL_SECS: i64 = 5 * 60;

/// Browser metadata is classification/alias data only. Discovery comes from the filesystem.
static BROWSER_BUNDLE_IDS: &[&str] = &[
    "com.google.Chrome",
    "com.apple.Safari",
    "org.mozilla.firefox",
    "com.brave.Browser",
    "company.thebrowser.Browser",
    "com.microsoft.edgemac",
    "com.operasoftware.Opera",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppDiscoveryCache {
    scanned_at: DateTime<Utc>,
    installed_browsers: Vec<BrowserInfo>,
    installed_apps: Vec<AppInfo>,
}

static APP_DISCOVERY_CACHE: OnceLock<Mutex<Option<AppDiscoveryCache>>> = OnceLock::new();

/// Return true when a bundle id is syntactically safe for `open -b` / AppleScript.
pub fn is_supported_bundle_id(bundle_id: &str) -> bool {
    !bundle_id.is_empty()
        && bundle_id.len() <= 256
        && bundle_id.split('.').count() >= 2
        && bundle_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// Scan for installed browsers using the real app discovery cache.
pub fn scan_browsers() -> Vec<BrowserInfo> {
    discover_installed_apps().installed_browsers
}

/// Scan for installed applications using the real app discovery cache.
pub fn scan_apps() -> Vec<AppInfo> {
    discover_installed_apps().installed_apps
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
    let discovered = discover_installed_apps();
    let installed_browsers = discovered.installed_browsers;
    let installed_apps = discovered.installed_apps;
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

pub fn app_cache_is_stale(info: &MachineInfo) -> bool {
    let cache = APP_DISCOVERY_CACHE.get_or_init(|| Mutex::new(None));
    let Ok(guard) = cache.lock() else {
        return true;
    };
    let Some(cache) = guard.as_ref() else {
        return true;
    };
    Utc::now() - cache.scanned_at > Duration::seconds(APP_CACHE_TTL_SECS)
        || (info.installed_apps.is_empty() && info.installed_browsers.is_empty())
}

fn discover_installed_apps() -> AppDiscoveryCache {
    if let Some(cache) = fresh_memory_cache() {
        return cache;
    }

    if let Some(cache) = fresh_disk_cache() {
        store_memory_cache(cache.clone());
        return cache;
    }

    let cache = scan_app_filesystem();
    store_memory_cache(cache.clone());
    let _ = persist_disk_cache(&cache);
    cache
}

fn fresh_memory_cache() -> Option<AppDiscoveryCache> {
    let cache = APP_DISCOVERY_CACHE.get_or_init(|| Mutex::new(None));
    let guard = cache.lock().ok()?;
    let cached = guard.as_ref()?;
    if Utc::now() - cached.scanned_at <= Duration::seconds(APP_CACHE_TTL_SECS) {
        Some(cached.clone())
    } else {
        None
    }
}

fn store_memory_cache(cache: AppDiscoveryCache) {
    let memory = APP_DISCOVERY_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = memory.lock() {
        *guard = Some(cache);
    }
}

fn fresh_disk_cache() -> Option<AppDiscoveryCache> {
    let path = app_cache_path()?;
    let bytes = std::fs::read(path).ok()?;
    let cache: AppDiscoveryCache = serde_json::from_slice(&bytes).ok()?;
    if Utc::now() - cache.scanned_at <= Duration::seconds(APP_CACHE_TTL_SECS) {
        Some(cache)
    } else {
        None
    }
}

fn persist_disk_cache(cache: &AppDiscoveryCache) -> Result<(), std::io::Error> {
    let Some(path) = app_cache_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(cache).unwrap_or_default();
    std::fs::write(path, json)
}

fn app_cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("extendead").join("installed_apps.json"))
}

fn scan_app_filesystem() -> AppDiscoveryCache {
    let roots = app_scan_roots();
    let mut discovered = Vec::new();
    let mut seen_paths = HashSet::new();

    for root in roots {
        collect_app_bundles(&root, 0, &mut seen_paths, &mut discovered);
    }

    if let Some(finder) = read_app_bundle(Path::new("/System/Library/CoreServices/Finder.app")) {
        discovered.push(finder);
    }

    let mut by_bundle: HashMap<String, AppInfo> = HashMap::new();
    for app in discovered {
        if app.bundle_id.is_empty() || !is_supported_bundle_id(&app.bundle_id) {
            continue;
        }
        by_bundle.entry(app.bundle_id.clone()).or_insert(app);
    }

    let mut installed_browsers = Vec::new();
    let mut installed_apps = Vec::new();
    for app in by_bundle.into_values() {
        if BROWSER_BUNDLE_IDS.contains(&app.bundle_id.as_str()) {
            installed_browsers.push(BrowserInfo {
                name: app.name,
                bundle_id: app.bundle_id,
                path: app.path,
            });
        } else {
            installed_apps.push(app);
        }
    }

    installed_browsers.sort_by(|a, b| a.name.cmp(&b.name));
    installed_apps.sort_by(|a, b| a.name.cmp(&b.name));

    AppDiscoveryCache {
        scanned_at: Utc::now(),
        installed_browsers,
        installed_apps,
    }
}

fn app_scan_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join("Applications"));
    }
    roots
}

fn collect_app_bundles(
    dir: &Path,
    depth: usize,
    seen_paths: &mut HashSet<PathBuf>,
    discovered: &mut Vec<AppInfo>,
) {
    if depth > 5 || !dir.exists() {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("app") {
            if seen_paths.insert(path.clone()) {
                if let Some(app) = read_app_bundle(&path) {
                    discovered.push(app);
                }
            }
            continue;
        }

        if path.is_dir() {
            collect_app_bundles(&path, depth + 1, seen_paths, discovered);
        }
    }
}

fn read_app_bundle(path: &Path) -> Option<AppInfo> {
    if !path.exists() {
        return None;
    }
    let info_plist = path.join("Contents").join("Info.plist");
    let bundle_id = read_info_plist_key(&info_plist, "CFBundleIdentifier")?;
    let name = read_info_plist_key(&info_plist, "CFBundleDisplayName")
        .or_else(|| read_info_plist_key(&info_plist, "CFBundleName"))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Application")
                .to_string()
        });

    Some(AppInfo {
        name,
        bundle_id,
        path: path.display().to_string(),
    })
}

#[cfg(target_os = "macos")]
fn read_info_plist_key(info_plist: &Path, key: &str) -> Option<String> {
    std::process::Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", &format!("Print :{key}")])
        .arg(info_plist)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(not(target_os = "macos"))]
fn read_info_plist_key(_info_plist: &Path, _key: &str) -> Option<String> {
    None
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
