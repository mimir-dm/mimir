---
id: migration-campaignasset-table-and
level: task
title: "Migration: CampaignAsset table and blob storage"
short_code: "MIMIR-T-0383"
created_at: 2026-01-20T21:49:41.767887+00:00
updated_at: 2026-01-20T22:13:48.105738+00:00
parent: MIMIR-I-0043
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0043
---

# Migration: CampaignAsset table and blob storage

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create the CampaignAsset table for tracking user-uploaded images (PNGs, SVGs for puzzles, artifacts, props). Files stored in app data directory, DB tracks metadata and path.

## Schema

```sql
CREATE TABLE campaign_assets (
    id TEXT PRIMARY KEY NOT NULL,
    -- Parent (exactly one must be set)
    campaign_id TEXT REFERENCES campaigns(id) ON DELETE CASCADE,
    module_id TEXT REFERENCES modules(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    blob_path TEXT NOT NULL,
    file_size INTEGER,
    uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (
        (campaign_id IS NOT NULL AND module_id IS NULL) OR
        (campaign_id IS NULL AND module_id IS NOT NULL)
    )
);

CREATE INDEX idx_campaign_assets_campaign ON campaign_assets(campaign_id);
CREATE INDEX idx_campaign_assets_module ON campaign_assets(module_id);
```

## File Storage

```
{app_data_dir}/assets/{campaign_id}/{uuid}.{ext}
```

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create migration with up.sql and down.sql
- [ ] Run diesel migration and update schema.rs
- [ ] Create Rust models: CampaignAsset, NewCampaignAsset
- [ ] Create DAL functions: insert, get, list_by_campaign, delete
- [ ] Create asset storage utilities (save_asset, resolve_path, delete_file)
- [ ] Validate mime_type against allowed list
- [ ] Add tests for CRUD and file operations

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates

### 2026-01-20
- Updated schema: assets can belong to campaign OR module (CHECK constraint)
- Created migration 010_campaign_assets
- Created CampaignAsset, NewCampaignAsset models with for_campaign/for_module constructors
- Added ALLOWED_MIME_TYPES and validation helpers
- Created DAL with campaign/module-specific list operations
- All 366 tests passing