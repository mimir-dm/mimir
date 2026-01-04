---
id: characters-tab-campaign-characters
level: task
title: "Characters Tab - Campaign Characters"
short_code: "MIMIR-T-0308"
created_at: 2026-01-03T21:05:44.793196+00:00
updated_at: 2026-01-03T21:05:44.793196+00:00
parent: MIMIR-I-0035
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0035
---

# Characters Tab - Campaign Characters

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative

[[MIMIR-I-0035]] - Unified Campaign Dashboard

## Objective

Create the Characters tab for managing campaign-scoped player characters and NPCs.

## Acceptance Criteria

- [ ] `CharactersTab.vue` component created in `components/dashboard/`
- [ ] Displays PCs and NPCs filtered by campaign
- [ ] Create/edit character functionality (inline or modal)
- [ ] Character cards or list view with key info
- [ ] Route `/campaigns/:id/dashboard/characters` renders this tab

## Implementation Notes

### File to Create
```
frontend/src/features/campaigns/components/dashboard/CharactersTab.vue
```

### Layout Structure
```
┌─────────────────────────────────────────┐
│  [+ Add PC]  [+ Add NPC]    [Filters]   │
├─────────────────────────────────────────┤
│  Player Characters                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │ PC Card │ │ PC Card │ │ PC Card │   │
│  └─────────┘ └─────────┘ └─────────┘   │
├─────────────────────────────────────────┤
│  NPCs                                   │
│  ┌─────────┐ ┌─────────┐               │
│  │NPC Card │ │NPC Card │               │
│  └─────────┘ └─────────┘               │
└─────────────────────────────────────────┘
```

### Components to Reuse
- Character components from `features/characters/`
- Character card patterns from existing UI
- Modal patterns for create/edit

### Behavior
- Filter characters by campaign_id
- Separate sections for PCs vs NPCs
- Click card to edit inline or open detail modal

### Complexity
**Medium** - Character management exists, needs campaign-scoped integration

### Dependencies
- Depends on: T-0303 (Foundation) for routing
- Reference: `features/characters/` for existing character components

## Status Updates

*To be added during implementation*