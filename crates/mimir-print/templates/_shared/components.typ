// Mimir Print System - Reusable Components
// Layout components and building blocks for templates

#import "styles.typ": *

// =============================================================================
// ABILITY SCORES
// =============================================================================

/// Calculate ability modifier from score
#let ability-mod(score) = {
  let mod = calc.floor((score - 10) / 2)
  if mod >= 0 { "+" + str(mod) } else { str(mod) }
}

/// Single ability score display
/// Shows the ability name, score, and modifier
#let ability-box(name, score) = {
  box(
    width: 100%,
    stroke: 0.5pt + colors.border-light,
    inset: spacing.sm,
    radius: 2pt,
    {
      align(center)[
        #label-text(name)
        #v(spacing.xs)
        #text(size: sizes.lg, weight: "bold")[#score]
        #v(spacing.xs)
        #text(size: sizes.sm, fill: colors.text-secondary)[#ability-mod(score)]
      ]
    }
  )
}

/// Full ability score block (6 abilities in 2x3 or 6x1 grid)
/// Parameters:
/// - str, dex, con, int, wis, cha: Ability scores
/// - layout: "grid" (2x3) or "row" (6x1)
#let ability-scores(
  str: 10,
  dex: 10,
  con: 10,
  int: 10,
  wis: 10,
  cha: 10,
  layout: "grid"
) = {
  let abilities = (
    ("STR", str),
    ("DEX", dex),
    ("CON", con),
    ("INT", int),
    ("WIS", wis),
    ("CHA", cha),
  )

  if layout == "row" {
    grid(
      columns: (1fr,) * 6,
      column-gutter: spacing.sm,
      ..abilities.map(a => ability-box(a.at(0), a.at(1)))
    )
  } else {
    // 2x3 grid layout
    grid(
      columns: (1fr, 1fr, 1fr),
      rows: (auto, auto),
      column-gutter: spacing.sm,
      row-gutter: spacing.sm,
      ..abilities.map(a => ability-box(a.at(0), a.at(1)))
    )
  }
}

// =============================================================================
// STAT BLOCKS (Monster/NPC Format)
// =============================================================================

/// Monster stat block header
#let stat-block-header(name, size-type, alignment: none) = {
  block(
    width: 100%,
    fill: colors.background-alt,
    inset: spacing.md,
    below: spacing.sm,
    {
      title-text(name)
      v(spacing.xs)
      text(style: "italic")[
        #size-type#if alignment != none [, #alignment]
      ]
    }
  )
}

/// Stat block property line (e.g., "Armor Class 15 (natural armor)")
#let property-line(label, value) = {
  text(weight: "bold")[#label ]
  value
  linebreak()
}

/// Full monster stat block
/// Parameters match D&D 5e stat block format
#let stat-block(
  name: "Monster",
  size: "Medium",
  type: "humanoid",
  alignment: "neutral",
  ac: "10",
  hp: "10 (2d8)",
  speed: "30 ft.",
  str: 10,
  dex: 10,
  con: 10,
  int: 10,
  wis: 10,
  cha: 10,
  saves: none,
  skills: none,
  damage-vulnerabilities: none,
  damage-resistances: none,
  damage-immunities: none,
  condition-immunities: none,
  senses: "passive Perception 10",
  languages: "Common",
  cr: "0",
  traits: (),
  actions: (),
  reactions: (),
  legendary-actions: (),
) = {
  box(
    width: 100%,
    stroke: (
      top: 2pt + colors.accent,
      bottom: 2pt + colors.accent,
      left: none,
      right: none,
    ),
    inset: 0pt,
    {
      // Header
      stat-block-header(name, [#size #type], alignment: alignment)

      // Basic stats
      block(
        inset: (x: spacing.md, y: spacing.sm),
        {
          property-line("Armor Class", ac)
          property-line("Hit Points", hp)
          property-line("Speed", speed)
        }
      )

      divider-heavy()

      // Ability scores
      block(inset: spacing.md)[
        #ability-scores(
          str: str,
          dex: dex,
          con: con,
          int: int,
          wis: wis,
          cha: cha,
          layout: "row"
        )
      ]

      divider-heavy()

      // Secondary stats
      block(
        inset: (x: spacing.md, y: spacing.sm),
        {
          if saves != none { property-line("Saving Throws", saves) }
          if skills != none { property-line("Skills", skills) }
          if damage-vulnerabilities != none { property-line("Damage Vulnerabilities", damage-vulnerabilities) }
          if damage-resistances != none { property-line("Damage Resistances", damage-resistances) }
          if damage-immunities != none { property-line("Damage Immunities", damage-immunities) }
          if condition-immunities != none { property-line("Condition Immunities", condition-immunities) }
          property-line("Senses", senses)
          property-line("Languages", languages)
          property-line("Challenge", cr)
        }
      )

      divider-heavy()

      // Traits
      if traits.len() > 0 {
        block(inset: spacing.md)[
          #for trait in traits {
            text(weight: "bold", style: "italic")[#trait.name. ]
            trait.description
            v(spacing.sm)
          }
        ]
      }

      // Actions
      if actions.len() > 0 {
        block(inset: spacing.md)[
          #heading-text("Actions")
          #divider()
          #v(spacing.sm)
          #for action in actions {
            text(weight: "bold", style: "italic")[#action.name. ]
            action.description
            v(spacing.sm)
          }
        ]
      }

      // Reactions
      if reactions.len() > 0 {
        block(inset: spacing.md)[
          #heading-text("Reactions")
          #divider()
          #v(spacing.sm)
          #for reaction in reactions {
            text(weight: "bold", style: "italic")[#reaction.name. ]
            reaction.description
            v(spacing.sm)
          }
        ]
      }

      // Legendary Actions
      if legendary-actions.len() > 0 {
        block(inset: spacing.md)[
          #heading-text("Legendary Actions")
          #divider()
          #v(spacing.sm)
          #for action in legendary-actions {
            text(weight: "bold", style: "italic")[#action.name. ]
            action.description
            v(spacing.sm)
          }
        ]
      }
    }
  )
}

// =============================================================================
// CARDS
// =============================================================================

/// Generic card frame for spell cards, NPC cards, etc.
/// Designed for poker card size (2.5" x 3.5")
#let card(
  title: "Card Title",
  subtitle: none,
  category: none,
  body,
  footer: none,
) = {
  box(
    width: 100%,
    height: 100%,
    stroke: 1pt + colors.border,
    radius: 4pt,
    clip: true,
    {
      // Header
      block(
        width: 100%,
        fill: colors.background-alt,
        inset: spacing.sm,
        {
          if category != none {
            align(right)[
              #text(size: sizes.xs, fill: colors.text-secondary)[#category]
            ]
          }
          text(size: sizes.md, weight: "bold")[#title]
          if subtitle != none {
            linebreak()
            text(size: sizes.xs, style: "italic")[#subtitle]
          }
        }
      )

      // Body
      block(
        width: 100%,
        inset: spacing.sm,
        body
      )

      // Footer (positioned at bottom)
      if footer != none {
        place(
          bottom + left,
          block(
            width: 100%,
            fill: colors.background-alt,
            inset: spacing.xs,
            text(size: sizes.xs, fill: colors.text-secondary)[#footer]
          )
        )
      }
    }
  )
}

// =============================================================================
// LAYOUT HELPERS
// =============================================================================

/// Two-column layout
#let two-columns(left, right, gutter: spacing.md) = {
  grid(
    columns: (1fr, 1fr),
    column-gutter: gutter,
    left,
    right,
  )
}

/// Three-column layout
#let three-columns(a, b, c, gutter: spacing.md) = {
  grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: gutter,
    a, b, c,
  )
}

/// Labeled value display (label above value)
#let labeled-value(label, value) = {
  stack(
    dir: ttb,
    spacing: spacing.xs,
    label-text(label),
    value-text(value),
  )
}

/// Inline labeled value (label: value)
#let inline-labeled(label, value) = {
  text(weight: "bold")[#label: ]
  value
}

/// Info box with optional title
#let info-box(title: none, body) = {
  block(
    width: 100%,
    stroke: 0.5pt + colors.border-light,
    inset: spacing.md,
    radius: 2pt,
    {
      if title != none {
        text(weight: "bold")[#title]
        v(spacing.sm)
      }
      body
    }
  )
}

/// Highlighted/important box
#let highlight-box(body) = {
  block(
    width: 100%,
    fill: colors.background-alt,
    inset: spacing.md,
    radius: 2pt,
    body
  )
}

// =============================================================================
// COMPACT MONSTER CARD (Inline format for documents)
// =============================================================================

/// Compact monster card for inline use in documents
/// Shows key combat stats in a condensed format
#let monster-card-inline(
  name: "Monster",
  size: "Medium",
  type: "humanoid",
  alignment: "neutral",
  ac: "10",
  hp: "10",
  speed: "30 ft.",
  str: 10,
  dex: 10,
  con: 10,
  int: 10,
  wis: 10,
  cha: 10,
  cr: "0",
  quantity: 1,
  encounter-tag: none,
  actions: (),
) = {
  box(
    width: 100%,
    stroke: 1pt + colors.border,
    radius: 4pt,
    inset: 0pt,
    {
      // Header with name and CR
      block(
        width: 100%,
        fill: colors.background-alt,
        inset: (x: spacing.sm, y: spacing.xs),
      )[
        #grid(
          columns: (1fr, auto),
          [
            #text(size: sizes.base, weight: "bold")[#name]
            #if quantity > 1 [ #text(size: sizes.sm, fill: colors.text-secondary)[(x#quantity)]]
            #linebreak()
            #text(size: sizes.xs, fill: colors.text-secondary, style: "italic")[#size #type, #alignment]
            #if encounter-tag != none [
              #h(spacing.sm)
              #text(size: sizes.xs, fill: colors.accent)[#encounter-tag]
            ]
          ],
          align(right + horizon)[
            #box(
              fill: colors.background-alt,
              stroke: 1pt + colors.border,
              inset: spacing.xs,
              radius: 2pt,
            )[
              #text(size: sizes.xs)[CR]
              #text(size: sizes.sm, weight: "bold")[ #cr]
            ]
          ]
        )
      ]

      // Combat stats row
      block(
        width: 100%,
        inset: spacing.xs,
        stroke: (bottom: 0.5pt + colors.border-light),
      )[
        #set text(size: sizes.sm)
        #grid(
          columns: (1fr, 1fr, 1fr),
          column-gutter: spacing.sm,
          [*AC* #ac],
          [*HP* #hp],
          [*Speed* #speed],
        )
      ]

      // Ability scores (compact single row)
      block(
        width: 100%,
        inset: (x: spacing.xs, y: spacing.xs),
        stroke: (bottom: if actions.len() > 0 { 0.5pt + colors.border-light } else { none }),
      )[
        #set text(size: sizes.xs)
        #grid(
          columns: (1fr,) * 6,
          align(center)[*STR* #ability-mod(str)],
          align(center)[*DEX* #ability-mod(dex)],
          align(center)[*CON* #ability-mod(con)],
          align(center)[*INT* #ability-mod(int)],
          align(center)[*WIS* #ability-mod(wis)],
          align(center)[*CHA* #ability-mod(cha)],
        )
      ]

      // Key actions (first 3, condensed)
      if actions.len() > 0 {
        block(
          width: 100%,
          inset: spacing.xs,
        )[
          #set text(size: sizes.xs)
          #for (i, action) in actions.slice(0, calc.min(3, actions.len())).enumerate() {
            if action.name != none [
              #text(weight: "bold", style: "italic")[#action.name.]
              #text(fill: colors.text-secondary)[ #action.description.codepoints().slice(0, calc.min(150, action.description.codepoints().len())).join("")#if action.description.codepoints().len() > 150 [...]]
              #if i < calc.min(3, actions.len()) - 1 [ #linebreak() ]
            ]
          }
        ]
      }
    }
  )
}
