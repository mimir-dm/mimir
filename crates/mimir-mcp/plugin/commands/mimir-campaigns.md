---
description: List all available Mimir campaigns and their status
allow_override: false
---

# List Mimir Campaigns

Use the `list_campaigns` MCP tool to show all available D&D campaigns.

For each campaign, display:
- Campaign name
- Campaign ID
- Creation date
- Whether it's the active campaign

If no campaigns exist, inform the user they can create one in the Mimir desktop app.

After listing, ask the user if they want to set one as active using `set_active_campaign`.
