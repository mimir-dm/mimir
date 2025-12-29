---
id: npc-creation-via-character-wizard
level: initiative
title: "NPC Creation via Character Wizard"
short_code: "MIMIR-I-0019"
created_at: 2025-12-16T18:00:00+00:00
updated_at: 2025-12-16T18:00:00+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: npc-creation-via-character-wizard
---

# NPC Creation via Character Wizard Initiative

## Context

The current Major NPC Tracker is a document template that doesn't integrate with the character system. NPCs should be first-class characters with `is_npc` flag, enabling:
- Unified character list with PC/NPC filtering
- NPC-specific fields (role, location, faction)
- PDF export using existing NPC card templates
- Better data model for future features

## Goals & Non-Goals

**Goals:**
- Extend character creation wizard to support NPC creation
- Add NPC-specific fields: Role, Location, Faction, Notes
- Make player_id nullable for NPCs (no owner)
- Add NPC filtering/display in character list
- Enable NPC rendering and PDF export
- Deprecate Major NPC Tracker template

**Non-Goals:**
- NPC stat blocks (use monster system for combat-ready NPCs)
- AI-generated NPC content
- NPC relationship mapping

## Implementation Plan

### Phase 1: Backend Changes
- Make player_id nullable for NPCs in Character/NewCharacter structs
- Add NPC fields to CharacterData (npc_role, npc_location, npc_faction, npc_notes)
- Update CreateCharacterRequest with is_npc and NPC fields
- Update Character Service to handle NPC creation

### Phase 2: Frontend Wizard Changes
- Add Character Type selection (PC vs NPC toggle)
- Update TypeScript interfaces
- Implement conditional wizard steps (skip Player Selection for NPCs)
- Add NPC Details step with Role, Location, Faction, Notes

### Phase 3: Character List Updates
- Add NPC filtering tabs (All | PCs | NPCs)
- Show NPC badge/indicator on character cards
- Display NPC role under character name

### Phase 4: NPC Rendering & PDF Export
- Adapt character sheet view for NPCs
- Add Tauri command for NPC card PDF export
- Add NPCs section to campaign PDF export

### Phase 5: Deprecate Major NPC Tracker
- Remove template from seeder
- Update board configuration
- Keep existing documents functional