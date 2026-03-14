---
id: vtt-rendering-performance-degrades
level: task
title: "VTT rendering performance degrades on large map images"
short_code: "MIMIR-T-0626"
created_at: 2026-03-14T11:36:11.521217+00:00
updated_at: 2026-03-14T11:36:11.521217+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# VTT rendering performance degrades on large map images

## Objective

Maps approaching the current 50MB upload limit cause severe rendering performance issues in the VTT — choppy panning, slow zoom, laggy token placement. Investigate the image rendering pipeline and improve performance so that large, high-resolution maps are usable.

## Details

- **Type**: Tech Debt
- **Priority**: P2 — Medium
- **Discovered**: 2026-03-14
- **Symptom**: VTT becomes very choppy when displaying large map images (near 50MB file size)
- **Related**: MIMIR-T-0625 (raising the upload limit makes this worse if not addressed)

## Investigation (2026-03-14)

### Current Architecture

The rendering pipeline has **zero image optimization** at any stage:

**Backend (`serve_map_image` in `commands/map/uvtt.rs`):**
1. Loads entire UVTT file from disk into memory (UVTT = JSON with embedded base64 PNG)
2. Parses the full JSON
3. Extracts the `image` field (base64 PNG string)
4. Returns as data URL: `data:image/png;base64,{...}`
5. **No caching** — re-extracts from UVTT JSON on every call

**Frontend (`DmMapViewer.vue`):**
1. Receives data URL via Tauri IPC
2. Sets it as `src` on a plain `<img>` tag
3. Browser decodes and renders the full-resolution image
4. CSS `translate3d`/`scale3d` transforms handle pan/zoom (GPU-accelerated)
5. SVG overlays layered on top: grid pattern, fog mask (with Gaussian blur filter), tokens, lights, LOS debug

**No image processing libraries in the project** — no `image` crate, no canvas manipulation, no WebGL.

### Identified Bottlenecks (ordered by likely impact)

**1. Base64 Data URL Transfer (~33% bloat + IPC overhead)**
A 40MB PNG image becomes ~53MB as base64, serialized through Tauri IPC as a JSON string. This is the single largest transfer in the app. Every map load pays this cost, every time.

**2. Full-Resolution `<img>` Rendering**
The browser must decode and composite the entire image regardless of zoom level. At zoom 0.1x, you're rendering a ~8000x6000px image to fill a ~800x600px viewport — the browser still holds the full bitmap in GPU memory.

**3. SVG Fog Mask with Gaussian Blur**
`feGaussianBlur stdDeviation="20"` applied over the entire fog mask. Blur filters are expensive and scale with the masked area. On a large map at low zoom, this covers the full viewport. The blur filter extends 50% in all directions via filter dimensions.

**4. Visibility Polygon Raycasting**
`useVisibilityPolygon.ts` does O(walls × rays) raycasting per token. With 100+ walls and 4+ PC tokens, each recomputation is expensive. Recomputes on: token move, portal toggle, any dependency change. No spatial acceleration (quad-tree, BSP).

**5. Token Image Loading**
All monster token images loaded at once via `Promise.all()` as individual base64 data URLs. With many tokens on a large map, this compounds memory pressure.

### Not a Bottleneck
- Pan/zoom transforms themselves are fast (GPU-accelerated CSS)
- Grid overlay uses SVG patterns (efficient, resolution-independent)
- Token positioning is simple DOM layout

## Options

### Option A: Extract Image on Upload + Serve as File URL (High Impact, Low Risk)
**Effort: S-M**

On upload, extract the PNG from the UVTT JSON and write it as a separate file alongside the UVTT:
```
assets/{uuid}.uvtt      ← metadata only (walls, lights, portals)
assets/{uuid}.png       ← extracted image
```

Serve via Tauri's `asset:` protocol or `convertFileSrc()` instead of data URLs. The browser loads the image as a normal file — no base64 bloat, no IPC serialization of megabytes of string data, and the browser can use its native image decoder with streaming/progressive decode.

**Pros:** Eliminates the #1 bottleneck entirely. Simple to implement. Browser caches the decoded image bitmap.
**Cons:** Requires migration for existing maps. Doubles disk usage briefly (UVTT still has the embedded image).

### Option B: Generate Multi-Resolution Variants on Upload (High Impact, Medium Risk)
**Effort: M**

On upload, extract the PNG and generate 2-3 resolution tiers using the `image` crate:
```
assets/{uuid}.png       ← full resolution (for max zoom)
assets/{uuid}_mid.png   ← 50% resolution (for normal zoom)
assets/{uuid}_thumb.png ← 25% resolution (for zoomed-out / overview)
```

Frontend swaps `src` based on current zoom level with debounced transitions. At zoom < 0.5x, use the mid or thumb variant. Only load full-res when zoomed in past 1x.

**Pros:** Dramatically reduces GPU memory at common zoom levels. Fast overview rendering.
**Cons:** Adds `image` crate dependency. Upload takes longer (processing time). More disk usage. Swap transitions need to be invisible.

### Option C: Optimize Fog Blur Filter (Medium Impact, Low Risk)
**Effort: XS**

Replace the expensive Gaussian blur with a simpler technique:
- Use `feDropShadow` instead of `feGaussianBlur` (typically faster)
- Reduce blur radius from 20 to 10 (perceptually similar at map scale)
- Apply blur only to the vision polygon edges, not the entire mask
- Or: pre-blur the mask at a fixed resolution and use `image-rendering: auto` to let the browser scale it

**Pros:** Quick win, no architectural change.
**Cons:** Subtle visual difference in fog edges.

### Option D: Canvas-Based Map Rendering (High Impact, High Risk)
**Effort: L-XL**

Replace the `<img>` + SVG overlay approach with an HTML5 Canvas or WebGL renderer:
- Render only the visible viewport tiles
- Implement tile-based level-of-detail
- Fog, grid, and tokens all rendered to canvas layers
- Libraries like Pixi.js or custom WebGL could handle this

**Pros:** Solves all rendering bottlenecks. Industry standard for VTTs (Roll20, Foundry use canvas).
**Cons:** Major rewrite. Loses SVG debugging ease. Complex interaction handling. High risk of regressions.

### Option E: Spatial Acceleration for Visibility (Medium Impact, Low Risk)
**Effort: S**

Add a spatial index (grid hash or quad-tree) for wall segments in `useVisibilityPolygon.ts`. Currently every ray tests every wall. With spatial indexing, only walls near the ray path are tested.

**Pros:** Directly addresses bottleneck #4. More impactful as wall count grows.
**Cons:** Only helps with fog/LOS, not image rendering.

### Option F: Lazy Token Image Loading (Low Impact, Low Risk)
**Effort: XS**

Load token images only when they enter the viewport (Intersection Observer), instead of all at once via `Promise.all()`. Also: cache decoded images across map loads.

**Pros:** Reduces initial load time and memory. Easy to implement.
**Cons:** Minor impact compared to the map image itself.

## Recommended Approach

**Phase 1 (quick wins):** Options A + C + F
- Extract images on upload, serve as file URLs — eliminates base64 bloat and IPC serialization
- Optimize the fog blur filter — immediate rendering improvement
- Lazy-load token images — reduced memory pressure
- *Estimated combined effort: M*

**Phase 2 (medium term):** Option B + E
- Generate resolution variants — smooth experience at all zoom levels
- Spatial acceleration for visibility — scales with complex maps
- *Estimated combined effort: M-L*

**Phase 3 (if needed):** Option D
- Canvas/WebGL rewrite — only if Phase 1+2 don't suffice
- *Estimated effort: XL*

## Acceptance Criteria

- [ ] Maps at the upload size limit render smoothly (no perceptible choppiness during pan/zoom)
- [ ] Token placement and fog of war remain responsive on large maps
- [ ] No regression in rendering quality when zoomed in

## Status Updates

*Investigation completed 2026-03-14 — see Options section above*