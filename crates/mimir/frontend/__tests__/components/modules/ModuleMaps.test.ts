/**
 * Tests for ModuleMaps.vue
 *
 * Tests the module maps panel including:
 * - Loading and displaying maps for a module
 * - Map card rendering with name and dimensions
 * - Grid type badge
 * - Upload Map button
 * - Empty state when no maps exist
 * - Map selection emits event
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
} from '@tests/helpers/mockInvoke'
import ModuleMaps from '@/features/modules/components/ModuleMaps.vue'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeMap(overrides: Record<string, unknown> = {}) {
  return {
    id: 'map-1',
    campaign_id: 'camp-1',
    module_id: 'mod-1',
    name: 'Dungeon Level 1',
    image_path: '/path/to/map.png',
    width_px: 2800,
    height_px: 2100,
    grid_type: 'square',
    grid_size_px: 70,
    grid_offset_x: 0,
    grid_offset_y: 0,
    original_width_px: 2800,
    original_height_px: 2100,
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('ModuleMaps', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  function mountComponent(maps = [makeMap()]) {
    mockCommand('list_maps', maps)
    // Mock thumbnail loading — each map triggers serve_map_image
    mockCommand('serve_map_image', 'data:image/png;base64,abc')
    return mountWithPlugins(ModuleMaps, {
      props: {
        moduleId: 'mod-1',
        campaignId: 'camp-1',
      },
      stubs: {
        EmptyState: false,
        MapUploadModal: true,
        MapTokenSetupModal: true,
        MapPrintDialog: true,
      },
    })
  }

  describe('rendering maps', () => {
    it('displays map names', async () => {
      const wrapper = mountComponent([
        makeMap({ id: 'm1', name: 'Dungeon Level 1' }),
        makeMap({ id: 'm2', name: 'Boss Chamber' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Dungeon Level 1')
        expect(wrapper.text()).toContain('Boss Chamber')
      })
    })

    it('shows map dimensions', async () => {
      const wrapper = mountComponent([
        makeMap({ width_px: 2800, height_px: 2100 }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('2800x2100')
      })
    })

    it('shows grid type badge', async () => {
      const wrapper = mountComponent([
        makeMap({ grid_type: 'square' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('square grid')
      })
    })

    it('does not show grid badge for none grid type', async () => {
      const wrapper = mountComponent([
        makeMap({ grid_type: 'none' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Dungeon Level 1')
      })
      expect(wrapper.text()).not.toContain('grid')
    })
  })

  describe('header', () => {
    it('displays Module Maps title', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Module Maps')
      })
    })

    it('has Upload Map button', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        const buttons = wrapper.findAll('button')
        const uploadBtn = buttons.find(b => b.text().includes('Upload Map'))
        expect(uploadBtn).toBeTruthy()
      })
    })
  })

  describe('empty state', () => {
    it('shows empty message when no maps exist', async () => {
      const wrapper = mountComponent([])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No maps for this module yet')
      })
    })
  })

  describe('selection', () => {
    it('emits selectMap when a map card is clicked', async () => {
      const map = makeMap()
      const wrapper = mountComponent([map])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Dungeon Level 1')
      })
      await wrapper.find('.map-card').trigger('click')
      expect(wrapper.emitted('selectMap')).toBeTruthy()
    })
  })
})
