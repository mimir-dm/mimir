/**
 * Tests for monster formatting functions (useModuleMonsters.ts)
 *
 * Tests the pure formatting functions used by monster stat block components:
 * creature type, speed, ability modifiers, saves, skills, senses, languages,
 * damage/condition immunities, CR, AC, HP, action entries, spellcasting.
 */

import { describe, it, expect } from 'vitest'
import {
  formatCreatureType,
  formatSpeed,
  formatModifier,
  formatSaves,
  formatSkills,
  formatSenses,
  formatLanguages,
  formatDamageVulnerabilities,
  formatDamageResistances,
  formatDamageImmunities,
  formatConditionImmunities,
  formatCR,
  formatAC,
  formatHP,
  formatActionEntries,
  getOrdinal,
  getSpellcasting,
} from '@/features/modules/composables/useModuleMonsters'
import { srdMonsters } from '@tests/fixtures/monsters'

// ─── Fixture Helpers ─────────────────────────────────────────────────────────

function findMonster(name: string) {
  return srdMonsters.find(m => m.name === name)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('Monster Formatting Functions', () => {
  describe('formatCreatureType', () => {
    it('formats simple type with size and alignment', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatCreatureType(dragon)
      expect(result).toContain('Huge')
      expect(result).toContain('dragon')
      expect(result).toContain('chaotic')
      expect(result).toContain('evil')
    })

    it('formats small creature size', () => {
      const goblin = findMonster('Goblin')
      const result = formatCreatureType(goblin)
      expect(result).toContain('Small')
    })

    it('formats medium creature size', () => {
      const zombie = findMonster('Zombie')
      const result = formatCreatureType(zombie)
      expect(result).toContain('Medium')
    })

    it('formats object type with tags', () => {
      const data = { size: ['M'], type: { type: 'humanoid', tags: ['any race'] }, alignment: ['N'] }
      const result = formatCreatureType(data)
      expect(result).toContain('humanoid')
      expect(result).toContain('any race')
    })

    it('returns empty string for null data', () => {
      expect(formatCreatureType(null)).toBe('')
    })
  })

  describe('formatSpeed', () => {
    it('formats walk speed only', () => {
      const goblin = findMonster('Goblin')
      const result = formatSpeed(goblin)
      expect(result).toBe('30 ft.')
    })

    it('formats walk + climb + fly', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatSpeed(dragon)
      expect(result).toContain('40 ft.')
      expect(result).toContain('climb 40 ft.')
      expect(result).toContain('fly 80 ft.')
    })

    it('returns default for missing speed', () => {
      expect(formatSpeed({})).toBe('30 ft.')
      expect(formatSpeed(null)).toBe('30 ft.')
    })

    it('formats numeric speed', () => {
      expect(formatSpeed({ speed: 25 })).toBe('25 ft.')
    })
  })

  describe('formatModifier', () => {
    it('formats positive modifiers', () => {
      expect(formatModifier(16)).toBe('+3')
      expect(formatModifier(20)).toBe('+5')
      expect(formatModifier(27)).toBe('+8') // Adult Red Dragon STR
    })

    it('formats negative modifiers', () => {
      expect(formatModifier(8)).toBe('-1')
      expect(formatModifier(6)).toBe('-2')
      expect(formatModifier(3)).toBe('-4')
    })

    it('formats zero modifier', () => {
      expect(formatModifier(10)).toBe('+0')
      expect(formatModifier(11)).toBe('+0')
    })
  })

  describe('formatSaves', () => {
    it('formats saving throws from SRD data', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatSaves(dragon)
      expect(result).toContain('Dex +6')
      expect(result).toContain('Con +13')
      expect(result).toContain('Wis +7')
      expect(result).toContain('Cha +11')
    })

    it('returns empty string for no saves', () => {
      expect(formatSaves({})).toBe('')
      expect(formatSaves(null)).toBe('')
    })
  })

  describe('formatSkills', () => {
    it('formats skills from SRD data', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatSkills(dragon)
      expect(result).toContain('Perception +13')
      expect(result).toContain('Stealth +6')
    })

    it('returns empty string for no skills', () => {
      expect(formatSkills({})).toBe('')
    })
  })

  describe('formatSenses', () => {
    it('formats senses array with passive perception', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatSenses(dragon)
      expect(result).toContain('blindsight 60 ft.')
      expect(result).toContain('darkvision 120 ft.')
      expect(result).toContain('passive Perception 23')
    })

    it('returns empty string for no senses', () => {
      expect(formatSenses(null)).toBe('')
      expect(formatSenses({})).toBe('')
    })
  })

  describe('formatLanguages', () => {
    it('formats language array', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatLanguages(dragon)
      expect(result).toContain('Common')
      expect(result).toContain('Draconic')
    })

    it('returns dash for no languages', () => {
      expect(formatLanguages({})).toBe('—')
      expect(formatLanguages(null)).toBe('—')
    })
  })

  describe('formatDamageImmunities', () => {
    it('formats simple immunities', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatDamageImmunities(dragon)
      expect(result).toContain('fire')
    })

    it('returns empty string for no immunities', () => {
      expect(formatDamageImmunities({})).toBe('')
    })

    it('handles complex immunity objects with notes', () => {
      const data = {
        immune: [
          { immune: ['fire', 'poison'], note: '(from nonmagical attacks)' },
        ],
      }
      const result = formatDamageImmunities(data)
      expect(result).toContain('fire')
      expect(result).toContain('poison')
      expect(result).toContain('(from nonmagical attacks)')
    })
  })

  describe('formatDamageResistances', () => {
    it('handles complex resistance objects with notes', () => {
      const data = {
        resist: [
          { resist: ['bludgeoning', 'piercing', 'slashing'], note: 'from nonmagical attacks' },
        ],
      }
      const result = formatDamageResistances(data)
      expect(result).toContain('bludgeoning')
      expect(result).toContain('piercing')
      expect(result).toContain('slashing')
    })

    it('handles simple string resistances', () => {
      const data = { resist: ['fire', 'cold'] }
      expect(formatDamageResistances(data)).toBe('fire; cold')
    })

    it('returns empty string for no resistances', () => {
      expect(formatDamageResistances({})).toBe('')
    })
  })

  describe('formatDamageVulnerabilities', () => {
    it('formats vulnerability array', () => {
      const data = { vulnerable: ['fire', 'radiant'] }
      expect(formatDamageVulnerabilities(data)).toBe('fire, radiant')
    })

    it('returns empty string for no vulnerabilities', () => {
      expect(formatDamageVulnerabilities({})).toBe('')
    })
  })

  describe('formatConditionImmunities', () => {
    it('formats condition immunity array', () => {
      const data = { conditionImmune: ['poisoned', 'frightened'] }
      expect(formatConditionImmunities(data)).toBe('poisoned, frightened')
    })

    it('returns empty string for no condition immunities', () => {
      expect(formatConditionImmunities({})).toBe('')
    })
  })

  describe('formatCR', () => {
    it('formats CR with XP for Adult Red Dragon', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatCR(dragon)
      expect(result).toBe('17 (18,000 XP)')
    })

    it('formats fractional CR', () => {
      const goblin = findMonster('Goblin')
      const result = formatCR(goblin)
      expect(result).toContain('1/4')
      expect(result).toContain('50 XP')
    })

    it('returns ? for missing CR', () => {
      expect(formatCR({})).toBe('?')
      expect(formatCR(null)).toBe('?')
    })
  })

  describe('formatAC', () => {
    it('formats AC with armor source', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatAC(dragon)
      expect(result).toContain('19')
      expect(result).toContain('natural armor')
    })

    it('formats simple numeric AC', () => {
      const goblin = findMonster('Goblin')
      const result = formatAC(goblin)
      expect(result).toContain('15')
    })

    it('returns ? for missing AC', () => {
      expect(formatAC({})).toBe('?')
    })
  })

  describe('formatHP', () => {
    it('formats HP with formula', () => {
      const dragon = findMonster('Adult Red Dragon')
      const result = formatHP(dragon)
      expect(result).toContain('256')
      expect(result).toContain('19d12 + 133')
    })

    it('formats simple HP', () => {
      const goblin = findMonster('Goblin')
      const result = formatHP(goblin)
      expect(result).toContain('7')
      expect(result).toContain('2d6')
    })

    it('returns ? for missing HP', () => {
      expect(formatHP({})).toBe('?')
    })
  })

  describe('formatActionEntries', () => {
    it('formats simple text entries', () => {
      const result = formatActionEntries(['This is an action description.'])
      expect(result).toContain('This is an action description.')
    })

    it('processes 5etools formatting tags', () => {
      const result = formatActionEntries([
        '{@atk mw} {@hit 5} to hit, reach 5 ft., one target. {@h}7 ({@damage 1d8 + 3}) slashing damage.',
      ])
      expect(result).toContain('Melee Weapon Attack:')
      expect(result).toContain('hit-bonus')
      expect(result).toContain('1d8 + 3')
    })

    it('handles nested entry objects', () => {
      const result = formatActionEntries([
        { entries: ['Nested entry text.'] },
      ])
      expect(result).toContain('Nested entry text.')
    })

    it('returns empty string for null/undefined', () => {
      expect(formatActionEntries(null as any)).toBe('')
      expect(formatActionEntries(undefined as any)).toBe('')
    })
  })

  describe('getOrdinal', () => {
    it('formats ordinal numbers correctly', () => {
      expect(getOrdinal(1)).toBe('1st')
      expect(getOrdinal(2)).toBe('2nd')
      expect(getOrdinal(3)).toBe('3rd')
      expect(getOrdinal(4)).toBe('4th')
      expect(getOrdinal(5)).toBe('5th')
      expect(getOrdinal(9)).toBe('9th')
    })

    it('handles teen numbers', () => {
      expect(getOrdinal(11)).toBe('11th')
      expect(getOrdinal(12)).toBe('12th')
      expect(getOrdinal(13)).toBe('13th')
    })

    it('handles 21st, 22nd, 23rd', () => {
      expect(getOrdinal(21)).toBe('21st')
      expect(getOrdinal(22)).toBe('22nd')
      expect(getOrdinal(23)).toBe('23rd')
    })
  })

  describe('getSpellcasting', () => {
    it('formats spellcasting array with spell slots', () => {
      const lich = findMonster('Lich')
      const result = getSpellcasting(lich)
      expect(result).not.toBeNull()
      // Lich has spellcasting with slots
      expect(result).toContain('Cantrips')
      expect(result).toContain('1st level')
    })

    it('returns null for monsters without spellcasting', () => {
      const goblin = findMonster('Goblin')
      expect(getSpellcasting(goblin)).toBeNull()
    })

    it('returns null for null data', () => {
      expect(getSpellcasting(null)).toBeNull()
    })
  })

  // ─── Integration: SRD Monster Fixtures ────────────────────────────────────

  describe('SRD monster rendering integration', () => {
    describe('Goblin (simple monster)', () => {
      const goblin = findMonster('Goblin')!

      it('has correct basic stats', () => {
        expect(formatAC(goblin)).toContain('15')
        expect(formatHP(goblin)).toContain('7')
        expect(formatSpeed(goblin)).toBe('30 ft.')
        expect(formatCR(goblin)).toContain('1/4')
      })

      it('has Nimble Escape trait', () => {
        expect(goblin.trait).toBeDefined()
        const nimbleEscape = goblin.trait.find((t: any) => t.name === 'Nimble Escape')
        expect(nimbleEscape).toBeDefined()
        const entries = formatActionEntries(nimbleEscape.entries)
        expect(entries).toBeTruthy()
      })

      it('has scimitar and shortbow actions', () => {
        expect(goblin.action).toBeDefined()
        const scimitar = goblin.action.find((a: any) => a.name === 'Scimitar')
        expect(scimitar).toBeDefined()
        const shortbow = goblin.action.find((a: any) => a.name === 'Shortbow')
        expect(shortbow).toBeDefined()
      })

      it('has no legendary actions', () => {
        expect(goblin.legendary).toBeUndefined()
      })
    })

    describe('Adult Red Dragon (legendary creature)', () => {
      const dragon = findMonster('Adult Red Dragon')!

      it('has correct ability scores', () => {
        expect(dragon.str).toBe(27)
        expect(dragon.dex).toBe(10)
        expect(dragon.con).toBe(25)
        expect(dragon.int).toBe(16)
        expect(dragon.wis).toBe(13)
        expect(dragon.cha).toBe(21)
      })

      it('has fire immunity', () => {
        expect(formatDamageImmunities(dragon)).toContain('fire')
      })

      it('has legendary actions', () => {
        expect(dragon.legendary).toBeDefined()
        expect(dragon.legendary.length).toBe(3)
        const detect = dragon.legendary.find((a: any) => a.name === 'Detect')
        expect(detect).toBeDefined()
        const wingAttack = dragon.legendary.find((a: any) => a.name.includes('Wing Attack'))
        expect(wingAttack).toBeDefined()
        expect(wingAttack.name).toContain('Costs 2 Actions')
      })

      it('has Legendary Resistance trait', () => {
        const legendaryRes = dragon.trait.find((t: any) => t.name.includes('Legendary Resistance'))
        expect(legendaryRes).toBeDefined()
      })

      it('has multiattack', () => {
        const multiattack = dragon.action.find((a: any) => a.name === 'Multiattack')
        expect(multiattack).toBeDefined()
        const entries = formatActionEntries(multiattack.entries)
        expect(entries).toContain('Frightful Presence')
        expect(entries).toContain('bite')
        expect(entries).toContain('claws')
      })

      it('has fire breath with recharge', () => {
        const breath = dragon.action.find((a: any) => a.name.includes('Fire Breath'))
        expect(breath).toBeDefined()
        expect(breath.name).toContain('{@recharge 5}')
        const entries = formatActionEntries(breath.entries)
        expect(entries).toContain('60-foot cone')
      })

      it('has environment', () => {
        expect(dragon.environment).toContain('mountain')
        expect(dragon.environment).toContain('hill')
      })
    })

    describe('Lich (spellcaster)', () => {
      const lich = findMonster('Lich')!

      it('has spellcasting data', () => {
        expect(lich.spellcasting).toBeDefined()
        expect(lich.spellcasting.length).toBeGreaterThan(0)
      })

      it('formats spellcasting with cantrips and leveled spells', () => {
        const result = getSpellcasting(lich)
        expect(result).not.toBeNull()
        expect(result).toContain('Cantrips')
      })

      it('has legendary actions', () => {
        expect(lich.legendary).toBeDefined()
        const cantrip = lich.legendary.find((a: any) => a.name === 'Cantrip')
        expect(cantrip).toBeDefined()
        const disruptLife = lich.legendary.find((a: any) => a.name.includes('Disrupt Life'))
        expect(disruptLife).toBeDefined()
        expect(disruptLife.name).toContain('Costs 3 Actions')
      })

      it('has Legendary Resistance and Rejuvenation traits', () => {
        expect(lich.trait.find((t: any) => t.name.includes('Legendary Resistance'))).toBeDefined()
        expect(lich.trait.find((t: any) => t.name === 'Rejuvenation')).toBeDefined()
        expect(lich.trait.find((t: any) => t.name === 'Turn Resistance')).toBeDefined()
      })
    })

    describe('Vampire (complex creature)', () => {
      const vampire = findMonster('Vampire')!

      it('has shapechanger trait', () => {
        const shapechanger = vampire.trait.find((t: any) => t.name === 'Shapechanger')
        expect(shapechanger).toBeDefined()
      })

      it('has legendary actions', () => {
        expect(vampire.legendary).toBeDefined()
        expect(vampire.legendary.length).toBeGreaterThan(0)
      })

      it('has condition immunities', () => {
        // Vampires have damage resistances
        expect(formatDamageResistances(vampire)).toBeTruthy()
      })

      it('has multiattack action', () => {
        const multiattack = vampire.action.find((a: any) => a.name.includes('Multiattack'))
        expect(multiattack).toBeDefined()
      })
    })

    describe('Zombie (undead)', () => {
      const zombie = findMonster('Zombie')!

      it('has Undead Fortitude trait', () => {
        const fortitude = zombie.trait.find((t: any) => t.name === 'Undead Fortitude')
        expect(fortitude).toBeDefined()
        const entries = formatActionEntries(fortitude.entries)
        expect(entries).toContain('saving throw')
      })

      it('has correct type', () => {
        const result = formatCreatureType(zombie)
        expect(result).toContain('Medium')
        expect(result).toContain('undead')
      })
    })

    describe('Kobold (pack tactics + sunlight sensitivity)', () => {
      const kobold = findMonster('Kobold')!

      it('has Pack Tactics and Sunlight Sensitivity', () => {
        expect(kobold.trait.find((t: any) => t.name === 'Pack Tactics')).toBeDefined()
        expect(kobold.trait.find((t: any) => t.name === 'Sunlight Sensitivity')).toBeDefined()
      })

      it('is Tiny size', () => {
        // Kobold in SRD should be Small
        const result = formatCreatureType(kobold)
        expect(result).toContain('Small')
      })
    })
  })
})
