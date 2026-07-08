---
id: shared-catalogsearch-module-in
level: initiative
title: "Shared CatalogSearch module in mimir-core"
short_code: "MIMIR-I-0072"
created_at: 2026-07-08T11:08:00.953162+00:00
updated_at: 2026-07-08T11:08:00.953162+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
initiative_id: shared-catalogsearch-module-in
---

# Shared CatalogSearch module in mimir-core Initiative

## Context **[REQUIRED]**

Candidate #4 (Worth exploring) from the 2026-07-07 architecture review.

The catalog has a good deep floor — the `CatalogEntityService` trait
(`crates/mimir-core/src/services/catalog/mod.rs`) gives 24 entity types a uniform
search/get/count interface. But the two frontends built different interfaces on
top of it:

- Tauri: per-category commands in `crates/mimir/src/commands/catalog/*`
- MCP: one `search_catalog(category, ...)` tool in `crates/mimir-mcp/src/tools/catalog.rs`
  — which is ALSO the only place that merges campaign homebrew monsters into results

So "search that respects the campaign's sources and homebrew" exists once, in the
wrong layer, in one frontend. The UI and the agent can see different monsters for
the same query. Two adapters over the same floor with different behavior = a real
seam missing its module.

Deletion test passes: delete the MCP tool's merge logic and the behavior must
reappear somewhere shared to keep both frontends honest.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- A `CatalogSearch` module in `mimir-core`: category-dispatched search with campaign-source filtering and opt-in homebrew merging
- Both frontends become thin adapters over it
- Merge/filter semantics get service-level tests (today reachable only through the MCP tool)

**Non-Goals:**
- Replacing `CatalogEntityService` or the 24 per-entity services (CatalogSearch composes them)
- Changing the Tauri commands' public signatures or the MCP tool's argument shape
- FTS changes (`fts/` module untouched)

## Detailed Design **[REQUIRED]**

Rough shape: `CatalogSearch::new(conn)` with
`search(category, query/filters, campaign_ctx: Option<CampaignContext>) -> SearchResults`,
where `CampaignContext` carries source codes + homebrew-merge flag. Homebrew merge
initially for monsters (current behavior), extensible to items/spells. Category
dispatch reuses the existing per-entity services/filters.

## Alternatives Considered **[REQUIRED]**

- **Move homebrew merge into each entity service**: rejected — merging is a cross-cutting search concern, not a per-entity one.
- **Have the Tauri UI call the same per-category shape and skip unification**: rejected — leaves the sources/homebrew rules duplicated or one-sided forever.

## Implementation Plan **[REQUIRED]**

1. CatalogSearch over the trait + tests (source filter, homebrew merge)
2. Rewire MCP `search_catalog` (behavior identical; the wire tests pin it)
3. Rewire Tauri catalog commands as adapters
4. Follow-up decision: should the UI's pickers get homebrew-merged results too? (product call, record as it lands)

Related: independent of MIMIR-I-0069/0070/0071; benefits from MIMIR-I-0069's registry if MCP-side arg handling changes.
