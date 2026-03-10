import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { nextTick } from 'vue'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandHandler,
  expectCommandCalled,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import CharacterStatsTab from '@/features/characters/components/sheet/CharacterStatsTab.vue'
import type { Character, CharacterInventory } from '@/types/character'

// ─── Test data factories ────────────────────────────────────────────────────

function makeCharacter(overrides: Partial<Character> = {}): Character {
  return {
    id: 'test-fighter-1',
    campaign_id: 'campaign-1',
    name: 'Test Fighter',
    is_npc: 0,
    player_name: 'Player',
    race_name: 'Human',
    race_source: 'PHB',
    background_name: 'Soldier',
    background_source: 'PHB',
    strength: 16,
    dexterity: 14,
    constitution: 14,
    intelligence: 10,
    wisdom: 12,
    charisma: 8,
    cp: 0,
    sp: 0,
    ep: 0,
    gp: 50,
    pp: 0,
    traits: 'I am always polite.',
    ideals: 'Honor.',
    bonds: 'I protect the weak.',
    flaws: 'I am too trusting.',
    role: null,
    location: null,
    faction: null,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    classes: [
      {
        id: 'class-1',
        character_id: 'test-fighter-1',
        class_name: 'Fighter',
        class_source: 'PHB',
        level: 5,
        subclass_name: 'Champion',
        subclass_source: 'PHB',
        starting_class: 1,
      },
    ],
    proficiencies: [
      { id: 'p1', character_id: 'test-fighter-1', proficiency_type: 'skill', name: 'Athletics', expertise: 0 },
      { id: 'p2', character_id: 'test-fighter-1', proficiency_type: 'skill', name: 'Intimidation', expertise: 0 },
      { id: 'p3', character_id: 'test-fighter-1', proficiency_type: 'save', name: 'strength', expertise: 0 },
      { id: 'p4', character_id: 'test-fighter-1', proficiency_type: 'save', name: 'constitution', expertise: 0 },
      { id: 'p5', character_id: 'test-fighter-1', proficiency_type: 'armor', name: 'All Armor', expertise: 0 },
      { id: 'p6', character_id: 'test-fighter-1', proficiency_type: 'armor', name: 'Shields', expertise: 0 },
      { id: 'p7', character_id: 'test-fighter-1', proficiency_type: 'weapon', name: 'Simple Weapons', expertise: 0 },
      { id: 'p8', character_id: 'test-fighter-1', proficiency_type: 'weapon', name: 'Martial Weapons', expertise: 0 },
      { id: 'p9', character_id: 'test-fighter-1', proficiency_type: 'language', name: 'Common', expertise: 0 },
    ],
    ...overrides,
  }
}

function makeInventoryItem(overrides: Partial<CharacterInventory> = {}): CharacterInventory {
  return {
    id: 'inv-1',
    character_id: 'test-fighter-1',
    item_name: 'Longsword',
    item_source: 'PHB',
    quantity: 1,
    equipped: 1,
    attuned: 0,
    notes: null,
    ...overrides,
  }
}

interface ClassFeature {
  name: string
  source: string
  class_name: string
  class_source: string
  level: number
  data: string
  subclass_name?: string
  subclass_short_name?: string
  subclass_source?: string
}

function makeClassFeature(overrides: Partial<ClassFeature> = {}): ClassFeature {
  return {
    name: 'Second Wind',
    source: 'PHB',
    class_name: 'Fighter',
    class_source: 'PHB',
    level: 1,
    data: '{}',
    ...overrides,
  }
}

const defaultProps = () => ({
  character: makeCharacter(),
  inventory: [] as CharacterInventory[],
  classFeatures: [] as ClassFeature[],
  speed: 30,
  characterIsSpellcaster: false,
  spellcastingAbility: null as string | null,
  spellSaveDC: null as number | null,
  spellAttackBonus: null as number | null,
})

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('CharacterStatsTab', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  // ── Ability Scores ──────────────────────────────────────────────────────

  describe('ability scores', () => {
    it('renders all six ability scores', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })

      expect(wrapper.text()).toContain('STR')
      expect(wrapper.text()).toContain('DEX')
      expect(wrapper.text()).toContain('CON')
      expect(wrapper.text()).toContain('INT')
      expect(wrapper.text()).toContain('WIS')
      expect(wrapper.text()).toContain('CHA')
    })

    it('renders ability score values', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })

      // STR 16, DEX 14, CON 14, INT 10, WIS 12, CHA 8
      const abilityBoxes = wrapper.findAll('.ability-box')
      expect(abilityBoxes).toHaveLength(6)

      // Check STR box: value 16, modifier +3
      const strBox = abilityBoxes[0]
      expect(strBox.find('.ability-value').text()).toBe('16')
      expect(strBox.find('.ability-modifier').text()).toBe('+3')
    })

    it('renders negative modifiers correctly', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })

      // CHA 8 → modifier -1
      const chaBox = wrapper.findAll('.ability-box')[5]
      expect(chaBox.find('.ability-value').text()).toBe('8')
      expect(chaBox.find('.ability-modifier').text()).toBe('-1')
    })

    it('renders zero modifier correctly', () => {
      const props = defaultProps()
      props.character = makeCharacter({ intelligence: 10 })
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })

      // INT 10 → modifier +0
      const intBox = wrapper.findAll('.ability-box')[3]
      expect(intBox.find('.ability-value').text()).toBe('10')
      expect(intBox.find('.ability-modifier').text()).toBe('+0')
    })
  })

  // ── Combat Stats ────────────────────────────────────────────────────────

  describe('combat stats', () => {
    it('renders speed', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('30 ft')
    })

    it('renders initiative from DEX modifier', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      // DEX 14 → +2 initiative
      const combatStats = wrapper.findAll('.combat-stat')
      const initiative = combatStats.find((s) => s.find('.stat-label').text() === 'Initiative')
      expect(initiative?.find('.stat-value').text()).toBe('+2')
    })

    it('renders proficiency bonus based on level', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      // Level 5 → +3 proficiency bonus
      const combatStats = wrapper.findAll('.combat-stat')
      const prof = combatStats.find((s) => s.find('.stat-label').text() === 'Proficiency')
      expect(prof?.find('.stat-value').text()).toBe('+3')
    })

    it('renders hit dice for single class', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      // Fighter 5 → 5d10
      const combatStats = wrapper.findAll('.combat-stat')
      const hitDice = combatStats.find((s) => s.find('.stat-label').text() === 'Hit Dice')
      expect(hitDice?.find('.stat-value').text()).toBe('5d10')
    })

    it('renders hit dice for multiclass', () => {
      const props = defaultProps()
      props.character = makeCharacter({
        classes: [
          { id: 'c1', character_id: 'test-1', class_name: 'Fighter', class_source: 'PHB', level: 3, subclass_name: 'Champion', subclass_source: 'PHB', starting_class: 1 },
          { id: 'c2', character_id: 'test-1', class_name: 'Rogue', class_source: 'PHB', level: 2, subclass_name: null, subclass_source: null, starting_class: 0 },
        ],
      })
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).toContain('3d10 + 2d8')
    })

    it('renders passive perception', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      // WIS 12 → +1, not proficient in Perception → passive = 10 + 1 = 11
      const combatStats = wrapper.findAll('.combat-stat')
      const pp = combatStats.find((s) => s.find('.stat-label').text() === 'Passive Perception')
      expect(pp?.find('.stat-value').text()).toBe('11')
    })

    it('renders AC without armor (10 + DEX mod)', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      // No armor in inventory, DEX 14 → AC 12
      const combatStats = wrapper.findAll('.combat-stat')
      const ac = combatStats.find((s) => s.find('.stat-label').text() === 'Armor Class')
      expect(ac?.find('.stat-value').text()).toBe('12')
    })

    it('renders AC with equipped armor', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ item_name: 'Chain Mail', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      // Chain mail = AC 16
      const combatStats = wrapper.findAll('.combat-stat')
      const ac = combatStats.find((s) => s.find('.stat-label').text() === 'Armor Class')
      expect(ac?.find('.stat-value').text()).toBe('16')
    })

    it('shows armor name when equipped', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ item_name: 'Chain Mail', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).toContain('Chain Mail')
    })
  })

  // ── Saving Throws ────────────────────────────────────────────────────────

  describe('saving throws', () => {
    it('renders all six saves', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      const saves = wrapper.findAll('.save-item')
      expect(saves).toHaveLength(6)
    })

    it('highlights proficient saves', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      const saves = wrapper.findAll('.save-item')

      // Fighter is proficient in STR and CON saves
      const strSave = saves[0]
      expect(strSave.find('.save-proficient').classes()).toContain('active')

      // DEX save is not proficient
      const dexSave = saves[1]
      expect(dexSave.find('.save-proficient').classes()).not.toContain('active')
    })

    it('calculates save bonus with proficiency', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      const saves = wrapper.findAll('.save-item')

      // STR save: mod +3 + prof +3 = +6
      const strSave = saves[0]
      expect(strSave.find('.save-bonus').text()).toBe('+6')

      // DEX save: mod +2, no proficiency = +2
      const dexSave = saves[1]
      expect(dexSave.find('.save-bonus').text()).toBe('+2')
    })
  })

  // ── Skills ──────────────────────────────────────────────────────────────

  describe('skills', () => {
    it('renders all 18 skills', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      const skills = wrapper.findAll('.skill-item')
      expect(skills).toHaveLength(18)
    })

    it('highlights proficient skills', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      const skills = wrapper.findAll('.skill-item')

      // Athletics is proficient
      const athletics = skills.find((s) => s.find('.skill-name').text() === 'Athletics')
      expect(athletics?.find('.skill-proficient').classes()).toContain('active')

      // Arcana is not proficient
      const arcana = skills.find((s) => s.find('.skill-name').text() === 'Arcana')
      expect(arcana?.find('.skill-proficient').classes()).not.toContain('active')
    })

    it('calculates skill bonus with proficiency', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      const skills = wrapper.findAll('.skill-item')

      // Athletics: STR mod +3 + prof +3 = +6
      const athletics = skills.find((s) => s.find('.skill-name').text() === 'Athletics')
      expect(athletics?.find('.skill-bonus').text()).toBe('+6')

      // Arcana: INT mod +0, no proficiency = +0
      const arcana = skills.find((s) => s.find('.skill-name').text() === 'Arcana')
      expect(arcana?.find('.skill-bonus').text()).toBe('+0')
    })

    it('shows expertise indicator', () => {
      const props = defaultProps()
      props.character = makeCharacter({
        proficiencies: [
          ...makeCharacter().proficiencies,
          { id: 'pe1', character_id: 'test-fighter-1', proficiency_type: 'skill', name: 'Stealth', expertise: 1 },
        ],
      })
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      const skills = wrapper.findAll('.skill-item')
      const stealth = skills.find((s) => s.find('.skill-name').text() === 'Stealth')
      expect(stealth?.find('.skill-proficient').classes()).toContain('expertise')
      expect(stealth?.find('.skill-proficient').text()).toBe('**')
    })
  })

  // ── Attacks ─────────────────────────────────────────────────────────────

  describe('attacks', () => {
    it('shows no attacks section when no weapons equipped', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      expect(wrapper.find('.attacks-list').exists()).toBe(false)
    })

    it('renders attacks from equipped weapons', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'w1', item_name: 'Longsword', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      const attacks = wrapper.findAll('.attack-item')
      expect(attacks).toHaveLength(1)
      expect(attacks[0].find('.attack-name').text()).toBe('Longsword')
    })

    it('calculates melee attack bonus correctly', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'w1', item_name: 'Longsword', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      // STR mod +3 + prof +3 = +6
      const attack = wrapper.find('.attack-item')
      expect(attack.find('.attack-bonus').text()).toBe('+6')
    })

    it('calculates weapon damage correctly', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'w1', item_name: 'Longsword', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      // Longsword: 1d8+3 (STR mod)
      const attack = wrapper.find('.attack-item')
      expect(attack.find('.attack-damage').text()).toBe('1d8+3')
    })

    it('uses DEX for ranged weapons', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'w1', item_name: 'Longbow', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      // DEX mod +2 + prof +3 = +5
      const attack = wrapper.find('.attack-item')
      expect(attack.find('.attack-bonus').text()).toBe('+5')
      expect(attack.find('.attack-damage').text()).toBe('1d8+2')
    })

    it('uses STR for finesse weapons when STR is higher', () => {
      const props = defaultProps()
      // STR 16 (+3) > DEX 14 (+2), so STR is used for shortsword (finesse)
      props.inventory = [
        makeInventoryItem({ id: 'w1', item_name: 'Shortsword', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      // STR mod +3 + prof +3 = +6
      expect(wrapper.find('.attack-item .attack-bonus').text()).toBe('+6')
    })

    it('uses DEX for finesse weapons when DEX is higher', () => {
      const props = defaultProps()
      props.character = makeCharacter({ strength: 10, dexterity: 18 })
      props.inventory = [
        makeInventoryItem({ id: 'w1', item_name: 'Dagger', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      // DEX mod +4 + prof +3 = +7
      expect(wrapper.find('.attack-item .attack-bonus').text()).toBe('+7')
    })

    it('ignores unequipped weapons', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'w1', item_name: 'Longsword', equipped: 0 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.find('.attacks-list').exists()).toBe(false)
    })
  })

  // ── Proficiencies ──────────────────────────────────────────────────────

  describe('proficiencies', () => {
    it('renders armor proficiencies', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('All Armor')
      expect(wrapper.text()).toContain('Shields')
    })

    it('renders weapon proficiencies', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Simple Weapons')
      expect(wrapper.text()).toContain('Martial Weapons')
    })

    it('renders languages', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Common')
    })

    it('shows empty message when no proficiencies', () => {
      const props = defaultProps()
      props.character = makeCharacter({ proficiencies: [] })
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).toContain('No proficiencies recorded')
    })
  })

  // ── Class Features ─────────────────────────────────────────────────────

  describe('class features', () => {
    it('renders class feature names', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'Second Wind', level: 1 }),
        makeClassFeature({ name: 'Action Surge', level: 2 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).toContain('Second Wind')
      expect(wrapper.text()).toContain('Action Surge')
    })

    it('shows level and class name in feature meta', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'Second Wind', level: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      const meta = wrapper.find('.feature-meta')
      expect(meta.text()).toContain('Fighter')
      expect(meta.text()).toContain('1')
    })

    it('shows subclass badge for subclass features', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({
          name: 'Improved Critical',
          level: 3,
          subclass_name: 'Champion',
          subclass_source: 'PHB',
        }),
      ]
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.find('.subclass-badge').text()).toBe('Champion')
    })

    it('expands feature on click and loads details', async () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'Second Wind', level: 1 }),
      ]

      mockCommandHandler('get_class_feature', (params) => ({
        success: true,
        data: {
          entries: ['You can use a bonus action to regain hit points.'],
        },
      }))

      const wrapper = mountWithPlugins(CharacterStatsTab, { props })

      // Click to expand
      await wrapper.find('.feature-header').trigger('click')
      await nextTick()
      // Wait for async invoke
      await vi.waitFor(() => {
        expect(wrapper.find('.feature-details').exists()).toBe(true)
      })

      expectCommandCalledWith('get_class_feature', {
        name: 'Second Wind',
        className: 'Fighter',
      })
    })

    it('collapses feature on second click', async () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'Second Wind', level: 1 }),
      ]

      mockCommandHandler('get_class_feature', () => ({
        success: true,
        data: { entries: ['Description'] },
      }))

      const wrapper = mountWithPlugins(CharacterStatsTab, { props })

      // Expand
      await wrapper.find('.feature-header').trigger('click')
      await nextTick()
      await vi.waitFor(() => {
        expect(wrapper.find('.feature-details').exists()).toBe(true)
      })

      // Collapse
      await wrapper.find('.feature-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.feature-details').exists()).toBe(false)
    })

    it('loads subclass feature details with correct params', async () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({
          name: 'Improved Critical',
          level: 3,
          subclass_name: 'Champion',
          subclass_short_name: 'Champion',
          subclass_source: 'PHB',
        }),
      ]

      mockCommandHandler('get_subclass_feature', () => ({
        success: true,
        data: { entries: ['Your weapon attacks score a critical hit on a roll of 19 or 20.'] },
      }))

      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      await wrapper.find('.feature-header').trigger('click')
      await nextTick()

      await vi.waitFor(() => {
        expectCommandCalledWith('get_subclass_feature', {
          name: 'Improved Critical',
          subclassName: 'Champion',
          subclassSource: 'PHB',
        })
      })
    })

    it('hides class features section when none provided', () => {
      const props = defaultProps()
      props.classFeatures = []
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).not.toContain('Class Features')
    })
  })

  // ── Spellcasting ───────────────────────────────────────────────────────

  describe('spellcasting', () => {
    it('hides spellcasting section for non-casters', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      expect(wrapper.text()).not.toContain('Spell Save DC')
    })

    it('shows spellcasting section for spellcasters', () => {
      const props = defaultProps()
      props.characterIsSpellcaster = true
      props.spellcastingAbility = 'intelligence'
      props.spellSaveDC = 14
      props.spellAttackBonus = 6

      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).toContain('Spellcasting')
      expect(wrapper.text()).toContain('14')
      expect(wrapper.text()).toContain('+6')
      expect(wrapper.text()).toContain('INT')
    })

    it('shows spell note directing to Spells tab', () => {
      const props = defaultProps()
      props.characterIsSpellcaster = true
      props.spellcastingAbility = 'wisdom'
      props.spellSaveDC = 13
      props.spellAttackBonus = 5

      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).toContain('See Spells tab for full spell list')
    })
  })

  // ── Personality ────────────────────────────────────────────────────────

  describe('personality', () => {
    it('renders personality traits', () => {
      const wrapper = mountWithPlugins(CharacterStatsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('I am always polite.')
      expect(wrapper.text()).toContain('Honor.')
      expect(wrapper.text()).toContain('I protect the weak.')
      expect(wrapper.text()).toContain('I am too trusting.')
    })

    it('hides personality section when all fields empty', () => {
      const props = defaultProps()
      props.character = makeCharacter({
        traits: null,
        ideals: null,
        bonds: null,
        flaws: null,
      })
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).not.toContain('Personality')
    })

    it('shows personality section when only some fields present', () => {
      const props = defaultProps()
      props.character = makeCharacter({
        traits: 'Brave',
        ideals: null,
        bonds: null,
        flaws: null,
      })
      const wrapper = mountWithPlugins(CharacterStatsTab, { props })
      expect(wrapper.text()).toContain('Brave')
    })
  })
})
