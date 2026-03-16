---
id: contour-aware-path-and-water
level: task
title: "Contour-aware path and water generation"
short_code: "MIMIR-T-0634"
created_at: 2026-03-16T13:22:18.123944+00:00
updated_at: 2026-03-16T13:22:18.123944+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Contour-aware path and water generation

## Problem

Roads, rivers, and water bodies are generated independently from terrain elevation (the noise map). This produces geographically unrealistic results:

- **Cliff contours cut through roads** — roads should follow the terrain, not punch through cliff faces
- **Rivers ignore elevation** — rivers should follow valleys (noise low points), not meander randomly
- **Lakes overlap contour lines** — lakes should fill depressions, not sit on top of cliffs
- **Roads cross water without bridges** — no awareness of generated water features

The fundamental issue: the noise map defines terrain, but roads/rivers/water don't respect it as physical terrain. They treat it as a placement filter rather than a landscape they must navigate.

## Correct Physical Model

The noise map is the source of truth. Everything else derives from it:

1. **Noise map** → terrain elevation exists
2. **Contour lines** → visual representation of that terrain (drawn first)
3. **Water** → fills local minima (basins surrounded by higher ground)
4. **Rivers** → flow from high to low following steepest descent, constrained to valleys between contour lines
5. **Roads** → human-built, follow paths of least elevation change (stay within an elevation band, avoid crossing contour lines)

### Pipeline Order Change

Current: noise → terrain → roads → rivers → objects → water → contours
Correct: noise → terrain → **contours** → roads → rivers → objects → water

Contours must be generated before roads/rivers so pathfinding can avoid them.

### Physical Constraints

- **Rivers** must fit in valleys: river width + bank width must fit between surrounding contour lines.
- **Lakes** only form where the terrain has a basin (local noise minimum bounded by higher ground on all sides).
- **Highly contoured maps** have natural anti-affinity for roads, rivers, and lakes. Dense cliff lines mean fewer viable paths, narrower rivers, and smaller/fewer lakes.

### Road Effort Model

Real roads cut through terrain when necessary — road cuts, switchbacks, mountain passes. The question isn't "can a road cross a contour" but "how much effort was the builder willing to invest."

A configurable `effort` parameter (0.0–1.0) on road configs controls this:

- **0.0 (natural path)** — road follows the easiest path, strongly avoids all contour crossings, may take very indirect routes or fail entirely on heavily contoured maps
- **0.5 (moderate construction)** — road will cut through minor contours (1 line) but routes around major elevation changes. Default for most maps.
- **1.0 (major infrastructure)** — road punches through anything. Mountain passes, cliff cuts, tunnels. Direct routing regardless of terrain.

When a road crosses a contour with sufficient effort:
- The contour path gets **split and trimmed** around the road corridor
- This visually shows the road cut — cliff faces terminate at the road edge
- A high-effort road through heavy contours looks like a mountain highway with cliff walls on both sides, which is geographically correct

Implementation: the road pathfinding's candidate scoring function adds a contour-crossing penalty scaled by `(1.0 - effort)`. At effort=0 the penalty is maximum (strong avoidance), at effort=1 it's zero (ignore contours). When the final road path does cross contour lines, a post-processing step clips contour paths within `road_width/2 + margin` of the road corridor.

## Terrain Passability Warnings

When the config requests features that conflict with the terrain complexity, the generator should warn:

- "Warning: N% of map area is within contour corridors — road/river generation may fail or produce unrealistic paths"
- "Warning: no viable path found between contour lines for road [id] — consider fewer elevation levels or wider contour spacing"
- "Warning: lake at threshold X overlaps N contour lines — the threshold may be too low for this terrain"
- "Warning: river corridor is narrower than river width at N points — river may clip through contour lines"

These are advisory, not errors — the generator still produces output, but the user understands why it looks odd.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Pipeline order changed: contours generated before roads/rivers
- [ ] Road pathfinding penalizes crossing contour lines (elevation consistency scoring)
- [ ] Road pathfinding respects contour width: rejects candidates within `road_width/2 + contour_width/2` of a contour path
- [ ] River pathfinding prefers noise valleys (low ground) over random meandering
- [ ] Lake water polygons are constrained to noise depressions (local minima)
- [ ] Terrain passability analysis runs after contour generation, warns when contour density conflicts with requested roads/rivers/water
- [ ] Generated maps pass visual inspection: no cliffs through roads, rivers flowing "uphill", or lakes on hilltops

## Examples from Current Output

- `gull-rock-forest.dungeondraft_map` (seed 42): cliff contours at thresholds 0.55 and 0.4 cross directly through the road
- Lake preset: water polygon shape is arbitrary noise blob, not constrained to a basin

## Status Updates

*To be added during implementation*