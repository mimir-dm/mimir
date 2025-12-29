---
id: npc-rendering-pdf-export
level: task
title: "NPC Rendering and PDF Export"
short_code: "MIMIR-T-0152"
created_at: 2025-12-16T18:00:00.000000+00:00
updated_at: 2025-12-16T18:00:00.000000+00:00
parent: MIMIR-I-0019
blocked_by: [MIMIR-T-0151]
archived: false

tags:
  - "#task"
  - "#phase/pending"

status: pending
estimated_complexity: M
strategy_id: NULL
initiative_id: npc-character-creation
---

# NPC Rendering and PDF Export

## Description

Adapt character sheet view for NPCs and add PDF export functionality.

## Acceptance Criteria

- [ ] Adapt CharacterSheetView for NPCs (show role, location, faction, notes)
- [ ] Hide player-specific sections for NPCs
- [ ] Add Tauri command to export NPC cards as PDF
- [ ] Use existing npc-card.typ and npc-cards-multiup.typ templates
- [ ] Add NPCs section to campaign PDF export

## Files to Modify

- `frontend/src/features/characters/views/CharacterSheetView.vue`
- `crates/mimir-dm/src/commands/print/mod.rs`
