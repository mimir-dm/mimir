/**
 * TypeScript type definitions for character management
 *
 * Core types are auto-generated from Rust via ts-rs.
 * This file re-exports them and provides frontend-only request types and helpers.
 */

// =============================================================================
// Auto-generated types from Rust (via ts-rs)
// =============================================================================

// Import for use in helper functions
import type { CharacterProficiency } from './generated/CharacterProficiency'
import type { CharacterClass } from './generated/CharacterClass'
import type { CharacterInventory } from './generated/CharacterInventory'
import type { CharacterResponse } from './generated/CharacterResponse'

// Re-export generated types
export type { CharacterProficiency, CharacterClass, CharacterInventory, CharacterResponse }

// CharacterResponse from Rust is what the frontend uses as "Character"
// (includes classes and proficiencies arrays)
export type Character = CharacterResponse

// =============================================================================
// Character Request Types
// =============================================================================

/**
 * Request for creating a new player character.
 */
export interface CreatePcRequest {
  campaign_id?: string
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
  campaign_id?: string
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

/**
 * Get total character level across all classes.
 */
export function totalLevel(character: Character): number {
  return character.classes.reduce((sum, c) => sum + c.level, 0)
}

/**
 * Format class string (e.g., "Fighter 5 / Rogue 3").
 */
export function classString(character: Character): string {
  if (!character.classes || character.classes.length === 0) {
    return 'No Class'
  }
  return character.classes
    .map((c) => {
      if (c.subclass_name) {
        return `${c.class_name} (${c.subclass_name}) ${c.level}`
      }
      return `${c.class_name} ${c.level}`
    })
    .join(' / ')
}

// =============================================================================
// Proficiency Helpers
// =============================================================================

/**
 * Check if a proficiency grants expertise.
 */
export function hasExpertise(prof: CharacterProficiency): boolean {
  return prof.expertise !== 0
}

/**
 * Get all skill proficiencies for a character.
 */
export function getSkillProficiencies(character: Character): CharacterProficiency[] {
  return character.proficiencies?.filter((p) => p.proficiency_type === 'skill') || []
}

/**
 * Get all saving throw proficiencies for a character.
 */
export function getSaveProficiencies(character: Character): CharacterProficiency[] {
  return character.proficiencies?.filter((p) => p.proficiency_type === 'save') || []
}

/**
 * Get all tool proficiencies for a character.
 */
export function getToolProficiencies(character: Character): CharacterProficiency[] {
  return character.proficiencies?.filter((p) => p.proficiency_type === 'tool') || []
}

/**
 * Get all weapon proficiencies for a character.
 */
export function getWeaponProficiencies(character: Character): CharacterProficiency[] {
  return character.proficiencies?.filter((p) => p.proficiency_type === 'weapon') || []
}

/**
 * Get all armor proficiencies for a character.
 */
export function getArmorProficiencies(character: Character): CharacterProficiency[] {
  return character.proficiencies?.filter((p) => p.proficiency_type === 'armor') || []
}

/**
 * Get all language proficiencies for a character.
 */
export function getLanguages(character: Character): CharacterProficiency[] {
  return character.proficiencies?.filter((p) => p.proficiency_type === 'language') || []
}

/**
 * Check if a character is proficient in a specific skill.
 */
export function isProficientInSkill(character: Character, skillName: string): boolean {
  return getSkillProficiencies(character).some(
    (p) => p.name.toLowerCase() === skillName.toLowerCase()
  )
}

/**
 * Check if a character has expertise in a specific skill.
 */
export function hasSkillExpertise(character: Character, skillName: string): boolean {
  return getSkillProficiencies(character).some(
    (p) => p.name.toLowerCase() === skillName.toLowerCase() && p.expertise !== 0
  )
}

/**
 * Check if a character is proficient in a specific saving throw.
 */
export function isProficientInSave(character: Character, saveName: string): boolean {
  return getSaveProficiencies(character).some(
    (p) => p.name.toLowerCase() === saveName.toLowerCase()
  )
}

/**
 * Calculate proficiency bonus based on total character level.
 * Formula: floor((level - 1) / 4) + 2
 */
export function proficiencyBonus(character: Character): number {
  const level = totalLevel(character)
  if (level === 0) return 2 // Default for level 0
  return Math.floor((level - 1) / 4) + 2
}

// =============================================================================
// Level Up Types - matches mimir-core LevelUpRequest and related types
// =============================================================================

/**
 * Method for gaining HP on level up.
 */
export type HpGainMethod =
  | { type: 'Average' }
  | { type: 'Roll'; value: number }
  | { type: 'Manual'; value: number }

/**
 * Ability Score Improvement or Feat selection.
 */
export type AsiOrFeat =
  | {
      type: 'AbilityScoreImprovement'
      ability1: string
      increase1: number
      ability2?: string
      increase2?: number
    }
  | {
      type: 'Feat'
      name: string
      source: string
    }

/**
 * Subclass choice for level up.
 */
export interface SubclassChoice {
  name: string
  source: string
}

/**
 * Reference to a spell from the catalog.
 */
export interface SpellReference {
  name: string
  source: string
  /** Optional spell level (0 for cantrips, 1-9 for leveled spells) */
  level?: number
}

/**
 * Spell changes during level up.
 */
export interface SpellChanges {
  new_spells?: SpellReference[]
  new_cantrips?: SpellReference[]
  swap_out?: SpellReference
  swap_in?: SpellReference
}

/**
 * Reference to a class feature option from the catalog.
 */
export interface FeatureReference {
  name: string
  source: string
}

/**
 * Maneuver choices with optional swap for Battle Master.
 */
export interface ManeuverChoices {
  new_maneuvers: FeatureReference[]
  swap_out?: FeatureReference
  swap_in?: FeatureReference
}

/**
 * Invocation choices with optional swap for Warlock.
 */
export interface InvocationChoices {
  new_invocations: FeatureReference[]
  swap_out?: FeatureReference
  swap_in?: FeatureReference
}

/**
 * Class feature choices during level up.
 */
export interface FeatureChoices {
  fighting_style?: FeatureReference
  metamagic?: FeatureReference[]
  maneuvers?: ManeuverChoices
  invocations?: InvocationChoices
  pact_boon?: FeatureReference
  expertise_skills?: string[]
}

/**
 * Request for leveling up a character.
 * Matches the Rust LevelUpRequest struct.
 */
export interface LevelUpRequest {
  class_name: string
  class_source: string
  hit_points_method: HpGainMethod
  subclass?: SubclassChoice
  asi_or_feat?: AsiOrFeat
  spell_changes?: SpellChanges
  feature_choices?: FeatureChoices
}

/**
 * Response from level up operation.
 */
export interface LevelUpResult {
  character: Character
  class: CharacterClass
  hp_gained: number
  new_total_level: number
  is_multiclass: boolean
}

