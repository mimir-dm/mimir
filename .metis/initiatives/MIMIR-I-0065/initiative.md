---
id: information-architecture-onboarding
level: initiative
title: "Information Architecture & Onboarding"
short_code: "MIMIR-I-0065"
created_at: 2026-05-08T13:12:30.448404+00:00
updated_at: 2026-05-08T13:12:30.448404+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: information-architecture-onboarding
---

# Information Architecture & Onboarding Initiative

## Context

The way users move through Mimir costs flow at every step:

- **Home is inert.** `HomeView.vue` is the strongest aesthetic moment in the app — floating skull, ambient pulse, Cinzel title. But it has zero CTAs. New users have to discover the campaign-selector dropdown in the header. Returning users have to click into it to "continue."
- **Campaign Dashboard chrome is two horizontal bars too tall.** `AppHeader` (72px) → campaign-header strip in `CampaignDashboardView.vue:19-35` (~52px) → `DashboardTabs` (~44px) = ~168px before any content. The campaign name and tab nav could share one row.
- **Module Dashboard sections look identical.** Documents, NPCs, Maps, Dangers all use the same `--color-surface` background and the same H3. Eyes have nowhere to land. The `is-npc` left-border pattern in `CharacterCard.vue:84` already exists but isn't applied here.
- **Play sits next to Delete in the module header** (`ModulesTab.vue:51-60`). Same color weight, same size. Play is the culminating action; Delete is destructive. One misclick away.
- **Reorder chevrons hide on hover** in `ModuleList.vue:147` and `DocumentSidebar.vue` (`opacity: 0` until hover). Invisible to keyboard users, easy to miss while scanning.
- **Settings sidebar nav launches modals** instead of navigating (`SettingsView.vue:13-28`). "Manage Campaigns" and "Import Books" highlight in the sidebar but pop AppModals — the "left = where I am" mental model breaks.
- **Module dashboard has three absolutely-positioned overlay panels** for Monster, Trap, and POI details (`ModulesTab.vue:1252-1269`), z-index-stacked with shadows. With shadow-stacking it's easy to lose track of which is on top.

None of these is broken. All of them cost flow.

## Goals & Non-Goals

**Goals:**
- New users have an obvious first action on Home.
- Campaign dashboard chrome is one row, not three.
- Module dashboard sections gain visual hierarchy via content-typed accent borders.
- Play action is visually dominant; Delete demoted to overflow.
- Reorder controls always visible.
- Settings sidebar nav and the right panel stay in sync (no modal-pop nav).
- Module dashboard's three overlay panels collapse into one contextual Inspector.

**Non-Goals:**
- Visual polish unrelated to hierarchy (covered by UI Cohesion Pass).
- New combat tooling (covered by Play Mode Combat Tools).
- Theme/color/typography changes (covered by Theme System Maturation).

## Detailed Design

- **Home CTAs.** Three buttons under the tagline, lower in the viewport so the skull stays the visual anchor: `Continue [last campaign]` (visible only if `localStorage.selectedCampaignId` resolves), `Create campaign`, `Import campaign`. Inherit existing fade-in animation; staggered `animation-delay`.
- **Campaign chrome consolidation.** Collapse `CampaignDashboardView.vue` campaign-header into a single row with `DashboardTabs`: name (left), tabs (center), Sources/PDF/Export menu (right). ~50px reclaimed.
- **Dashboard section accent borders.** 3px left-border per content type: Documents → `--color-dnd-spell` (purple), NPCs → `--color-warning` (amber), Maps → `--color-info` (blue), Dangers → `--color-dnd-damage` (red). Pattern reuse from `CharacterCard.vue:84`.
- **Play promotion / Delete demotion.** `ModulesTab.vue:52-60`: Play becomes `btn-primary btn-lg` with leading arrow icon. PDF stays secondary. Delete moves into a `…` overflow menu (or into the title-edit area — design-phase decision).
- **Reorder always visible.** Drop the `opacity: 0; transition: opacity 0.15s` pattern. Either small always-visible chevrons (cheap), or vue-draggable handles (richer). Decision deferred to first task.
- **Settings sidebar nav rework.** `Manage Campaigns` and `Import Books` become embedded panels in the right content area, not modal launchers. Sidebar selection drives content, modals stop pretending to be navigation.
- **Module dashboard Inspector merge.** Replace the three `position: absolute; z-index: 10` panels (Monster / Trap / POI) with a single `Inspector.vue` that morphs based on which entity is selected. State: `selectedEntity: { type, data } | null`. One panel slot, one transition.

## UI/UX Design

Wireframes attached at decompose time. Reference targets:

- **Home:** hero (skull + title + tagline) + CTA group below tagline + ambient pulse unchanged.
- **Campaign Dashboard:** single chrome row + tab content (~50px reclaimed vertical).
- **Module Dashboard:** 2-col grid with 3px-accent-bordered sections + single Inspector slot in the right column.
- **Settings:** sidebar drives right-pane content; no modals from sidebar nav.

## Alternatives Considered

- **Leave HomeView decorative.** Defer onboarding to first-time wizards. Rejected because three CTAs cost almost nothing and help every visit, not just the first.
- **Stack Inspector panels with z-index ordering.** Rejected — the cognitive cost of "which panel is on top?" is *the* problem, not stacking quality.
- **Replace tabs with a vertical sidebar nav** (Linear-style). Rejected for now: existing tab pattern is recognizable for D&D-tool users and the dashboard isn't deep enough vertically to justify another nav rail.
- **Always-visible drag handles via vue-draggable.** Considered, kept as an option for the Reorder task. Cheaper to start with always-visible chevrons.

## Implementation Plan

- **Phase 1 — Home CTAs.** Smallest, cleanest demo of intent. One PR.
- **Phase 2 — Campaign chrome consolidation.** Touches `CampaignDashboardView` + `DashboardTabs` styling + a few callers. One PR.
- **Phase 3 — Module Dashboard hierarchy.** Color-bordered sections + Play promotion + Delete demotion. One PR.
- **Phase 4 — Reorder always visible.** `ModuleList`, `DocumentSidebar`. One PR.
- **Phase 5 — Settings sidebar nav rework.** Convert modal-launch nav items into embedded panels. One PR.
- **Phase 6 — Inspector panel merge.** Build `Inspector.vue`, retire the three overlay panels. One PR (largest of the set).