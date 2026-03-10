import { describe, it, expect } from 'vitest'
import {
  findClass,
  findSubclass,
  findSubclassByClass,
  findClassFeatures,
  findClassFeaturesAtLevel,
  findSubclassFeatures,
  findSubclassFeaturesAtLevel,
  findBackground,
  findRace,
  findItem,
  findSpell,
  findSpellsByLevel,
  findMonster,
  getTestCharacter,
  getTestCharacterByName,
  getAllTestCharacters,
  getHomebrewItem,
  getAllHomebrewItems,
  buildClassDataMap,
  srdClasses,
  srdSubclasses,
  srdBackgrounds,
  srdRaces,
  srdItems,
  srdSpells,
  srdMonsters,
} from '../fixtures'

describe('fixture helpers', () => {
  describe('class lookups', () => {
    it('finds Fighter by name', () => {
      const fighter = findClass('Fighter')
      expect(fighter.name).toBe('Fighter')
      expect(fighter.source).toBe('PHB')
      expect(fighter.hd).toBeDefined()
    })

    it('finds class case-insensitively', () => {
      const wizard = findClass('wizard')
      expect(wizard.name).toBe('Wizard')
    })

    it('throws for unknown class', () => {
      expect(() => findClass('Artificer')).toThrow('Fixture not found')
    })

    it('has all 12 SRD classes', () => {
      expect(srdClasses).toHaveLength(12)
    })
  })

  describe('subclass lookups', () => {
    it('finds Champion by name', () => {
      const champion = findSubclass('Champion')
      expect(champion.name).toBe('Champion')
      expect(champion.className).toBe('Fighter')
    })

    it('finds subclasses by class name', () => {
      const fighterSubs = findSubclassByClass('Fighter')
      expect(fighterSubs.length).toBeGreaterThan(0)
      expect(fighterSubs[0].className).toBe('Fighter')
    })

    it('has all 12 SRD subclasses', () => {
      expect(srdSubclasses).toHaveLength(12)
    })
  })

  describe('class feature lookups', () => {
    it('finds features for Fighter', () => {
      const features = findClassFeatures('Fighter')
      expect(features.length).toBeGreaterThan(0)
      expect(features[0].className).toBe('Fighter')
    })

    it('filters features by level', () => {
      const level3Features = findClassFeaturesAtLevel('Fighter', 3)
      const level20Features = findClassFeaturesAtLevel('Fighter', 20)
      expect(level3Features.length).toBeLessThanOrEqual(level20Features.length)
      for (const f of level3Features) {
        expect(f.level as number).toBeLessThanOrEqual(3)
      }
    })
  })

  describe('subclass feature lookups', () => {
    it('finds Thief features including children', () => {
      const features = findSubclassFeatures('Thief')
      const names = features.map((f) => f.name)
      expect(names).toContain('Fast Hands')
      expect(names).toContain('Second-Story Work')
    })

    it('finds Champion features including Improved Critical', () => {
      const features = findSubclassFeatures('Champion')
      const names = features.map((f) => f.name)
      expect(names).toContain('Improved Critical')
    })

    it('filters subclass features by level', () => {
      const level3 = findSubclassFeaturesAtLevel('Thief', 3)
      expect(level3.length).toBeGreaterThan(0)
      for (const f of level3) {
        expect(f.level as number).toBeLessThanOrEqual(3)
      }
    })
  })

  describe('background lookups', () => {
    it('finds Acolyte', () => {
      const bg = findBackground('Acolyte')
      expect(bg.name).toBe('Acolyte')
      expect(bg.source).toBe('PHB')
    })

    it('has 7 SRD backgrounds', () => {
      expect(srdBackgrounds).toHaveLength(7)
    })
  })

  describe('race lookups', () => {
    it('finds Human', () => {
      const human = findRace('Human')
      expect(human.name).toBe('Human')
    })

    it('includes subraces', () => {
      expect(srdRaces.length).toBeGreaterThan(9)
    })
  })

  describe('item lookups', () => {
    it('finds Longsword', () => {
      const item = findItem('Longsword')
      expect(item.name).toBe('Longsword')
    })

    it('has a good sample of items', () => {
      expect(srdItems.length).toBeGreaterThanOrEqual(20)
    })
  })

  describe('spell lookups', () => {
    it('finds Fireball', () => {
      const spell = findSpell('Fireball')
      expect(spell.name).toBe('Fireball')
      expect(spell._level).toBe(3)
    })

    it('finds cantrips', () => {
      const cantrips = findSpellsByLevel(0)
      expect(cantrips.length).toBeGreaterThan(0)
    })

    it('has spells across levels', () => {
      expect(srdSpells.length).toBeGreaterThanOrEqual(20)
    })
  })

  describe('monster lookups', () => {
    it('finds Goblin', () => {
      const goblin = findMonster('Goblin')
      expect(goblin.name).toBe('Goblin')
    })

    it('has a range of CRs', () => {
      expect(srdMonsters.length).toBeGreaterThanOrEqual(10)
    })
  })

  describe('test character lookups', () => {
    it('finds by id', () => {
      const fighter = getTestCharacter('test-fighter-champion-5')
      expect(fighter.name).toBe('Test Fighter')
      expect(fighter.level).toBe(5)
    })

    it('finds by name', () => {
      const rogue = getTestCharacterByName('Test Rogue')
      expect(rogue.id).toBe('test-rogue-thief-3')
    })

    it('has 4 test characters', () => {
      expect(getAllTestCharacters()).toHaveLength(4)
    })

    it('includes multiclass character', () => {
      const mc = getTestCharacter('test-multiclass-fighter3-rogue2')
      const classes = mc.classes as Array<{ class_name: string }>
      expect(classes).toHaveLength(2)
      expect(classes[0].class_name).toBe('Fighter')
      expect(classes[1].class_name).toBe('Rogue')
    })
  })

  describe('homebrew lookups', () => {
    it('finds homebrew item by name', () => {
      const item = getHomebrewItem('Blade of Testing')
      expect(item._isHomebrew).toBe(true)
    })

    it('has 3 homebrew items', () => {
      expect(getAllHomebrewItems()).toHaveLength(3)
    })
  })

  describe('bulk helpers', () => {
    it('buildClassDataMap creates lowercase-keyed map', () => {
      const map = buildClassDataMap(['Fighter', 'Wizard'])
      expect(map['fighter']).toBeDefined()
      expect(map['wizard']).toBeDefined()
      expect(map['fighter'].name).toBe('Fighter')
    })

    it('buildClassDataMap with no args includes all classes', () => {
      const map = buildClassDataMap()
      expect(Object.keys(map)).toHaveLength(12)
    })
  })
})
