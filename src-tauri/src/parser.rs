#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    OpenYoutube,
    OpenYoutubeInSafari,
    OpenYoutubeInChrome,
    OpenYoutubeInFirefox,
    OpenYoutubeInBrave,
    OpenYoutubeInArc,
    OpenSafari,
    OpenChrome,
    OpenFirefox,
    OpenBrave,
    OpenArc,
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

    if matches_any(s, &["open youtube in safari"]) {
        return Intent::OpenYoutubeInSafari;
    }
    if matches_any(
        s,
        &["open youtube in chrome", "open youtube in google chrome"],
    ) {
        return Intent::OpenYoutubeInChrome;
    }
    if matches_any(s, &["open youtube in firefox"]) {
        return Intent::OpenYoutubeInFirefox;
    }
    if matches_any(
        s,
        &["open youtube in brave", "open youtube in brave browser"],
    ) {
        return Intent::OpenYoutubeInBrave;
    }
    if matches_any(s, &["open youtube in arc"]) {
        return Intent::OpenYoutubeInArc;
    }

    if matches_any(s, &["open youtube", "youtube"]) {
        return Intent::OpenYoutube;
    }

    if matches_any(s, &["open safari", "safari"]) {
        return Intent::OpenSafari;
    }
    if matches_any(
        s,
        &[
            "open chrome",
            "open google chrome",
            "chrome",
            "google chrome",
        ],
    ) {
        return Intent::OpenChrome;
    }
    if matches_any(s, &["open firefox", "firefox"]) {
        return Intent::OpenFirefox;
    }
    if matches_any(
        s,
        &["open brave", "open brave browser", "brave", "brave browser"],
    ) {
        return Intent::OpenBrave;
    }
    if matches_any(s, &["open arc", "arc"]) {
        return Intent::OpenArc;
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
    needles
        .iter()
        .any(|n| haystack == *n || haystack.starts_with(n))
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
    fn test_parse_browser_apps() {
        assert_eq!(parse_intent("open safari"), Intent::OpenSafari);
        assert_eq!(parse_intent("open google chrome"), Intent::OpenChrome);
        assert_eq!(parse_intent("open firefox"), Intent::OpenFirefox);
        assert_eq!(parse_intent("open brave browser"), Intent::OpenBrave);
        assert_eq!(parse_intent("open arc"), Intent::OpenArc);
    }

    #[test]
    fn test_parse_finder() {
        assert_eq!(parse_intent("open finder"), Intent::OpenFinder);
    }

    #[test]
    fn test_parse_youtube_in_browser() {
        assert_eq!(
            parse_intent("open youtube in safari"),
            Intent::OpenYoutubeInSafari
        );
        assert_eq!(
            parse_intent("open youtube in chrome"),
            Intent::OpenYoutubeInChrome
        );
    }

    #[test]
    fn test_parse_existing_commands() {
        assert_eq!(parse_intent("mute"), Intent::MuteVolume);
        assert_eq!(parse_intent("set volume to 30"), Intent::SetVolume(30));
        assert_eq!(parse_intent("downloads"), Intent::RevealDownloads);
        assert_eq!(
            parse_intent("display settings"),
            Intent::OpenDisplaySettings
        );
        assert_eq!(parse_intent("slack"), Intent::OpenSlack);
    }
}
