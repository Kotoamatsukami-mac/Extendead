# macOS Runtime Verification Guide

This document covers what to verify on a real macOS machine. CI validates
compilation, formatting, and unit tests. Runtime behavior requires an Apple
Silicon or Intel Mac running macOS 13 (Ventura) or later.

## Build for testing

```bash
cd ~/Desktop/anbu/Extended
npm run tauri build         # production bundle → src-tauri/target/release/bundle/
# or for faster dev cycle:
npm run tauri dev
```

---

## 1. App launch

- [ ] App starts without crashing
- [ ] Lounge strip appears centered on screen, slim (50 px tall), transparent glass background
- [ ] No dock icon (skipTaskbar active)
- [ ] Strip is always-on-top by default (⊛ pin button shows active state)
- [ ] **Phase 2**: always-on-top state is persisted — relaunching the app restores the previous pin state

---

## 2. Global shortcut — ⌘ ⇧ Space

- [ ] Press `⌘ ⇧ Space` → strip appears / hides (toggle)
- [ ] Works when another app has focus
- [ ] If shortcut is already registered by another app, a warning is logged but the app does not crash
- [ ] Strip regains input focus when shown

---

## 3. Always-on-top toggle

- [ ] Click the ⊛ pin button → strip no longer floats above all windows
- [ ] Click again → strip goes back to always-on-top
- [ ] State survives mode transitions (lounge ↔ expanded)
- [ ] **Phase 2**: preference is written to `~/Library/Application Support/extendead/config.json` and applied on next launch

---

## 4. Open YouTube — browser selection

Type: `open youtube`

- [ ] Strip expands to console mode
- [ ] Intent chip shows `mixed workflow`
- [ ] Risk badge shows `R1`
- [ ] If multiple browsers are installed, route selector lists each (Chrome, Safari, Firefox, Brave, Arc)
- [ ] Selecting a route shows the confirmation rail with Y / N buttons
- [ ] Press `Y` → browser opens youtube.com in the selected browser
- [ ] Press `N` → console collapses, command is denied
- [ ] Press `Esc` → same as N
- [ ] Undo button is **not** shown (OpenUrl has no inverse)
- [ ] **Phase 2**: event timeline shows concrete "Opening https://www.youtube.com in Safari" message

---

## 5. Open Slack

Type: `slack` or `open slack`

- [ ] If Slack is installed (`com.tinyspeck.slackmacgap`), single route resolves
- [ ] Risk `R0`, no confirmation required — executes immediately
- [ ] Slack launches (or comes to front)
- [ ] Event timeline shows `Started → Progress → Completed`
- [ ] **Phase 2**: result message reads "✓ Open Slack", progress says "Launching Slack"

---

## 6. Open a browser directly

Type:
- `open safari`
- `open chrome`
- `open firefox`
- `open brave`
- `open arc`

- [ ] Matching installed browser resolves as `app_control`
- [ ] Risk `R0`, no confirmation required — executes immediately
- [ ] Matching browser launches (or comes to front)
- [ ] If the requested browser is not installed, no route is produced
- [ ] Missing browser shows a precise blocked result, not `Command not recognised`

---

## 7. Mute system volume

Type: `mute`

- [ ] Intent: `local system`, Risk: `R1`, confirmation required
- [ ] Confirm → system audio mutes
- [ ] **Phase 2**: result card shows `✓ Mute Mac`, event timeline shows "Muting system audio output"
- [ ] **Undo** button appears → click it → system unmutes

---

## 8. Set volume

Type: `set volume to 40`

- [ ] Intent: `local system`, Risk: `R1`, confirmation required
- [ ] Confirm → system volume changes to 40 %
- [ ] Result card shows duration
- [ ] **Phase 2**: event timeline shows "Setting output volume"
- [ ] Undo button appears → click → volume returns to the pre-execution level (captured before execution)

---

## 9. Display settings

Type: `display settings`

- [ ] Risk: `R0`, no confirmation, executes immediately
- [ ] System Settings → Displays pane opens
- [ ] No Undo button shown
- [ ] **Phase 2**: event timeline shows "Opening System Settings"

---

## 10. Reveal Downloads

Type: `downloads`

- [ ] Risk: `R0`, no confirmation
- [ ] Finder opens ~/Downloads
- [ ] No Undo button
- [ ] **Phase 2**: event timeline shows "Revealing ~/Downloads in Finder"

---

## 11. Permission prompts

### Accessibility
First time an AppleScript command (mute, volume) runs, macOS may prompt for
Accessibility access.

- [ ] System Settings → Privacy & Security → Accessibility dialog appears
- [ ] Granting permission → command proceeds
- [ ] Denying → command returns an error result card, not a crash

### Apple Events
osascript requires Apple Events permission.

- [ ] First use triggers Apple Events prompt
- [ ] **Phase 2**: If Apple Events is denied, execution returns `blocked` outcome — NOT silent failure
- [ ] **Phase 2**: Result card shows `✗ Permission required — Apple Events permission required. Grant access in System Settings → Privacy & Security → Automation.`
- [ ] **Phase 2**: The amber ⚠ banner in the console shows actionable text: "Grant in System Settings → Privacy & Security → Automation."

### Permission banner
- [ ] Open expanded console (type any command)
- [ ] If Accessibility is `unknown` or `denied`, the amber ⚠ banner appears at the bottom
- [ ] **Phase 2**: Accessibility banner says "required for UI automation. Grant in System Settings → Privacy & Security → Accessibility."
- [ ] **Phase 2**: Apple Events banner says "required for volume & audio commands. Grant in System Settings → Privacy & Security → Automation."
- [ ] If both permissions are granted, no banner is shown

---

## 12. History drawer (Phase 2)

- [ ] Type any command and execute it
- [ ] Click the 🕒 clock button in the expanded console header
- [ ] History drawer opens showing recent commands (newest first, up to 5 entries)
- [ ] Each entry shows: command text, outcome icon (✓/✗/⊘), relative timestamp
- [ ] If the most recent entry has an inverse action, an ↩ undo button appears on it
- [ ] Clicking ↩ triggers undo; result updates
- [ ] Clicking 🕒 again closes the drawer
- [ ] History is persisted at `~/Library/Application Support/extendead/history.json`

---

## 13. Focus and keyboard flow (Phase 2)

- [ ] After collapsing the console (Esc or N), the strip input immediately regains focus
- [ ] Typing a new command works without clicking first
- [ ] `Enter` submits the input
- [ ] `Y` confirms when the approval rail is visible
- [ ] `N` cancels when the approval rail is visible
- [ ] `Escape` collapses expanded console

---

## 14. Provider key storage

Open System Settings flow is manual for v1. Test via the Rust layer:

```bash
# From a terminal (not the app)
security find-generic-password -s com.extendead.app -a openai
# Expect: nothing (key not set)
```

Via the app (requires a UI surface for key management — Phase 3):  
For now, the keychain commands are tested at the command layer only.  
Verify that no key material appears in any log output, IPC payload, or event message.

---

## 15. Keyboard shortcuts in expanded console

- [ ] `Y` → confirms pending approval
- [ ] `N` → denies and collapses
- [ ] `Escape` → collapses console, denies pending approval

---

## 16. History persistence

- [ ] Execute several commands
- [ ] Quit and relaunch the app
- [ ] History is retained across sessions (stored at `~/Library/Application Support/extendead/history.json`)

---

## Phase 2 expected outcomes summary

After Phase 2, the following must work reliably on a real Mac:

| Command | Expected |
|---|---|
| `open youtube` | Route selector → Y → browser opens youtube.com |
| `open slack` | Launches Slack immediately (R0, no confirm) |
| `open safari` | Launches Safari immediately if installed |
| `open chrome` | Launches Google Chrome immediately if installed |
| `mute` | Y → audio mutes, ↩ Undo unmutes |
| `set volume to 30` | Y → volume set to 30, ↩ Undo restores prior level |
| `display settings` | Displays pane opens immediately |
| `downloads` | ~/Downloads opens in Finder |

### Permission-sensitive behaviors

- Volume / mute commands use `set volume` AppleScript — no Apple Events prompt on first use  
  (these commands are built-in osascript, not targeting another app)
- If osascript is blocked at the process level (rare), execution returns `blocked` with a clear next-action message — never silent failure
- Accessibility is reported as `unknown` (Phase 3 will add AXIsProcessTrusted native binding)
- Apple Events probe uses a harmless volume-read that does not trigger a permission dialog

### Known Phase 2 limitations

- Accessibility permission state is always `unknown` — native `AXIsProcessTrusted()` binding is Phase 3
- Remote planner is not available — unrecognised commands show "Command not recognised"
- No per-entry history undo (only the most recent reversible entry can be undone)
- History drawer shows last 5 entries; full history is persisted but not paginated in UI

---

## Known CI limitations

CI runs on Linux (ubuntu-22.04) and verifies:
- TypeScript build
- Rust fmt / clippy / unit tests / cargo check

CI does **not** verify:
- macOS-specific runtime behavior (AppleScript, open URLs, system pref panes)
- Global shortcut registration
- Keychain operations
- Window transparency / always-on-top / skip-taskbar
- Full Tauri `.app` / `.dmg` bundle generation

All items above require a real macOS machine and are covered by this document.
