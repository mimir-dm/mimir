// Book/Source-related type definitions

export interface BookInfo {
  /** Source code (e.g., "PHB", "MM") - acts as unique ID */
  id: string
  /** Display name (e.g., "Player's Handbook") */
  name: string
  /** Whether this source is enabled */
  enabled: boolean
  /** ISO 8601 timestamp of when imported */
  imported_at: string
}

export interface ImportResponse {
  /** Number of sources successfully imported */
  sources_imported: number
  /** Number of sources that failed to import */
  sources_failed: number
  /** Total entities imported */
  total_entities: number
  /** Summary message */
  message: string
}

export interface BookContent {
  success: boolean
  data: BookSection[] | null
  message?: string
}

export interface BookSection {
  type?: string
  name?: string
  entries?: BookEntry[]
  page?: number
  source?: string
  id?: string
  [key: string]: any // Allow for additional properties
}

export type BookEntry = string | ComplexEntry

export interface ComplexEntry {
  type: string
  name?: string
  entries?: BookEntry[]
  items?: any[]
  page?: number
  source?: string
  id?: string
  caption?: string
  colLabels?: string[]
  colStyles?: string[]
  rows?: any[][]
  href?: {
    path?: string
    type?: string
  }
  title?: string
  style?: string
  by?: string
  roll?: {
    min?: number
    max?: number
  }
  [key: string]: any
}

export interface SubEntry {
  id: string
  name: string
  level: number
  children?: SubEntry[]
}