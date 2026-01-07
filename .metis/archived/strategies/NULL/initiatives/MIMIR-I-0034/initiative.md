---
id: ux-flow-improvements
level: initiative
title: "UX Flow Improvements"
short_code: "MIMIR-I-0034"
created_at: 2026-01-03T14:18:48.480164+00:00
updated_at: 2026-01-03T14:18:48.480164+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
strategy_id: NULL
initiative_id: ux-flow-improvements
---

# UX Flow Improvements Initiative

Reduce friction in common workflows by applying "ceremony for milestones, efficiency for routine."

## Context

The app's workflow is methodologically sound but creates friction through:
- Stage guidance that's always fully expanded (cognitive overload)
- Minor UX inconsistencies (directory naming)
- Linear stage progression that feels like gatekeeping rather than guidance

The "bogged down" feeling comes from treating every action with equal ceremony.

## Goals

- Reduce cognitive load on routine tasks
- Preserve ceremony for significant milestones (new campaign, stage advancement)
- Quick wins that compound into meaningful friction reduction

## Scope

### Tasks

| Code | Title | Effort | Priority |
|------|-------|--------|----------|
| T-0294 | Collapse stage guidance after first visit | 2-3 hrs | High |
| T-0295 | Kebab-case campaign directory defaults | 30 min | High |
| T-0285 | UX Quick Wins: Reduce Friction | - | High |
| T-0286 | Command Palette for Quick Navigation | 4-6 hrs | Low |
| T-0287 | Unified Campaign Dashboard Architecture | 3-4 wks | Low |
| T-0288 | ModulePlayView Prep/Play Mode Split | 2-3 wks | Medium |

## Key Principle

**Ceremony for milestones, efficiency for routine.**
- Starting a new campaign = ceremony (exciting!)
- Switching between documents = efficiency (routine)
- Advancing to Active stage = ceremony (ready to play!)
- Checking off a document = efficiency (just marking progress)

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