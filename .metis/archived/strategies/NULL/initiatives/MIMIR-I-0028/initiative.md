---
id: uvtt-map-format-and-line-of-sight
level: initiative
title: "UVTT Map Format and Line of Sight Blocking"
short_code: "MIMIR-I-0028"
created_at: 2025-12-25T16:01:48.977407+00:00
updated_at: 2025-12-29T03:36:58.383108+00:00
parent: MIMIR-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: uvtt-map-format-and-line-of-sight
---

# UVTT Map Format and Line of Sight Blocking Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

Maps in Mimir currently support PNG upload with a custom grid overlay. To enable proper line-of-sight (LOS) blocking for vision systems, we need wall/obstacle geometry data that PNG images don't provide.

The **Universal VTT (UVTT)** format (`.dd2vtt`) is an industry-standard export format from Dungeondraft and other map tools. It bundles:
- Map image (base64 PNG/WebP)
- Grid configuration
- LOS blocking geometry (wall polylines)
- Portals (doors/windows with open/closed state)
- Light sources

### UVTT Format Structure

```json
{
  "format": 0.3,
  "resolution": {
    "map_origin": { "x": 0, "y": 0 },
    "map_size": { "x": 35, "y": 20 },      // Grid squares
    "pixels_per_grid": 54
  },
  "line_of_sight": [                        // Array of polylines
    [{"x": 8, "y": 0}, {"x": 5, "y": 0}, ...],  // Each polyline is wall segment
    [{"x": 17.5, "y": 8}, ...]                   // Can have fractional coords
  ],
  "objects_line_of_sight": [],              // Object-based LOS (pillars, etc.)
  "portals": [                              // Doors and windows
    {
      "position": {"x": 5, "y": 5},
      "bounds": [{"x": 4.5, "y": 5}, {"x": 5.5, "y": 5}],
      "rotation": 0,
      "closed": true,                       // true = blocks LOS
      "freestanding": false
    }
  ],
  "lights": [
    {
      "position": {"x": 10, "y": 10},
      "range": 5,                           // Grid squares
      "intensity": 1.0,
      "color": "ffaa00",
      "shadows": true
    }
  ],
  "environment": {
    "baked_lighting": true,
    "ambient_light": "ffffffff"
  },
  "image": "data:image/png;base64,..."      // Embedded map image
}
```

### Current State

| Feature | Current | With UVTT |
|---------|---------|-----------|
| Map upload | PNG only | PNG or UVTT |
| Grid | Manual overlay config | Auto from UVTT or manual |
| LOS walls | None | From `line_of_sight` |
| Doors | None | From `portals` |
| Lights | Manual placement | Import from UVTT + manual |

### Design Decision: Unified UVTT Format

**All maps will be stored as UVTT internally**, regardless of upload format:

- **UVTT upload:** Parse and store directly
- **PNG upload:** Convert to UVTT after grid configuration

This means:
- Single code path for all map operations
- Consistent data model
- Ready for future LOS editing features
- Grid config in standard UVTT `resolution` structure

**Breaking changes are acceptable** - we'll migrate existing maps to the new format.

## Goals & Non-Goals

**Goals:**
- Import UVTT files with full LOS geometry
- Render LOS blocking on player display (fog of war respects walls)
- Support portal states (open/closed doors)
- Convert PNG uploads to minimal UVTT format (image + grid)
- Maintain backwards compatibility with existing PNG maps

**Non-Goals:**
- Full Dungeondraft `.dungeondraft_map` import (project files, not exports)
- LOS editing UI (draw walls manually) - future initiative
- Dynamic lighting with shadows - use existing light system
- Multi-level map support (UVTT is single-level)

## Use Cases

### UC-1: Import Dungeondraft Map
- **Actor**: DM with Dungeondraft
- **Scenario**:
  1. Export map from Dungeondraft as `.dd2vtt`
  2. In Mimir, click "Add Map" in module
  3. Select `.dd2vtt` file
  4. Map imports with image, grid, and LOS walls
  5. In play mode, fog of war respects wall geometry
- **Expected Outcome**: Tokens can't see through walls

### UC-2: Upgrade PNG Map
- **Actor**: DM with existing PNG map
- **Scenario**:
  1. Upload PNG map (current flow)
  2. Configure grid overlay as before
  3. System converts to UVTT internally (PNG + grid, no walls)
  4. Map works as before, ready for future LOS editing
- **Expected Outcome**: Seamless upgrade path

### UC-3: Toggle Door State
- **Actor**: DM during play
- **Scenario**:
  1. Map has doors from UVTT import
  2. DM clicks on door in play mode
  3. Door toggles open/closed
  4. LOS updates - open door allows vision through
- **Expected Outcome**: Dynamic door control

### UC-4: Player Vision Through Walls
- **Actor**: Player viewing display
- **Scenario**:
  1. DM sends map to player display
  2. Player tokens have vision configured
  3. Vision rays stop at LOS walls
  4. Areas behind walls remain fogged
- **Expected Outcome**: Realistic vision blocking

## Architecture

### Data Model (Breaking Changes OK)

**Redesigned `maps` table:**
```sql
CREATE TABLE maps (
  id INTEGER PRIMARY KEY,
  module_id INTEGER NOT NULL REFERENCES modules(id),
  name TEXT NOT NULL,
  
  -- Image stored as file, path reference
  image_path TEXT NOT NULL,
  
  -- UVTT resolution data (all maps have this)
  grid_width INTEGER NOT NULL,        -- map_size.x in grid squares
  grid_height INTEGER NOT NULL,       -- map_size.y in grid squares  
  pixels_per_grid INTEGER NOT NULL,   -- pixels_per_grid from UVTT
  origin_x REAL DEFAULT 0,            -- map_origin.x
  origin_y REAL DEFAULT 0,            -- map_origin.y
  
  -- Environment
  ambient_light TEXT DEFAULT 'ffffffff',
  
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

**New: `los_walls` table (polylines):**
```sql
CREATE TABLE los_walls (
  id INTEGER PRIMARY KEY,
  map_id INTEGER NOT NULL REFERENCES maps(id),
  points TEXT NOT NULL,         -- JSON array of {x, y} in grid coords
  created_at TEXT NOT NULL
);
```

**New: `los_portals` table (doors/windows):**
```sql
CREATE TABLE los_portals (
  id INTEGER PRIMARY KEY,
  map_id INTEGER NOT NULL REFERENCES maps(id),
  position_x REAL NOT NULL,
  position_y REAL NOT NULL,
  bounds TEXT NOT NULL,         -- JSON array of boundary points
  rotation REAL DEFAULT 0,
  is_closed BOOLEAN DEFAULT true,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

**Existing `light_sources` table** - reuse for UVTT lights, add any missing fields.

### Processing Pipeline

```
UVTT Upload
    ↓
Parse JSON
    ↓
┌───────────────────────────────────────┐
│ Extract image → Save as PNG           │
│ Extract resolution → Update map.grid  │
│ Extract line_of_sight → los_geometry  │
│ Extract portals → los_geometry        │
│ Extract lights → light_sources        │
│ Store metadata → maps.uvtt_metadata   │
└───────────────────────────────────────┘

PNG Upload (existing)
    ↓
Save PNG
    ↓
User configures grid
    ↓
Convert to minimal UVTT internally
(format='uvtt', no LOS geometry)
```

### LOS Rendering

```
Player Display Render
    ↓
For each visible token:
    ↓
    Get token vision range
    ↓
    Load LOS geometry for map
    ↓
    Cast rays from token position
    ↓
    Intersect rays with wall segments
    ↓
    Build visibility polygon
    ↓
    Mask fog with visibility polygon
```

**Raycasting approach:**
- Use standard 2D raycasting algorithm
- Cast rays to corners of each wall segment
- Build visibility polygon from ray endpoints
- Composite multiple token visibility for party vision

## Detailed Design

### UVTT Parser (`mimir-dm-core/src/services/uvtt_service.rs`)

```rust
pub struct UvttMap {
    pub format: f32,
    pub resolution: UvttResolution,
    pub line_of_sight: Vec<Vec<Point>>,
    pub objects_line_of_sight: Vec<Vec<Point>>,
    pub portals: Vec<UvttPortal>,
    pub lights: Vec<UvttLight>,
    pub environment: UvttEnvironment,
    pub image_data: Vec<u8>,  // Decoded from base64
}

pub struct UvttResolution {
    pub map_origin: Point,
    pub map_size: Point,      // Grid dimensions
    pub pixels_per_grid: i32,
}

pub struct UvttPortal {
    pub position: Point,
    pub bounds: Vec<Point>,
    pub rotation: f32,
    pub closed: bool,
    pub freestanding: bool,
}

impl UvttService {
    pub fn parse_uvtt(data: &[u8]) -> Result<UvttMap>;
    pub fn import_to_map(uvtt: UvttMap, module_id: i32) -> Result<Map>;
    pub fn create_minimal_uvtt(image: &[u8], grid: GridConfig) -> Result<UvttMap>;
}
```

### LOS Geometry Service

```rust
pub struct LosGeometry {
    pub id: i32,
    pub map_id: i32,
    pub geometry_type: GeometryType,
    pub points: Vec<Point>,
    pub is_closed: bool,
    pub metadata: Option<serde_json::Value>,
}

pub enum GeometryType {
    Wall,
    Portal,
    Object,
}

impl LosGeometryService {
    pub fn get_for_map(map_id: i32) -> Result<Vec<LosGeometry>>;
    pub fn import_from_uvtt(map_id: i32, uvtt: &UvttMap) -> Result<()>;
    pub fn toggle_portal(id: i32) -> Result<bool>;  // Returns new state
}
```

### Frontend Visibility Calculation

```typescript
// services/VisionService.ts

interface WallSegment {
  start: Point;
  end: Point;
}

interface VisibilityPolygon {
  points: Point[];
}

class VisionService {
  // Load LOS geometry from backend
  async loadLosGeometry(mapId: number): Promise<WallSegment[]>;
  
  // Calculate visibility from a point
  calculateVisibility(
    origin: Point,
    range: number,
    walls: WallSegment[],
    portals: Portal[]
  ): VisibilityPolygon;
  
  // Combine visibility for multiple tokens
  combinePartyVision(polygons: VisibilityPolygon[]): VisibilityPolygon;
}
```

### Canvas Rendering

```typescript
// In PlayerDisplayWindow or DmMapViewer

function renderFogWithVision(
  ctx: CanvasRenderingContext2D,
  fogMask: boolean[][],
  visionPolygon: VisibilityPolygon
) {
  // 1. Draw base fog (revealed areas)
  drawFogFromMask(ctx, fogMask);
  
  // 2. Clip to vision polygon
  ctx.save();
  ctx.beginPath();
  for (const point of visionPolygon.points) {
    ctx.lineTo(point.x, point.y);
  }
  ctx.closePath();
  ctx.clip();
  
  // 3. Clear fog within vision (or draw with transparency)
  ctx.globalCompositeOperation = 'destination-out';
  ctx.fill();
  ctx.restore();
}
```

## UI Changes

### Map Upload Dialog

```
┌─────────────────────────────────────┐
│  Add Map                            │
├─────────────────────────────────────┤
│                                     │
│  [Drop file here or click to browse]│
│                                     │
│  Supported formats:                 │
│  • PNG, JPG, WebP (image only)      │
│  • UVTT (.dd2vtt) with LOS data     │
│                                     │
├─────────────────────────────────────┤
│              [Cancel] [Upload]      │
└─────────────────────────────────────┘
```

### UVTT Import Preview

After selecting `.dd2vtt`:
```
┌─────────────────────────────────────┐
│  Import UVTT Map                    │
├─────────────────────────────────────┤
│  ┌─────────────────────────────┐   │
│  │     [Map Preview Image]     │   │
│  └─────────────────────────────┘   │
│                                     │
│  Grid: 35 x 20 squares              │
│  LOS Walls: 47 segments             │
│  Portals: 3 doors                   │
│  Lights: 5 sources                  │
│                                     │
│  ☑ Import LOS geometry              │
│  ☑ Import lights                    │
│                                     │
├─────────────────────────────────────┤
│              [Cancel] [Import]      │
└─────────────────────────────────────┘
```

### Play Mode Door Interaction

- Doors highlighted when hovering
- Click to toggle open/closed
- Visual indicator (open door icon vs closed)
- LOS updates immediately

## Implementation Plan

### Phase 1: UVTT Parser & Storage

1. **Create UVTT parser in mimir-dm-core**
   - Parse JSON structure
   - Decode base64 image
   - Extract all geometry

2. **Add database schema**
   - `los_geometry` table
   - Migration for `maps` table changes

3. **Create LosGeometryService**
   - CRUD for LOS geometry
   - Import from UVTT

### Phase 2: Map Import Flow

4. **Update map upload UI**
   - Accept `.dd2vtt` files
   - Show import preview for UVTT

5. **Backend import command**
   - Process UVTT upload
   - Store image and geometry
   - Import lights to existing system

6. **PNG to UVTT conversion**
   - After grid config, create minimal UVTT
   - Update map format field

### Phase 3: LOS Rendering

7. **Frontend VisionService**
   - Load LOS geometry
   - Raycasting algorithm
   - Visibility polygon calculation

8. **Update fog rendering**
   - Integrate vision with fog of war
   - Support combined party vision

9. **Portal interaction**
   - Door click handlers
   - Toggle portal state
   - Sync state via IPC

### Phase 4: Polish

10. **Visual feedback**
    - Door open/closed indicators
    - Wall preview on DM view (optional)
    - LOS debug view

11. **Performance optimization**
    - Cache visibility calculations
    - Spatial indexing for walls
    - Incremental updates

12. **Testing**
    - Unit tests for parser
    - Integration tests for import
    - Visual regression for rendering