import type { TokenSize } from '@/types/api'

/**
 * Maps size strings (both abbreviated and full names) to TokenSize values.
 * Used for converting monster/creature sizes from various formats (5etools abbreviated, full names)
 * to the standardized TokenSize type.
 */
export const SIZE_TO_TOKEN_SIZE: Record<string, TokenSize> = {
  // Abbreviated (5etools format)
  't': 'tiny',
  's': 'small',
  'm': 'medium',
  'l': 'large',
  'h': 'huge',
  'g': 'gargantuan',
  // Full names (case-insensitive lookups should lowercase first)
  'tiny': 'tiny',
  'small': 'small',
  'medium': 'medium',
  'large': 'large',
  'huge': 'huge',
  'gargantuan': 'gargantuan'
}

/**
 * Normalizes a size value from various formats (string, array, object) to a lowercase string.
 * Handles the different ways 5etools data can represent size.
 */
export function normalizeSize(size: unknown): string {
  if (!size) return 'm'
  if (typeof size === 'string') return size.toLowerCase()
  if (Array.isArray(size) && size.length > 0) return String(size[0]).toLowerCase()
  return 'm'
}

/**
 * Converts a size value (in any format) to a TokenSize.
 * Returns 'medium' as the default if the size cannot be determined.
 */
export function sizeToTokenSize(size: unknown): TokenSize {
  const normalized = normalizeSize(size)
  return SIZE_TO_TOKEN_SIZE[normalized] || 'medium'
}
