import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { nextTick } from 'vue'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandHandler,
} from '@tests/helpers/mockInvoke'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import SpellsSection from '@/features/characters/components/sheet/SpellsSection.vue'
import type { Character } from '@/types/character'

// ─── Test data factories ────────────────────────────────────────────────────

function makeWizard(overrides: Partial<Character> = {}): Character {
  return {
    id: 'wizard-1',
    campaign_id: 'campaign-1',
    name: 'Test Wizard',
    is_npc: 0,
    player_name: 'Player',
    race_name: 'High Elf',
    race_source: 'PHB',
    background_name: 'Sage',
    background_source: 'PHB',
    strength: 8,
    dexterity: 14,
    constitution: 12,
    intelligence: 17,
    wisdom: 13,
    charisma: 10,
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
      { id: 'c1', character_id: 'wizard-1', class_name: 'Wizard', class_source: 'PHB', level: 5, subclass_name: 'School of Evocation', subclass_source: 'PHB', starting_class: 1 },
    ],
    proficiencies: [],
    ...overrides,
  }
}

function makeSpellResponse(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    name: 'Fireball',
    source: 'PHB',
    level: 3,
    school: 'V',
    ritual: 0,
    concentration: 0,
    time: [{ number: 1, unit: 'action' }],
    range: { type: 'point', distance: { type: 'feet', amount: 150 } },
    components: { v: true, s: true, m: 'A tiny ball of bat guano and sulfur' },
    duration: [{ type: 'instant' }],
    entries: ['A bright streak flashes from your pointing finger to a point you choose.'],
    ...overrides,
  }
}

const formatMod = (mod: number) => (mod >= 0 ? `+${mod}` : `${mod}`)

function setupSpellMocks(spells: Record<string, unknown>[] = []) {
  // Mock list_character_sources (no source filtering)
  mockCommand('list_character_sources', [])

  // Mock get_spells_by_class
  mockCommandHandler('get_spells_by_class', () => ({
    success: true,
    data: spells,
  }))
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('SpellsSection', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  // ── Spellcasting Stats ────────────────────────────────────────────────

  describe('spellcasting stats', () => {
    it('renders spell save DC', async () => {
      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        // Wizard level 5, INT 17 (+3), prof +3: DC = 8 + 3 + 3 = 14
        expect(wrapper.text()).toContain('14')
      })
    })

    it('renders spell attack bonus', async () => {
      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        // Prof +3 + INT mod +3 = +6
        expect(wrapper.text()).toContain('+6')
      })
    })

    it('renders spellcasting ability abbreviation', async () => {
      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('INT')
      })
    })

    it('shows multiclass stats for multiple caster classes', async () => {
      const character = makeWizard({
        classes: [
          { id: 'c1', character_id: 'wizard-1', class_name: 'Wizard', class_source: 'PHB', level: 3, subclass_name: null, subclass_source: null, starting_class: 1 },
          { id: 'c2', character_id: 'wizard-1', class_name: 'Cleric', class_source: 'PHB', level: 2, subclass_name: null, subclass_source: null, starting_class: 0 },
        ],
      })

      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character, formatMod },
      })
      await vi.waitFor(() => {
        // Should show separate stats per class
        const multiclassRows = wrapper.findAll('.spell-stats-row.multiclass')
        expect(multiclassRows.length).toBe(2)
      })
    })
  })

  // ── Spell Slots ───────────────────────────────────────────────────────

  describe('spell slots', () => {
    it('shows cantrips as Unlimited', async () => {
      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await nextTick()
      expect(wrapper.text()).toContain('Unlimited')
    })

    it('renders spell slot boxes for each level', async () => {
      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await nextTick()

      // Wizard level 5 (caster level 5): 4/3/2 slots at levels 1/2/3
      const slotRows = wrapper.findAll('.spell-slot-row')
      expect(slotRows.length).toBeGreaterThanOrEqual(4) // cantrip + 3 levels

      // Check slot counts
      expect(wrapper.text()).toContain('4 slots')
      expect(wrapper.text()).toContain('3 slots')
      expect(wrapper.text()).toContain('2 slots')
    })

    it('shows tracking note', async () => {
      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await nextTick()
      expect(wrapper.text()).toContain('Track used slots on paper')
    })
  })

  // ── Available Spells ─────────────────────────────────────────────────

  describe('available spells', () => {
    it('shows loading state while fetching', () => {
      // Don't setup mocks so it stays loading
      mockCommandHandler('list_character_sources', () => new Promise(() => {}))

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      expect(wrapper.text()).toContain('Loading spells...')
    })

    it('shows empty state when no spells available', async () => {
      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No spells available')
      })
    })

    it('renders spell list grouped by level', async () => {
      setupSpellMocks([
        makeSpellResponse({ name: 'Fire Bolt', level: 0, school: 'V' }),
        makeSpellResponse({ name: 'Magic Missile', level: 1, school: 'V' }),
        makeSpellResponse({ name: 'Fireball', level: 3, school: 'V' }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.findAll('.spell-level-group').length).toBeGreaterThanOrEqual(2)
      })
    })

    it('shows spell count in level header', async () => {
      setupSpellMocks([
        makeSpellResponse({ name: 'Fire Bolt', level: 0 }),
        makeSpellResponse({ name: 'Ray of Frost', level: 0 }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-count').text()).toBe('(2)')
      })
    })

    it('toggles spell level collapse on header click', async () => {
      setupSpellMocks([
        makeSpellResponse({ name: 'Fire Bolt', level: 0 }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-level-header').exists()).toBe(true)
      })

      // All levels start collapsed — click to expand
      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.spell-level-header').classes()).not.toContain('collapsed')

      // Click again to collapse
      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.spell-level-header').classes()).toContain('collapsed')
    })

    it('shows ritual badge for ritual spells', async () => {
      setupSpellMocks([
        makeSpellResponse({ name: 'Detect Magic', level: 1, ritual: 1 }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-level-header').exists()).toBe(true)
      })

      // Expand level to see spells
      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.spell-tag.ritual').exists()).toBe(true)
      expect(wrapper.find('.spell-tag.ritual').text()).toBe('R')
    })

    it('shows concentration badge for concentration spells', async () => {
      setupSpellMocks([
        makeSpellResponse({ name: 'Haste', level: 3, concentration: 1, duration: [{ type: 'timed', duration: { type: 'minute', amount: 1 }, concentration: true }] }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-level-header').exists()).toBe(true)
      })

      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.spell-tag.conc').exists()).toBe(true)
      expect(wrapper.find('.spell-tag.conc').text()).toBe('C')
    })

    it('shows school name for spells', async () => {
      setupSpellMocks([
        makeSpellResponse({ name: 'Fireball', level: 3, school: 'V' }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-level-header').exists()).toBe(true)
      })

      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.spell-school').text()).toBe('Evocation')
    })

    it('expands spell to show details on click', async () => {
      setupSpellMocks([
        makeSpellResponse({
          name: 'Fireball',
          level: 3,
          time: [{ number: 1, unit: 'action' }],
          range: { type: 'point', distance: { type: 'feet', amount: 150 } },
          components: { v: true, s: true, m: true },
          duration: [{ type: 'instant' }],
          entries: ['A bright streak flashes.'],
        }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-level-header').exists()).toBe(true)
      })

      // Expand level
      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()

      // Expand spell
      await wrapper.find('.spell-card-header').trigger('click')
      await nextTick()

      expect(wrapper.find('.spell-card-details').exists()).toBe(true)
      expect(wrapper.text()).toContain('1 action')
      expect(wrapper.text()).toContain('150 feet')
      expect(wrapper.text()).toContain('V, S, M')
      expect(wrapper.text()).toContain('Instantaneous')
      expect(wrapper.text()).toContain('A bright streak flashes.')
    })

    it('collapses spell on second click', async () => {
      setupSpellMocks([
        makeSpellResponse({ name: 'Fireball', level: 3 }),
      ])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-level-header').exists()).toBe(true)
      })

      // Expand level
      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()

      // Expand spell
      await wrapper.find('.spell-card-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.spell-card-details').exists()).toBe(true)

      // Collapse spell
      await wrapper.find('.spell-card-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.spell-card-details').exists()).toBe(false)
    })
  })

  // ── Spell Detail Formatting ──────────────────────────────────────────

  describe('spell detail formatting', () => {
    async function mountWithSpell(spellOverrides: Record<string, unknown> = {}) {
      setupSpellMocks([makeSpellResponse({ name: 'TestSpell', level: 1, ...spellOverrides })])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: makeWizard(), formatMod },
      })
      await vi.waitFor(() => {
        expect(wrapper.find('.spell-level-header').exists()).toBe(true)
      })

      // Expand level and spell
      await wrapper.find('.spell-level-header').trigger('click')
      await nextTick()
      await wrapper.find('.spell-card-header').trigger('click')
      await nextTick()

      return wrapper
    }

    it('formats Self range', async () => {
      const wrapper = await mountWithSpell({
        range: { type: 'point', distance: { type: 'self' } },
      })
      expect(wrapper.text()).toContain('Self')
    })

    it('formats Touch range', async () => {
      const wrapper = await mountWithSpell({
        range: { type: 'point', distance: { type: 'touch' } },
      })
      expect(wrapper.text()).toContain('Touch')
    })

    it('formats concentration duration', async () => {
      const wrapper = await mountWithSpell({
        duration: [{ type: 'timed', duration: { type: 'minute', amount: 10 }, concentration: true }],
      })
      expect(wrapper.text()).toContain('Concentration, 10 minute')
    })

    it('formats permanent duration', async () => {
      const wrapper = await mountWithSpell({
        duration: [{ type: 'permanent' }],
      })
      expect(wrapper.text()).toContain('Permanent')
    })

    it('formats components V only', async () => {
      const wrapper = await mountWithSpell({
        components: { v: true },
      })
      // Should contain V but not S or M in components section
      const details = wrapper.find('.spell-card-details')
      const componentLine = details.findAll('.spell-stat-mini').find(
        (el) => el.text().includes('Components:')
      )
      expect(componentLine?.text()).toContain('V')
    })
  })

  // ── Non-spellcaster ───────────────────────────────────────────────────

  describe('non-spellcaster', () => {
    it('does not load spells for non-spellcaster classes', async () => {
      const fighter: Character = {
        ...makeWizard(),
        classes: [
          { id: 'c1', character_id: 'f1', class_name: 'Fighter', class_source: 'PHB', level: 5, subclass_name: 'Champion', subclass_source: 'PHB', starting_class: 1 },
        ],
      }

      setupSpellMocks([])

      const wrapper = mountWithPlugins(SpellsSection, {
        props: { character: fighter, formatMod },
      })
      await nextTick()

      // Should not show spells since Fighter isn't a caster
      expect(wrapper.text()).toContain('No spells available')
    })
  })
})
