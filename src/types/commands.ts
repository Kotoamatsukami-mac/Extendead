import type { ExecutionEvent } from './events';

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

export type UnresolvedCode =
  | 'unsupported_command'
  | 'unsupported_service'
  | 'browser_not_installed'
  | 'app_not_installed'
  | 'path_not_found'
  | 'source_path_not_found'
  | 'base_path_unresolved'
  | 'target_already_exists'
  | 'destination_path_unresolved'
  | 'destination_parent_missing'
  | 'permanent_delete_blocked'
  | 'ambiguous_target'
  | 'provider_configuration_required';

export type ExecutionOutcome =
  | 'success'
  | 'recoverable_failure'
  | 'blocked'
  | 'timed_out'
  | 'partial_success';

export type InterpretationDecision = 'execute' | 'clarify' | 'offer_choices' | 'deny';

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

export interface QuitAppAction {
  type: 'quit_app';
  bundle_id: string;
  app_name: string;
}

export interface HideAppAction {
  type: 'hide_app';
  bundle_id: string;
  app_name: string;
}

export interface ForceQuitAppAction {
  type: 'force_quit_app';
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

export interface CreateFolderAction {
  type: 'create_folder';
  path: string;
}

export interface MovePathAction {
  type: 'move_path';
  source_path: string;
  destination_path: string;
}

export interface ResolvedPlanStep {
  label: string;
  description: string;
  action: ResolvedAction;
  execution_group: string;
  risk: RiskLevel;
  requires_approval: boolean;
}

export interface RunPlanAction {
  type: 'run_plan';
  mode_name: string;
  steps: ResolvedPlanStep[];
}

export type ResolvedAction =
  | OpenUrlAction
  | OpenAppAction
  | QuitAppAction
  | HideAppAction
  | ForceQuitAppAction
  | AppleScriptTemplateAction
  | OpenSystemPreferencesAction
  | OpenPathAction
  | CreateFolderAction
  | MovePathAction
  | RunPlanAction;

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
  unresolved_code?: UnresolvedCode | null;
  unresolved_message?: string | null;
  interpretation_decision?: InterpretationDecision | null;
  clarification_message?: string | null;
  clarification_slots?: string[];
  choices?: string[];
}

export interface ExecutionResult {
  command_id: string;
  outcome: ExecutionOutcome;
  message: string;
  human_message: string;
  duration_ms: number;
  inverse_action?: ResolvedAction;
}

export interface HistoryEntry {
  command: ParsedCommand;
  outcome: ExecutionOutcome;
  execution_events?: ExecutionEvent[];
  duration_ms: number;
  inverse_action?: ResolvedAction;
  timestamp: string;
}

export interface BrowserInfo {
  name: string;
  bundle_id: string;
  path: string;
}

export interface AppInfo {
  name: string;
  bundle_id: string;
  path: string;
}

export interface MachineInfo {
  hostname: string;
  username: string;
  os_version: string;
  architecture: string;
  installed_browsers: BrowserInfo[];
  installed_apps: AppInfo[];
  home_dir: string;
}

export type PermState = 'granted' | 'denied' | 'unknown';

export interface PermissionStatus {
  accessibility: PermState;
  apple_events: PermState;
}

export interface AppConfig {
  always_on_top: boolean;
  max_history: number;
}

export type KeyStatus = 'set' | 'not_set' | 'access_denied';

export interface ProviderKeyStatus {
  provider: string;
  status: KeyStatus;
}

export interface ResultFeedback {
  message: string;
  type: 'success' | 'error';
}

export interface CommandSuggestion {
  id: string;
  family: string;
  canonical: string;
  detail: string;
}

export interface ServiceDefinition {
  id: string;
  display_name: string;
  aliases: string[];
  url: string;
  category: string;
}
