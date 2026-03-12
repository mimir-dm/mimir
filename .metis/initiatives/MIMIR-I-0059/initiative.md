---
id: documentation-review-accuracy
level: initiative
title: "Documentation review, accuracy audit, and gap analysis"
short_code: "MIMIR-I-0059"
created_at: 2026-03-11T23:12:26.439213+00:00
updated_at: 2026-03-11T23:13:17.233905+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: M
initiative_id: documentation-review-accuracy
---

# Documentation review, accuracy audit, and gap analysis Initiative

## Context

Mimir's documentation (125 mdBook pages) has grown alongside rapid feature development. Several features have shipped since the docs were last comprehensively reviewed — including homebrew monsters, character spell management, map generation (mimir-mapgen), MCP tool updates, and print/PDF improvements. The docs follow the Diataxis framework (tutorials, how-to, reference, explanation) but need an accuracy pass against the current codebase and UI, plus gap analysis for undocumented features.

### Current Documentation Structure
- **Tutorials** (4): First campaign, first module, first session, player display
- **How-To Guides** (23): Campaigns, maps, characters, modules, play mode
- **Reference** (17): UI reference, keyboard shortcuts, file formats, vision/lighting, glossary
- **Explanation** (5): Campaigns vs modules, two-board system, document workflow, vision system
- **Campaign Framework** (54): Methodology, templates, session management
- **Developer** (21): Architecture, contributing, setup, source data schemas

### Known Documentation Gaps
- Homebrew system (items, monsters, spells) — no user-facing docs
- Character spell management — not documented
- MCP server / Claude Code plugin — no user-facing docs
- Map generation (mimir-mapgen) — brand new, no docs
- Export/import campaigns — how-to exists but may be stale
- Print/PDF features — how-to exists for maps and character sheets, may need update
- Developer docs may not reflect current architecture (service layer, mapgen crate)

## Goals & Non-Goals

**Goals:**
- Verify accuracy of all existing documentation against the current codebase and UI
- Identify gaps where features exist but documentation is missing or incomplete
- Request screenshots where visual context would significantly help users
- Ensure developer docs reflect current architecture and crate structure
- Ensure the glossary and reference pages are up to date

**Non-Goals:**
- Rewriting the Campaign Framework methodology content (it's stable)
- Writing all the new documentation (that's follow-up work — this initiative produces the audit and tasks for new content)
- Redesigning the docs site or changing the mdBook setup

## Detailed Design

Each task systematically reviews one section of the docs by:
1. Reading every page in the section
2. Cross-referencing against the current codebase (UI components, services, models)
3. Flagging inaccuracies (wrong steps, outdated screenshots, renamed features)
4. Identifying missing content that users would need
5. Producing a concrete list of fixes and additions needed
6. Noting where screenshots would be valuable

Tasks are organized by documentation section to keep reviews focused and independently completable.

## Implementation Plan

1. Transition through discovery → design → ready → decompose
2. Create tasks per docs section (tutorials, how-to categories, reference, explanation, developer, plus gap-fill tasks for undocumented features)
3. Each task produces an audit report as a document edit with findings and recommendations
4. Follow-up initiative(s) can take the audit findings and produce the actual doc updates