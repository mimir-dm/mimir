---
id: audit-reference-pages-ui-reference
level: task
title: "Audit reference pages: UI reference, keyboard shortcuts, file formats, glossary"
short_code: "MIMIR-T-0581"
created_at: 2026-03-11T23:13:28.622267+00:00
updated_at: 2026-03-11T23:13:28.622267+00:00
parent: MIMIR-I-0059
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0059
---

# Audit reference pages: UI reference, keyboard shortcuts, file formats, glossary

## Parent Initiative

[[MIMIR-I-0059]]

## Objective

Review all reference pages for accuracy — UI reference (6 pages), keyboard shortcuts, file formats, and glossary. Reference docs are where users go to look things up, so completeness and accuracy matter most here.

## Scope

### UI Reference (6 pages)
- `docs/src/reference/ui/home-screen.md`
- `docs/src/reference/ui/campaign-dashboard.md`
- `docs/src/reference/ui/module-prep-view.md`
- `docs/src/reference/ui/play-mode.md`
- `docs/src/reference/ui/token-setup-modal.md`
- `docs/src/reference/ui/player-display.md`

### Other Reference (4 pages)
- `docs/src/reference/keyboard-shortcuts.md`
- `docs/src/reference/file-formats.md`
- `docs/src/reference/vision-and-lighting.md`
- `docs/src/reference/glossary.md`

## Acceptance Criteria

- [ ] Each UI reference page verified against current Vue components — check every described element still exists
- [ ] Verify keyboard shortcuts against current keybinding handlers in frontend code
- [ ] Check file formats reference covers UVTT import, campaign export (.mimir), and any new formats
- [ ] Verify glossary includes all current terminology (homebrew, spell slots, sidecar, MCP, etc.)
- [ ] Check whether UI reference pages need updating for homebrew panels, spell management, inventory views
- [ ] Identify where screenshots would help — UI reference pages are prime candidates
- [ ] Produce findings report

## Screenshot Candidates

UI reference pages should ideally each have an annotated screenshot showing the key elements described:
- Home screen with campaign list
- Campaign dashboard with all tabs visible
- Module prep view showing monsters, items, documents tabs
- Play mode with encounter tracker, map, and token list
- Token setup modal (recently updated with new fields)
- Player display window

## Status Updates

*To be added during implementation*