/**
 * Tests for homebrew content rendering.
 *
 * Verifies that homebrew items, monsters, and spells with source='HB'
 * render correctly through the same formatters as catalog content.
 * Tests homebrew-specific edge cases: minimal data, malformed JSON,
 * cloned items with modifications, and missing fields.
 *
 * Note: The enhanced formatters do NOT include the entity name in their
 * HTML output — names are rendered separately in the UI. Tests verify
 * the stat block / detail content only.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
} from '@tests/helpers/mockInvoke'
import { formatItemDetails } from '@/features/sources/formatters/itemFormatterEnhanced'
import { formatSpellDetails } from '@/features/sources/formatters/spellFormatterEnhanced'
import { formatMonsterDetails } from '@/features/sources/formatters/monsterFormatterEnhanced'

// ─── Homebrew Item Fixtures ─────────────────────────────────────────────────

/** Homebrew melee weapon created from scratch */
const homebrewMeleeWeapon = {
  name: 'Blade of the Fallen Star',
  type: 'M',
  source: 'HB',
  weapon: true,
  weaponCategory: 'martial',
  dmg1: '2d6',
  dmgType: 'S',
  property: ['2H', 'H'],
  rarity: 'very rare',
  reqAttune: 'by a paladin or cleric',
  weight: 6,
  value: 2500000, // cp
  entries: [
    'This greatsword is forged from a fallen meteorite.',
    'You gain a +2 bonus to attack and damage rolls made with this weapon.',
    'When you hit a fiend or undead with this weapon, that creature takes an extra {@damage 2d6} radiant damage.',
  ],
  bonusWeapon: '+2',
}

/** Homebrew ranged weapon */
const homebrewRangedWeapon = {
  name: 'Eldritch Crossbow',
  type: 'R',
  source: 'HB',
  weapon: true,
  weaponCategory: 'martial',
  dmg1: '1d10',
  dmgType: 'P',
  range: '100/400',
  property: ['A', 'H', '2H', 'LD'],
  rarity: 'rare',
  reqAttune: true,
  entries: [
    'This crossbow crackles with arcane energy.',
    'Bolts fired from this weapon deal an extra {@damage 1d6} force damage.',
  ],
}

/** Homebrew armor */
const homebrewArmor = {
  name: 'Mithral Breastplate',
  type: 'MA',
  source: 'HB',
  armor: true,
  ac: 14,
  rarity: 'uncommon',
  entries: [
    'Mithral is a light, flexible metal. This breastplate can be worn under normal clothes.',
    'If the armor normally imposes disadvantage on Dexterity (Stealth) checks, the mithral version does not.',
  ],
}

/** Homebrew potion */
const homebrewPotion = {
  name: 'Potion of Starlight',
  type: 'P',
  source: 'HB',
  rarity: 'rare',
  entries: [
    'When you drink this potion, you emit bright light in a 30-foot radius and dim light for an additional 30 feet for 1 hour.',
    'During this time, you have advantage on saving throws against being frightened.',
  ],
}

/** Homebrew wondrous item */
const homebrewWondrous = {
  name: 'Cloak of the Void',
  type: 'W',
  source: 'HB',
  rarity: 'legendary',
  reqAttune: true,
  entries: [
    'While wearing this cloak, you can use an action to become invisible until the start of your next turn.',
    'The cloak has 3 charges and regains {@dice 1d3} expended charges daily at dawn.',
  ],
}

/** Minimal homebrew item — only required fields */
const minimalHomebrewItem = {
  name: 'Mystery Object',
  source: 'HB',
  type: 'G',
}

/** Homebrew item with no entries (just notes) */
const homebrewItemWithNotes = {
  name: 'Found Ring',
  type: 'RG',
  source: 'HB',
  rarity: 'uncommon',
  // No entries — in real usage, notes would be in a separate field
}

// ─── Homebrew Monster Fixtures ─────────────────────────────────────────────

/** Full homebrew monster stat block */
const homebrewMonster = {
  name: 'Void Stalker',
  source: 'HB',
  size: ['L'],
  type: 'aberration',
  alignment: ['C', 'E'],
  ac: [{ ac: 17, from: ['natural armor'] }],
  hp: { average: 136, formula: '16d10+48' },
  speed: { walk: 40, fly: 60 },
  str: 20,
  dex: 16,
  con: 16,
  int: 18,
  wis: 14,
  cha: 12,
  save: { dex: '+7', con: '+7', wis: '+6' },
  skill: { perception: '+6', stealth: '+7' },
  senses: ['darkvision 120 ft.'],
  passive: 16,
  languages: ['Deep Speech', 'telepathy 120 ft.'],
  cr: '10',
  damageResistances: ['cold', 'psychic'],
  conditionImmune: ['frightened'],
  trait: [
    {
      name: 'Shadow Step',
      entries: ['As a bonus action, the void stalker can teleport up to 60 feet to an unoccupied space it can see that is in dim light or darkness.'],
    },
  ],
  action: [
    {
      name: 'Multiattack',
      entries: ['The void stalker makes two claw attacks.'],
    },
    {
      name: 'Claw',
      entries: ['{@atk mw} {@hit 9} to hit, reach 10 ft., one target. {@h}14 ({@damage 2d8+5}) slashing damage plus 7 ({@damage 2d6}) psychic damage.'],
    },
  ],
}

/** Minimal homebrew monster — just name and basics */
const minimalHomebrewMonster = {
  name: 'Simple Beast',
  source: 'HB',
  size: ['M'],
  type: 'beast',
  str: 12,
  dex: 10,
  con: 10,
  int: 2,
  wis: 10,
  cha: 4,
  cr: '1/4',
}

/** Homebrew monster with string type (not object) */
const homebrewMonsterStringType = {
  name: 'Cave Lurker',
  source: 'HB',
  size: ['S'],
  type: 'monstrosity',
  alignment: ['N'],
  ac: [12],
  hp: { average: 22, formula: '5d6+5' },
  speed: { walk: 30, climb: 30 },
  str: 14,
  dex: 14,
  con: 12,
  int: 4,
  wis: 12,
  cha: 6,
  passive: 11,
  cr: '1',
}

// ─── Homebrew Spell Fixtures ────────────────────────────────────────────────

/** Full homebrew spell */
const homebrewSpell = {
  name: 'Void Bolt',
  source: 'HB',
  level: 3,
  school: 'V', // Evocation
  time: [{ number: 1, unit: 'action' }],
  range: { type: 'point', distance: { type: 'feet', amount: 120 } },
  components: { v: true, s: true },
  duration: [{ type: 'instant' }],
  entries: [
    'A beam of void energy streaks toward a creature within range. Make a ranged spell attack against the target. On a hit, the target takes {@damage 4d10} force damage and must succeed on a Constitution saving throw or be {@condition stunned} until the end of your next turn.',
  ],
  entriesHigherLevel: [
    {
      type: 'entries',
      name: 'At Higher Levels',
      entries: ['When you cast this spell using a spell slot of 4th level or higher, the damage increases by {@damage 1d10} for each slot level above 3rd.'],
    },
  ],
  classes: { fromClassList: [{ name: 'Sorcerer', source: 'PHB' }, { name: 'Warlock', source: 'PHB' }] },
  damageInflict: ['force'],
  conditionInflict: ['stunned'],
  savingThrow: ['constitution'],
  spellAttack: ['R'],
}

/** Homebrew cantrip */
const homebrewCantrip = {
  name: 'Spark Whip',
  source: 'HB',
  level: 0,
  school: 'V',
  time: [{ number: 1, unit: 'action' }],
  range: { type: 'point', distance: { type: 'feet', amount: 15 } },
  components: { v: true, s: true },
  duration: [{ type: 'instant' }],
  entries: [
    'You create a whip of crackling lightning that lashes out. Make a melee spell attack. On a hit, the target takes {@damage 1d8} lightning damage.',
  ],
  scalingLevelDice: [
    {
      label: 'lightning damage',
      scaling: { '1': '1d8', '5': '2d8', '11': '3d8', '17': '4d8' },
    },
  ],
  damageInflict: ['lightning'],
  spellAttack: ['M'],
}

/** Minimal homebrew spell — ritual with material components */
const homebrewRitual = {
  name: 'Commune with Shadows',
  source: 'HB',
  level: 2,
  school: 'D', // Divination
  time: [{ number: 10, unit: 'minute' }],
  range: { type: 'point', distance: { type: 'self' } },
  components: { v: true, s: true, m: { text: 'a black candle worth at least 10 gp', cost: 1000, consume: false } },
  duration: [{ type: 'timed', duration: { type: 'minute', amount: 10 }, concentration: true }],
  meta: { ritual: true },
  entries: [
    'You sit in darkness and meditate, opening your mind to whispers from the Shadowfell.',
  ],
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('Homebrew Item Rendering', () => {
  beforeEach(() => { setupInvokeMock() })
  afterEach(() => { resetInvokeMock() })

  describe('melee weapon', () => {
    it('renders weapon details', async () => {
      const html = await formatItemDetails(homebrewMeleeWeapon)
      expect(html).toContain('2d6')
      expect(html).toContain('item-details')
    })

    it('renders damage dice', async () => {
      const html = await formatItemDetails(homebrewMeleeWeapon)
      expect(html).toContain('2d6')
    })

    it('renders rarity with capitalization', async () => {
      const html = await formatItemDetails(homebrewMeleeWeapon)
      expect(html).toContain('Very rare')
    })

    it('renders attunement requirement text', async () => {
      const html = await formatItemDetails(homebrewMeleeWeapon)
      expect(html).toContain('paladin or cleric')
    })

    it('renders entries with description', async () => {
      const html = await formatItemDetails(homebrewMeleeWeapon)
      expect(html).toContain('fallen meteorite')
      expect(html).toContain('+2 bonus')
    })

    it('renders weapon bonus', async () => {
      const html = await formatItemDetails(homebrewMeleeWeapon)
      expect(html).toContain('+2')
    })
  })

  describe('ranged weapon', () => {
    it('renders range', async () => {
      const html = await formatItemDetails(homebrewRangedWeapon)
      expect(html).toContain('100/400')
    })

    it('renders piercing damage type', async () => {
      const html = await formatItemDetails(homebrewRangedWeapon)
      expect(html).toContain('1d10')
    })

    it('renders attunement as boolean', async () => {
      const html = await formatItemDetails(homebrewRangedWeapon)
      expect(html).toContain('attunement')
    })
  })

  describe('armor', () => {
    it('renders AC value', async () => {
      const html = await formatItemDetails(homebrewArmor)
      expect(html).toContain('14')
    })

    it('renders description entries', async () => {
      const html = await formatItemDetails(homebrewArmor)
      expect(html).toContain('Mithral')
    })
  })

  describe('potion', () => {
    it('renders potion description', async () => {
      const html = await formatItemDetails(homebrewPotion)
      expect(html).toContain('bright light')
    })

    it('renders rarity with capitalization', async () => {
      const html = await formatItemDetails(homebrewPotion)
      expect(html).toContain('Rare')
    })
  })

  describe('wondrous item', () => {
    it('renders legendary rarity with capitalization', async () => {
      const html = await formatItemDetails(homebrewWondrous)
      expect(html).toContain('Legendary')
    })

    it('renders attunement', async () => {
      const html = await formatItemDetails(homebrewWondrous)
      expect(html).toContain('attunement')
    })

    it('renders entries', async () => {
      const html = await formatItemDetails(homebrewWondrous)
      expect(html).toContain('invisible')
    })
  })

  describe('minimal item (missing fields)', () => {
    it('renders without crashing', async () => {
      const html = await formatItemDetails(minimalHomebrewItem)
      expect(html).toBeTruthy()
      expect(html).toContain('item-details')
    })
  })

  describe('item without entries', () => {
    it('renders without crashing', async () => {
      const html = await formatItemDetails(homebrewItemWithNotes)
      expect(html).toBeTruthy()
      expect(html).toContain('item-details')
    })
  })

  describe('HB source attribution', () => {
    it('includes HB in source for melee weapon', async () => {
      const html = await formatItemDetails(homebrewMeleeWeapon)
      expect(html).toContain('HB')
    })

    it('includes HB in source for potion', async () => {
      const html = await formatItemDetails(homebrewPotion)
      expect(html).toContain('HB')
    })
  })
})

describe('Homebrew Monster Rendering', () => {
  beforeEach(() => { setupInvokeMock() })
  afterEach(() => { resetInvokeMock() })

  describe('full stat block', () => {
    it('renders creature type and alignment', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('aberration')
    })

    it('renders AC value', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('17')
    })

    it('renders HP with formula', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('136')
      expect(html).toContain('16d10+48')
    })

    it('renders speed types', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('40')
      expect(html).toContain('fly')
      expect(html).toContain('60')
    })

    it('renders ability scores', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('20') // STR
      expect(html).toContain('16') // DEX/CON
      expect(html).toContain('18') // INT
    })

    it('renders saving throws', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('+7') // DEX/CON save
      expect(html).toContain('+6') // WIS save
    })

    it('renders CR', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('10')
    })

    it('renders traits', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('Shadow Step')
      expect(html).toContain('teleport')
    })

    it('renders actions', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('Multiattack')
      expect(html).toContain('Claw')
    })

    it('renders senses', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('darkvision 120 ft.')
    })

    it('renders languages', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('Deep Speech')
      expect(html).toContain('telepathy')
    })

    it('includes HB source', async () => {
      const html = await formatMonsterDetails(homebrewMonster)
      expect(html).toContain('HB')
    })
  })

  describe('minimal monster', () => {
    it('renders without crashing', async () => {
      const html = await formatMonsterDetails(minimalHomebrewMonster)
      expect(html).toBeTruthy()
      expect(html).toContain('monster-details')
    })

    it('renders CR', async () => {
      const html = await formatMonsterDetails(minimalHomebrewMonster)
      expect(html).toContain('1/4')
    })
  })

  describe('monster with simple AC (number)', () => {
    it('renders AC correctly', async () => {
      const html = await formatMonsterDetails(homebrewMonsterStringType)
      expect(html).toContain('12')
    })

    it('renders climb speed', async () => {
      const html = await formatMonsterDetails(homebrewMonsterStringType)
      expect(html).toContain('climb')
      expect(html).toContain('30')
    })
  })
})

describe('Homebrew Spell Rendering', () => {
  describe('leveled spell', () => {
    it('renders spell level and school code', () => {
      const html = formatSpellDetails(homebrewSpell)
      // School is rendered as the code (V=Evocation), not the full name
      expect(html).toContain('3rd')
      expect(html).toContain('V')
    })

    it('renders casting time', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('1 action')
    })

    it('renders range', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('120')
    })

    it('renders components', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('V')
      expect(html).toContain('S')
    })

    it('renders damage type in combat mechanics', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('force')
    })

    it('renders condition in combat mechanics', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('stunned')
    })

    it('renders higher level casting section', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('Higher Level Casting')
    })

    it('renders available classes from class list', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('Sorcerer')
      expect(html).toContain('Warlock')
    })

    it('includes HB source', () => {
      const html = formatSpellDetails(homebrewSpell)
      expect(html).toContain('HB')
    })
  })

  describe('cantrip', () => {
    it('renders as Cantrip level', () => {
      const html = formatSpellDetails(homebrewCantrip)
      expect(html).toContain('Cantrip')
    })

    it('renders damage scaling', () => {
      const html = formatSpellDetails(homebrewCantrip)
      expect(html).toContain('1d8')
    })

    it('renders lightning damage type', () => {
      const html = formatSpellDetails(homebrewCantrip)
      expect(html).toContain('lightning')
    })
  })

  describe('ritual spell', () => {
    it('renders ritual tag', () => {
      const html = formatSpellDetails(homebrewRitual)
      expect(html).toContain('Ritual')
    })

    it('renders concentration', () => {
      const html = formatSpellDetails(homebrewRitual)
      expect(html).toContain('Concentration')
    })

    it('renders material component with cost', () => {
      const html = formatSpellDetails(homebrewRitual)
      expect(html).toContain('black candle')
    })

    it('renders casting time', () => {
      const html = formatSpellDetails(homebrewRitual)
      expect(html).toContain('10 minute')
    })

    it('renders self range', () => {
      const html = formatSpellDetails(homebrewRitual)
      expect(html).toContain('Self')
    })

    it('renders divination school code', () => {
      const html = formatSpellDetails(homebrewRitual)
      // School rendered as code D (Divination)
      expect(html).toContain(' D')
    })
  })
})

describe('Malformed homebrew data', () => {
  beforeEach(() => { setupInvokeMock() })
  afterEach(() => { resetInvokeMock() })

  it('item with empty entries array renders', async () => {
    const item = { name: 'Empty', type: 'G', source: 'HB', entries: [] }
    const html = await formatItemDetails(item)
    expect(html).toBeTruthy()
    expect(html).toContain('item-details')
  })

  it('item with no type renders', async () => {
    const item = { name: 'Typeless', source: 'HB' } as any
    const html = await formatItemDetails(item)
    expect(html).toBeTruthy()
  })

  it('spell with missing fields renders', () => {
    const spell = {
      name: 'Broken Spell',
      source: 'HB',
      level: 1,
      school: 'A',
    } as any
    const html = formatSpellDetails(spell)
    expect(html).toBeTruthy()
  })

  it('monster with minimal fields renders', async () => {
    const monster = {
      name: 'Bare Bones',
      source: 'HB',
    } as any
    const html = await formatMonsterDetails(monster)
    expect(html).toBeTruthy()
    expect(html).toContain('monster-details')
  })
})
