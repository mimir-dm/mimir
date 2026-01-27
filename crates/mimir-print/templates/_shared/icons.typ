// Mimir Print System - Icons
// Simple text-based icons for class, spell school, and damage types
// These render well in B&W and can be replaced with SVGs later

#import "styles.typ": *

// =============================================================================
// ICON HELPERS
// =============================================================================

/// Base icon wrapper
#let icon-base(content, size: sizes.md) = {
  box(
    inset: 2pt,
    text(size: size, weight: "bold")[#content]
  )
}

/// Circled icon
#let icon-circle(content, size: sizes.md) = {
  box(
    width: size + 6pt,
    height: size + 6pt,
    radius: 50%,
    stroke: 0.5pt + colors.border,
    align(center + horizon)[
      #text(size: size * 0.7, weight: "bold")[#content]
    ]
  )
}

/// Boxed icon
#let icon-box(content, size: sizes.md) = {
  box(
    width: size + 6pt,
    height: size + 6pt,
    radius: 2pt,
    stroke: 0.5pt + colors.border,
    align(center + horizon)[
      #text(size: size * 0.7, weight: "bold")[#content]
    ]
  )
}

// =============================================================================
// CLASS ICONS
// =============================================================================
// Simple letter abbreviations for character classes

#let class-icons = (
  barbarian: "Ba",
  bard: "Bd",
  cleric: "Cl",
  druid: "Dr",
  fighter: "Fi",
  monk: "Mk",
  paladin: "Pa",
  ranger: "Ra",
  rogue: "Ro",
  sorcerer: "So",
  warlock: "Wa",
  wizard: "Wi",
  artificer: "Ar",
)

/// Get class icon by name
#let class-icon(class-name, size: sizes.md) = {
  let key = lower(class-name)
  let abbrev = class-icons.at(key, default: str(class-name).slice(0, 2))
  icon-circle(abbrev, size: size)
}

// =============================================================================
// SPELL SCHOOL ICONS
// =============================================================================
// Two-letter abbreviations for spell schools

#let spell-school-icons = (
  abjuration: "Ab",
  conjuration: "Co",
  divination: "Di",
  enchantment: "En",
  evocation: "Ev",
  illusion: "Il",
  necromancy: "Ne",
  transmutation: "Tr",
)

// Single-letter code to full school name mapping (for Rust enum serialization)
#let spell-school-codes = (
  a: "abjuration",
  c: "conjuration",
  d: "divination",
  e: "enchantment",
  v: "evocation",
  i: "illusion",
  n: "necromancy",
  t: "transmutation",
)

/// Get spell school icon by name or single-letter code
#let spell-school-icon(school, size: sizes.md) = {
  let school-str = str(school)
  // Handle single-letter codes from Rust enum
  let key = if school-str.len() == 1 {
    spell-school-codes.at(lower(school-str), default: school-str)
  } else {
    lower(school-str)
  }
  let abbrev = spell-school-icons.at(key, default: if school-str.len() >= 2 { school-str.slice(0, 2) } else { school-str })
  icon-box(abbrev, size: size)
}

/// Spell school with label
#let spell-school-label(school) = {
  h(spacing.xs)
  spell-school-icon(school, size: sizes.sm)
  h(spacing.xs)
  text(size: sizes.sm, style: "italic")[#school]
}

// =============================================================================
// DAMAGE TYPE ICONS
// =============================================================================
// Simple symbols for damage types

#let damage-type-symbols = (
  acid: "~",
  bludgeoning: "O",
  cold: "*",
  fire: "^",
  force: "!",
  lightning: "/",
  necrotic: "X",
  piercing: "|",
  poison: "%",
  psychic: "@",
  radiant: "+",
  slashing: "-",
  thunder: "#",
)

/// Get damage type icon
#let damage-icon(damage-type, size: sizes.sm) = {
  let key = lower(damage-type)
  let symbol = damage-type-symbols.at(key, default: "?")
  icon-base(symbol, size: size)
}

/// Damage with type indicator
#let damage-with-type(amount, damage-type) = {
  mono-text(amount)
  h(spacing.xs)
  damage-icon(damage-type)
  h(spacing.xs)
  text(size: sizes.xs)[#damage-type]
}

// =============================================================================
// DICE NOTATION
// =============================================================================

/// Format dice notation (e.g., "2d6+3")
#let dice(notation) = {
  mono-text(notation)
}

/// Dice with average
#let dice-with-avg(notation, average) = {
  [#average (#dice(notation))]
}

// =============================================================================
// CONDITION ICONS
// =============================================================================

#let condition-icons = (
  blinded: "Bl",
  charmed: "Ch",
  deafened: "De",
  exhaustion: "Ex",
  frightened: "Fr",
  grappled: "Gr",
  incapacitated: "In",
  invisible: "Iv",
  paralyzed: "Pa",
  petrified: "Pe",
  poisoned: "Po",
  prone: "Pr",
  restrained: "Re",
  stunned: "St",
  unconscious: "Un",
)

/// Get condition icon
#let condition-icon(condition, size: sizes.sm) = {
  let key = lower(condition)
  let abbrev = condition-icons.at(key, default: str(condition).slice(0, 2))
  icon-circle(abbrev, size: size)
}

// =============================================================================
// LEVEL/CR INDICATORS
// =============================================================================

/// Spell level indicator
#let spell-level-indicator(level) = {
  if level == 0 {
    text(size: sizes.xs, weight: "bold")[Cantrip]
  } else {
    text(size: sizes.xs, weight: "bold")[Level #level]
  }
}

/// Challenge rating indicator
#let cr-indicator(cr) = {
  box(
    inset: (x: spacing.sm, y: spacing.xs),
    radius: 2pt,
    fill: colors.background-alt,
    text(size: sizes.sm, weight: "bold")[CR #cr]
  )
}

// =============================================================================
// COMPONENT MARKERS (V, S, M)
// =============================================================================

/// Spell component markers
#let spell-components(verbal: false, somatic: false, material: none) = {
  let parts = ()
  if verbal { parts.push("V") }
  if somatic { parts.push("S") }
  if material != none { parts.push("M") }

  if parts.len() > 0 {
    text(size: sizes.sm, weight: "bold")[#parts.join(", ")]
    if material != none {
      text(size: sizes.xs)[ (#material)]
    }
  }
}
