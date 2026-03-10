/**
 * Tests for ModuleMonsters.vue
 *
 * Tests the module monster management panel including:
 * - Loading and displaying tagged monsters
 * - Monster count badge
 * - Empty state when no monsters tagged
 * - Monster name display (with display_name override)
 * - Quantity and encounter tag inputs
 * - Remove button
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandRaw,
} from '@tests/helpers/mockInvoke'
import ModuleMonsters from '@/features/modules/components/ModuleMonsters.vue'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeModuleMonster(overrides: Record<string, unknown> = {}) {
  return {
    id: 1,
    module_id: 42,
    monster_name: 'Goblin',
    monster_source: 'MM',
    quantity: 3,
    encounter_tag: 'Ambush',
    display_name: null,
    notes: null,
    monster_data: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('ModuleMonsters', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  function mountComponent(monsters = [makeModuleMonster()]) {
    // ModuleMonsters uses invoke directly (not ApiResponse wrapper for list_module_monsters_with_data)
    mockCommandRaw('list_module_monsters_with_data', { data: monsters })
    return mountWithPlugins(ModuleMonsters, {
      props: {
        moduleId: 'mod-1',
        moduleName: 'Test Module',
        moduleNumber: 1,
        campaignId: 'camp-1',
      },
      stubs: {
        EmptyState: false,
        AppModal: true,
      },
    })
  }

  describe('rendering monsters', () => {
    it('displays monster names', async () => {
      const wrapper = mountComponent([
        makeModuleMonster({ id: 1, monster_name: 'Goblin' }),
        makeModuleMonster({ id: 2, monster_name: 'Owlbear' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Goblin')
        expect(wrapper.text()).toContain('Owlbear')
      })
    })

    it('shows display_name override when set', async () => {
      const wrapper = mountComponent([
        makeModuleMonster({ monster_name: 'Goblin', display_name: 'Frost Goblin' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Frost Goblin')
      })
    })

    it('shows monster source', async () => {
      const wrapper = mountComponent([
        makeModuleMonster({ monster_source: 'MM' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('MM')
      })
    })

    it('shows monster count badge', async () => {
      const wrapper = mountComponent([
        makeModuleMonster({ id: 1 }),
        makeModuleMonster({ id: 2 }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('2 tagged')
      })
    })

    it('shows quantity input for each monster', async () => {
      const wrapper = mountComponent([
        makeModuleMonster({ quantity: 3 }),
      ])
      await vi.waitFor(() => {
        const qtyInput = wrapper.find('.quantity-input')
        expect(qtyInput.exists()).toBe(true)
        expect((qtyInput.element as HTMLInputElement).value).toBe('3')
      })
    })

    it('shows encounter tag input', async () => {
      const wrapper = mountComponent([
        makeModuleMonster({ encounter_tag: 'Ambush' }),
      ])
      await vi.waitFor(() => {
        const tagInput = wrapper.find('.tag-input')
        expect(tagInput.exists()).toBe(true)
        expect((tagInput.element as HTMLInputElement).value).toBe('Ambush')
      })
    })

    it('shows remove button for each monster', async () => {
      const wrapper = mountComponent([makeModuleMonster()])
      await vi.waitFor(() => {
        expect(wrapper.find('.remove-button').exists()).toBe(true)
      })
    })
  })

  describe('header', () => {
    it('displays Module Monsters title', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Module Monsters')
      })
    })

    it('shows search input', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        expect(wrapper.find('.search-input').exists()).toBe(true)
      })
    })
  })

  describe('empty state', () => {
    it('shows empty message when no monsters tagged', async () => {
      const wrapper = mountComponent([])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No monsters tagged yet')
      })
    })

    it('does not show count badge when empty', async () => {
      const wrapper = mountComponent([])
      await vi.waitFor(() => {
        expect(wrapper.find('.monster-count').exists()).toBe(false)
      })
    })
  })

  describe('tagged monsters section', () => {
    it('shows Tagged Monsters heading', async () => {
      const wrapper = mountComponent([makeModuleMonster()])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Tagged Monsters')
      })
    })
  })
})
