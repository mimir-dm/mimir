/**
 * Tests for ModuleUserDocuments.vue
 *
 * Tests the module user documents panel including:
 * - Loading and displaying documents
 * - Document title and file type display
 * - Document count badge
 * - Add button
 * - Empty state when no documents exist
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mountWithPlugins } from '@tests/helpers/mountHelpers'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommandRaw,
} from '@tests/helpers/mockInvoke'
import ModuleUserDocuments from '@/features/modules/components/ModuleUserDocuments.vue'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeDocument(overrides: Record<string, unknown> = {}) {
  return {
    id: 'doc-1',
    campaign_id: 'camp-1',
    module_id: 'mod-1',
    title: 'Session Notes',
    file_path: '/notes/session-1.md',
    file_type: 'markdown',
    is_user_created: true,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('ModuleUserDocuments', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  function mountComponent(documents = [makeDocument()]) {
    // get_user_documents returns { data: [...] } without ApiResponse wrapper
    mockCommandRaw('get_user_documents', { data: documents })
    return mountWithPlugins(ModuleUserDocuments, {
      props: {
        moduleId: 'mod-1',
        campaignId: 'camp-1',
      },
      stubs: {
        EmptyState: false,
        CreateDocumentModal: true,
        AppModal: true,
      },
    })
  }

  describe('rendering documents', () => {
    it('displays document titles', async () => {
      const wrapper = mountComponent([
        makeDocument({ id: 'd1', title: 'Session Notes' }),
        makeDocument({ id: 'd2', title: 'Treasure List' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('Session Notes')
        expect(wrapper.text()).toContain('Treasure List')
      })
    })

    it('shows document file type', async () => {
      const wrapper = mountComponent([
        makeDocument({ file_type: 'markdown' }),
      ])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('markdown')
      })
    })

    it('shows document count badge', async () => {
      const wrapper = mountComponent([
        makeDocument({ id: 'd1' }),
        makeDocument({ id: 'd2' }),
      ])
      await vi.waitFor(() => {
        const count = wrapper.find('.document-count')
        expect(count.exists()).toBe(true)
        expect(count.text()).toBe('2')
      })
    })
  })

  describe('header', () => {
    it('displays My Documents title', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('My Documents')
      })
    })

    it('has add button', async () => {
      const wrapper = mountComponent()
      await vi.waitFor(() => {
        expect(wrapper.find('.add-btn').exists()).toBe(true)
      })
    })
  })

  describe('empty state', () => {
    it('shows empty message when no documents exist', async () => {
      const wrapper = mountComponent([])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No custom documents yet')
      })
    })

    it('does not show count badge when empty', async () => {
      const wrapper = mountComponent([])
      await vi.waitFor(() => {
        expect(wrapper.text()).toContain('No custom documents yet')
      })
      expect(wrapper.find('.document-count').exists()).toBe(false)
    })
  })
})
