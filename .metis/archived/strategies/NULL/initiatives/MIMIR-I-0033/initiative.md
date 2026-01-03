---
id: faction-forces-and-assets
level: initiative
title: "Faction Forces and Assets Integration"
short_code: "MIMIR-I-0033"
created_at: 2026-01-03T13:33:36.989332+00:00
updated_at: 2026-01-03T13:33:36.989332+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: faction-forces-and-assets
---

# Faction Forces and Assets Integration Initiative

Add catalog-linked forces, key members, and artifacts to faction templates.

## Context

Factions are major campaign elements with military forces (creatures), key NPCs, and significant items/artifacts. Currently this information is free-form markdown. This initiative adds machine-parseable front matter enabling:
- Automatic force roster with stat blocks
- Key member stat blocks from campaign_npcs
- Artifact/item catalog references

## Goals & Non-Goals

**Goals:**
- Add structured YAML front matter for forces, key_members, artifacts
- Create `FactionForce` model for military forces (creature references)
- Create `FactionArtifact` model for significant items
- Link key members to `campaign_npcs` table
- Generate companion files for faction stat blocks

**Non-Goals:**
- Building faction relationship graphs (that's narrative)
- Automatic faction conflict simulation

## Proposed Front Matter Schema

```yaml
forces:
  - name: "[Creature Type]"
    source: MM
    quantity: "[number or estimate]"
    role: standing | elite | summonable
    notes: "Guard standard"

key_members:
  - name: "[NPC Name]"
    role: leader | lieutenant | specialist
    stat_block:
      name: "[Base Stat Block]"
      source: MM
    equipment:
      - name: "[Item]"
        source: DMG

artifacts:
  - name: "[Item Name]"
    source: DMG
    holder: "[Who has it]"
    significance: "[Why it matters]"
```

## Implementation Plan

### Phase 1: Database Schema
- Create `faction_forces` migration with: id, faction_id, creature_name, creature_source, quantity, role
- Create `faction_artifacts` migration with: id, faction_id, item_name, item_source, holder, significance
- Link faction key_members through campaign_npcs with faction_id FK

### Phase 2: Backend Models
- Create `FactionForce` model
- Create `FactionArtifact` model
- Create `FactionFrontmatter` struct for parsing
- Create `FactionAssetsService`

### Phase 3: Template Update
- Update `docs/src/campaign-framework/06-templates/templates/faction_template.md`

### Phase 4: Sync Service
- Generate force roster markdown with creature stats
- Generate artifact list with item descriptions
- Generate key member quick stats

## Files to Create/Modify

**New Files:**
- `crates/mimir-dm-core/src/models/campaign/faction_forces.rs`
- `crates/mimir-dm-core/src/models/campaign/faction_artifacts.rs`
- `crates/mimir-dm-core/src/models/campaign/faction_frontmatter.rs`
- `crates/mimir-dm-core/src/services/faction_assets_service.rs`
- `crates/mimir-dm-core/src/dal/campaign/faction_forces.rs`
- `crates/mimir-dm-core/src/dal/campaign/faction_artifacts.rs`
- `crates/mimir-dm-core/migrations/0XX_create_faction_forces/`
- `crates/mimir-dm-core/migrations/0XX_create_faction_artifacts/`

**Modified Files:**
- `docs/src/campaign-framework/06-templates/templates/faction_template.md`
- `crates/mimir-dm-core/src/models/campaign/mod.rs`
- `crates/mimir-dm-core/src/services/mod.rs`

## Dependencies

- Requires MIMIR-I-0031 (campaign_npcs table) for key_members linkage

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