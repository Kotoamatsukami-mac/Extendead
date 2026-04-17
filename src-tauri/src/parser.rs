/// Parsed intent extracted from normalized input text.
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    OpenYoutube,
    OpenSlack,
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
    }

    #[test]
    fn test_parse_volume() {
        assert_eq!(parse_intent("set volume to 30"), Intent::SetVolume(30));
        assert_eq!(parse_intent("volume to 50 percent"), Intent::SetVolume(50));
        assert_eq!(parse_intent("set volume 75"), Intent::SetVolume(75));
    }

    #[test]
    fn test_parse_youtube() {
        assert_eq!(parse_intent("open youtube"), Intent::OpenYoutube);
        assert_eq!(parse_intent("youtube"), Intent::OpenYoutube);
    }

    #[test]
    fn test_parse_downloads() {
        assert_eq!(parse_intent("downloads"), Intent::RevealDownloads);
        assert_eq!(parse_intent("reveal downloads"), Intent::RevealDownloads);
    }
}
