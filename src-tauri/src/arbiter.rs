use serde::{Deserialize, Serialize};

use crate::intent_language::CandidateIntent;
use crate::models::RiskLevel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArbitrationDecision {
    Execute,
    Clarify,
    OfferChoices,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationResult {
    pub decision: ArbitrationDecision,
    pub chosen_index: Option<usize>,
    pub explanation: String,
}

pub fn decide(candidates: &[CandidateIntent]) -> ArbitrationResult {
    if candidates.is_empty() {
        return ArbitrationResult {
            decision: ArbitrationDecision::Deny,
            chosen_index: None,
            explanation: "No candidate intent reached the executable contract.".to_string(),
        };
    }

    let best = &candidates[0];

    if !best.missing_slots.is_empty() || best.clarification_needed {
        return ArbitrationResult {
            decision: ArbitrationDecision::Clarify,
            chosen_index: Some(0),
            explanation: "A likely task family exists, but required slots still need repair.".to_string(),
        };
    }

    if candidates.len() > 1 {
        let second = &candidates[1];
        if (best.confidence - second.confidence).abs() < 0.10 && best.family != second.family {
            return ArbitrationResult {
                decision: ArbitrationDecision::OfferChoices,
                chosen_index: None,
                explanation: "Multiple materially different candidate actions remain plausible.".to_string(),
            };
        }
    }

    if best.confidence >= 0.82 && best.risk_baseline <= RiskLevel::R1 {
        return ArbitrationResult {
            decision: ArbitrationDecision::Execute,
            chosen_index: Some(0),
            explanation: "One candidate dominates and is safe enough to continue.".to_string(),
        };
    }

    ArbitrationResult {
        decision: ArbitrationDecision::Clarify,
        chosen_index: Some(0),
        explanation: "A probable task exists, but execution certainty is still below the fast-path threshold.".to_string(),
    }
}
