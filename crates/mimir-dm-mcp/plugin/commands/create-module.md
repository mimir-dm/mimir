---
name: create-module
description: Create a new D&D module in the active Mimir campaign
args:
  - name: name
    description: Name for the new module
    required: true
  - name: type
    description: "Module type: mystery, dungeon, heist, horror, or political"
    required: false
    default: dungeon
---

Create a new module in the active Mimir campaign.

## Steps

1. First check if there's an active campaign using `list_campaigns`
2. If no campaign is set, ask the user to select one using `set_active_campaign`
3. Create the module using `create_module` with:
   - name: {{name}}
   - module_type: {{type}}
4. After creation, use `get_module_details` to show the created module
5. List the auto-generated documents with `list_documents`

## Module Types

- **mystery** - Investigation-focused adventures with clue tracking
- **dungeon** - Classic dungeon crawls with room-by-room structure
- **heist** - Stealth and planning adventures
- **horror** - Dark, atmospheric scenarios
- **political** - Intrigue and social encounters

Present the created module details and suggest next steps like adding monsters, NPCs, or editing the overview document.
