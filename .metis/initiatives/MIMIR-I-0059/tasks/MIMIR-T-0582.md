---
id: audit-explanation-section-and
level: task
title: "Audit explanation section and vision/lighting reference"
short_code: "MIMIR-T-0582"
created_at: 2026-03-11T23:13:28.723830+00:00
updated_at: 2026-03-11T23:13:28.723830+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit explanation section and vision/lighting reference

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review the 4 explanation pages and the vision/lighting reference for conceptual accuracy. These pages explain *why* things work the way they do — they need to reflect the current architecture and design decisions.

## Scope

### Explanation Pages (4)
- `docs/src/explanation/campaign-vs-module.md` — Core organizational concept
- `docs/src/explanation/two-board-system.md` — Prep board vs play board
- `docs/src/explanation/document-workflow.md` — How documents work in campaigns/modules
- `docs/src/explanation/vision-system.md` — D&D 5e vision and lighting model

### Vision & Lighting Reference
- `docs/src/reference/vision-and-lighting.md` — Detailed reference for the vision system

## Acceptance Criteria

- [ ] Verify "campaigns vs modules" explanation matches current data model and UI organization
- [ ] Check "two-board system" against current campaign dashboard and play mode — are these still the right mental model?
- [ ] Verify document workflow matches current document creation/editing/ordering in the UI
- [ ] Check vision system explanation against current implementation in play mode (darkvision, light sources, fog of war integration)
- [ ] Verify vision/lighting reference covers all current light source types and vision modes
- [ ] Check whether the explanation section should mention homebrew as a concept (creating custom content vs catalog content)
- [ ] Identify where diagrams or screenshots would clarify concepts — Mermaid diagrams are supported
- [ ] Produce findings report

## Screenshot/Diagram Candidates

- Diagram: campaign → module → document hierarchy
- Diagram: prep board vs play board workflow
- Screenshot: vision/lighting in action (darkvision radius, light source overlap)

## Status Updates

*To be added during implementation*