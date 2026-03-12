---
id: audit-tutorials-for-accuracy
level: task
title: "Audit tutorials for accuracy against current UI and workflows"
short_code: "MIMIR-T-0578"
created_at: 2026-03-11T23:13:28.320314+00:00
updated_at: 2026-03-11T23:13:28.320314+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit tutorials for accuracy against current UI and workflows

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review all 4 tutorial pages for accuracy against the current Mimir UI and workflows. Tutorials are the first thing new users read, so accuracy is critical.

## Scope

Review these files against the current codebase and UI:
- `docs/src/tutorials/01-first-campaign.md` — Create and explore campaigns
- `docs/src/tutorials/02-first-module.md` — Build adventures with maps/monsters
- `docs/src/tutorials/03-first-session.md` — Run encounters in Play Mode
- `docs/src/tutorials/04-player-display.md` — Set up second screen for players

## Acceptance Criteria

- [ ] Each tutorial page read in full and cross-referenced against current Vue components and Tauri commands
- [ ] Every step-by-step instruction verified against current UI flow (button names, menu locations, form fields)
- [ ] Flag any instructions that reference removed, renamed, or relocated UI elements
- [ ] Check that tutorials mention features added since they were written (e.g., homebrew, spell management)
- [ ] Identify steps where a screenshot would help the reader — note exactly what the screenshot should show
- [ ] Produce a findings report as a status update listing: inaccuracies found, missing content, screenshot requests

## How to Review

For each tutorial page:
1. Read the doc page
2. Trace the described workflow through the frontend code (`crates/mimir/frontend/src/`)
3. Verify component names, route paths, and Tauri command invocations match what the doc describes
4. Check whether new features (homebrew items, spells, character inventory) should be mentioned
5. Note any places where a screenshot would clarify a step — describe what the screenshot should capture

## Status Updates

*To be added during implementation*