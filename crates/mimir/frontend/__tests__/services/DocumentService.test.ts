/**
 * Tests for DocumentService
 *
 * Tests the document service layer that wraps Tauri invoke commands
 * for document CRUD operations.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  setupInvokeMock,
  resetInvokeMock,
  mockCommand,
  mockCommandError,
  expectCommandCalledWith,
} from '@tests/helpers/mockInvoke'
import { DocumentService } from '@/services/DocumentService'

// ─── Factories ──────────────────────────────────────────────────────────────

function makeDocument(overrides: Record<string, unknown> = {}) {
  return {
    id: 'doc-1',
    campaign_id: 'camp-1',
    module_id: null,
    title: 'Session Notes',
    doc_type: 'custom',
    content: '# Notes',
    sort_order: 0,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

describe('DocumentService', () => {
  beforeEach(() => {
    setupInvokeMock()
  })

  afterEach(() => {
    resetInvokeMock()
  })

  describe('listForCampaign', () => {
    it('returns campaign documents', async () => {
      const docs = [makeDocument(), makeDocument({ id: 'doc-2', title: 'World Primer' })]
      mockCommand('list_campaign_documents', docs)

      const result = await DocumentService.listForCampaign('camp-1')
      expect(result).toHaveLength(2)
      expectCommandCalledWith('list_campaign_documents', { campaignId: 'camp-1' })
    })

    it('throws on error', async () => {
      mockCommandError('list_campaign_documents', 'Database error')
      await expect(DocumentService.listForCampaign('camp-1')).rejects.toThrow('Database error')
    })
  })

  describe('listForModule', () => {
    it('returns module documents', async () => {
      const docs = [makeDocument({ module_id: 'mod-1' })]
      mockCommand('list_module_documents', docs)

      const result = await DocumentService.listForModule('mod-1')
      expect(result).toHaveLength(1)
      expectCommandCalledWith('list_module_documents', { moduleId: 'mod-1' })
    })
  })

  describe('create', () => {
    it('creates a document and returns it', async () => {
      mockCommand('create_document', makeDocument())

      const result = await DocumentService.create({
        campaign_id: 'camp-1',
        title: 'Session Notes',
      })

      expect(result.title).toBe('Session Notes')
      expectCommandCalledWith('create_document', {
        request: { campaign_id: 'camp-1', title: 'Session Notes' },
      })
    })

    it('creates a module document', async () => {
      mockCommand('create_document', makeDocument({ module_id: 'mod-1' }))

      await DocumentService.create({
        campaign_id: 'camp-1',
        module_id: 'mod-1',
        title: 'Module Notes',
      })

      expectCommandCalledWith('create_document', {
        request: { campaign_id: 'camp-1', module_id: 'mod-1', title: 'Module Notes' },
      })
    })

    it('throws on error', async () => {
      mockCommandError('create_document', 'Validation failed')
      await expect(
        DocumentService.create({ campaign_id: 'camp-1', title: '' }),
      ).rejects.toThrow('Validation failed')
    })
  })

  describe('update', () => {
    it('updates document content', async () => {
      mockCommand('update_document', makeDocument({ content: 'Updated content' }))

      const result = await DocumentService.update('doc-1', { content: 'Updated content' })
      expect(result.content).toBe('Updated content')
      expectCommandCalledWith('update_document', {
        id: 'doc-1',
        request: { content: 'Updated content' },
      })
    })

    it('throws on error', async () => {
      mockCommandError('update_document', 'Not found')
      await expect(DocumentService.update('doc-1', { title: 'x' })).rejects.toThrow('Not found')
    })
  })

  describe('delete', () => {
    it('deletes a document', async () => {
      mockCommand('delete_document', null)

      await DocumentService.delete('doc-1')
      expectCommandCalledWith('delete_document', { id: 'doc-1' })
    })

    it('throws on error', async () => {
      mockCommandError('delete_document', 'Not found')
      await expect(DocumentService.delete('doc-1')).rejects.toThrow('Not found')
    })
  })

  describe('reorder', () => {
    it('swaps two documents', async () => {
      const reordered = [
        makeDocument({ id: 'doc-2', sort_order: 0 }),
        makeDocument({ id: 'doc-1', sort_order: 1 }),
      ]
      mockCommand('reorder_document', reordered)

      const result = await DocumentService.reorder('doc-1', 'doc-2')
      expect(result).toHaveLength(2)
      expectCommandCalledWith('reorder_document', {
        documentId: 'doc-1',
        swapWithId: 'doc-2',
      })
    })
  })

  describe('search', () => {
    it('searches documents in a campaign', async () => {
      const results = [
        { id: 'doc-1', campaign_id: 'camp-1', module_id: null, title: 'Session Notes', doc_type: 'custom', snippet: '...matching text...', created_at: '', updated_at: '' },
      ]
      mockCommand('search_documents', results)

      const result = await DocumentService.search('camp-1', 'session')
      expect(result).toHaveLength(1)
      expect(result[0].snippet).toContain('matching text')
      expectCommandCalledWith('search_documents', { campaignId: 'camp-1', query: 'session' })
    })
  })

  describe('convenience methods', () => {
    it('updateContent delegates to update', async () => {
      mockCommand('update_document', makeDocument({ content: 'new content' }))

      const result = await DocumentService.updateContent('doc-1', 'new content')
      expect(result.content).toBe('new content')
      expectCommandCalledWith('update_document', {
        id: 'doc-1',
        request: { content: 'new content' },
      })
    })

    it('updateTitle delegates to update', async () => {
      mockCommand('update_document', makeDocument({ title: 'New Title' }))

      const result = await DocumentService.updateTitle('doc-1', 'New Title')
      expect(result.title).toBe('New Title')
    })

    it('getByType filters module documents', async () => {
      const docs = [
        makeDocument({ id: 'd1', doc_type: 'custom' }),
        makeDocument({ id: 'd2', doc_type: 'template' }),
        makeDocument({ id: 'd3', doc_type: 'custom' }),
      ]
      mockCommand('list_module_documents', docs)

      const result = await DocumentService.getByType('mod-1', 'custom')
      expect(result).toHaveLength(2)
      expect(result.every(d => d.doc_type === 'custom')).toBe(true)
    })
  })
})
