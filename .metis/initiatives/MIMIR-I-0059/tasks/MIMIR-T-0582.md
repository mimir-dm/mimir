---
id: audit-explanation-section-and
level: task
title: "Audit explanation section and vision/lighting reference"
short_code: "MIMIR-T-0582"
created_at: 2026-03-11T23:13:28.723830+00:00
updated_at: 2026-03-13T12:36:49.302746+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

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

### Audit Completed 2026-03-12

---

#### Explanation Pages

**campaign-vs-module.md — Accurate.** The two-level hierarchy (campaign → module) is well-explained and matches the current data model. Practical examples are clear. Boundary cases section is helpful. No issues found.

**two-board-system.md — 2 issues found**

1. **INACCURACY: "Dangers" section in Module Board** — Lists "Dangers" as a module section. The actual component is `ModuleMonsters.vue`, labeled "Monsters" not "Dangers." Fix: Rename to "Monsters" and include mention of homebrew monsters.

2. **MISSING: Homebrew tab not mentioned** — The Campaign Board section lists 4 tabs (Campaign, Modules, NPCs, PCs). The dashboard now has a 5th Homebrew tab. Fix: Add Homebrew tab to the Campaign Board section with a brief description.

**document-workflow.md — 3 major issues (entire page is inaccurate)**

1. **INACCURACY: Document stages don't exist** — The entire page is built around a "stage-based document system" with Draft → Review → Complete stages. The actual `Document` type has NO stage/phase/status field. Documents are created and auto-saved with no progression system. This makes the entire premise of the page false.

2. **INACCURACY: Templates don't exist** — Page describes 4 templates (Session Notes, Location Description, Encounter Plan, NPC Profile). `CreateDocumentModal.vue` has no template selection — just a title input. Fix: Remove templates section entirely.

3. **RECOMMENDATION: Rewrite or remove this page** — Since the core concept (stage-based progression) doesn't exist, this page needs a complete rewrite. Could be restructured around the actual document workflow: create → edit → auto-save, with a focus on how documents organize campaign/module content. Or remove entirely and let the how-to guides cover document usage.

**vision-system.md — Accurate.** D&D 5e vision rules, Mimir implementation (fog of war, vision radius, light sources, wall occlusion), vision modes (Fog/Token/Reveal), what creates vision — all correct. Well-structured conceptual explanation. No issues.

---

#### Vision & Lighting Reference (already audited in MIMIR-T-0581)

**vision-and-lighting.md — Accurate.** Already confirmed in previous task. Vision types, ambient light levels, light source ranges, fog of war modes — all match.

---

#### Cross-Cutting Issues

1. **Document workflow page is fundamentally broken** — This is the most significant finding. An entire explanation page describes a feature (document stages + templates) that doesn't exist. This page should either be rewritten around the actual workflow or removed.

2. **"Dangers" vs "Monsters"** — Same issue as module-prep-view.md reference page. Should be "Monsters" consistently.

3. **Homebrew as a concept** — The explanation section would benefit from a new page explaining homebrew content: what it is, how it differs from catalog content, when to create custom vs use catalog. This would help new users understand the Homebrew tab.

#### Diagram Recommendations
- Campaign → Module hierarchy diagram (Mermaid) — would enhance campaign-vs-module.md
- Prep board vs Play board visual — would clarify two-board-system.md
- Vision calculation flowchart — would help vision-system.md