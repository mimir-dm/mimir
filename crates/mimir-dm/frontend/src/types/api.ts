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

// Campaign types
export interface Campaign {
  id: number
  name: string
  status: string
  directory_path: string
  created_at: string
  session_zero_date?: string
  first_session_date?: string
  last_activity_at: string
  archived_at?: string
}

export interface NewCampaign {
  name: string
  description?: string
  directory_location: string
}

// Module types
export interface Module {
  id: number
  campaign_id: number
  name: string
  module_number: number
  description?: string
  status: string
  sessions_planned: number
  sessions_completed: number
  expected_sessions?: number
  actual_sessions?: number
  created_at: string
}

// Session types
export interface Session {
  id: number
  module_id: number
  session_number: number
  name: string
  status: string
  scheduled_date?: string
  actual_date?: string
  notes?: string
  created_at: string
}

// Workflow card types
export interface WorkflowCard {
  id: number
  board_type: 'campaign' | 'module' | 'session'
  board_id: number
  title: string
  description?: string
  status: string
  position: number
  due_date?: string
  created_at: string
  updated_at: string
}

// Template types
export interface Template {
  document_id: string
  version_number: number
  document_content: string
  document_type?: string
  document_level?: string
  purpose?: string
  is_active: boolean
}

export interface RenderTemplateRequest {
  template_id: string
  variables: Record<string, any>
}

// Character types moved to ./character.ts
// (Placeholder removed - now using full character management system)

// Token types for Visual Display System
export type TokenType = 'monster' | 'pc' | 'npc' | 'trap' | 'marker'
export type TokenSize = 'tiny' | 'small' | 'medium' | 'large' | 'huge' | 'gargantuan'
export type VisionType = 'normal' | 'darkvision' | 'blindsight' | 'tremorsense' | 'truesight' | 'devils_sight'

export interface Token {
  id: number
  map_id: number
  name: string
  token_type: TokenType
  size: TokenSize
  x: number
  y: number
  visible_to_players: boolean
  color: string | null
  image_path: string | null
  monster_id: number | null
  character_id: number | null
  notes: string | null
  vision_type: string
  vision_range_ft: number | null
  created_at: string
  updated_at: string
}

export interface TokenSummary extends Token {
  monster_name: string | null
  character_name: string | null
}

export interface CreateTokenRequest {
  map_id: number
  name: string
  token_type: string
  size: string
  x: number
  y: number
  visible_to_players?: boolean
  color?: string
  image_path?: string
  monster_id?: number
  character_id?: number
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