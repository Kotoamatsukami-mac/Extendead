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
    CloseAppNamed(String),
    OpenAppNamed(String),
    OpenPath(String),
    CreateFolder {
        name: String,
        base: Option<String>,
    },
    MovePath {
        source: String,
        destination: String,
    },
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

    if let Some(intent) = extract_move_path(raw) {
        return intent;
    }

    if let Some(intent) = extract_create_folder(raw) {
        return intent;
    }

    if let Some(app) = extract_close_app_name(raw) {
        return Intent::CloseAppNamed(app);
    }

    if let Some(path) = extract_open_path(raw) {
        return Intent::OpenPath(path);
    }

    if let Some(app) = extract_open_app_name(raw) {
        return Intent::OpenAppNamed(app);
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

fn extract_close_app_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = normalize(trimmed);
    for prefix in ["close ", "quit ", "exit "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let raw_rest = trimmed.get(prefix.len()..)?.trim();
            if !raw_rest.is_empty() {
                return Some(clean_token(raw_rest));
            }
            if !rest.trim().is_empty() {
                return Some(clean_token(rest));
            }
        }
    }
    None
}

fn extract_open_app_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = normalize(trimmed);
    for prefix in ["open ", "launch ", "start ", "run "] {
        if let Some(_rest) = lower.strip_prefix(prefix) {
            let raw_rest = trimmed.get(prefix.len()..)?.trim();
            if raw_rest.is_empty() || looks_like_path(raw_rest) || looks_like_url(raw_rest) {
                return None;
            }
            return Some(clean_token(raw_rest));
        }
    }
    None
}

fn extract_open_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = normalize(trimmed);

    for prefix in ["open folder ", "show folder ", "open file ", "show file "] {
        if lower.starts_with(prefix) {
            let rest = trimmed.get(prefix.len()..)?.trim();
            if !rest.is_empty() {
                return Some(clean_token(rest));
            }
        }
    }

    for prefix in ["open ", "show "] {
        if lower.starts_with(prefix) {
            let rest = trimmed.get(prefix.len()..)?.trim();
            if !rest.is_empty() && looks_like_path(rest) {
                return Some(clean_token(rest));
            }
        }
    }

    None
}

fn extract_create_folder(raw: &str) -> Option<Intent> {
    let trimmed = raw.trim();
    let lower = normalize(trimmed);
    if !(lower.starts_with("create folder") || lower.starts_with("make folder")) {
        return None;
    }

    let (name_marker, marker_len) = if let Some(idx) = lower.find(" called ") {
        (idx, " called ".len())
    } else if let Some(idx) = lower.find(" named ") {
        (idx, " named ".len())
    } else {
        return None;
    };

    let after_name = trimmed.get(name_marker + marker_len..)?.trim();
    if after_name.is_empty() {
        return None;
    }

    let after_name_lower = normalize(after_name);
    for in_marker in [" in ", " inside ", " under "] {
        if let Some(idx) = after_name_lower.find(in_marker) {
            let name = clean_token(after_name.get(..idx)?.trim());
            let base = clean_token(after_name.get(idx + in_marker.len()..)?.trim());
            if !name.is_empty() && !base.is_empty() {
                return Some(Intent::CreateFolder {
                    name,
                    base: Some(base),
                });
            }
        }
    }

    Some(Intent::CreateFolder {
        name: clean_token(after_name),
        base: None,
    })
}

fn extract_move_path(raw: &str) -> Option<Intent> {
    let trimmed = raw.trim();
    let lower = normalize(trimmed);
    for prefix in ["move ", "put "] {
        if !lower.starts_with(prefix) {
            continue;
        }
        let rest = trimmed.get(prefix.len()..)?.trim();
        let rest_lower = normalize(rest);
        for marker in [" to ", " into "] {
            if let Some(idx) = rest_lower.find(marker) {
                let source = clean_token(rest.get(..idx)?.trim());
                let destination = clean_token(rest.get(idx + marker.len()..)?.trim());
                if !source.is_empty() && !destination.is_empty() {
                    return Some(Intent::MovePath { source, destination });
                }
            }
        }
    }
    None
}

fn looks_like_path(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with('~')
        || trimmed.starts_with('/')
        || trimmed.contains("/")
        || trimmed.contains("\\")
        || is_known_path_alias(trimmed)
}

fn looks_like_url(value: &str) -> bool {
    let lower = value.trim().to_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.")
}

fn is_known_path_alias(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "desktop" | "downloads" | "documents" | "applications" | "home"
    )
}

fn clean_token(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('“')
        .trim_matches('”')
        .trim_matches('\'')
        .to_string()
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

    #[test]
    fn test_parse_close_app() {
        assert_eq!(
            parse_intent("close safari"),
            Intent::CloseAppNamed("safari".to_string())
        );
    }

    #[test]
    fn test_parse_open_path() {
        assert_eq!(
            parse_intent("open ~/Desktop"),
            Intent::OpenPath("~/Desktop".to_string())
        );
    }

    #[test]
    fn test_parse_create_folder() {
        assert_eq!(
            parse_intent("create folder called Chat in home"),
            Intent::CreateFolder {
                name: "Chat".to_string(),
                base: Some("home".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_move_path() {
        assert_eq!(
            parse_intent("move ~/Desktop/test.txt to ~/Documents"),
            Intent::MovePath {
                source: "~/Desktop/test.txt".to_string(),
                destination: "~/Documents".to_string(),
            }
        );
    }
}
