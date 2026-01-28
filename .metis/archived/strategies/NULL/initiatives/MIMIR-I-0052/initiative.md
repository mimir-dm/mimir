---
id: large-frontend-file-decomposition
level: initiative
title: "Large Frontend File Decomposition"
short_code: "MIMIR-I-0052"
created_at: 2026-01-28T04:54:47.266453+00:00
updated_at: 2026-01-28T20:54:26.919179+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: large-frontend-file-decomposition
---

# Large Frontend File Decomposition Initiative

## Context

Following the completion of MIMIR-I-0049 (Frontend Organizational Debt Cleanup), an analysis of file sizes revealed several extremely large Vue components that warrant decomposition. These files have grown organically and now contain thousands of lines of code, making them difficult to maintain, test, and extend.

**Candidate Files Identified:**
| File | Lines | Location |
|------|-------|----------|
| CharacterSheetView.vue | 3,481 | src/features/characters/views/ |
| DmMapViewer.vue | 2,621 | src/features/maps/components/ |
| MapTokenSetupModal.vue | 2,250 | src/components/tokens/ |
| ModulesTab.vue | 2,210 | src/features/campaigns/components/dashboard/ |

## Goals & Non-Goals

**Goals:**
- Improve maintainability by breaking large files into focused, single-responsibility components
- Extract reusable logic into composables
- Reduce cognitive load when working on specific features
- Enable easier testing of isolated functionality
- Identify and eliminate code duplication

**Non-Goals:**
- Complete architectural overhaul of the frontend
- Changing the visual appearance or behavior of any component
- Adding new features during decomposition
- Refactoring files under 1,000 lines

---

## File Analysis & Decomposition Strategies

### 1. CharacterSheetView.vue (3,481 lines) - HIGH PRIORITY

**Structure Overview:**
- Template: Lines 1-1,235 (massive multi-tab layout)
- Script: Lines 1,237-3,200
- Styles: Lines 3,202-3,481

**Current Responsibilities:**
- 4-tab character sheet (Character, Equipment, Spells, Details)
- 25+ reactive refs for state management
- 15+ computed properties
- 20+ methods for CRUD operations
- Complex spell slot tracking and spell management
- Equipment with attunement tracking
- Ability score management and modifiers

**High-Value Extraction Candidates:**

| Component/Composable | Lines | Complexity | Benefit |
|---------------------|-------|------------|---------|
| CharacterStatsPanel | ~200 | Medium | Core stats display reusable |
| EquipmentSection | ~350 | High | Self-contained equipment logic |
| SpellsSection | ~400 | High | Complex spell management isolated |
| DetailsTab | ~250 | Low | Clean separation |
| useCharacterSheet() | ~300 | Medium | State management extracted |
| useSpellManagement() | ~200 | High | Spell slot/casting logic |

**Duplication Patterns Found:**
- ItemCard rendering pattern appears 2x (equipment, spells)
- Expansion toggle pattern appears 3x
- Data fetching pattern repeated 5x with similar error handling

**Recommendation:** Extract 500-600 lines into 6-8 focused components. Priority: SpellsSection and EquipmentSection first as they're most complex.

---

### 2. DmMapViewer.vue (2,621 lines) - HIGH PRIORITY

**Structure Overview:**
- Template: Lines 1-450 (canvas + UI controls)
- Script: Lines 452-2,350
- Styles: Lines 2,352-2,621

**Current Responsibilities:**
- Canvas-based map rendering with zoom/pan
- Vision and fog of war system
- Token management (drag, place, select)
- Lighting system with dynamic shadows
- Marker/POI management
- Player display synchronization
- Context menus and UI overlays

**High-Value Extraction Candidates:**

| Composable | Lines | Complexity | Benefit |
|------------|-------|------------|---------|
| useFogOfWar() | ~250 | High | Vision/fog logic isolated |
| usePlayerDisplaySync() | ~150 | Medium | WebSocket sync extracted |
| useMapDataLoader() | ~180 | Medium | Data fetching centralized |
| useTokenDragHandler() | ~200 | High | Drag/drop logic reusable |
| useLightingSystem() | ~220 | High | Dynamic shadow calculations |
| useMapContextMenu() | ~120 | Low | Context menu state/actions |

**Duplication Patterns Found:**
- Canvas coordinate transformation logic repeated 4x
- Event handler setup patterns similar across drag operations
- Token/marker selection logic nearly identical

**Recommendation:** Target reduction from 2,621 to ~1,400 lines. Extract composables for fog, lighting, and token handling first as they're the most complex and self-contained.

---

### 3. MapTokenSetupModal.vue (2,250 lines) - MEDIUM PRIORITY

**Structure Overview:**
- Template: Lines 1-680 (modal with 4 entity type tabs)
- Script: Lines 682-2,000
- Styles: Lines 2,002-2,250

**Current Responsibilities:**
- Token placement and configuration
- Light source placement
- Trap placement
- POI (Point of Interest) placement
- Drag-and-drop from catalog to map preview
- Entity editing and deletion

**Critical Finding - Massive Duplication:**
The drag handler system (lines 760-1,093) contains **75% duplicate logic** across 4 entity types:
- `handleTokenDragStart/Move/End`
- `handleLightDragStart/Move/End`
- `handleTrapDragStart/Move/End`
- `handlePoiDragStart/Move/End`

**High-Value Extraction Candidates:**

| Component/Composable | Lines | Complexity | Benefit |
|---------------------|-------|------------|---------|
| useEntityDragDrop() | ~200 | Medium | Replaces 400+ lines of duplication |
| TokenConfigPanel | ~150 | Medium | Token-specific UI |
| LightConfigPanel | ~120 | Low | Light-specific UI |
| TrapConfigPanel | ~100 | Low | Trap-specific UI |
| PoiConfigPanel | ~100 | Low | POI-specific UI |
| EntityPreviewCanvas | ~180 | Medium | Shared canvas preview |

**Recommendation:** Extract 30-40% of code. The generic `useEntityDragDrop()` composable would eliminate the most duplication and could be parameterized by entity type.

---

### 4. ModulesTab.vue (2,210 lines) - MEDIUM PRIORITY

**Structure Overview:**
- Template: Lines 1-850 (complex dashboard layout)
- Script: Lines 852-1,980
- Styles: Lines 1,982-2,210

**"God Component" Indicators:**
- 10+ modal dialogs managed inline
- 35+ functions
- 25+ refs for state management
- Mixes data fetching, UI state, and business logic

**Current Responsibilities:**
- Module list display and filtering
- Module CRUD operations
- Danger/encounter management
- Monster/NPC quick-add
- Loot generation
- Map association
- Bulk operations

**High-Value Extraction Candidates:**

| Component/Composable | Lines | Complexity | Benefit |
|---------------------|-------|------------|---------|
| DangersList | ~180 | Medium | Self-contained dangers UI |
| ModuleCard | ~120 | Low | Reusable module display |
| useModalState() | ~100 | Low | Modal visibility centralized |
| useDangerSelection() | ~150 | Medium | Selection logic extracted |
| useModuleCrud() | ~200 | Medium | CRUD operations isolated |
| QuickAddPanel | ~150 | Medium | Monster/NPC quick add UI |

**Duplication Patterns Found:**
- Modal open/close patterns repeated 10x
- List filtering logic similar across modules/dangers
- Selection state management duplicated

**Recommendation:** Start by extracting `useModalState()` to clean up the 10+ modal patterns, then extract DangersList as a self-contained component.

---

---

## Marginal Files Analysis (900-1,200 lines)

### 5. SettingsView.vue (1,177 lines) - HIGH PRIORITY

**Structure:** Template 299 lines | Script 215 lines | Style 661 lines

**Current Responsibilities:**
- Navigation hub with 4 settings sections
- Theme settings with MCP server integration
- Claude Code/Desktop connection with copy-to-clipboard
- Developer tools (database seeding controls)

**Critical Finding - Mixed Concerns:**
- 4 unrelated domains in one file (theme, MCP, dev tools, campaign/book management)
- 3 nearly identical handlers (`handleSeedData`, `handleReseedData`, `handleClearData`)
- 3 identical copy-to-clipboard buttons

**High-Value Extractions:**
| Composable/Component | Benefit |
|---------------------|---------|
| useDevTools() | Consolidates all dev seeding logic (~90 lines) |
| useMcpIntegration() | Separates MCP server management (~60 lines) |
| MCPStatusCard.vue | Self-contained status display |
| DevToolsCard.vue | Isolated seeding UI |

**Recommendation:** HIGH priority. Size is NOT justified - this is a settings router masquerading as a monolith. Target: reduce to ~400-500 lines.

---

### 6. ModulePlayView.vue (1,174 lines) - MEDIUM PRIORITY

**Structure:** Template 177 lines | Script 253 lines | Style 740 lines

**Current Responsibilities:**
- Play mode layout orchestration
- Player display management (toggle, blackout)
- Monster encounter management
- Map management and display sync
- Play notes with auto-save

**Assessment:**
- Size is more justified as a page/view component
- Clear separation possible: header, sidebar, notes are discrete sections
- Already uses composables for some logic

**High-Value Extractions:**
| Component | Lines | Benefit |
|-----------|-------|---------|
| PlayModeHeader.vue | ~45 | Reusable display controls |
| PlayModeSidebar.vue | ~50 | Self-contained monster/map lists |
| usePlayModeLayout() | ~60 | UI state management isolated |

**Recommendation:** MEDIUM priority. Legitimate page responsibilities, but header and sidebar are clearly extractable.

---

### 7. FeatureChoicesStep.vue (1,107 lines) - HIGH PRIORITY

**Structure:** Template 280 lines | Script 542 lines | Style 283 lines

**Current Responsibilities:**
- SIX distinct D&D class feature selection systems:
  - Fighting Styles, Metamagic, Maneuvers, Invocations, Pact Boon, Expertise

**Critical Finding - Massive Duplication:**
- Same grid/card pattern copy-pasted 6 times
- Selection toggle logic identical across 3 functions
- Data loading functions follow identical patterns
- 13 refs + 16 computed + 24 functions for a single step

**High-Value Extractions:**
| Composable/Component | Benefit |
|---------------------|---------|
| useFeatureSelection() | Generic toggle/select with slot limits |
| FeatureGridSection.vue | Reusable grid card rendering |
| FeatureCard.vue | Selection card component |
| 6 feature-specific sections | Self-contained per feature type |

**Recommendation:** HIGH priority. Adding a 7th feature type would require surgical changes across 3 sections. Target: reduce to ~200 lines.

---

### 8. PlayerDisplayWindow.vue (1,080 lines) - HIGH PRIORITY

**Structure:** Script 580 lines | Template 293 lines | Style 200 lines

**Current Responsibilities:**
- Map rendering with scaling/viewport
- Fog of war visualization
- Grid overlay (square/hex)
- Token rendering with LOS filtering
- Light sources and shadow casting
- IPC event handling (6 listeners)

**Critical Finding - Reusable Logic:**
- Viewport logic (pan/zoom) needed by other map windows
- 6 repetitive event listener setups
- SVG overlays are independent and could be components

**High-Value Extractions:**
| Composable | Lines | Benefit |
|------------|-------|---------|
| usePlayerViewport() | ~80 | Reusable pan/zoom/scale |
| useMapEventSync() | ~150 | Consolidates 6 IPC listeners |
| useFogOfWar() | ~60 | Vision/LOS logic isolated |

**Recommendation:** HIGH priority. Core component: ~400 lines after extraction.

---

### 9. TokenPalette.vue (953 lines) - HIGH PRIORITY

**Structure:** Template 173 lines | Script 441 lines | Style 337 lines

**Current Responsibilities:**
- Token type selection (Monster, NPC, Trap, Marker)
- Light source selection
- Monster search & linking
- Trap search & linking
- Module monsters quick-selection

**Critical Finding - Search Duplication:**
- Monster/trap search logic is 95% identical
- `sizeMap` defined twice identically
- 14 refs + 13 functions for mixed concerns

**High-Value Extractions:**
| Composable | Benefit |
|------------|---------|
| useMonsterSearch() | Reusable monster catalog search |
| useTrapSearch() | Mirror of monster search |
| useTokenConfiguration() | Name/size/color state |
| TokenTypeSelector.vue | Type grid component |

**Recommendation:** HIGH priority. Search duplication is clear candidate. Target: ~400 lines (58% reduction).

---

### 10. MapGridConfigModal.vue (939 lines) - HIGH PRIORITY

**Structure:** Template 256 lines | Script 344 lines | Style 335 lines

**Current Responsibilities:**
- Grid type selection (none/square/hex)
- Interactive map preview with pan/zoom
- Grid alignment editor (position mode → grid mode)
- Grid calculator for rescaled maps
- Manual offset input

**Critical Finding - Multiple Semi-Independent Features:**
- Calculator, preview editor, grid configurator could function independently
- 29+ refs create cognitive load
- Preview logic (zoom/pan/transformation) hard to test

**High-Value Extractions:**
| Composable/Component | Lines | Benefit |
|---------------------|-------|---------|
| useMapPreview() | ~120 | Zoom, pan, viewport |
| useMapGridCalculator() | ~40 | Compression ratio |
| GridOverlay.vue | ~80 | SVG rendering |
| GridCalculator.vue | ~60 | Reusable calculator UI |

**Recommendation:** HIGH priority. Target: ~250 lines main modal.

---

## Updated Priority Ranking

| Rank | File | Lines | Effort | Impact | Rationale |
|------|------|-------|--------|--------|-----------|
| 1 | CharacterSheetView.vue | 3,481 | High | High | Largest file, most complex |
| 2 | DmMapViewer.vue | 2,621 | High | High | Core DM feature, complex canvas logic |
| 3 | FeatureChoicesStep.vue | 1,107 | Medium | High | 6x duplication pattern, easy win |
| 4 | TokenPalette.vue | 953 | Medium | High | 95% duplicate search logic |
| 5 | SettingsView.vue | 1,177 | Medium | Medium | 4 unrelated domains mixed |
| 6 | PlayerDisplayWindow.vue | 1,080 | Medium | Medium | Reusable viewport logic |
| 7 | MapTokenSetupModal.vue | 2,250 | Medium | Medium | 75% duplicate drag handlers |
| 8 | MapGridConfigModal.vue | 939 | Low | Medium | Semi-independent features |
| 9 | ModulesTab.vue | 2,210 | High | Medium | "God component" but functional |
| 10 | ModulePlayView.vue | 1,174 | Low | Low | Justified as page view |

## Implementation Approach

**Phase 1: Extract Composables (Lower Risk)**
- Pull reusable logic into composables without changing component structure
- Verify functionality unchanged after each extraction
- Composables are easier to test in isolation

**Phase 2: Extract Child Components (Higher Risk)**
- Break template sections into child components
- Pass props/events rather than shared refs
- More invasive but provides better encapsulation

**Phase 3: Verify & Clean Up**
- Run full test suite after each file
- Remove dead code
- Update imports throughout codebase

## Alternatives Considered

**Alternative 1: Leave As-Is**
- Rejected: Files will continue to grow, making future maintenance harder

**Alternative 2: Complete Rewrite**
- Rejected: Too risky, could introduce regressions, not necessary

**Alternative 3: Gradual Extraction (Selected)**
- Incremental improvements with verification at each step
- Lower risk, can be paused if issues arise