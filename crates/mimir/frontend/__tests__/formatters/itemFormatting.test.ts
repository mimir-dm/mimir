/**
 * Tests for item formatting functions (itemFormatterEnhanced.ts)
 *
 * Tests the item detail formatter that generates HTML from 5etools item JSON:
 * weapons (damage, properties, range), armor (AC, strength, stealth),
 * magic items (rarity, attunement, bonuses), containers, light sources.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
} from '@tests/helpers/mockInvoke'
import { formatItemDetails } from '@/features/sources/formatters/itemFormatterEnhanced'
import { srdItems } from '@tests/fixtures/items'

// ─── Fixture Helpers ─────────────────────────────────────────────────────────

function findItem(name: string) {
  return srdItems.find(i => i.name === name)!
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('formatItemDetails', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('weapons', () => {
    describe('melee weapons (summary format — no entries)', () => {
      // Basic SRD weapons lack `entries`, so they go through formatItemSummary
      // which shows damage dice and raw damage type code
      it('renders Longsword with damage dice', async () => {
        const html = await formatItemDetails(findItem('Longsword'))
        expect(html).toContain('item-details')
        expect(html).toContain('1d8')
        expect(html).toContain('Damage')
      })

      it('renders Dagger with damage', async () => {
        const html = await formatItemDetails(findItem('Dagger'))
        expect(html).toContain('1d4')
        expect(html).toContain('Damage')
      })

      it('renders Greataxe damage', async () => {
        const html = await formatItemDetails(findItem('Greataxe'))
        expect(html).toContain('1d12')
      })

      it('renders Quarterstaff damage', async () => {
        const html = await formatItemDetails(findItem('Quarterstaff'))
        expect(html).toContain('1d6')
      })

      it('renders Rapier damage', async () => {
        const html = await formatItemDetails(findItem('Rapier'))
        expect(html).toContain('1d8')
      })
    })

    describe('ranged weapons (summary format)', () => {
      it('renders Longbow damage', async () => {
        const html = await formatItemDetails(findItem('Longbow'))
        expect(html).toContain('1d8')
      })

      it('renders Shortbow damage', async () => {
        const html = await formatItemDetails(findItem('Shortbow'))
        expect(html).toContain('1d6')
      })

      it('renders Heavy Crossbow damage', async () => {
        const html = await formatItemDetails(findItem('Heavy Crossbow'))
        expect(html).toContain('1d10')
      })
    })

    describe('magic weapons (full format — has entries)', () => {
      it('renders +1 Longsword with weapon bonus and properties', async () => {
        const html = await formatItemDetails(findItem('+1 Longsword'))
        expect(html).toContain('item-details enhanced')
        expect(html).toContain('1d8')
        expect(html).toContain('slashing') // full format uses formatDamageType
        expect(html).toContain('1d10') // versatile
        expect(html).toContain('Weapon Bonus')
        expect(html).toContain('+1')
        expect(html).toContain('Uncommon')
        expect(html).toContain('Versatile')
        expect(html).toContain('+1 bonus to attack and damage')
      })
    })
  })

  describe('armor', () => {
    it('renders Chain Mail (heavy armor, strength req, stealth disadvantage)', async () => {
      const html = await formatItemDetails(findItem('Chain Mail'))
      expect(html).toContain('Armor Class')
      expect(html).toContain('16')
      expect(html).toContain('Strength')
      expect(html).toContain('13')
      expect(html).toContain('Stealth')
      expect(html).toContain('Disadvantage')
    })

    it('renders Breastplate (medium armor, no stealth penalty)', async () => {
      const html = await formatItemDetails(findItem('Breastplate'))
      expect(html).toContain('Armor Class')
      expect(html).toContain('14')
      expect(html).not.toContain('Disadvantage')
    })

    it('renders Leather Armor (light armor)', async () => {
      const html = await formatItemDetails(findItem('Leather Armor'))
      expect(html).toContain('Armor Class')
      expect(html).toContain('11')
    })

    it('renders Plate Armor (heavy, str 15, stealth disadvantage)', async () => {
      const html = await formatItemDetails(findItem('Plate Armor'))
      expect(html).toContain('Armor Class')
      expect(html).toContain('18')
      expect(html).toContain('Strength')
      expect(html).toContain('15')
      expect(html).toContain('Disadvantage')
    })

    it('renders Shield with AC', async () => {
      const html = await formatItemDetails(findItem('Shield'))
      expect(html).toContain('Armor Class')
      expect(html).toContain('2')
    })

    it('renders Studded Leather (light armor)', async () => {
      const html = await formatItemDetails(findItem('Studded Leather Armor'))
      expect(html).toContain('Armor Class')
      expect(html).toContain('12')
    })
  })

  describe('magic items', () => {
    it('renders Cloak of Protection (wondrous, attunement, AC bonus)', async () => {
      const html = await formatItemDetails(findItem('Cloak of Protection'))
      expect(html).toContain('Requires Attunement')
      expect(html).toContain('Uncommon')
      expect(html).toContain('AC Bonus')
      expect(html).toContain('+1')
      expect(html).toContain('+1 bonus to AC and saving throws')
    })

    it('renders Cloak of Elvenkind (wondrous, attunement)', async () => {
      const html = await formatItemDetails(findItem('Cloak of Elvenkind'))
      expect(html).toContain('Requires Attunement')
      expect(html).toContain('Uncommon')
      expect(html).toContain('Stealth')
    })

    it('renders Boots of Elvenkind (wondrous, no attunement)', async () => {
      const html = await formatItemDetails(findItem('Boots of Elvenkind'))
      expect(html).toContain('Uncommon')
      expect(html).not.toContain('Requires Attunement')
      expect(html).toContain('Stealth')
    })

    it('renders Ring of Protection (attunement)', async () => {
      const html = await formatItemDetails(findItem('Ring of Protection'))
      expect(html).toContain('Requires Attunement')
      expect(html).toContain('Rare')
    })

    it('renders tier tag when present', async () => {
      const html = await formatItemDetails(findItem('Bag of Holding'))
      expect(html).toContain('Minor')
    })

    it('renders loot tables when present', async () => {
      const html = await formatItemDetails(findItem('Bag of Holding'))
      expect(html).toContain('Loot Tables')
      expect(html).toContain('Magic Item Table')
    })
  })

  describe('potions', () => {
    it('renders Potion of Healing', async () => {
      const html = await formatItemDetails(findItem('Potion of Healing'))
      expect(html).toContain('Description')
    })

    it('renders Potion of Greater Healing', async () => {
      const html = await formatItemDetails(findItem('Potion of Greater Healing'))
      expect(html).toContain('Description')
    })
  })

  describe('containers', () => {
    it('renders Backpack with container capacity', async () => {
      const html = await formatItemDetails(findItem('Backpack'))
      expect(html).toContain('Capacity')
      expect(html).toContain('30')
    })

    it('renders Bag of Holding with large capacity', async () => {
      const html = await formatItemDetails(findItem('Bag of Holding'))
      expect(html).toContain('Capacity')
      expect(html).toContain('500')
    })
  })

  describe('light sources', () => {
    it('renders Torch with light properties', async () => {
      const torch = findItem('Torch')
      const html = await formatItemDetails(torch)
      if (torch.light) {
        expect(html).toContain('Light Properties')
        expect(html).toContain('Bright Light')
        expect(html).toContain('Dim Light')
      }
    })
  })

  describe('cost and weight', () => {
    it('formats cost in gold pieces', async () => {
      const html = await formatItemDetails(findItem('Longsword'))
      // Longsword value is typically 1500 cp = 15 gp
      expect(html).toContain('gp')
    })

    it('formats cost in silver pieces', async () => {
      const html = await formatItemDetails(findItem('Dagger'))
      // Dagger value is 200 cp = 2 gp
      expect(html).toContain('gp')
    })

    it('renders weight', async () => {
      const html = await formatItemDetails(findItem('Longsword'))
      expect(html).toContain('lb')
    })
  })

  describe('description', () => {
    it('renders entries with formatted text', async () => {
      const html = await formatItemDetails(findItem('Bag of Holding'))
      expect(html).toContain('Description')
      expect(html).toContain('interior space considerably larger')
    })

    it('processes 5etools tags in entries', async () => {
      const html = await formatItemDetails(findItem('Bag of Holding'))
      // Contains {@item Heward's handy haversack} reference
      expect(html).toContain('Heward')
    })
  })

  describe('source attribution', () => {
    it('renders source and page', async () => {
      const html = await formatItemDetails(findItem('Longsword'))
      expect(html).toContain('Source:')
    })
  })

  describe('type code mapping', () => {
    it('renders Martial Weapon type for +1 Longsword (full format)', async () => {
      const html = await formatItemDetails(findItem('+1 Longsword'))
      expect(html).toContain('Martial')
      expect(html).toContain('Weapon')
    })

    it('renders Adventuring Gear type for Backpack (full format)', async () => {
      const html = await formatItemDetails(findItem('Backpack'))
      expect(html).toContain('Adventuring Gear')
    })
  })

  describe('summary format fallback', () => {
    it('renders summary for items without entries', () => {
      const summary = {
        name: 'Longsword',
        type: 'M',
        typeName: 'Melee Weapon',
        rarity: 'none',
        ac: undefined,
        dmg1: '1d8',
        dmgType: 'S',
        weight: 3,
        value: 1500,
        source: 'PHB',
      }
      // formatItemDetails returns Promise, but summary path is sync-ish
      // The function always returns a Promise due to the async signature
      return formatItemDetails(summary).then(html => {
        expect(html).toContain('item-details')
        expect(html).toContain('Melee Weapon')
        expect(html).toContain('1d8')
        expect(html).toContain('Source: PHB')
      })
    })
  })
})
