---
id: frontend-wizard-npc
level: task
title: "Frontend Wizard NPC Support"
short_code: "MIMIR-T-0150"
created_at: 2025-12-16T18:00:00.000000+00:00
updated_at: 2025-12-16T18:00:00.000000+00:00
parent: MIMIR-I-0019
blocked_by: [MIMIR-T-0154]
archived: false

tags:
  - "#task"
  - "#phase/pending"

status: pending
estimated_complexity: M
strategy_id: NULL
initiative_id: npc-character-creation
---

# Frontend Wizard NPC Support

## Description

Extend the character creation wizard to support NPC creation with type selection and NPC-specific fields.

## Acceptance Criteria

- [ ] Add Character Type selection at wizard start (PC vs NPC toggle)
- [ ] Update TypeScript interfaces with is_npc and NPC fields
- [ ] Make player_id optional in CreateCharacterRequest
- [ ] Skip Player Selection step for NPCs
- [ ] Add NPC Details step with Role, Location, Faction, Notes fields

## Files to Modify

- `frontend/src/types/character.ts`
- `frontend/src/features/characters/components/CharacterCreationWizard.vue`
