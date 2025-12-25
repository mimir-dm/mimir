---
id: backend-npc-support
level: task
title: "Backend NPC Support"
short_code: "MIMIR-T-0154"
created_at: 2025-12-16T18:00:00.000000+00:00
updated_at: 2025-12-16T18:00:00.000000+00:00
parent: MIMIR-I-0019
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"

status: pending
estimated_complexity: M
strategy_id: NULL
initiative_id: npc-character-creation
---

# Backend NPC Support

## Description

Update backend models and services to support NPC creation with nullable player_id and NPC-specific fields.

## Acceptance Criteria

- [ ] Make player_id Option<i32> in Character/NewCharacter structs
- [ ] Add NPC fields to CharacterData: npc_role, npc_location, npc_faction, npc_notes
- [ ] Update CreateCharacterRequest with is_npc and NPC fields
- [ ] Update Character Service to accept is_npc from request
- [ ] Handle nullable player_id for NPCs in service layer

## Files to Modify

- `crates/mimir-dm-core/src/models/character/mod.rs`
- `crates/mimir-dm-core/src/models/character/data/mod.rs`
- `crates/mimir-dm/src/commands/character/character.rs`
- `crates/mimir-dm-core/src/services/character/mod.rs`
