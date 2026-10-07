/**
 * Combat tracker types (COLLIERY-I-0468). Mirror mimir-core's CombatService
 * results (services/combat.rs).
 */

export interface CombatSession {
  id: string
  module_id: string
  round: number
  turn_index: number
  status: 'active' | 'ended'
  created_at: string
  updated_at: string
}

export interface ActiveCondition {
  name: string
  /** Last round the condition applies; null lasts until removed. */
  expires_round: number | null
}

export interface HpChange {
  kind: 'damage' | 'heal'
  amount: number
  round: number
}

export type CombatSourceKind = 'module_monster' | 'module_npc' | 'character' | 'custom'

export interface CombatEntry {
  id: string
  source_kind: CombatSourceKind
  source_id: string | null
  token_id: string | null
  display_name: string
  initiative: number | null
  dex_modifier: number | null
  max_hp: number | null
  current_hp: number | null
  temp_hp: number
  is_concentrating: boolean
  conditions: ActiveCondition[]
  damage_log: HpChange[]
}

export interface CombatState {
  session: CombatSession
  /** In turn order. */
  entries: CombatEntry[]
  current_entry_id: string | null
}

export interface DamageResult {
  entry: CombatEntry
  concentration_dc: number | null
}

/**
 * The SRD conditions the backend accepts (mirror of SRD_CONDITIONS in
 * mimir-core services/combat.rs, which validates them).
 */
export const SRD_CONDITIONS = [
  'blinded',
  'charmed',
  'deafened',
  'exhaustion',
  'frightened',
  'grappled',
  'incapacitated',
  'invisible',
  'paralyzed',
  'petrified',
  'poisoned',
  'prone',
  'restrained',
  'stunned',
  'unconscious',
] as const
