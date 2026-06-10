---
id: ui-cohesion-pass
level: initiative
title: "UI Cohesion Pass"
short_code: "MIMIR-I-0064"
created_at: 2026-05-08T13:12:27.830010+00:00
updated_at: 2026-05-08T13:12:27.830010+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: ui-cohesion-pass
---

# UI Cohesion Pass Initiative

## Context

The Mimir frontend has accumulated cross-cutting inconsistencies that no individual screen authored — they emerge from independent components solving the same problem differently:

- **Three icon idioms** in one app: ASCII placeholders in `DashboardTabs.vue:39-49` (`~ + @ @@ * >`), one emoji escapee (`📄` in `WorldTab.vue:89`), and ~78 inline `viewBox` SVG occurrences across 20+ files where the same trash/edit/plus paths are duplicated raw.
- **26 native `confirm()` / `alert()` calls** across `WorldTab.vue`, `ModulesTab.vue`, `DocumentEditor.vue`, `BookManagementModal.vue`, `MapTokenSetupModal.vue`, etc., living alongside a polished `AppModal` with focus management, scroll lock, size variants, and Esc/overlay handling.
- **231 hard-coded hex fallbacks** (`var(--color-text, #888)`) across 42 files. Every fallback is a place where the developer didn't trust the token. The fallbacks mask broken token wiring (see also bug MIMIR-T-0641).
- **~60 per-component `.theme-dark` overrides** in scoped CSS that the variable system was specifically meant to eliminate. Each is a place where a token difference lives in the wrong layer.
- **`EmptyState.vue` exists** with six clean SVG variants but is used in only ~15 files; ~20 other files re-roll their own empty-icon divs with ad-hoc text.

None of this individually blocks usage, but the cumulative drift makes the app feel stitched together. This initiative collapses the drift without changing functionality.

## Goals & Non-Goals

**Goals:**
- Single icon component / sprite system used everywhere; ASCII placeholders and the emoji escapee retired.
- Every empty-state surface routes through `EmptyState.vue` with a typed variant, title, description, and action slot.
- Zero remaining native `confirm()` / `alert()` calls — all flows use `AppModal`.
- Per-component `.theme-dark` overrides reduced to a small, justified residue (target <10).
- Hard-coded hex fallbacks (`var(--color-x, #hex)`) eliminated; tokens guaranteed defined in all themes.

**Non-Goals:**
- Restructuring color ramps or adding themes (see Theme System Maturation).
- Information-architecture changes (see IA & Onboarding).
- Adding new functionality.

## Detailed Design

- **Icon system**: pick lucide-vue-next or scaffold an `<Icon name="..." />` component backed by an SVG sprite. Decision deferred to first task; preference for lucide-vue-next given it ships the icons used everywhere already.
- **DashboardTabs**: replace the ASCII map with proper SVG icons matching the `EmptyState` variant set (globe, folder, users, user, flask).
- **EmptyState migration**: every `<div class="empty-icon">…</div>` + heading combo becomes `<EmptyState variant="..." title="..." description="..."><template #action>…</template></EmptyState>`. Targets: `WorldTab`, `ModulesTab`, `NPCsTab`, `PCsTab`, plus any others surfaced by the audit.
- **Native dialog sweep**: each `confirm(`/`alert(` callsite (26 instances, 9 files) converts to `AppModal`-driven flow. The delete-confirmation pattern in `ModulesTab.vue:206-218` (AppModal + `delete-warning` paragraph + named buttons) is the template.
- **`.theme-dark` override audit**: for each, decide (a) delete because the token already covers it, (b) move the override into `themes/dark.css` as a token redefinition, or (c) keep with a comment explaining why component-local override is necessary.
- **Hex fallback removal**: once token presence is verified, `var(--color-x, #hex)` becomes `var(--color-x)`. Fallbacks are noise.

## UI/UX Design

This initiative produces no intentional visual change. Users should perceive "feels more polished" without being able to point to anything specific. Regression risk lives in the edge cases — a `.theme-dark` override removed in error, a confirm replaced with a modal that's missing a button. Validation is visual diff per surface across all three themes.

## Alternatives Considered

- **Do nothing.** Tolerable in the sense that no individual issue blocks usage. Rejected because the drift produces real bugs (broken color expressions in `CampaignCreateView` ship to users today; see MIMIR-T-0640) and slows future work — every new component has to choose between conventions.
- **Big-bang design system rewrite.** Too disruptive, diminishing returns. The token system is fine; the problem is callsite hygiene. Rejected.

## Implementation Plan

- **Phase 1 — Icon system foundation.** Pick library, scaffold `<Icon>` with the 6–8 most-used icons (plus, trash, edit, x, chevron-up/down, ellipsis). One PR.
- **Phase 2 — High-traffic icon migration.** `DashboardTabs`, `AppHeader`, `AppModal` close, `ModuleList`, `DocumentSidebar`, `ModuleDocumentsPanel`. One PR per file group.
- **Phase 3 — `EmptyState` sweep.** Migrate all five dashboard tabs + `WorldTab` + `ModulesTab` empty states. One PR.
- **Phase 4 — `confirm()` / `alert()` to `AppModal`.** Sweep the 9 files; one PR per high-traffic file, batched for the lower-traffic ones.
- **Phase 5 — `.theme-dark` override audit.** Spreadsheet the overrides, decide disposition, execute. One PR per disposition class.
- **Phase 6 — Hex fallback cleanup.** Verify token presence in all themes (depends on MIMIR-T-0641), then strip fallbacks. One sweeping PR.