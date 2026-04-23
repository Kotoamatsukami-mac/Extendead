use crate::intent_language::{CanonicalAction, ExecutorFamily, IntentFamily};
use crate::models::RiskLevel;

#[derive(Debug, Clone)]
pub struct ActionDefinition {
    pub family: IntentFamily,
    pub canonical_action: CanonicalAction,
    pub surface_synonyms: &'static [&'static str],
    pub required_slots: &'static [&'static str],
    pub optional_slots: &'static [&'static str],
    pub clarification_prompts: &'static [&'static str],
    pub executor_family: ExecutorFamily,
    pub risk_baseline: RiskLevel,
    pub reversible: bool,
}

pub static ACTIONS: &[ActionDefinition] = &[
    ActionDefinition {
        family: IntentFamily::AppOpen,
        canonical_action: CanonicalAction::OpenApp,
        surface_synonyms: &["open", "launch", "start", "run"],
        required_slots: &["app"],
        optional_slots: &[],
        clarification_prompts: &["Which app should I open?"],
        executor_family: ExecutorFamily::App,
        risk_baseline: RiskLevel::R0,
        reversible: false,
    },
    ActionDefinition {
        family: IntentFamily::AppClose,
        canonical_action: CanonicalAction::QuitApp,
        surface_synonyms: &["close", "quit", "exit", "shut"],
        required_slots: &["app"],
        optional_slots: &[],
        clarification_prompts: &["Which app should I close?"],
        executor_family: ExecutorFamily::App,
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: IntentFamily::PathOpen,
        canonical_action: CanonicalAction::OpenPath,
        surface_synonyms: &["open", "show", "reveal"],
        required_slots: &["path"],
        optional_slots: &[],
        clarification_prompts: &["Which file or folder should I open?"],
        executor_family: ExecutorFamily::Path,
        risk_baseline: RiskLevel::R0,
        reversible: false,
    },
    ActionDefinition {
        family: IntentFamily::FolderCreate,
        canonical_action: CanonicalAction::CreateFolder,
        surface_synonyms: &["create folder", "make folder", "new folder"],
        required_slots: &["name"],
        optional_slots: &["base_path"],
        clarification_prompts: &[
            "What should I name the folder?",
            "Where should I create it?",
        ],
        executor_family: ExecutorFamily::Filesystem,
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: IntentFamily::FileMove,
        canonical_action: CanonicalAction::MovePath,
        surface_synonyms: &["move", "put", "place"],
        required_slots: &["source", "destination"],
        optional_slots: &[],
        clarification_prompts: &["What should I move?", "Where should I move it?"],
        executor_family: ExecutorFamily::Filesystem,
        risk_baseline: RiskLevel::R2,
        reversible: true,
    },
    ActionDefinition {
        family: IntentFamily::ServiceOpen,
        canonical_action: CanonicalAction::OpenService,
        surface_synonyms: &["open", "watch", "browse", "visit"],
        required_slots: &["service"],
        optional_slots: &["browser"],
        clarification_prompts: &[
            "Which service should I open?",
            "Which browser should I use?",
        ],
        executor_family: ExecutorFamily::Browser,
        risk_baseline: RiskLevel::R1,
        reversible: false,
    },
    ActionDefinition {
        family: IntentFamily::BrowserTab,
        canonical_action: CanonicalAction::BrowserNewTab,
        surface_synonyms: &["new tab", "open new tab"],
        required_slots: &[],
        optional_slots: &["browser"],
        clarification_prompts: &["Which browser should I use?"],
        executor_family: ExecutorFamily::Browser,
        risk_baseline: RiskLevel::R1,
        reversible: false,
    },
    ActionDefinition {
        family: IntentFamily::BrowserTab,
        canonical_action: CanonicalAction::BrowserCloseTab,
        surface_synonyms: &["close tab"],
        required_slots: &[],
        optional_slots: &["browser"],
        clarification_prompts: &["Which browser should I use?"],
        executor_family: ExecutorFamily::Browser,
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: IntentFamily::BrowserTab,
        canonical_action: CanonicalAction::BrowserReopenClosedTab,
        surface_synonyms: &["reopen tab", "reopen closed tab", "undo close tab"],
        required_slots: &[],
        optional_slots: &["browser"],
        clarification_prompts: &["Which browser should I use?"],
        executor_family: ExecutorFamily::Browser,
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: IntentFamily::DisplayBrightness,
        canonical_action: CanonicalAction::BrightnessUp,
        surface_synonyms: &["brightness up", "increase brightness", "raise brightness"],
        required_slots: &[],
        optional_slots: &[],
        clarification_prompts: &[],
        executor_family: ExecutorFamily::Settings,
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: IntentFamily::DisplayBrightness,
        canonical_action: CanonicalAction::BrightnessDown,
        surface_synonyms: &["brightness down", "decrease brightness", "dim"],
        required_slots: &[],
        optional_slots: &[],
        clarification_prompts: &[],
        executor_family: ExecutorFamily::Settings,
        risk_baseline: RiskLevel::R1,
        reversible: true,
    },
    ActionDefinition {
        family: IntentFamily::PathTrash,
        canonical_action: CanonicalAction::TrashPath,
        surface_synonyms: &["trash", "delete", "remove"],
        required_slots: &["path"],
        optional_slots: &[],
        clarification_prompts: &["Which file or folder should I move to Trash?"],
        executor_family: ExecutorFamily::Filesystem,
        risk_baseline: RiskLevel::R2,
        reversible: true,
    },
];

pub fn all_actions() -> &'static [ActionDefinition] {
    ACTIONS
}

pub fn action_for_canonical_action(
    canonical_action: CanonicalAction,
) -> Option<&'static ActionDefinition> {
    ACTIONS
        .iter()
        .find(|action| action.canonical_action == canonical_action)
}

pub fn actions_for_surface_token(token: &str) -> Vec<&'static ActionDefinition> {
    let normalized = token.trim().to_lowercase();
    ACTIONS
        .iter()
        .filter(|action| {
            action.surface_synonyms.iter().any(|surface| {
                normalized == *surface || normalized.starts_with(&format!("{surface} "))
            })
        })
        .collect()
}
