// Mimir Print System - Shared Styles
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
/// Parameters:
/// - paper: Page size ("us-letter", "a4", or custom dimensions)
/// - margin: Page margins
/// - body: Document content
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
