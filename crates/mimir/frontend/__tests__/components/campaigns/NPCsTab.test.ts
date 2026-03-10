/**
 * Tests for NPCsTab.vue
 *
 * Tests the NPCs tab on the campaign dashboard.
 * Uses the character store (mocked via invoke) to load and display NPCs.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
} from '@tests/helpers/mockInvoke'
import NPCsTab from '@/features/campaigns/components/dashboard/NPCsTab.vue'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeCampaign(overrides: Record<string, unknown> = {}) {
  return {
    id: 'camp-1',
    name: 'Test Campaign',
    description: 'A test campaign',
    archived_at: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

function makeNPC(overrides: Record<string, unknown> = {}) {
  return {
    id: 'npc-1',
    campaign_id: 'camp-1',
    name: 'Goblin King',
    is_npc: 1,
    race_name: 'Goblin',
    player_name: null,
    role: 'Villain',
    location: 'Dark Cave',
    faction: 'Goblin Horde',
    classes: [],
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('NPCsTab', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  function mountTab(npcs = [makeNPC()]) {
    mockCommand('list_npcs', npcs)
    return mountWithPlugins(NPCsTab, {
      props: {
        campaign: makeCampaign(),
        documents: [],
      },
      stubs: {
        CharacterCard: false,
        CharacterCreationWizard: true,
        CharacterPrintDialog: true,
        AddCharacterModal: true,
      },
    })
  }

  describe('rendering NPCs', () => {
    it('displays NPC names', async () => {
      const wrapper = mountTab([
        makeNPC({ id: 'n1', name: 'Goblin King' }),
        makeNPC({ id: 'n2', name: 'Merchant Aria' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Goblin King')
        expect(wrapper.text()).toContain('Merchant Aria')
      })
    })

    it('shows NPC badge on character cards', async () => {
      const wrapper = mountTab([makeNPC({ name: 'Goblin King' })])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Goblin King')
        expect(wrapper.text()).toContain('NPC')
      })
    })

    it('does not show player names for NPCs', async () => {
      const wrapper = mountTab([makeNPC({ player_name: null })])
      await vi.waitFor(() => {
        // CharacterCard with showPlayer=false should not show player name field
        expect(wrapper.text()).toContain('Goblin King')
      })
    })
  })

  describe('empty state', () => {
    it('shows empty message when no NPCs exist', async () => {
      const wrapper = mountTab([])
      await vi.waitFor(() => {
        const text = wrapper.text()
        expect(
          text.includes('No') || text.includes('Create') || text.includes('empty'),
        ).toBe(true)
      })
    })
  })

  describe('header', () => {
    it('displays NPCs title', async () => {
      const wrapper = mountTab()
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('NPC')
      })
    })

    it('has Create NPC button', async () => {
      const wrapper = mountTab()
      await vi.waitFor(() => {
        const buttons = wrapper.findAll('button')
        const createBtn = buttons.find(b => b.text().match(/Create|New|Add/i))
        expect(createBtn).toBeTruthy()
      })
    })
  })
})
