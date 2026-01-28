---
description: Create a new D&D module in the active Mimir campaign
arguments:
  - name: name
    description: Name of the module (e.g., "The Sunless Citadel")
    required: true
  - name: description
    description: Brief description of the module's theme or purpose
    required: false
allow_override: false
---

# Create a New Module

First verify there's an active campaign using `get_campaign_details`. If none is active, list campaigns and ask the user to select one.

Create a new module using `create_module` with the provided name and description.

After creation:
1. Report the module ID
2. Offer to add initial content:
   - Module overview document
   - Backstory document
   - DM notes
3. Ask if the user wants to populate with monsters or items
