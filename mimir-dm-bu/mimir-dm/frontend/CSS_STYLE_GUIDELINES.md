# CSS Style Guidelines - Mimir DM

## Overview

This document establishes clear guidelines for CSS architecture in the Mimir DM application to maintain consistency, improve maintainability, and reduce code duplication.

## CSS Architecture

### File Structure
```
src/assets/styles/
├── main.css                    # Main entry point
├── themes/                     # Theme-specific variables
│   ├── light.css
│   ├── dark.css
│   └── hyper.css
├── utilities/                  # Utility classes
│   └── animations.css
├── components/                 # Component-specific styles
│   ├── catalog-tables.css      # Consolidated table styles
│   ├── base-modal.css          # Consolidated modal styles  
│   ├── form-inputs.css         # Consolidated form styles
│   ├── buttons.css             # Button system
│   └── ...existing files
└── layouts/                    # Layout-specific styles
```

## Scoped vs Global CSS Rules

### ✅ **Use Global CSS Classes When:**

1. **Reusable UI Patterns** (3+ components using similar styling)
   - Buttons, form inputs, modals, tables
   - Layout containers (flex, grid patterns)
   - Status indicators, badges, tooltips

2. **Theme-Aware Styling**
   - Any styling that changes between light/dark/hyper themes
   - Colors, shadows, borders that need theme support

3. **Consistent Design System Elements**
   - Typography scales, spacing systems
   - Animation patterns, transitions
   - Interactive states (hover, focus, active)

**Examples:**
```css
/* ✅ Good - Use global classes */
<button class="btn btn-primary">Save</button>
<div class="catalog-table">...</div>
<input class="form-input" />
```

### ✅ **Use Scoped CSS When:**

1. **Component-Specific Layout**
   - Unique internal component structure
   - Component-specific positioning/sizing
   - Custom component animations

2. **Third-Party Integration Styling**
   - Styling for external library components
   - Overrides for third-party CSS

3. **Complex Component Logic**
   - Conditional styling based on component state
   - Dynamic styling that can't be handled by classes

**Examples:**
```vue
<style scoped>
/* ✅ Good - Scoped for component-specific layout */
.editor-container {
  display: grid;
  grid-template-rows: auto 1fr auto;
  height: 100vh;
}

/* ✅ Good - Third-party override */
.tiptap-editor .ProseMirror {
  outline: none;
  padding: 1rem;
}
</style>
```

### ❌ **Avoid Scoped CSS For:**

1. **Basic Button Styling**
   ```vue
   <!-- ❌ Bad -->
   <style scoped>
   .my-button {
     padding: 8px 16px;
     background: #3b82f6;
     color: white;
     border-radius: 4px;
   }
   </style>
   
   <!-- ✅ Good -->
   <template>
     <button class="btn btn-primary">Click me</button>
   </template>
   ```

2. **Theme-Dependent Colors**
   ```vue
   <!-- ❌ Bad -->
   <style scoped>
   .card {
     background: #ffffff;
     border: 1px solid #e5e7eb;
   }
   
   .theme-dark .card {
     background: #1e293b;
     border-color: #334155;
   }
   </style>
   
   <!-- ✅ Good -->
   <template>
     <div class="card">
       <div class="bg-surface border-default">...</div>
     </div>
   </template>
   ```

3. **Form Input Styling**
   ```vue
   <!-- ❌ Bad -->
   <style scoped>
   .search-input {
     padding: 8px 12px;
     border: 1px solid #d1d5db;
     border-radius: 4px;
   }
   
   .search-input:focus {
     border-color: #3b82f6;
     box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
   }
   </style>
   
   <!-- ✅ Good -->
   <template>
     <input class="form-input" type="search" />
   </template>
   ```

## Consolidated Class Usage

### Catalog Tables
Use `catalog-table` system for all data tables:

```vue
<template>
  <div class="catalog-table">
    <div class="catalog-table__header">
      <h2 class="catalog-table__title">Spells</h2>
      <div class="catalog-table__filters">
        <div class="catalog-table__search">
          <input type="search" placeholder="Search spells..." />
        </div>
        <div class="catalog-table__filter-group">
          <label class="catalog-table__filter-label">Level:</label>
          <!-- Use btn-filter for filter buttons -->
          <button class="btn-filter" :class="{ 'btn-filter--active': isActive }">
            All Levels
            <span v-if="count" class="btn-filter__count">{{ count }}</span>
          </button>
        </div>
      </div>
    </div>
    
    <div class="catalog-table__content">
      <div class="catalog-table__results-info">
        <span class="catalog-table__result-count">{{ count }} results</span>
      </div>
      
      <div class="catalog-table__scroll-container">
        <table class="catalog-table__table">
          <thead>
            <tr>
              <th>
                <div class="catalog-table__sort-header" 
                     :class="{ 'catalog-table__sort-header--active': sortActive }">
                  Name
                  <span class="catalog-table__sort-icon">↓</span>
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr class="catalog-table__row" 
                :class="{ 'catalog-table__row--selected': isSelected }">
              <td>
                <div class="catalog-table__cell-name">Fireball</div>
                <div class="catalog-table__cell-secondary">3rd level evocation</div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    
    <div class="catalog-table__pagination">
      <div class="catalog-table__pagination-info">Showing 1-50 of 200</div>
      <div class="catalog-table__pagination-controls">
        <button class="btn-pagination" :disabled="!hasPrev">Previous</button>
        <button class="btn-pagination btn-pagination--current">1</button>
        <button class="btn-pagination">2</button>
        <button class="btn-pagination" :disabled="!hasNext">Next</button>
      </div>
    </div>
  </div>
</template>
```

### Modal Components
Use `modal-*` system for all modal dialogs:

```vue
<template>
  <div class="modal-overlay" @click="closeOnOverlay">
    <div class="modal-container modal-container--medium" @click.stop>
      <div class="modal-header">
        <h2 class="modal-header__title">{{ title }}</h2>
        <button class="btn-close" @click="close">×</button>
      </div>
      
      <div class="modal-content">
        <form class="modal-form">
          <div class="form-group">
            <label class="form-label form-label--required">Name</label>
            <input class="form-input" v-model="name" required />
            <div class="form-help">Enter a unique name for this item</div>
          </div>
        </form>
      </div>
      
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="close">Cancel</button>
        <button class="btn btn-primary" @click="save">Save</button>
      </div>
    </div>
  </div>
</template>
```

### Form Components
Use `form-*` system for all form elements:

```vue
<template>
  <form class="modal-form">
    <!-- Basic input -->
    <div class="form-group">
      <label class="form-label form-label--required">Email</label>
      <input class="form-input" type="email" v-model="email" />
      <div class="form-error" v-if="errors.email">{{ errors.email }}</div>
    </div>
    
    <!-- Select dropdown -->
    <div class="form-group">
      <label class="form-label">Status</label>
      <select class="form-select" v-model="status">
        <option value="">Choose status...</option>
        <option value="active">Active</option>
        <option value="inactive">Inactive</option>
      </select>
    </div>
    
    <!-- Custom multi-select -->
    <div class="form-group">
      <label class="form-label">Tags</label>
      <div class="form-select-custom">
        <div class="form-select-custom__trigger" 
             :class="{ 'form-select-custom__trigger--active': isOpen }">
          Select tags...
          <span class="form-select-custom__chevron">▼</span>
        </div>
        <div class="form-select-custom__dropdown" v-if="isOpen">
          <div class="form-select-custom__search">
            <input type="search" placeholder="Search tags..." />
          </div>
          <div class="form-select-custom__options">
            <label class="form-select-custom__option">
              <input type="checkbox" />
              <span>Important</span>
            </label>
          </div>
          <div class="form-select-custom__actions">
            <button class="form-select-custom__action-btn">Clear All</button>
            <button class="form-select-custom__action-btn">Select All</button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Checkbox -->
    <label class="form-checkbox">
      <input type="checkbox" class="form-checkbox__input" v-model="agree" />
      <span class="form-checkbox__box"></span>
      <span class="form-checkbox__label">I agree to the terms</span>
    </label>
  </form>
</template>
```

### Button System
Use the extended button system for all interactive elements:

```vue
<template>
  <div>
    <!-- Primary actions -->
    <button class="btn btn-primary">Save Changes</button>
    
    <!-- Filter buttons -->
    <button class="btn-filter" :class="{ 'btn-filter--active': isActive }">
      All Items
      <span class="btn-filter__count">{{ count }}</span>
    </button>
    
    <!-- Toolbar buttons (editors) -->
    <div class="editor-toolbar">
      <button class="btn-toolbar" :class="{ 'btn-toolbar--active': isBold }">
        <strong>B</strong>
      </button>
      <div class="btn-toolbar-divider"></div>
      <button class="btn-toolbar">Save</button>
    </div>
    
    <!-- Status selects -->
    <select class="btn-status-select btn-status-select--ready">
      <option value="ready">Ready</option>
    </select>
    
    <!-- Tab navigation -->
    <div class="tab-group">
      <button class="btn-tab btn-tab--active">Overview</button>
      <button class="btn-tab">Details</button>
    </div>
    
    <!-- Close buttons -->
    <button class="btn-close" @click="close">×</button>
    
    <!-- Pagination -->
    <div class="pagination">
      <button class="btn-pagination" :disabled="!hasPrev">Previous</button>
      <button class="btn-pagination btn-pagination--current">1</button>
      <button class="btn-pagination">Next</button>
    </div>
  </div>
</template>
```

## CSS Variable Usage

### ✅ **Always Use CSS Variables For:**
- Colors: `var(--color-primary-500)`, `var(--color-text)`  
- Spacing: `var(--spacing-sm)`, `var(--spacing-lg)`
- Border radius: `var(--radius-sm)`, `var(--radius-md)`
- Shadows: `var(--shadow)`, `var(--shadow-lg)`
- Transitions: `var(--transition-base)`

### ❌ **Never Use Hard-coded Values For:**
- Colors: `#3b82f6`, `rgb(59, 130, 246)`
- Spacing: `8px`, `1rem`
- Theme-dependent values

## Component Refactoring Checklist

When refactoring a component to use consolidated styles:

### 1. **Identify Patterns**
- [ ] Look for button-like elements → use `btn-*` classes
- [ ] Look for form inputs → use `form-*` classes  
- [ ] Look for table structures → use `catalog-table-*` classes
- [ ] Look for modal patterns → use `modal-*` classes

### 2. **Replace Scoped Styles**
- [ ] Remove scoped CSS that duplicates global patterns
- [ ] Replace hard-coded colors with CSS variables
- [ ] Replace custom spacing with spacing variables

### 3. **Test Theme Consistency**
- [ ] Verify component works in light theme
- [ ] Verify component works in dark theme
- [ ] Verify component works in hyper theme
- [ ] Check hover/focus/active states

### 4. **Update Documentation**
- [ ] Update component props if needed
- [ ] Add usage examples to component documentation
- [ ] Update any related stories/tests

## Migration Strategy

### Phase 1: High-Impact Components ✅ **COMPLETED**
- [x] Catalog table components (SpellTable, ItemTable, etc.) 
- [x] Modal components (BaseModal, confirmation dialogs)
- [x] Button system extensions
- [x] Form input consolidation

### Phase 2: Medium-Impact Components
- [ ] DocumentEditor toolbar refactoring
- [ ] MultiSelectFilter component
- [ ] Chat message components
- [ ] Sidebar components

### Phase 3: Low-Impact Components
- [ ] Remaining scoped style cleanup
- [ ] Legacy component updates
- [ ] Performance optimizations

## Enforcement

### Code Review Checklist
- [ ] No hardcoded colors (use CSS variables)
- [ ] No duplicate button/form/table styling in scoped CSS
- [ ] Proper use of consolidated class systems
- [ ] Theme support for all UI elements
- [ ] Consistent spacing using spacing variables

### Automated Checks (Future)
- ESLint rules for hardcoded colors
- CSS analysis for duplicate patterns
- Theme consistency testing

## Examples & Resources

### Quick Reference Card

| Pattern | Use This Class | Instead of Scoped CSS |
|---------|---------------|-----------------------|
| Primary button | `btn btn-primary` | Custom button styles |
| Filter button | `btn-filter` | Custom filter styling |
| Text input | `form-input` | Custom input styles |
| Modal | `modal-container` | Custom modal CSS |
| Data table | `catalog-table` | Custom table styles |
| Status badge | `catalog-table__cell-badge` | Custom badge styles |

### Before/After Examples

**❌ Before:**
```vue
<style scoped>
.my-button {
  padding: 8px 16px;
  background: #3b82f6;
  color: white;
  border: 1px solid #3b82f6;
  border-radius: 6px;
  cursor: pointer;
}

.my-button:hover {
  background: #2563eb;
}

.theme-dark .my-button {
  background: #1e40af;
}
</style>
```

**✅ After:**
```vue
<template>
  <button class="btn btn-primary">Save</button>
</template>
```

This reduces ~15 lines of CSS to 1 line of HTML and ensures automatic theme support.

---

**Last Updated:** Phase 1 Implementation - January 2025
**Next Review:** After Phase 2 completion