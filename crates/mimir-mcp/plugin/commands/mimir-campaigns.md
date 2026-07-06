---
description: List all available Mimir campaigns and their status
allow_override: false
---

# List Mimir Campaigns

Use the `list_campaigns` MCP tool to show all available D&D campaigns, and `get_active_campaign` to learn which one is currently active (`list_campaigns` does not include active status, so compare its `id` against the id returned by `get_active_campaign`).

For each campaign, display:
- Campaign name
- Campaign ID
- Creation date
- Whether it's the active campaign (matches `get_active_campaign`)

If no campaigns exist, offer to create one using `create_campaign`.

After listing, ask the user if they want to set one as active using `set_active_campaign`.
