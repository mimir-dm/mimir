---
id: frontend-modal-system-consolidation
level: initiative
title: "Frontend Modal System Consolidation"
short_code: "MIMIR-I-0029"
created_at: 2025-12-29T14:50:21.505678+00:00
updated_at: 2025-12-29T14:50:21.505678+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: frontend-modal-system-consolidation
---

# Frontend Modal System Consolidation Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

The frontend has accumulated significant modal-related tech debt:
- **17 Vue components** with inline `modal-overlay` template code
- **8+ components** with duplicated scoped modal CSS
- **2 conflicting CSS files** (`modals.css` and `base-modal.css`) both defining `.modal-overlay`
- **1 specialized modal** (`BaseModal.vue`) that only handles D&D content display
- **No general-purpose** slot-based modal wrapper component

This leads to inconsistent UX, duplicated code, and maintenance burden when updating modal behavior.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Consolidate `modals.css` and `base-modal.css` into single source of truth
- Create slot-based `AppModal.vue` component for general-purpose modals
- Migrate existing modals to use shared component (remove duplicated template/CSS)
- Consistent modal UX across the application

**Non-Goals:**
- Changing modal visual design (keep current look)
- Refactoring BaseModal.vue (specialized for D&D content, leave as-is)

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

**Phase 1: CSS Consolidation**
- Merge `modals.css` and `base-modal.css` into unified `modals.css`
- Remove `base-modal.css` import from `main.css`
- Verify no visual regressions

**Phase 2: Create AppModal Component**
- Create `AppModal.vue` with slots: header, default (body), footer
- Props: visible, title, size (sm/md/lg/xl), closable
- Emit: close
- Handle overlay click, escape key, focus trap

**Phase 3: Migrate Modals (incremental)**
- Start with simpler modals (CreateModuleModal, MapUploadModal)
- Migrate management modals (BookManagementModal, CampaignManagementModal)
- Migrate complex modals (CharacterCreationWizard, etc.)
- Remove scoped CSS as each modal is migrated

**Affected Components (17):**
- BookManagementModal, CampaignManagementModal
- MapGridConfigModal, MapUploadModal, CreateModuleModal
- CharacterCreationWizard, LevelUpDialog, InventoryManager
- MapTokenSetupModal, QuickAddTokenModal
- PdfPreviewModal, ChatSidebar (delete modal)
- ReaderView, ModulePlayView, ModuleListView
- SourceSearch, SearchView