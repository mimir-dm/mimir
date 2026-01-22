---
id: implement-sidebar-navigation-shell
level: task
title: "Implement sidebar navigation shell"
short_code: "MIMIR-T-0406"
created_at: 2026-01-21T16:34:58.247360+00:00
updated_at: 2026-01-21T16:34:58.247360+00:00
parent: MIMIR-I-0045
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0045
---

# Implement sidebar navigation shell

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0045]]

## Objective

Replace header-based navigation with sidebar navigation per v0.5 design (T-0363). This is a structural change to the app shell.

## Acceptance Criteria

- [ ] Create Sidebar.vue component with navigation sections
- [ ] Add CampaignSwitcher dropdown to sidebar
- [ ] Navigation items: Campaigns, Modules, Characters, Documents, Maps, Catalog
- [ ] Sidebar collapsible (icon-only mode)
- [ ] Update App.vue to use sidebar layout
- [ ] Add breadcrumb component to main area
- [ ] Settings and theme toggle in sidebar footer

## Implementation Notes

### Target Layout (from T-0363)

```
┌─────────┬───────────────────────────────────────────────────────┐
│  Side   │  Breadcrumb: Campaign > Module > Document             │
│  bar    ├───────────────────────────────────────────────────────┤
│         │                                                       │
│  [C]    │                    Main Content                       │
│  [M]    │                    (Router View)                      │
│  [Ch]   │                                                       │
│  [D]    │                                                       │
│  [Ma]   │                                                       │
│  [Ca]   │                                                       │
│  ───    │                                                       │
│  [⚙]    │                                                       │
└─────────┴───────────────────────────────────────────────────────┘
```

### Components to Create
- `Sidebar.vue` - Main sidebar container
- `NavSection.vue` - Grouped nav items
- `NavItem.vue` - Individual nav link
- `CampaignSwitcher.vue` - Dropdown for active campaign
- `Breadcrumb.vue` - Path breadcrumb

### Dependencies
- Blocked by: [[MIMIR-T-0399]] (frontend migration)

## Status Updates

*To be added during implementation*