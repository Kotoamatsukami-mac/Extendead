# Extendead Architecture

**Desktop Application** (Tauri + React)

## Principle

**Single source of truth: Rust backend.**

The React frontend is a **thin UI layer** that communicates with the Tauri/Rust backend via IPC. No business logic, parsing, or command processing in React.

```
User Input
    ↓
React UI (display only)
    ↓
Tauri IPC invoke()
    ↓
Rust Backend (source of truth)
    ├── Parser: tokenization → intent extraction
    ├── Resolver: intent → possible routes
    ├── Executor: execute resolved action
    └── Validator: safety/risk checks
    ↓
Tauri IPC response
    ↓
React displays result
```

## Directory Structure

### Frontend (`src/`)

**React + TypeScript UI layer**

```
src/
├── App.tsx                      # Root shell, window management
├── main.tsx                     # React entry point
├── index.css                    # Design tokens + reset
├── components/                  # Functional UI components
│   ├── LoungeStrip.tsx         # Main input bar
│   ├── ExpandedConsole.tsx     # Detailed view
│   ├── DeveloperPanel.tsx      # Diagnostics UI
│   ├── WindowDragHandle.tsx    # Tauri drag region
│   ├── RouteSelector.tsx       # Choose execution path
│   ├── HistoryList.tsx         # Past commands
│   ├── EventTimeline.tsx       # Execution events
│   ├── PermissionBanner.tsx    # Permission status
│   ├── RiskBadge.tsx           # Risk indicator
│   └── ConfirmationRail.tsx    # Approval UI
├── hooks/                       # **IPC Bridges only**
│   ├── useCommandBridge.ts     # invoke() → Rust commands
│   ├── useMachineState.ts      # Machine info from Rust
│   └── usePermissionStatus.ts  # Permission checks from Rust
└── types/                       # **Mirrors Rust types**
    ├── commands.ts             # ParsedCommand, ResolvedRoute, etc
    └── events.ts               # ExecutionEvent, ExecutionEventPayload
```

**Rules for React code:**
- ✓ Call `invoke()` from useCommandBridge
- ✓ Transform Rust responses for UI display
- ✓ Handle UI state (focused field, expanded view, etc)
- ✗ NO business logic
- ✗ NO command parsing
- ✗ NO semantic pipeline
- ✗ NO API handling

### Backend (`src-tauri/src/`)

**Rust + Tauri backend (actual command processing)**

```
src-tauri/src/
├── commands.rs               # Tauri IPC command handlers
├── parser.rs                 # Tokenize → intent
├── resolver.rs               # Intent → routes
├── executor.rs               # Execute action safely
├── validator.rs              # Validate routes before execution
├── risk.rs                   # Risk classification
├── machine.rs                # System state (apps, browsers, etc)
├── applescript.rs            # Native macOS operations
├── modes.rs                  # Built-in workflows
├── semantic.rs               # SemanticFrame decomposition
├── models.rs                 # Shared types
└── [other modules]
```

**Phases:**

1. **Phase 1**: Semantic decomposition
   - `semantic.rs`: SemanticFrame (verb, target, scope, qualifier, temporal, intensity)
   - Input: raw string → Output: universal shape

2. **Phase 2**: Built-in modes
   - `modes.rs`: Study, Focus, Reading modes
   - Multi-step workflows with constraint satisfaction
   - Proof that pipeline architecture works

3. **Phase 3**: Parser migration
   - Migrate parser to produce SemanticFrame
   - Add constraint hierarchy
   - Add context retrieval
   - Add reasoning-effort selection

4. **Phase 4**: Full semantic pipeline
   - 10-stage constraint-based reasoning
   - Full verification loop
   - API enrichment for edge cases

## Data Flow (Example: "open Safari")

```
LoungeStrip.tsx (user types "open Safari")
    ↓
onSubmit(input)
    ↓
useCommandBridge.parseCommand("open Safari")
    ↓
invoke("parse_command", {input})
    ↓
Rust: commands::parse_command()
    ├── parser::parse() → ParsedCommand
    ├── resolver::resolve() → ResolvedRoute[]
    ├── risk::score() → RiskLevel
    └── returns ParsedCommand
    ↓
React displays:
├── Intent: "open app"
├── Routes: [Safari, other browsers]
├── Risk: R0 (safe)
└── Buttons: Execute, Clarify, etc
```

## Key Concepts

### ParsedCommand
Complete representation of user intent:
- `raw_input`: Original text
- `kind`: CommandKind (app_control, query, filesystem, etc)
- `routes`: Possible execution paths (ResolvedRoute[])
- `risk`: RiskLevel (R0-R3)
- `requires_approval`: boolean

### ResolvedRoute
A single executable path:
- `label`: Human-readable name
- `description`: What will happen
- `action`: ResolvedAction (OpenApp, QuitApp, RunPlan, etc)

### ResolvedAction (enum)
Concrete action to execute:
- `OpenApp { bundle_id, app_name }`
- `OpenUrl { url, browser_bundle }`
- `AppleScriptTemplate { script, template_id }`
- `ActivateMode { mode_id, mode_name }`
- `MovePath { source_path, destination_path }`
- etc

### ExecutionEvent
Real-time progress:
```
Started → Progress → Progress → Completed
```
Emitted via Tauri event listener.

### Capability Ontology
See `CAPABILITY_ONTOLOGY.md` for:
- All supported commands
- Apps/browsers
- Modes
- Constraint patterns
- Risk classification
- Ambiguity gradient

## Window Management

**Tauri window:**
- No decorations (native titlebar hidden)
- Transparent background
- Always on top
- 800x76px (expandable to 800x400px)

**Drag:**
- `data-tauri-drag-region` attribute on WindowDragHandle
- Tauri native window movement (not JavaScript)
- Pin/unpin button controls lock state

## Testing

**React side:**
- Component snapshots
- IPC call mocking
- UI state transitions

**Rust side:**
- Unit tests per module
- Integration tests for full flow
- Run: `cargo test --lib`

## Next Steps

1. **Mode executor**: Parallel execution with concurrent groups
2. **Plan preview UI**: Show steps before execution
3. **Parser migration**: Use SemanticFrame as intermediate
4. **Constraint system**: Full hierarchy implementation
5. **Context retrieval**: Deictic reference resolution
6. **Reasoning selection**: Dynamic complexity assessment

---

**This is a Tauri desktop app. All logic flows through the Rust backend. React is UI only.**
