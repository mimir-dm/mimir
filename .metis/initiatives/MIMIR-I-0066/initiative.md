---
id: theme-system-maturation
level: initiative
title: "Theme System Maturation"
short_code: "MIMIR-I-0066"
created_at: 2026-05-08T13:12:30.669983+00:00
updated_at: 2026-05-08T13:12:30.669983+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: theme-system-maturation
---

# Theme System Maturation Initiative

## Context

Three themes ship — light, dark, hyper — with strong identity in their image assets (per-theme `mimir.png` and `gear.png`) but inconsistent execution in CSS:

- **Hyper is mostly a darker dark theme.** Its neon-glow promise lives in three component overrides at the bottom of `themes/hyper.css:101-122` (`.btn-primary`, `.action-card`, `.hero-image`). Most of the rest of the app looks like dark theme even when Hyper is active.
- **Light theme primary is loud.** `--color-primary-500: #7c3aed` is a saturated purple. `CampaignSelector.vue:243` paints a giant lavender halo when the dropdown opens (`box-shadow: 0 0 0 3px var(--color-primary-100)`).
- **Dark theme inverts the primary ramp.** `--color-primary-500: #93bbfd` (very light blue) is barely a step from `--color-text: #f1f5f9`. Primary buttons don't pop, links don't read as accent. The actual accent token (`--color-accent-blue: #3b82f6`) only wires up in a couple of places.
- **Hyper contrast hurts.** Body `#000000` + text `#fafafa` is 21:1 — technically AAA, painful for long sessions.
- **Cinzel is inconsistent.** `main.css:162-171` declares Cinzel for `h1, h2, h3, .font-display, .hero-title, .page-title, .section-title, .modal-title`. But scoped CSS rules like `WorldTab.vue:357` (`.maps-header h4 { font-weight: 600; ... }`) and `ModulesTab.vue:1169` (`.module-title h2 { font-size: 1.25rem; font-weight: 600 }`) implicitly drop `font-family` because more specific scoped rules don't inherit. Some H3s render Cinzel, most don't.
- **Body uses a generic system stack.** Pairing Cinzel with `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto…` (`main.css:48`) flattens the brand voice the rest of the app reaches for.

## Goals & Non-Goals

**Goals:**
- Hyper either commits fully (chromatic borders, scanline/grain backgrounds, glow-on-hover throughout) or sunsets to a "Dark — Neon" accent toggle on dark theme.
- Light theme primary tuned: focus halos no longer dominate the campaign selector.
- Dark theme primary ramp restructured so 500 is mid-tone accent, OR the existing tokens renamed to reflect their actual semantic role (foreground vs. accent).
- Cinzel applied consistently across all heading levels — no scoped `font-weight` rules drop it.
- Body face replaced with a deliberate display+body pair.
- Color ramps complete (`-100` / `-300` / `-700` / `-900`) in every theme so badges/messages render correctly (depends on bug MIMIR-T-0641).

**Non-Goals:**
- Adding more themes.
- Per-feature theming (e.g., custom colors per campaign).
- Component-level visual changes outside theming.

## Detailed Design

- **Hyper decision.** Stakeholder check-in early: keep with full commitment, or sunset to a neon-accent toggle. *Keep* requires:
  - Body texture (subtle scanlines or grain via SVG pattern + `background-blend-mode`).
  - Animated chromatic accent borders on focus (`background: linear-gradient(...)` + `mask` + keyframes).
  - Neon-shadow on cards and active items.
  - Maybe a different display weight or letter-spacing for headers in Hyper specifically.
- **Light primary tuning.** Reduce focus-ring lightness — `box-shadow: 0 0 0 3px var(--color-primary-100)` becomes `0 0 0 3px color-mix(in srgb, var(--color-primary-500) 25%, transparent)`. Or define a dedicated `--color-focus-ring` token per theme.
- **Dark primary restructuring.** Two options to evaluate:
  - **Re-key the ramp** so `--color-primary-500` is the conventional mid-tone accent (e.g., `#3b82f6`), and `--color-primary-50/-100` etc. radiate out from there. Existing usages get the conventional behavior automatically.
  - **Rename existing tokens** to reflect that they're already inverted. `--color-primary-*` becomes `--color-foreground-*` (since those values are foreground-leaning), and `--color-accent-*` is wired into all action elements.
  - First option is simpler and lower-blast-radius. Default to it.
- **Hyper contrast.** Raise body from `#000000` to `#080812`; keep `--color-surface: #0a0a0f`. Add a subtle vertical scanline gradient or noise texture.
- **Cinzel consistency.** Audit every scoped `font-family`, `font-weight` rule that touches headings; remove anything that drops the cascade. Or set `font-family: var(--font-display)` explicitly on `:where(h1, h2, h3, h4, h5, h6)` in global CSS so scoped rules can't accidentally drop it without explicit intent.
- **Body face.** Pick from **Inter** (clean, modern), **Source Sans 3** (warmer), or **EB Garamond** (manuscript, leans into D&D). User selects during decompose. Provided as a question with previews.
- **Color ramp completeness.** Define `--color-error-100/-300/-700/-900`, `--color-success-*`, `--color-warning-*`, `--color-info-*` in `light.css`, `dark.css`, `hyper.css`. Reuse Tailwind's standard ramps as a starting point, adjust for theme polarity. (Bug task MIMIR-T-0641 covers the immediate rendering breakage; this initiative ensures it stays solved.)

## UI/UX Design

Three reference cards (one per theme) demonstrating: button (primary/secondary/ghost), link, focus state, error message, success badge, warning badge, input field at rest and focused. Used as before/after artifacts and as a manual regression sheet.

## Alternatives Considered

- **Drop to two themes (light + dark).** Considered. Rejected because Hyper has user investment expressed in the per-theme image assets (skull, gear) — give it a real chance before sunsetting.
- **Keep generic body stack.** Rejected because pairing Cinzel with `-apple-system` flattens the brand voice the rest of the app is reaching for.
- **Use a CSS-in-JS theme system.** Rejected — current variable system is fine; the problem is callsite hygiene (already covered by UI Cohesion Pass) and ramp completeness (this initiative). No need to swap the foundation.

## Implementation Plan

- **Phase 1 — Hyper decision.** Stakeholder check-in. One AskUserQuestion at decompose-time, then either Phase 5a (commit) or Phase 5b (sunset) below.
- **Phase 2 — Color-ramp completeness.** Define missing `-100/-300/-700/-900` ramps across all three themes. (MIMIR-T-0641 covers the urgent subset; this generalizes to all semantic colors.)
- **Phase 3 — Light/dark primary tuning.** Light: tune focus halo. Dark: re-key primary ramp.
- **Phase 4 — Typography.** Pick body face, install via `@import` (or self-host), apply Cinzel cascade fix to global CSS, audit scoped overrides.
- **Phase 5a — Hyper commitment.** Body texture + chromatic borders + glow-on-hover + display weight tweaks.
- **Phase 5b — Hyper sunset.** Move neon accents into a `dark` variant toggle, retire `themes/hyper.css`.