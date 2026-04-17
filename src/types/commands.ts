// Mirror of Rust models.rs — snake_case matches Tauri's default serde output.

export type CommandKind =
  | 'app_control'
  | 'mixed_workflow'
  | 'local_system'
  | 'filesystem'
  | 'ui_automation'
  | 'shell_execution'
  | 'settings'
  | 'query'
  | 'unknown';

export type RiskLevel = 'R0' | 'R1' | 'R2' | 'R3';

export type ApprovalStatus = 'not_required' | 'pending' | 'approved' | 'denied';

export type ExecutionOutcome =
  | 'success'
  | 'recoverable_failure'
  | 'blocked'
  | 'timed_out'
  | 'partial_success';

// ── Resolved actions ─────────────────────────────────────────────────────────

export interface OpenUrlAction {
  type: 'open_url';
  url: string;
  browser_bundle: string;
  browser_name: string;
}

export interface OpenAppAction {
  type: 'open_app';
  bundle_id: string;
  app_name: string;
}

export interface AppleScriptTemplateAction {
  type: 'apple_script_template';
  script: string;
  template_id: string;
}

export interface OpenSystemPreferencesAction {
  type: 'open_system_preferences';
  pane_url: string;
}

export interface OpenPathAction {
  type: 'open_path';
  path: string;
}

export type ResolvedAction =
  | OpenUrlAction
  | OpenAppAction
  | AppleScriptTemplateAction
  | OpenSystemPreferencesAction
  | OpenPathAction;

// ── Route / command ──────────────────────────────────────────────────────────

export interface ResolvedRoute {
  label: string;
  description: string;
  action: ResolvedAction;
}

export interface ParsedCommand {
  id: string;
  raw_input: string;
  normalized: string;
  kind: CommandKind;
  routes: ResolvedRoute[];
  risk: RiskLevel;
  requires_approval: boolean;
  approval_status: ApprovalStatus;
}

export interface ExecutionResult {
  command_id: string;
  outcome: ExecutionOutcome;
  message: string;
  human_message: string;
  duration_ms: number;
  inverse_action?: ResolvedAction;
}

// ── Machine info ─────────────────────────────────────────────────────────────

export interface BrowserInfo {
  name: string;
  bundle_id: string;
  path: string;
}

export interface MachineInfo {
  hostname: string;
  username: string;
  installed_browsers: BrowserInfo[];
  home_dir: string;
}

// ── Permissions ──────────────────────────────────────────────────────────────

export type PermState = 'granted' | 'denied' | 'unknown';

export interface PermissionStatus {
  accessibility: PermState;
  apple_events: PermState;
}

// ── Provider keys ─────────────────────────────────────────────────────────────
// Only masked status is ever returned from Rust — never the raw key value.

export type KeyStatus = 'set' | 'not_set' | 'access_denied';

export interface ProviderKeyStatus {
  provider: string;
  status: KeyStatus;
}
