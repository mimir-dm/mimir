import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { nextTick } from 'vue'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommandHandler,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import EquipmentSection from '@/features/characters/components/sheet/EquipmentSection.vue'
import type { Character, CharacterInventory } from '@/types/character'

// ─── Test data factories ────────────────────────────────────────────────────

function makeCharacter(overrides: Partial<Character> = {}): Character {
  return {
    id: 'char-1',
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
    cp: 50,
    sp: 25,
    ep: 0,
    gp: 100,
    pp: 5,
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
      { id: 'c1', character_id: 'char-1', class_name: 'Fighter', class_source: 'PHB', level: 5, subclass_name: 'Champion', subclass_source: 'PHB', starting_class: 1 },
    ],
    proficiencies: [],
    ...overrides,
  }
}

function makeInventoryItem(overrides: Partial<CharacterInventory> = {}): CharacterInventory {
  return {
    id: 'inv-1',
    character_id: 'char-1',
    item_name: 'Longsword',
    item_source: 'PHB',
    quantity: 1,
    equipped: 0,
    attuned: 0,
    notes: null,
    ...overrides,
  }
}

const defaultProps = () => ({
  character: makeCharacter(),
  inventory: [] as CharacterInventory[],
  loadingInventory: false,
})

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('EquipmentSection', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  // ── Currency ──────────────────────────────────────────────────────────

  describe('currency', () => {
    it('renders all five currency types', () => {
      const wrapper = mountWithPlugins(EquipmentSection, { props: defaultProps() })
      expect(wrapper.text()).toContain('PP')
      expect(wrapper.text()).toContain('GP')
      expect(wrapper.text()).toContain('EP')
      expect(wrapper.text()).toContain('SP')
      expect(wrapper.text()).toContain('CP')
    })

    it('renders currency values', () => {
      const wrapper = mountWithPlugins(EquipmentSection, { props: defaultProps() })
      const currencyItems = wrapper.findAll('.currency-item')
      expect(currencyItems).toHaveLength(5)

      // PP=5, GP=100, EP=0, SP=25, CP=50
      expect(currencyItems[0].find('.currency-value').text()).toBe('5')
      expect(currencyItems[1].find('.currency-value').text()).toBe('100')
      expect(currencyItems[2].find('.currency-value').text()).toBe('0')
      expect(currencyItems[3].find('.currency-value').text()).toBe('25')
      expect(currencyItems[4].find('.currency-value').text()).toBe('50')
    })

    it('renders PP and GP as large currency items', () => {
      const wrapper = mountWithPlugins(EquipmentSection, { props: defaultProps() })
      const largeItems = wrapper.findAll('.currency-item.large')
      expect(largeItems).toHaveLength(2)
    })
  })

  // ── Equipped Items ────────────────────────────────────────────────────

  describe('equipped items', () => {
    it('shows empty state when no items equipped', () => {
      const wrapper = mountWithPlugins(EquipmentSection, { props: defaultProps() })
      expect(wrapper.text()).toContain('No items equipped')
    })

    it('renders equipped items', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', equipped: 1 }),
        makeInventoryItem({ id: 'i2', item_name: 'Chain Mail', equipped: 1 }),
        makeInventoryItem({ id: 'i3', item_name: 'Potion of Healing', equipped: 0 }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })

      // Equipped section should show only equipped items
      const sections = wrapper.findAll('.sheet-section')
      const equippedSection = sections[1] // Currency, Equipped, Inventory
      const equippedCards = equippedSection.findAll('.item-card')
      expect(equippedCards).toHaveLength(2)
    })

    it('shows attuned badge', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Flame Tongue', equipped: 1, attuned: 1 }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      expect(wrapper.find('.item-attuned').exists()).toBe(true)
      expect(wrapper.find('.item-attuned').text()).toBe('Attuned')
    })

    it('shows homebrew badge for HB source', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Custom Sword', item_source: 'HB', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      expect(wrapper.find('.homebrew-badge').exists()).toBe(true)
      expect(wrapper.find('.homebrew-badge').text()).toBe('HB')
    })

    it('shows source book for non-HB items', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', item_source: 'PHB', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      expect(wrapper.find('.item-source').text()).toBe('PHB')
    })

    it('expands item on click and loads details via invoke', async () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', item_source: 'PHB', equipped: 1 }),
      ]

      mockCommandHandler('get_item_by_name', () => ({
        success: true,
        data: {
          name: 'Longsword',
          source: 'PHB',
          type: 'M',
          rarity: 'none',
          weapon: true,
          dmg1: '1d8',
          dmgType: 'S',
          weight: 3,
          property: ['V'],
        },
      }))

      const wrapper = mountWithPlugins(EquipmentSection, { props })

      await wrapper.find('.item-card-header').trigger('click')
      await nextTick()

      await vi.waitFor(() => {
        expect(wrapper.find('.item-card-details').exists()).toBe(true)
      })

      expectCommandCalledWith('get_item_by_name', {
        name: 'Longsword',
        source: 'PHB',
        campaignId: 'campaign-1',
      })
    })

    it('collapses item on second click', async () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', equipped: 1 }),
      ]

      mockCommandHandler('get_item_by_name', () => ({
        success: true,
        data: { name: 'Longsword', source: 'PHB', type: 'M' },
      }))

      const wrapper = mountWithPlugins(EquipmentSection, { props })

      // Expand
      await wrapper.find('.item-card-header').trigger('click')
      await nextTick()
      await vi.waitFor(() => {
        expect(wrapper.find('.item-card-details').exists()).toBe(true)
      })

      // Collapse
      await wrapper.find('.item-card-header').trigger('click')
      await nextTick()
      expect(wrapper.find('.item-card-details').exists()).toBe(false)
    })

    it('does not expand item until details have loaded', async () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', equipped: 1 }),
      ]

      // Use a handler that returns a pending promise
      let resolvePromise!: (value: unknown) => void
      mockCommandHandler('get_item_by_name', () => {
        return new Promise((resolve) => {
          resolvePromise = resolve
        })
      })

      const wrapper = mountWithPlugins(EquipmentSection, { props })
      await wrapper.find('.item-card-header').trigger('click')
      await nextTick()

      // Item should NOT be expanded while invoke is pending
      expect(wrapper.find('.item-card-details').exists()).toBe(false)

      // Resolve the promise
      resolvePromise({ success: true, data: { name: 'Longsword', source: 'PHB' } })
      await vi.waitFor(() => {
        expect(wrapper.find('.item-card-details').exists()).toBe(true)
      })
    })
  })

  // ── Full Inventory ────────────────────────────────────────────────────

  describe('full inventory', () => {
    it('shows empty state when inventory is empty', () => {
      const wrapper = mountWithPlugins(EquipmentSection, { props: defaultProps() })
      expect(wrapper.text()).toContain('No items in inventory')
    })

    it('shows loading state while inventory loads', () => {
      const props = defaultProps()
      props.loadingInventory = true
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      expect(wrapper.text()).toContain('Loading inventory...')
    })

    it('renders all inventory items', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword' }),
        makeInventoryItem({ id: 'i2', item_name: 'Shield' }),
        makeInventoryItem({ id: 'i3', item_name: 'Potion of Healing' }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      expect(wrapper.text()).toContain('Longsword')
      expect(wrapper.text()).toContain('Shield')
      expect(wrapper.text()).toContain('Potion of Healing')
    })

    it('shows quantity badge for stacked items', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Potion of Healing', quantity: 3 }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      expect(wrapper.find('.item-qty').exists()).toBe(true)
      expect(wrapper.find('.item-qty').text()).toBe('x3')
    })

    it('hides quantity badge for single items', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', quantity: 1 }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      expect(wrapper.find('.item-qty').exists()).toBe(false)
    })

    it('shows equipped badge in inventory section', () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', equipped: 1 }),
      ]
      const wrapper = mountWithPlugins(EquipmentSection, { props })
      // The inventory section (3rd section) shows an "Equipped" badge
      const inventorySection = wrapper.findAll('.sheet-section')[2]
      expect(inventorySection.find('.item-equipped-badge').exists()).toBe(true)
    })

    it('shows item notes when expanded', async () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', notes: 'Family heirloom' }),
      ]

      mockCommandHandler('get_item_by_name', () => ({
        success: true,
        data: { name: 'Longsword', source: 'PHB', type: 'M' },
      }))

      const wrapper = mountWithPlugins(EquipmentSection, { props })

      // Expand the item in the inventory section
      const inventorySection = wrapper.findAll('.sheet-section')[2]
      await inventorySection.find('.item-card-header').trigger('click')
      await nextTick()

      await vi.waitFor(() => {
        expect(wrapper.find('.item-notes').exists()).toBe(true)
      })
      expect(wrapper.text()).toContain('Family heirloom')
    })

    it('emits openInventory when Manage button is clicked', async () => {
      const wrapper = mountWithPlugins(EquipmentSection, { props: defaultProps() })

      const manageBtn = wrapper.find('.btn-secondary')
      expect(manageBtn.text()).toBe('Manage')

      await manageBtn.trigger('click')
      expect(wrapper.emitted('openInventory')).toBeTruthy()
    })
  })

  // ── Item Detail Caching ───────────────────────────────────────────────

  describe('item detail caching', () => {
    it('does not re-fetch details for already loaded items', async () => {
      const props = defaultProps()
      props.inventory = [
        makeInventoryItem({ id: 'i1', item_name: 'Longsword', equipped: 1 }),
      ]

      let callCount = 0
      mockCommandHandler('get_item_by_name', () => {
        callCount++
        return { success: true, data: { name: 'Longsword', source: 'PHB' } }
      })

      const wrapper = mountWithPlugins(EquipmentSection, { props })

      // Expand
      await wrapper.find('.item-card-header').trigger('click')
      await nextTick()
      await vi.waitFor(() => {
        expect(wrapper.find('.item-card-details').exists()).toBe(true)
      })

      // Collapse
      await wrapper.find('.item-card-header').trigger('click')
      await nextTick()

      // Expand again — should not invoke again
      await wrapper.find('.item-card-header').trigger('click')
      await nextTick()

      expect(callCount).toBe(1)
    })
  })
})
