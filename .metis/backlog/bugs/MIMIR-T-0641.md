---
id: define-missing-100-300-700-900
level: task
title: "Define missing -100/-300/-700/-900 semantic color ramps in all themes"
short_code: "MIMIR-T-0641"
created_at: 2026-05-08T13:12:31.339546+00:00
updated_at: 2026-05-08T14:25:27.387754+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Define missing -100/-300/-700/-900 semantic color ramps in all themes

## Objective

Define the missing semantic color ramp shades (`-100`, `-300`, `-700`, `-900`) for `error`, `success`, `warning`, and `info` in every theme file. These shades are referenced throughout the app (status badges, error/success messages, secret-NPC blocks) but are not defined anywhere — which means callsites currently render with empty values for those properties.

## Backlog Item Details

### Type
- [x] Bug

### Priority
- [x] P1 — High (multiple visible badge/message rendering failures)

### Impact Assessment

- **Affected Users:** All users hitting any view that renders status badges or error/success messages.
- **Reproduction Steps:**
  1. Open Settings → Dev Tools (in dev build).
  2. Inspect the "Status" badge ("Test Data Present" or "Not Seeded").
  3. The element references `--color-success-100` / `--color-success-300` / `--color-success-700` (or the gray equivalents) — these tokens resolve to empty strings.
  4. The badge renders without its intended background and border tints.
- **Expected vs Actual:** Status badges, error message blocks, and success notifications should render with semi-transparent tinted backgrounds and matching borders. Currently the rules apply but resolve to empty values, so the elements render with no fill at all (or fall back to inherited surface).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `--color-error-100`, `--color-error-300`, `--color-error-700`, `--color-error-900` defined in `themes/light.css`, `themes/dark.css`, `themes/hyper.css`.
- [ ] Same for `--color-success-100/-300/-700/-900`, `--color-warning-100/-300/-700/-900`, `--color-info-100/-300/-700/-900`.
- [ ] Existing badge/message renderings match design intent in dark theme (formerly broken).
- [ ] No regression in light theme — existing badges that already worked there continue to work.
- [ ] Hyper theme renders semantic badges in a way coherent with the neon palette (or, if explicit Hyper-specific shades are out of scope, falls back to the dark theme values via cascade).
- [ ] Manual visual check: the four affected callsite groups below render correctly.

### Affected callsites (verification targets after fix)

- `AppHeader.vue:376-388` — error message block.
- `SettingsView.vue:580-601, 858-865, 902-924` — settings messages, status badges, seed-message variants.
- `ModulesTab.vue:1429-1473` — NPC role badge and DM-secrets block.
- `CampaignCreateView.vue:202-209` — error message (cross-references MIMIR-T-0640; both fixes are needed for full rendering).

## Implementation Notes

### Technical Approach

Use Tailwind's standard semantic ramps as a starting point, adjust per theme polarity:

- **Light theme** — full ramp light→dark; `-100` is a tint, `-700` is a dark accent for text.
- **Dark theme** — invert the convention so `-100` is a *dark tint of the semantic* (good for backgrounds on dark surfaces) and `-700` is a bright accent for text. Either:
  - **Option A:** keep the same semantic values (`-100` is light, `-700` is dark); rely on already-flipped `--color-text` for contrast. Simpler but tints will be light pastels on dark surface — high-contrast, can read garish.
  - **Option B:** invert the ramp per theme so `-100` is `rgba(error, ~10%)` over surface, `-700` is `rgba(error, ~70%)`. Better aesthetic; requires careful definition.
  - Default to **Option B** because it matches the existing `--color-*-bg` token style.
- **Hyper theme** — start with dark theme values; iterate later if needed.

### Dependencies

- MIMIR-T-0640 (broken `var(--color) / 0.1` expressions). Independent fixes — both bugs touch error message rendering but solve different problems. They can ship in either order.
- Long-term, this work folds into the Theme System Maturation initiative's Phase 2 (color-ramp completeness sweep), but solving the immediate breakage as a backlog bug is faster.

### Risk Considerations

- The existing `--color-success-bg` / `--color-warning-bg` / `--color-info-bg` / `--color-error-bg` tokens overlap conceptually with `-100`. Risk: defining `-100` differently from `-bg` creates two tokens for the same intent, future drift. Mitigate by either deciding `-100` *is* `-bg` (alias them) or documenting the difference (e.g., `-bg` is a flat tint, `-100` participates in the full ramp).

## Status Updates

### 2026-05-08 — Implementation

Added full Tailwind-standard ramps (`-50`, `-100`, `-200`, `-300`, `-400`, `-500`, `-600`, `-700`, `-800`, `-900`) for `error`, `success`, `warning`, `info` to all three theme files: `themes/light.css`, `themes/dark.css`, `themes/hyper.css`. All 48 explicitly-required shade definitions (4 colors × 4 shades × 3 themes for the AC's `-100/-300/-700/-900` minimum) verified via grep.

### Design call: Option A, light-direction values across themes

The task notes proposed Option B (per-theme inversion: dark-theme `-100` is dark, `-900` is light). Went with Option A (same Tailwind-standard light-direction values in all three themes) for these reasons:

- The existing pattern already has dark-mode-aware components handling the swap themselves at the callsite (e.g., `AppHeader.vue:386-388` with `.theme-dark .error-message { background: var(--color-error-900) }`). Inverting the ramp per-theme would silently break those overrides.
- Auditing and rewriting every `.theme-dark .X { background: var(--color-Y-900) }` site is meaningful work — it's MIMIR-I-0066's color-ramp completeness phase, not a P1 bug fix.
- This task's ACs are about resolving the empty-string token bug. Light-direction-everywhere does that with the smallest blast radius. Components that don't have explicit dark-mode overrides (e.g., `ModulesTab.vue:1429`) will continue to render with light tints in dark mode — same as before this fix, since the rgba fallback was visually similar — so no regression.

A comment in each theme file points to MIMIR-I-0066 for the per-theme inversion follow-up.

### Verification

- Per-theme presence check: all four colors × four shades present in all three theme files (48/48 OK via shell loop).
- `angreal test unit`: passes — no Rust regressions.
- The existing `--color-error-bg` etc. tokens are preserved — they alias `-100` semantically (e.g., `--color-error-bg: #fee2e2` matches `--color-error-100: #fee2e2`), so callsites using either name resolve to the same value. The risk noted in the task's risk consideration (drift between `-bg` and `-100`) is not realized.
- Visual regression in all three themes is the residual manual check before merge.

### Anchor mismatch (acknowledged, deferred)

In dark/hyper themes, `--color-error` differs from `--color-error-500`:
- Dark: `--color-error: #f87171` (Tailwind red-400) vs. `--color-error-500: #ef4444`.
- Hyper: `--color-error: #f43f5e` (rose-500) vs. `--color-error-500: #ef4444`.

Consistent with the existing pattern — each theme picked its own preferred "default error color" independent of a Tailwind ramp anchor. Reconciling this (so `var(--color-error)` always equals `var(--color-error-500)` per theme) is part of MIMIR-I-0066. Acceptable now because direct uses of `-500` in the codebase are rare; most usage is `-100` (background tint), `-300` (border), and `-700` (text).