---
id: implement-campaignservice-with
level: task
title: "Implement CampaignService with document creation"
short_code: "MIMIR-T-0394"
created_at: 2026-01-21T03:02:30.337707+00:00
updated_at: 2026-01-21T03:40:52.607869+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0044
---

# Implement CampaignService with document creation

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Implement `CampaignService` that handles campaign CRUD operations and automatically creates all 11 initial documents when a new campaign is created. This is the primary entry point for campaign management.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `CampaignService` struct with stateful connection pattern
- [x] `create()` - creates campaign + 11 documents in transaction
- [x] `list()` - list campaigns with optional archived filter
- [x] `get()` - get campaign by ID
- [x] `update()` - update campaign name/description
- [x] `archive()` - soft-delete campaign (sets archived_at)
- [x] `delete()` - hard delete campaign and all related data
- [x] All 11 documents created from embedded templates
- [x] Unit tests for all operations (12 tests)
- [x] Integration test: create campaign → verify 11 documents exist

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/services/
├── mod.rs              # Add campaign module export
├── campaign.rs         # CampaignService implementation
```

### CampaignService API

```rust
pub struct CampaignService<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> CampaignService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self;
    
    /// Create campaign with all 11 initial documents
    pub fn create(&mut self, input: CreateCampaign) -> ServiceResult<Campaign>;
    
    pub fn list(&mut self, include_archived: bool) -> ServiceResult<Vec<Campaign>>;
    pub fn get(&mut self, id: i32) -> ServiceResult<Option<Campaign>>;
    pub fn update(&mut self, id: i32, input: UpdateCampaign) -> ServiceResult<Campaign>;
    pub fn archive(&mut self, id: i32) -> ServiceResult<()>;
    pub fn delete(&mut self, id: i32) -> ServiceResult<()>;
}
```

### Documents Created on Campaign Creation

| doc_type | Title |
|----------|-------|
| `campaign_pitch` | Campaign Pitch |
| `starting_scenario` | Starting Scenario |
| `world_primer` | World Primer |
| `character_guidelines` | Character Guidelines |
| `table_expectations` | Table Expectations |
| `character_integration` | Character Integration |
| `campaign_bible` | Campaign Bible |
| `safety_tools` | Safety Tools |
| `house_rules` | House Rules |
| `player_secrets` | Player Secrets |
| `faction_overview` | Faction Overview |

### Transaction Pattern

```rust
pub fn create(&mut self, input: CreateCampaign) -> ServiceResult<Campaign> {
    self.conn.transaction(|conn| {
        // 1. Insert campaign
        let campaign = dal::campaign::insert(conn, &new_campaign)?;
        
        // 2. Create all 11 documents
        for (doc_type, title) in templates::list_campaign_templates() {
            let content = templates::get_campaign_template(doc_type)
                .ok_or_else(|| ServiceError::Validation("Missing template".into()))?;
            dal::document::insert(conn, &NewDocument {
                campaign_id: campaign.id,
                module_id: None,
                title: title.to_string(),
                doc_type: doc_type.to_string(),
                content: content.to_string(),
            })?;
        }
        
        Ok(campaign)
    })
}
```

### Dependencies

- MIMIR-T-0390 (ServiceError type)
- MIMIR-T-0393 (templates module)
- Existing `dal::campaign` module
- Existing `dal::document` module (or create in T-0396)

## Status Updates

### Completed - 2026-01-20

Created `crates/mimir-core/src/services/campaign.rs` with:
- `CampaignService` with stateful connection pattern
- `CreateCampaignInput` and `UpdateCampaignInput` types
- Full CRUD: create, list, get, update, archive, unarchive, delete
- `create()` uses diesel transaction to insert campaign + 11 documents atomically
- Documents populated from `templates::CAMPAIGN_TEMPLATES`
- 12 unit tests covering all operations