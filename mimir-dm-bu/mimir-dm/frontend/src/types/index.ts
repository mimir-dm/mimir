// Re-export all existing types from their current locations
export * from './api'
// Selectively export from domain to avoid conflicts with api.ts
export type {
  Document,
  ExitCriterion,
  BoardConfig,
  BoardColumn,
  BoardTransition,
  BoardDocumentType,
  Stage,
  StageInfo,
  DocumentTemplate
} from './domain'
// Campaign, Module, and Session are already exported from api.ts

// Character and player types (full character management system)
export * from './character'

// Additional shared types specific to this app
import type { DocumentTemplate } from './domain'

// Board stage type (extends the base board config)
export interface BoardStage {
  key: string
  display_name: string
  templates?: DocumentTemplate[]
  required_documents?: string[]
  optional_documents?: string[]
  no_completion_required_documents?: string[]
}

// Theme constants (these are actually static in the app)
export const THEMES = {
  LIGHT: 'light',
  DARK: 'dark',
  HYPER: 'hyper'
} as const

export type Theme = typeof THEMES[keyof typeof THEMES]

// Entity type for polymorphic components
export type EntityType = 'campaign' | 'module' | 'session'

// Stage progress data structure
export interface StageDocuments {
  documents: DocumentTemplate[]
  completed: number
  total: number
  percentage: number
}

// Common board entity interface for abstraction
export interface BoardEntity {
  id: number
  name: string
  status: string
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