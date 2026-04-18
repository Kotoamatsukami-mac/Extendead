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

---

## 5. Open Slack

Type: `slack` or `open slack`

- [ ] If Slack is installed (`com.tinyspeck.slackmacgap`), single route resolves
- [ ] Risk `R0`, no confirmation required — executes immediately
- [ ] Slack launches (or comes to front)
- [ ] Event timeline shows `Started → Progress → Completed`

---

## 6. Mute system volume

Type: `mute`

- [ ] Intent: `local system`, Risk: `R1`, confirmation required
- [ ] Confirm → system audio mutes
- [ ] Result card shows `✓ Mute Mac`
- [ ] **Undo** button appears → click it → system unmutes

---

## 7. Set volume

Type: `set volume to 40`

- [ ] Intent: `local system`, Risk: `R1`, confirmation required
- [ ] Confirm → system volume changes to 40 %
- [ ] Result card shows duration
- [ ] Undo button appears → click → volume returns to the pre-execution level

---

## 8. Display settings

Type: `display settings`

- [ ] Risk: `R0`, no confirmation, executes immediately
- [ ] System Settings → Displays pane opens
- [ ] No Undo button shown

---

## 9. Reveal Downloads

Type: `downloads`

- [ ] Risk: `R0`, no confirmation
- [ ] Finder opens ~/Downloads
- [ ] No Undo button

---

## 10. Permission prompts

### Accessibility
First time an AppleScript command (mute, volume) runs, macOS may prompt for
Accessibility access.

- [ ] System Settings → Privacy & Security → Accessibility dialog appears
- [ ] Granting permission → command proceeds
- [ ] Denying → command returns an error result card, not a crash

### Apple Events
osascript requires Apple Events permission.

- [ ] First use triggers Apple Events prompt
- [ ] Deny → osascript fails, error result shown

### Permission banner
- [ ] Open expanded console (type any command)
- [ ] If Accessibility is `unknown` or `denied`, the amber ⚠ banner appears at the bottom
- [ ] Banner message references System Settings → Privacy & Security → Accessibility
- [ ] If both permissions are granted, no banner is shown

---

## 11. Provider key storage

Open System Settings flow is manual for v1. Test via the Rust layer:

```bash
# From a terminal (not the app)
security find-generic-password -s com.extendead.app -a openai
# Expect: nothing (key not set)
```

Via the app (requires a UI surface for key management — Phase 2):  
For now, the keychain commands are tested at the command layer only.  
Verify that no key material appears in any log output, IPC payload, or event message.

---

## 12. Keyboard shortcuts in expanded console

- [ ] `Y` → confirms pending approval
- [ ] `N` → denies and collapses
- [ ] `Escape` → collapses console, denies pending approval

---

## 13. History persistence

- [ ] Execute several commands
- [ ] Quit and relaunch the app
- [ ] History is retained across sessions (stored at `~/Library/Application Support/extendead/history.json`)

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
