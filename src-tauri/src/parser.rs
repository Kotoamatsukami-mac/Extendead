/// Parsed browser target extracted from normalized input text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserTarget {
    Chrome,
    Safari,
    Firefox,
    Brave,
    Arc,
}

/// Parsed intent extracted from normalized input text.
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    OpenYoutube,
    OpenSlack,
    OpenBrowser(BrowserTarget),
    MuteVolume,
    SetVolume(u8),
    OpenDisplaySettings,
    RevealDownloads,
    Unknown(String),
}

/// Parse a raw user string into a typed intent.
/// Uses keyword matching — no ML, no network calls.
pub fn parse_intent(raw: &str) -> Intent {
    let s = raw.trim().to_lowercase();
    let s = s.as_str();

    // Volume control — check before generic "open" to catch "set volume to X"
    if let Some(level) = extract_volume_level(s) {
        return Intent::SetVolume(level);
    }

    if matches_any(s, &["mute the mac", "mute sound", "mute audio", "mute"]) {
        return Intent::MuteVolume;
    }

    if matches_any(
        s,
        &[
            "open display settings",
            "display settings",
            "displays settings",
            "screen settings",
            "monitor settings",
        ],
    ) {
        return Intent::OpenDisplaySettings;
    }

    if matches_any(
        s,
        &[
            "reveal downloads",
            "open downloads",
            "show downloads",
            "downloads",
        ],
    ) {
        return Intent::RevealDownloads;
    }

    if matches_any(s, &["open youtube", "youtube"]) {
        return Intent::OpenYoutube;
    }

    if matches_any(s, &["open slack", "slack"]) {
        return Intent::OpenSlack;
    }

    if let Some(target) = parse_browser_target(s) {
        return Intent::OpenBrowser(target);
    }

    Intent::Unknown(raw.to_string())
}

/// Normalize raw input: trim, collapse whitespace, lower-case.
pub fn normalize(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn matches_any(haystack: &str, needles: &[&str]) -> bool {
    needles
        .iter()
        .any(|n| haystack == *n || haystack.starts_with(n))
}

fn parse_browser_target(s: &str) -> Option<BrowserTarget> {
    match s {
        "chrome" | "open chrome" | "google chrome" | "open google chrome" => {
            Some(BrowserTarget::Chrome)
        }
        "safari" | "open safari" => Some(BrowserTarget::Safari),
        "firefox" | "open firefox" => Some(BrowserTarget::Firefox),
        "brave" | "open brave" | "brave browser" | "open brave browser" => {
            Some(BrowserTarget::Brave)
        }
        "arc" | "open arc" => Some(BrowserTarget::Arc),
        _ => None,
    }
}

/// Try to extract a volume level (0–100) from strings like:
/// "set volume to 30", "volume to 30 percent", "set volume 30"
fn extract_volume_level(s: &str) -> Option<u8> {
    let triggers = ["set volume to", "volume to", "set volume", "volume at"];
    for trigger in &triggers {
        if let Some(rest) = s.strip_prefix(trigger) {
            let rest = rest.trim().trim_end_matches("percent").trim();
            if let Ok(n) = rest.parse::<u8>() {
                return Some(n.min(100));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mute() {
        assert_eq!(parse_intent("mute"), Intent::MuteVolume);
        assert_eq!(parse_intent("mute the mac"), Intent::MuteVolume);
        assert_eq!(parse_intent("mute sound"), Intent::MuteVolume);
        assert_eq!(parse_intent("mute audio"), Intent::MuteVolume);
        assert_eq!(parse_intent("MUTE"), Intent::MuteVolume);
    }

    #[test]
    fn test_parse_volume() {
        assert_eq!(parse_intent("set volume to 30"), Intent::SetVolume(30));
        assert_eq!(parse_intent("volume to 50 percent"), Intent::SetVolume(50));
        assert_eq!(parse_intent("set volume 75"), Intent::SetVolume(75));
        assert_eq!(parse_intent("set volume to 0"), Intent::SetVolume(0));
        assert_eq!(parse_intent("set volume to 100"), Intent::SetVolume(100));
        assert_eq!(parse_intent("set volume to 150"), Intent::SetVolume(100));
    }

    #[test]
    fn test_parse_youtube() {
        assert_eq!(parse_intent("open youtube"), Intent::OpenYoutube);
        assert_eq!(parse_intent("youtube"), Intent::OpenYoutube);
        assert_eq!(parse_intent("YOUTUBE"), Intent::OpenYoutube);
        assert_eq!(parse_intent("  youtube  "), Intent::OpenYoutube);
    }

    #[test]
    fn test_parse_slack() {
        assert_eq!(parse_intent("open slack"), Intent::OpenSlack);
        assert_eq!(parse_intent("slack"), Intent::OpenSlack);
        assert_eq!(parse_intent("SLACK"), Intent::OpenSlack);
    }

    #[test]
    fn test_parse_browser_targets() {
        assert_eq!(
            parse_intent("open safari"),
            Intent::OpenBrowser(BrowserTarget::Safari)
        );
        assert_eq!(
            parse_intent("chrome"),
            Intent::OpenBrowser(BrowserTarget::Chrome)
        );
        assert_eq!(
            parse_intent("open google chrome"),
            Intent::OpenBrowser(BrowserTarget::Chrome)
        );
        assert_eq!(
            parse_intent("firefox"),
            Intent::OpenBrowser(BrowserTarget::Firefox)
        );
        assert_eq!(
            parse_intent("open brave browser"),
            Intent::OpenBrowser(BrowserTarget::Brave)
        );
        assert_eq!(parse_intent("arc"), Intent::OpenBrowser(BrowserTarget::Arc));
    }

    #[test]
    fn test_parse_downloads() {
        assert_eq!(parse_intent("downloads"), Intent::RevealDownloads);
        assert_eq!(parse_intent("reveal downloads"), Intent::RevealDownloads);
        assert_eq!(parse_intent("open downloads"), Intent::RevealDownloads);
        assert_eq!(parse_intent("show downloads"), Intent::RevealDownloads);
    }

    #[test]
    fn test_parse_display_settings() {
        assert_eq!(
            parse_intent("display settings"),
            Intent::OpenDisplaySettings
        );
        assert_eq!(
            parse_intent("open display settings"),
            Intent::OpenDisplaySettings
        );
        assert_eq!(parse_intent("screen settings"), Intent::OpenDisplaySettings);
        assert_eq!(
            parse_intent("monitor settings"),
            Intent::OpenDisplaySettings
        );
    }

    #[test]
    fn test_parse_unknown() {
        let result = parse_intent("do something weird");
        assert!(matches!(result, Intent::Unknown(_)));
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("  hello   world  "), "hello world");
        assert_eq!(normalize("HELLO"), "hello");
        assert_eq!(normalize("set  volume  to  30"), "set volume to 30");
    }

    #[test]
    fn test_volume_priority_over_open() {
        assert_eq!(parse_intent("set volume to 30"), Intent::SetVolume(30));
        let result = parse_intent("volume");
        assert!(matches!(result, Intent::Unknown(_)));
    }
}
