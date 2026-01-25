---
id: token-repository-layer-crud-with
level: task
title: "Token repository layer (CRUD with sqlx)"
short_code: "MIMIR-T-0413"
created_at: 2026-01-25T02:44:09.787463+00:00
updated_at: 2026-01-25T16:04:43.187486+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# Token repository layer (CRUD with sqlx)

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Implement the data access layer for tokens using sqlx, providing CRUD operations and queries needed by Tauri commands.

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

- [ ] Token model struct defined with sqlx derives
- [ ] `create_token()` - insert new token, return created record
- [ ] `get_token()` - fetch single token by ID
- [ ] `list_tokens_for_map()` - fetch all tokens for a map with joined monster/character names
- [ ] `update_token()` - update token fields
- [ ] `update_token_position()` - optimized position-only update for drag operations
- [ ] `delete_token()` - remove token by ID
- [ ] `toggle_token_visibility()` - flip visible_to_players boolean

## Implementation Notes

### Model Struct

```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Token {
    pub id: i64,
    pub map_id: i64,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub x: f64,
    pub y: f64,
    pub visible_to_players: bool,
    pub color: Option<String>,
    pub monster_id: Option<i64>,
    pub character_id: Option<i64>,
    pub vision_type: String,
    pub vision_range_ft: Option<i32>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// For list queries with joined names
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct TokenWithData {
    #[sqlx(flatten)]
    pub token: Token,
    pub monster_name: Option<String>,
    pub character_name: Option<String>,
    pub token_image_path: Option<String>,  // from monsters.token_image_path
}
```

### Files to Create/Modify

- `crates/mimir-core/src/models/token.rs` (new)
- `crates/mimir-core/src/db/token.rs` (new)
- `crates/mimir-core/src/db/mod.rs` (add module)
- `crates/mimir-core/src/models/mod.rs` (add module)

### Dependencies

- MIMIR-T-0412 (Token table schema)

## Status Updates

*To be added during implementation*