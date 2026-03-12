---
id: audit-how-to-guides-characters-and
level: task
title: "Audit how-to guides: characters and play mode"
short_code: "MIMIR-T-0580"
created_at: 2026-03-11T23:13:28.523096+00:00
updated_at: 2026-03-11T23:13:28.523096+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit how-to guides: characters and play mode

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review the 9 how-to guide pages covering characters and play mode for accuracy. These guides cover the most interactive parts of Mimir — character management and live session features — where the UI has evolved significantly.

## Scope

### Characters (4 pages)
- `docs/src/how-to/characters/create-pc.md`
- `docs/src/how-to/characters/create-npc.md`
- `docs/src/how-to/characters/assign-to-campaign.md`
- `docs/src/how-to/characters/print-character-sheet.md`

### Play Mode (5 pages)
- `docs/src/how-to/play-mode/start-session.md`
- `docs/src/how-to/play-mode/manage-encounters.md`
- `docs/src/how-to/play-mode/fog-of-war.md`
- `docs/src/how-to/play-mode/use-player-display.md`

## Acceptance Criteria

- [ ] Each page read and cross-referenced against current Vue components and Tauri commands
- [ ] Verify character creation steps match current form fields (race, class, background, ability scores)
- [ ] Check if character docs mention spell management (added in v0.6.0) — likely missing
- [ ] Check if character docs mention inventory management and item assignment — likely missing
- [ ] Verify NPC creation flow matches current implementation
- [ ] Check print character sheet guide against current `mimir-print` PDF output
- [ ] Verify play mode steps: session start, encounter management, initiative tracking
- [ ] Check fog of war docs against current implementation
- [ ] Verify player display setup and WebSocket connection flow
- [ ] Identify where screenshots would help — especially character sheet, play mode encounter panel, fog of war controls
- [ ] Produce findings report

## Screenshot Candidates

- Character creation form (PC and NPC variants)
- Character sheet view with spells tab (new)
- Character inventory panel (new)
- Play mode encounter tracker
- Fog of war controls overlay
- Player display connection setup
- Printed character sheet PDF example

## Status Updates

*To be added during implementation*