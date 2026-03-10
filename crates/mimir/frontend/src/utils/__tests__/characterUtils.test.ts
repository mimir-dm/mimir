/**
 * Regression tests for characterUtils.
 *
 * Covers every exported function in characterUtils.ts with emphasis on
 * bug-fix regression targets: multiclass spellcasting stats (d3bdbbd),
 * multiclass caster level calculation (d3bdbbd), and edge cases in
 * ability modifiers, proficiency, armor AC, and weapon damage.
 */

import { describe, it, expect } from 'vitest'
import type { Character, CharacterClass, CharacterProficiency } from '@/types/character'
import {
  ALL_SKILLS,
  ABILITIES,
  SPELLCASTING_ABILITY,
  SPELLCASTING_CLASSES,
  CLASS_HIT_DICE,
  getModifier,
  formatModifier,
  getProficiencyBonus,
  getTotalLevel,
  getProficienciesByType,
  isProficientInSkill,
  hasSkillExpertise,
  isProficientInSave,
  getSkillBonus,
  getSaveBonus,
  getPassivePerception,
  getArmorAC,
  getWeaponDamage,
  isWeapon,
  isFinesse,
  isRanged,
  isSpellcaster,
  getSpellcastingAbility,
  getSpellSaveDC,
  getSpellAttackBonus,
  getAllSpellcastingStats,
  getMulticlassCasterLevel,
  getHitDiceString,
  formatClassString,
  getAbilityScore,
} from '../characterUtils'

// --- Factories ---

function makeClass(overrides: Partial<CharacterClass> = {}): CharacterClass {
  return {
    id: 'cc-1',
    character_id: 'char-1',
    class_name: 'Fighter',
    class_source: 'PHB',
    level: 5,
    subclass_name: null,
    subclass_source: null,
    starting_class: 1,
    ...overrides,
  }
}

function makeProf(overrides: Partial<CharacterProficiency> = {}): CharacterProficiency {
  return {
    id: 'prof-1',
    character_id: 'char-1',
    proficiency_type: 'skill',
    name: 'Perception',
    expertise: 0,
    ...overrides,
  }
}

function makeCharacter(overrides: Partial<Character> = {}): Character {
  return {
    id: 'char-1',
    campaign_id: 'camp-1',
    name: 'Test Character',
    is_npc: 0,
    player_name: 'Player',
    race_name: 'Human',
    race_source: 'PHB',
    background_name: 'Soldier',
    background_source: 'PHB',
    strength: 16,
    dexterity: 14,
    constitution: 13,
    intelligence: 10,
    wisdom: 12,
    charisma: 8,
    cp: 0, sp: 0, ep: 0, gp: 50, pp: 0,
    traits: null, ideals: null, bonds: null, flaws: null,
    role: null, location: null, faction: null,
    created_at: '2024-01-01',
    updated_at: '2024-01-01',
    classes: [makeClass()],
    proficiencies: [],
    ...overrides,
  } as Character
}

// --- Constants ---

describe('constants', () => {
  it('ALL_SKILLS has 18 skills', () => {
    expect(ALL_SKILLS).toHaveLength(18)
  })

  it('ABILITIES has 6 abilities', () => {
    expect(ABILITIES).toHaveLength(6)
    expect(ABILITIES).toContain('strength')
    expect(ABILITIES).toContain('charisma')
  })

  it('SPELLCASTING_CLASSES includes all casters', () => {
    expect(SPELLCASTING_CLASSES).toContain('wizard')
    expect(SPELLCASTING_CLASSES).toContain('cleric')
    expect(SPELLCASTING_CLASSES).toContain('warlock')
    expect(SPELLCASTING_CLASSES).not.toContain('fighter')
  })

  it('CLASS_HIT_DICE maps all PHB classes', () => {
    expect(CLASS_HIT_DICE['barbarian']).toBe('d12')
    expect(CLASS_HIT_DICE['wizard']).toBe('d6')
    expect(CLASS_HIT_DICE['fighter']).toBe('d10')
    expect(CLASS_HIT_DICE['rogue']).toBe('d8')
  })
})

// --- Core Calculations ---

describe('getModifier', () => {
  it('calculates standard modifiers', () => {
    expect(getModifier(10)).toBe(0)
    expect(getModifier(11)).toBe(0)
    expect(getModifier(12)).toBe(1)
    expect(getModifier(14)).toBe(2)
    expect(getModifier(16)).toBe(3)
    expect(getModifier(18)).toBe(4)
    expect(getModifier(20)).toBe(5)
  })

  it('handles low scores', () => {
    expect(getModifier(8)).toBe(-1)
    expect(getModifier(6)).toBe(-2)
    expect(getModifier(3)).toBe(-4)
    expect(getModifier(1)).toBe(-5)
  })

  it('handles extreme high scores', () => {
    expect(getModifier(30)).toBe(10)
  })

  it('handles odd vs even boundary', () => {
    expect(getModifier(9)).toBe(-1)
    expect(getModifier(10)).toBe(0)
    expect(getModifier(11)).toBe(0)
    expect(getModifier(12)).toBe(1)
    expect(getModifier(13)).toBe(1)
  })
})

describe('formatModifier', () => {
  it('adds + prefix for positive', () => {
    expect(formatModifier(3)).toBe('+3')
  })

  it('adds + prefix for zero', () => {
    expect(formatModifier(0)).toBe('+0')
  })

  it('uses - prefix for negative', () => {
    expect(formatModifier(-2)).toBe('-2')
  })
})

describe('getProficiencyBonus', () => {
  it('returns +2 for levels 1-4', () => {
    expect(getProficiencyBonus(1)).toBe(2)
    expect(getProficiencyBonus(4)).toBe(2)
  })

  it('returns +3 for levels 5-8', () => {
    expect(getProficiencyBonus(5)).toBe(3)
    expect(getProficiencyBonus(8)).toBe(3)
  })

  it('returns +4 for levels 9-12', () => {
    expect(getProficiencyBonus(9)).toBe(4)
    expect(getProficiencyBonus(12)).toBe(4)
  })

  it('returns +5 for levels 13-16', () => {
    expect(getProficiencyBonus(13)).toBe(5)
    expect(getProficiencyBonus(16)).toBe(5)
  })

  it('returns +6 for levels 17-20', () => {
    expect(getProficiencyBonus(17)).toBe(6)
    expect(getProficiencyBonus(20)).toBe(6)
  })

  it('returns +2 for level 0 or negative', () => {
    expect(getProficiencyBonus(0)).toBe(2)
    expect(getProficiencyBonus(-1)).toBe(2)
  })
})

describe('getTotalLevel', () => {
  it('returns single class level', () => {
    const char = makeCharacter({ classes: [makeClass({ level: 7 })] })
    expect(getTotalLevel(char)).toBe(7)
  })

  it('sums multiclass levels', () => {
    const char = makeCharacter({
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Fighter', level: 5 }),
        makeClass({ id: 'cc-2', class_name: 'Rogue', level: 3 }),
      ],
    })
    expect(getTotalLevel(char)).toBe(8)
  })

  it('returns 0 for no classes', () => {
    const char = makeCharacter({ classes: [] })
    expect(getTotalLevel(char)).toBe(0)
  })

  it('returns 0 for undefined classes', () => {
    const char = makeCharacter()
    ;(char as any).classes = undefined
    expect(getTotalLevel(char)).toBe(0)
  })
})

// --- Proficiency Helpers ---

describe('proficiency helpers', () => {
  describe('getProficienciesByType', () => {
    it('filters by type', () => {
      const char = makeCharacter({
        proficiencies: [
          makeProf({ id: 'p1', proficiency_type: 'skill', name: 'Athletics' }),
          makeProf({ id: 'p2', proficiency_type: 'save', name: 'Strength' }),
          makeProf({ id: 'p3', proficiency_type: 'skill', name: 'Perception' }),
        ],
      })
      expect(getProficienciesByType(char, 'skill')).toHaveLength(2)
      expect(getProficienciesByType(char, 'save')).toHaveLength(1)
    })

    it('returns empty for no match', () => {
      const char = makeCharacter({ proficiencies: [] })
      expect(getProficienciesByType(char, 'skill')).toEqual([])
    })
  })

  describe('isProficientInSkill', () => {
    it('returns true when proficient', () => {
      const char = makeCharacter({
        proficiencies: [makeProf({ name: 'Athletics', proficiency_type: 'skill' })],
      })
      expect(isProficientInSkill(char, 'Athletics')).toBe(true)
    })

    it('is case insensitive', () => {
      const char = makeCharacter({
        proficiencies: [makeProf({ name: 'Athletics', proficiency_type: 'skill' })],
      })
      expect(isProficientInSkill(char, 'athletics')).toBe(true)
    })

    it('returns false when not proficient', () => {
      const char = makeCharacter({ proficiencies: [] })
      expect(isProficientInSkill(char, 'Athletics')).toBe(false)
    })
  })

  describe('hasSkillExpertise', () => {
    it('returns true when expertise != 0', () => {
      const char = makeCharacter({
        proficiencies: [makeProf({ name: 'Stealth', proficiency_type: 'skill', expertise: 1 })],
      })
      expect(hasSkillExpertise(char, 'Stealth')).toBe(true)
    })

    it('returns false when expertise == 0', () => {
      const char = makeCharacter({
        proficiencies: [makeProf({ name: 'Stealth', proficiency_type: 'skill', expertise: 0 })],
      })
      expect(hasSkillExpertise(char, 'Stealth')).toBe(false)
    })
  })

  describe('isProficientInSave', () => {
    it('returns true for proficient save', () => {
      const char = makeCharacter({
        proficiencies: [makeProf({ name: 'Strength', proficiency_type: 'save' })],
      })
      expect(isProficientInSave(char, 'Strength')).toBe(true)
    })

    it('returns false for non-proficient save', () => {
      const char = makeCharacter({ proficiencies: [] })
      expect(isProficientInSave(char, 'Dexterity')).toBe(false)
    })
  })
})

// --- Skill & Save Calculations ---

describe('getSkillBonus', () => {
  it('returns bare modifier without proficiency', () => {
    const char = makeCharacter({ proficiencies: [] })
    // STR 16 = +3 mod, Fighter 5 = +3 prof
    expect(getSkillBonus(char, 'Athletics', 16)).toBe(3)
  })

  it('adds proficiency when proficient', () => {
    const char = makeCharacter({
      proficiencies: [makeProf({ name: 'Athletics', proficiency_type: 'skill' })],
    })
    // STR 16 = +3, prof +3 (level 5) = 6
    expect(getSkillBonus(char, 'Athletics', 16)).toBe(6)
  })

  it('doubles proficiency with expertise', () => {
    const char = makeCharacter({
      proficiencies: [
        makeProf({ name: 'Stealth', proficiency_type: 'skill', expertise: 1 }),
      ],
    })
    // DEX 14 = +2, expertise = +3*2 = +6, total = 8
    expect(getSkillBonus(char, 'Stealth', 14)).toBe(8)
  })

  it('works with negative modifier', () => {
    const char = makeCharacter({ proficiencies: [] })
    // CHA 8 = -1 mod
    expect(getSkillBonus(char, 'Deception', 8)).toBe(-1)
  })
})

describe('getSaveBonus', () => {
  it('returns modifier without proficiency', () => {
    const char = makeCharacter({ proficiencies: [] })
    // DEX 14 = +2
    expect(getSaveBonus(char, 'Dexterity', 14)).toBe(2)
  })

  it('adds proficiency bonus when proficient', () => {
    const char = makeCharacter({
      proficiencies: [makeProf({ name: 'Constitution', proficiency_type: 'save' })],
    })
    // CON 13 = +1, prof +3 (level 5) = 4
    expect(getSaveBonus(char, 'Constitution', 13)).toBe(4)
  })
})

describe('getPassivePerception', () => {
  it('returns 10 + wisdom mod for unproficient', () => {
    // WIS 12 = +1
    const char = makeCharacter({ wisdom: 12, proficiencies: [] })
    expect(getPassivePerception(char)).toBe(11)
  })

  it('adds proficiency bonus when proficient', () => {
    // WIS 12 = +1, prof +3 (level 5) = 10 + 4 = 14
    const char = makeCharacter({
      wisdom: 12,
      proficiencies: [makeProf({ name: 'Perception', proficiency_type: 'skill' })],
    })
    expect(getPassivePerception(char)).toBe(14)
  })

  it('doubles proficiency with expertise', () => {
    // WIS 12 = +1, expertise = +3*2 = +6, total = 10 + 7 = 17
    const char = makeCharacter({
      wisdom: 12,
      proficiencies: [
        makeProf({ name: 'Perception', proficiency_type: 'skill', expertise: 1 }),
      ],
    })
    expect(getPassivePerception(char)).toBe(17)
  })

  it('handles negative wisdom', () => {
    // WIS 8 = -1, unproficient => 10 - 1 = 9
    const char = makeCharacter({ wisdom: 8, proficiencies: [] })
    expect(getPassivePerception(char)).toBe(9)
  })
})

// --- Combat Calculations ---

describe('getArmorAC', () => {
  describe('light armor', () => {
    it('padded armor: 11 + DEX', () => {
      expect(getArmorAC('Padded', 3)).toBe(14)
    })

    it('leather armor: 11 + DEX', () => {
      expect(getArmorAC('Leather', 2)).toBe(13)
    })

    it('studded leather: 12 + DEX', () => {
      expect(getArmorAC('Studded Leather', 3)).toBe(15)
    })
  })

  describe('medium armor', () => {
    it('hide armor: 12 + DEX (max 2)', () => {
      expect(getArmorAC('Hide', 3)).toBe(14)
      expect(getArmorAC('Hide', 1)).toBe(13)
    })

    it('chain shirt: 13 + DEX (max 2)', () => {
      expect(getArmorAC('Chain Shirt', 4)).toBe(15) // caps at +2
    })

    it('scale mail: 14 + DEX (max 2)', () => {
      expect(getArmorAC('Scale Mail', 2)).toBe(16)
    })

    it('breastplate: 14 + DEX (max 2)', () => {
      expect(getArmorAC('Breastplate', 2)).toBe(16)
    })

    it('half plate: 15 + DEX (max 2)', () => {
      expect(getArmorAC('Half Plate', 2)).toBe(17)
    })
  })

  describe('heavy armor', () => {
    it('ring mail: 14 flat', () => {
      expect(getArmorAC('Ring Mail', 5)).toBe(14)
    })

    it('chain mail: 16 flat', () => {
      expect(getArmorAC('Chain Mail', 3)).toBe(16)
    })

    it('splint: 17 flat', () => {
      expect(getArmorAC('Splint', 2)).toBe(17)
    })

    it('plate: 18 flat', () => {
      expect(getArmorAC('Plate', 0)).toBe(18)
    })
  })

  describe('magic armor', () => {
    it('+1 leather: 11 + DEX + 1', () => {
      expect(getArmorAC('Leather +1', 2)).toBe(14)
    })

    it('+2 plate: 18 + 2', () => {
      expect(getArmorAC('Plate +2', 0)).toBe(20)
    })

    it('+3 studded leather: 12 + DEX + 3', () => {
      expect(getArmorAC('Studded Leather +3', 4)).toBe(19)
    })
  })

  describe('edge cases', () => {
    it('unknown armor defaults to 11 + DEX', () => {
      expect(getArmorAC('Mithral Something', 2)).toBe(13)
    })

    it('negative dex mod reduces AC', () => {
      expect(getArmorAC('Leather', -1)).toBe(10)
    })
  })
})

describe('getWeaponDamage', () => {
  it('heavy two-handed: 2d6', () => {
    expect(getWeaponDamage('Greatsword', 3)).toBe('2d6+3')
    expect(getWeaponDamage('Maul', 4)).toBe('2d6+4')
  })

  it('1d12 weapons', () => {
    expect(getWeaponDamage('Greataxe', 3)).toBe('1d12+3')
    expect(getWeaponDamage('Lance', 3)).toBe('1d12+3')
  })

  it('reach/heavy martial: 1d10', () => {
    expect(getWeaponDamage('Glaive', 3)).toBe('1d10+3')
    expect(getWeaponDamage('Halberd', 3)).toBe('1d10+3')
    expect(getWeaponDamage('Pike', 3)).toBe('1d10+3')
  })

  it('versatile/martial: 1d8', () => {
    expect(getWeaponDamage('Longsword', 3)).toBe('1d8+3')
    expect(getWeaponDamage('Rapier', 2)).toBe('1d8+2')
    expect(getWeaponDamage('Warhammer', 3)).toBe('1d8+3')
    expect(getWeaponDamage('Battleaxe', 3)).toBe('1d8+3')
    expect(getWeaponDamage('Flail', 3)).toBe('1d8+3')
    expect(getWeaponDamage('Morningstar', 3)).toBe('1d8+3')
    expect(getWeaponDamage('War Pick', 3)).toBe('1d8+3')
    expect(getWeaponDamage('Greatclub', 3)).toBe('1d8+3')
  })

  it('light/simple melee: 1d6', () => {
    expect(getWeaponDamage('Shortsword', 3)).toBe('1d6+3')
    expect(getWeaponDamage('Scimitar', 2)).toBe('1d6+2')
    expect(getWeaponDamage('Quarterstaff', 3)).toBe('1d6+3')
    expect(getWeaponDamage('Spear', 3)).toBe('1d6+3')
    expect(getWeaponDamage('Trident', 3)).toBe('1d6+3')
    expect(getWeaponDamage('Handaxe', 3)).toBe('1d6+3')
    expect(getWeaponDamage('Javelin', 3)).toBe('1d6+3')
    expect(getWeaponDamage('Mace', 3)).toBe('1d6+3')
  })

  it('small weapons: 1d4', () => {
    expect(getWeaponDamage('Dagger', 2)).toBe('1d4+2')
    expect(getWeaponDamage('Club', 2)).toBe('1d4+2')
    expect(getWeaponDamage('Light Hammer', 2)).toBe('1d4+2')
    expect(getWeaponDamage('Sickle', 2)).toBe('1d4+2')
    expect(getWeaponDamage('Whip', 2)).toBe('1d4+2')
  })

  it('ranged weapons', () => {
    expect(getWeaponDamage('Longbow', 3)).toBe('1d8+3')
    expect(getWeaponDamage('Shortbow', 2)).toBe('1d6+2')
    expect(getWeaponDamage('Light Crossbow', 2)).toBe('1d6+2')
    expect(getWeaponDamage('Heavy Crossbow', 2)).toBe('1d10+2')
    expect(getWeaponDamage('Hand Crossbow', 2)).toBe('1d6+2')
    expect(getWeaponDamage('Dart', 2)).toBe('1d4+2')
    expect(getWeaponDamage('Sling', 2)).toBe('1d4+2')
    expect(getWeaponDamage('Blowgun', 2)).toBe('1+2')
  })

  it('net: no damage', () => {
    expect(getWeaponDamage('Net', 2)).toBe('0')
  })

  it('negative modifier', () => {
    expect(getWeaponDamage('Dagger', -1)).toBe('1d4-1')
  })

  it('unknown weapon defaults to 1d6', () => {
    expect(getWeaponDamage('Exotic Blade', 3)).toBe('1d6+3')
  })
})

// --- Regression: isWeapon (T-0541) ---

describe('isWeapon', () => {
  it('detects all simple melee weapons', () => {
    expect(isWeapon('Club')).toBe(true)
    expect(isWeapon('Dagger')).toBe(true)
    expect(isWeapon('Greatclub')).toBe(true)
    expect(isWeapon('Handaxe')).toBe(true)
    expect(isWeapon('Javelin')).toBe(true)
    expect(isWeapon('Light Hammer')).toBe(true)
    expect(isWeapon('Mace')).toBe(true)
    expect(isWeapon('Quarterstaff')).toBe(true)
    expect(isWeapon('Sickle')).toBe(true)
    expect(isWeapon('Spear')).toBe(true)
  })

  it('detects all martial melee weapons', () => {
    expect(isWeapon('Battleaxe')).toBe(true)
    expect(isWeapon('Flail')).toBe(true)
    expect(isWeapon('Glaive')).toBe(true)
    expect(isWeapon('Greataxe')).toBe(true)
    expect(isWeapon('Greatsword')).toBe(true)
    expect(isWeapon('Halberd')).toBe(true)
    expect(isWeapon('Lance')).toBe(true)
    expect(isWeapon('Longsword')).toBe(true)
    expect(isWeapon('Maul')).toBe(true)
    expect(isWeapon('Morningstar')).toBe(true)
    expect(isWeapon('Pike')).toBe(true)
    expect(isWeapon('Rapier')).toBe(true)
    expect(isWeapon('Scimitar')).toBe(true)
    expect(isWeapon('Shortsword')).toBe(true)
    expect(isWeapon('Trident')).toBe(true)
    expect(isWeapon('War Pick')).toBe(true)
    expect(isWeapon('Warhammer')).toBe(true)
    expect(isWeapon('Whip')).toBe(true)
  })

  it('detects all ranged weapons', () => {
    expect(isWeapon('Blowgun')).toBe(true)
    expect(isWeapon('Dart')).toBe(true)
    expect(isWeapon('Hand Crossbow')).toBe(true)
    expect(isWeapon('Heavy Crossbow')).toBe(true)
    expect(isWeapon('Light Crossbow')).toBe(true)
    expect(isWeapon('Longbow')).toBe(true)
    expect(isWeapon('Net')).toBe(true)
    expect(isWeapon('Shortbow')).toBe(true)
    expect(isWeapon('Sling')).toBe(true)
  })

  it('detects magic weapon variants (suffix)', () => {
    expect(isWeapon('Longsword +1')).toBe(true)
    expect(isWeapon('Rapier +2')).toBe(true)
    expect(isWeapon('Dagger of Venom')).toBe(true)
  })

  it('detects magic weapon variants (prefix)', () => {
    expect(isWeapon('Flame Tongue Longsword')).toBe(true)
    expect(isWeapon('Vorpal Greatsword')).toBe(true)
  })

  it('rejects non-weapons', () => {
    expect(isWeapon('Plate')).toBe(false)
    expect(isWeapon('Shield')).toBe(false)
    expect(isWeapon('Potion of Healing')).toBe(false)
    expect(isWeapon('Rope')).toBe(false)
    expect(isWeapon('Chain Mail')).toBe(false)
  })
})

describe('isFinesse', () => {
  it('identifies finesse weapons', () => {
    expect(isFinesse('Rapier')).toBe(true)
    expect(isFinesse('Dagger')).toBe(true)
    expect(isFinesse('Shortsword')).toBe(true)
    expect(isFinesse('Scimitar')).toBe(true)
    expect(isFinesse('Whip')).toBe(true)
  })

  it('rejects non-finesse weapons', () => {
    expect(isFinesse('Longsword')).toBe(false)
    expect(isFinesse('Greatsword')).toBe(false)
    expect(isFinesse('Greataxe')).toBe(false)
  })
})

describe('isRanged', () => {
  it('identifies ranged weapons', () => {
    expect(isRanged('Longbow')).toBe(true)
    expect(isRanged('Shortbow')).toBe(true)
    expect(isRanged('Light Crossbow')).toBe(true)
    expect(isRanged('Dart')).toBe(true)
    expect(isRanged('Sling')).toBe(true)
    expect(isRanged('Blowgun')).toBe(true)
    expect(isRanged('Net')).toBe(true)
  })

  it('rejects melee weapons', () => {
    expect(isRanged('Longsword')).toBe(false)
    expect(isRanged('Rapier')).toBe(false)
  })
})

// --- Spellcasting ---

describe('spellcasting detection', () => {
  it('isSpellcaster returns true for caster classes', () => {
    const char = makeCharacter({ classes: [makeClass({ class_name: 'Wizard' })] })
    expect(isSpellcaster(char)).toBe(true)
  })

  it('isSpellcaster returns false for non-casters', () => {
    const char = makeCharacter({ classes: [makeClass({ class_name: 'Fighter' })] })
    expect(isSpellcaster(char)).toBe(false)
  })

  it('isSpellcaster returns false for no classes', () => {
    const char = makeCharacter({ classes: [] })
    expect(isSpellcaster(char)).toBe(false)
  })

  it('getSpellcastingAbility maps class to correct ability', () => {
    expect(getSpellcastingAbility(
      makeCharacter({ classes: [makeClass({ class_name: 'Wizard' })] })
    )).toBe('intelligence')

    expect(getSpellcastingAbility(
      makeCharacter({ classes: [makeClass({ class_name: 'Cleric' })] })
    )).toBe('wisdom')

    expect(getSpellcastingAbility(
      makeCharacter({ classes: [makeClass({ class_name: 'Sorcerer' })] })
    )).toBe('charisma')

    expect(getSpellcastingAbility(
      makeCharacter({ classes: [makeClass({ class_name: 'Warlock' })] })
    )).toBe('charisma')
  })

  it('getSpellcastingAbility returns null for non-casters', () => {
    const char = makeCharacter({ classes: [makeClass({ class_name: 'Fighter' })] })
    expect(getSpellcastingAbility(char)).toBeNull()
  })

  it('getSpellcastingAbility picks first caster in multiclass', () => {
    const char = makeCharacter({
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Fighter', level: 5 }),
        makeClass({ id: 'cc-2', class_name: 'Wizard', level: 3 }),
      ],
    })
    expect(getSpellcastingAbility(char)).toBe('intelligence')
  })
})

describe('getSpellSaveDC', () => {
  it('calculates DC = 8 + prof + ability mod', () => {
    // Wizard 5, INT 18 => 8 + 3 + 4 = 15
    const char = makeCharacter({
      intelligence: 18,
      classes: [makeClass({ class_name: 'Wizard', level: 5 })],
    })
    expect(getSpellSaveDC(char)).toBe(15)
  })

  it('returns null for non-casters', () => {
    const char = makeCharacter({ classes: [makeClass({ class_name: 'Fighter' })] })
    expect(getSpellSaveDC(char)).toBeNull()
  })
})

describe('getSpellAttackBonus', () => {
  it('calculates bonus = prof + ability mod', () => {
    // Wizard 5, INT 18 => 3 + 4 = 7
    const char = makeCharacter({
      intelligence: 18,
      classes: [makeClass({ class_name: 'Wizard', level: 5 })],
    })
    expect(getSpellAttackBonus(char)).toBe(7)
  })

  it('returns null for non-casters', () => {
    const char = makeCharacter({ classes: [makeClass({ class_name: 'Fighter' })] })
    expect(getSpellAttackBonus(char)).toBeNull()
  })
})

// --- Regression: getAllSpellcastingStats (d3bdbbd) ---

describe('getAllSpellcastingStats', () => {
  it('returns stats for single caster class', () => {
    const char = makeCharacter({
      intelligence: 18,
      classes: [makeClass({ class_name: 'Wizard', level: 5 })],
    })
    const stats = getAllSpellcastingStats(char)
    expect(stats).toHaveLength(1)
    expect(stats[0].className).toBe('Wizard')
    expect(stats[0].ability).toBe('intelligence')
    expect(stats[0].abilityAbbrev).toBe('INT')
    expect(stats[0].saveDC).toBe(15) // 8 + 3 + 4
    expect(stats[0].attackBonus).toBe(7) // 3 + 4
  })

  it('returns separate stats per class for multiclass casters', () => {
    // Wizard 3 / Cleric 2, total level 5, prof +3
    // INT 16 (+3), WIS 14 (+2)
    const char = makeCharacter({
      intelligence: 16,
      wisdom: 14,
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Wizard', level: 3 }),
        makeClass({ id: 'cc-2', class_name: 'Cleric', level: 2 }),
      ],
    })
    const stats = getAllSpellcastingStats(char)
    expect(stats).toHaveLength(2)

    const wizardStats = stats.find((s) => s.className === 'Wizard')!
    expect(wizardStats.ability).toBe('intelligence')
    expect(wizardStats.saveDC).toBe(14) // 8 + 3 + 3
    expect(wizardStats.attackBonus).toBe(6) // 3 + 3

    const clericStats = stats.find((s) => s.className === 'Cleric')!
    expect(clericStats.ability).toBe('wisdom')
    expect(clericStats.saveDC).toBe(13) // 8 + 3 + 2
    expect(clericStats.attackBonus).toBe(5) // 3 + 2
  })

  it('skips non-caster classes in multiclass', () => {
    const char = makeCharacter({
      intelligence: 16,
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Fighter', level: 5 }),
        makeClass({ id: 'cc-2', class_name: 'Wizard', level: 3 }),
      ],
    })
    const stats = getAllSpellcastingStats(char)
    expect(stats).toHaveLength(1)
    expect(stats[0].className).toBe('Wizard')
  })

  it('returns empty array for no classes', () => {
    const char = makeCharacter({ classes: [] })
    expect(getAllSpellcastingStats(char)).toEqual([])
  })

  it('returns empty array for undefined classes', () => {
    const char = makeCharacter()
    ;(char as any).classes = undefined
    expect(getAllSpellcastingStats(char)).toEqual([])
  })

  it('uses total character level for proficiency (not individual class level)', () => {
    // Warlock 5 / Wizard 4 = level 9, prof +4
    // INT 16 (+3) for Wizard, CHA 14 (+2) for Warlock
    const char = makeCharacter({
      intelligence: 16,
      charisma: 14,
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Warlock', level: 5 }),
        makeClass({ id: 'cc-2', class_name: 'Wizard', level: 4 }),
      ],
    })
    const stats = getAllSpellcastingStats(char)
    expect(stats).toHaveLength(2)

    const wizardStats = stats.find((s) => s.className === 'Wizard')!
    // prof +4 (level 9), INT +3 => DC 15, attack +7
    expect(wizardStats.saveDC).toBe(15)
    expect(wizardStats.attackBonus).toBe(7)

    const warlockStats = stats.find((s) => s.className === 'Warlock')!
    // prof +4 (level 9), CHA +2 => DC 14, attack +6
    expect(warlockStats.saveDC).toBe(14)
    expect(warlockStats.attackBonus).toBe(6)
  })
})

// --- Regression: getMulticlassCasterLevel (d3bdbbd) ---

describe('getMulticlassCasterLevel', () => {
  it('full casters contribute full level', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Wizard', level: 7 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(7)
  })

  it('half casters contribute half level (floor)', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Paladin', level: 6 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(3)
  })

  it('half casters at level 1 contribute 0', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Ranger', level: 1 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(0)
  })

  it('half casters at level 2 contribute 1', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Paladin', level: 2 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(1)
  })

  it('Warlock is excluded (uses Pact Magic)', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Warlock', level: 10 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(0)
  })

  it('Artificer rounds up (special case)', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Artificer', level: 5 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(3) // ceil(5/2) = 3
  })

  it('Artificer level 1 = caster level 1', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Artificer', level: 1 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(1) // ceil(1/2) = 1
  })

  it('multiclass combines correctly', () => {
    // Wizard 3 (full=3) + Paladin 4 (half=2) + Warlock 5 (excluded) = 5
    const char = makeCharacter({
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Wizard', level: 3 }),
        makeClass({ id: 'cc-2', class_name: 'Paladin', level: 4 }),
        makeClass({ id: 'cc-3', class_name: 'Warlock', level: 5 }),
      ],
    })
    expect(getMulticlassCasterLevel(char)).toBe(5)
  })

  it('Fighter contributes 0', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Fighter', level: 10 })],
    })
    expect(getMulticlassCasterLevel(char)).toBe(0)
  })

  it('all full casters combine', () => {
    // Bard 3 + Cleric 4 + Druid 2 = 9
    const char = makeCharacter({
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Bard', level: 3 }),
        makeClass({ id: 'cc-2', class_name: 'Cleric', level: 4 }),
        makeClass({ id: 'cc-3', class_name: 'Druid', level: 2 }),
      ],
    })
    expect(getMulticlassCasterLevel(char)).toBe(9)
  })

  it('returns 0 for no classes', () => {
    const char = makeCharacter({ classes: [] })
    expect(getMulticlassCasterLevel(char)).toBe(0)
  })
})

// --- Hit Dice ---

describe('getHitDiceString', () => {
  it('formats single class hit dice', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Fighter', level: 5 })],
    })
    expect(getHitDiceString(char)).toBe('5d10')
  })

  it('formats multiclass hit dice', () => {
    const char = makeCharacter({
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Fighter', level: 5 }),
        makeClass({ id: 'cc-2', class_name: 'Rogue', level: 3 }),
      ],
    })
    expect(getHitDiceString(char)).toBe('5d10 + 3d8')
  })

  it('returns dash for no classes', () => {
    const char = makeCharacter({ classes: [] })
    expect(getHitDiceString(char)).toBe('-')
  })

  it('uses d8 as default for unknown class', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Homebrew', level: 3 })],
    })
    expect(getHitDiceString(char)).toBe('3d8')
  })

  it('maps all standard classes correctly', () => {
    const testCases: [string, string][] = [
      ['Barbarian', '5d12'],
      ['Wizard', '5d6'],
      ['Sorcerer', '5d6'],
      ['Paladin', '5d10'],
      ['Ranger', '5d10'],
      ['Bard', '5d8'],
      ['Cleric', '5d8'],
      ['Warlock', '5d8'],
    ]
    for (const [className, expected] of testCases) {
      const char = makeCharacter({
        classes: [makeClass({ class_name: className, level: 5 })],
      })
      expect(getHitDiceString(char)).toBe(expected)
    }
  })
})

// --- Display Helpers ---

describe('formatClassString', () => {
  it('formats single class', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Fighter', level: 5 })],
    })
    expect(formatClassString(char)).toBe('Fighter 5')
  })

  it('formats class with subclass', () => {
    const char = makeCharacter({
      classes: [makeClass({ class_name: 'Fighter', level: 5, subclass_name: 'Champion' })],
    })
    expect(formatClassString(char)).toBe('Fighter (Champion) 5')
  })

  it('formats multiclass', () => {
    const char = makeCharacter({
      classes: [
        makeClass({ id: 'cc-1', class_name: 'Fighter', level: 5, subclass_name: 'Champion' }),
        makeClass({ id: 'cc-2', class_name: 'Rogue', level: 3 }),
      ],
    })
    expect(formatClassString(char)).toBe('Fighter (Champion) 5 / Rogue 3')
  })

  it('returns "No Class" for empty classes', () => {
    const char = makeCharacter({ classes: [] })
    expect(formatClassString(char)).toBe('No Class')
  })
})

describe('getAbilityScore', () => {
  it('returns correct score by name', () => {
    const char = makeCharacter({
      strength: 16,
      dexterity: 14,
      constitution: 13,
      intelligence: 10,
      wisdom: 12,
      charisma: 8,
    })
    expect(getAbilityScore(char, 'strength')).toBe(16)
    expect(getAbilityScore(char, 'dexterity')).toBe(14)
    expect(getAbilityScore(char, 'constitution')).toBe(13)
    expect(getAbilityScore(char, 'intelligence')).toBe(10)
    expect(getAbilityScore(char, 'wisdom')).toBe(12)
    expect(getAbilityScore(char, 'charisma')).toBe(8)
  })
})
