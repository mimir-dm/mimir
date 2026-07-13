---
id: mcp-add-character-feat-tool-for
level: task
title: "MCP: add_character_feat tool for retroactive feats"
short_code: "MIMIR-T-0658"
created_at: 2026-07-10T09:54:21.544015+00:00
updated_at: 2026-07-10T09:54:21.544015+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# MCP: add_character_feat tool for retroactive feats

## Objective **[REQUIRED]**

The only MCP path that records a feat is `level_up_character` (asi_type "feat"),
taken at level-up time. If the choice was deferred (as with the 2026-07-10
level-4 pass for The Frost Architect's PCs) there is no way to apply a feat
afterward except direct DB insert into `character_feats`. The DAL and model
(`NewCharacterFeat`) already exist.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [ ] P2 - Medium (nice to have)

### Business Justification
- **User Value**: deferred ASI/feat choices become a normal operation; agents can record player picks whenever they land
- **Effort Estimate**: S — consider a CharacterService method (`add_feat`, mirroring `add_spell`) + one registry tool; possibly `list_character_feats`/`remove_character_feat` for the full lifecycle

## Acceptance Criteria **[REQUIRED]**

- [ ] `CharacterService::add_feat` (validate against catalog feat by name+source; duplicate check) — shared by MCP and any future UI path per the one-implementation rule
- [ ] Registry tool `add_character_feat` (typed args: character_id, feat_name, feat_source default PHB, optional note/level)
- [ ] Companion `list_character_feats` (and optionally remove) for round-tripping
- [ ] Functional tests: add + duplicate rejection + catalog-miss error
- [ ] Authoring-only surface — consistent with feedback_agent_prep_not_play (feats are build/prep, so exposure is fine)

## Status Updates **[REQUIRED]**

*To be added during implementation*
