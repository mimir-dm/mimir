/**
 * D&D 5e Character Utility Functions
 *
 * Provides calculations and helpers for character sheet displays.
 * These are reference calculations - the app displays computed values
 * but players track consumption (HP, spell slots) on paper.
 */

import type { Character, CharacterProficiency } from '../types/character'

// =============================================================================
// Constants
// =============================================================================

/** All 18 D&D 5e skills with their associated abilities */
export const ALL_SKILLS = [
  { name: 'Acrobatics', ability: 'dexterity' },
  { name: 'Animal Handling', ability: 'wisdom' },
  { name: 'Arcana', ability: 'intelligence' },
  { name: 'Athletics', ability: 'strength' },
  { name: 'Deception', ability: 'charisma' },
  { name: 'History', ability: 'intelligence' },
  { name: 'Insight', ability: 'wisdom' },
  { name: 'Intimidation', ability: 'charisma' },
  { name: 'Investigation', ability: 'intelligence' },
  { name: 'Medicine', ability: 'wisdom' },
  { name: 'Nature', ability: 'intelligence' },
  { name: 'Perception', ability: 'wisdom' },
  { name: 'Performance', ability: 'charisma' },
  { name: 'Persuasion', ability: 'charisma' },
  { name: 'Religion', ability: 'intelligence' },
  { name: 'Sleight of Hand', ability: 'dexterity' },
  { name: 'Stealth', ability: 'dexterity' },
  { name: 'Survival', ability: 'wisdom' },
] as const

/** Six ability score names */
export const ABILITIES = [
  'strength',
  'dexterity',
  'constitution',
  'intelligence',
  'wisdom',
  'charisma',
] as const

export type AbilityName = (typeof ABILITIES)[number]

// =============================================================================
// Core Calculations
// =============================================================================

/**
 * Calculate ability modifier from ability score.
 * Formula: floor((score - 10) / 2)
 */
export function getModifier(score: number): number {
  return Math.floor((score - 10) / 2)
}

/**
 * Format modifier for display with + or - prefix.
 */
export function formatModifier(mod: number): string {
  return mod >= 0 ? `+${mod}` : `${mod}`
}

/**
 * Calculate proficiency bonus based on total character level.
 * Formula: floor((level - 1) / 4) + 2
 */
export function getProficiencyBonus(totalLevel: number): number {
  if (totalLevel <= 0) return 2
  return Math.floor((totalLevel - 1) / 4) + 2
}

/**
 * Get total character level across all classes.
 */
export function getTotalLevel(character: Character): number {
  return character.classes?.reduce((sum, c) => sum + c.level, 0) || 0
}

// =============================================================================
// Proficiency Helpers
// =============================================================================

/**
 * Get all proficiencies of a specific type.
 */
export function getProficienciesByType(
  character: Character,
  type: CharacterProficiency['proficiency_type']
): CharacterProficiency[] {
  return character.proficiencies?.filter((p) => p.proficiency_type === type) || []
}

/**
 * Check if character is proficient in a specific skill.
 */
export function isProficientInSkill(character: Character, skillName: string): boolean {
  return getProficienciesByType(character, 'skill').some(
    (p) => p.name.toLowerCase() === skillName.toLowerCase()
  )
}

/**
 * Check if character has expertise in a specific skill.
 */
export function hasSkillExpertise(character: Character, skillName: string): boolean {
  return getProficienciesByType(character, 'skill').some(
    (p) => p.name.toLowerCase() === skillName.toLowerCase() && p.expertise !== 0
  )
}

/**
 * Check if character is proficient in a specific saving throw.
 */
export function isProficientInSave(character: Character, ability: string): boolean {
  return getProficienciesByType(character, 'save').some(
    (p) => p.name.toLowerCase() === ability.toLowerCase()
  )
}

// =============================================================================
// Skill & Save Calculations
// =============================================================================

/**
 * Get skill bonus for a character.
 */
export function getSkillBonus(
  character: Character,
  skillName: string,
  abilityScore: number
): number {
  const mod = getModifier(abilityScore)
  const profBonus = getProficiencyBonus(getTotalLevel(character))

  if (hasSkillExpertise(character, skillName)) {
    return mod + profBonus * 2
  }
  if (isProficientInSkill(character, skillName)) {
    return mod + profBonus
  }
  return mod
}

/**
 * Get saving throw bonus for a character.
 */
export function getSaveBonus(character: Character, ability: string, score: number): number {
  const mod = getModifier(score)
  const profBonus = getProficiencyBonus(getTotalLevel(character))

  if (isProficientInSave(character, ability)) {
    return mod + profBonus
  }
  return mod
}

/**
 * Calculate passive perception.
 */
export function getPassivePerception(character: Character): number {
  const wisMod = getModifier(character.wisdom)
  const profBonus = getProficiencyBonus(getTotalLevel(character))
  const isProficient = isProficientInSkill(character, 'Perception')
  const hasExpertise = hasSkillExpertise(character, 'Perception')

  let bonus = wisMod
  if (hasExpertise) {
    bonus += profBonus * 2
  } else if (isProficient) {
    bonus += profBonus
  }

  return 10 + bonus
}

// =============================================================================
// Combat Calculations
// =============================================================================

/**
 * Calculate AC from armor name and dex modifier.
 * Handles light, medium, and heavy armor types.
 */
export function getArmorAC(armorName: string, dexMod: number): number {
  const name = armorName.toLowerCase()

  // Extract magic bonus (+1, +2, +3)
  const magicMatch = name.match(/\+(\d)/)
  const magicBonus = magicMatch ? parseInt(magicMatch[1]) : 0

  // Light armor (full DEX)
  if (name.includes('padded') || (name.includes('leather') && !name.includes('studded'))) {
    return 11 + dexMod + magicBonus
  }
  if (name.includes('studded leather')) {
    return 12 + dexMod + magicBonus
  }

  // Medium armor (DEX max +2)
  const cappedDex = Math.min(dexMod, 2)
  if (name.includes('hide')) {
    return 12 + cappedDex + magicBonus
  }
  if (name.includes('chain shirt')) {
    return 13 + cappedDex + magicBonus
  }
  if (name.includes('scale mail') || name.includes('scale')) {
    return 14 + cappedDex + magicBonus
  }
  if (name.includes('breastplate')) {
    return 14 + cappedDex + magicBonus
  }
  if (name.includes('half plate')) {
    return 15 + cappedDex + magicBonus
  }

  // Heavy armor (no DEX)
  if (name.includes('ring mail')) {
    return 14 + magicBonus
  }
  if (name.includes('chain mail')) {
    return 16 + magicBonus
  }
  if (name.includes('splint')) {
    return 17 + magicBonus
  }
  if (name.includes('plate')) {
    return 18 + magicBonus
  }

  // Default: treat as light armor
  return 11 + dexMod + magicBonus
}

/**
 * Get weapon damage dice based on weapon name.
 */
export function getWeaponDamage(weaponName: string, abilityMod: number): string {
  const w = weaponName.toLowerCase()
  const modStr = abilityMod >= 0 ? `+${abilityMod}` : `${abilityMod}`

  // Heavy two-handed weapons
  if (w.includes('greatsword') || w.includes('maul')) return `2d6${modStr}`
  if (w.includes('greataxe')) return `1d12${modStr}`

  // Versatile / martial weapons
  if (w.includes('longsword') || w.includes('warhammer') || w.includes('battleaxe'))
    return `1d8${modStr}`
  if (w.includes('rapier')) return `1d8${modStr}`

  // Light weapons
  if (w.includes('shortsword') || w.includes('scimitar')) return `1d6${modStr}`
  if (w.includes('dagger')) return `1d4${modStr}`

  // Simple weapons
  if (w.includes('quarterstaff') || w.includes('spear')) return `1d6${modStr}`

  // Ranged weapons
  if (w.includes('longbow')) return `1d8${modStr}`
  if (w.includes('shortbow') || w.includes('light crossbow')) return `1d6${modStr}`
  if (w.includes('heavy crossbow')) return `1d10${modStr}`
  if (w.includes('hand crossbow')) return `1d6${modStr}`

  // Default
  return `1d6${modStr}`
}

/**
 * Check if a weapon uses finesse property.
 */
export function isFinesse(weaponName: string): boolean {
  const w = weaponName.toLowerCase()
  return (
    w.includes('rapier') ||
    w.includes('dagger') ||
    w.includes('shortsword') ||
    w.includes('scimitar') ||
    w.includes('whip')
  )
}

/**
 * Check if a weapon is ranged.
 */
export function isRanged(weaponName: string): boolean {
  const w = weaponName.toLowerCase()
  return w.includes('bow') || w.includes('crossbow') || w.includes('dart') || w.includes('sling')
}

// =============================================================================
// Spellcasting Calculations
// =============================================================================

/** Spellcasting ability by class */
export const SPELLCASTING_ABILITY: Record<string, AbilityName> = {
  bard: 'charisma',
  cleric: 'wisdom',
  druid: 'wisdom',
  paladin: 'charisma',
  ranger: 'wisdom',
  sorcerer: 'charisma',
  warlock: 'charisma',
  wizard: 'intelligence',
  artificer: 'intelligence',
}

/** Classes that can cast spells */
export const SPELLCASTING_CLASSES = Object.keys(SPELLCASTING_ABILITY)

/**
 * Check if character has any spellcasting class.
 */
export function isSpellcaster(character: Character): boolean {
  return character.classes?.some((c) =>
    SPELLCASTING_CLASSES.includes(c.class_name.toLowerCase())
  ) || false
}

/**
 * Get the spellcasting ability for a character.
 * Returns the first spellcasting class's ability.
 */
export function getSpellcastingAbility(character: Character): AbilityName | null {
  const spellcastingClass = character.classes?.find((c) =>
    SPELLCASTING_CLASSES.includes(c.class_name.toLowerCase())
  )
  if (!spellcastingClass) return null
  return SPELLCASTING_ABILITY[spellcastingClass.class_name.toLowerCase()] || null
}

/**
 * Calculate spell save DC.
 * Formula: 8 + proficiency bonus + spellcasting ability modifier
 */
export function getSpellSaveDC(character: Character): number | null {
  const ability = getSpellcastingAbility(character)
  if (!ability) return null

  const abilityScore = character[ability]
  const mod = getModifier(abilityScore)
  const profBonus = getProficiencyBonus(getTotalLevel(character))

  return 8 + profBonus + mod
}

/**
 * Calculate spell attack bonus.
 * Formula: proficiency bonus + spellcasting ability modifier
 */
export function getSpellAttackBonus(character: Character): number | null {
  const ability = getSpellcastingAbility(character)
  if (!ability) return null

  const abilityScore = character[ability]
  const mod = getModifier(abilityScore)
  const profBonus = getProficiencyBonus(getTotalLevel(character))

  return profBonus + mod
}

/** Spellcasting stats for a single class */
export interface SpellcastingStats {
  className: string
  ability: AbilityName
  abilityAbbrev: string
  saveDC: number
  attackBonus: number
}

/**
 * Get spellcasting stats for all spellcasting classes a character has.
 * For multiclass characters, returns one entry per spellcasting class.
 */
export function getAllSpellcastingStats(character: Character): SpellcastingStats[] {
  if (!character.classes) return []

  const profBonus = getProficiencyBonus(getTotalLevel(character))
  const stats: SpellcastingStats[] = []

  for (const cls of character.classes) {
    const ability = SPELLCASTING_ABILITY[cls.class_name.toLowerCase()]
    if (!ability) continue

    const abilityScore = character[ability]
    const mod = getModifier(abilityScore)

    stats.push({
      className: cls.class_name,
      ability,
      abilityAbbrev: ability.slice(0, 3).toUpperCase(),
      saveDC: 8 + profBonus + mod,
      attackBonus: profBonus + mod,
    })
  }

  return stats
}

/**
 * Calculate multiclass spellcaster level for spell slot determination.
 * Per D&D 5e rules:
 * - Full casters (Bard, Cleric, Druid, Sorcerer, Wizard): class level
 * - Half casters (Paladin, Ranger): floor(class level / 2)
 * - Third casters (Arcane Trickster, Eldritch Knight): floor(class level / 3)
 * - Warlock: excluded (uses Pact Magic, not Spellcasting)
 */
export function getMulticlassCasterLevel(character: Character): number {
  if (!character.classes) return 0

  let casterLevel = 0

  for (const cls of character.classes) {
    const name = cls.class_name.toLowerCase()

    // Full casters
    if (['bard', 'cleric', 'druid', 'sorcerer', 'wizard'].includes(name)) {
      casterLevel += cls.level
    }
    // Half casters (only count if level 2+)
    else if (['paladin', 'ranger'].includes(name) && cls.level >= 2) {
      casterLevel += Math.floor(cls.level / 2)
    }
    // Artificer is a special case - rounds up
    else if (name === 'artificer') {
      casterLevel += Math.ceil(cls.level / 2)
    }
    // Note: Warlock uses Pact Magic, not included in multiclass spell slots
    // Eldritch Knight/Arcane Trickster would need subclass detection
  }

  return casterLevel
}

// =============================================================================
// Hit Dice
// =============================================================================

/** Hit die type by class */
export const CLASS_HIT_DICE: Record<string, string> = {
  barbarian: 'd12',
  bard: 'd8',
  cleric: 'd8',
  druid: 'd8',
  fighter: 'd10',
  monk: 'd8',
  paladin: 'd10',
  ranger: 'd10',
  rogue: 'd8',
  sorcerer: 'd6',
  warlock: 'd8',
  wizard: 'd6',
  artificer: 'd8',
}

/**
 * Get hit dice string for a character.
 * Example: "5d10 + 3d8" for Fighter 5 / Rogue 3
 */
export function getHitDiceString(character: Character): string {
  if (!character.classes || character.classes.length === 0) {
    return '-'
  }

  return character.classes
    .map((c) => {
      const die = CLASS_HIT_DICE[c.class_name.toLowerCase()] || 'd8'
      return `${c.level}${die}`
    })
    .join(' + ')
}

// =============================================================================
// Display Helpers
// =============================================================================

/**
 * Format class string for display.
 * Example: "Fighter 5 / Rogue 3" or "Fighter (Champion) 5"
 */
export function formatClassString(character: Character): string {
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

/**
 * Get ability score by name from character.
 */
export function getAbilityScore(character: Character, ability: AbilityName): number {
  return character[ability]
}
