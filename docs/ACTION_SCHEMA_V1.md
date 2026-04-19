# Extendead Action Schema V1

This document describes the current command, route, execution, history, and failure contract for Extendead.

It should track the live Rust/TypeScript model layer.
If the code changes, update this document.

## Purpose

The model layer may interpret requests.
It may **not** invent its own execution format.
All interpreted commands must normalize into the same internal schema and pass through the same validation and risk pipeline.

## Core command object

Current frontend mirror: `src/types/commands.ts`
Current backend source: Rust models consumed by `commands.rs`

### ParsedCommand

```ts
interface ParsedCommand {
  id: string;
  raw_input: string;
  normalized: string;
  kind:
    | 'app_control'
    | 'mixed_workflow'
    | 'local_system'
    | 'filesystem'
    | 'ui_automation'
    | 'shell_execution'
    | 'settings'
    | 'query'
    | 'unknown';
  routes: ResolvedRoute[];
  risk: 'R0' | 'R1' | 'R2' | 'R3';
  requires_approval: boolean;
  approval_status: 'not_required' | 'pending' | 'approved' | 'denied';
}
```

## Route object

A command may resolve to zero, one, or many routes.

```ts
interface ResolvedRoute {
  label: string;
  description: string;
  action: ResolvedAction;
}
```

### Current action types

```ts
type ResolvedAction =
  | { type: 'open_url'; url: string; browser_bundle: string; browser_name: string }
  | { type: 'open_app'; bundle_id: string; app_name: string }
  | { type: 'apple_script_template'; script: string; template_id: string }
  | { type: 'open_system_preferences'; pane_url: string }
  | { type: 'open_path'; path: string };
```

## Execution result

```ts
interface ExecutionResult {
  command_id: string;
  outcome:
    | 'success'
    | 'recoverable_failure'
    | 'blocked'
    | 'timed_out'
    | 'partial_success';
  message: string;
  human_message: string;
  duration_ms: number;
  inverse_action?: ResolvedAction;
}
```

## History entry

```ts
interface HistoryEntry {
  command: ParsedCommand;
  outcome: ExecutionOutcome;
  duration_ms: number;
  inverse_action?: ResolvedAction;
  timestamp: string;
}
```

## Approval contract

- `R0`: no approval by default
- `R1`: low risk, usually no approval unless action semantics justify it
- `R2`: approval required
- `R3`: approval required and destructive or security-sensitive by nature

Current code assigns risk after parse/resolve via `risk::annotate`.
Approval state must be explicit and never inferred by the UI alone.

## Route-selection contract

- `0 routes` = unresolved or unsupported command
- `1 route` = auto-execute when approval is not required
- `>1 routes` = explicit route selection UI

The shell must not pretend uncertainty does not exist.
If multiple viable routes exist, the user should choose.

## Typed unresolved/failure doctrine

The current generic fallback of `Command not recognised` is not sufficient as the long-term contract.
V1 should move toward typed unresolved/failure reasons.

### Minimum failure categories

- `unsupported_local_command`
- `app_not_installed`
- `permission_required`
- `malformed_request`
- `ambiguous_target`
- `route_unavailable`
- `execution_blocked`

### UX rule

The user-facing message should explain **why** the command did not run, not just that it failed.

Bad:

- Command not recognised

Good:

- Safari is not installed on this Mac
- Accessibility permission is required for that action
- I could not resolve which browser you meant
- That command is outside current local coverage

## Provider/interpreter contract

A provider may help with:

- intent understanding
- typo recovery
- argument extraction
- route proposal
- ambiguity reduction

A provider may **not**:

- fabricate machine state
- fabricate installed apps
- fabricate permissions
- fabricate execution success
- bypass validation, risk, approval, or history

## V1 next schema additions

These are not all live yet, but they are the intended next contract extensions:

- explicit unresolved result type for parse/resolve failures
- explicit permission requirement reason in result payload
- optional confidence score for interpreter-suggested routes
- optional provenance field: deterministic vs interpreted
- structured reason code on `ExecutionResult`

## Source of truth rule

When schema docs and live code disagree, code wins temporarily.
Then fix the docs immediately.
