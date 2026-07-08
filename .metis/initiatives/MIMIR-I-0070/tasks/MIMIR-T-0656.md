---
id: mcp-placement-tools-for-traps-and
level: task
title: "MCP placement tools for traps, POIs, and lights (authoring-only surface)"
short_code: "MIMIR-T-0656"
created_at: 2026-07-08T11:59:15.197870+00:00
updated_at: 2026-07-08T11:59:15.197870+00:00
parent: MIMIR-I-0070
blocked_by: ["MIMIR-T-0655"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0070
---

# MCP placement tools for traps, POIs, and lights (authoring-only surface)

## Parent Initiative

[[MIMIR-I-0070]] — MapStateService: fog, lights, traps, POIs behind one seam

## Objective **[REQUIRED]**

Expose trap, POI, and light **placement/authoring** to agents via the MCP
registry — closing the world-building capability gap while enforcing the
refined design ruling: **agents may place and edit any token/object; they never
edit the viewable layer during live play** (Dylan, 2026-07-08; memory:
feedback_agent_prep_not_play).

## Acceptance Criteria **[REQUIRED]**

- [ ] New registry-based MCP tools (one registration each, typed args) for traps (`add/list/update/remove`), POIs (`add/list/update/remove`), and lights (`add/list/update/remove`, incl. torch/lantern presets — via a preset arg or dedicated tools, implementer's call). Final names consistent with existing tool naming
- [ ] Authoring fields only: position (move via update), name/description/severity/radius/color etc. — whatever the models carry as authoring data
- [ ] **Explicitly NOT exposed**: trap trigger/reset, player-visibility toggles, light on/off toggling, fog anything. Tool descriptions must state that live-play control is DM-only so agents don't go looking for it
- [ ] Tool descriptions written for agentic clients (say when to use, reference get_map for map ids)
- [ ] Functional tests in handler.rs (lifecycle per subsystem) + stdio e2e still green
- [ ] Server instructions string in main.rs updated if it enumerates capabilities

## Implementation Notes

- All tools call MapStateService — zero new business logic in the MCP layer.
- Blocked by T-0655 (needs the service trap/POI methods).

## Status Updates **[REQUIRED]**

*To be added during implementation*
