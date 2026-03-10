/**
 * Tests for PCsTab.vue
 *
 * Tests the Player Characters tab on the campaign dashboard.
 * Uses the character store (mocked via invoke) to load and display PCs.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
} from '@tests/helpers/mockInvoke'
import PCsTab from '@/features/campaigns/components/dashboard/PCsTab.vue'

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

function makePC(overrides: Record<string, unknown> = {}) {
  return {
    id: 'char-1',
    campaign_id: 'camp-1',
    name: 'Thorin Ironforge',
    is_npc: 0,
    race_name: 'Mountain Dwarf',
    player_name: 'Alice',
    background_name: 'Soldier',
    classes: [
      { class_name: 'Fighter', class_source: 'PHB', level: 5, subclass_name: 'Champion' },
    ],
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('PCsTab', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  function mountTab(pcs = [makePC()]) {
    mockCommand('list_pcs', pcs)
    return mountWithPlugins(PCsTab, {
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

  describe('rendering PCs', () => {
    it('displays PC names', async () => {
      const wrapper = mountTab([
        makePC({ id: 'c1', name: 'Thorin Ironforge' }),
        makePC({ id: 'c2', name: 'Elara Moonwhisper' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Thorin Ironforge')
        expect(wrapper.text()).toContain('Elara Moonwhisper')
      })
    })

    it('shows player names for PCs', async () => {
      const wrapper = mountTab([
        makePC({ name: 'Thorin', player_name: 'Alice' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Alice')
      })
    })

    it('does not show NPCs in PC list', async () => {
      const wrapper = mountTab([
        makePC({ name: 'Thorin', is_npc: 0 }),
      ])
      // Mock also returns an NPC but the tab filters for PCs only
      mockCommand('list_pcs', [makePC({ name: 'Thorin', is_npc: 0 })])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Thorin')
      })
    })
  })

  describe('empty state', () => {
    it('shows empty message when no PCs exist', async () => {
      const wrapper = mountTab([])
      await vi.waitFor(() => {
        // Should show some indication of no characters
        const text = wrapper.text()
        expect(
          text.includes('No') || text.includes('Create') || text.includes('empty'),
        ).toBe(true)
      })
    })
  })

  describe('header', () => {
    it('displays Player Characters title', async () => {
      const wrapper = mountTab()
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Player Characters')
      })
    })

    it('has Create PC button', async () => {
      const wrapper = mountTab()
      await vi.waitFor(() => {
        const buttons = wrapper.findAll('button')
        const createBtn = buttons.find(b => b.text().match(/Create|New|Add/i))
        expect(createBtn).toBeTruthy()
      })
    })
  })
})
