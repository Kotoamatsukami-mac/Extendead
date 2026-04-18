#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserTarget {
    Safari,
    Chrome,
    Firefox,
    Brave,
    Arc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    OpenYoutube,
    OpenYoutubeInBrowser(BrowserTarget),
    OpenBrowserApp(BrowserTarget),
    OpenFinder,
    OpenSlack,
    MuteVolume,
    SetVolume(u8),
    OpenDisplaySettings,
    RevealDownloads,
    Unknown(String),
}

pub fn parse_intent(raw: &str) -> Intent {
    let normalized = normalize(raw);
    let s = normalized.as_str();

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

    if matches_any(s, &["open finder", "finder"]) {
        return Intent::OpenFinder;
    }

    if matches_any(s, &["open slack", "slack"]) {
        return Intent::OpenSlack;
    }

    if let Some(browser) = parse_open_youtube_in_browser(s) {
        return Intent::OpenYoutubeInBrowser(browser);
    }

    if matches_any(s, &["open youtube", "youtube"]) {
        return Intent::OpenYoutube;
    }

    if let Some(browser) = parse_open_browser_app(s) {
        return Intent::OpenBrowserApp(browser);
    }

    Intent::Unknown(raw.to_string())
}

pub fn normalize(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn matches_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack == *n || haystack.starts_with(n))
}

fn parse_open_youtube_in_browser(s: &str) -> Option<BrowserTarget> {
    parse_browser_target(s.strip_prefix("open youtube in ")?)
}

fn parse_open_browser_app(s: &str) -> Option<BrowserTarget> {
    parse_browser_target(s.strip_prefix("open ")?)
}

fn parse_browser_target(s: &str) -> Option<BrowserTarget> {
    match s.trim() {
        "safari" => Some(BrowserTarget::Safari),
        "chrome" | "google chrome" => Some(BrowserTarget::Chrome),
        "firefox" => Some(BrowserTarget::Firefox),
        "brave" | "brave browser" => Some(BrowserTarget::Brave),
        "arc" => Some(BrowserTarget::Arc),
        _ => None,
    }
}

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
    fn parse_existing_core_intents() {
        assert_eq!(parse_intent("mute"), Intent::MuteVolume);
        assert_eq!(parse_intent("display settings"), Intent::OpenDisplaySettings);
        assert_eq!(parse_intent("downloads"), Intent::RevealDownloads);
        assert_eq!(parse_intent("open youtube"), Intent::OpenYoutube);
        assert_eq!(parse_intent("open slack"), Intent::OpenSlack);
        assert_eq!(parse_intent("set volume to 150"), Intent::SetVolume(100));
    }

    #[test]
    fn parse_targeted_browser_commands() {
        assert_eq!(
            parse_intent("open safari"),
            Intent::OpenBrowserApp(BrowserTarget::Safari)
        );
        assert_eq!(
            parse_intent("open google chrome"),
            Intent::OpenBrowserApp(BrowserTarget::Chrome)
        );
        assert_eq!(parse_intent("open finder"), Intent::OpenFinder);
    }

    #[test]
    fn parse_targeted_youtube_commands() {
        assert_eq!(
            parse_intent("open youtube in safari"),
            Intent::OpenYoutubeInBrowser(BrowserTarget::Safari)
        );
        assert_eq!(
            parse_intent("open youtube in chrome"),
            Intent::OpenYoutubeInBrowser(BrowserTarget::Chrome)
        );
    }

    #[test]
    fn unknown_falls_through() {
        assert!(matches!(parse_intent("do something weird"), Intent::Unknown(_)));
    }
}
