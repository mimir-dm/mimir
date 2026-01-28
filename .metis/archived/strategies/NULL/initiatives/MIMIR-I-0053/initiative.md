---
id: character-creation-wizard
level: initiative
title: "Character Creation Wizard"
short_code: "MIMIR-I-0053"
created_at: 2026-01-28T17:28:05.116153+00:00
updated_at: 2026-01-28T20:54:24.707619+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: character-creation-wizard
---

# Character Creation Wizard Initiative

## Context

The current character creation wizard is a simple single-step form with free-text inputs for race and background, no class selection, no ability score assignment, and no skill/spell selection. It creates characters with minimal data — essentially just a name and player name. The backup codebase (`mimir-dm-bu`) contains a full 7-step wizard with catalog-based selection, ability score methods, skill proficiencies, and spell selection that serves as prior art.

## Goals & Non-Goals

**Goals:**
- Multi-step wizard flow for PC creation with catalog-integrated selection
- Race selection from catalog (with search/filter)
- Class selection from catalog (base classes, filtering subclasses)
- Background selection from catalog
- Ability score assignment (standard array, point buy, manual entry)
- Skill proficiency selection (background-granted + class choices)
- Spell selection for spellcasters (cantrips + known spells)
- Campaign assignment (optional, supports unassigned characters)
- Review step before final creation

**Non-Goals:**
- NPC creation overhaul (current simple form is sufficient for NPCs)
- Equipment/inventory selection during creation
- Multiclassing at creation time (handled by level-up flow)
- Subrace/subclass selection (can be added later)
- Legendary action configuration for NPCs

## Prior Art

The `mimir-dm-bu` backup contains a complete implementation:
- `mimir-dm-bu/mimir-dm/frontend/src/features/characters/components/CharacterCreationWizard.vue` — 7-step wizard
- `mimir-dm-bu/mimir-dm/frontend/src/features/characters/components/SpellSelector.vue` — spell grid selector
- `mimir-dm-bu/mimir-dm/src/commands/character/character.rs` — backend with full `CreateCharacterRequest`

Key differences from current codebase:
- Backend already has `race_name`/`race_source`/`background_name`/`background_source` and `ability_scores` in `CreatePcRequest`
- Catalog search commands exist: `search_races`, `search_classes`, `search_backgrounds`
- Detail commands exist: `get_race_by_name`, `get_class_by_name`, `get_background_by_name`
- Level-up flow already handles class addition post-creation

## Detailed Design

### Wizard Steps (PC Flow)

1. **Basics** — Name, Player Name, Character Type toggle (PC/NPC)
2. **Race** — Searchable catalog selector, shows race traits preview
3. **Class** — Searchable catalog selector, shows class features preview
4. **Background** — Searchable catalog selector, shows background features
5. **Ability Scores** — Three methods: Standard Array, Point Buy, Manual; racial bonuses displayed
6. **Skills** — Background-granted skills (locked) + class skill choices
7. **Spells** — (Spellcasters only) Cantrip and known spell selection
8. **Review** — Summary of all selections, Create button

### Backend Changes

- Extend `CreatePcRequest` to include `class_name`, `class_source`, `skill_proficiencies`, `cantrips`, `known_spells`
- Add corresponding fields to service layer and DAL
- Validate catalog references on creation

### Frontend Components

- Refactor `CharacterCreationWizard.vue` into multi-step wizard with step navigation
- Create `CatalogSelector.vue` — reusable searchable dropdown for race/class/background
- Create `AbilityScoreEditor.vue` — standard array / point buy / manual entry
- Create `SkillSelector.vue` — skill proficiency picker with locked background skills
- Port `SpellSelector.vue` from backup

## Implementation Plan

1. Scaffold multi-step wizard framework (step navigation, validation per step)
2. Build `CatalogSelector` component using existing search commands
3. Implement race/class/background selection steps
4. Implement ability score assignment step with all three methods
5. Implement skill proficiency selection step
6. Port spell selector from backup for spellcaster step
7. Build review step
8. Extend backend `CreatePcRequest` with new fields
9. Wire up end-to-end creation flow
10. Test with various class/race combinations