---
id: implement-moduleservice-with-type
level: task
title: "Implement ModuleService with type-based document creation"
short_code: "MIMIR-T-0395"
created_at: 2026-01-21T03:02:30.509384+00:00
updated_at: 2026-01-21T03:02:30.509384+00:00
parent: MIMIR-I-0044
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0044
---

# Implement ModuleService with type-based document creation

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Implement `ModuleService` that handles module CRUD operations. On creation, the module type (mystery, dungeon, heist, etc.) determines which overview template is used. Each module gets exactly 2 documents: the type-specific overview and a blank play_notes document.

## Acceptance Criteria

- [ ] `ModuleService` struct with stateful connection pattern
- [ ] `create()` - creates module + 2 documents based on module type
- [ ] `list_for_campaign()` - list modules for a campaign
- [ ] `get()` - get module by ID
- [ ] `update()` - update module name/description
- [ ] `delete()` - delete module and related documents
- [ ] Module type selects correct overview template
- [ ] Play notes document always created (blank template)
- [ ] Unit tests for all operations
- [ ] Test: each module type creates correct template

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/services/
├── mod.rs              # Add module export
├── module.rs           # ModuleService implementation
```

### ModuleService API

```rust
pub struct ModuleService<'a> {
    conn: &'a mut SqliteConnection,
}

/// Module types that determine which template is used
pub enum ModuleType {
    General,
    Mystery,
    Dungeon,
    Heist,
    Horror,
    Political,
}

pub struct CreateModule {
    pub campaign_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub module_type: ModuleType,  // Transient - not stored in DB
}

impl<'a> ModuleService<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self;
    
    /// Create module with type-specific overview + play_notes
    pub fn create(&mut self, input: CreateModule) -> ServiceResult<Module>;
    
    pub fn list_for_campaign(&mut self, campaign_id: i32) -> ServiceResult<Vec<Module>>;
    pub fn get(&mut self, id: i32) -> ServiceResult<Option<Module>>;
    pub fn update(&mut self, id: i32, input: UpdateModule) -> ServiceResult<Module>;
    pub fn delete(&mut self, id: i32) -> ServiceResult<()>;
}
```

### Module Type → Template Mapping

| ModuleType | Template | doc_type stored |
|------------|----------|-----------------|
| `General` | module_overview.md | `module_overview` |
| `Mystery` | module_mystery.md | `module_overview` |
| `Dungeon` | module_dungeon.md | `module_overview` |
| `Heist` | module_heist.md | `module_overview` |
| `Horror` | module_horror.md | `module_overview` |
| `Political` | module_political.md | `module_overview` |

Note: All store `doc_type = "module_overview"` - the type just affects initial content.

### Documents Created on Module Creation

1. **Module Overview** - from type-specific template
2. **Play Notes** - blank document for session tracking

### Creation Flow

```rust
pub fn create(&mut self, input: CreateModule) -> ServiceResult<Module> {
    self.conn.transaction(|conn| {
        // 1. Insert module (module_type NOT stored)
        let module = dal::module::insert(conn, &NewModule {
            campaign_id: input.campaign_id,
            name: input.name,
            description: input.description,
        })?;
        
        // 2. Create overview doc from type-specific template
        let overview_content = templates::get_module_template(&input.module_type.to_string())
            .unwrap_or(templates::get_module_template("general").unwrap());
        dal::document::insert(conn, &NewDocument {
            campaign_id: input.campaign_id,
            module_id: Some(module.id),
            title: "Module Overview".to_string(),
            doc_type: "module_overview".to_string(),
            content: overview_content.to_string(),
        })?;
        
        // 3. Create blank play notes
        dal::document::insert(conn, &NewDocument {
            campaign_id: input.campaign_id,
            module_id: Some(module.id),
            title: "Play Notes".to_string(),
            doc_type: "play_notes".to_string(),
            content: templates::get_play_notes_template().to_string(),
        })?;
        
        Ok(module)
    })
}
```

### Dependencies

- MIMIR-T-0390 (ServiceError type)
- MIMIR-T-0393 (templates module)
- MIMIR-T-0396 (DocumentService / document DAL)
- Existing `dal::module` module

## Status Updates

*To be added during implementation*