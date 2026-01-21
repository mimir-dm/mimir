import { describe, it, expect, beforeEach, vi } from 'vitest'
import { DocumentService } from '../DocumentService'
import { mockInvoke, createMockDocument } from '../../test/utils/mockTauri'

vi.mock('@tauri-apps/api/core')

describe('DocumentService', () => {
  let invoke: ReturnType<typeof mockInvoke>

  beforeEach(async () => {
    // Clear cache before each test
    DocumentService.clearCache()
    vi.clearAllMocks()
    
    // Import fresh mock for each test
    const { invoke: mockInvokeImport } = await import('@tauri-apps/api/core')
    invoke = mockInvokeImport as any
  })

  describe('list', () => {
    it('should fetch documents for a module', async () => {
      const mockDocs = [
        createMockDocument({ id: 1, title: 'Doc 1' }),
        createMockDocument({ id: 2, title: 'Doc 2' })
      ]
      
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      const result = await DocumentService.list(1, 2)
      
      expect(invoke).toHaveBeenCalledWith('get_documents_by_level', {
        campaignId: 2,
        level: 'module',
        moduleId: 1,
        sessionId: null
      })
      expect(result).toEqual(mockDocs)
    })

    it('should fetch documents for a campaign', async () => {
      const mockDocs = [
        createMockDocument({ id: 1, title: 'Doc 1' }),
        createMockDocument({ id: 2, title: 'Doc 2' })
      ]
      
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      const result = await DocumentService.list(undefined, 1)
      
      expect(invoke).toHaveBeenCalledWith('get_campaign_documents', {
        campaignId: 1
      })
      expect(result).toEqual(mockDocs)
    })

    it('should cache documents after fetching', async () => {
      const mockDocs = [createMockDocument()]
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      // First call
      await DocumentService.list(1)
      
      // Second call should use cache
      const result = await DocumentService.list(1)
      
      expect(invoke).toHaveBeenCalledTimes(1)
      expect(result).toEqual(mockDocs)
    })

    it('should handle errors gracefully', async () => {
      invoke.mockRejectedValueOnce(new Error('Network error'))
      
      await expect(DocumentService.list(1))
        .rejects
        .toThrow('Network error')
    })
  })

  describe('create', () => {
    it('should create a new document', async () => {
      const newDoc = createMockDocument({ id: 3, title: 'New Doc' })
      invoke.mockResolvedValueOnce({ data: newDoc })
      
      const result = await DocumentService.create({
        title: 'New Doc',
        content: 'Content',
        documentType: 'task'
      })
      
      expect(invoke).toHaveBeenCalledWith('create_document', {
        newDocument: {
          title: 'New Doc',
          content: 'Content',
          documentType: 'task'
        }
      })
      expect(result).toEqual(newDoc)
    })

    it('should clear cache after creation', async () => {
      const mockDocs = [createMockDocument()]
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      // Populate cache
      await DocumentService.list(1)
      
      // Create new document
      const newDoc = createMockDocument({ id: 2 })
      invoke.mockResolvedValueOnce({ data: newDoc })
      await DocumentService.create({ 
        title: 'New',
        content: 'Content',
        documentType: 'task'
      })
      
      // Next list should fetch fresh data
      invoke.mockResolvedValueOnce({ data: [...mockDocs, newDoc] })
      await DocumentService.list(1)
      
      expect(invoke).toHaveBeenCalledTimes(3)
    })
  })

  describe('update', () => {
    it('should update document content', async () => {
      const updatedDoc = createMockDocument({ id: 1, content: 'Updated content' })
      invoke.mockResolvedValueOnce({ data: updatedDoc })
      
      const result = await DocumentService.update(1, 'Updated content')
      
      expect(invoke).toHaveBeenCalledWith('update_document', {
        documentId: 1,
        update: {
          content: 'Updated content'
        }
      })
      expect(result).toEqual(updatedDoc)
    })

    it('should clear cache after update', async () => {
      const originalDocs = [createMockDocument({ id: 1, content: 'Original' })]
      invoke.mockResolvedValueOnce({ data: originalDocs })
      
      // Populate cache
      await DocumentService.list(1)
      
      // Update document
      const updatedDoc = createMockDocument({ id: 1, content: 'Updated' })
      invoke.mockResolvedValueOnce({ data: updatedDoc })
      await DocumentService.update(1, 'Updated')
      
      // Next list should fetch fresh data
      invoke.mockResolvedValueOnce({ data: [updatedDoc] })
      const result = await DocumentService.list(1)
      
      expect(result[0].content).toBe('Updated')
      expect(invoke).toHaveBeenCalledTimes(3)
    })
  })

  describe('updateMetadata', () => {
    it('should update document metadata', async () => {
      const updatedDoc = createMockDocument({ id: 1, title: 'Updated Title' })
      invoke.mockResolvedValueOnce({ data: updatedDoc })
      
      const result = await DocumentService.updateMetadata(1, { title: 'Updated Title' })
      
      expect(invoke).toHaveBeenCalledWith('update_document_metadata', {
        documentId: 1,
        title: 'Updated Title'
      })
      expect(result.title).toBe('Updated Title')
    })
  })

  describe('delete', () => {
    it('should delete a document', async () => {
      invoke.mockResolvedValueOnce(undefined)
      
      await DocumentService.delete(1)
      
      expect(invoke).toHaveBeenCalledWith('delete_document', {
        documentId: 1
      })
    })

    it('should clear cache after deletion', async () => {
      // Populate cache
      const mockDocs = [createMockDocument({ id: 1 })]
      invoke.mockResolvedValueOnce({ data: mockDocs })
      await DocumentService.list(1)
      
      // Delete document
      invoke.mockResolvedValueOnce(undefined)
      await DocumentService.delete(1)
      
      // Next list should fetch fresh data
      invoke.mockResolvedValueOnce({ data: [] })
      await DocumentService.list(1)
      
      expect(invoke).toHaveBeenCalledTimes(3)
    })
  })

  describe('complete', () => {
    it('should mark a document as complete', async () => {
      const completedDoc = createMockDocument({ 
        id: 1, 
        completed_at: '2024-01-02T00:00:00Z' 
      })
      invoke.mockResolvedValueOnce({ data: completedDoc })
      
      const result = await DocumentService.complete(1)
      
      expect(invoke).toHaveBeenCalledWith('complete_document', {
        documentId: 1
      })
      expect(result.completed_at).toBeTruthy()
    })
  })

  describe('uncomplete', () => {
    it('should mark a document as incomplete', async () => {
      const incompleteDoc = createMockDocument({ 
        id: 1, 
        completed_at: null 
      })
      invoke.mockResolvedValueOnce({ data: incompleteDoc })
      
      const result = await DocumentService.uncomplete(1)
      
      expect(invoke).toHaveBeenCalledWith('update_document', {
        documentId: 1,
        update: {
          completed_at: null
        }
      })
      expect(result.completed_at).toBeNull()
    })
  })

  describe('transition', () => {
    it('should transition document to new phase', async () => {
      const transitionedDoc = createMockDocument({ 
        id: 1, 
        phase: 'published' 
      })
      invoke.mockResolvedValueOnce({ data: transitionedDoc })
      
      const result = await DocumentService.transition(1, 'published')
      
      expect(invoke).toHaveBeenCalledWith('transition_document_phase', {
        documentId: 1,
        phase: 'published'
      })
      expect(result.phase).toBe('published')
    })
  })

  describe('validateExitCriteria', () => {
    it('should validate exit criteria', async () => {
      invoke.mockResolvedValueOnce({ data: true })
      
      const result = await DocumentService.validateExitCriteria(1)
      
      expect(invoke).toHaveBeenCalledWith('validate_exit_criteria', {
        documentId: 1
      })
      expect(result).toBe(true)
    })
  })

  describe('getByType', () => {
    it('should get documents by type', async () => {
      const mockDocs = [
        createMockDocument({ document_type: 'task' }),
        createMockDocument({ document_type: 'task' })
      ]
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      const result = await DocumentService.getByType(1, 'task')
      
      expect(invoke).toHaveBeenCalledWith('get_documents_by_level', {
        campaignId: undefined,
        level: 'module',
        moduleId: 1,
        sessionId: null
      })
      expect(result).toHaveLength(2)
    })
  })

  describe('getByTemplate', () => {
    it('should get document by template', async () => {
      const mockDoc = createMockDocument({ template_id: 'test-template' })
      const mockDocs = [mockDoc]
      invoke.mockResolvedValueOnce({ data: mockDocs })
      
      const result = await DocumentService.getByTemplate(1, 'test-template')
      
      expect(invoke).toHaveBeenCalledWith('get_documents_by_level', {
        campaignId: undefined,
        level: 'module',
        moduleId: 1,
        sessionId: null
      })
      expect(result).toEqual(mockDoc)
    })

    it('should return undefined if no matching template', async () => {
      invoke.mockResolvedValueOnce({ data: [] })
      
      const result = await DocumentService.getByTemplate(1, 'non-existent')
      
      expect(result).toBeUndefined()
    })
  })

  describe('batchUpdate', () => {
    it('should batch update multiple documents', async () => {
      const updates = [
        { id: 1, content: 'Content 1' },
        { id: 2, content: 'Content 2' }
      ]
      const updatedDocs = [
        createMockDocument({ id: 1, content: 'Content 1' }),
        createMockDocument({ id: 2, content: 'Content 2' })
      ]
      
      invoke.mockResolvedValueOnce({ data: updatedDocs[0] })
      invoke.mockResolvedValueOnce({ data: updatedDocs[1] })
      
      const result = await DocumentService.batchUpdate(updates)
      
      expect(invoke).toHaveBeenCalledTimes(2)
      expect(result).toEqual(updatedDocs)
    })
  })

  describe('cache management', () => {
    it('should clear all caches with clearCache', () => {
      // This just ensures the method exists and doesn't throw
      DocumentService.clearCache()
      expect(true).toBe(true)
    })

    it('should handle cache key collisions properly', async () => {
      const moduleDocs = [createMockDocument({ id: 1, title: 'Module Doc' })]
      const campaignDocs = [createMockDocument({ id: 2, title: 'Campaign Doc' })]
      
      invoke.mockResolvedValueOnce({ data: moduleDocs })
      await DocumentService.list(1)
      
      invoke.mockResolvedValueOnce({ data: campaignDocs })
      await DocumentService.list(undefined, 1)
      
      // Should have made two separate calls
      expect(invoke).toHaveBeenCalledTimes(2)
    })
  })
})