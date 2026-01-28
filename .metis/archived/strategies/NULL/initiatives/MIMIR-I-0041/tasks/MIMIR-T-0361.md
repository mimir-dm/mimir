---
id: v1-to-v2-migration-strategy
level: task
title: "v1 to v2 Migration Strategy"
short_code: "MIMIR-T-0361"
created_at: 2026-01-19T22:07:00.004569+00:00
updated_at: 2026-01-26T19:41:11.485472+00:00
parent: MIMIR-I-0041
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0041
---

# Migration Strategy (Current to v0.5)

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Define strategy for migrating existing Mimir data to v0.5 schema. This is a separate effort from the rewrite itself - v0.5 can launch without migration, but we need a path for existing users.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] Migration script specification
- [ ] Data mapping from old to new schema
- [ ] Handling of dropped features (versioning, sessions, workflow)
- [ ] Backup/rollback strategy

## Migration Approach

### Strategy: Export/Import Based

Rather than in-place migration, use the new export/import capability:

1. **Current app exports** campaign data to intermediate format
2. **v0.5 imports** from that format
3. User runs both versions side-by-side during transition

This is safer and simpler than SQL migrations across breaking schema changes.

### Why Not SQL Migrations?

- Schema changes are too extensive (not incremental)
- Current data is split between DB and filesystem
- Versioning data needs to be collapsed, not just dropped
- Sessions need to become documents
- This isn't an upgrade, it's a rewrite

## Data Mapping

### Campaigns

| Current | v0.5 | Notes |
|---------|------|-------|
| id | id | Regenerated on import |
| name | name | Direct |
| status | (dropped) | No workflow |
| directory_path | (dropped) | DB-first |
| session_zero_date | (dropped) | No workflow |
| first_session_date | (dropped) | No workflow |

### Modules

| Current | v0.5 | Notes |
|---------|------|-------|
| id | id | Regenerated |
| campaign_id | campaign_id | Remapped |
| name | name | Direct |
| module_number | module_number | Direct |
| status | (dropped) | No workflow |
| expected_sessions | (dropped) | No sessions |
| actual_sessions | (dropped) | No sessions |

### Documents

| Current | v0.5 | Notes |
|---------|------|-------|
| id | id | Regenerated |
| file_path | content | Read file, store content |
| template_id | (dropped) | No templates |
| level | (dropped) | Inferred from module_id |
| session_id | (dropped) | Sessions removed |
| completed_at | (dropped) | No workflow |

### Characters

| Current | v0.5 | Notes |
|---------|------|-------|
| character_versions.character_data | Normalized columns | Parse YAML, extract fields |
| current_version | (dropped) | No versioning |
| All version history | (dropped) | Keep only latest |

The big transformation: Parse the YAML blob from the latest CharacterVersion and extract into normalized columns.

### Sessions → Documents

Session entities become play notes documents:

```
Session {
  id: 1,
  session_number: 5,
  status: 'complete',
  scheduled_date: '2024-01-15'
}
↓
Document {
  title: "Session 5 Notes",
  content: "Played on 2024-01-15\n\n[content from session notes doc]",
  document_type: "play_notes",
  module_id: <parent module>
}
```

## Export Format

The current app should export to a JSON format that v0.5 can import:

```json
{
  "version": "1.0",
  "exported_at": "2024-01-15T10:30:00Z",
  "campaign": {
    "name": "My Campaign",
    "description": "..."
  },
  "modules": [
    {
      "name": "Module 1",
      "module_number": 1,
      "description": "..."
    }
  ],
  "documents": [
    {
      "title": "World Building",
      "content": "# My World\n\n...",
      "document_type": "world_building",
      "module_index": null
    },
    {
      "title": "Session 5 Notes",
      "content": "...",
      "document_type": "play_notes",
      "module_index": 0
    }
  ],
  "characters": [
    {
      "name": "Gandalf",
      "is_npc": true,
      "race": "Human",
      "class": "Wizard",
      "level": 20,
      "abilities": { "strength": 10, "..." },
      "inventory": [...],
      "npc_role": "quest_giver",
      "..."
    }
  ],
  "maps": [
    {
      "name": "Dungeon Level 1",
      "image_base64": "...",
      "width_px": 2000,
      "height_px": 1500,
      "grid_type": "square",
      "tokens": [...]
    }
  ],
  "module_monsters": [...],
  "module_items": [...],
  "module_npcs": [...]
}
```

## Implementation Steps

### Phase 1: Add Export to Current App
1. Add `export_campaign_legacy` Tauri command
2. Reads all campaign data + files
3. Produces JSON blob in format above
4. User downloads/saves the file

### Phase 2: Add Import to v0.5
1. Add `import_campaign` MCP tool and Tauri command
2. Parses JSON format
3. Creates all entities in new schema
4. Copies map images to new filesystem location

### Phase 3: Documentation
1. User guide for migration process
2. Known limitations (lost data)
3. Troubleshooting common issues

## What Gets Lost

Users should understand these are not migrated:

- **Version history**: Only latest character state kept
- **Workflow status**: No concept of campaign/module stages
- **Session structure**: Collapsed into documents
- **Document completion tracking**: No exit criteria
- **Template associations**: Documents are just markdown

## Rollback

Since this is export/import based:

- Original data remains untouched in current app
- User can continue using current app if migration fails
- No destructive operations on source data

## Dependencies

- Depends on: [[MIMIR-T-0357]] Database Schema (defines target schema)
- Depends on: v0.5 being functional enough to import

## Priority

**Low priority for initial v0.5 launch**. New users can start fresh. Migration can be added later for existing users who want to bring data over.

## Progress

*To be updated during implementation*