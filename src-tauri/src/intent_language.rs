use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::models::RiskLevel;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IntentFamily {
    #[serde(rename = "app.open")]
    AppOpen,
    #[serde(rename = "app.close")]
    AppClose,
    #[serde(rename = "path.open")]
    PathOpen,
    #[serde(rename = "folder.create")]
    FolderCreate,
    #[serde(rename = "file.move")]
    FileMove,
    #[serde(rename = "service.open")]
    ServiceOpen,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CanonicalAction {
    #[serde(rename = "open_app")]
    OpenApp,
    #[serde(rename = "quit_app")]
    QuitApp,
    #[serde(rename = "open_path")]
    OpenPath,
    #[serde(rename = "create_folder")]
    CreateFolder,
    #[serde(rename = "move_path")]
    MovePath,
    #[serde(rename = "open_service")]
    OpenService,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExecutorFamily {
    #[serde(rename = "app")]
    App,
    #[serde(rename = "path")]
    Path,
    #[serde(rename = "filesystem")]
    Filesystem,
    #[serde(rename = "browser")]
    Browser,
    #[serde(rename = "settings")]
    Settings,
    #[serde(rename = "ui_automation")]
    UiAutomation,
    #[serde(rename = "workflow")]
    Workflow,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InterpretationSource {
    #[serde(rename = "local_pattern")]
    LocalPattern,
    #[serde(rename = "local_ontology")]
    LocalOntology,
    #[serde(rename = "provider")]
    Provider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateIntent {
    pub family: IntentFamily,
    pub canonical_action: CanonicalAction,
    pub slots: BTreeMap<String, String>,
    pub missing_slots: Vec<String>,
    pub confidence: f32,
    pub clarification_needed: bool,
    pub risk_baseline: RiskLevel,
    pub executor_family: ExecutorFamily,
    pub source: InterpretationSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlanStep {
    pub family: IntentFamily,
    pub canonical_action: CanonicalAction,
    pub slots: BTreeMap<String, String>,
    pub risk_baseline: RiskLevel,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionPlanStep>,
}
