// Re-export all existing types from their current locations
export * from './api'

// Domain types (re-exports from api.ts for backwards compatibility)
export * from './domain'

// Character types (full character management system)
export * from './character'

// Theme constants
export const THEMES = {
  LIGHT: 'light',
  DARK: 'dark',
  HYPER: 'hyper'
} as const

export type Theme = typeof THEMES[keyof typeof THEMES]

// Entity type for polymorphic components
export type EntityType = 'campaign' | 'module'

// Common entity interface for abstraction
export interface BaseEntity {
  id: string
  name: string
  created_at: string
}

// Icon paths map for themes
export interface ThemeIcons {
  edit: string
  locked: string
  gear?: string
  mimir?: string
  new?: string
}

export type IconMap = Record<Theme, ThemeIcons>
