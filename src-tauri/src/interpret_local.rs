use crate::arbiter::CandidateIntent;
use crate::intent_ontology;
use crate::models::RiskLevel;

pub fn interpret(input: &str) -> Vec<CandidateIntent> {
    let normalized = input.trim().to_lowercase();
    if normalized.is_empty() {
        return vec![];
    }

    if let Some(app) = remainder_after(&normalized, &["close ", "quit ", "exit ", "shut "]) {
        return vec![CandidateIntent {
            family: "app.close".to_string(),
            canonical_action: format!("quit_app:{app}"),
            missing_slots: if app.is_empty() { vec!["app".to_string()] } else { vec![] },
            confidence: if app.is_empty() { 0.55 } else { 0.92 },
            clarification_needed: app.is_empty(),
            risk_baseline: RiskLevel::R1,
        }];
    }

    if let Some(app) = remainder_after(&normalized, &["open ", "launch ", "start ", "run "]) {
        let family = if looks_like_path(&app) { "path.open" } else { "app.open" };
        let slot = if family == "path.open" { "path" } else { "app" };
        return vec![CandidateIntent {
            family: family.to_string(),
            canonical_action: if family == "path.open" {
                format!("open_path:{app}")
            } else {
                format!("open_app:{app}")
            },
            missing_slots: if app.is_empty() { vec![slot.to_string()] } else { vec![] },
            confidence: if app.is_empty() { 0.55 } else { 0.91 },
            clarification_needed: app.is_empty(),
            risk_baseline: RiskLevel::R0,
        }];
    }

    let surface_matches = intent_ontology::actions_for_surface_token(&normalized);
    surface_matches
        .into_iter()
        .map(|action| CandidateIntent {
            family: action.family.to_string(),
            canonical_action: action.canonical_action.to_string(),
            missing_slots: action.required_slots.iter().map(|slot| slot.to_string()).collect(),
            confidence: 0.60,
            clarification_needed: !action.required_slots.is_empty(),
            risk_baseline: action.risk_baseline.clone(),
        })
        .collect()
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
