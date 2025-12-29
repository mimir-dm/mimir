// Mimir Map Print Template - Tiled (Play Mode)
// Renders map tiles at 1"=5ft scale for physical tabletop play

#import "/_shared/styles.typ": *
#import "/_shared/cutouts.typ": render-cutout-pages

// Document setup - landscape for map tiles
#set page(
  paper: "us-letter",
  flipped: true,
  margin: 0.5in,
)

// Helper to safely get nested data
#let get(obj, key, default: none) = {
  if obj != none and key in obj { obj.at(key) } else { default }
}

// =============================================================================
// DATA EXTRACTION
// =============================================================================

#let map-name = get(data, "name", default: "Map")
#let tiles = get(data, "tiles", default: ())
#let tiles-x = get(data, "tiles_x", default: 1)
#let tiles-y = get(data, "tiles_y", default: 1)
#let total-tiles = get(data, "total_tiles", default: 0)
#let grid-width = get(data, "grid_width", default: 0)
#let grid-height = get(data, "grid_height", default: 0)
#let tile-grid-w = get(data, "tile_grid_width", default: 9)
#let tile-grid-h = get(data, "tile_grid_height", default: 6)
#let include-cutouts = get(data, "include_cutouts", default: false)
#let tokens = get(data, "tokens", default: ())

#let cell-size = 40pt

// =============================================================================
// ASSEMBLY GUIDE (First Page)
// =============================================================================

#align(center)[
  #text(size: 18pt, weight: "bold")[#map-name]
  #v(4pt)
  #text(size: 11pt, fill: luma(100))[Play Mode - 1" = 5ft Scale]
]

#v(12pt)

// Assembly diagram
#align(center)[
  #block(
    stroke: 1pt + luma(180),
    radius: 4pt,
    inset: 16pt,
  )[
    #text(size: 12pt, weight: "bold")[Assembly Guide]
    #v(8pt)

    #box(
      width: tiles-x * cell-size,
      height: tiles-y * cell-size,
    )[
      #for row in range(tiles-y) {
        for col in range(tiles-x) {
          let label = str.from-unicode(65 + row) + str(col + 1)
          place(
            dx: col * cell-size,
            dy: row * cell-size,
          )[
            #box(
              width: cell-size,
              height: cell-size,
              stroke: 1pt + luma(150),
              fill: luma(245),
            )[
              #align(center + horizon)[
                #text(size: 10pt, weight: "bold")[#label]
              ]
            ]
          ]
        }
      }
    ]

    #v(12pt)
    #text(size: 10pt, fill: luma(80))[
      Total: #total-tiles tiles (#tiles-x columns × #tiles-y rows)
    ]
  ]
]

#v(16pt)

// Instructions
#block(
  width: 100%,
  inset: 12pt,
  stroke: 1pt + luma(200),
  radius: 4pt,
  fill: luma(250),
)[
  #text(size: 11pt, weight: "bold")[Assembly Instructions]
  #v(6pt)
  #set text(size: 10pt)

  + Print all pages on US Letter paper
  + Cut along the outer edges of each tile
  + Arrange tiles according to the grid above
  + Tape tiles together on the back
  + Each tile is labeled with its position (e.g., A1, B2)
]

#v(16pt)

// Map info
#align(center)[
  #text(size: 10pt, fill: luma(120))[
    Map: #grid-width × #grid-height grid squares
  ]
]

// =============================================================================
// TILE PAGES
// =============================================================================

#for (idx, tile) in tiles.enumerate() {
  let tile-path = get(tile, "path", default: none)
  let tile-label = get(tile, "label", default: "?")
  let left-n = get(tile, "left_neighbor", default: none)
  let right-n = get(tile, "right_neighbor", default: none)
  let top-n = get(tile, "top_neighbor", default: none)
  let bottom-n = get(tile, "bottom_neighbor", default: none)
  let tile-w = get(tile, "width_px", default: 486)
  let tile-h = get(tile, "height_px", default: 324)

  pagebreak()

  // Header
  align(center)[
    #text(size: 12pt, weight: "bold")[#map-name - Tile #tile-label]
    #h(12pt)
    #text(size: 9pt, fill: luma(120))[(#tile-w × #tile-h px)]
  ]

  v(4pt)

  // Neighbor indicators
  if top-n != none {
    align(center)[
      #text(size: 8pt, fill: luma(120))[↑ #top-n]
    ]
  }

  v(2pt)

  // Main content row
  align(center)[
    // Left indicator
    #if left-n != none [
      #text(size: 8pt, fill: luma(120))[← #left-n]
      #h(4pt)
    ]

    // Tile image - use actual dimensions, scaled to fit
    #if tile-path != none [
      #box(
        width: 9in,
        height: 6in,
        stroke: 0.5pt + luma(200),
      )[
        #image(tile-path, width: 100%, height: 100%, fit: "contain")
      ]
    ] else [
      #rect(width: 9in, height: 6in, fill: luma(240))[
        #align(center + horizon)[No image]
      ]
    ]

    // Right indicator
    #if right-n != none [
      #h(4pt)
      #text(size: 8pt, fill: luma(120))[#right-n →]
    ]
  ]

  v(2pt)

  // Bottom indicator
  if bottom-n != none {
    align(center)[
      #text(size: 8pt, fill: luma(120))[#bottom-n ↓]
    ]
  }

  // Footer
  place(bottom + center)[
    #text(size: 8pt, fill: luma(150))[
      Page #(idx + 2) of #(total-tiles + 1)
    ]
  ]
}

// =============================================================================
// TOKEN CUTOUTS (Optional)
// =============================================================================

#if include-cutouts and tokens.len() > 0 {
  render-cutout-pages(map-name, tokens)
}
