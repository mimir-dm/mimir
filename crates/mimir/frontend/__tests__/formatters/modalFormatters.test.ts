/**
 * Tests for modal content formatters (modalFormatters.ts)
 *
 * Tests the renderModalContent dispatcher and individual renderers for each
 * ref_type: spell, item, creature/monster, condition, action, feat, background,
 * race, class, classFeature, subclass, subclassFeature, generic fallback.
 * Also tests helper functions: formatCurrency, getOrdinalSuffix, spell helpers.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
} from '@tests/helpers/mockInvoke'
import { renderModalContent } from '@/features/sources/formatters/modalFormatters'

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('renderModalContent', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('dispatcher', () => {
    it('returns empty string for null data', async () => {
      expect(await renderModalContent(null)).toBe('')
    })

    it('returns empty string for undefined data', async () => {
      expect(await renderModalContent(undefined)).toBe('')
    })

    it('renders name as h3 heading', async () => {
      const html = await renderModalContent({ name: 'Test Item', ref_type: 'item' })
      expect(html).toContain('<h3>Test Item</h3>')
    })

    it('wraps content in modal-content div', async () => {
      const html = await renderModalContent({ ref_type: 'item' })
      expect(html).toContain('<div class="modal-content">')
      expect(html).toContain('</div>')
    })

    it('routes creature ref_type to monster formatter', async () => {
      mockCommand('serve_book_image', null)
      const html = await renderModalContent({
        name: 'Goblin',
        ref_type: 'creature',
        str: 8, dex: 14, con: 10, int: 10, wis: 8, cha: 8,
      })
      expect(html).toContain('Goblin')
    })

    it('routes monster ref_type to monster formatter', async () => {
      mockCommand('serve_book_image', null)
      const html = await renderModalContent({
        name: 'Goblin',
        ref_type: 'monster',
        str: 8, dex: 14, con: 10, int: 10, wis: 8, cha: 8,
      })
      expect(html).toContain('Goblin')
    })

    it('falls back to generic for unknown ref_type', async () => {
      const html = await renderModalContent({
        ref_type: 'unknown_type',
        entries: ['Some text here'],
      })
      expect(html).toContain('generic-content')
      expect(html).toContain('Some text here')
    })

    it('uses type field when ref_type is absent', async () => {
      const html = await renderModalContent({
        type: 'condition',
        entries: ['A blinded creature cannot see.'],
      })
      expect(html).toContain('condition-content')
    })
  })

  describe('spell content', () => {
    it('renders cantrip level', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fire Bolt',
        level: 0,
        school: 'V',
      })
      expect(html).toContain('Cantrip')
      expect(html).toContain('Evocation')
    })

    it('renders leveled spell with ordinal', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fireball',
        level: 3,
        school: 'V',
      })
      expect(html).toContain('3rd-level')
      expect(html).toContain('Evocation')
    })

    it('renders casting time from array', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fireball',
        time: [{ number: 1, unit: 'action' }],
      })
      expect(html).toContain('Casting Time')
      expect(html).toContain('1 action')
    })

    it('renders reaction with condition', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Shield',
        time: [{ number: 1, unit: 'reaction', condition: 'when you are hit by an attack' }],
      })
      expect(html).toContain('1 reaction')
      expect(html).toContain('when you are hit by an attack')
    })

    it('renders point range in feet', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fireball',
        range: { type: 'point', distance: { type: 'feet', amount: 150 } },
      })
      expect(html).toContain('Range')
      expect(html).toContain('150 feet')
    })

    it('renders touch range', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Cure Wounds',
        range: { type: 'point', distance: { type: 'touch' } },
      })
      expect(html).toContain('Touch')
    })

    it('renders self range', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Shield',
        range: { type: 'point', distance: { type: 'self' } },
      })
      expect(html).toContain('Self')
    })

    it('renders V, S, M components', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fireball',
        components: { v: true, s: true, m: 'a tiny ball of bat guano' },
      })
      expect(html).toContain('Components')
      expect(html).toContain('V')
      expect(html).toContain('S')
      expect(html).toContain('M')
      expect(html).toContain('a tiny ball of bat guano')
    })

    it('renders instantaneous duration', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fireball',
        duration: [{ type: 'instant' }],
      })
      expect(html).toContain('Duration')
      expect(html).toContain('Instantaneous')
    })

    it('renders concentration duration', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Hold Person',
        duration: [{ type: 'timed', concentration: true, duration: { amount: 1, type: 'minute' } }],
      })
      expect(html).toContain('Concentration')
      expect(html).toContain('1 minute')
    })

    it('renders permanent duration with dispel', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Glyph of Warding',
        duration: [{ type: 'permanent', ends: ['dispel', 'trigger'] }],
      })
      expect(html).toContain('Until dispelled or triggered')
    })

    it('renders string entries in description', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fireball',
        entries: ['A bright streak flashes from your finger.'],
      })
      expect(html).toContain('spell-description')
      expect(html).toContain('A bright streak flashes')
    })

    it('renders nested entries with name', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Test Spell',
        entries: [
          { type: 'entries', name: 'Special Effect', entries: ['Does something cool.'] },
        ],
      })
      expect(html).toContain('Special Effect')
      expect(html).toContain('Does something cool')
    })

    it('renders list entries', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Test Spell',
        entries: [
          { type: 'list', items: ['Option A', 'Option B'] },
        ],
      })
      expect(html).toContain('<ul>')
      expect(html).toContain('Option A')
      expect(html).toContain('Option B')
    })

    it('renders higher level scaling', async () => {
      const html = await renderModalContent({
        ref_type: 'spell',
        name: 'Fireball',
        entriesHigherLevel: [
          { entries: ['When you cast this spell using a slot of 4th level or higher, the damage increases.'] },
        ],
      })
      expect(html).toContain('At Higher Levels')
      expect(html).toContain('damage increases')
    })

    it('maps all school codes', async () => {
      const schools: Record<string, string> = {
        A: 'Abjuration', C: 'Conjuration', D: 'Divination', E: 'Enchantment',
        V: 'Evocation', I: 'Illusion', N: 'Necromancy', T: 'Transmutation',
      }
      for (const [code, name] of Object.entries(schools)) {
        const html = await renderModalContent({
          ref_type: 'spell',
          level: 1,
          school: code,
        })
        expect(html).toContain(name)
      }
    })
  })

  describe('item content', () => {
    it('renders item type', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Longsword',
        type: 'Melee Weapon',
      })
      // type field also used for routing, but item renderer shows it
      expect(html).toContain('item-content')
    })

    it('renders rarity (non-none)', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Bag of Holding',
        rarity: 'uncommon',
      })
      expect(html).toContain('Rarity')
      expect(html).toContain('uncommon')
    })

    it('hides rarity when none', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Longsword',
        rarity: 'none',
      })
      expect(html).not.toContain('Rarity')
    })

    it('renders value in gold pieces', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Longsword',
        value: 1500, // 15 gp
      })
      expect(html).toContain('Value')
      expect(html).toContain('15 gp')
    })

    it('renders value in silver pieces', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Candle',
        value: 10, // 1 sp
      })
      expect(html).toContain('1 sp')
    })

    it('renders value in copper pieces', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Cheap Thing',
        value: 5, // 5 cp
      })
      expect(html).toContain('5 cp')
    })

    it('renders weight', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Longsword',
        weight: 3,
      })
      expect(html).toContain('Weight')
      expect(html).toContain('3 lb.')
    })

    it('renders item description entries', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        name: 'Bag of Holding',
        entries: ['This bag has an interior space considerably larger than its outside dimensions.'],
      })
      expect(html).toContain('item-description')
      expect(html).toContain('interior space considerably larger')
    })
  })

  describe('condition content', () => {
    it('renders string entries', async () => {
      const html = await renderModalContent({
        ref_type: 'condition',
        name: 'Blinded',
        entries: ['A blinded creature can\'t see and automatically fails any ability check that requires sight.'],
      })
      expect(html).toContain('condition-content')
      expect(html).toContain('blinded creature')
    })

    it('renders list entries', async () => {
      const html = await renderModalContent({
        ref_type: 'condition',
        name: 'Exhaustion',
        entries: [
          { type: 'list', items: ['Disadvantage on ability checks', 'Speed halved'] },
        ],
      })
      expect(html).toContain('<ul>')
      expect(html).toContain('Disadvantage on ability checks')
      expect(html).toContain('Speed halved')
    })

    it('renders nested entries with name', async () => {
      const html = await renderModalContent({
        ref_type: 'condition',
        name: 'Petrified',
        entries: [
          { type: 'entries', name: 'Effects', entries: ['The creature is transformed into stone.'] },
        ],
      })
      expect(html).toContain('<h4>Effects</h4>')
      expect(html).toContain('transformed into stone')
    })

    it('shows fallback when no entries', async () => {
      const html = await renderModalContent({
        ref_type: 'condition',
        name: 'Unknown',
      })
      expect(html).toContain('No description available')
    })
  })

  describe('action content', () => {
    it('renders time as array of objects', async () => {
      const html = await renderModalContent({
        ref_type: 'action',
        name: 'Dash',
        time: [{ number: 1, unit: 'action' }],
      })
      expect(html).toContain('Time')
      expect(html).toContain('1 action')
    })

    it('renders string time', async () => {
      const html = await renderModalContent({
        ref_type: 'action',
        name: 'Free Action',
        time: 'Free',
      })
      expect(html).toContain('Time')
      expect(html).toContain('Free')
    })

    it('renders entries with list', async () => {
      const html = await renderModalContent({
        ref_type: 'action',
        name: 'Attack',
        entries: [
          'You make a melee or ranged attack.',
          { type: 'list', items: ['Melee attack', 'Ranged attack'] },
        ],
      })
      expect(html).toContain('melee or ranged attack')
      expect(html).toContain('<ul>')
      expect(html).toContain('Melee attack')
    })

    it('renders nested entries with name', async () => {
      const html = await renderModalContent({
        ref_type: 'action',
        name: 'Grapple',
        entries: [
          { type: 'entries', name: 'Escaping a Grapple', entries: ['A grappled creature can use its action to escape.'] },
        ],
      })
      expect(html).toContain('<h4>Escaping a Grapple</h4>')
      expect(html).toContain('can use its action to escape')
    })
  })

  describe('feat content', () => {
    it('renders level prerequisite', async () => {
      const html = await renderModalContent({
        ref_type: 'feat',
        name: 'Epic Boon',
        prerequisite: [{ level: 20 }],
      })
      expect(html).toContain('Prerequisite')
      expect(html).toContain('Level 20')
    })

    it('renders race prerequisite', async () => {
      const html = await renderModalContent({
        ref_type: 'feat',
        name: 'Elven Accuracy',
        prerequisite: [{ race: [{ name: 'Elf' }, { name: 'Half-Elf' }] }],
      })
      expect(html).toContain('Prerequisite')
      expect(html).toContain('Elf or Half-Elf')
    })

    it('renders ability prerequisite', async () => {
      const html = await renderModalContent({
        ref_type: 'feat',
        name: 'Heavy Armor Master',
        prerequisite: [{ ability: [{ str: 13 }] }],
      })
      expect(html).toContain('Prerequisite')
      expect(html).toContain('STR 13')
    })

    it('renders spellcasting prerequisite', async () => {
      const html = await renderModalContent({
        ref_type: 'feat',
        name: 'Ritual Caster',
        prerequisite: [{ spellcasting: true }],
      })
      expect(html).toContain('Spellcasting ability')
    })

    it('renders feat entries with list', async () => {
      const html = await renderModalContent({
        ref_type: 'feat',
        name: 'Alert',
        entries: [
          'You gain the following benefits:',
          { type: 'list', items: ['+5 to initiative', 'You can\'t be surprised'] },
        ],
      })
      expect(html).toContain('feat-description')
      expect(html).toContain('+5 to initiative')
    })

    it('renders nested entries in feats', async () => {
      const html = await renderModalContent({
        ref_type: 'feat',
        name: 'Magic Initiate',
        entries: [
          { type: 'entries', name: 'Spellcasting', entries: ['Choose a class: bard, cleric, or wizard.'] },
        ],
      })
      expect(html).toContain('Spellcasting')
      expect(html).toContain('Choose a class')
    })
  })

  describe('background content', () => {
    it('renders skill proficiencies', async () => {
      const html = await renderModalContent({
        ref_type: 'background',
        name: 'Acolyte',
        skillProficiencies: [{ insight: true, religion: true }],
      })
      expect(html).toContain('Skill Proficiencies')
      expect(html).toContain('insight')
      expect(html).toContain('religion')
    })

    it('renders tool proficiencies', async () => {
      const html = await renderModalContent({
        ref_type: 'background',
        name: 'Criminal',
        toolProficiencies: [{ "thieves' tools": true, "one type of gaming set": true }],
      })
      expect(html).toContain('Tool Proficiencies')
      expect(html).toContain("thieves' tools")
    })

    it('renders language proficiencies', async () => {
      const html = await renderModalContent({
        ref_type: 'background',
        name: 'Acolyte',
        languageProficiencies: [{ anyStandard: 2 }],
      })
      expect(html).toContain('Languages')
      expect(html).toContain('2 of your choice')
    })

    it('renders starting equipment', async () => {
      const html = await renderModalContent({
        ref_type: 'background',
        name: 'Acolyte',
        startingEquipment: [{ _: 'A holy symbol' }, { _: 'A prayer book' }],
      })
      expect(html).toContain('Equipment')
      expect(html).toContain('A holy symbol')
      expect(html).toContain('A prayer book')
    })

    it('renders background features', async () => {
      const html = await renderModalContent({
        ref_type: 'background',
        name: 'Acolyte',
        entries: [
          { type: 'entries', name: 'Shelter of the Faithful', entries: ['As an acolyte, you command respect.'] },
        ],
      })
      expect(html).toContain('background-features')
      expect(html).toContain('<h4>Shelter of the Faithful</h4>')
      expect(html).toContain('command respect')
    })
  })

  describe('race content', () => {
    it('renders size', async () => {
      const html = await renderModalContent({
        ref_type: 'race',
        name: 'Human',
        size: ['M'],
      })
      expect(html).toContain('Size')
      expect(html).toContain('Medium')
    })

    it('renders multiple sizes', async () => {
      const html = await renderModalContent({
        ref_type: 'race',
        name: 'Changeling',
        size: ['S', 'M'],
      })
      expect(html).toContain('Small or Medium')
    })

    it('renders numeric speed', async () => {
      const html = await renderModalContent({
        ref_type: 'race',
        name: 'Human',
        speed: 30,
      })
      expect(html).toContain('Speed')
      expect(html).toContain('30 ft.')
    })

    it('renders object speed with multiple types', async () => {
      const html = await renderModalContent({
        ref_type: 'race',
        name: 'Aarakocra',
        speed: { walk: 25, fly: 50 },
      })
      expect(html).toContain('25 ft.')
      expect(html).toContain('fly 50 ft.')
    })

    it('renders darkvision', async () => {
      const html = await renderModalContent({
        ref_type: 'race',
        name: 'Elf',
        darkvision: 60,
      })
      expect(html).toContain('Darkvision')
      expect(html).toContain('60 ft.')
    })

    it('renders race traits', async () => {
      const html = await renderModalContent({
        ref_type: 'race',
        name: 'Elf',
        entries: [
          { type: 'entries', name: 'Trance', entries: ['Elves do not sleep.'] },
        ],
      })
      expect(html).toContain('race-traits')
      expect(html).toContain('<h4>Trance</h4>')
      expect(html).toContain('Elves do not sleep')
    })
  })

  describe('class content', () => {
    it('renders hit die', async () => {
      const html = await renderModalContent({
        ref_type: 'class',
        name: 'Fighter',
        hd: { faces: 10 },
      })
      expect(html).toContain('Hit Die')
      expect(html).toContain('d10')
    })

    it('renders primary ability', async () => {
      const html = await renderModalContent({
        ref_type: 'class',
        name: 'Wizard',
        primaryAbility: [{ int: true }],
      })
      expect(html).toContain('Primary Ability')
      expect(html).toContain('INT')
    })

    it('renders saving throws', async () => {
      const html = await renderModalContent({
        ref_type: 'class',
        name: 'Fighter',
        proficiency: ['str', 'con'],
      })
      expect(html).toContain('Saving Throws')
      expect(html).toContain('STR')
      expect(html).toContain('CON')
    })

    it('renders armor and weapon proficiencies', async () => {
      const html = await renderModalContent({
        ref_type: 'class',
        name: 'Fighter',
        startingProficiencies: {
          armor: ['light', 'medium', 'heavy', 'shields'],
          weapons: ['simple', 'martial'],
        },
      })
      expect(html).toContain('Armor')
      expect(html).toContain('heavy')
      expect(html).toContain('Weapons')
      expect(html).toContain('martial')
    })

    it('renders subclass title', async () => {
      const html = await renderModalContent({
        ref_type: 'class',
        name: 'Fighter',
        subclassTitle: 'Martial Archetype',
      })
      expect(html).toContain('Subclass')
      expect(html).toContain('Martial Archetype')
    })

    it('renders level 1 features (object format)', async () => {
      const html = await renderModalContent({
        ref_type: 'class',
        name: 'Fighter',
        classFeatures: [
          { name: 'Fighting Style', level: 1 },
          { name: 'Second Wind', level: 1 },
          { name: 'Action Surge', level: 2 },
        ],
      })
      expect(html).toContain('Features at 1st Level')
      expect(html).toContain('Fighting Style')
      expect(html).toContain('Second Wind')
    })

    it('renders level 1 features (string format with |1|)', async () => {
      const html = await renderModalContent({
        ref_type: 'class',
        name: 'Fighter',
        classFeatures: [
          'Fighting Style|Fighter|PHB|1|',
          'Second Wind|Fighter|PHB|1|',
          'Action Surge|Fighter|PHB|2|',
        ],
      })
      expect(html).toContain('Features at 1st Level')
      expect(html).toContain('Fighting Style')
      expect(html).toContain('Second Wind')
    })
  })

  describe('class feature content', () => {
    it('renders class name and level', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Action Surge',
        className: 'Fighter',
        level: 2,
      })
      expect(html).toContain('Fighter feature (Level 2)')
    })

    it('renders class_name variant', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Rage',
        class_name: 'Barbarian',
        level: 1,
      })
      expect(html).toContain('Barbarian feature (Level 1)')
    })

    it('renders class name without level', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Feature',
        className: 'Wizard',
      })
      expect(html).toContain('Wizard feature')
      expect(html).not.toContain('Level')
    })

    it('renders entries with text', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Second Wind',
        className: 'Fighter',
        level: 1,
        entries: ['You have a limited well of stamina.'],
      })
      expect(html).toContain('feature-description')
      expect(html).toContain('limited well of stamina')
    })

    it('renders table entry', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Rage',
        entries: [
          {
            type: 'table',
            caption: 'Rage Damage',
            colLabels: ['Level', 'Damage'],
            rows: [['1st', '+2'], ['9th', '+3']],
          },
        ],
      })
      expect(html).toContain('feature-table')
      expect(html).toContain('Rage Damage')
      expect(html).toContain('<th>Level</th>')
      expect(html).toContain('<th>Damage</th>')
      expect(html).toContain('+2')
      expect(html).toContain('+3')
    })

    it('renders inset/quote entry', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Channel Divinity',
        entries: [
          { type: 'inset', name: 'Note', entries: ['This is a sidebar note.'] },
        ],
      })
      expect(html).toContain('<blockquote>')
      expect(html).toContain('Note')
      expect(html).toContain('sidebar note')
    })

    it('renders options entry', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Fighting Style',
        entries: [
          {
            type: 'options',
            entries: [
              { name: 'Archery', entries: ['+2 to ranged attack rolls'] },
              { name: 'Defense', entries: ['+1 AC while wearing armor'] },
            ],
          },
        ],
      })
      expect(html).toContain('feature-options')
      expect(html).toContain('<h5>Archery</h5>')
      expect(html).toContain('+2 to ranged attack rolls')
      expect(html).toContain('<h5>Defense</h5>')
    })

    it('renders list entry', async () => {
      const html = await renderModalContent({
        ref_type: 'classFeature',
        name: 'Expertise',
        entries: [
          { type: 'list', items: ['Stealth', 'Perception'] },
        ],
      })
      expect(html).toContain('<ul>')
      expect(html).toContain('Stealth')
      expect(html).toContain('Perception')
    })
  })

  describe('subclass content', () => {
    it('renders parent class name', async () => {
      const html = await renderModalContent({
        ref_type: 'subclass',
        name: 'Champion',
        className: 'Fighter',
      })
      expect(html).toContain('Fighter subclass')
    })

    it('renders class_name variant', async () => {
      const html = await renderModalContent({
        ref_type: 'subclass',
        name: 'Thief',
        class_name: 'Rogue',
      })
      expect(html).toContain('Rogue subclass')
    })

    it('renders subclass features list (string format)', async () => {
      const html = await renderModalContent({
        ref_type: 'subclass',
        name: 'Champion',
        className: 'Fighter',
        subclassFeatures: [
          'Improved Critical|Fighter||Champion||3',
          'Remarkable Athlete|Fighter||Champion||7',
        ],
      })
      expect(html).toContain('Features')
      expect(html).toContain('Improved Critical')
      expect(html).toContain('Level 3')
      expect(html).toContain('Remarkable Athlete')
      expect(html).toContain('Level 7')
    })

    it('renders subclass features list (object format)', async () => {
      const html = await renderModalContent({
        ref_type: 'subclass',
        name: 'Champion',
        subclassFeatures: [
          { name: 'Improved Critical', level: 3 },
        ],
      })
      expect(html).toContain('Improved Critical')
      expect(html).toContain('Level 3')
    })

    it('renders subclass description entries', async () => {
      const html = await renderModalContent({
        ref_type: 'subclass',
        name: 'Champion',
        entries: [
          { type: 'entries', name: 'Combat Superiority', entries: ['You are a master of martial combat.'] },
        ],
      })
      expect(html).toContain('subclass-description')
      expect(html).toContain('Combat Superiority')
    })
  })

  describe('subclass feature content', () => {
    it('renders full subtitle with class, subclass, and level', async () => {
      const html = await renderModalContent({
        ref_type: 'subclassFeature',
        name: 'Improved Critical',
        className: 'Fighter',
        subclassShortName: 'Champion',
        level: 3,
      })
      expect(html).toContain('Fighter (Champion) feature, Level 3')
    })

    it('renders with class_name and subclass_short_name variants', async () => {
      const html = await renderModalContent({
        ref_type: 'subclassFeature',
        name: 'Sneak Attack',
        class_name: 'Rogue',
        subclass_short_name: 'Thief',
        level: 1,
      })
      expect(html).toContain('Rogue (Thief) feature, Level 1')
    })

    it('renders with only className and level', async () => {
      const html = await renderModalContent({
        ref_type: 'subclassFeature',
        name: 'Extra Attack',
        className: 'Fighter',
        level: 5,
      })
      expect(html).toContain('Fighter feature, Level 5')
    })

    it('renders entries with table', async () => {
      const html = await renderModalContent({
        ref_type: 'subclassFeature',
        name: 'Superiority Dice',
        entries: [
          {
            type: 'table',
            colLabels: ['Die', 'Level'],
            rows: [['d8', '3rd'], ['d10', '10th']],
          },
        ],
      })
      expect(html).toContain('feature-table')
      expect(html).toContain('d8')
      expect(html).toContain('d10')
    })

    it('renders entries with quote block', async () => {
      const html = await renderModalContent({
        ref_type: 'subclassFeature',
        name: 'Path Feature',
        entries: [
          { type: 'quote', name: 'Flavor', entries: ['The path of the berserker is one of fury.'] },
        ],
      })
      expect(html).toContain('<blockquote>')
      expect(html).toContain('Flavor')
      expect(html).toContain('fury')
    })

    it('renders entries with options', async () => {
      const html = await renderModalContent({
        ref_type: 'subclassFeature',
        name: 'Maneuvers',
        entries: [
          {
            type: 'options',
            entries: [
              { name: 'Disarming Attack', entries: ['When you hit a creature, you can disarm it.'] },
            ],
          },
        ],
      })
      expect(html).toContain('feature-options')
      expect(html).toContain('Disarming Attack')
    })
  })

  describe('generic content fallback', () => {
    it('renders entries', async () => {
      const html = await renderModalContent({
        ref_type: 'something_else',
        entries: ['This is generic content.'],
      })
      expect(html).toContain('generic-content')
      expect(html).toContain('This is generic content')
    })

    it('renders text field', async () => {
      const html = await renderModalContent({
        ref_type: 'unknown',
        text: 'Fallback text content.',
      })
      expect(html).toContain('Fallback text content')
    })

    it('renders description field', async () => {
      const html = await renderModalContent({
        ref_type: 'unknown',
        description: 'A description of something.',
      })
      expect(html).toContain('A description of something')
    })
  })

  describe('currency formatting', () => {
    // Currency is tested indirectly through item rendering
    it('formats gold pieces (no remainder)', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        value: 5000, // 50 gp
      })
      expect(html).toContain('50 gp')
    })

    it('formats gold pieces (with decimal)', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        value: 250, // 2.5 gp
      })
      expect(html).toContain('2.5 gp')
    })

    it('formats silver pieces', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        value: 50, // 5 sp
      })
      expect(html).toContain('5 sp')
    })

    it('formats copper pieces', async () => {
      const html = await renderModalContent({
        ref_type: 'item',
        value: 3,
      })
      expect(html).toContain('3 cp')
    })
  })

  describe('ordinal suffix (via spell level)', () => {
    it('renders 1st', async () => {
      const html = await renderModalContent({ ref_type: 'spell', level: 1, school: 'A' })
      expect(html).toContain('1st-level')
    })

    it('renders 2nd', async () => {
      const html = await renderModalContent({ ref_type: 'spell', level: 2, school: 'A' })
      expect(html).toContain('2nd-level')
    })

    it('renders 3rd', async () => {
      const html = await renderModalContent({ ref_type: 'spell', level: 3, school: 'A' })
      expect(html).toContain('3rd-level')
    })

    it('renders 4th', async () => {
      const html = await renderModalContent({ ref_type: 'spell', level: 4, school: 'A' })
      expect(html).toContain('4th-level')
    })

    it('renders 9th', async () => {
      const html = await renderModalContent({ ref_type: 'spell', level: 9, school: 'A' })
      expect(html).toContain('9th-level')
    })
  })
})
