---
id: audit-how-to-guides-campaigns-maps
level: task
title: "Audit how-to guides: campaigns, maps, and modules"
short_code: "MIMIR-T-0579"
created_at: 2026-03-11T23:13:28.421185+00:00
updated_at: 2026-03-11T23:13:28.421185+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit how-to guides: campaigns, maps, and modules

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review the 11 how-to guide pages covering campaigns, maps, and modules for accuracy against the current codebase. These are task-oriented guides that users follow to accomplish specific goals — incorrect steps will directly frustrate users.

## Scope

### Campaigns (3 pages)
- `docs/src/how-to/campaigns/create-campaign.md`
- `docs/src/how-to/campaigns/manage-documents.md`
- `docs/src/how-to/campaigns/export-campaign.md`

### Maps (6 pages)
- `docs/src/how-to/maps/upload-map.md`
- `docs/src/how-to/maps/configure-grid.md`
- `docs/src/how-to/maps/place-tokens.md`
- `docs/src/how-to/maps/manage-light-sources.md`
- `docs/src/how-to/maps/print-map.md`

### Modules (3 pages)
- `docs/src/how-to/modules/create-module.md`
- `docs/src/how-to/modules/add-monsters.md`
- `docs/src/how-to/modules/module-documents.md`

## Acceptance Criteria

- [ ] Each how-to page read and cross-referenced against current Vue components, Tauri commands, and service layer
- [ ] Verify all steps match current UI (button labels, form fields, navigation paths)
- [ ] Check export/import campaign flow against current `CampaignService` capabilities
- [ ] Verify map upload flow — does it still match UVTT processing pipeline?
- [ ] Check token placement steps against current `TokenService` and token setup modal
- [ ] Verify print/PDF flow against current `mimir-print` crate capabilities
- [ ] Check whether `add-monsters.md` covers homebrew monsters (added in v0.6.1)
- [ ] Identify where screenshots would help — especially for map grid config, token placement, and light source management
- [ ] Produce findings report listing inaccuracies, missing content, and screenshot requests

## Screenshot Candidates

Pay special attention to these — they are likely areas where a screenshot would significantly help:
- Campaign creation dialog
- Map upload flow (UVTT file selection)
- Grid configuration overlay
- Token setup modal (recently updated)
- Light source placement UI
- Print preview / PDF export dialog
- Module monster list (with homebrew monsters)

## Status Updates

*To be added during implementation*