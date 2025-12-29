// Mimir Map Print Template - Token Cutouts
// Generates printable paper standee cutouts for tabletop play

#import "/_shared/styles.typ": *

// Document setup - portrait for cutout sheets
#set page(
  paper: "us-letter",
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
#let tokens = get(data, "tokens", default: ())

// =============================================================================
// TOKEN SIZE MAPPING
// =============================================================================

// Size in grid squares (and inches at 1"=5ft scale)
#let size-to-inches(size) = {
  let s = lower(size)
  if s == "tiny" or s == "t" { 0.5 }
  else if s == "small" or s == "s" { 1.0 }
  else if s == "medium" or s == "m" { 1.0 }
  else if s == "large" or s == "l" { 2.0 }
  else if s == "huge" or s == "h" { 3.0 }
  else if s == "gargantuan" or s == "g" { 4.0 }
  else { 1.0 }
}

// Token type to color
#let type-to-color(token-type) = {
  let t = lower(token-type)
  if t == "monster" { rgb("#dc3545") }      // Red
  else if t == "pc" { rgb("#28a745") }      // Green
  else if t == "npc" { rgb("#007bff") }     // Blue
  else if t == "trap" { rgb("#ffc107") }    // Yellow
  else if t == "marker" { rgb("#6c757d") }  // Gray
  else { rgb("#808080") }
}

// =============================================================================
// HEADER
// =============================================================================

#align(center)[
  #text(size: 16pt, weight: "bold")[Token Cutouts]
  #v(2pt)
  #text(size: 11pt, fill: luma(100))[#map-name]
]

#v(8pt)

// Instructions
#block(
  width: 100%,
  inset: 8pt,
  stroke: 1pt + luma(200),
  radius: 4pt,
  fill: luma(250),
)[
  #text(size: 9pt, weight: "bold")[Assembly Instructions]
  #v(4pt)
  #set text(size: 8pt)
  #set par(leading: 0.5em)

  1. Cut out each token along the outer circle edge
  2. For standees: Fold along the dashed line at the bottom, gluing the back to create a tent shape
  3. Alternatively: Glue to cardboard or use with token bases

  #v(4pt)
  #text(fill: luma(100), style: "italic")[
    Token sizes at 1" = 5ft scale: Tiny (0.5"), Small/Medium (1"), Large (2"), Huge (3"), Gargantuan (4")
  ]
]

#v(12pt)

// =============================================================================
// TOKEN CUTOUTS
// =============================================================================

#if tokens.len() == 0 [
  #align(center)[
    #text(fill: luma(120), style: "italic")[No tokens on this map]
  ]
] else [
  // Group tokens by size for efficient layout
  #let tiny-tokens = tokens.filter(t => {
    let s = lower(get(t, "size", default: "medium"))
    s == "tiny" or s == "t"
  })
  #let small-tokens = tokens.filter(t => {
    let s = lower(get(t, "size", default: "medium"))
    s == "small" or s == "s" or s == "medium" or s == "m"
  })
  #let large-tokens = tokens.filter(t => {
    let s = lower(get(t, "size", default: "medium"))
    s == "large" or s == "l"
  })
  #let huge-tokens = tokens.filter(t => {
    let s = lower(get(t, "size", default: "medium"))
    s == "huge" or s == "h"
  })
  #let garg-tokens = tokens.filter(t => {
    let s = lower(get(t, "size", default: "medium"))
    s == "gargantuan" or s == "g"
  })

  // Render token cutout with standee fold
  #let render-token(token, size-in) = {
    let name = get(token, "name", default: "?")
    let token-type = get(token, "token_type", default: "monster")
    let color = type-to-color(token-type)
    let token-size = size-in * 1in

    // Token with fold area
    box(
      width: token-size + 0.2in,
      height: token-size * 2 + 0.4in,
    )[
      // Top token (front)
      #place(dx: 0.1in, dy: 0.1in)[
        #circle(
          radius: token-size / 2,
          fill: color.lighten(60%),
          stroke: 2pt + color,
        )[
          #align(center + horizon)[
            #text(
              size: if size-in < 1 { 6pt } else if size-in == 1 { 8pt } else { 10pt },
              weight: "bold",
              fill: color.darken(40%),
            )[
              #if name.len() > 8 [
                #name.slice(0, calc.min(6, name.len()))...
              ] else [
                #name
              ]
            ]
          ]
        ]
      ]

      // Fold line
      #place(dx: 0.1in, dy: token-size + 0.15in)[
        #line(
          length: token-size,
          stroke: (dash: "dashed", thickness: 1pt, paint: luma(150)),
        )
      ]

      // Bottom token (back - for folding)
      #place(dx: 0.1in, dy: token-size + 0.2in)[
        #circle(
          radius: token-size / 2,
          fill: luma(240),
          stroke: 1pt + luma(180),
        )[
          #align(center + horizon)[
            #text(size: 7pt, fill: luma(120))[BACK]
          ]
        ]
      ]

      // Cut line indicator
      #place(dx: 0in, dy: token-size * 2 + 0.25in)[
        #align(center)[
          #text(size: 6pt, fill: luma(150))[#name]
        ]
      ]
    ]
  }

  // Render section of tokens
  #let render-section(title, token-list, size-in) = {
    if token-list.len() > 0 [
      #text(size: 10pt, weight: "bold")[#title (#token-list.len())]
      #v(4pt)

      // Calculate how many fit per row based on size
      #let token-width = size-in + 0.3
      #let page-width = 7.5 // printable width in inches
      #let per-row = calc.floor(page-width / token-width)

      #let rows = calc.ceil(token-list.len() / per-row)
      #for row in range(rows) {
        let start = row * per-row
        let end = calc.min(start + per-row, token-list.len())
        let row-tokens = token-list.slice(start, end)

        box[
          #for token in row-tokens {
            render-token(token, size-in)
            h(0.1in)
          }
        ]
        v(0.2in)
      }
      #v(8pt)
    ]
  }

  // Render all sizes
  #render-section("Tiny Tokens (0.5\")", tiny-tokens, 0.5)
  #render-section("Small/Medium Tokens (1\")", small-tokens, 1.0)
  #render-section("Large Tokens (2\")", large-tokens, 2.0)

  // Huge and Gargantuan may need new pages
  #if huge-tokens.len() > 0 [
    #pagebreak()
    #render-section("Huge Tokens (3\")", huge-tokens, 3.0)
  ]

  #if garg-tokens.len() > 0 [
    #pagebreak()
    #render-section("Gargantuan Tokens (4\")", garg-tokens, 4.0)
  ]
]

// Footer
#place(bottom + center)[
  #text(size: 8pt, fill: luma(150))[
    Generated by Mimir DM - Token Cutouts for #map-name
  ]
]
