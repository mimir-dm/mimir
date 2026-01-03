---
id: region-random-encounter-integration
level: initiative
title: "Region Random Encounter Integration"
short_code: "MIMIR-I-0032"
created_at: 2026-01-03T13:33:36.870573+00:00
updated_at: 2026-01-03T13:33:36.870573+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: region-random-encounter-integration
---

# Region Random Encounter Integration Initiative

Add structured random encounter tables to region templates with catalog creature references.

## Context

Region documents define the areas where adventures take place, including random encounter tables. Currently these are free-form markdown tables. This initiative adds machine-parseable front matter for encounter tables, enabling:
- Automatic stat block generation for encounter creatures
- Portable encounter data for import/export
- Validation that referenced creatures exist in catalog

## Goals & Non-Goals

**Goals:**
- Add structured YAML front matter for random encounter tables
- Create `RegionEncounter` model linking regions to creatures
- Generate formatted encounter tables with creature stats
- Reference notable NPCs from campaign_npcs table

**Non-Goals:**
- Changing the narrative sections of the region template
- Building a full encounter generator (just linking to catalog)

## Proposed Front Matter Schema

```yaml
random_encounters:
  - roll: "2-3"
    creatures:
      - name: "[Monster]"
        source: MM
        quantity: "1d4"
  - roll: "4-5"
    creatures:
      - name: "[Monster]"
        source: MM
        quantity: 2

notable_npcs:
  - name: "[NPC Name]"
    type: campaign | catalog
    source: campaign          # or "VGTM" for catalog
    location: "[Settlement]"
    role: power_player | local_notable | common_folk
```

## Implementation Plan

### Phase 1: Database Schema
- Create `region_encounters` migration with: id, region_id, roll_range, created_at
- Create `region_encounter_creatures` join table with quantity field
- Create `region_npcs` join table linking to campaign_npcs

### Phase 2: Backend Models
- Create `RegionEncounter` model
- Create `RegionFrontmatter` struct for parsing
- Create `RegionEncounterService`

### Phase 3: Template Update
- Update `docs/src/campaign-framework/06-templates/templates/region_overview.md`

### Phase 4: Sync Service
- Generate formatted encounter table markdown with creature stats
- Generate NPC quick reference for region

## Files to Create/Modify

**New Files:**
- `crates/mimir-dm-core/src/models/campaign/region_encounters.rs`
- `crates/mimir-dm-core/src/models/campaign/region_frontmatter.rs`
- `crates/mimir-dm-core/src/services/region_encounter_service.rs`
- `crates/mimir-dm-core/src/dal/campaign/region_encounters.rs`
- `crates/mimir-dm-core/migrations/0XX_create_region_encounters/`

**Modified Files:**
- `docs/src/campaign-framework/06-templates/templates/region_overview.md`
- `crates/mimir-dm-core/src/models/campaign/mod.rs`
- `crates/mimir-dm-core/src/services/mod.rs`

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