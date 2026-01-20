---
id: deprecate-major-npc-tracker
level: task
title: "Deprecate Major NPC Tracker Template"
short_code: "MIMIR-T-0153"
created_at: 2025-12-16T18:00:00+00:00
updated_at: 2025-12-16T18:00:00+00:00
parent: MIMIR-I-0019
blocked_by: [MIMIR-T-0152]
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: npc-character-creation
---

# Deprecate Major NPC Tracker Template

## Description

Remove the Major NPC Tracker document template now that NPCs are managed through the character system.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Delete major_npc_tracker.md template file
- [ ] Remove MAJOR_NPC_TRACKER from template_seeder.rs
- [ ] Update campaign_board.rs to remove major_npc_tracker from requirements
- [ ] Existing Major NPC Tracker documents remain functional (just no new ones)

## Files to Modify

- `docs/src/campaign-framework/06-templates/templates/major_npc_tracker.md` (delete)
- `crates/mimir-dm-core/src/seed/template_seeder.rs`
- `crates/mimir-dm-core/src/domain/boards/campaign_board.rs`