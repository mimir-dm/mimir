// API Response types
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// Theme types
export interface Theme {
  id: string
  name: string
  description: string
}

// =============================================================================
// Campaign types - auto-generated from Rust via ts-rs
// =============================================================================
export type { Campaign } from './generated/Campaign'

/** Request for creating a new campaign */
export interface CreateCampaignRequest {
  name: string
  description?: string
}

/** Request for updating a campaign */
export interface UpdateCampaignRequest {
  name?: string
  description?: string | null
}

// =============================================================================
// Campaign Archive types - matches mimir-core archive service
// =============================================================================

/** Counts of entities in an archive */
export interface ArchiveCounts {
  modules: number
  documents: number
  characters: number
  maps: number
  tokens: number
  module_monsters: number
  module_npcs: number
  assets: number
}

/** A reference to a catalog item found in campaign content */
export interface CatalogReference {
  type: string
  name: string
  source: string
}

/** Preview information about an archive without importing */
export interface ArchivePreview {
  campaign_name: string
  counts: ArchiveCounts
  catalog_references: CatalogReference[]
  mimir_version: string
  created_at: string
  archive_version: string
}

// =============================================================================
// Module types - auto-generated from Rust via ts-rs
// =============================================================================
export type { Module } from './generated/Module'

/** Request for creating a new module */
export interface CreateModuleRequest {
  campaign_id: string
  name: string
  description?: string
  module_number?: number
}

/** Request for updating a module */
export interface UpdateModuleRequest {
  name?: string
  description?: string | null
  module_number?: number
}

// =============================================================================
// Document types - auto-generated from Rust via ts-rs
// =============================================================================
export type { Document } from './generated/Document'

/** Request for creating a new document */
export interface CreateDocumentRequest {
  campaign_id: string
  module_id?: string
  title: string
  content?: string
  doc_type: string
}

/** Request for updating a document */
export interface UpdateDocumentRequest {
  title?: string
  content?: string
  doc_type?: string
  module_id?: string | null
}

// Character types moved to ./character.ts
// (Placeholder removed - now using full character management system)

// Token types for Visual Display System
export type TokenType = 'monster' | 'pc' | 'npc' | 'trap' | 'marker'
export type TokenSize = 'tiny' | 'small' | 'medium' | 'large' | 'huge' | 'gargantuan'
export type VisionType = 'normal' | 'darkvision' | 'blindsight' | 'tremorsense' | 'truesight' | 'devils_sight'

export interface Token {
  id: string
  map_id: string
  name: string
  token_type: TokenType
  size: TokenSize
  x: number
  y: number
  visible_to_players: boolean
  color: string | null
  image_path: string | null
  monster_id: string | null
  character_id: string | null
  notes: string | null
  vision_type: string
  vision_range_ft: number | null
  // New D&D 5e vision fields
  vision_bright_ft: number | null  // Vision range in bright light (null = unlimited)
  vision_dim_ft: number | null     // Vision range in dim light (null = unlimited)
  vision_dark_ft: number           // Vision range in darkness (0 = blind, 60 = darkvision)
  light_radius_ft: number          // Light source dim radius (bright = half)
  created_at: string
  updated_at: string
}

export interface TokenSummary extends Token {
  monster_name: string | null
  character_name: string | null
}

export interface CreateTokenRequest {
  map_id: string
  name: string
  token_type: string
  size: string
  x: number
  y: number
  visible_to_players?: boolean
  color?: string
  image_path?: string
  monster_id?: string
  character_id?: string
  notes?: string
  vision_type?: VisionType
  vision_range_ft?: number | null
}

/**
 * Extended token config used by frontend for tracking monster info.
 * The monster_name/monster_source fields are used to auto-add to module_monsters.
 */
export interface TokenConfigWithMonster extends CreateTokenRequest {
  monster_name?: string
  monster_source?: string
}

export interface UpdateTokenRequest {
  name?: string
  token_type?: string
  size?: string
  x?: number
  y?: number
  visible_to_players?: boolean
  color?: string | null
  notes?: string | null
  vision_type?: VisionType
  vision_range_ft?: number | null
}

// Token size to grid squares mapping (D&D 5e)
export const TOKEN_SIZE_GRID_SQUARES: Record<TokenSize, number> = {
  tiny: 0.5,
  small: 1,
  medium: 1,
  large: 2,
  huge: 3,
  gargantuan: 4
}

// Token type colors (fallback when no image)
export const TOKEN_TYPE_COLORS: Record<TokenType, string> = {
  monster: '#dc2626', // red
  pc: '#16a34a',      // green
  npc: '#2563eb',     // blue
  trap: '#ea580c',    // orange
  marker: '#9333ea'   // purple
}

// Vision type labels for UI display
export const VISION_TYPE_LABELS: Record<VisionType, string> = {
  normal: 'Normal Vision',
  darkvision: 'Darkvision',
  blindsight: 'Blindsight',
  tremorsense: 'Tremorsense',
  truesight: 'Truesight',
  devils_sight: "Devil's Sight"
}

// Common vision presets for quick selection
export const VISION_PRESETS: { label: string; type: VisionType; range: number | null }[] = [
  { label: 'Normal Vision', type: 'normal', range: null },
  { label: 'Darkvision 60 ft.', type: 'darkvision', range: 60 },
  { label: 'Darkvision 120 ft.', type: 'darkvision', range: 120 },
  { label: 'Blindsight 30 ft.', type: 'blindsight', range: 30 },
  { label: 'Blindsight 60 ft.', type: 'blindsight', range: 60 },
  { label: 'Tremorsense 60 ft.', type: 'tremorsense', range: 60 },
  { label: 'Truesight 120 ft.', type: 'truesight', range: 120 },
  { label: "Devil's Sight 120 ft.", type: 'devils_sight', range: 120 }
]