---
id: extract-charactersheetview-tab
level: task
title: "Extract CharacterSheetView tab components"
short_code: "MIMIR-T-0523"
created_at: 2026-02-06T13:33:37.273142+00:00
updated_at: 2026-02-06T13:33:37.273142+00:00
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

# Extract CharacterSheetView tab components

## Objective

Extract `CharacterSheetView.vue` (2413 lines) tab contents into separate components and D&D computation logic into composables for better maintainability.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: `CharacterSheetView.vue` has 2413 lines with 4 tabs (character, equipment, spells, details), 30+ computed properties, and significant D&D calculation logic mixed with UI code. Hard to navigate and test.
- **Benefits of Fixing**: Smaller, focused components. Reusable D&D computation composables. Easier testing of calculation logic. Better separation of concerns.
- **Risk Assessment**: Medium — Vue component extraction requires careful prop/emit design. D&D calculations must remain accurate after extraction.

## Acceptance Criteria

- [ ] Extract `CharacterStatsTab.vue` - Ability scores, combat stats, saving throws, skills
- [ ] Extract `EquipmentTab.vue` - Inventory management, equipped items, attacks
- [ ] Extract `SpellsTab.vue` - Spellcasting, spell slots, spell list
- [ ] Extract `CharacterDetailsTab.vue` - Personality traits, background, features
- [ ] Create `useCharacterStats` composable - Modifiers, proficiency bonus, passive perception
- [ ] Create `useSpellcasting` composable - Spell slots, spell save DC, spell attack
- [ ] Parent view manages tab state and passes character data to children
- [ ] All existing functionality preserved
- [ ] `vue-tsc --noEmit` passes

## Implementation Notes

### Proposed Structure
```
features/characters/
├── views/
│   └── CharacterSheetView.vue    # Reduced to ~300 lines - tab container + routing
├── components/
│   └── sheet/
│       ├── CharacterStatsTab.vue    # Abilities, combat, saves, skills
│       ├── EquipmentTab.vue         # Inventory, attacks
│       ├── SpellsTab.vue            # Spellcasting
│       └── CharacterDetailsTab.vue  # Personality, features
└── composables/
    ├── useCharacterStats.ts    # getModifier, profBonus, passivePerception, etc.
    └── useSpellcasting.ts      # spellSlots, spellSaveDC, spellAttack, etc.
```

### Current State Analysis
- **Template**: ~470 lines with 4 major tab sections
- **Script**: ~900 lines with loading, computed props, methods
- **Style**: ~1000 lines of scoped CSS

### Risk Considerations
- D&D 5e calculations (spell slots, multiclass, etc.) are complex — unit test the composables
- Inventory and equipment state is shared across tabs — careful prop design needed
- Print functionality depends on character state — ensure it still works after extraction

## Status Updates

*To be added during implementation*