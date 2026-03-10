/**
 * Tests for ModuleNPCs.vue
 *
 * Tests the module NPC panel including:
 * - Loading and displaying NPCs assigned to a module
 * - NPC card rendering with name and role
 * - View and PDF action buttons
 * - Empty state when no NPCs exist
 * - Add NPC button
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
} from '@tests/helpers/mockInvoke'
import ModuleNPCs from '@/features/modules/components/ModuleNPCs.vue'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeModuleNpc(overrides: Record<string, unknown> = {}) {
  return {
    id: 'mnpc-1',
    module_id: 'mod-1',
    character_id: 'char-1',
    role: 'Villain',
    encounter_tag: null,
    notes: null,
    character_name: 'Strahd von Zarovich',
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('ModuleNPCs', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  function mountComponent(npcs = [makeModuleNpc()]) {
    mockCommand('list_module_npcs_with_data', npcs)
    return mountWithPlugins(ModuleNPCs, {
      props: {
        moduleId: 'mod-1',
        campaignId: 'camp-1',
      },
      stubs: {
        EmptyState: false,
        NpcSelectorModal: true,
        CharacterPrintDialog: true,
      },
    })
  }

  describe('rendering NPCs', () => {
    it('displays NPC names', async () => {
      const wrapper = mountComponent([
        makeModuleNpc({ id: 'n1', character_name: 'Strahd von Zarovich' }),
        makeModuleNpc({ id: 'n2', character_name: 'Ireena Kolyana' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Strahd von Zarovich')
        expect(wrapper.text()).toContain('Ireena Kolyana')
      })
    })

    it('shows NPC role badge', async () => {
      const wrapper = mountComponent([
        makeModuleNpc({ role: 'Villain' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Villain')
      })
    })

    it('shows View and PDF action buttons', async () => {
      const wrapper = mountComponent([makeModuleNpc()])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('View')
        expect(wrapper.text()).toContain('PDF')
      })
    })
  })

  describe('header', () => {
    it('displays Module NPCs title', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Module NPCs')
      })
    })

    it('has Add NPC button', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        const buttons = wrapper.findAll('button')
        const addBtn = buttons.find(b => b.text().includes('Add NPC'))
        expect(addBtn).toBeTruthy()
      })
    })
  })

  describe('empty state', () => {
    it('shows empty message when no NPCs exist', async () => {
      const wrapper = mountComponent([])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No NPCs in this module')
      })
    })
  })
})
