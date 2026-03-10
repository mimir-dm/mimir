/**
 * Tests for monsterFormatterEnhanced.ts
 *
 * Tests the formatMonsterDetails function which generates HTML stat blocks
 * from full monster data. Requires invoke mock for image loading.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
} from '@tests/helpers/mockInvoke'
import { formatMonsterDetails } from '@/features/sources/formatters/monsterFormatterEnhanced'
import { srdMonsters } from '@tests/fixtures/monsters'

// ─── Fixture Helpers ─────────────────────────────────────────────────────────

function findMonster(name: string) {
  return srdMonsters.find(m => m.name === name)!
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('formatMonsterDetails', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('header section', () => {
    it('renders creature type line with size, type, alignment', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Huge')
      expect(html).toContain('dragon')
      expect(html).toContain('monster-header')
    })

    it('renders CR', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Challenge')
      expect(html).toContain('17')
    })
  })

  describe('stat block', () => {
    it('renders AC, HP, Speed', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('AC')
      expect(html).toContain('19')
      expect(html).toContain('HP')
      expect(html).toContain('256')
      expect(html).toContain('Speed')
    })
  })

  describe('ability scores', () => {
    it('renders all six ability scores', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('STR')
      expect(html).toContain('DEX')
      expect(html).toContain('CON')
      expect(html).toContain('INT')
      expect(html).toContain('WIS')
      expect(html).toContain('CHA')
      // STR 27 (+8)
      expect(html).toContain('27')
      expect(html).toContain('+8')
    })
  })

  describe('secondary properties', () => {
    it('renders saving throws', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Saving Throws')
    })

    it('renders skills', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Skills')
    })

    it('renders senses', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Senses')
      expect(html).toContain('blindsight 60 ft.')
      expect(html).toContain('darkvision 120 ft.')
    })

    it('renders languages', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Languages')
      expect(html).toContain('Common')
      expect(html).toContain('Draconic')
    })
  })

  describe('traits', () => {
    it('renders traits section', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Traits')
      expect(html).toContain('Legendary Resistance (3/Day)')
    })
  })

  describe('actions', () => {
    it('renders actions section', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Actions')
      expect(html).toContain('Multiattack')
      expect(html).toContain('Bite')
      expect(html).toContain('Claw')
      expect(html).toContain('Tail')
    })

    it('renders Fire Breath action', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Fire Breath')
      expect(html).toContain('60-foot cone')
    })
  })

  describe('legendary actions', () => {
    it('renders legendary actions for dragon', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Legendary Actions')
      expect(html).toContain('Detect')
      expect(html).toContain('Tail Attack')
      expect(html).toContain('Wing Attack')
    })

    it('does not render legendary actions for non-legendary creatures', async () => {
      const html = await formatMonsterDetails(findMonster('Goblin'))
      expect(html).not.toContain('Legendary Actions')
    })
  })

  describe('environment', () => {
    it('renders environment when present', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Environment')
      expect(html).toContain('mountain')
      expect(html).toContain('hill')
    })
  })

  describe('source attribution', () => {
    it('renders source and page number', async () => {
      const html = await formatMonsterDetails(findMonster('Adult Red Dragon'))
      expect(html).toContain('Source: MM')
      expect(html).toContain('p. 98')
    })
  })

  describe('simple monster (Goblin)', () => {
    it('renders complete stat block', async () => {
      const html = await formatMonsterDetails(findMonster('Goblin'))
      expect(html).toContain('Small')
      expect(html).toContain('AC')
      expect(html).toContain('HP')
      expect(html).toContain('Speed')
      expect(html).toContain('Traits')
      expect(html).toContain('Nimble Escape')
      expect(html).toContain('Actions')
      expect(html).toContain('Scimitar')
      expect(html).toContain('Shortbow')
    })
  })

  describe('undead (Zombie)', () => {
    it('renders Undead Fortitude trait', async () => {
      const html = await formatMonsterDetails(findMonster('Zombie'))
      expect(html).toContain('Undead Fortitude')
    })
  })

  describe('images', () => {
    it('renders image when serve_book_image succeeds', async () => {
      mockCommand('serve_book_image', 'data:image/webp;base64,abc123')
      // Need fluffImages at top level for the formatter to find them
      const dragon = {
        ...findMonster('Adult Red Dragon'),
        fluffImages: [{ href: { path: 'bestiary/MM/Adult Red Dragon.webp', type: 'internal' }, type: 'image' }],
      }
      const html = await formatMonsterDetails(dragon)
      expect(html).toContain('monster-image')
      expect(html).toContain('data:image/webp;base64,abc123')
    })
  })

  describe('summary format fallback', () => {
    it('renders summary for monsters without ability scores', async () => {
      const summary = {
        name: 'Goblin',
        size: 'S',
        creature_type: 'humanoid',
        alignment: 'neutral evil',
        cr: '1/4',
        hp: '7',
        ac: '15',
        speed: '30 ft.',
        source: 'MM',
      }
      const html = await formatMonsterDetails(summary as any)
      expect(html).toContain('Small')
      expect(html).toContain('humanoid')
      expect(html).toContain('Source: MM')
    })
  })
})
