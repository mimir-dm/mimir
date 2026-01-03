---
id: major-npc-tracker-catalog
level: initiative
title: "Major NPC Tracker Catalog Integration"
short_code: "MIMIR-I-0031"
created_at: 2026-01-03T13:33:36.766013+00:00
updated_at: 2026-01-03T13:33:36.766013+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: major-npc-tracker-catalog
---

# Major NPC Tracker Catalog Integration Initiative

Bridge the `major_npc_tracker.md` template with the existing character database system.

## Status Update

**Foundation work now in progress via T-0290/T-0291:**
- `module_npcs` table with `character_id` FK being created
- `ModuleNpcService` with character lookup being implemented
- Frontmatter sync for NPCs included in T-0292

**Remaining work for this initiative:**
- Update `major_npc_tracker.md` template front matter
- Generate companion `npc_stats.md` files from character data
- UI for NPC ↔ character linking

## Context

**Existing Infrastructure (already implemented):**
- `characters` table with `is_npc` flag - NPCs are stored as characters
- `CharacterData` struct with catalog references:
  - `SpellReference` (name + source)
  - `FeatureReference` (name + class_name + source + level)
  - `InventoryItem` (name + optional source)
  - NPC fields: `npc_role`, `npc_location`, `npc_faction`, `npc_notes`
  - `legendary_actions` for boss NPCs
- `character_versions` table for version history

**What's Missing:**
- No linking tables to associate NPCs with modules/regions/factions (only text fields in YAML)
- `major_npc_tracker.md` template doesn't sync with character database
- No companion file generation for NPC stat blocks

This initiative follows the `ModuleMonster` pattern: create linking tables rather than adding FKs to characters (allows same NPC in multiple contexts).

## Goals & Non-Goals

**Goals:**
- Create `module_npcs` linking table (following `module_monsters` pattern)
- Update `major_npc_tracker.md` template front matter to align with `CharacterData` schema
- Sync template front matter with existing character records
- Generate companion `npc_stats.md` files from character data

**Non-Goals:**
- Replacing the character system (we're extending it)
- Changing narrative sections of the NPC template
- Creating a full character builder

## Proposed Front Matter Schema

Aligned with existing `CharacterData` structure:

```yaml
# Links to existing character record (if exists)
character_id: null  # Optional - links to characters table

# Or inline definition (creates/updates character record)
stat_block:
  race: "Human"
  classes:
    - class_name: "Rogue"
      level: 5
      subclass: "Assassin"
  abilities:
    strength: 10
    dexterity: 16
    constitution: 12
    intelligence: 14
    wisdom: 12
    charisma: 14

# Catalog references (aligned with CharacterData)
spells:
  - name: "Disguise Self"
    source: "PHB"

equipment:
  - name: "Shortsword"
    source: "PHB"
    equipped: true
  - name: "Cloak of Elvenkind"
    source: "DMG"
    equipped: true

class_features:
  - name: "Sneak Attack"
    class_name: "Rogue"
    source: "PHB"
    level: 1

# Module context (for linking table)
module_context:
  role: "antagonist"  # quest_giver, ally, antagonist, informant, wild_card
  encounter_tag: "tavern_scene"
  notes: "Appears in Part 2"
```

## Implementation Plan

### Phase 1: Database Schema
- Create `module_npcs` linking table: id, module_id, character_id, role, encounter_tag, notes, created_at
- Similar to `module_monsters` but references `characters` table

### Phase 2: Backend Service
- Create `ModuleNPCService` following `ModuleMonsterService` pattern
- Add methods to link/unlink NPCs from modules
- Add method to get NPCs with full character data

### Phase 3: Template Update
- Update `major_npc_tracker.md` front matter schema
- Ensure alignment with `CharacterData` structure

### Phase 4: Sync Service
- Parse front matter and sync to character record
- Generate companion `npc_stats.md` with rendered stat blocks
- Integrate with document save workflow

## Files to Create/Modify

**New Files:**
- `crates/mimir-dm-core/src/models/campaign/module_npcs.rs`
- `crates/mimir-dm-core/src/services/module_npc_service.rs`
- `crates/mimir-dm-core/src/dal/campaign/module_npcs.rs`
- `crates/mimir-dm-core/migrations/0XX_create_module_npcs/`

**Modified Files:**
- `docs/src/campaign-framework/06-templates/templates/major_npc_tracker.md`
- `crates/mimir-dm-core/src/models/campaign/mod.rs`
- `crates/mimir-dm-core/src/services/mod.rs`
- `crates/mimir-dm-core/src/dal/campaign/mod.rs`
- `crates/mimir-dm-core/src/schema.rs`

## Key Design Decisions

1. **Linking table, not FKs**: Same NPC can appear in multiple modules with different roles
2. **Align with CharacterData**: Front matter uses same schema as existing character system
3. **Optional character_id**: Can reference existing character or define inline
4. **Follow ModuleMonster pattern**: Proven architecture for catalog linkage

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

{Describe the context and background for this initiative}

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- {Primary objective 1}
- {Primary objective 2}

**Non-Goals:**
- {What this initiative will not address}

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

{Delete if not a requirements-focused initiative}

### User Requirements
- **User Characteristics**: {Technical background, experience level, etc.}
- **System Functionality**: {What users expect the system to do}
- **User Interfaces**: {How users will interact with the system}

### System Requirements
- **Functional Requirements**: {What the system should do - use unique identifiers}
  - REQ-001: {Functional requirement 1}
  - REQ-002: {Functional requirement 2}
- **Non-Functional Requirements**: {How the system should behave}
  - NFR-001: {Performance requirement}
  - NFR-002: {Security requirement}

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

{Delete if not user-facing}

### Use Case 1: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

### Use Case 2: {Use Case Name}
- **Actor**: {Who performs this action}
- **Scenario**: {Step-by-step interaction}
- **Expected Outcome**: {What should happen}

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

{Delete if not technically complex}

### Overview
{High-level architectural approach}

### Component Diagrams
{Describe or link to component diagrams}

### Class Diagrams
{Describe or link to class diagrams - for OOP systems}

### Sequence Diagrams
{Describe or link to sequence diagrams - for interaction flows}

### Deployment Diagrams
{Describe or link to deployment diagrams - for infrastructure}

## Detailed Design **[REQUIRED]**

{Technical approach and implementation details}

## UI/UX Design **[CONDITIONAL: Frontend Initiative]**

{Delete if no UI components}

### User Interface Mockups
{Describe or link to UI mockups}

### User Flows
{Describe key user interaction flows}

### Design System Integration
{How this fits with existing design patterns}

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

{Delete if covered by separate testing initiative}

### Unit Testing
- **Strategy**: {Approach to unit testing}
- **Coverage Target**: {Expected coverage percentage}
- **Tools**: {Testing frameworks and tools}

### Integration Testing
- **Strategy**: {Approach to integration testing}
- **Test Environment**: {Where integration tests run}
- **Data Management**: {Test data strategy}

### System Testing
- **Strategy**: {End-to-end testing approach}
- **User Acceptance**: {How UAT will be conducted}
- **Performance Testing**: {Load and stress testing}

### Test Selection
{Criteria for determining what to test}

### Bug Tracking
{How defects will be managed and prioritized}

## Alternatives Considered **[REQUIRED]**

{Alternative approaches and why they were rejected}

## Implementation Plan **[REQUIRED]**

{Phases and timeline for execution}