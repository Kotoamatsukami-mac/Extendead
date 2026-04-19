use crate::models::RiskLevel;

#[derive(Debug, Clone)]
pub struct ActionDefinition {
    pub family: &'static str,
    pub canonical_action: &'static str,
    pub surface_synonyms: &'static [&'static str],
    pub required_slots: &'static [&'static str],
    pub optional_slots: &'static [&'static str],
    pub clarification_prompts: &'static [&'static str],
    pub risk_baseline: RiskLevel,
    pub reversible: bool,
}

pub static ACTIONS: &[ActionDefinition] = &[
    ActionDefinition {
        family: "app.open",
        canonical_action: "open_app",
        surface_synonyms: &["open", "launch", "start", "run"],
        required_slots: &["app"],
        optional_slots: &[],
        clarification_prompts: &["Which app should I open?"],
        risk_baseline: RiskLevel::R0,
        reversible: false,
    },
    ActionDefinition {
        family: "app.close",
        canonical_action: "quit_app",
        surface_synonyms: &["close", "quit", "exit", "shut"],
        required_slots: &["app"],
        optional_slots: &[],
        clarification_prompts: &["Which app should I close?"],
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: "path.open",
        canonical_action: "open_path",
        surface_synonyms: &["open", "show", "reveal"],
        required_slots: &["path"],
        optional_slots: &[],
        clarification_prompts: &["Which file or folder should I open?"],
        risk_baseline: RiskLevel::R0,
        reversible: false,
    },
    ActionDefinition {
        family: "folder.create",
        canonical_action: "create_folder",
        surface_synonyms: &["create folder", "make folder", "new folder"],
        required_slots: &["name"],
        optional_slots: &["base_path"],
        clarification_prompts: &["What should I name the folder?", "Where should I create it?"],
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: "file.move",
        canonical_action: "move_path",
        surface_synonyms: &["move", "put", "place"],
        required_slots: &["source", "destination"],
        optional_slots: &[],
        clarification_prompts: &["What should I move?", "Where should I move it?"],
        risk_baseline: RiskLevel::R2,
        reversible: true,
    },
];

pub fn all_actions() -> &'static [ActionDefinition] {
    ACTIONS
}

pub fn actions_for_surface_token(token: &str) -> Vec<&'static ActionDefinition> {
    let normalized = token.trim().to_lowercase();
    ACTIONS
        .iter()
        .filter(|action| {
            action.surface_synonyms.iter().any(|surface| {
                normalized == *surface || normalized.starts_with(surface)
            })
        })
        .collect()
}
