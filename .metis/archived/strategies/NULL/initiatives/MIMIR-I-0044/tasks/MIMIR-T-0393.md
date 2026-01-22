---
id: create-templates-module-with
level: task
title: "Create templates module with embedded campaign/module templates"
short_code: "MIMIR-T-0393"
created_at: 2026-01-21T03:02:30.165811+00:00
updated_at: 2026-01-21T03:33:50.064383+00:00
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

# Create templates module with embedded campaign/module templates

## Parent Initiative

[[MIMIR-I-0044]] - Implement Service Layer for mimir-core

## Objective

Create a templates module that embeds all campaign and module document templates at compile time using `include_str!`. This provides the initial content for documents created when campaigns and modules are created.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `src/templates/mod.rs` module created and exported
- [x] All 11 campaign templates embedded and accessible
- [x] All 7 module templates embedded (6 type-specific + play_notes)
- [x] `get_campaign_template(doc_type: &str) -> Option<&'static str>`
- [x] `get_module_template(module_type: &str) -> Option<&'static str>`
- [x] `get_play_notes_template() -> &'static str`
- [x] `list_campaign_templates() -> Vec<(&'static str, &'static str)>` (type, title)
- [x] Unit tests verify all templates are accessible (9 tests)

## Implementation Notes

### Files to Create

```
crates/mimir-core/src/templates/
├── mod.rs          # Template access functions
└── (templates embedded from docs/src/campaign-framework/06-templates/templates/)
```

### Campaign Templates (11)

| doc_type | Source File | Title |
|----------|-------------|-------|
| `campaign_pitch` | campaign_pitch.md | Campaign Pitch |
| `starting_scenario` | starting_scenario.md | Starting Scenario |
| `world_primer` | world_primer.md | World Primer |
| `character_guidelines` | character_guidelines.md | Character Guidelines |
| `table_expectations` | table_expectations.md | Table Expectations |
| `character_integration` | character_integration.md | Character Integration |
| `campaign_bible` | campaign_bible.md | Campaign Bible |
| `safety_tools` | safety_tools.md | Safety Tools |
| `house_rules` | house_rules.md | House Rules |
| `player_secrets` | player_secrets.md | Player Secrets |
| `faction_overview` | faction_overview.md | Faction Overview |

### Module Templates (7)

| module_type | Source File | Title |
|-------------|-------------|-------|
| `general` | module_overview.md | Module Overview |
| `mystery` | module_mystery.md | Mystery Module |
| `dungeon` | module_dungeon.md | Dungeon Crawl |
| `heist` | module_heist.md | Heist Module |
| `horror` | module_horror.md | Horror Module |
| `political` | module_political.md | Political Intrigue |
| (play_notes) | play_notes.md | Play Notes |

### Embedding Pattern

```rust
pub mod templates {
    // Campaign templates
    pub const CAMPAIGN_PITCH: &str = include_str!("../../../docs/src/campaign-framework/06-templates/templates/campaign_pitch.md");
    // ... etc
    
    pub fn get_campaign_template(doc_type: &str) -> Option<&'static str> {
        match doc_type {
            "campaign_pitch" => Some(CAMPAIGN_PITCH),
            // ...
            _ => None,
        }
    }
    
    pub fn get_module_template(module_type: &str) -> Option<&'static str> {
        match module_type {
            "general" => Some(MODULE_OVERVIEW),
            "mystery" => Some(MODULE_MYSTERY),
            // ...
            _ => None,
        }
    }
}
```

### Dependencies

- Template markdown files at `docs/src/campaign-framework/06-templates/templates/`
- No runtime dependencies - all compile-time embedding

## Status Updates

### Completed - 2026-01-20

Created `crates/mimir-core/src/templates/mod.rs` with:
- 11 campaign templates embedded via `include_str!`
- 6 module templates + play_notes template
- `CampaignTemplateInfo` and `ModuleTemplateInfo` structs with metadata
- `CAMPAIGN_TEMPLATES` and `MODULE_TEMPLATES` const arrays
- Access functions: `get_campaign_template()`, `get_module_template()`, `get_play_notes_template()`
- List functions: `list_campaign_templates()`, `list_module_templates()`
- 9 unit tests verifying all templates load and have content