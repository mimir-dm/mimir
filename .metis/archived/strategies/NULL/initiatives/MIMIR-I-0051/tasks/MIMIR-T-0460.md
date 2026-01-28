---
id: mcp-tools-for-campaign-export
level: task
title: "MCP Tools for Campaign Export/Import"
short_code: "MIMIR-T-0460"
created_at: 2026-01-28T04:02:49.722532+00:00
updated_at: 2026-01-28T14:16:06.654980+00:00
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

# MCP Tools for Campaign Export/Import

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0051]]

## Objective

Add MCP tools for campaign export/import to enable Claude-driven campaign sharing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `export_campaign` MCP tool - exports active campaign
- [ ] `import_campaign` MCP tool - imports from base64 data or file path
- [ ] Tools return appropriate status and error messages
- [ ] Add to mimir-mcp handler.rs tool list

## Tool Definitions

```json
{
  "name": "export_campaign",
  "description": "Export the active campaign as a shareable archive",
  "parameters": {
    "output_path": { "type": "string", "description": "Path to save the archive" }
  },
  "returns": {
    "success": "boolean",
    "path": "string",
    "size_bytes": "integer"
  }
}

{
  "name": "import_campaign",
  "description": "Import a campaign from an archive file",
  "parameters": {
    "archive_path": { "type": "string", "required": true },
    "new_name": { "type": "string", "description": "Optional new name for the campaign" }
  },
  "returns": {
    "success": "boolean",
    "campaign_id": "string",
    "campaign_name": "string"
  }
}
```

## Dependencies

- MIMIR-I-0050 (MCP Server Migration) - mimir-mcp crate must exist
- MIMIR-T-0456 (Export Service)
- MIMIR-T-0457 (Import Service)

## Status Updates

### Completed
- Added `export_campaign_tool()` definition - exports active campaign to specified directory
- Added `import_campaign_tool()` definition - imports archive with optional rename
- Added `preview_archive_tool()` definition - preview contents without importing
- Implemented `export_campaign()` handler - uses ArchiveService, returns path and size
- Implemented `import_campaign()` handler - imports and sets as active campaign
- Implemented `preview_archive()` handler - returns campaign name, counts, catalog references
- Registered all tools in handler.rs `get_tools()` and `execute_tool()`
- Build passes successfully