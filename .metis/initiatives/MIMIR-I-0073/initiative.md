---
id: unified-serviceerror-mapping-at
level: initiative
title: "Unified ServiceError mapping at both frontend seams"
short_code: "MIMIR-I-0073"
created_at: 2026-07-08T11:08:02.290226+00:00
updated_at: 2026-07-08T11:08:02.290226+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XS
initiative_id: unified-serviceerror-mapping-at
---

# Unified ServiceError mapping at both frontend seams Initiative

## Context **[REQUIRED]**

Candidate #5 (Speculative as a standalone) from the 2026-07-07 architecture review.

`ServiceError` is translated at both seams by hand:

- Tauri flattens everything to strings via `to_api_response`
  (`crates/mimir/src/commands/mod.rs`) — error *kind* is lost at the seam
- MCP handlers each write their own `match` to pick `InvalidArguments` vs
  `Internal` (`crates/mimir-mcp/src/tools/*.rs`) — the same mapping was
  hand-written three times during the fc5a3d5 consolidation alone

One `impl From<ServiceError> for McpError` plus a typed error code on
`ApiResponse` would let the service layer's error semantics survive both seams.

Deletion test passes but this is one impl block, not a project — hence
Speculative standalone.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Single `From<ServiceError> for McpError` impl; per-handler matches deleted
- `ApiResponse` optionally carries an error kind so the Vue frontend can distinguish not-found / validation / internal

**Non-Goals:**
- Unifying the ApiResponse and McpResponse envelope shapes (different clients, different needs — not worth forcing)

## Detailed Design **[REQUIRED]**

`NotFound → InvalidArguments("{entity} '{id}' not found")`, `Validation(msg) →
InvalidArguments(msg)`, `Database/Io → Internal`. Frontend: add optional
`error_kind` field to ApiResponse, defaulted so ts-rs types stay
backward-compatible.

## Alternatives Considered **[REQUIRED]**

- **Do it inside MIMIR-I-0069's registry dispatch**: PREFERRED — this initiative
  likely dissolves into that one (its step 4). Kept as a separate record so the
  decision is visible; close it as absorbed if 0069 lands first.

## Implementation Plan **[REQUIRED]**

If executed standalone: one PR — From impl, delete per-handler matches, optional
ApiResponse kind. Otherwise: close when MIMIR-I-0069 step 4 lands.
