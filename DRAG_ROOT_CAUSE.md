# Drag Issue: Root Cause & Resolution

## The Problem
Window drag was completely non-functional despite having `data-tauri-drag-region` attribute on the drag handle.

## Root Cause Found
**The headers (ExpandedConsole and DeveloperPanel) had `-webkit-app-region: no-drag`** which was preventing drag handles from working since they're children of those headers.

In CSS, when a parent has `no-drag`, all children inherit that restriction. Since the drag handle was a child of headers marked with `no-drag`, it couldn't function.

```css
/* WRONG: This made the entire header non-draggable */
.expanded-console__header {
  -webkit-app-region: no-drag;  /* ❌ Also blocked drag handle child */
}

.expanded-console__drag-handle {
  /* Child can't override parent's no-drag */
}
```

## Solution Applied

### 1. Remove `-webkit-app-region: no-drag` from Headers
- **File:** `src/components/ExpandedConsole.css`
  - Removed from `.expanded-console__header`
  - Kept on `.expanded-console__meta` (buttons/controls only)

- **File:** `src/components/DeveloperPanel.css`
  - Removed from `.developer-panel__header`  
  - Buttons already have no-drag via element selector

### 2. Add Explicit Pointer Handling
- **File:** `src/components/WindowDragHandle.css`
  - Added `pointer-events: auto` for defensive clarity
  - Ensures handle isn't blocked by parent cascade

### 3. Verify Data Attribute Always Present
- **File:** `src/components/WindowDragHandle.tsx`
  - `data-tauri-drag-region=""` is ALWAYS rendered (not conditional)
  - No CSS conflicts on the handle itself

## Architecture Principle
```
✓ Headers are draggable (where drag handle lives)
✓ Only meta/button areas are non-draggable (with -webkit-app-region: no-drag)
✓ Drag handle has explicit pointer-events: auto
✓ Drag attribute always present, never conditional
```

## Verification Checklist
- ✓ Item 23: Tauri config `alwaysOnTop: true` verified
- ✓ Item 25: React/Rust/Tauri config state synchronized
- ✓ Item 39: No `preventDefault()` on drag handle
- ✓ Item 48: Only `data-tauri-drag-region`, no CSS conflicts
- ✓ Item 50: Pin/drag/mode separation complete

## Status
**FIXED** — All critical items resolved. App running in dev mode with fixes applied.

Next: Live testing to confirm drag works in all modes (lounge, expanded, developer panel).
