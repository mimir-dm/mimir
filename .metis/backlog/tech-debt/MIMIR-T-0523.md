---
id: extract-charactersheetview-tab
level: task
title: "Extract CharacterSheetView tab components"
short_code: "MIMIR-T-0523"
created_at: 2026-02-06T13:33:37.273142+00:00
updated_at: 2026-02-07T01:18:21.685478+00:00
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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Extract `CharacterStatsTab.vue` - Ability scores, combat stats, saving throws, skills
- [x] Extract `EquipmentTab.vue` - (Already exists as EquipmentSection.vue)
- [x] Extract `SpellsTab.vue` - (Already exists as SpellsSection.vue)
- [x] Extract `CharacterDetailsTab.vue` - Personality traits, background, features
- [x] Create `useCharacterStats` composable - (Existing characterUtils.ts provides all needed utilities)
- [x] Create `useSpellcasting` composable - (Already exists as useSpellManagement.ts)
- [x] Parent view manages tab state and passes character data to children
- [x] All existing functionality preserved
- [x] `vue-tsc --noEmit` passes

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

### 2026-02-06 - Analysis

**Current State:**
- `CharacterSheetView.vue` is 2413 lines
- `SpellsSection.vue` (Spells tab) already extracted ✓
- `EquipmentSection.vue` (Equipment tab) already extracted ✓
- `useSpellManagement.ts` composable already exists ✓

**Remaining Work:**
1. Extract `CharacterStatsTab.vue` - Character tab content (lines 64-257): abilities, combat, saves, skills, proficiencies, class features, spellcasting summary, personality
2. Extract `CharacterDetailsTab.vue` - Details tab content (lines 275-425): background info, NPC info, class details
3. Create `useCharacterStats.ts` composable - shared D&D computation logic (modifiers, proficiency bonus, AC, attacks, etc.)

### 2026-02-06 - Implementation Complete

**Completed:**

1. **Created `CharacterStatsTab.vue`** (~510 lines)
   - Abilities grid with modifiers
   - Combat stats (AC, initiative, speed, passive perception, hit dice, proficiency)
   - Saving throws with proficiency indicators
   - Attacks from equipped weapons
   - Skills list with proficiency/expertise
   - Proficiencies (armor, weapon, tool, language)
   - Class features with expandable descriptions
   - Spellcasting summary
   - Personality traits

2. **Created `CharacterDetailsTab.vue`** (~350 lines)
   - Background section with proficiencies, equipment, features
   - NPC details (role, location, faction)
   - Class details with hit dice, primary ability, spellcasting, saving throws
   - Starting proficiencies for each class
   - Subclass info with description
   - Class features organized by level

3. **Updated `CharacterSheetView.vue`**
   - Reduced from 2413 lines to 1607 lines (~33% reduction)
   - Now uses CharacterStatsTab and CharacterDetailsTab as child components
   - Removed ~400 lines of helper functions moved to child components
   - Removed unused imports
   - Template is now a clean tab container

4. **TypeScript passes: `vue-tsc --noEmit` ✓**

**Decision on useCharacterStats composable:**
- Evaluated `characterUtils.ts` (505 lines) which already contains:
  - `getModifier`, `formatModifier`, `getProficiencyBonus`, `getTotalLevel`
  - Skill/save bonus calculations, AC calculations, weapon damage
  - Spellcasting stats, hit dice calculations
- Decided NOT to create a new composable since `characterUtils.ts` already provides these utilities
- Child components import directly from `@/utils/characterUtils`

**Files Created:**
- `features/characters/components/sheet/CharacterStatsTab.vue`
- `features/characters/components/sheet/CharacterDetailsTab.vue`

**Files Modified:**
- `features/characters/views/CharacterSheetView.vue` - major refactor