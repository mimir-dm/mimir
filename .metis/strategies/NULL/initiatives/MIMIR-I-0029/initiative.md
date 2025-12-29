---
id: frontend-modal-system-consolidation
level: initiative
title: "Frontend Modal System Consolidation"
short_code: "MIMIR-I-0029"
created_at: 2025-12-29T14:50:21.505678+00:00
updated_at: 2025-12-29T15:12:59.731358+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


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

## Discovery Findings

### CSS File Analysis

| File | Lines | Location | Container Class | Overlay Style |
|------|-------|----------|-----------------|---------------|
| `modals.css` | 300 | `styles/components/` | `.modal-content` | `rgba(0,0,0,0.8)` |
| `base-modal.css` | 356 | `styles/components/` | `.modal-container` | `var(--color-overlay)` + blur |

**Import order in `main.css`:**
- Line 17: `@import './components/modals.css';`
- Line 30: `@import './components/base-modal.css';`

**Conflict:** Both define `.modal-overlay` - `base-modal.css` wins due to later import order.

**Recommendation:** Keep `base-modal.css` (BEM naming, exit animations, variants, print styles). Merge unique `modals.css` styles, then delete it.

### Components with Inline Modal Template (17 files)

All use pattern: `<div class="modal-overlay">` → `<div class="modal-content|dialog-content|wizard-content">`

| Component | Nested Modals | Container Class | Notes |
|-----------|---------------|-----------------|-------|
| BookManagementModal | Yes (delete) | modal-content | |
| CampaignManagementModal | Yes (delete) | modal-content | |
| PlayerManager | Yes (2 dialogs) | modal-content, dialog-content | |
| MapUploadModal | No | modal-content | |
| MapGridConfigModal | No | modal-content | |
| CreateModuleModal | No | modal-content | Simple |
| MapTokenSetupModal | No | modal-content | |
| QuickAddTokenModal | No | modal-content | |
| PdfPreviewModal | No | modal-content | Large modal |
| CharacterCreationWizard | No | wizard-content | Multi-step |
| LevelUpDialog | No | dialog-content | |
| InventoryManager | No | dialog-content | |
| ChatSidebar | Yes (delete) | modal-content | |
| ReaderView | No | modal-content | D&D content |
| ModulePlayView | No | modal-content | D&D content |
| ModuleListView | No | modal-content | Simple |
| BaseModal | No | modal-content | **Keep as-is** (specialized) |

### Components with Scoped Modal CSS (13 files)

Each reimplements `.modal-overlay` styles in `<style scoped>`:
- BookManagementModal, CampaignManagementModal, PlayerManager
- MapUploadModal, MapGridConfigModal, MapTokenSetupModal, QuickAddTokenModal  
- PdfPreviewModal, CharacterCreationWizard, LevelUpDialog, InventoryManager
- ModuleListView, ModulePlayView

### BaseModal.vue Analysis

**Current purpose:** Specialized D&D content modal with:
- `v-html` rendering of formatted content
- Cross-reference click handling (creature, spell, item, etc.)
- Emits `reference-click` events for navigation

**Decision:** Leave as-is (non-goal to refactor). New `AppModal.vue` will be separate.

## Detailed Design **[REQUIRED]**

### AppModal.vue Component Design

```vue
<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-overlay" @click.self="handleOverlayClick">
      <div class="modal-container" :class="sizeClass">
        <div v-if="$slots.header || title" class="modal-header">
          <slot name="header">
            <h3 class="modal-header__title">{{ title }}</h3>
          </slot>
          <button v-if="closable" class="modal-header__close" @click="close">×</button>
        </div>
        <div class="modal-content" :class="{ 'modal-content--no-padding': noPadding }">
          <slot></slot>
        </div>
        <div v-if="$slots.footer" class="modal-footer">
          <slot name="footer"></slot>
        </div>
      </div>
    </div>
  </Teleport>
</template>
```

**Props:**
- `visible: boolean` - Show/hide modal
- `title?: string` - Header title (optional if using header slot)
- `size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'` - Modal width
- `closable?: boolean` - Show close button (default: true)
- `closeOnOverlay?: boolean` - Close on overlay click (default: true)
- `noPadding?: boolean` - Remove body padding

**Emits:**
- `close` - When modal should close
- `update:visible` - For v-model support

**Features:**
- Teleport to body (proper stacking)
- Focus trap (accessibility)
- Escape key handling
- Prevent body scroll when open

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

### 1. Use Vue UI Library (Vuetify, PrimeVue, etc.)
**Rejected:** Would require significant refactoring, adds large dependency, conflicts with existing custom design system.

### 2. Keep Both CSS Files, Fix Naming Conflicts
**Rejected:** Doesn't address root cause. Still have duplicate class definitions and confusing which to use.

### 3. Extend BaseModal.vue to Be General-Purpose
**Rejected:** BaseModal has specialized D&D content logic (v-html, reference click handling). Making it general-purpose would complicate it. Better to have two separate components with clear purposes.

### 4. Migrate All Modals at Once (Big Bang)
**Rejected:** High risk, hard to test. Incremental migration allows validation at each step.

**Chosen approach:** Create new `AppModal.vue`, keep `base-modal.css` as canonical styles, migrate incrementally.

## Implementation Plan **[REQUIRED]**

### Phase 1: CSS Consolidation
1. Audit `modals.css` for any unique styles not in `base-modal.css`
2. Merge necessary styles (`.modal-actions`, form enhancements) into `base-modal.css`
3. Update class references: `.modal-content` → `.modal-container` where needed
4. Remove `modals.css` import from `main.css`
5. Delete `modals.css`
6. Verify no visual regressions

### Phase 2: Create AppModal Component
1. Create `AppModal.vue` in `components/shared/`
2. Implement props: visible, title, size, closable, closeOnOverlay, noPadding
3. Implement slots: header, default, footer
4. Add keyboard handling (Escape to close)
5. Add focus trap for accessibility
6. Add body scroll lock when modal open
7. Use `<Teleport to="body">` for proper z-index stacking

### Phase 3: Migrate Modals (incremental by complexity)

**Batch 1 - Simple modals (low risk):**
- CreateModuleModal
- ModuleListView (create modal)

**Batch 2 - Standard modals:**
- MapUploadModal
- MapGridConfigModal
- QuickAddTokenModal
- MapTokenSetupModal

**Batch 3 - Management modals (nested delete dialogs):**
- BookManagementModal
- CampaignManagementModal
- PlayerManager
- ChatSidebar

**Batch 4 - Large/complex modals:**
- PdfPreviewModal
- CharacterCreationWizard
- LevelUpDialog
- InventoryManager

**Batch 5 - D&D content modals (may need BaseModal):**
- ReaderView
- ModulePlayView

### Migration Checklist Per Component
- [ ] Replace inline modal template with `<AppModal>`
- [ ] Remove scoped `.modal-overlay` CSS
- [ ] Update container class to `.modal-container` variant
- [ ] Test open/close behavior
- [ ] Test overlay click
- [ ] Test escape key
- [ ] Verify styling matches original