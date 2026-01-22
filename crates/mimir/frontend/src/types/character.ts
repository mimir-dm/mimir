/**
 * TypeScript type definitions for character management
 *
 * These types match the Rust structs used in mimir-core and Tauri commands.
 */

// =============================================================================
// Character Core Types - matches mimir-core Character model
// =============================================================================

/**
 * A character - either a player character or NPC in a campaign.
 * Matches the mimir-core Character struct.
 */
export interface Character {
  /** Unique character ID (UUID) */
  id: string
  /** Campaign this character belongs to */
  campaign_id: string
  /** Character name */
  name: string
  /** Whether this is an NPC (1) or PC (0) */
  is_npc: number
  /** Player name (for PCs) */
  player_name: string | null

  // Race and background (catalog references)
  /** Race name (e.g., "Elf", "Human") */
  race_name: string | null
  /** Race source (e.g., "PHB", "VGtM") */
  race_source: string | null
  /** Background name (e.g., "Acolyte", "Criminal") */
  background_name: string | null
  /** Background source (e.g., "PHB") */
  background_source: string | null

  // Ability scores
  strength: number
  dexterity: number
  constitution: number
  intelligence: number
  wisdom: number
  charisma: number

  // Currency
  cp: number
  sp: number
  ep: number
  gp: number
  pp: number

  // Roleplay elements
  traits: string | null
  ideals: string | null
  bonds: string | null
  flaws: string | null

  // NPC-specific fields
  role: string | null
  location: string | null
  faction: string | null

  /** ISO8601 timestamp of creation */
  created_at: string
  /** ISO8601 timestamp of last update */
  updated_at: string
}

// =============================================================================
// Character Inventory - matches mimir-core CharacterInventory model
// =============================================================================

/**
 * An item in a character's inventory.
 */
export interface CharacterInventory {
  /** Unique ID (UUID) */
  id: string
  /** Character who has this item */
  character_id: string
  /** Item name */
  item_name: string
  /** Item source (e.g., "PHB", "DMG") */
  item_source: string
  /** Quantity of this item */
  quantity: number
  /** Whether the item is equipped (0 or 1) */
  equipped: number
  /** Whether the item is attuned (0 or 1) */
  attuned: number
  /** Additional notes about the item */
  notes: string | null
}

// =============================================================================
// Character Request Types
// =============================================================================

/**
 * Request for creating a new player character.
 */
export interface CreatePcRequest {
  campaign_id: string
  name: string
  player_name: string
  race_name?: string
  race_source?: string
  background_name?: string
  background_source?: string
  /** Ability scores: [str, dex, con, int, wis, cha] */
  ability_scores?: [number, number, number, number, number, number]
}

/**
 * Request for creating a new NPC.
 */
export interface CreateNpcRequest {
  campaign_id: string
  name: string
  race_name?: string
  race_source?: string
  role?: string
  location?: string
  faction?: string
}

/**
 * Request for updating a character.
 */
export interface UpdateCharacterRequest {
  name?: string
  player_name?: string | null
  race_name?: string | null
  race_source?: string | null
  background_name?: string | null
  background_source?: string | null
  /** Ability scores: [str, dex, con, int, wis, cha] */
  ability_scores?: [number, number, number, number, number, number]
  /** Currency: [cp, sp, ep, gp, pp] */
  currency?: [number, number, number, number, number]
  traits?: string | null
  ideals?: string | null
  bonds?: string | null
  flaws?: string | null
  role?: string | null
  location?: string | null
  faction?: string | null
}

// =============================================================================
// Inventory Request Types
// =============================================================================

/**
 * Request for adding an item to inventory.
 */
export interface AddInventoryRequest {
  item_name: string
  item_source: string
  quantity?: number
  equipped?: boolean
  attuned?: boolean
  notes?: string
}

/**
 * Request for updating an inventory item.
 */
export interface UpdateInventoryRequest {
  quantity?: number
  equipped?: boolean
  attuned?: boolean
}

// =============================================================================
// Helper Types and Utilities
// =============================================================================

/**
 * Check if a character is an NPC.
 */
export function isNpc(character: Character): boolean {
  return character.is_npc !== 0
}

/**
 * Check if a character is a player character.
 */
export function isPc(character: Character): boolean {
  return character.is_npc === 0
}

/**
 * Calculate ability modifier for a given score.
 * Uses floor division to match D&D 5e rules.
 */
export function abilityModifier(score: number): number {
  return Math.floor((score - 10) / 2)
}

/**
 * Calculate total gold value of all currency.
 */
export function totalGoldValue(character: Character): number {
  return (
    character.cp / 100 +
    character.sp / 10 +
    character.ep / 2 +
    character.gp +
    character.pp * 10
  )
}

/**
 * Check if an inventory item is equipped.
 */
export function isEquipped(item: CharacterInventory): boolean {
  return item.equipped !== 0
}

/**
 * Check if an inventory item is attuned.
 */
export function isAttuned(item: CharacterInventory): boolean {
  return item.attuned !== 0
}

