/**
 * Tests for ModuleDocumentsPanel.vue
 *
 * Tests the module documents panel on the campaign dashboard including:
 * - Document title display with formatting
 * - Empty state
 * - Create button
 * - Document selection
 * - Delete button
 */

import { describe, it, expect } from 'vitest'
import { shallowMountWithPlugins } from '@tests/helpers/mountHelpers'
import ModuleDocumentsPanel from '@/features/campaigns/components/dashboard/modules/ModuleDocumentsPanel.vue'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeDocument(overrides: Record<string, unknown> = {}) {
  return {
    id: 'doc-1',
    campaign_id: 'camp-1',
    module_id: 'mod-1',
    title: 'session-notes',
    doc_type: 'custom',
    content: '# Session Notes',
    sort_order: 0,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('ModuleDocumentsPanel', () => {
  function mountPanel(documents = [makeDocument()]) {
    return shallowMountWithPlugins(ModuleDocumentsPanel, {
      props: { documents },
    })
  }

  describe('rendering documents', () => {
    it('displays formatted document titles', () => {
      const wrapper = mountPanel([
        makeDocument({ title: 'session-notes' }),
      ])
      // kebab-case should be converted to Title Case
      expect(wrapper.text()).toContain('Session Notes')
    })

    it('formats underscore titles', () => {
      const wrapper = mountPanel([
        makeDocument({ title: 'world_primer' }),
      ])
      expect(wrapper.text()).toContain('World Primer')
    })

    it('displays multiple documents', () => {
      const wrapper = mountPanel([
        makeDocument({ id: 'd1', title: 'session-notes' }),
        makeDocument({ id: 'd2', title: 'treasure-list' }),
      ])
      expect(wrapper.text()).toContain('Session Notes')
      expect(wrapper.text()).toContain('Treasure List')
    })

    it('handles Untitled for documents with no title', () => {
      const wrapper = mountPanel([
        makeDocument({ title: '' }),
      ])
      // Empty title falls back to formatDocumentTitle('Untitled')
      expect(wrapper.text()).toContain('Untitled')
    })
  })

  describe('header', () => {
    it('displays Documents title', () => {
      const wrapper = mountPanel()
      expect(wrapper.text()).toContain('Documents')
    })

    it('has create button', () => {
      const wrapper = mountPanel()
      expect(wrapper.find('.btn-add').exists()).toBe(true)
    })
  })

  describe('empty state', () => {
    it('shows empty message when no documents exist', () => {
      const wrapper = mountPanel([])
      expect(wrapper.text()).toContain('No documents yet')
    })
  })

  describe('events', () => {
    it('emits select when document is clicked', async () => {
      const doc = makeDocument()
      const wrapper = mountPanel([doc])
      await wrapper.find('.document-card').trigger('click')
      expect(wrapper.emitted('select')).toBeTruthy()
      expect(wrapper.emitted('select')![0]).toEqual([doc])
    })

    it('emits create when add button is clicked', async () => {
      const wrapper = mountPanel()
      await wrapper.find('.btn-add').trigger('click')
      expect(wrapper.emitted('create')).toBeTruthy()
    })

    it('emits delete when delete button is clicked', async () => {
      const doc = makeDocument()
      const wrapper = mountPanel([doc])
      await wrapper.find('.doc-delete-btn').trigger('click')
      expect(wrapper.emitted('delete')).toBeTruthy()
      expect(wrapper.emitted('delete')![0]).toEqual([doc])
    })
  })
})
