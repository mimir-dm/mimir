---
id: v0-5-mcp-tool-specification
level: task
title: "v0.5 MCP Tool Specification"
short_code: "MIMIR-T-0359"
created_at: 2026-01-19T22:06:59.680898+00:00
updated_at: 2026-01-28T03:58:09.765776+00:00
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

# v0.5 MCP Tool Specification

## Parent Initiative
[[MIMIR-I-0041]] - Mimir v0.5 Architecture Rewrite

## Objective
Define the complete MCP tool interface for v0.5. Preserve compatibility where sensible, simplify where possible, add new export/import tools.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] All tool signatures documented with parameters and return types
- [ ] Simplified tools (no workflow/versioning parameters)
- [ ] New export/import tools specified
- [ ] Claude Code plugin instructions updated

## Tool Changes Summary

| Category | Removed | Simplified | Added |
|----------|---------|------------|-------|
| Campaign | - | get_campaign_details (no stages) | export_campaign, import_campaign |
| Module | - | get_module_details (no stages) | - |
| Character | - | get_character (no versions), edit_character (no snapshot_reason) | - |
| Document | create_document_from_template, list_templates | list_documents (no level/session params) | export_document_markdown |
| Catalog | - | - | search_spells, search_classes, search_races, search_feats, search_backgrounds |

## Tool Specifications

### Campaign Tools

#### list_campaigns
```json
{
  "name": "list_campaigns",
  "description": "List all campaigns",
  "parameters": {
    "include_archived": { "type": "boolean", "default": false }
  },
  "returns": [{
    "id": "integer",
    "name": "string",
    "description": "string | null",
    "created_at": "string",
    "archived_at": "string | null"
  }]
}
```

#### set_active_campaign
```json
{
  "name": "set_active_campaign",
  "description": "Set the active campaign for subsequent operations",
  "parameters": {
    "campaign_id": { "type": "integer", "required": true }
  },
  "returns": {
    "success": "boolean",
    "campaign_id": "integer",
    "campaign_name": "string"
  }
}
```

#### get_campaign_details
```json
{
  "name": "get_campaign_details",
  "description": "Get details about the active campaign",
  "parameters": {},
  "returns": {
    "id": "integer",
    "name": "string",
    "description": "string | null",
    "modules": [{ "id": "integer", "name": "string", "module_number": "integer" }],
    "character_count": "integer",
    "document_count": "integer"
  }
}
```

#### export_campaign (NEW)
```json
{
  "name": "export_campaign",
  "description": "Export active campaign as a shareable data blob",
  "parameters": {
    "include_maps": { "type": "boolean", "default": true }
  },
  "returns": {
    "success": "boolean",
    "data": "string (base64 encoded JSON)",
    "size_bytes": "integer"
  }
}
```

#### import_campaign (NEW)
```json
{
  "name": "import_campaign",
  "description": "Import a campaign from exported data",
  "parameters": {
    "data": { "type": "string", "required": true, "description": "base64 encoded export data" },
    "new_name": { "type": "string", "description": "Optional new name for imported campaign" }
  },
  "returns": {
    "success": "boolean",
    "campaign_id": "integer",
    "campaign_name": "string"
  }
}
```

### Module Tools

#### create_module
```json
{
  "name": "create_module",
  "description": "Create a new module in the active campaign",
  "parameters": {
    "name": { "type": "string", "required": true },
    "description": { "type": "string" }
  },
  "returns": {
    "success": "boolean",
    "module_id": "integer",
    "name": "string",
    "module_number": "integer"
  }
}
```

#### list_modules
```json
{
  "name": "list_modules",
  "description": "List modules in the active campaign",
  "parameters": {},
  "returns": [{
    "id": "integer",
    "name": "string",
    "module_number": "integer",
    "monster_count": "integer",
    "item_count": "integer"
  }]
}
```

#### get_module_details
```json
{
  "name": "get_module_details",
  "description": "Get full details about a module",
  "parameters": {
    "module_id": { "type": "integer", "required": true }
  },
  "returns": {
    "id": "integer",
    "name": "string",
    "description": "string | null",
    "monsters": [{ "id": "integer", "name": "string", "quantity": "integer", "encounter_tag": "string | null" }],
    "items": [{ "id": "integer", "name": "string", "quantity": "integer", "location": "string | null" }],
    "npcs": [{ "id": "integer", "name": "string", "role": "string | null" }],
    "documents": [{ "id": "integer", "title": "string" }]
  }
}
```

#### add_monster_to_module
```json
{
  "name": "add_monster_to_module",
  "description": "Add a monster from the catalog to a module",
  "parameters": {
    "module_id": { "type": "integer", "required": true },
    "monster_name": { "type": "string", "required": true },
    "monster_source": { "type": "string", "required": true },
    "quantity": { "type": "integer", "default": 1 },
    "encounter_tag": { "type": "string" },
    "display_name": { "type": "string" },
    "notes": { "type": "string" }
  }
}
```

#### add_item_to_module
```json
{
  "name": "add_item_to_module",
  "description": "Add an item from the catalog to a module",
  "parameters": {
    "module_id": { "type": "integer", "required": true },
    "item_name": { "type": "string", "required": true },
    "item_source": { "type": "string", "required": true },
    "quantity": { "type": "integer", "default": 1 },
    "location": { "type": "string" },
    "notes": { "type": "string" }
  }
}
```

### Character Tools

#### list_characters
```json
{
  "name": "list_characters",
  "description": "List characters in the active campaign",
  "parameters": {
    "character_type": { "type": "string", "enum": ["pc", "npc", "all"], "default": "all" }
  },
  "returns": [{
    "id": "integer",
    "name": "string",
    "is_npc": "boolean",
    "race": "string | null",
    "class": "string | null",
    "level": "integer"
  }]
}
```

#### get_character
```json
{
  "name": "get_character",
  "description": "Get full character details",
  "parameters": {
    "character_id": { "type": "integer", "required": true }
  },
  "returns": {
    "id": "integer",
    "name": "string",
    "is_npc": "boolean",
    "race": "string | null",
    "class": "string | null",
    "level": "integer",
    "abilities": { "strength": "integer", "dexterity": "integer", "..." },
    "max_hp": "integer | null",
    "current_hp": "integer | null",
    "armor_class": "integer | null",
    "currency": { "copper": "integer", "silver": "integer", "..." },
    "inventory": [{ "name": "string", "quantity": "integer" }],
    "npc_role": "string | null",
    "npc_location": "string | null",
    "backstory": "string | null"
  }
}
```

#### create_character
```json
{
  "name": "create_character",
  "description": "Create a new character (PC or NPC)",
  "parameters": {
    "name": { "type": "string", "required": true },
    "is_npc": { "type": "boolean", "default": true },
    "race": { "type": "string" },
    "class": { "type": "string" },
    "level": { "type": "integer", "default": 1 },
    "background": { "type": "string" },
    "alignment": { "type": "string" },
    "npc_role": { "type": "string" },
    "npc_location": { "type": "string" },
    "backstory": { "type": "string" }
  }
}
```

#### edit_character
```json
{
  "name": "edit_character",
  "description": "Update character attributes",
  "parameters": {
    "character_id": { "type": "integer", "required": true },
    "name": { "type": "string" },
    "race": { "type": "string" },
    "class": { "type": "string" },
    "level": { "type": "integer" },
    "max_hp": { "type": "integer" },
    "current_hp": { "type": "integer" },
    "abilities": { "type": "object" },
    "npc_role": { "type": "string" },
    "npc_location": { "type": "string" },
    "backstory": { "type": "string" }
  }
}
```

### Document Tools

#### list_documents
```json
{
  "name": "list_documents",
  "description": "List documents in the active campaign",
  "parameters": {
    "module_id": { "type": "integer", "description": "Filter by module" }
  },
  "returns": [{
    "id": "integer",
    "title": "string",
    "document_type": "string | null",
    "module_id": "integer | null"
  }]
}
```

#### read_document
```json
{
  "name": "read_document",
  "description": "Read document content",
  "parameters": {
    "document_id": { "type": "integer", "required": true }
  },
  "returns": {
    "id": "integer",
    "title": "string",
    "content": "string"
  }
}
```

#### create_user_document
```json
{
  "name": "create_user_document",
  "description": "Create a new document",
  "parameters": {
    "title": { "type": "string", "required": true },
    "content": { "type": "string", "default": "" },
    "module_id": { "type": "integer" },
    "document_type": { "type": "string" }
  }
}
```

#### edit_document
```json
{
  "name": "edit_document",
  "description": "Edit document using search and replace",
  "parameters": {
    "document_id": { "type": "integer", "required": true },
    "search": { "type": "string", "required": true },
    "replace": { "type": "string", "required": true },
    "replace_all": { "type": "boolean", "default": false }
  }
}
```

#### export_document_markdown (NEW)
```json
{
  "name": "export_document_markdown",
  "description": "Export a document as a markdown file",
  "parameters": {
    "document_id": { "type": "integer", "required": true }
  },
  "returns": {
    "title": "string",
    "content": "string",
    "suggested_filename": "string"
  }
}
```

### Catalog Tools

#### search_monsters
```json
{
  "name": "search_monsters",
  "description": "Search the monster catalog",
  "parameters": {
    "name": { "type": "string" },
    "creature_type": { "type": "string" },
    "min_cr": { "type": "number" },
    "max_cr": { "type": "number" },
    "source": { "type": "string" },
    "limit": { "type": "integer", "default": 20 }
  }
}
```

#### search_items
```json
{
  "name": "search_items",
  "description": "Search the item catalog",
  "parameters": {
    "name": { "type": "string" },
    "item_type": { "type": "string" },
    "rarity": { "type": "string" },
    "source": { "type": "string" },
    "limit": { "type": "integer", "default": 20 }
  }
}
```

#### search_traps
```json
{
  "name": "search_traps",
  "description": "Search the trap/hazard catalog",
  "parameters": {
    "name": { "type": "string" },
    "category": { "type": "string", "enum": ["Trap", "Hazard"] },
    "source": { "type": "string" },
    "limit": { "type": "integer", "default": 20 }
  }
}
```

#### search_spells (NEW)
```json
{
  "name": "search_spells",
  "description": "Search the spell catalog",
  "parameters": {
    "name": { "type": "string", "description": "Partial spell name match" },
    "min_level": { "type": "integer", "description": "Minimum spell level (0 for cantrips)" },
    "max_level": { "type": "integer", "description": "Maximum spell level" },
    "school": { "type": "string", "enum": ["Abjuration", "Conjuration", "Divination", "Enchantment", "Evocation", "Illusion", "Necromancy", "Transmutation"] },
    "class": { "type": "string", "description": "Filter by casting class (e.g., 'Wizard', 'Cleric')" },
    "concentration": { "type": "boolean", "description": "Filter concentration spells" },
    "ritual": { "type": "boolean", "description": "Filter ritual spells" },
    "source": { "type": "string", "description": "Source book code (e.g., 'PHB')" },
    "limit": { "type": "integer", "default": 20 }
  },
  "returns": [{
    "name": "string",
    "level": "integer",
    "school": "string",
    "casting_time": "string",
    "range": "string",
    "components": "string",
    "duration": "string",
    "concentration": "boolean",
    "ritual": "boolean",
    "classes": ["string"],
    "source": "string",
    "description": "string (truncated)"
  }]
}
```

#### search_classes (NEW)
```json
{
  "name": "search_classes",
  "description": "Search the class catalog",
  "parameters": {
    "name": { "type": "string" },
    "source": { "type": "string" },
    "limit": { "type": "integer", "default": 20 }
  },
  "returns": [{
    "name": "string",
    "hit_die": "integer",
    "primary_ability": "string",
    "saving_throws": ["string"],
    "subclass_name": "string",
    "source": "string"
  }]
}
```

#### search_races (NEW)
```json
{
  "name": "search_races",
  "description": "Search the race catalog",
  "parameters": {
    "name": { "type": "string" },
    "size": { "type": "string", "enum": ["Tiny", "Small", "Medium", "Large"] },
    "source": { "type": "string" },
    "limit": { "type": "integer", "default": 20 }
  },
  "returns": [{
    "name": "string",
    "size": "string",
    "speed": "integer",
    "ability_bonuses": "object",
    "traits": ["string"],
    "source": "string"
  }]
}
```

#### search_feats (NEW)
```json
{
  "name": "search_feats",
  "description": "Search the feat catalog",
  "parameters": {
    "name": { "type": "string" },
    "has_prerequisite": { "type": "boolean" },
    "source": { "type": "string" },
    "limit": { "type": "integer", "default": 20 }
  },
  "returns": [{
    "name": "string",
    "prerequisite": "string | null",
    "description": "string (truncated)",
    "source": "string"
  }]
}
```

#### search_backgrounds (NEW)
```json
{
  "name": "search_backgrounds",
  "description": "Search the background catalog",
  "parameters": {
    "name": { "type": "string" },
    "source": { "type": "string" },
    "limit": { "type": "integer", "default": 20 }
  },
  "returns": [{
    "name": "string",
    "skill_proficiencies": ["string"],
    "tool_proficiencies": ["string"],
    "languages": "integer",
    "feature_name": "string",
    "source": "string"
  }]
}
```

## Tools Removed

- `create_document_from_template` - No more templates, just create documents
- `list_templates` - No more templates
- Session-related parameters on list_documents

## Dependencies
- Depends on: [[MIMIR-T-0358]] Service Layer API Design

## Progress

*To be updated during implementation*