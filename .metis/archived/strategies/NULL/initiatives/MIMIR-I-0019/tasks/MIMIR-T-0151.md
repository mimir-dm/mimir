---
id: character-list-npc-filtering
level: task
title: "Character List NPC Filtering"
short_code: "MIMIR-T-0151"
created_at: 2025-12-16T18:00:00.000000+00:00
updated_at: 2025-12-16T18:00:00.000000+00:00
parent: MIMIR-I-0019
blocked_by: [MIMIR-T-0150]
archived: false

tags:
  - "#task"
  - "#phase/pending"

status: pending
estimated_complexity: S
strategy_id: NULL
initiative_id: npc-character-creation
---

# Character List NPC Filtering

## Description

Add NPC filtering and visual distinction in the character list view.

## Acceptance Criteria

- [ ] Add filter tabs: All | Player Characters | NPCs
- [ ] Show NPC badge/indicator on character cards
- [ ] Display NPC role under character name
- [ ] Different visual treatment for NPCs (badge, icon, or border)

## Files to Modify

- `frontend/src/features/characters/views/CharacterListView.vue`
