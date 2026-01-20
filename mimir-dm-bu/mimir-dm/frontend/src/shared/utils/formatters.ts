/**
 * Shared formatter utilities for consistent data formatting across the application
 *
 * This module consolidates formatter functions that were previously duplicated
 * across multiple files. Use these functions for consistent formatting behavior.
 */

// ============================================================================
// Spell Formatters
// ============================================================================

/**
 * Format spell level as ordinal string (e.g., "Cantrip", "1st", "2nd")
 * @param level - The spell level (0-9)
 * @returns Formatted spell level string
 */
export function formatSpellLevel(level: number): string {
  if (level === 0) return 'Cantrip'
  if (level === 1) return '1st'
  if (level === 2) return '2nd'
  if (level === 3) return '3rd'
  return `${level}th`
}

/**
 * Extract spell tags (Concentration, Ritual) from spell data
 * @param spell - Spell data object
 * @returns Array of tag strings
 */
export function formatSpellTags(spell: any): string[] {
  const tags = []
  if (spell.concentration) tags.push('Concentration')
  if (spell.ritual) tags.push('Ritual')
  return tags
}

// ============================================================================
// Monster/Creature Formatters
// ============================================================================

/**
 * Format Challenge Rating (CR) with robust type handling
 * Handles string, number, object, and null/undefined values
 * @param cr - CR value in various formats
 * @returns Formatted CR string or em dash if invalid
 */
export function formatCR(cr: any): string {
  if (!cr) return '—'
  if (typeof cr === 'string') return cr
  if (typeof cr === 'number') return cr.toString()
  if (cr.cr) return cr.cr
  return '—'
}

// ============================================================================
// Item Formatters
// ============================================================================

/**
 * Format weight with appropriate handling of null/zero values
 * @param weight - Weight value in pounds (or null)
 * @returns Formatted weight string with "lb" unit or em dash
 */
export function formatWeight(weight: number | null): string {
  if (!weight || weight === 0) return '—'
  return `${weight} lb`
}
