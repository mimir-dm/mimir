---
id: extract-modulestab-panel-components
level: task
title: "Extract ModulesTab panel components"
short_code: "MIMIR-T-0524"
created_at: 2026-02-06T13:33:38.038202+00:00
updated_at: 2026-02-07T13:00:38.071474+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Extract ModulesTab panel components

## Objective

Extract `ModulesTab.vue` (1943 lines) panel contents into separate components for each sub-domain (monsters, traps, POIs, documents, maps, NPCs).

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: `ModulesTab.vue` manages 6 different sub-domains in one file: modules list, monsters, traps, POIs, documents, maps, and NPCs. Each has its own state, loading, CRUD operations. Finding related code requires scrolling through 1900+ lines.
- **Benefits of Fixing**: Focused components for each domain. Easier to add features to individual panels. Better code organization matching domain boundaries.
- **Risk Assessment**: Low-medium — Each panel is relatively independent. Main complexity is coordinating selected module state across panels.

## Acceptance Criteria

## Acceptance Criteria

- [x] Extract `ModuleList.vue` - Module selection sidebar with create/delete
- [x] Extract `ModuleDocumentsPanel.vue` - Document management for selected module
- [x] Extract `ModuleMapsPanel.vue` - Map management for selected module
- [x] Extract `ModuleNpcsPanel.vue` - NPC management for selected module
- [N/A] Extract `ModuleMonstersPanel.vue` - Already handled by DangersList + MonsterStatsPanel
- [N/A] Extract `ModuleTrapsPanel.vue` - Already handled by DangersList + TrapDetailsPanel
- [N/A] Extract `ModulePoisPanel.vue` - Already handled by DangersList + PoiDetailsPanel
- [x] Parent component coordinates selected module + panel visibility
- [x] All existing functionality preserved
- [x] `vue-tsc --noEmit` passes

## Implementation Notes

### Proposed Structure
```
features/campaigns/components/dashboard/
├── ModulesTab.vue              # Reduced to ~200 lines - layout + coordination
└── modules/
    ├── ModuleList.vue          # Module sidebar with CRUD
    ├── ModuleMonstersPanel.vue # Monster list + detail
    ├── ModuleTrapsPanel.vue    # Trap list + detail  
    ├── ModulePoisPanel.vue     # POI list + detail
    ├── ModuleDocumentsPanel.vue # Document list + editor
    ├── ModuleMapsPanel.vue     # Map list + token setup trigger
    └── ModuleNpcsPanel.vue     # NPC list + detail modal
```

### State Management
- `selectedModule` stays in parent, passed as prop to all panels
- Each panel manages its own loading/error/list state
- Panels emit events for cross-panel updates (e.g., monster added → refresh NPCs)

### Current Refs per Domain
- **Modules**: `modules`, `selectedModule`, `loading`
- **Monsters**: `monsterPanelOpen`, `monsterToEdit`, `savingMonster`
- **Traps**: `moduleTraps`, `loadingTraps`, `selectedTrap`, `trapPanelOpen`
- **POIs**: `modulePois`, `loadingPois`, `selectedPoi`, `poiPanelOpen`
- **Documents**: `moduleDocuments`, `selectedDocument`, `documentToDelete`
- **Maps**: `moduleMaps`, `loadingMaps`, `selectedMapForTokens`
- **NPCs**: `moduleNpcs`, `loadingNpcs`, `selectedNpc`

## Status Updates

### 2026-02-07 - In Progress

**Completed:**
- Created `modules/` subdirectory with extracted components
- Extracted `ModuleList.vue` (180 lines) - module sidebar with CRUD/reorder
- Extracted `ModuleDocumentsPanel.vue` (166 lines) - document list with create/delete
- Extracted `ModuleNpcsPanel.vue` (129 lines) - NPC cards with add/view
- Extracted `ModuleMapsPanel.vue` (134 lines) - map cards with upload/select
- Created `modules/index.ts` with exports and type re-exports
- Updated ModulesTab.vue to use extracted components
- Removed duplicate interfaces (MapData, ModuleNpc) now imported from panels
- Cleaned up unused styles from parent component
- `vue-tsc --noEmit` passes

**Progress:**
- ModulesTab.vue reduced from 1943 to 1480 lines (~24% reduction)
- Total new component lines: 609

**Remaining:**
- The DangersList, MonsterStatsPanel, TrapDetailsPanel, PoiDetailsPanel are already separate components
- The module header (title editing, play/PDF/delete buttons) could be extracted but is tightly coupled
- Modals remain in parent for coordination