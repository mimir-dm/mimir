/**
 * TypeScript type definitions for character and player management
 *
 * These types match the Rust structs used in Tauri commands for character operations.
 */

// ============================================================================
// Player Types
// ============================================================================

export interface Player {
  id: number
  name: string
  email: string | null
  notes: string | null
  created_at: string
  updated_at: string
}

// ============================================================================
// Character Core Types
// ============================================================================

export interface Character {
  id: number
  campaign_id: number | null
  player_id: number | null  // Nullable for NPCs
  character_name: string
  is_npc: boolean
  directory_path: string
  current_level: number
  current_version: number
  created_at: string
  updated_at: string
  class: string | null
  race: string | null
}

export interface CharacterVersion {
  id: number
  character_id: number
  version_number: number
  file_path: string
  character_data: string  // YAML data
  snapshot_reason: string | null
  level: number
  created_at: string
}

// ============================================================================
// Character Data Types
// ============================================================================

export interface ClassLevel {
  class_name: string
  level: number
  subclass: string | null
  hit_dice_type: string
  hit_dice_remaining: number
}

export interface CharacterData {
  character_name: string
  player_id: number | null  // Nullable for NPCs
  level: number
  experience_points: number
  version: number
  snapshot_reason: string | null
  created_at: string
  race: string
  subrace: string | null
  classes: ClassLevel[]
  background: string
  alignment: string | null
  abilities: AbilityScores
  max_hp: number
  current_hp: number
  speed: number
  proficiencies: Proficiencies
  class_features: FeatureReference[]
  feats: string[]
  spells: SpellData
  inventory: InventoryItem[]
  currency: Currency
  equipped: EquippedItems
  personality: Personality
  // Extended character details
  player_name: string | null
  appearance: Appearance
  backstory: string | null
  background_feature: string | null
  roleplay_notes: RoleplayNotes
  // NPC-specific fields
  npc_role: string | null
  npc_location: string | null
  npc_faction: string | null
  npc_notes: string | null
  // Boss NPC abilities
  legendary_actions: LegendaryAction[]
  legendary_action_count: number | null
}

export interface AbilityScores {
  strength: number
  dexterity: number
  constitution: number
  intelligence: number
  wisdom: number
  charisma: number
}

export interface Proficiencies {
  skills: string[]
  saves: string[]
  armor: string[]
  weapons: string[]
  tools: string[]
  languages: string[]
}

export interface SpellReference {
  name: string
  source: string
}

export interface FeatureReference {
  name: string
  class_name: string
  subclass_name: string | null
  source: string
  level: number
}

export interface FeatureDetail {
  name: string
  class_name: string
  subclass_name: string | null
  source: string
  level: number
  description: string
}

export interface SpellData {
  known_spells: SpellReference[]
  prepared_spells: SpellReference[]
  cantrips: SpellReference[]
  spell_slots: Record<number, SpellSlots>
}

export interface SpellSlots {
  max: number
  current: number
}

export interface SpellReferenceInput {
  name: string
  source: string
}

export interface InventoryItem {
  name: string
  source: string | null
  quantity: number
  weight: number
  value: number
  notes: string | null
}

export interface Currency {
  copper: number
  silver: number
  electrum: number
  gold: number
  platinum: number
}

export interface EquippedItems {
  armor: string | null
  shield: string | null
  main_hand: string | null
  off_hand: string | null
}

export interface Personality {
  traits: string | null
  ideals: string | null
  bonds: string | null
  flaws: string | null
}

export interface Appearance {
  age: string | null
  height: string | null
  weight: string | null
  eyes: string | null
  hair: string | null
  skin: string | null
  physical_description: string | null
  distinctive_features: string | null
}

export interface RoleplayNotes {
  voice_and_mannerisms: string | null
  key_relationships: string | null
  character_goals: string | null
  play_reminders: string | null
  allies_and_organizations: string | null
  additional_treasure_notes: string | null
}

export interface LegendaryAction {
  name: string
  cost: number
  description: string
}

// ============================================================================
// Character Creation Request Types
// ============================================================================

export interface CreateCharacterRequest {
  character_name: string
  player_id: number | null  // Nullable for NPCs
  race: string
  race_source: string
  subrace: string | null
  class: string
  class_source: string
  subclass: string | null
  background: string
  background_source: string
  ability_score_method: 'standard_array' | 'point_buy' | 'manual'
  ability_scores: AbilityScoresInput | null
  alignment: string | null
  personality: PersonalityInput | null
  skill_proficiencies: string[] | null
  equipment: InventoryItemInput[] | null
  cantrips: SpellReferenceInput[] | null
  known_spells: SpellReferenceInput[] | null
  // NPC-specific fields
  is_npc: boolean | null
  npc_role: string | null
  npc_location: string | null
  npc_faction: string | null
  npc_notes: string | null
}

export interface AbilityScoresInput {
  strength: number
  dexterity: number
  constitution: number
  intelligence: number
  wisdom: number
  charisma: number
}

export interface PersonalityInput {
  traits: string | null
  ideals: string | null
  bonds: string | null
  flaws: string | null
}

export interface InventoryItemInput {
  name: string
  source: string | null
  quantity: number
  weight: number
  value: number
  notes: string | null
}

// ============================================================================
// Level Up Request Types
// ============================================================================

export interface LevelUpRequest {
  class_name: string
  class_source: string
  hit_points_roll: number | null
  take_average_hp: boolean
  subclass: string | null
  ability_score_improvement: string | null  // JSON string with ASI data
  feat: string | null
  new_spell_slots: string | null  // JSON string with spell slot updates
  new_known_spells: SpellReferenceInput[] | null  // Updated known spells list
  new_cantrips: SpellReferenceInput[] | null  // Updated cantrips list
}

export interface AsiOrFeat {
  type: 'asi' | 'feat'
  asi?: {
    ability1: string
    increase1: number
    ability2?: string
    increase2?: number
  }
  feat?: string
}

// ============================================================================
// Currency Update Types
// ============================================================================

export interface CurrencyUpdate {
  copper?: number
  silver?: number
  electrum?: number
  gold?: number
  platinum?: number
}

// ============================================================================
// Response Types (matches what Tauri commands return)
// ============================================================================

export interface CharacterWithData {
  character: Character
  data: CharacterData
}

// ============================================================================
// Tauri Command Function Signatures
// ============================================================================

/**
 * Type-safe wrappers for Tauri commands
 * Import these from '@tauri-apps/api/core' and cast the invoke function
 */
export interface PlayerCommands {
  create_player(name: string, email?: string, notes?: string): Promise<Player>
  get_player(player_id: number): Promise<Player>
  list_players(): Promise<Player[]>
  update_player(
    player_id: number,
    name?: string,
    email?: string | null,
    notes?: string | null
  ): Promise<Player>
  delete_player(player_id: number): Promise<void>
}

export interface CharacterCommands {
  create_character(request: CreateCharacterRequest): Promise<CharacterData>
  get_character(character_id: number): Promise<CharacterWithData>
  list_characters_for_campaign(campaign_id: number): Promise<Character[]>
  update_character_hp(
    character_id: number,
    new_hp: number,
    reason: string
  ): Promise<CharacterVersion>
  level_up_character(
    character_id: number,
    request: LevelUpRequest
  ): Promise<CharacterVersion>
  add_spell_to_known(
    character_id: number,
    spell_name: string,
    spell_source: string,
    is_cantrip: boolean
  ): Promise<CharacterVersion>
  prepare_spells(
    character_id: number,
    spells: SpellReferenceInput[],
    spellcasting_ability: string
  ): Promise<CharacterVersion>
  cast_spell(
    character_id: number,
    spell_name: string,
    spell_level: number,
    is_ritual: boolean
  ): Promise<CharacterVersion>
  rest_character(
    character_id: number,
    rest_type: 'short' | 'long'
  ): Promise<CharacterVersion>
  add_item(
    character_id: number,
    item_name: string,
    item_source: string,
    quantity: number,
    notes?: string
  ): Promise<CharacterVersion>
  remove_item(
    character_id: number,
    item_name: string,
    quantity: number
  ): Promise<CharacterVersion>
  update_character_currency(
    character_id: number,
    update: CurrencyUpdate
  ): Promise<CharacterVersion>
  delete_character(character_id: number): Promise<void>
  get_character_versions(character_id: number): Promise<CharacterVersion[]>
  get_character_version(
    character_id: number,
    version_number: number
  ): Promise<CharacterData>
}
