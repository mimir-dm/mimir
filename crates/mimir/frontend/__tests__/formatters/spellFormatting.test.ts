/**
 * Tests for spell formatting functions (spellFormatterEnhanced.ts)
 *
 * Tests the spell detail formatter that generates HTML stat blocks from
 * 5etools spell JSON data — casting time, range, components, duration,
 * description, higher level scaling, combat mechanics, cantrip scaling.
 */

import { describe, it, expect } from 'vitest'
import { formatSpellDetails } from '@/features/sources/formatters/spellFormatterEnhanced'
import { srdSpells } from '@tests/fixtures/spells'

// ─── Fixture Helpers ─────────────────────────────────────────────────────────

function findSpell(name: string) {
  return srdSpells.find(s => s.name === name)!
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('formatSpellDetails', () => {
  describe('header section', () => {
    it('renders level and school for leveled spell', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('spell-level-school')
      // Fireball is 3rd-level evocation (school "V")
      expect(html).toContain('3rd')
    })

    it('renders cantrip level', () => {
      const html = formatSpellDetails(findSpell('Fire Bolt'))
      expect(html).toContain('cantrip')
    })

    it('renders ritual tag for ritual spells', () => {
      const html = formatSpellDetails(findSpell('Detect Magic'))
      expect(html).toContain('ritual')
    })

    it('does not render ritual tag for non-ritual spells', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).not.toContain('Ritual')
    })

    it('renders concentration tag', () => {
      const html = formatSpellDetails(findSpell('Guidance'))
      expect(html).toContain('concentration')
    })
  })

  describe('casting time', () => {
    it('renders 1 action', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Casting Time')
      expect(html).toContain('1 action')
    })

    it('renders 1 bonus action', () => {
      const html = formatSpellDetails(findSpell('Healing Word'))
      // 5etools format: unit is "bonus" not "bonus action"
      expect(html).toContain('1 bonus')
    })

    it('renders reaction with condition', () => {
      const html = formatSpellDetails(findSpell('Shield'))
      expect(html).toContain('1 reaction')
      expect(html).toContain('hit by an attack')
    })

    it('renders Counterspell reaction trigger', () => {
      const html = formatSpellDetails(findSpell('Counterspell'))
      expect(html).toContain('1 reaction')
      expect(html).toContain('casting a spell')
    })
  })

  describe('range', () => {
    it('renders point range in feet', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Range')
      expect(html).toContain('150 feet')
    })

    it('renders touch range', () => {
      const html = formatSpellDetails(findSpell('Cure Wounds'))
      expect(html).toContain('Touch')
    })

    it('renders self range', () => {
      const html = formatSpellDetails(findSpell('Shield'))
      expect(html).toContain('Self')
    })

    it('renders long range', () => {
      const html = formatSpellDetails(findSpell('Eldritch Blast'))
      expect(html).toContain('120 feet')
    })
  })

  describe('components', () => {
    it('renders V, S components', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Components')
      expect(html).toContain('V')
      expect(html).toContain('S')
      expect(html).toContain('M')
    })

    it('renders material component with text', () => {
      const html = formatSpellDetails(findSpell('Light'))
      expect(html).toContain('firefly or phosphorescent moss')
    })

    it('renders material component with cost (consumed)', () => {
      const html = formatSpellDetails(findSpell('Revivify'))
      expect(html).toContain('diamonds worth 300 gp')
      expect(html).toContain('consumes')
    })

    it('renders V-only component', () => {
      const html = formatSpellDetails(findSpell('Counterspell'))
      expect(html).toContain('Components')
      // Counterspell has only S component
    })
  })

  describe('duration', () => {
    it('renders instantaneous', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Instantaneous')
    })

    it('renders concentration with time', () => {
      const html = formatSpellDetails(findSpell('Guidance'))
      expect(html).toContain('Concentration')
      expect(html).toContain('1 minute')
    })

    it('renders timed duration without concentration', () => {
      const html = formatSpellDetails(findSpell('Mage Armor'))
      // Mage Armor lasts 8 hours, no concentration
      expect(html).toContain('8 hour')
    })

    it('renders timed duration for Light', () => {
      const html = formatSpellDetails(findSpell('Light'))
      expect(html).toContain('1 hour')
    })
  })

  describe('description', () => {
    it('renders spell description text', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Description')
      expect(html).toContain('bright streak flashes')
    })

    it('processes 5etools formatting tags in description', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      // {@damage 8d6} should be processed
      expect(html).toContain('8d6')
    })

    it('renders Eldritch Blast description', () => {
      const html = formatSpellDetails(findSpell('Eldritch Blast'))
      expect(html).toContain('crackling energy')
      expect(html).toContain('ranged spell attack')
    })
  })

  describe('at higher levels', () => {
    it('renders higher level section when present', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Higher Level')
      // Fireball: "the damage increases by {@scaledamage ...} for each slot level above 3rd"
      expect(html).toContain('slot level above 3rd')
    })

    it('renders Cure Wounds higher level scaling', () => {
      const html = formatSpellDetails(findSpell('Cure Wounds'))
      expect(html).toContain('Higher Level')
    })

    it('does not render higher level for spells without it', () => {
      const html = formatSpellDetails(findSpell('Eldritch Blast'))
      expect(html).not.toContain('Higher Level')
    })
  })

  describe('combat mechanics', () => {
    it('renders spell attack type', () => {
      const html = formatSpellDetails(findSpell('Fire Bolt'))
      expect(html).toContain('Combat Mechanics')
      expect(html).toContain('Ranged spell attack')
    })

    it('renders damage type', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Fire')
    })

    it('renders saving throw', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Dexterity')
    })

    it('renders condition inflicted', () => {
      const html = formatSpellDetails(findSpell('Hold Person'))
      expect(html).toContain('Paralyzed')
    })

    it('renders area type', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Sphere')
    })

    it('renders force damage for Eldritch Blast', () => {
      const html = formatSpellDetails(findSpell('Eldritch Blast'))
      expect(html).toContain('Force')
    })
  })

  describe('cantrip scaling', () => {
    it('renders scaling section for cantrips with scalingLevelDice', () => {
      const html = formatSpellDetails(findSpell('Fire Bolt'))
      expect(html).toContain('Cantrip Scaling')
      expect(html).toContain('1d10')
      expect(html).toContain('2d10')
      expect(html).toContain('3d10')
      expect(html).toContain('4d10')
    })

    it('does not render scaling for leveled spells', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).not.toContain('Cantrip Scaling')
    })
  })

  describe('misc tags', () => {
    it('renders scaling tag for cantrips', () => {
      const html = formatSpellDetails(findSpell('Fire Bolt'))
      expect(html).toContain('Scaling')
    })

    it('renders light tag', () => {
      const html = formatSpellDetails(findSpell('Light'))
      expect(html).toContain('Light')
    })
  })

  describe('source attribution', () => {
    it('renders source and page', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('Source: PHB')
      expect(html).toContain('p. 241')
    })
  })

  describe('classes', () => {
    it('renders class list when present', () => {
      const fireball = findSpell('Fireball')
      if (fireball.classes?.fromClassList) {
        const html = formatSpellDetails(fireball)
        expect(html).toContain('Classes')
      }
    })
  })

  // ─── Integration: Specific Spell Types ──────────────────────────────────

  describe('specific spell integration tests', () => {
    it('Fireball: 3rd-level evocation, 150 ft, V/S/M, instant, 8d6 fire, Dex save', () => {
      const html = formatSpellDetails(findSpell('Fireball'))
      expect(html).toContain('spell-details enhanced')
      expect(html).toContain('3rd')
      expect(html).toContain('150 feet')
      expect(html).toContain('V')
      expect(html).toContain('Instantaneous')
      expect(html).toContain('8d6')
      expect(html).toContain('Dexterity')
      expect(html).toContain('Fire')
      expect(html).toContain('Sphere')
    })

    it('Shield: 1st-level, reaction, self, 1 round', () => {
      const html = formatSpellDetails(findSpell('Shield'))
      expect(html).toContain('1st')
      expect(html).toContain('1 reaction')
      expect(html).toContain('Self')
      expect(html).toContain('1 round')
    })

    it('Detect Magic: 1st-level, ritual, concentration 10 min', () => {
      const html = formatSpellDetails(findSpell('Detect Magic'))
      expect(html).toContain('1st')
      expect(html).toContain('ritual')
      expect(html).toContain('Concentration')
      expect(html).toContain('10 minute')
    })

    it('Wish: 9th-level, 1 action, self, instant', () => {
      const html = formatSpellDetails(findSpell('Wish'))
      expect(html).toContain('9th')
      expect(html).toContain('1 action')
      expect(html).toContain('Self')
      expect(html).toContain('Instantaneous')
    })

    it('Misty Step: 2nd-level, bonus, self, instant', () => {
      const html = formatSpellDetails(findSpell('Misty Step'))
      expect(html).toContain('2nd')
      expect(html).toContain('1 bonus')
      expect(html).toContain('Self')
      expect(html).toContain('Instantaneous')
    })

    it('Power Word Kill: 9th-level, no save, no attack', () => {
      const html = formatSpellDetails(findSpell('Power Word Kill'))
      expect(html).toContain('9th')
      expect(html).toContain('60 feet')
    })
  })

  describe('summary format fallback', () => {
    it('renders summary for spells without time field', () => {
      const summary = {
        name: 'Fireball',
        level: 3,
        school: 'evocation',
        casting_time: '1 action',
        range: '150 feet',
        components: 'V, S, M',
        description: 'A bright streak flashes from your finger.',
        source: 'PHB',
      }
      const html = formatSpellDetails(summary)
      expect(html).toContain('spell-details')
      expect(html).toContain('evocation')
      expect(html).toContain('1 action')
      expect(html).toContain('150 feet')
      expect(html).toContain('Source: PHB')
    })
  })
})
