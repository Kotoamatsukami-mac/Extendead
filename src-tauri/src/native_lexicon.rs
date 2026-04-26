use serde::Serialize;

use crate::models::MachineInfo;
use crate::service_catalog;

#[derive(Debug, Clone, Serialize)]
pub struct NativeLexicon {
    pub apps: Vec<LexiconEntry>,
    pub browsers: Vec<LexiconEntry>,
    pub folders: Vec<LexiconEntry>,
    pub services: Vec<LexiconEntry>,
    pub settings: Vec<LexiconEntry>,
    pub verbs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LexiconEntry {
    pub label: String,
    pub canonical: String,
    pub aliases: Vec<String>,
    pub kind: String,
}

pub fn build(machine: &MachineInfo) -> NativeLexicon {
    NativeLexicon {
        apps: machine
            .installed_apps
            .iter()
            .map(|app| LexiconEntry {
                label: app.name.clone(),
                canonical: app.bundle_id.clone(),
                aliases: app_aliases(&app.name),
                kind: "app".to_string(),
            })
            .collect(),
        browsers: machine
            .installed_browsers
            .iter()
            .map(|browser| LexiconEntry {
                label: browser.name.clone(),
                canonical: browser.bundle_id.clone(),
                aliases: app_aliases(&browser.name),
                kind: "browser".to_string(),
            })
            .collect(),
        folders: folder_entries(&machine.home_dir),
        services: service_catalog::all_services()
            .iter()
            .map(|service| LexiconEntry {
                label: service.display_name.to_string(),
                canonical: service.id.to_string(),
                aliases: service.aliases.iter().map(|alias| alias.to_string()).collect(),
                kind: "service".to_string(),
            })
            .collect(),
        settings: vec![LexiconEntry {
            label: "Displays".to_string(),
            canonical: "x-apple.systempreferences:com.apple.preference.displays".to_string(),
            aliases: vec![
                "display settings".to_string(),
                "displays".to_string(),
                "screen settings".to_string(),
                "monitor settings".to_string(),
            ],
            kind: "setting".to_string(),
        }],
        verbs: vec![
            "open".to_string(),
            "close".to_string(),
            "hide".to_string(),
            "force quit".to_string(),
            "make".to_string(),
            "create".to_string(),
            "move".to_string(),
            "trash".to_string(),
            "mute".to_string(),
            "set".to_string(),
            "increase".to_string(),
            "decrease".to_string(),
            "run".to_string(),
        ],
    }
}

pub fn contains_word(lexicon: &NativeLexicon, word: &str, kind: &str) -> bool {
    let normalized = normalize(word);
    entries_for_kind(lexicon, kind).iter().any(|entry| {
        normalize(&entry.label) == normalized
            || entry
                .aliases
                .iter()
                .any(|alias| normalize(alias) == normalized || normalize(alias).starts_with(&normalized))
    })
}

fn entries_for_kind<'a>(lexicon: &'a NativeLexicon, kind: &str) -> &'a [LexiconEntry] {
    match kind {
        "app" => &lexicon.apps,
        "browser" => &lexicon.browsers,
        "service" => &lexicon.services,
        "folder" => &lexicon.folders,
        "setting" => &lexicon.settings,
        _ => &[],
    }
}

fn folder_entries(home_dir: &str) -> Vec<LexiconEntry> {
    [
        ("Home", home_dir, &["home"] as &[&str]),
        ("Desktop", "~/Desktop", &["desktop", "desk"]),
        ("Downloads", "~/Downloads", &["downloads", "download"]),
        ("Documents", "~/Documents", &["documents", "docs"]),
        ("Applications", "/Applications", &["applications", "apps"]),
    ]
    .iter()
    .map(|(label, canonical, aliases)| LexiconEntry {
        label: (*label).to_string(),
        canonical: (*canonical).to_string(),
        aliases: aliases.iter().map(|alias| (*alias).to_string()).collect(),
        kind: "folder".to_string(),
    })
    .collect()
}

fn app_aliases(name: &str) -> Vec<String> {
    let normalized = normalize(name).trim_end_matches(".app").trim().to_string();
    let mut aliases = vec![normalized.clone()];
    if let Some(stripped) = normalized.strip_suffix(" browser") {
        aliases.push(stripped.to_string());
    }
    if normalized == "google chrome" {
        aliases.push("chrome".to_string());
    }
    if normalized == "visual studio code" {
        aliases.push("vscode".to_string());
        aliases.push("code".to_string());
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}
