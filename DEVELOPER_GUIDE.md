# Developer Guide

## Quick Start

```bash
# Install dependencies
npm install

# Start dev server (Tauri + React)
npm run tauri dev

# Build for macOS
npm run tauri build

# Run tests
cargo test --lib          # Rust tests
npm run build             # TypeScript check + build
```

## Adding a New Command

**Example: "enable airplane mode"**

### 1. Rust Backend (commands.rs → resolver.rs)

Add pattern to `parser.rs`:
```rust
"enable airplane mode" → Intent {
    category: Settings,
    action: "enable",
    target: "airplane_mode",
    ...
}
```

Add resolver in `resolver.rs`:
```rust
fn resolve_settings(intent: &Intent) -> Vec<ResolvedRoute> {
    if intent.target == "airplane_mode" && intent.action == "enable" {
        return vec![
            ResolvedRoute {
                label: "Enable Airplane Mode",
                action: ResolvedAction::AppleScriptTemplate {
                    script: "tell application \"System Events\" to ...".to_string(),
                    template_id: "airplane_mode_enable",
                },
            }
        ];
    }
}
```

### 2. Test in Rust

```rust
#[test]
fn test_enable_airplane_mode() {
    let intent = parse_intent("enable airplane mode");
    let routes = resolve(&intent);
    assert!(routes.len() > 0);
    assert_eq!(routes[0].label, "Enable Airplane Mode");
}
```

Run: `cargo test --lib resolver`

### 3. React automatically displays it

No React code needed. When user types "enable airplane mode":
1. `useCommandBridge.parseCommand()` calls Rust
2. Rust returns ParsedCommand with routes
3. React displays routes in RouteSelector
4. User clicks Execute
5. `useCommandBridge.approveAndExecute()` calls Rust executor
6. Result displays in UI

**That's it. React is just a display layer.**

---

## Adding a Mode

**Example: Gaming Mode**

### 1. Define mode in `modes.rs`

```rust
fn gaming_mode() -> Mode {
    Mode {
        id: "gaming".to_string(),
        name: "Gaming Mode".to_string(),
        description: "Optimized for gaming: disable notifications, maximize performance".to_string(),
        groups: vec![
            ConcurrentGroup {
                label: "Disable interruptions".to_string(),
                steps: vec![
                    ModeStep {
                        action: "enable_dnd".to_string(),
                        target: None,
                        params: HashMap::new(),
                    },
                    // ... more steps
                ],
            },
        ],
    }
}

pub fn builtin_modes() -> Vec<Mode> {
    vec![study_mode(), focus_mode(), reading_mode(), gaming_mode()]
}
```

### 2. Add test

```rust
#[test]
fn test_gaming_mode_exists() {
    let mode = get_mode("gaming").expect("gaming mode");
    assert_eq!(mode.name, "Gaming Mode");
    assert!(!mode.groups.is_empty());
}
```

### 3. User can say "gaming mode"

React shows it automatically when user types. No UI changes needed.

---

## Debugging

### See what Rust is doing

Add to `commands.rs`:
```rust
pub async fn debug_interpret_local(input: String) -> Result<String, String> {
    let intent = match parser::parse(&input) {
        Some(i) => i,
        None => return Err("Parse failed".to_string()),
    };
    Ok(format!("Intent: {:?}", intent))
}
```

In React (DeveloperPanel.tsx):
```tsx
const output = await bridge.debugInterpretLocal("your input");
console.log(output);
```

### Check machine state

```tsx
const info = await bridge.getMachineInfo();
console.log(info.installed_apps);
console.log(info.installed_browsers);
```

---

## Common Mistakes

### ❌ DON'T: Add logic to React

```tsx
// Wrong - parsing in React
const intent = input.split(' ');
const verb = intent[0];
const target = intent[1];
```

### ✅ DO: Call Rust

```tsx
// Correct - Rust parses
const cmd = await invoke('parse_command', { input });
```

---

### ❌ DON'T: Duplicate type definitions

```tsx
// Wrong - type exists in Rust
interface ParsedCommand {
    id: string;
    raw_input: string;
    // ... duplicate fields
}
```

### ✅ DO: Mirror from Rust

```tsx
// Correct - types generated/synced from Rust
import type { ParsedCommand } from '../types/commands';
```

---

## Architecture Rules

1. **Rust is source of truth** - all business logic lives there
2. **React is display layer** - transforms Rust output for UI
3. **IPC is the boundary** - everything crosses via invoke()
4. **Types must align** - Rust models ↔ TypeScript types
5. **No web code** - this is a desktop app (Tauri)

---

## File Locations

- **Add command logic** → `src-tauri/src/parser.rs` or `resolver.rs`
- **Add AppleScript** → `src-tauri/src/applescript.rs` (template enum)
- **Add mode** → `src-tauri/src/modes.rs`
- **Add UI component** → `src/components/*.tsx`
- **Add IPC hook** → `src/hooks/useCommandBridge.ts` (if new Tauri command)
- **Add type** → `src/types/commands.ts` (mirror from Rust)

---

## Build & Release

```bash
# Test everything
cargo test --lib
npm run build

# Build app
npm run tauri build

# Output: src-tauri/target/release/bundle/macos/Extendead.app
```

---

**Remember: Rust does the work. React shows the results.**
