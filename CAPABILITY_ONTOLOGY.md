# Capability Ontology

Complete reference of supported commands, apps, modes, and integrations that the semantic pipeline understands and can execute.

## Command Categories

### App Control (`app_control`)
Launch, manage, and interact with applications.

**Verbs:** open, close, quit, hide, show, focus, minimize, maximize

**Supported Apps:**
- Safari, Chrome, Firefox, Edge, Arc (browsers)
- Finder, Spotlight (system)
- Terminal, iTerm (shells)
- VS Code, Cursor, Sublime Text, Neovim (editors)
- Slack, Discord, Telegram (messaging)
- Spotify, Apple Music, Tidal (music)
- Preview, Photos, Aperture (media)
- Xcode, Git Tower (dev tools)

**Examples:**
- "open Safari"
- "close Slack"
- "focus VS Code"
- "hide Discord"

**Constraints:**
- Required: `app_installed`, `bundle_id_valid`
- Forbidden: None
- Preferred: `visible_window_exists` (for focus/show)

### Settings (`settings`)
Configure system and application preferences.

**Verbs:** set, enable, disable, toggle, adjust

**Supported Settings:**
- **Volume:** "set volume to 50", "mute volume", "volume up"
- **Brightness:** "set brightness to 75", "brightness down"
- **Do Not Disturb:** "enable DND", "disable focus mode", "toggle DND"
- **Display:** "switch to dark mode", "set resolution to 1920x1080"
- **Keyboard:** "enable key repeat", "set repeat delay to 10"
- **Mouse:** "adjust tracking speed", "enable tap to click"
- **WiFi/Bluetooth:** "turn on WiFi", "connect to [network]"
- **Energy:** "enable low power mode", "wake on LAN"

**Examples:**
- "set volume to 75"
- "enable dark mode"
- "toggle DND"
- "brightness max"

**Constraints:**
- Required: `permissions_granted`, `setting_exists`
- Forbidden: None
- Preferred: `safe_defaults`, `no_system_harm`

### UI Automation (`ui_automation`)
Interact with window management and screen elements.

**Verbs:** show, hide, move, resize, arrange

**Supported Actions:**
- **Window Management:** maximize, minimize, fullscreen, arrange windows
- **Screen Regions:** show menubar, hide dock, show desktop
- **Workspace:** switch space, move to space, arrange displays
- **Focus:** bring to front, send to back

**Examples:**
- "maximize window"
- "show desktop"
- "arrange windows side by side"
- "switch to space 2"

**Constraints:**
- Required: `window_exists`, `permissions_granted`
- Forbidden: None
- Preferred: `preserve_layout`, `smooth_animation`

### Query (`query`)
Search and retrieve information.

**Verbs:** find, search, lookup, get, show

**Supported Sources:**
- **File System:** "find [filename]", "search documents for [term]"
- **Web:** "search [topic]" (via configured provider)
- **System:** "list installed apps", "show running processes"
- **History:** "what was the last command", "show clipboard"

**Examples:**
- "find my notes"
- "search documentation for typescript"
- "list browsers"

**Constraints:**
- Required: `search_available`, `permissions_granted`
- Forbidden: None
- Preferred: `fast_execution`, `relevant_results`

### Filesystem (`filesystem`)
File and folder operations.

**Verbs:** move, copy, delete, create, rename

**Supported Operations:**
- **Move:** "move file to folder", "relocate documents"
- **Copy:** "copy file to destination", "duplicate folder"
- **Delete:** "delete file", "trash folder" (permanent_delete blocked)
- **Create:** "create folder", "new file"
- **Rename:** "rename to [new_name]"

**Examples:**
- "move downloads to archive"
- "copy config to backup"
- "trash old project"
- "create studies folder"

**Constraints:**
- Required: `source_exists`, `permissions_granted`
- Forbidden: `permanent_delete` (use trash instead)
- Preferred: `no_overwrite`, `destination_parent_exists`, `backup_before_delete`
- Conditional[move]: `destination_parent_exists`

### Browser Control (`browser_control`)
Interact with web browsers.

**Verbs:** open, close, go to, reload, find, read

**Supported Actions:**
- **Navigation:** "go to [url]", "open [url] in Safari"
- **Tab Management:** "new tab", "close tab", "reload"
- **Reading:** "read this page", "extract text"
- **Search:** "search google for [term]"

**Examples:**
- "open github.com in Chrome"
- "go to localhost:3000"
- "reload page"
- "search for typescript documentation"

**Constraints:**
- Required: `browser_installed`, `url_valid`
- Forbidden: None
- Preferred: `preferred_browser`

## Modes

**Complex multi-step workflows** combining multiple commands with constraint satisfaction.

### Study Mode
Optimizes environment for focused work: disable notifications, reduce distractions, setup workspace.

**Activates:**
- Enable DND / focus mode
- Close messaging apps (Slack, Discord, Telegram)
- Minimize notifications
- Set brightness to 75%
- Arrange editor and reference windows
- Set timer if specified

**Example:** "study mode" or "study mode for 90 minutes"

**Constraints:** No concurrent mode changes, requires DND permissions

### Focus Mode
Similar to Study but lighter: just disable interruptions.

**Activates:**
- Toggle DND
- Close specific apps or mute notifications

**Example:** "focus mode"

### Reading Mode
Optimize display for reading: dark mode, font size, remove distractions.

**Activates:**
- Enable dark mode
- Increase display brightness
- Close notifications
- Fullscreen preferred window

**Example:** "reading mode"

## Service Integrations

### API Providers
**Fallback resolvers for 10% of inputs** that local command resolution can't handle.

**Supported Providers:**
- Anthropic Claude (default for complex reasoning)
- Perplexity (web search, knowledge)
- OpenAI GPT-4 (alternative reasoning)

**When Used:**
- Query ambiguity > threshold
- Information gathering requires external knowledge
- Complex reasoning needed

**Example:** User asks "what's the weather" → API provider called

### Long-Context Binding
Enrichment that grounds commands in conversation history and machine state.

**Data Bound:**
- Previous commands in current session
- Application state (focused app, open windows)
- User preferences (preferred browser, default settings)
- Deictic references ("this file", "that app", "my documents")

## Constraint Hierarchy

### Validation Levels

**Hard Constraints (must satisfy):**
- `app_installed` - app exists and bundle ID valid
- `source_exists` - file/folder must exist before operating on it
- `permissions_granted` - user permissions in place
- `url_valid` - URL format is correct

**Soft Constraints (preferred):**
- `no_overwrite` - warn before overwriting files
- `safe_defaults` - use safe system values
- `visible_window_exists` - app has visible window
- `backup_before_delete` - keep backup before destructive ops

**Implicit Constraints (inferred):**
- `permissions_granted_or_requestable` - can ask for permission if needed
- `reversible_operation` - can undo if needed

## Risk Classification

**R0 (Safe):** app control (open/close), volume/brightness, window arrangement
- No approval needed
- Verification: confirmation only

**R1 (Low-Risk):** workspace changes, file reads, settings adjustments
- Minor system impact
- Verification: show affected items

**R2 (Medium-Risk):** app permissions changes, install apps, batch file moves
- System-wide impact, user notices
- Verification: confirm action, show affected items
- Requires approval: yes

**R3 (High-Risk):** permanent delete, system preferences, batch operations > 10 items
- Destructive, irreversible, user-visible
- Verification: explain full action, list items, confirm
- Requires approval: yes

## Ambiguity Gradient

**Clear:** "open Safari" → no ambiguity, local resolution
- Verb + target clear
- Intent unambiguous
- Execute immediately

**Slightly Ambiguous:** "open the browser" → target unclear (which one? use default)
- Deterministic fallback available
- Low confidence match
- Ask or use preferred

**Ambiguous:** "show me my project" → target context-dependent
- Needs conversation history
- Deictic reference resolution
- Clarify with user

**Opaque:** "make me productive" → intent unclear
- Requires understanding user context
- May need API enrichment
- Ask clarifying questions or route to provider

## Intent Parameters

Every parsed command produces these fields:

```
category: CommandKind (enum above)
action: string (verb)
target: string | null (object of action)
scope: string | null (where/how wide)
qualifier: string | null (how/style)
temporal: string | null (when/duration)
intensity: number (0-1, "max volume" vs "set volume to 50")
params: Record<string, any>
confidence: 0-1 (match confidence)
```

## Unsupported Commands

**Explicitly Out of Scope:**
- Shell execution (use Terminal app instead)
- Network diagnostics (ping, traceroute, nslookup)
- Package management (npm, brew, pip)
- Source control (git, gh commands)
- Custom scripts / arbitrary code

**Feedback Messages:**
- `unsupported_command`: "Not in local command set"
- `shell_execution`: "Shell command not in local coverage"
- `out_of_scope`: "Requires manual interaction"
- `permanent_delete_blocked`: "Use trash instead"

---

**This ontology is the source of truth for:**
- SemanticFrame parsing shape
- Intent extraction mapping
- Constraint hierarchy definitions
- Risk classification rules
- API provider routing logic
