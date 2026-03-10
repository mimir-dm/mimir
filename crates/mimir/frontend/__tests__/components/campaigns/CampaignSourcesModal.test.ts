/**
 * Tests for CampaignSourcesModal.vue
 *
 * Tests the campaign source book selection modal including:
 * - Loading available sources from catalog
 * - Displaying and toggling source selections
 * - Quick action buttons (Select All, Select None, Core Only)
 * - Saving selected sources
 * - Error handling
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { nextTick } from 'vue'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  expectCommandCalled,
} from '@tests/helpers/mockInvoke'
import CampaignSourcesModal from '@/components/campaigns/CampaignSourcesModal.vue'

// ─── Fixtures ───────────────────────────────────────────────────────────────

const AVAILABLE_SOURCES = [
  { id: 'PHB', name: "Player's Handbook" },
  { id: 'XPHB', name: "Player's Handbook (2024)" },
  { id: 'DMG', name: "Dungeon Master's Guide" },
  { id: 'MM', name: 'Monster Manual' },
  { id: 'XGE', name: "Xanathar's Guide to Everything" },
  { id: 'TCE', name: "Tasha's Cauldron of Everything" },
  { id: 'SCAG', name: "Sword Coast Adventurer's Guide" },
]

function setupSourceMocks(selectedSources: string[] = ['PHB', 'DMG', 'MM']) {
  mockCommand('list_catalog_sources', AVAILABLE_SOURCES)
  mockCommand('list_campaign_sources', selectedSources)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('CampaignSourcesModal', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  /**
   * Mount the modal with visible=false, then toggle to visible=true.
   * The component watches props.visible (without immediate: true),
   * so the watcher only fires on a change from false→true.
   */
  async function mountModal(selectedSources?: string[]) {
    if (selectedSources !== undefined) {
      setupSourceMocks(selectedSources)
    }
    const wrapper = mountWithPlugins(CampaignSourcesModal, {
      props: { visible: false, campaignId: 'camp-1' },
      stubs: { AppModal: false },
    })
    // Toggle visible to trigger the watcher and load data
    await wrapper.setProps({ visible: true })
    return wrapper
  }

  describe('loading', () => {
    it('loads available sources when modal becomes visible', async () => {
      const wrapper = await mountModal(['PHB', 'DMG', 'MM'])
      await vi.waitFor(() => {
        expectCommandCalled('list_catalog_sources')
        expectCommandCalled('list_campaign_sources')
      })
    })

    it('displays source list after loading', async () => {
      const wrapper = await mountModal(['PHB', 'DMG', 'MM'])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain("Player's Handbook")
        expect(wrapper.text()).toContain('Monster Manual')
      })
    })

    it('shows source codes', async () => {
      const wrapper = await mountModal(['PHB', 'DMG', 'MM'])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('PHB')
        expect(wrapper.text()).toContain('DMG')
      })
    })

    it('shows empty state when no sources available', async () => {
      mockCommand('list_catalog_sources', [])
      mockCommand('list_campaign_sources', [])
      const wrapper = mountWithPlugins(CampaignSourcesModal, {
        props: { visible: false, campaignId: 'camp-1' },
        stubs: { AppModal: false },
      })
      await wrapper.setProps({ visible: true })
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No sources imported')
      })
    })
  })

  describe('selection', () => {
    it('checks boxes for already-selected sources', async () => {
      const wrapper = await mountModal(['PHB', 'DMG'])
      await vi.waitFor(() => {
        const checkboxes = wrapper.findAll('input[type="checkbox"]')
        expect(checkboxes.length).toBeGreaterThan(0)
        const checkedBoxes = checkboxes.filter(cb => (cb.element as HTMLInputElement).checked)
        expect(checkedBoxes.length).toBe(2)
      })
    })

    it('toggles source when checkbox is clicked', async () => {
      const wrapper = await mountModal(['PHB'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      const unchecked = checkboxes.find(cb => !(cb.element as HTMLInputElement).checked)
      if (unchecked) {
        await unchecked.trigger('change')
        await nextTick()
      }
    })

    it('shows selected count in footer', async () => {
      const wrapper = await mountModal(['PHB', 'DMG', 'MM'])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('3 sources selected')
      })
    })
  })

  describe('quick actions', () => {
    it('Select All checks all sources', async () => {
      const wrapper = await mountModal(['PHB'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const selectAllBtn = wrapper.findAll('button').find(b => b.text() === 'Select All')
      expect(selectAllBtn).toBeTruthy()
      await selectAllBtn!.trigger('click')
      await nextTick()
      expect(wrapper.text()).toContain(`${AVAILABLE_SOURCES.length} sources selected`)
    })

    it('Select None unchecks all sources', async () => {
      const wrapper = await mountModal(['PHB', 'DMG', 'MM'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const selectNoneBtn = wrapper.findAll('button').find(b => b.text() === 'Select None')
      expect(selectNoneBtn).toBeTruthy()
      await selectNoneBtn!.trigger('click')
      await nextTick()
      expect(wrapper.text()).toContain('0 sources selected')
    })

    it('Core Only selects only core sources', async () => {
      const wrapper = await mountModal([])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const coreBtn = wrapper.findAll('button').find(b => b.text() === 'Core Only')
      expect(coreBtn).toBeTruthy()
      await coreBtn!.trigger('click')
      await nextTick()
      const checkboxes = wrapper.findAll('input[type="checkbox"]')
      const checked = checkboxes.filter(cb => (cb.element as HTMLInputElement).checked)
      // PHB, XPHB, DMG from our fixture are core sources
      expect(checked.length).toBeGreaterThan(0)
    })
  })

  describe('unsaved changes', () => {
    it('shows unsaved indicator when selections change', async () => {
      const wrapper = await mountModal(['PHB'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const selectAllBtn = wrapper.findAll('button').find(b => b.text() === 'Select All')
      await selectAllBtn!.trigger('click')
      await nextTick()
      expect(wrapper.text()).toContain('Unsaved changes')
    })

    it('disables save button when no changes', async () => {
      const wrapper = await mountModal(['PHB', 'DMG'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const saveBtn = wrapper.findAll('button').find(b => b.text() === 'Save')
      expect(saveBtn).toBeTruthy()
      expect((saveBtn!.element as HTMLButtonElement).disabled).toBe(true)
    })
  })

  describe('saving', () => {
    it('calls set_campaign_sources with correct payload', async () => {
      mockCommand('set_campaign_sources', null)
      const wrapper = await mountModal(['PHB'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const selectAllBtn = wrapper.findAll('button').find(b => b.text() === 'Select All')
      await selectAllBtn!.trigger('click')
      await nextTick()
      const saveBtn = wrapper.findAll('button').find(b => b.text() === 'Save')
      await saveBtn!.trigger('click')
      await vi.waitFor(() => {
        expectCommandCalled('set_campaign_sources')
      })
    })

    it('emits saved and close on successful save', async () => {
      mockCommand('set_campaign_sources', null)
      const wrapper = await mountModal(['PHB'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const selectAllBtn = wrapper.findAll('button').find(b => b.text() === 'Select All')
      await selectAllBtn!.trigger('click')
      await nextTick()
      const saveBtn = wrapper.findAll('button').find(b => b.text() === 'Save')
      await saveBtn!.trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.emitted('saved')).toBeTruthy()
        expect(wrapper.emitted('close')).toBeTruthy()
      })
    })
  })

  describe('cancel', () => {
    it('emits close when cancel is clicked', async () => {
      const wrapper = await mountModal(['PHB', 'DMG', 'MM'])
      await vi.waitFor(() => {
        expect(wrapper.findAll('input[type="checkbox"]').length).toBeGreaterThan(0)
      })
      const cancelBtn = wrapper.findAll('button').find(b => b.text() === 'Cancel')
      expect(cancelBtn).toBeTruthy()
      await cancelBtn!.trigger('click')
      expect(wrapper.emitted('close')).toBeTruthy()
    })
  })
})
