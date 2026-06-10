---
id: remove-duplicate-reseed-button
level: task
title: "Remove duplicate Reseed button from AppHeader"
short_code: "MIMIR-T-0642"
created_at: 2026-05-08T13:12:31.579150+00:00
updated_at: 2026-05-08T13:12:31.579150+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Remove duplicate Reseed button from AppHeader

## Objective

Remove the Reseed dev-only button (and its confirmation modal, state, and handlers) from `AppHeader.vue`. The same functionality already lives in `SettingsView.vue` → Dev Tools, with a richer interface (status indicator, seed/reseed/clear actions, message feedback). The header version is a duplicate that hardens production chrome around a dev-only affordance.

## Backlog Item Details

### Type
- [x] Tech Debt

### Priority
- [x] P3 — Low (no functional impact; cleanup)

### Technical Debt Impact

- **Current problems:** Duplicate code path for the same operation. AppHeader queries `is_dev_mode` on mount even in production builds, and reserves layout space for a button that ~all users never see. Two confirmation modals exist for the same destructive action — the SettingsView version has more context (current seed status, prerequisites, messages) and is the one to keep.
- **Benefits of fixing:** Less code in the global header (`AppHeader.vue` drops ~80 lines including modal markup, state, handlers, imports). Production chrome stops conditionally branching on dev mode at all. Single source of truth for the reseed flow.
- **Risk assessment:** Trivial. The Settings path already exists and is functional. Pure deletion.

## Acceptance Criteria

- [ ] `AppHeader.vue` no longer queries `is_dev_mode` (the `onMounted` block can be removed).
- [ ] `AppHeader.vue` no longer renders a Reseed button or its confirmation `AppModal`.
- [ ] `AppHeader.vue` state for `isDevMode`, `isReseeding`, `showConfirmModal`, `reseedError` is removed.
- [ ] `AppHeader.vue` handlers `showReseedConfirm`, `cancelReseed`, `confirmReseed` are removed.
- [ ] Imports unused after removal are cleaned up (`AppModal` if no other modal in the header, `invoke`, `ApiResponse`).
- [ ] `SettingsView.vue` Dev Tools section continues to function in dev build.
- [ ] No console errors in non-dev build (header should not call backend dev-mode probe).
- [ ] Visual check: header in non-dev build looks identical (the dev button was hidden anyway, but verify spacing).

## Implementation Notes

### Affected Files

- `crates/mimir/frontend/src/app/AppHeader.vue` — primary deletion target. Lines roughly: template ~30-43 (button) and ~52-73 (modal); script onMounted ~101-108, state ~96-99, handlers ~111-141.

### Technical Approach

Single-file deletion. After removal:

```vue
<!-- header right slot -->
<div class="header-right">
  <router-link to="/settings" class="settings-icon" title="Settings">
    <img :src="gearIcon" alt="Settings" class="gear-icon" />
  </router-link>
</div>
```

Drop the `dev-button` styles in the scoped `<style>` block (`.dev-button`, `.dev-button:hover`, `.dev-button:active`, `.dev-button:disabled`, `.dev-button svg`). Keep `.warning-text`, `.error-message`, `.modal-actions`, `.btn`, `.btn-secondary`, `.btn-warning` — they may be referenced elsewhere; verify with grep before deleting.

### Dependencies

None. Independent.

### Risk Considerations

- A user with the dev button in muscle memory will need to navigate to Settings → Dev Tools instead. Acceptable — they're devs, and Settings is one click away.

## Status Updates

*To be added during implementation.*