---
id: create-map-model-and-mapservice-in
level: task
title: "Create Map model and MapService in mimir-dm-core"
short_code: "MIMIR-T-0196"
created_at: 2025-12-20T22:24:17.269573+00:00
updated_at: 2025-12-21T01:45:13.663813+00:00
parent: MIMIR-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0015
---

# Create Map model and MapService in mimir-dm-core

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective

Create the Rust model and service layer for map CRUD operations, following the existing service patterns in mimir-dm-core.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Map` struct with Diesel schema mapping
- [ ] `NewMap` struct for insertions
- [ ] `MapService` with CRUD operations
- [ ] `list_campaign_maps(campaign_id)` - get campaign-level maps
- [ ] `list_module_maps(module_id)` - get module-specific maps
- [ ] `get_map(id)` - get single map with all fields
- [ ] `create_map(new_map)` - insert new map
- [ ] `update_map(id, updates)` - update grid config, name, etc.
- [ ] `delete_map(id)` - remove map and image file

## Implementation Notes

### File Locations
- Model: `crates/mimir-dm-core/src/models/map.rs`
- Service: `crates/mimir-dm-core/src/services/map_service.rs`
- Schema: Add to `crates/mimir-dm-core/src/schema.rs`

### Model Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = maps)]
pub struct Map {
    pub id: i32,
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub name: String,
    pub image_path: String,
    pub width_px: i32,
    pub height_px: i32,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub created_at: String,
    pub updated_at: String,
}

pub enum GridType {
    Square,
    Hex,
    None,
}
```

### Dependencies
- MIMIR-T-0195 (database schema must exist first)