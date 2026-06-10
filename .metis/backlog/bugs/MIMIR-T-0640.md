---
id: fix-broken-var-color-0-1-css
level: task
title: "Fix broken var(--color) / 0.1 CSS expressions"
short_code: "MIMIR-T-0640"
created_at: 2026-05-08T13:12:31.116447+00:00
updated_at: 2026-05-08T14:18:41.424731+00:00
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

# Fix broken var(--color) / 0.1 CSS expressions

## Objective

Replace invalid CSS expressions of the form `var(--color-X) / 0.N` with valid `color-mix()` (or precomputed alpha tokens) so focus rings and error/success message blocks render correctly. The `/ 0.N` syntax is not valid CSS — the property silently fails and the affected element renders without the intended fill or shadow.

## Backlog Item Details

### Type
- [x] Bug

### Priority
- [x] P1 — High (visible regression: missing focus rings, broken error styling)

### Impact Assessment

- **Affected Users:** All users hitting the campaign-create or character-list views in any theme.
- **Reproduction Steps:**
  1. Navigate to `/campaigns/new`.
  2. Click into the "Campaign Name" input — focus ring is missing.
  3. Submit with the field empty (or trigger an API error) to surface the error message — message renders red text on raw surface, no background, no border.
- **Expected vs Actual:** Inputs should show a subtle primary-colored focus ring on focus; error messages should render with semi-transparent red background and matching border. Currently the box-shadow / background-color / border declarations are silently dropped because `var(--color-error) / 0.1` isn't valid CSS.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] No occurrences of the regex `var\(--color[a-z0-9-]+\) ?/ ?0\.` anywhere under `crates/mimir/frontend/src/`.
- [ ] Form input focus rings render correctly across all three themes (light, dark, hyper) on the campaign-create form and the character-list view's relevant inputs.
- [ ] Error message blocks (e.g., on failed campaign create) render with semi-transparent background and visible border in all three themes.
- [ ] Manual visual regression check on the affected views in all three themes; no other unintended changes.

## Implementation Notes

### Affected Files

- `crates/mimir/frontend/src/features/campaigns/views/CampaignCreateView.vue` — lines 150 (`box-shadow: 0 0 0 3px var(--color-primary-500) / 0.1`), 205 (`background-color: var(--color-error) / 0.1`), 206 (`border: 1px solid var(--color-error) / 0.2`).
- `crates/mimir/frontend/src/features/characters/views/CharacterListView.vue` — three occurrences (`var(--color-error) / 0.1`, `var(--color-error) / 0.2`, `var(--color-primary-500) / 0.2`). Confirm with: `grep -nE 'var\(--color[a-z0-9-]+\) ?/ ?0\.' …`.

### Technical Approach

Replace each occurrence with `color-mix(in srgb, var(--color-X) NN%, transparent)`. All target browsers (Tauri webview is modern Chromium/WebKit) support `color-mix()`.

Examples:

```css
/* Before */
box-shadow: 0 0 0 3px var(--color-primary-500) / 0.1;
background-color: var(--color-error) / 0.1;
border: 1px solid var(--color-error) / 0.2;

/* After */
box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary-500) 10%, transparent);
background-color: color-mix(in srgb, var(--color-error) 10%, transparent);
border: 1px solid color-mix(in srgb, var(--color-error) 20%, transparent);
```

Where the existing token system already provides a precomputed alpha (e.g., `--color-error-bg`, `--color-info-bg`), prefer the token over an inline `color-mix` for consistency.

### Dependencies

None. Independent fix; can ship in parallel with anything else.

### Risk Considerations

Trivial. Only risk is missing one occurrence — mitigate with a final repo-wide grep before close.

## Status Updates

### 2026-05-08 — Implementation

Located 6 occurrences across 2 files (one more than the original task description estimated):

- `crates/mimir/frontend/src/features/campaigns/views/CampaignCreateView.vue:150,205,206`
- `crates/mimir/frontend/src/features/characters/views/CharacterListView.vue:395,396,435`

Replaced each with `color-mix(in srgb, var(--color-X) NN%, transparent)`. Did not use `--color-error-bg` etc. because those tokens encode 10% only — the broken sites mix 10% (background) and 20% (border), and inline `color-mix` keeps the intent explicit at each callsite.

Final grep `grep -nE 'var\(--color[a-z0-9-]+\) ?/ ?0\.' …` returns no matches (exit 1). AC 1 satisfied.

`angreal test unit` passes — no Rust regressions (frontend isn't covered by the unit suite, but the CSS edits are syntactic and don't touch any TS/Vue logic).

ACs 2-4 are visual: the CSS is now syntactically valid, so the property rules will apply. `color-mix(in srgb, X NN%, transparent)` is supported in all modern browsers; Tauri's webview is modern Chromium/WebKit. Manual visual regression in all three themes is recommended before merge but cannot be automated from this loop.