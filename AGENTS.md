# Extendead agent contract

This repository is agent-sensitive. Do not improvise product shape.

## Absolute rules

- Do not convert Extendead into a chat app
- Do not introduce transcript UI
- Do not introduce arbitrary shell execution
- Do not add features not requested
- Do not widen scope to browser automation frameworks, daemons, or plugin marketplaces
- Do not replace deterministic local logic with model calls
- Do not store secrets in plain files or frontend state

## Build priority

1. command bar strip
2. machine scan
3. local resolver
4. typed validator
5. approval gate
6. executor
7. event stream
8. history + undo
9. interpreter fallback
10. UI automation helpers

## Required output style

When proposing code changes:
- state affected files
- state why the change is needed
- keep diffs minimal
- avoid speculative rewrites
- preserve existing stable architecture

## Failure behavior

Every executor returns one of:
- success
- recoverable_failure
- blocked
- timed_out
- partial_success

Never swallow an error.
Never silently downgrade a dangerous action into execution.
If a step fails, report:
- machine-readable cause
- human-readable explanation
- safe next action

## Shell rules

V1 shell policy:
- no arbitrary shell strings
- no &&, ||, ;, subshells, redirects, or command substitution
- no sudo in v1
- no unknown binaries in v1
- use validated command templates only

## macOS automation rules

Use:
1. app/scriptable AppleScript
2. System Events UI scripting only when required

Selector preference:
1. bundle id
2. process name
3. window title
4. accessibility role
5. button/menu text
6. index selector as last resort only

## Permission rules

Treat Accessibility / Apple Events status as first-class state.
Missing permission is not an exception-only detail; it is a visible product state.
