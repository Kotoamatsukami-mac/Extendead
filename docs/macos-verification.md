# macOS Runtime Verification Guide

This document covers what to verify on a real macOS machine.

CI validates compilation, formatting, and unit tests. A macOS Intel smoke job also
verifies that `tauri build --no-bundle` compiles on GitHub’s macOS Intel runner.
Runtime behavior still requires testing on a real Mac.

## Build for testing

From the repo root:

```bash
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
- [ ] Always-on-top state is persisted — relaunching the app restores the previous pin state

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
- [ ] Preference is written to `~/Library/Application Support/extendead/config.json` and applied on next launch

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
- [ ] Event timeline shows concrete "Opening https://www.youtube.com in Safari" message

---

## 5. Open Slack

Type: `slack` or `open slack`

- [ ] If Slack is installed (`com.tinyspeck.slackmacgap`), single route resolves
- [ ] Risk `R0`, no confirmation required — executes immediately
- [ ] Slack launches (or comes to front)
- [ ] Event timeline shows `Started → Progress → Completed`
- [ ] Result message reads "✓ Open Slack", progress says "Launching Slack"

---

## 6. Mute system volume

Type: `mute`

- [ ] Intent: `local system`, Risk: `R1`, confirmation required
- [ ] Confirm → system audio mutes
- [ ] Result card shows `✓ Mute Mac`, event timeline shows "Muting system audio output"
- [ ] **Undo** button appears → click it → system unmutes

---

## 7. Set volume

Type: `set volume to 40`

- [ ] Intent: `local system`, Risk: `R1`, confirmation required
- [ ] Confirm → system volume changes to 40 %
- [ ] Result card shows duration
- [ ] Event timeline shows "Setting output volume"
- [ ] Undo button appears → click → volume returns to the pre-execution level (captured before execution)

---

## 8. Display settings

Type: `display settings`

- [ ] Risk: `R0`, no confirmation, executes immediately
- [ ] System Settings → Displays pane opens
- [ ] No Undo button shown
- [ ] Event timeline shows "Opening System Settings"

---

## 9. Reveal Downloads

Type: `downloads`

- [ ] Risk: `R0`, no confirmation
- [ ] Finder opens ~/Downloads
- [ ] No Undo button
- [ ] Event timeline shows "Revealing ~/Downloads in Finder"

---

## 10. Permission prompts and status

### Accessibility
Extendead reports Accessibility permission via the native `AXIsProcessTrusted()` API.
This is required for **UI automation** features (click/type) in later phases.

- [ ] If denied, the amber ⚠ banner shows: "Accessibility: denied — required for UI automation."
- [ ] Grant in System Settings → Privacy & Security → Accessibility, then relaunch and re-check

### Apple Events / Automation
Current Phase 1 volume/mute AppleScript does **not** target other apps, so it typically does
not trigger an Automation prompt.

- [ ] If Apple Events is denied/blocked at the process level, execution returns `blocked` outcome — not silent failure
- [ ] Result card shows `✗ Permission required — ...` with a concrete next action
- [ ] The amber ⚠ banner shows: "Apple Events: denied/unknown — required for volume & audio commands."

---

## 11. History drawer

- [ ] Type any command and execute it
- [ ] Click the 🕒 clock button in the expanded console header
- [ ] History drawer opens showing recent commands (newest first, up to 5 entries)
- [ ] Each entry shows: command text, outcome icon (✓/✗/⊘), relative timestamp
- [ ] If the most recent entry has an inverse action, an ↩ undo button appears on it
- [ ] Clicking ↩ triggers undo; result updates
- [ ] Clicking 🕒 again closes the drawer
- [ ] History is persisted at `~/Library/Application Support/extendead/history.json`

---

## 12. Focus and keyboard flow

- [ ] After collapsing the console (Esc or N), the strip input immediately regains focus
- [ ] Typing a new command works without clicking first
- [ ] `Enter` submits the input
- [ ] `Y` confirms when the approval rail is visible
- [ ] `N` cancels when the approval rail is visible
- [ ] `Escape` collapses expanded console

---

## 13. Provider key storage

Test via the Rust layer (terminal, not the app UI):

```bash
security find-generic-password -s com.extendead.app -a perplexity
# Expect: nothing (key not set)
```

Verify that no key material appears in any log output, IPC payload, or event message.

---

## 14. Keyboard shortcuts in expanded console

- [ ] `Y` → confirms pending approval
- [ ] `N` → denies and collapses
- [ ] `Escape` → collapses console, denies pending approval

---

## 15. History persistence

- [ ] Execute several commands
- [ ] Quit and relaunch the app
- [ ] History is retained across sessions (stored at `~/Library/Application Support/extendead/history.json`)

---

## Current expected outcomes summary

| Command | Expected |
|---|---|
| `open youtube` | Route selector → Y → browser opens youtube.com |
| `open slack` | Launches Slack immediately (R0, no confirm) |
| `mute` | Y → audio mutes, ↩ Undo unmutes |
| `set volume to 30` | Y → volume set to 30, ↩ Undo restores prior level |
| `display settings` | Displays pane opens immediately |
| `downloads` | ~/Downloads opens in Finder |

---

## Known CI limitations

CI verifies:
- TypeScript build
- Rust fmt / clippy / unit tests / cargo check (Linux)
- Tauri compile-path via `tauri build --no-bundle` (macOS Intel)

CI does **not** verify:
- macOS-specific runtime behavior (permissions dialogs, AppleScript behavior on your machine)
- Global shortcut registration
- Keychain operations
- Window transparency / always-on-top / skip-taskbar behavior
- Signed `.app` / `.dmg` bundle generation

All items above require a real macOS machine and are covered by this document.
