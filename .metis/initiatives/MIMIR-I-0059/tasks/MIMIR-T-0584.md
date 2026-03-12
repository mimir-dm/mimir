---
id: gap-analysis-homebrew-system
level: task
title: "Gap analysis: homebrew system documentation (items, monsters, spells)"
short_code: "MIMIR-T-0584"
created_at: 2026-03-11T23:13:28.922783+00:00
updated_at: 2026-03-11T23:13:28.922783+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Gap analysis: homebrew system documentation (items, monsters, spells)

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

The homebrew system (items, monsters, spells) is a major feature with zero user-facing documentation. Analyze the feature surface area and produce a documentation plan specifying what pages need to be written, where they should live in the docs structure, and what screenshots are needed.

## Scope

Analyze the homebrew system across three domains — items, monsters, and spells — to determine what documentation is needed.

### Systems to Analyze
- **Homebrew Items**: `HomebrewService` in mimir-core, UI components for create/edit/list, integration with character inventory
- **Homebrew Monsters**: `HomebrewMonsterService`, VTT stat block integration, module monster list integration
- **Homebrew Spells**: `HomebrewSpellService`, character spell assignment, spell slot tracking

### Code to Review
- `crates/mimir-core/src/services/homebrew*.rs` — service layer capabilities
- `crates/mimir/frontend/src/` — homebrew UI components
- `crates/mimir-mcp/src/tools/homebrew*.rs` — MCP tool surface area
- Tauri commands for homebrew operations

## Acceptance Criteria

- [ ] Map the complete homebrew item workflow: create → edit → assign to character → view in inventory
- [ ] Map the complete homebrew monster workflow: create → edit → add to module → place on map as token
- [ ] Map the complete homebrew spell workflow: create → edit → assign to character → view in spell list
- [ ] For each workflow, specify what how-to guide pages are needed and their proposed titles
- [ ] Propose where in the SUMMARY.md structure new homebrew docs should live (likely `how-to/homebrew/`)
- [ ] Identify all UI screens involved and flag which need screenshots — describe what each screenshot should show
- [ ] Check whether homebrew features need reference page additions (glossary terms, file format notes)
- [ ] Produce a documentation plan as the task output

## Expected Output

A documentation plan listing:
1. New pages needed (with proposed filenames and SUMMARY.md placement)
2. Existing pages that need homebrew mentions added
3. Screenshots needed (with description of what to capture)
4. Glossary terms to add

## Status Updates

*To be added during implementation*