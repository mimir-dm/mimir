---
id: pass-1-3-add-missing-feature
level: task
title: "Pass 1.3: Add missing feature mentions to tutorials"
short_code: "MIMIR-T-0599"
created_at: 2026-03-13T13:50:09.694998+00:00
updated_at: 2026-03-13T14:04:55.993384+00:00
parent: MIMIR-I-0061
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0061
---

# Pass 1.3: Add missing feature mentions to tutorials

## Parent Initiative

[[MIMIR-I-0061]]

## Objective

Add mentions of features that didn't exist when tutorials were written: Homebrew tab, character spell management, character inventory/equipment, Campaign Sources. These are additive changes, not corrections.

## Scope

Features that launched after the tutorials were written and should be mentioned:

### Homebrew Tab (all tutorials that reference the dashboard)
- Tutorial 01 (first-campaign) should mention Homebrew as the 5th tab with a brief description: "Create custom items, monsters, and spells for your campaign"
- Tutorial 02 (first-module) should note that homebrew monsters can be added to module encounter lists alongside catalog monsters

### Character Spell Management
- Tutorial 01 or 02 should note that character sheets have a Spells tab for managing prepared/known spells
- **Verification:** `CharacterSheetView.vue` — check for Spells tab/section, `SpellsSection.vue`

### Character Inventory/Equipment
- Tutorial 01 or 02 should note that character sheets have an Equipment tab for managing items
- **Verification:** `CharacterSheetView.vue` — check for Equipment tab, `InventoryManager.vue`

### Campaign Sources
- Tutorial 01 should mention the Campaign Sources feature (selecting which D&D source books are enabled)
- **Verification:** `CampaignSourcesModal.vue`

### Guidelines
- Keep additions brief — tutorials should introduce features, not deep-dive
- Add as "tip" or "note" callouts where appropriate, not full new sections
- Link to relevant how-to pages for deeper coverage (e.g., "See [Manage Character Spells](../how-to/characters/manage-spells.md) for details")

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All 4 tutorials mention the Homebrew tab where the dashboard is discussed
- [ ] Character spell and equipment tabs mentioned in context of character creation/management
- [ ] Campaign Sources mentioned in tutorial 01
- [ ] All additions verified against source code
- [ ] Additions are concise and don't disrupt tutorial flow

## Status Updates

### 2026-03-13: Completed
Most items in scope were already handled by T-0597 (glossary) and T-0598 (Homebrew tab, Campaign Sources). Remaining additions:

- **Tutorial 01**: Added character sheet tabs mention (stats, equipment, spells, details) under PCs tab description. Verified against `CharacterSheetView.vue` line 262.
- **Tutorial 02**: Added tip about homebrew monsters appearing in search results alongside catalog monsters.
- **Homebrew tab description**: Already added in T-0598 with full content.
- **Campaign Sources**: Already added in T-0598 with modal description.