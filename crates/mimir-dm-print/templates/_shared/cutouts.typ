// Shared token cutout rendering functions
// Import into map templates to add cutout sheets

// Helper for safe property access
#let get(obj, key, default: none) = {
  if obj != none and key in obj { obj.at(key) } else { default }
}

// Size in grid squares (and inches at 1"=5ft scale)
#let size-to-inches(size) = {
  let s = lower(str(size))
  if s == "tiny" or s == "t" { 0.5 }
  else if s == "small" or s == "s" { 1.0 }
  else if s == "medium" or s == "m" { 1.0 }
  else if s == "large" or s == "l" { 2.0 }
  else if s == "huge" or s == "h" { 3.0 }
  else if s == "gargantuan" or s == "g" { 4.0 }
  else { 1.0 }
}

// Check if token matches a size category
#let is-tiny(token) = {
  let s = lower(str(get(token, "size", default: "medium")))
  s == "tiny" or s == "t"
}

#let is-small-or-medium(token) = {
  let s = lower(str(get(token, "size", default: "medium")))
  s == "small" or s == "s" or s == "medium" or s == "m"
}

#let is-large(token) = {
  let s = lower(str(get(token, "size", default: "medium")))
  s == "large" or s == "l"
}

#let is-huge(token) = {
  let s = lower(str(get(token, "size", default: "medium")))
  s == "huge" or s == "h"
}

#let is-gargantuan(token) = {
  let s = lower(str(get(token, "size", default: "medium")))
  s == "gargantuan" or s == "g"
}

// Token type to color
#let type-to-color(token-type) = {
  let t = lower(str(token-type))
  if t == "monster" { rgb("#dc3545") }
  else if t == "pc" { rgb("#28a745") }
  else if t == "npc" { rgb("#007bff") }
  else if t == "trap" { rgb("#ffc107") }
  else if t == "marker" { rgb("#6c757d") }
  else { rgb("#808080") }
}

// Render a single token cutout with standee fold
#let render-token(token, size-in) = {
  let name = get(token, "name", default: "?")
  let token-type = get(token, "token_type", default: "monster")
  let img-path = get(token, "image_path", default: none)
  let color = type-to-color(token-type)
  let token-size = size-in * 1in
  let font-size = if size-in < 1 { 6pt } else if size-in == 1 { 8pt } else { 10pt }
  let display-name = if name.len() > 8 { name.slice(0, 6) + ".." } else { name }

  box(
    width: token-size + 0.2in,
    height: token-size * 2 + 0.5in,
  )[
    // Front of token - image if available, colored circle otherwise
    #place(dx: 0.1in, dy: 0.1in)[
      #if img-path != none [
        #box(
          width: token-size,
          height: token-size,
          clip: true,
          radius: token-size / 2,
          stroke: 2pt + color,
        )[
          #image(img-path, width: 100%, height: 100%, fit: "cover")
        ]
      ] else [
        #circle(
          radius: token-size / 2,
          fill: color.lighten(60%),
          stroke: 2pt + color,
        )[
          #align(center + horizon)[
            #text(size: font-size, weight: "bold", fill: color.darken(40%))[#display-name]
          ]
        ]
      ]
    ]
    // Fold line
    #place(dx: 0.1in, dy: token-size + 0.15in)[
      #line(length: token-size, stroke: (dash: "dashed", thickness: 1pt, paint: luma(150)))
    ]
    // Back of token - always show name
    #place(dx: 0.1in, dy: token-size + 0.2in)[
      #circle(
        radius: token-size / 2,
        fill: color.lighten(80%),
        stroke: 1pt + color,
      )[
        #align(center + horizon)[
          #text(size: font-size, weight: "bold", fill: color.darken(20%))[#display-name]
        ]
      ]
    ]
    // Label below
    #place(dx: 0in, dy: token-size * 2 + 0.3in)[
      #box(width: token-size + 0.2in)[
        #align(center)[
          #text(size: 6pt, fill: luma(100))[#name]
        ]
      ]
    ]
  ]
}

// Render a section of tokens with a title
#let render-token-section(title, token-list, size-in) = {
  if token-list.len() > 0 {
    let token-width = size-in + 0.3
    let page-width = 7.5
    let per-row = calc.max(1, calc.floor(page-width / token-width))
    let rows = calc.ceil(token-list.len() / per-row)

    [
      #text(size: 10pt, weight: "bold")[#title (#token-list.len())]
      #v(4pt)
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
}

// Main function to render all cutout pages
#let render-cutout-pages(map-name, tokens) = {
  if tokens.len() > 0 {
    // Group tokens by size using filter
    let tiny-tokens = tokens.filter(is-tiny)
    let small-tokens = tokens.filter(is-small-or-medium)
    let large-tokens = tokens.filter(is-large)
    let huge-tokens = tokens.filter(is-huge)
    let garg-tokens = tokens.filter(is-gargantuan)

    [
      #pagebreak()

      #align(center)[
        #text(size: 14pt, weight: "bold")[Token Cutouts]
        #v(2pt)
        #text(size: 10pt, fill: luma(100))[#map-name]
      ]

      #v(8pt)

      #block(
        width: 100%,
        inset: 8pt,
        stroke: 1pt + luma(200),
        radius: 4pt,
        fill: luma(250),
      )[
        #text(size: 8pt, weight: "bold")[Assembly]
        #h(8pt)
        #text(size: 7pt)[
          Cut circles, fold on dashed line, glue backs together for standees.
        ]
      ]

      #v(12pt)

      #render-token-section("Tiny (0.5\")", tiny-tokens, 0.5)
      #render-token-section("Small/Medium (1\")", small-tokens, 1.0)
      #render-token-section("Large (2\")", large-tokens, 2.0)

      #if huge-tokens.len() > 0 {
        pagebreak()
        render-token-section("Huge (3\")", huge-tokens, 3.0)
      }

      #if garg-tokens.len() > 0 {
        pagebreak()
        render-token-section("Gargantuan (4\")", garg-tokens, 4.0)
      }
    ]
  }
}
