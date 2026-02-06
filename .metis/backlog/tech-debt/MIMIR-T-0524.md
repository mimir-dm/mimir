---
id: extract-modulestab-panel-components
level: task
title: "Extract ModulesTab panel components"
short_code: "MIMIR-T-0524"
created_at: 2026-02-06T13:33:38.038202+00:00
updated_at: 2026-02-06T13:33:38.038202+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


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

- [ ] Extract `ModuleList.vue` - Module selection sidebar with create/delete
- [ ] Extract `ModuleMonstersPanel.vue` - Monster management for selected module
- [ ] Extract `ModuleTrapsPanel.vue` - Trap management for selected module
- [ ] Extract `ModulePoisPanel.vue` - POI management for selected module
- [ ] Extract `ModuleDocumentsPanel.vue` - Document management for selected module
- [ ] Extract `ModuleMapsPanel.vue` - Map management for selected module
- [ ] Extract `ModuleNpcsPanel.vue` - NPC management for selected module
- [ ] Parent component coordinates selected module + panel visibility
- [ ] All existing functionality preserved
- [ ] `vue-tsc --noEmit` passes

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

*To be added during implementation*