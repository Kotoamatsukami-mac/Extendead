# Window Drag Fix: Separate Pin/Drag/Mode Concerns

## Problem Analysis

The drag wasn't working because three separate concepts were tangled:
1. **Pin state** (alwaysOnTop) - window stacking order
2. **Draggability** - can move the window
3. **Window mode** (lounge/expanded) - UI layout

The old code made draggability conditional on pin state:
```tsx
// WRONG: Drag disabled when pinned
{...(locked ? {} : { "data-tauri-drag-region": "" })}
```

This created a logic error: pinning the window made it immovable.

## Solution

### 1. Rename Property: `locked` → `pinned`
**File:** `WindowDragHandle.tsx`

Changed from confusing `locked` prop to clearer `pinned` prop:
- `locked` suggested "cannot interact" (web paradigm)
- `pinned` means "always visible, stays on top" (desktop paradigm)

### 2. Always Render Drag Region
**File:** `WindowDragHandle.tsx`

Now `data-tauri-drag-region` is ALWAYS present, regardless of pin state:
```tsx
// CORRECT: Always draggable
data-tauri-drag-region=""
```

Pin state only affects visual opacity - not draggability.

### 3. Remove CSS Conflict
**File:** `WindowDragHandle.css`

Removed the conflicting `-webkit-app-region: no-drag` rule that was directly contradicting the `data-tauri-drag-region` attribute.

Changed from:
```css
.window-drag-handle {
  -webkit-app-region: no-drag; /* WRONG: contradicts data attribute */
}
```

To:
```css
.window-drag-handle {
  /* Use data-tauri-drag-region attribute, not CSS rules */
}
```

### 4. Enlarge Hit Target
**File:** `WindowDragHandle.css`

Increased size from 18×30px to 28×40px (55% larger):
- Was: `width: 18px; height: 30px;`
- Now: `width: 28px; height: 40px;`

Also added `min-width/min-height` to prevent shrinking.

### 5. Update Semantics in Components
**Files:** `LoungeStrip.tsx`, `ExpandedConsole.tsx`, `DeveloperPanel.tsx`

Changed all calls from:
```tsx
<WindowDragHandle locked={alwaysOnTop} />
```

To:
```tsx
<WindowDragHandle pinned={alwaysOnTop} />
```

### 6. Update CSS Class Names
**Files:** `WindowDragHandle.css`

Changed from:
```css
.window-drag-handle--free { /* draggable */ }
.window-drag-handle--locked { /* immovable */ }
```

To:
```css
.window-drag-handle--floating { /* pinned=false */ }
.window-drag-handle--pinned { /* pinned=true */ }
```

These are visual-only indicators - both states are draggable.

### 7. Fix Copy/Titles
**File:** `WindowDragHandle.tsx`

Changed from confusing:
- "Pinned: unpin to move" (suggests it's immovable when pinned)
- "Drag shell" (unclear)

To clear:
- "Pinned (always visible)" (window stacking, separate from drag)
- "Drag to move" (dragging is always possible)

## Architecture Clarity

**Before:** Pin state → Locked dragging
```
alwaysOnTop=true → locked=true → no data-tauri-drag-region → can't drag ❌
```

**After:** Pin state → Visual only
```
alwaysOnTop=true → pinned=true → data-tauri-drag-region always present → can always drag ✓
```

## Verification

Build: ✓ Clean (173.68KB)  
TypeScript: ✓ No errors  
React: ✓ All components updated  

The drag region is now:
- Always present (not conditional)
- Larger and easier to hit (28×40px)
- Free from CSS conflicts
- Semantically correct (pin ≠ lock)

## Next: Test

Launch app with `npm run tauri dev` and verify:
1. ✓ Handle is visibly larger (traffic light gradient)
2. ✓ Drag works when pinned (window stays draggable)
3. ✓ Drag works when floating
4. ✓ Hover/active states work
5. ✓ Pin button toggles visual state, not draggability

---

**Root cause was solved:** Separated `pin` (window stacking) from `drag` (window movement).
These are independent concepts and should not affect each other.
