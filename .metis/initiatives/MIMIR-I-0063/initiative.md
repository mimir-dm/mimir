---
id: contour-aware-terrain-generation
level: initiative
title: "Contour-aware terrain generation: physically plausible roads, rivers, and water"
short_code: "MIMIR-I-0063"
created_at: 2026-03-16T13:36:32.139085+00:00
updated_at: 2026-03-16T13:36:32.139085+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: contour-aware-terrain-generation
---

# Contour-aware terrain generation: physically plausible roads, rivers, and water Initiative

## Problem

Generated maps treat the noise map as a placement filter rather than physical terrain. Roads punch through cliff faces, rivers meander randomly instead of following valleys, lakes form arbitrary blobs on hilltops. The result looks procedurally generated rather than geographically plausible.

The fundamental issue: the noise map defines terrain elevation, but roads, rivers, and water don't navigate it as a landscape — they ignore it.

## Success Criteria

1. **A forest map with cliffs and a road looks like a real place** — the road follows a valley or ridge between cliff faces, not through them. Where a road does cross a contour, the cliff visually terminates at the road edge (road cut).
2. **Rivers flow downhill** — river paths follow noise valleys, not random meanders. The river corridor fits between surrounding contour lines.
3. **Lakes sit in basins** — water fills local noise minima, not arbitrary threshold blobs that overlap high ground.
4. **Heavily contoured maps produce appropriate warnings** — the generator tells you when terrain is too rough for the features you requested.
5. **Users have an intuitive control** — the `effort` parameter on roads lets them dial between "natural trail" and "major highway."

## Correct Physical Model

The noise map is the source of truth. Everything derives from it physically:

1. **Noise map** → terrain elevation exists
2. **Contour lines** → visual representation of terrain (generated first)
3. **Water** → fills local minima (basins surrounded by higher ground)
4. **Rivers** → flow from high to low following steepest descent, constrained to valleys between contours
5. **Roads** → human-built, follow paths of least elevation change, cross contours only with sufficient effort

### Pipeline Order Change

Current: noise → terrain → roads → rivers → objects → water → contours
Required: noise → terrain → **contours** → roads → rivers → objects → water

Contours must exist before roads/rivers so pathfinding can see them.

### Road Effort Model

A configurable `effort` parameter (0.0–1.0) on road configs:

- **0.0 (natural path)** — strongly avoids contour crossings, may take very indirect routes or fail on heavily contoured maps
- **0.5 (moderate construction)** — cuts through minor contours (1 line) but routes around major elevation changes. Default.
- **1.0 (major infrastructure)** — punches through anything. Mountain passes, cliff cuts.

When a road crosses a contour with sufficient effort, the contour path gets **split and trimmed** around the road corridor — cliff faces terminate at the road edge, showing the road cut. A high-effort road through heavy contours looks like a mountain highway with cliff walls on both sides.

Implementation: contour-crossing penalty in pathfinding scored as `penalty * (1.0 - effort)`. Post-processing clips contour paths within `road_width/2 + margin` of the final road corridor.

### Physical Constraints

- **Roads** need clearance between contour lines: `road_width/2 + contour_width/2`. Pathfinding rejects candidates that don't fit.
- **Rivers** need valley width: `river_width + bank_width` must fit between surrounding contours.
- **Lakes** only form at local noise minima bounded by higher ground.
- **Highly contoured maps** have natural anti-affinity for all features — fewer viable paths, narrower rivers, smaller lakes.

### Terrain Passability Warnings

Advisory warnings (not errors) when terrain conflicts with requested features:

- "Warning: N% of map area is within contour corridors — road/river generation may fail or produce unrealistic paths"
- "Warning: no viable path found between contour lines for road [id]"
- "Warning: lake at threshold X overlaps N contour lines"
- "Warning: river corridor narrower than river width at N points"

## Scope

**In scope:**
- Pipeline reorder (contours before roads/rivers)
- Road effort model with contour-crossing penalty and contour clipping
- Elevation-consistent road pathfinding
- Valley-following river pathfinding
- Basin-constrained lake generation
- Terrain passability analysis and warnings

**Out of scope (future):**
- Bridges and fords (road-water interaction)
- Switchback road generation
- Tunnel visualization
- Waterfall generation where rivers cross contours

## Implementation Plan

### Task 1: Pipeline reorder + contour clipping
Reorder generation so contours exist before roads. Add post-processing to clip contour paths around road/river corridors. This alone fixes the visual issue of "cliffs through roads."

### Task 2: Road effort model + elevation-aware pathfinding
Add `effort` parameter to `RoadConfig`. Modify greedy walk scoring to penalize contour crossings scaled by `(1.0 - effort)`. Reject candidates that don't have clearance for road + contour widths.

### Task 3: River effort model + river-lake connections
Rivers get the same effort model as roads — low effort follows valleys, high effort erodes through contours (contour clipping at river corridor). Rivers can connect to lakes via `source` or `drain` references instead of only running edge-to-edge.

### Task 4: Declarative lakes with organic shorelines
Lakes become placed features (center, radius, colors) rather than noise-threshold blobs. The pipeline:
1. Place lake at declared position/size
2. Perturb shoreline using noise map for organic irregular shape (configurable `roughness`: 0.0 = smooth oval, 1.0 = jagged natural shore)
3. Bezier smooth the perturbed boundary
4. Depress the noise map within the lake boundary (terrain textures naturally show mud/sand at shore)
5. Contours terminate at the lake edge
6. Objects excluded from the lake area
7. Water polygon is the organic lake boundary

Example config:
```yaml
lakes:
  - id: forest_lake
    center: [16, 12]
    radius: 6
    roughness: 0.5
    deep_color: "ff2a7f6f"
    shallow_color: "ff3ac3b2"
    blend_distance: 3.0
```

The existing `water` config remains for swamp-style scattered puddles where emergent water makes sense, but is no longer the primary way to place lakes.

### Task 5: Terrain passability analysis and warnings
Post-generation analysis that warns when terrain density conflicts with requested features. Advisory, not blocking.

## Verification

**Every task requires human-mediated testing before signoff.** Generated maps must be opened in Dungeondraft and visually verified. Automated tests confirm code correctness but cannot validate DD format compatibility or visual quality. No task is complete until the user has opened the output in DD and confirmed it works.

## Examples from Current Output

- `gull-rock-forest.dungeondraft_map` (seed 42): cliff contours at thresholds 0.55 and 0.4 cross directly through the road
- Lake preset: water polygon is arbitrary noise blob, not constrained to a basin