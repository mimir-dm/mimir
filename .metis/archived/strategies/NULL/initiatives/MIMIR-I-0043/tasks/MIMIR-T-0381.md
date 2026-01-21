---
id: migration-campaign-campaignsource
level: task
title: "Migration: Campaign, CampaignSource, Module tables"
short_code: "MIMIR-T-0381"
created_at: 2026-01-20T21:49:40.975605+00:00
updated_at: 2026-01-20T22:02:43.365498+00:00
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

# Migration: Campaign, CampaignSource, Module tables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0043]]

## Objective

Create the core campaign hierarchy tables: Campaign (top-level container), CampaignSource (allowed source books), and Module (adventure chapter container).

## Schema

```sql
CREATE TABLE campaigns (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE campaign_sources (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    source_code TEXT NOT NULL REFERENCES catalog_sources(code),
    UNIQUE(campaign_id, source_code)
);

CREATE TABLE modules (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    module_number INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, module_number)
);

CREATE INDEX idx_modules_campaign ON modules(campaign_id);
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create migration with up.sql and down.sql
- [ ] Run diesel migration and update schema.rs
- [ ] Create Rust models: Campaign, NewCampaign, CampaignSource, NewCampaignSource, Module, NewModule
- [ ] Create DAL functions for CRUD operations
- [ ] Add tests for all DAL operations
- [ ] Verify CASCADE delete works (deleting campaign removes sources and modules)

## Implementation Notes

- Follow established patterns from catalog migrations (MIMIR-I-0042)
- CampaignSource.source_code references catalog_sources.code
- module_number allows ordering within a campaign
- archived_at is ISO8601 timestamp, NULL means active campaign

## Status Updates

### 2026-01-20
- Created migration 009_campaigns with Campaign, CampaignSource, Module tables
- Created Rust models with NewX and UpdateX variants
- Created DAL operations with full CRUD + helper functions
- Fixed FTS schema conflicts in schema.rs
- All 352 tests passing