use std::collections::BTreeMap;

use crate::intent_language::{
    CandidateIntent, CanonicalAction, ExecutorFamily, IntentFamily, InterpretationSource,
};
use crate::intent_ontology;
use crate::models::RiskLevel;

pub fn interpret(input: &str) -> Vec<CandidateIntent> {
    let normalized = input.trim().to_lowercase();
    if normalized.is_empty() {
        return vec![];
    }

    if let Some(app) = remainder_after(&normalized, &["close ", "quit ", "exit ", "shut "]) {
        return vec![CandidateIntent {
            family: IntentFamily::AppClose,
            canonical_action: CanonicalAction::QuitApp,
            slots: slot_map(if app.is_empty() { None } else { Some(("app", app.as_str())) }),
            missing_slots: if app.is_empty() { vec!["app".to_string()] } else { vec![] },
            confidence: if app.is_empty() { 0.55 } else { 0.92 },
            clarification_needed: app.is_empty(),
            risk_baseline: RiskLevel::R1,
            executor_family: ExecutorFamily::App,
            source: InterpretationSource::LocalPattern,
        }];
    }

    if let Some(target) = remainder_after(&normalized, &["open ", "launch ", "start ", "run "]) {
        let is_path = looks_like_path(&target);
        let slot_name = if is_path { "path" } else { "app" };
        return vec![CandidateIntent {
            family: if is_path { IntentFamily::PathOpen } else { IntentFamily::AppOpen },
            canonical_action: if is_path {
                CanonicalAction::OpenPath
            } else {
                CanonicalAction::OpenApp
            },
            slots: slot_map(if target.is_empty() { None } else { Some((slot_name, target.as_str())) }),
            missing_slots: if target.is_empty() {
                vec![slot_name.to_string()]
            } else {
                vec![]
            },
            confidence: if target.is_empty() { 0.55 } else { 0.91 },
            clarification_needed: target.is_empty(),
            risk_baseline: RiskLevel::R0,
            executor_family: if is_path {
                ExecutorFamily::Path
            } else {
                ExecutorFamily::App
            },
            source: InterpretationSource::LocalPattern,
        }];
    }

    let surface_matches = intent_ontology::actions_for_surface_token(&normalized);
    surface_matches
        .into_iter()
        .map(|action| CandidateIntent {
            family: action.family,
            canonical_action: action.canonical_action,
            slots: BTreeMap::new(),
            missing_slots: action.required_slots.iter().map(|slot| slot.to_string()).collect(),
            confidence: 0.60,
            clarification_needed: !action.required_slots.is_empty(),
            risk_baseline: action.risk_baseline.clone(),
            executor_family: action.executor_family,
            source: InterpretationSource::LocalOntology,
        })
        .collect()
}

fn slot_map(entry: Option<(&str, &str)>) -> BTreeMap<String, String> {
    let mut slots = BTreeMap::new();
    if let Some((key, value)) = entry {
        slots.insert(key.to_string(), value.to_string());
    }
    slots
}

fn remainder_after(value: &str, prefixes: &[&str]) -> Option<String> {
    for prefix in prefixes {
        if let Some(rest) = value.strip_prefix(prefix) {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn looks_like_path(value: &str) -> bool {
    value.starts_with("~/")
        || value.starts_with('/')
        || value.contains('/')
        || matches!(value, "desktop" | "downloads" | "documents" | "home")
}
