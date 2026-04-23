use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::errors::AppError;
use crate::intent_language::{CandidateIntent, CanonicalAction, InterpretationSource};
use crate::intent_ontology;
use crate::models::MachineInfo;
use crate::{parser, provider_keys, service_catalog};

const PROVIDER_NAME: &str = "perplexity";
const PROVIDER_MODEL: &str = "sonar-pro";
const PROVIDER_ENDPOINT: &str = "https://api.perplexity.ai/v1/sonar";

#[derive(Debug, Clone, Deserialize)]
struct ProviderInterpretationResponse {
    candidates: Vec<ProviderCandidatePayload>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProviderCandidatePayload {
    canonical_action: CanonicalAction,
    slots: BTreeMap<String, String>,
    missing_slots: Vec<String>,
    confidence: f32,
    clarification_needed: bool,
}

#[derive(Debug, Deserialize)]
struct SonarResponse {
    choices: Vec<SonarChoice>,
}

#[derive(Debug, Deserialize)]
struct SonarChoice {
    message: SonarMessage,
}

#[derive(Debug, Deserialize)]
struct SonarMessage {
    content: String,
}

pub async fn interpret(
    input: &str,
    machine: &MachineInfo,
) -> Result<Vec<CandidateIntent>, AppError> {
    let key = provider_keys::retrieve_key(PROVIDER_NAME)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| AppError::ExecutionError(format!("provider client init failed: {e}")))?;

    let payload = json!({
        "model": PROVIDER_MODEL,
        "temperature": 0.1,
        "max_tokens": 500,
        "disable_search": true,
        "messages": [
            {
                "role": "system",
                "content": system_prompt(),
            },
            {
                "role": "user",
                "content": user_prompt(input, machine),
            }
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "extendead_interpretation",
                "schema": response_schema(),
            }
        }
    });

    let response = client
        .post(PROVIDER_ENDPOINT)
        .bearer_auth(key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::ExecutionError(format!("provider request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "provider response body unavailable".to_string());
        return Err(AppError::ExecutionError(format!(
            "provider API returned {status}: {body}"
        )));
    }

    let response: SonarResponse = response.json().await.map_err(|e| {
        AppError::SerializationError(format!("provider response decode failed: {e}"))
    })?;
    let content = response
        .choices
        .first()
        .map(|choice| choice.message.content.trim())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| {
            AppError::SerializationError("provider returned no message content".to_string())
        })?;

    Ok(repair_candidates(
        parse_candidates(content)?,
        input,
        machine,
    ))
}

fn parse_candidates(content: &str) -> Result<Vec<CandidateIntent>, AppError> {
    let parsed = parse_response_body(content)?;
    parsed
        .candidates
        .into_iter()
        .take(3)
        .filter_map(candidate_from_payload)
        .collect::<Result<Vec<_>, _>>()
}

fn parse_response_body(content: &str) -> Result<ProviderInterpretationResponse, AppError> {
    serde_json::from_str(content)
        .or_else(|_| {
            extract_json_object(content)
                .ok_or_else(|| serde_json::Error::io(std::io::Error::other("no JSON object found")))
                .and_then(|json| serde_json::from_str(&json))
        })
        .map_err(|e| AppError::SerializationError(format!("provider JSON parse failed: {e}")))
}

fn extract_json_object(content: &str) -> Option<String> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(content[start..=end].to_string())
}

fn candidate_from_payload(
    payload: ProviderCandidatePayload,
) -> Option<Result<CandidateIntent, AppError>> {
    if payload.confidence <= 0.0 {
        return None;
    }

    Some(
        intent_ontology::action_for_canonical_action(payload.canonical_action)
            .ok_or_else(|| {
                AppError::ValidationError(format!(
                    "provider returned unsupported action {:?}",
                    payload.canonical_action
                ))
            })
            .map(|action| {
                let mut missing_slots = payload.missing_slots;
                for required_slot in action.required_slots {
                    let has_value = payload
                        .slots
                        .get(*required_slot)
                        .map(|value| !value.trim().is_empty())
                        .unwrap_or(false);
                    if has_value
                        || missing_slots
                            .iter()
                            .any(|slot| slot.as_str() == *required_slot)
                    {
                        continue;
                    }
                    missing_slots.push((*required_slot).to_string());
                }

                let clarification_needed =
                    payload.clarification_needed || !missing_slots.is_empty();

                CandidateIntent {
                    family: action.family,
                    canonical_action: payload.canonical_action,
                    slots: payload.slots,
                    missing_slots,
                    confidence: payload.confidence.clamp(0.0, 1.0),
                    clarification_needed,
                    risk_baseline: action.risk_baseline.clone(),
                    executor_family: action.executor_family,
                    source: InterpretationSource::Provider,
                }
            }),
    )
}

fn repair_candidates(
    candidates: Vec<CandidateIntent>,
    input: &str,
    machine: &MachineInfo,
) -> Vec<CandidateIntent> {
    let normalized_input = parser::normalize(input);
    candidates
        .into_iter()
        .map(|candidate| repair_candidate(candidate, input, &normalized_input, machine))
        .collect()
}

fn repair_candidate(
    mut candidate: CandidateIntent,
    raw_input: &str,
    normalized_input: &str,
    machine: &MachineInfo,
) -> CandidateIntent {
    maybe_promote_browser_new_tab_to_service(&mut candidate, normalized_input);
    maybe_promote_open_path_to_trash(&mut candidate, normalized_input);

    match candidate.canonical_action {
        CanonicalAction::OpenApp | CanonicalAction::QuitApp => {
            repair_named_slot(
                &mut candidate,
                "app",
                match_unique_app_or_browser(normalized_input, machine),
            );
        }
        CanonicalAction::OpenService => {
            repair_named_slot(
                &mut candidate,
                "service",
                match_unique_service(normalized_input).map(str::to_string),
            );
            repair_named_slot(
                &mut candidate,
                "browser",
                match_unique_browser(normalized_input, machine),
            );
        }
        CanonicalAction::BrowserNewTab
        | CanonicalAction::BrowserCloseTab
        | CanonicalAction::BrowserReopenClosedTab => {
            repair_named_slot(
                &mut candidate,
                "browser",
                match_unique_browser(normalized_input, machine),
            );
        }
        CanonicalAction::OpenPath | CanonicalAction::TrashPath => {
            repair_named_slot(&mut candidate, "path", extract_obvious_path(raw_input));
        }
        _ => {}
    }

    candidate.missing_slots.retain(|slot| {
        candidate
            .slots
            .get(slot)
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
    });
    candidate.clarification_needed = !candidate.missing_slots.is_empty();
    candidate
}

fn maybe_promote_browser_new_tab_to_service(
    candidate: &mut CandidateIntent,
    normalized_input: &str,
) {
    if candidate.canonical_action != CanonicalAction::BrowserNewTab {
        return;
    }

    let Some(service_id) = match_unique_service(normalized_input) else {
        return;
    };
    let Some(action) = intent_ontology::action_for_canonical_action(CanonicalAction::OpenService)
    else {
        return;
    };

    candidate.family = action.family;
    candidate.canonical_action = CanonicalAction::OpenService;
    candidate.executor_family = action.executor_family;
    candidate.risk_baseline = action.risk_baseline.clone();
    candidate
        .missing_slots
        .retain(|slot| slot.as_str() != "service");
    candidate
        .slots
        .insert("service".to_string(), service_id.to_string());
}

fn maybe_promote_open_path_to_trash(candidate: &mut CandidateIntent, normalized_input: &str) {
    if candidate.canonical_action != CanonicalAction::OpenPath
        || !looks_like_trash_request(normalized_input)
    {
        return;
    }

    let Some(action) = intent_ontology::action_for_canonical_action(CanonicalAction::TrashPath)
    else {
        return;
    };

    candidate.family = action.family;
    candidate.canonical_action = CanonicalAction::TrashPath;
    candidate.executor_family = action.executor_family;
    candidate.risk_baseline = action.risk_baseline.clone();
}

fn repair_named_slot(candidate: &mut CandidateIntent, slot: &str, recovered_value: Option<String>) {
    let slot_is_present = candidate
        .slots
        .get(slot)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    if slot_is_present {
        return;
    }

    let Some(value) = recovered_value else {
        return;
    };
    candidate.slots.insert(slot.to_string(), value);
}

fn extract_obvious_path(raw_input: &str) -> Option<String> {
    let trimmed = raw_input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    for prefix in [
        "trash ", "delete ", "remove ", "toss ", "bin ", "open ", "show ", "reveal ",
    ] {
        if lower.starts_with(prefix) {
            let rest = trimmed.get(prefix.len()..)?.trim();
            if let Some(path) = sanitize_path_candidate(rest) {
                return Some(path);
            }
        }
    }

    if let Some((source, destination)) = trimmed.split_once(" to ") {
        if matches!(
            destination.trim().to_ascii_lowercase().as_str(),
            "trash" | "the trash" | "~/.trash"
        ) {
            return sanitize_path_candidate(source.trim());
        }
    }

    trimmed.split_whitespace().find_map(sanitize_path_candidate)
}

fn sanitize_path_candidate(candidate: &str) -> Option<String> {
    let cleaned = candidate
        .trim()
        .trim_matches(|c| matches!(c, '"' | '\'' | ',' | '.' | '!' | '?' | ';' | ':'))
        .trim();
    if looks_like_path_candidate(cleaned) {
        Some(cleaned.to_string())
    } else {
        None
    }
}

fn looks_like_path_candidate(value: &str) -> bool {
    value.starts_with("~/")
        || value.starts_with('/')
        || value.contains('/')
        || matches!(
            parser::normalize(value).as_str(),
            "desktop" | "downloads" | "documents" | "home"
        )
}

fn match_unique_app_or_browser(normalized_input: &str, machine: &MachineInfo) -> Option<String> {
    let matches = machine
        .installed_apps
        .iter()
        .filter(|app| phrase_in_input(normalized_input, &parser::normalize(&app.name)))
        .map(|app| app.name.clone())
        .chain(
            machine
                .installed_browsers
                .iter()
                .filter(|browser| {
                    browser_aliases(&browser.name)
                        .iter()
                        .any(|alias| phrase_in_input(normalized_input, alias))
                })
                .map(|browser| browser.name.clone()),
        )
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        None
    }
}

fn match_unique_browser(normalized_input: &str, machine: &MachineInfo) -> Option<String> {
    let matches = machine
        .installed_browsers
        .iter()
        .filter(|browser| {
            browser_aliases(&browser.name)
                .iter()
                .any(|alias| phrase_in_input(normalized_input, alias))
        })
        .map(|browser| browser.name.clone())
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        None
    }
}

fn browser_aliases(name: &str) -> Vec<String> {
    match parser::normalize(name).as_str() {
        "google chrome" => vec!["chrome".to_string(), "google chrome".to_string()],
        "safari" => vec!["safari".to_string()],
        "firefox" => vec!["firefox".to_string()],
        "brave" | "brave browser" => vec!["brave".to_string(), "brave browser".to_string()],
        "arc" => vec!["arc".to_string()],
        other => vec![other.to_string()],
    }
}

fn match_unique_service(normalized_input: &str) -> Option<&'static str> {
    let matches = service_catalog::all_services()
        .iter()
        .filter(|service| {
            phrase_in_input(normalized_input, &parser::normalize(service.display_name))
                || service
                    .aliases
                    .iter()
                    .any(|alias| phrase_in_input(normalized_input, &parser::normalize(alias)))
        })
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        Some(matches[0].id)
    } else {
        None
    }
}

fn looks_like_trash_request(normalized_input: &str) -> bool {
    ["trash ", "delete ", "remove ", "toss ", "bin ", "move "]
        .iter()
        .any(|prefix| normalized_input.starts_with(prefix))
}

fn phrase_in_input(normalized_input: &str, normalized_phrase: &str) -> bool {
    if normalized_phrase.is_empty() {
        return false;
    }
    if normalized_input == normalized_phrase {
        return true;
    }
    let padded_input = format!(" {normalized_input} ");
    let padded_phrase = format!(" {normalized_phrase} ");
    padded_input.contains(&padded_phrase)
}

fn system_prompt() -> String {
    [
        "You are Extendead's provider-backed interpreter.",
        "Return only structured JSON matching the provided schema.",
        "You may only propose commands already supported by Extendead.",
        "Never fabricate installed apps, browsers, services, paths, permissions, or success.",
        "If the request cannot be mapped safely, return an empty candidates array.",
        "If a canonical_action requires a slot, either include it in slots with a non-empty value or list it in missing_slots.",
        "Allowed canonical_action values: open_app, quit_app, open_path, create_folder, move_path, open_service, browser_new_tab, browser_close_tab, browser_reopen_closed_tab, brightness_up, brightness_down, trash_path.",
        "Allowed slot keys: app, path, name, base, base_path, source, destination, service, browser.",
        "Use supported service IDs exactly as provided.",
        "Use installed app or browser names exactly as provided.",
        "For open_app and quit_app, set slots.app to the exact installed app name whenever the user names one.",
        "For open_service, set slots.service to the exact supported service ID whenever the user names one.",
        "For browser tab actions, set slots.browser when a browser is specified.",
        "If required information is missing, keep the best candidate, list missing_slots, and set clarification_needed to true.",
        "Prefer one strong candidate. Return up to three only when materially different actions remain plausible.",
    ]
    .join(" ")
}

fn user_prompt(input: &str, machine: &MachineInfo) -> String {
    let installed_apps = machine
        .installed_apps
        .iter()
        .map(|app| app.name.as_str())
        .collect::<Vec<_>>();
    let installed_browsers = machine
        .installed_browsers
        .iter()
        .map(|browser| browser.name.as_str())
        .collect::<Vec<_>>();
    let services = service_catalog::all_services()
        .iter()
        .map(|service| {
            format!(
                "{} ({}) [{}]",
                service.id,
                service.display_name,
                service.aliases.join(", ")
            )
        })
        .collect::<Vec<_>>();

    format!(
        concat!(
            "User input: {input}\n",
            "Installed browsers: {installed_browsers}\n",
            "Installed apps: {installed_apps}\n",
            "Supported services: {services}\n",
            "Home directory: {home_dir}\n",
            "Path aliases: home, desktop, downloads, documents, ~/...\n",
            "Return candidate commands only if they fit the supported local schema."
        ),
        input = input.trim(),
        installed_browsers = installed_browsers.join(", "),
        installed_apps = installed_apps.join(", "),
        services = services.join("; "),
        home_dir = machine.home_dir,
    )
}

fn response_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["candidates"],
        "properties": {
            "candidates": {
                "type": "array",
                "maxItems": 3,
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": [
                        "canonical_action",
                        "slots",
                        "missing_slots",
                        "confidence",
                        "clarification_needed"
                    ],
                    "properties": {
                        "canonical_action": {
                            "type": "string",
                            "enum": [
                                "open_app",
                                "quit_app",
                                "open_path",
                                "create_folder",
                                "move_path",
                                "open_service",
                                "browser_new_tab",
                                "browser_close_tab",
                                "browser_reopen_closed_tab",
                                "brightness_up",
                                "brightness_down",
                                "trash_path"
                            ]
                        },
                        "slots": {
                            "type": "object",
                            "additionalProperties": {
                                "type": "string"
                            }
                        },
                        "missing_slots": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            }
                        },
                        "confidence": {
                            "type": "number",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "clarification_needed": {
                            "type": "boolean"
                        }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppInfo, BrowserInfo};

    fn machine_fixture() -> MachineInfo {
        MachineInfo {
            hostname: "test-host".to_string(),
            username: "tester".to_string(),
            os_version: "macOS".to_string(),
            architecture: "arm64".to_string(),
            installed_browsers: vec![
                BrowserInfo {
                    name: "Safari".to_string(),
                    bundle_id: "com.apple.Safari".to_string(),
                    path: "/Applications/Safari.app".to_string(),
                },
                BrowserInfo {
                    name: "Google Chrome".to_string(),
                    bundle_id: "com.google.Chrome".to_string(),
                    path: "/Applications/Google Chrome.app".to_string(),
                },
            ],
            installed_apps: vec![AppInfo {
                name: "Slack".to_string(),
                bundle_id: "com.tinyspeck.slackmacgap".to_string(),
                path: "/Applications/Slack.app".to_string(),
            }],
            home_dir: "/Users/tester".to_string(),
        }
    }

    #[test]
    fn parses_clean_json_response() {
        let candidates = parse_candidates(
            r#"{
                "candidates": [
                    {
                        "canonical_action": "open_app",
                        "slots": { "app": "Safari" },
                        "missing_slots": [],
                        "confidence": 0.91,
                        "clarification_needed": false
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].canonical_action, CanonicalAction::OpenApp);
        assert_eq!(
            candidates[0].slots.get("app").map(String::as_str),
            Some("Safari")
        );
    }

    #[test]
    fn extracts_json_when_provider_adds_wrapper_text() {
        let candidates = parse_candidates(
            r#"Here is the structured result:
            {
                "candidates": [
                    {
                        "canonical_action": "trash_path",
                        "slots": { "path": "~/Desktop/test.txt" },
                        "missing_slots": [],
                        "confidence": 0.83,
                        "clarification_needed": false
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].canonical_action, CanonicalAction::TrashPath);
    }

    #[test]
    fn ignores_zero_confidence_candidates() {
        let candidates = parse_candidates(
            r#"{
                "candidates": [
                    {
                        "canonical_action": "open_app",
                        "slots": { "app": "Safari" },
                        "missing_slots": [],
                        "confidence": 0.0,
                        "clarification_needed": false
                    }
                ]
            }"#,
        )
        .unwrap();

        assert!(candidates.is_empty());
    }

    #[test]
    fn infers_missing_required_slots_from_action_contract() {
        let candidates = parse_candidates(
            r#"{
                "candidates": [
                    {
                        "canonical_action": "open_app",
                        "slots": {},
                        "missing_slots": [],
                        "confidence": 0.82,
                        "clarification_needed": false
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].missing_slots, vec!["app".to_string()]);
        assert!(candidates[0].clarification_needed);
    }

    #[test]
    fn repairs_app_slot_from_input_when_machine_match_is_unique() {
        let candidates = repair_candidates(
            vec![CandidateIntent {
                family: crate::intent_language::IntentFamily::AppOpen,
                canonical_action: CanonicalAction::OpenApp,
                slots: BTreeMap::new(),
                missing_slots: vec!["app".to_string()],
                confidence: 0.9,
                clarification_needed: true,
                risk_baseline: crate::models::RiskLevel::R0,
                executor_family: crate::intent_language::ExecutorFamily::App,
                source: InterpretationSource::Provider,
            }],
            "spin up safari",
            &machine_fixture(),
        );

        assert_eq!(
            candidates[0].slots.get("app").map(String::as_str),
            Some("Safari")
        );
        assert!(candidates[0].missing_slots.is_empty());
        assert!(!candidates[0].clarification_needed);
    }

    #[test]
    fn repairs_service_and_browser_slots_from_input_when_unique() {
        let candidates = repair_candidates(
            vec![CandidateIntent {
                family: crate::intent_language::IntentFamily::ServiceOpen,
                canonical_action: CanonicalAction::OpenService,
                slots: BTreeMap::new(),
                missing_slots: vec!["service".to_string()],
                confidence: 0.9,
                clarification_needed: true,
                risk_baseline: crate::models::RiskLevel::R1,
                executor_family: crate::intent_language::ExecutorFamily::Browser,
                source: InterpretationSource::Provider,
            }],
            "take me to youtube in google chrome",
            &machine_fixture(),
        );

        assert_eq!(
            candidates[0].slots.get("service").map(String::as_str),
            Some("youtube")
        );
        assert_eq!(
            candidates[0].slots.get("browser").map(String::as_str),
            Some("Google Chrome")
        );
        assert!(candidates[0].missing_slots.is_empty());
        assert!(!candidates[0].clarification_needed);
    }

    #[test]
    fn promotes_browser_new_tab_to_open_service_when_service_is_named() {
        let candidates = repair_candidates(
            vec![CandidateIntent {
                family: crate::intent_language::IntentFamily::BrowserTab,
                canonical_action: CanonicalAction::BrowserNewTab,
                slots: BTreeMap::new(),
                missing_slots: vec![],
                confidence: 0.9,
                clarification_needed: false,
                risk_baseline: crate::models::RiskLevel::R1,
                executor_family: crate::intent_language::ExecutorFamily::Browser,
                source: InterpretationSource::Provider,
            }],
            "take me to youtube in chrome",
            &machine_fixture(),
        );

        assert_eq!(candidates[0].canonical_action, CanonicalAction::OpenService);
        assert_eq!(
            candidates[0].slots.get("service").map(String::as_str),
            Some("youtube")
        );
        assert_eq!(
            candidates[0].slots.get("browser").map(String::as_str),
            Some("Google Chrome")
        );
    }

    #[test]
    fn repairs_trash_path_slot_from_input() {
        let candidates = repair_candidates(
            vec![CandidateIntent {
                family: crate::intent_language::IntentFamily::PathTrash,
                canonical_action: CanonicalAction::TrashPath,
                slots: BTreeMap::new(),
                missing_slots: vec!["path".to_string()],
                confidence: 0.9,
                clarification_needed: true,
                risk_baseline: crate::models::RiskLevel::R2,
                executor_family: crate::intent_language::ExecutorFamily::Filesystem,
                source: InterpretationSource::Provider,
            }],
            "toss ~/Desktop/test.txt",
            &machine_fixture(),
        );

        assert_eq!(
            candidates[0].slots.get("path").map(String::as_str),
            Some("~/Desktop/test.txt")
        );
        assert!(candidates[0].missing_slots.is_empty());
        assert!(!candidates[0].clarification_needed);
    }

    #[test]
    fn promotes_open_path_to_trash_for_trash_verbs() {
        let candidates = repair_candidates(
            vec![CandidateIntent {
                family: crate::intent_language::IntentFamily::PathOpen,
                canonical_action: CanonicalAction::OpenPath,
                slots: BTreeMap::new(),
                missing_slots: vec!["path".to_string()],
                confidence: 0.9,
                clarification_needed: true,
                risk_baseline: crate::models::RiskLevel::R0,
                executor_family: crate::intent_language::ExecutorFamily::Path,
                source: InterpretationSource::Provider,
            }],
            "bin ~/Downloads/example.txt",
            &machine_fixture(),
        );

        assert_eq!(candidates[0].canonical_action, CanonicalAction::TrashPath);
        assert_eq!(
            candidates[0].slots.get("path").map(String::as_str),
            Some("~/Downloads/example.txt")
        );
        assert!(candidates[0].missing_slots.is_empty());
        assert!(!candidates[0].clarification_needed);
    }
}
