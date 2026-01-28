---
id: import-service-adapt-with-v0-5-id
level: task
title: "Import Service - Adapt with v0.5 ID Remapping"
short_code: "MIMIR-T-0457"
created_at: 2026-01-28T04:02:47.980918+00:00
updated_at: 2026-01-28T13:46:03.062660+00:00
parent: MIMIR-I-0051
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0051
---

# Import Service - Adapt with v0.5 ID Remapping

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0051]]

## Objective

Implement campaign import with proper UUID remapping for v0.5 models. Handles archive extraction, data parsing, and database insertion with reference resolution.

## Reference

`mimir-dm-bu/mimir-dm-core/src/services/campaign_archive_service.rs` - `import_campaign()` function

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Implement `preview_archive()` to read manifest without importing
- [ ] Implement `import_campaign()` that extracts and imports archive
- [ ] Generate new UUIDs for all imported entities
- [ ] Build ID mapping tables (old UUID → new UUID) for each entity type
- [ ] Remap all foreign key references (module→campaign, character→campaign, token→map, etc.)
- [ ] Resolve character→module assignments via NPC module assignments
- [ ] Copy map images and token images to appropriate asset directories
- [ ] Handle campaign name conflicts (append suffix or allow user rename)
- [ ] Catalog references (monster/item by name+source) remain as-is

## ID Mapping Required

| Entity | References to Remap |
|--------|---------------------|
| Campaign | - |
| Module | campaign_id |
| Character | campaign_id |
| Document | campaign_id, module_id |
| Map | campaign_id, module_id |
| Token | map_id, character_id |
| ModuleMonster | module_id |
| ModuleItem | module_id |
| ModuleNpc | module_id, character_id |

## Status Updates

*To be added during implementation*