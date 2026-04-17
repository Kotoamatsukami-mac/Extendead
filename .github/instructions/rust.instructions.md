---
applyTo: "src-tauri/**/*.rs"
---

Rust is the authority layer.

## Rust ownership

Rust owns:
- command routing
- machine scan
- settings + secure credential access
- local resolver
- shell policy
- validator
- risk engine
- executor
- event streaming
- history persistence
- undo generation
- AppleScript bridge
- permission inspection

## Code style

- Strong enums over stringly-typed logic
- Result-based error flow
- Clear domain modules
- Keep command structs serializable to frontend
- Avoid giant god files
- Prefer total functions and explicit matching
- Use typed event payloads
- Make destructive actions impossible to invoke without validator approval

## Required domains

Create separate modules for:
- machine
- permissions
- commands
- parser
- resolver
- validator
- risk
- executor
- history
- planner
- models
- applescript
- ui_automation
- events
- config
- errors

## Persistence rules

Persist:
- machine signature cache
- settings
- policy toggles
- history records
- reversible actions metadata

Do not persist:
- transient streaming state
- command draft text
- unresolved temporary UI state

## Secrets

Use macOS keychain-backed storage for provider keys on macOS.
Do not pass secret material back to the frontend except masked status.
