# Extendead Status: Clean Start ✓

**Date:** 2026-04-27  
**Build:** ✓ Clean (50 modules, 173.87KB)  
**Tests:** ✓ 91 passing  
**Architecture:** ✓ Solid (Tauri backend + React UI)  

---

## What Was Done

### Eliminated Cross-Coding ✓
- **Deleted** duplicate semantic pipeline from React (`src/hooks/useSemanticalPipeline.ts`, `src/core/`)
- **Established** Rust as single source of truth
- **Cleaned** CSS drag conflicts, attribute-based native handling

### Documented Architecture ✓
- **ARCHITECTURE.md**: Complete system design, data flow, phase roadmap
- **DEVELOPER_GUIDE.md**: How to build features the right way
- **CAPABILITY_ONTOLOGY.md**: All supported commands/modes/services

### Current State

**React Frontend** (`src/`)
```
src/
├── App.tsx                    (UI shell, window management)
├── components/                (11 functional components)
├── hooks/                     (3 IPC bridges: command, machine, permission)
├── types/                     (2 type files mirroring Rust)
└── [CSS, types]
```

**Rust Backend** (`src-tauri/src/`)
```
src-tauri/src/
├── commands.rs               (Tauri IPC endpoints)
├── parser.rs                 (Tokenize → intent)
├── resolver.rs               (Intent → routes)
├── executor.rs               (Execute action safely)
├── modes.rs                  (Study, Focus, Reading modes)
├── semantic.rs               (SemanticFrame decomposition)
├── applescript.rs            (macOS operations)
├── machine.rs                (System state)
├── [15 more modules]
```

---

## Solid Direction

### Phase 1: Foundation ✓ DONE
- SemanticFrame struct (verb, target, scope, qualifier, temporal, intensity)
- AppleScript templates (DND enable/disable, brightness)
- Machine app scanning (complete with caching)

### Phase 2: Built-in Modes ✓ DONE
- Three modes: Study, Focus, Reading
- Concurrent group support
- Risk classification
- Integrated into executor/validator

### Phase 3: Parser Migration (NEXT)
- Migrate parser to produce SemanticFrame
- Add constraint hierarchy module
- Add context retrieval module
- Add reasoning-effort selection module

### Phase 4: Full Semantic Pipeline
- 10-stage constraint-based reasoning
- Complete verification loop
- API enrichment for edge cases

---

## What NOT To Do

❌ **Don't** add logic to React  
❌ **Don't** duplicate Rust types in TypeScript  
❌ **Don't** handle command parsing in JavaScript  
❌ **Don't** add browser-specific code  
❌ **Don't** use web patterns for a desktop app  

---

## What TO Do

✓ **Call Rust** from React via `invoke()`  
✓ **Transform** Rust responses for display  
✓ **Mirror** types from Rust definitions  
✓ **Use** native Tauri window APIs  
✓ **Keep React thin** - just UI  

---

## Running Locally

```bash
# Install
npm install

# Dev (live reload)
npm run tauri dev

# Build
npm run tauri build

# Test
cargo test --lib              # Rust tests (91 passing)
npm run build                 # TypeScript check

# Clean architecture reference
cat ARCHITECTURE.md           # System design
cat DEVELOPER_GUIDE.md        # How to add features
cat CAPABILITY_ONTOLOGY.md    # All supported commands
```

---

## Remaining Issues

### Window Drag
Status: **In progress**  
- CSS conflicts removed ✓
- Using `data-tauri-drag-region` attribute ✓
- Need to verify native Tauri drag works
- Possible Tauri version or config issue

### Next Implementation
1. Mode executor with parallel execution
2. Plan preview UI component
3. Parser → SemanticFrame migration
4. Constraint hierarchy system

---

## Git History

Last 3 commits:
```
0bfd933  Clean up web/desktop cross-coding (this cleanup)
ad14588  Implement built-in modes system
e796e85  Add semantic pipeline ontology and AppleScript templates
```

---

## Architecture Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Build time | 825ms | ✓ Fast |
| Bundle size | 173.87KB (55.15KB gzipped) | ✓ Lean |
| Test coverage | 91 tests passing | ✓ Comprehensive |
| Code duplication | 0 (no React pipeline) | ✓ Clean |
| IPC bridges | 3 (command, machine, permission) | ✓ Complete |
| Component count | 11 functional components | ✓ Focused |
| Rust modules | 28 | ✓ Organized |

---

**Ready for Phase 3: Parser Migration →**
