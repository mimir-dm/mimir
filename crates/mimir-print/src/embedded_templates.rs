//! Embedded Typst templates
//!
//! These templates are embedded directly in the binary for stability.
//! No external template files are needed.

/// Shared styles template (colors, typography, page setup)
pub const STYLES_TYP: &str = r##"// Mimir Print System - Shared Styles
// Base typography, page settings, and visual constants for all templates

// =============================================================================
// COLOR PALETTE
// =============================================================================
// Using grayscale for B&W printer friendliness

#let colors = (
  text: black,
  text-secondary: luma(60),
  border: luma(40),
  border-light: luma(80),
  background-alt: luma(245),
  accent: luma(30),
)

// =============================================================================
// TYPOGRAPHY
// =============================================================================

// Font stacks (with fallbacks)
#let font-heading = ("Inter", "Helvetica", "Arial", "sans-serif")
#let font-body = ("Inter", "Helvetica", "Arial", "sans-serif")
#let font-mono = ("JetBrains Mono", "Consolas", "monospace")

// Text size scale
#let sizes = (
  xs: 7pt,
  sm: 8pt,
  base: 9pt,
  md: 10pt,
  lg: 12pt,
  xl: 14pt,
  xxl: 18pt,
  title: 24pt,
)

// =============================================================================
// DOCUMENT SETUP
// =============================================================================

/// Base document wrapper with standard Mimir styling
#let mimir-doc(
  paper: "us-letter",
  margin: 0.5in,
  body
) = {
  set page(
    paper: paper,
    margin: margin,
  )

  set text(
    font: font-body,
    size: sizes.base,
    fill: colors.text,
  )

  set heading(numbering: none)

  set par(
    leading: 0.65em,
    justify: false,
  )

  body
}

/// Card-sized document (poker card: 2.5" x 3.5")
#let mimir-card-doc(body) = {
  set page(
    width: 2.5in,
    height: 3.5in,
    margin: 0.1in,
  )

  set text(
    font: font-body,
    size: sizes.xs,
    fill: colors.text,
  )

  set heading(numbering: none)

  set par(
    leading: 0.5em,
    justify: false,
  )

  body
}

// =============================================================================
// TEXT STYLES
// =============================================================================

/// Large title text
#let title-text(content) = text(
  size: sizes.title,
  weight: "bold",
  font: font-heading,
  content
)

/// Section heading
#let heading-text(content) = text(
  size: sizes.xl,
  weight: "bold",
  font: font-heading,
  content
)

/// Subtitle or secondary heading
#let subtitle-text(content) = text(
  size: sizes.lg,
  style: "italic",
  fill: colors.text-secondary,
  content
)

/// Small label text (for stat labels, etc.)
#let label-text(content) = text(
  size: sizes.xs,
  weight: "bold",
  fill: colors.text-secondary,
  upper(content)
)

/// Value text (for stat values)
#let value-text(content) = text(
  size: sizes.md,
  weight: "bold",
  content
)

/// Small/fine print text
#let small-text(content) = text(
  size: sizes.sm,
  fill: colors.text-secondary,
  content
)

/// Monospace text (for dice notation, etc.)
#let mono-text(content) = text(
  font: font-mono,
  size: sizes.sm,
  content
)

// =============================================================================
// SPACING CONSTANTS
// =============================================================================

#let spacing = (
  xs: 2pt,
  sm: 4pt,
  md: 8pt,
  lg: 12pt,
  xl: 16pt,
  xxl: 24pt,
)

// =============================================================================
// HORIZONTAL RULES
// =============================================================================

/// Standard divider line
#let divider() = line(
  length: 100%,
  stroke: 0.5pt + colors.border-light,
)

/// Heavy divider line
#let divider-heavy() = line(
  length: 100%,
  stroke: 1pt + colors.border,
)

/// Decorative divider with padding
#let section-divider() = {
  v(spacing.md)
  divider-heavy()
  v(spacing.md)
}

// =============================================================================
// TABLE STYLES
// =============================================================================

/// Standard bordered table
#let mimir-table(columns: auto, ..cells) = {
  table(
    columns: columns,
    stroke: 0.5pt + colors.border-light,
    inset: spacing.sm,
    ..cells
  )
}

/// Zebra-striped table (alternating row colors)
#let mimir-table-zebra(columns: auto, header: none, ..rows) = {
  let row-data = rows.pos()

  table(
    columns: columns,
    stroke: none,
    inset: spacing.sm,
    fill: (_, y) => if y == 0 { colors.background-alt } else if calc.rem(y, 2) == 0 { colors.background-alt } else { white },
    ..if header != none { header } else { () },
    ..row-data.flatten()
  )
}

/// Compact table for stat blocks
#let stat-table(..cells) = {
  table(
    columns: (1fr, 1fr),
    stroke: none,
    inset: (x: spacing.sm, y: spacing.xs),
    ..cells
  )
}
"##;

/// Components template (ability scores, stat blocks, cards, layout helpers)
pub const COMPONENTS_TYP: &str = r##"// Mimir Print System - Reusable Components
// Layout components and building blocks for templates

#import "/_shared/styles.typ": *

// =============================================================================
// ABILITY SCORES
// =============================================================================

/// Calculate ability modifier from score
#let ability-mod(score) = {
  let mod = calc.floor((score - 10) / 2)
  if mod >= 0 { "+" + str(mod) } else { str(mod) }
}

/// Single ability score display
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
"##;

/// Icons template (class icons, spell school icons, damage type icons)
pub const ICONS_TYP: &str = r##"// Mimir Print System - Icons
// Simple text-based icons for class, spell school, and damage types

#import "/_shared/styles.typ": *

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
  let key = if school-str.len() == 1 {
    spell-school-codes.at(lower(school-str), default: school-str)
  } else {
    lower(school-str)
  }
  let abbrev = spell-school-icons.at(key, default: if school-str.len() >= 2 { school-str.slice(0, 2) } else { school-str })
  icon-box(abbrev, size: size)
}

// =============================================================================
// DAMAGE TYPE ICONS
// =============================================================================

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

// =============================================================================
// DICE NOTATION
// =============================================================================

/// Format dice notation (e.g., "2d6+3")
#let dice(notation) = {
  mono-text(notation)
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
"##;

/// Get embedded template content by path
/// Returns None if path is not a known embedded template
pub fn get_embedded_template(path: &str) -> Option<&'static str> {
    // Normalize the path - strip leading slash and handle variations
    let normalized = path.trim_start_matches('/');

    match normalized {
        "_shared/styles.typ" => Some(STYLES_TYP),
        "_shared/components.typ" => Some(COMPONENTS_TYP),
        "_shared/icons.typ" => Some(ICONS_TYP),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_embedded_template() {
        assert!(get_embedded_template("/_shared/styles.typ").is_some());
        assert!(get_embedded_template("_shared/styles.typ").is_some());
        assert!(get_embedded_template("/_shared/components.typ").is_some());
        assert!(get_embedded_template("/_shared/icons.typ").is_some());
        assert!(get_embedded_template("nonexistent.typ").is_none());
    }

    #[test]
    fn test_styles_contains_colors() {
        assert!(STYLES_TYP.contains("#let colors = ("));
        assert!(STYLES_TYP.contains("#let sizes = ("));
        assert!(STYLES_TYP.contains("#let mimir-doc"));
    }

    #[test]
    fn test_components_imports_styles() {
        assert!(COMPONENTS_TYP.contains("#import \"/_shared/styles.typ\": *"));
    }

    #[test]
    fn test_icons_imports_styles() {
        assert!(ICONS_TYP.contains("#import \"/_shared/styles.typ\": *"));
    }
}
