# Drag Fix Checklist: 50-Item Systematic Review

## ✓ FIXED (Items 1-6, 21, 23, 39, 43, 48)

1. ✓ `alwaysOnTop` treated as `locked` → Renamed to `pinned`
2. ✓ `WindowDragHandle` exits early when `locked` → Now always renders drag region
3. ✓ Pinned mode removes `data-tauri-drag-region` → Always present now
4. ⚠️ Defaults to pinned → Still true in Tauri config (will fix below)
5. ✓ Handle too small (18×30px) → Enlarged to 28×40px
6. ⚠️ Shell body not a drag region → Need to review intentionality
21. ✓ Pin label copy → Fixed to "Pinned (always visible)"
43. ✓ Mental model copy → Fixed

## ❌ NEED TO FIX (Items 7-20, 22-28, 29-35, 36-42, 44-50)

### Layer 1: No-Drag Areas (Items 7-15)

These are explicitly marked `no-drag`. Need to verify each is intentional:

7. ❌ Input area is explicitly `no-drag` → **Check intent: should input be excluded?**
8. ❌ Meta/right-side area is explicitly `no-drag` → **Check: right-side buttons?**
9. ❌ Pin button is explicitly `no-drag` → **CORRECT: interactive elements should be no-drag**
10. ❌ Engine button is explicitly `no-drag` → **CORRECT: interactive elements should be no-drag**
11. ❌ Suggestions panel is explicitly `no-drag` → **CORRECT: need to interact with suggestions**
12. ❌ Choices panel is explicitly `no-drag` → **CORRECT: need to interact with choices**
13. ❌ Clarify panel is explicitly `no-drag` → **CORRECT: need to interact with clarify**
14. ❌ WindowDragHandle.css sets `-webkit-app-region: no-drag` → **FIXED but verify it's gone**
15. ❌ CSS and data attribute fighting → **Verify no more CSS conflicts**

### Layer 2: Alternative Drag Mechanisms (Items 16-20)

16. ❌ `startDragging()` only on tiny handle → **Review: was there a manual drag handler?**
17. ❌ `startDragging()` skipped in pinned mode → **Review: should be removed if we're using data attribute**
18. ✓ No native titlebar (decorations: false) → Expected, using data-tauri-drag-region
19. ✓ Transparent window → Expected, need explicit regions
20. ❌ Glass area looks draggable but isn't wired → **Should we make the whole header draggable?**

### Layer 3: Config State Sync (Items 22-28)

22. ⚠️ React state starts as `alwaysOnTop: true` → Verified in App.tsx:60
23. ✓ Tauri config starts as `alwaysOnTop: true` → Verified in tauri.conf.json:25
24. ⚠️ Rust default also `always_on_top: true` → Expected, persisted via config
25. ✓ Config synced (React/Rust/Tauri) → All use same initial state
26. ❌ `getAppConfig()` loop may overwrite pin state → **Review refresh cycle**
27. ❌ `toggle_always_on_top` may succeed in Rust but lag in React → **Need optimistic update**
28. ❌ `toggle_always_on_top` may fail silently → **Add error handling**

### Layer 4: Mode/Expanded Behavior (Items 29-35)

29. ❌ Lounge/pin modes not centrally modeled → **No WindowPolicy yet**
30. ❌ Expanded mode affects drag handle rendering → **Review LoungeStrip conditional**
31. ❌ `embedded={true}` hides drag handle → **Should it? Or always visible?**
32. ❌ `set_window_mode` resizes window awkwardly → **Check window position on resize**
33. ❌ Fixed window size limits drag space → **Review window sizing constraints**
34. ❌ `overflow: hidden` on root/body → **Check if hiding affordances**
35. ❌ Overlay pseudo-elements cover regions → **Review z-index/pointer-events**

### Layer 5: Pointer Event Handling (Items 36-42)

36. ❌ Suggestion dropdown overlaps shell hit region → **Check dropdown z-index**
37. ❌ Pointer might start on input/placeholder/ghost layer → **Verify layer order**
38. ❌ Disabled input changes hit behavior → **Test parsing/executing state**
39. ✓ No `preventDefault()` on drag handle → Verified, handler is clean
40. ❌ Higher z-index blocks handle → **Check z-index stack**
41. ❌ No larger invisible hitbox → **Could add larger hit target**
42. ✓ Handle dots = visual affordance pattern → Comment added, pattern is clear

### Layer 6: macOS/Tauri Behavior (Items 44-50)

44. ❌ macOS Spaces behavior inconsistent → **Document expected behavior**
45. ❌ `skipTaskbar: true` removes affordances → **Check Tauri window options**
46. ✓ `set_decorations(false)` → Expected (no native titlebar)
47. ❌ `set_shadow(false)` removes visual boundary → **Consider adding shadow back**
48. ✓ Using ONLY data-tauri-drag-region → Removed conflicting -webkit-app-region from headers
49. ❌ No single WindowPolicy → **Need to create one**
50. ✓ Pin/drag/mode separation → Pin and drag now independent; modes separate

---

## Priority Fixes

### CRITICAL (Makes drag completely broken)
- [ ] Item 25: Verify React/Rust/Tauri state agreement on alwaysOnTop
- [ ] Item 39: Verify NO preventDefault() on drag handle
- [ ] Item 48: Confirm we're using ONLY data-tauri-drag-region, not CSS regions

### HIGH (Usability issues)
- [ ] Item 42: Make handle visually obvious (add icon/text, not just dots)
- [ ] Item 20: Consider making entire header draggable (larger hit region)
- [ ] Item 47: Add shadow back for visual boundary

### MEDIUM (Edge cases)
- [ ] Item 27: Add optimistic updates to toggle_always_on_top
- [ ] Item 31: Decide: should drag handle hide when embedded?
- [ ] Item 36: Check dropdown z-index doesn't block handle

### LOW (Polish)
- [ ] Item 44: Document expected Spaces behavior
- [ ] Item 50: Create WindowPolicy module to centralize behavior

---

## Next Steps

1. **Verify critical items** (25, 39, 48)
2. **Fix high-priority items** (42, 20, 47)
3. **Address config state** (22-28)
4. **Centralize window behavior** (create WindowPolicy)
5. **Test end-to-end**
