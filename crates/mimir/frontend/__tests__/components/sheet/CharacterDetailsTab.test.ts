import { describe, it, expect } from 'vitest'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import CharacterDetailsTab from '@/features/characters/components/sheet/CharacterDetailsTab.vue'
import type { Character } from '@/types/character'

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
    traits: null,
    ideals: null,
    bonds: null,
    flaws: null,
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
    proficiencies: [],
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

interface BackgroundDetail {
  name: string
  source: string
  data: Record<string, unknown>
  fluff: string | null
}

interface SubclassDetail {
  name: string
  source: string
  class_name: string
  class_source: string
  data: Record<string, unknown>
}

function makeBackgroundDetail(overrides: Partial<BackgroundDetail> = {}): BackgroundDetail {
  return {
    name: 'Soldier',
    source: 'PHB',
    data: {
      skillProficiencies: [{ athletics: true, intimidation: true }],
      toolProficiencies: [{ 'gaming set': true, "vehicles (land)": true }],
      languageProficiencies: [],
      entries: [
        {
          type: 'list',
          name: 'Equipment',
          items: ['An insignia of rank', 'A trophy from a fallen enemy', 'A set of common clothes', '10 gp'],
        },
        {
          type: 'entries',
          name: 'Military Rank',
          entries: [
            'You have a military rank from your career as a soldier.',
            'Soldiers still recognize your authority.',
          ],
        },
      ],
    },
    fluff: null,
    ...overrides,
  }
}

function makeSubclassDetail(overrides: Partial<SubclassDetail> = {}): SubclassDetail {
  return {
    name: 'Champion',
    source: 'PHB',
    class_name: 'Fighter',
    class_source: 'PHB',
    data: {
      entries: [
        {
          type: 'entries',
          entries: ['The archetypal Champion focuses on the development of raw physical power.'],
        },
      ],
    },
    ...overrides,
  }
}

function makeClassData(): Record<string, Record<string, unknown>> {
  return {
    fighter: {
      name: 'Fighter',
      source: 'PHB',
      hd: { faces: 10 },
      proficiency: [{ str: true }, { con: true }],
      startingProficiencies: {
        armor: ['light armor', 'medium armor', 'heavy armor', 'shields'],
        weapons: ['simple weapons', 'martial weapons'],
        skills: [{ choose: { from: ['Acrobatics', 'Animal Handling', 'Athletics', 'History', 'Insight', 'Intimidation', 'Perception', 'Survival'], count: 2 } }],
      },
      primaryAbility: [{ str: true }],
    },
  }
}

const defaultProps = () => ({
  character: makeCharacter(),
  classFeatures: [] as ClassFeature[],
  classData: makeClassData(),
  backgroundDetails: makeBackgroundDetail() as BackgroundDetail | null,
  subclassDetails: {
    'Fighter|Champion': makeSubclassDetail(),
  } as Record<string, SubclassDetail>,
})

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('CharacterDetailsTab', () => {
  // ── Background Section ─────────────────────────────────────────────────

  describe('background section', () => {
    it('renders background name', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Background: Soldier')
    })

    it('renders background skill proficiencies', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('athletics')
      expect(wrapper.text()).toContain('intimidation')
    })

    it('renders background tool proficiencies', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('gaming set')
      expect(wrapper.text()).toContain('vehicles (land)')
    })

    it('renders background equipment', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('An insignia of rank')
      expect(wrapper.text()).toContain('10 gp')
    })

    it('renders background feature', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Military Rank')
      expect(wrapper.text()).toContain('military rank from your career')
    })

    it('shows loading text when backgroundDetails is null', () => {
      const props = defaultProps()
      props.backgroundDetails = null
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).toContain('Loading background details...')
    })

    it('hides background section when character has no background', () => {
      const props = defaultProps()
      props.character = makeCharacter({ background_name: null })
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).not.toContain('Background:')
    })

    it('renders background languages when present', () => {
      const props = defaultProps()
      props.backgroundDetails = makeBackgroundDetail({
        data: {
          ...makeBackgroundDetail().data,
          languageProficiencies: [{ anyStandard: 2 }],
        },
      })
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).toContain('2 of your choice')
    })
  })

  // ── NPC Section ────────────────────────────────────────────────────────

  describe('NPC section', () => {
    it('shows NPC details for NPCs', () => {
      const props = defaultProps()
      props.character = makeCharacter({
        is_npc: 1,
        role: 'Shopkeeper',
        location: 'Market Square',
        faction: 'Merchants Guild',
      })
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).toContain('NPC Details')
      expect(wrapper.text()).toContain('Shopkeeper')
      expect(wrapper.text()).toContain('Market Square')
      expect(wrapper.text()).toContain('Merchants Guild')
    })

    it('hides NPC section for PCs', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).not.toContain('NPC Details')
    })

    it('hides NPC section when NPC has no role/location/faction', () => {
      const props = defaultProps()
      props.character = makeCharacter({ is_npc: 1 })
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).not.toContain('NPC Details')
    })
  })

  // ── Classes Section ───────────────────────────────────────────────────

  describe('classes section', () => {
    it('renders class name and level', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Fighter')
      expect(wrapper.text()).toContain('Level 5')
    })

    it('renders hit die from class data', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('d10')
    })

    it('renders primary ability from class data', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Strength')
    })

    it('renders saving throws from class data', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Strength')
      expect(wrapper.text()).toContain('Constitution')
    })

    it('renders starting proficiencies', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('light armor')
      expect(wrapper.text()).toContain('simple weapons')
    })

    it('renders subclass name', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('Subclass: Champion')
    })

    it('renders subclass description', () => {
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props: defaultProps() })
      expect(wrapper.text()).toContain('archetypal Champion')
    })

    it('shows empty state when no classes', () => {
      const props = defaultProps()
      props.character = makeCharacter({ classes: [] })
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).toContain('No classes')
    })

    it('renders spellcasting ability for caster classes', () => {
      const props = defaultProps()
      props.character = makeCharacter({
        classes: [
          { id: 'c1', character_id: 'test-1', class_name: 'Wizard', class_source: 'PHB', level: 5, subclass_name: 'School of Evocation', subclass_source: 'PHB', starting_class: 1 },
        ],
      })
      props.classData = {
        wizard: {
          name: 'Wizard',
          source: 'PHB',
          hd: { faces: 6 },
          proficiency: [{ int: true }, { wis: true }],
          startingProficiencies: {
            weapons: ['daggers', 'darts', 'slings', 'quarterstaffs', 'light crossbows'],
            skills: [{ choose: { from: ['Arcana', 'History', 'Insight', 'Investigation', 'Medicine', 'Religion'], count: 2 } }],
          },
          spellcastingAbility: 'int',
          primaryAbility: [{ int: true }],
        },
      }
      props.subclassDetails = {
        'Wizard|School of Evocation': makeSubclassDetail({ name: 'School of Evocation', class_name: 'Wizard' }),
      }
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).toContain('INT')
    })

    it('renders multiclass characters', () => {
      const props = defaultProps()
      props.character = makeCharacter({
        classes: [
          { id: 'c1', character_id: 'test-1', class_name: 'Fighter', class_source: 'PHB', level: 3, subclass_name: 'Champion', subclass_source: 'PHB', starting_class: 1 },
          { id: 'c2', character_id: 'test-1', class_name: 'Rogue', class_source: 'PHB', level: 2, subclass_name: null, subclass_source: null, starting_class: 0 },
        ],
      })
      props.classData = {
        ...makeClassData(),
        rogue: {
          name: 'Rogue',
          source: 'PHB',
          hd: { faces: 8 },
          proficiency: [{ dex: true }, { int: true }],
          startingProficiencies: {
            armor: ['light armor'],
            weapons: ['simple weapons', 'hand crossbows', 'longswords', 'rapiers', 'shortswords'],
            skills: [{ choose: { from: ['Acrobatics', 'Athletics', 'Deception', 'Insight', 'Intimidation', 'Investigation', 'Perception', 'Performance', 'Persuasion', 'Sleight of Hand', 'Stealth'], count: 4 } }],
          },
          primaryAbility: [{ dex: true }],
        },
      }
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      const classCards = wrapper.findAll('.class-detail-card')
      expect(classCards).toHaveLength(2)
    })

    it('renders "—" when class data is not available', () => {
      const props = defaultProps()
      props.classData = {} // No class data
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).toContain('—')
    })
  })

  // ── Features by Level ─────────────────────────────────────────────────

  describe('features by level', () => {
    it('renders features grouped by level', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'Second Wind', level: 1 }),
        makeClassFeature({ name: 'Fighting Style', level: 1 }),
        makeClassFeature({ name: 'Action Surge', level: 2 }),
      ]
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })

      const levelGroups = wrapper.findAll('.feature-level-group')
      expect(levelGroups.length).toBeGreaterThanOrEqual(2)

      // Check ordinal formatting
      expect(wrapper.text()).toContain('1st Level')
      expect(wrapper.text()).toContain('2nd Level')
    })

    it('renders feature names as links', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'Second Wind', level: 1 }),
      ]
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      const link = wrapper.find('.feature-ref')
      expect(link.exists()).toBe(true)
      expect(link.text()).toBe('Second Wind')
    })

    it('emits open-feature-modal on feature click', async () => {
      const feature = makeClassFeature({ name: 'Second Wind', level: 1 })
      const props = defaultProps()
      props.classFeatures = [feature]
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })

      await wrapper.find('.feature-ref').trigger('click')
      const emitted = wrapper.emitted('open-feature-modal')
      expect(emitted).toBeTruthy()
      expect(emitted![0][0]).toMatchObject({ name: 'Second Wind' })
    })

    it('styles subclass features differently', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({
          name: 'Improved Critical',
          level: 3,
          subclass_name: 'Champion',
        }),
      ]
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      const link = wrapper.find('.subclass-feature')
      expect(link.exists()).toBe(true)
    })

    it('includes subclass features in level grouping', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'Second Wind', level: 1 }),
        makeClassFeature({ name: 'Improved Critical', level: 3, subclass_name: 'Champion' }),
        makeClassFeature({ name: 'Extra Attack', level: 5 }),
      ]
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      const levelGroups = wrapper.findAll('.feature-level-group')
      expect(levelGroups.length).toBe(3) // Level 1, 3, 5
    })
  })

  // ── Ordinal Formatting ────────────────────────────────────────────────

  describe('ordinal formatting', () => {
    it('formats level numbers as ordinals', () => {
      const props = defaultProps()
      props.classFeatures = [
        makeClassFeature({ name: 'F1', level: 1 }),
        makeClassFeature({ name: 'F2', level: 2 }),
        makeClassFeature({ name: 'F3', level: 3 }),
        makeClassFeature({ name: 'F4', level: 4 }),
        makeClassFeature({ name: 'F11', level: 11 }),
        makeClassFeature({ name: 'F12', level: 12 }),
        makeClassFeature({ name: 'F13', level: 13 }),
        makeClassFeature({ name: 'F20', level: 20 }),
      ]
      const wrapper = mountWithPlugins(CharacterDetailsTab, { props })
      expect(wrapper.text()).toContain('1st Level')
      expect(wrapper.text()).toContain('2nd Level')
      expect(wrapper.text()).toContain('3rd Level')
      expect(wrapper.text()).toContain('4th Level')
      expect(wrapper.text()).toContain('11th Level')
      expect(wrapper.text()).toContain('12th Level')
      expect(wrapper.text()).toContain('13th Level')
      expect(wrapper.text()).toContain('20th Level')
    })
  })
})
