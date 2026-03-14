---
id: enforce-50mb-upper-limit-on-map
level: task
title: "Enforce 50MB upper limit on map uploads"
short_code: "MIMIR-T-0625"
created_at: 2026-03-14T11:29:50.746942+00:00
updated_at: 2026-03-14T11:29:50.746942+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Enforce 50MB upper limit on map uploads

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Objective

The 50MB map upload limit is enforced only in the UI (`MapUploadModal.vue:168`). Two issues: (1) the limit is too restrictive for high-res maps and should be raised, and (2) maps approaching 50MB cause severe rendering performance issues (choppy VTT display). The MCP `create_map` tool has no size check at all.

## Details

- **Type**: Bug / Enhancement
- **Priority**: P2 — Medium
- **Discovered**: 2026-03-14
- **Current limit**: 50MB, frontend-only in `MapUploadModal.vue:168-172`
- **Upload path**: UI only (file picker in `MapUploadModal.vue`), MCP doesn't handle map file uploads
- **Performance**: Maps near 50MB render very poorly in the VTT (see MIMIR-T-0626)

## Acceptance Criteria

- [ ] Raise the upload limit in `MapUploadModal.vue` (determine new ceiling based on testing)
- [ ] Document the limit in user-facing upload docs
- **Related**: MIMIR-T-0626 (VTT rendering perf for large maps — must be addressed before raising the limit significantly)

## Status Updates

*To be added during implementation*