use serde::{Deserialize, Serialize};

// ── Command classification ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    AppControl,
    MixedWorkflow,
    LocalSystem,
    Filesystem,
    UiAutomation,
    ShellExecution,
    Settings,
    Query,
    Unknown,
}

// ── Risk levels ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    #[serde(rename = "R0")]
    R0,
    #[serde(rename = "R1")]
    R1,
    #[serde(rename = "R2")]
    R2,
    #[serde(rename = "R3")]
    R3,
}

// ── Approval status ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    NotRequired,
    Pending,
    Approved,
    Denied,
}

// ── Execution outcome ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionOutcome {
    Success,
    RecoverableFailure,
    Blocked,
    TimedOut,
    PartialSuccess,
}

// ── Resolved action ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResolvedAction {
    OpenUrl {
        url: String,
        browser_bundle: String,
        browser_name: String,
    },
    OpenApp {
        bundle_id: String,
        app_name: String,
    },
    QuitApp {
        bundle_id: String,
        app_name: String,
    },
    AppleScriptTemplate {
        script: String,
        template_id: String,
    },
    OpenSystemPreferences {
        pane_url: String,
    },
    OpenPath {
        path: String,
    },
    CreateFolder {
        path: String,
    },
    MovePath {
        source_path: String,
        destination_path: String,
    },
}

// ── Resolved route ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRoute {
    pub label: String,
    pub description: String,
    pub action: ResolvedAction,
}

// ── Parsed command ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub id: String,
    pub raw_input: String,
    pub normalized: String,
    pub kind: CommandKind,
    pub routes: Vec<ResolvedRoute>,
    pub risk: RiskLevel,
    pub requires_approval: bool,
    pub approval_status: ApprovalStatus,
    pub unresolved_message: Option<String>,
}

// ── Execution event ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionEventKind {
    Started,
    Progress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub id: String,
    pub command_id: String,
    pub timestamp: String,
    pub kind: ExecutionEventKind,
    pub message: String,
}

// ── History entry ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: ParsedCommand,
    pub outcome: ExecutionOutcome,
    pub execution_events: Vec<ExecutionEvent>,
    pub duration_ms: u64,
    pub inverse_action: Option<ResolvedAction>,
    pub timestamp: String,
}

// ── Execution result ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub command_id: String,
    pub outcome: ExecutionOutcome,
    pub message: String,
    pub human_message: String,
    pub duration_ms: u64,
    pub inverse_action: Option<ResolvedAction>,
}

// ── Machine info ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    pub name: String,
    pub bundle_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub bundle_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    pub hostname: String,
    pub username: String,
    pub os_version: String,
    pub architecture: String,
    pub installed_browsers: Vec<BrowserInfo>,
    pub installed_apps: Vec<AppInfo>,
    pub home_dir: String,
}

// ── Permission status ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermState {
    Granted,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub accessibility: PermState,
    pub apple_events: PermState,
}
