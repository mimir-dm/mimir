/**
 * Document Service
 *
 * Provides access to document CRUD operations via Tauri commands.
 * Types match mimir-core Document model.
 */

import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse, Document } from '@/types/api'
import { dataEvents } from '@/utils/dataEvents'

// =============================================================================
// Request Types
// =============================================================================

export interface CreateDocumentRequest {
  campaign_id: string
  module_id?: string
  title: string
  doc_type?: string
  content?: string
}

export interface UpdateDocumentRequest {
  title?: string
  content?: string
  doc_type?: string
}

// =============================================================================
// Search Result Type
// =============================================================================

export interface DocumentSearchResult {
  id: string
  campaign_id: string
  module_id: string | null
  title: string
  doc_type: string
  snippet: string
  created_at: string
  updated_at: string
}

// =============================================================================
// Document Service
// =============================================================================

class DocumentServiceClass {
  /**
   * List campaign-level documents (not in any module)
   */
  async listForCampaign(campaignId: string): Promise<Document[]> {
    const response = await invoke<ApiResponse<Document[]>>('list_campaign_documents', {
      campaignId
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to list campaign documents')
  }

  /**
   * List all documents for a specific module
   */
  async listForModule(moduleId: string): Promise<Document[]> {
    const response = await invoke<ApiResponse<Document[]>>('list_module_documents', {
      moduleId
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to list module documents')
  }

  /**
   * Get a document by ID
   */
  async get(id: string): Promise<Document> {
    const response = await invoke<ApiResponse<Document>>('get_document', { id })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || `Failed to get document ${id}`)
  }

  /**
   * Create a new document
   */
  async create(request: CreateDocumentRequest): Promise<Document> {
    const response = await invoke<ApiResponse<Document>>('create_document', { request })

    if (response.success && response.data) {
      if (request.module_id) {
        dataEvents.emit('document:created', {
          moduleId: request.module_id,
          documentId: response.data.id
        })
      } else {
        dataEvents.emit('document:created', {
          campaignId: request.campaign_id,
          documentId: response.data.id
        })
      }
      return response.data
    }

    throw new Error(response.error || 'Failed to create document')
  }

  /**
   * Update a document
   */
  async update(id: string, request: UpdateDocumentRequest): Promise<Document> {
    const response = await invoke<ApiResponse<Document>>('update_document', {
      id,
      request
    })

    if (response.success && response.data) {
      dataEvents.emit('document:updated', { documentId: id })
      return response.data
    }

    throw new Error(response.error || `Failed to update document ${id}`)
  }

  /**
   * Delete a document
   */
  async delete(id: string, moduleId?: string): Promise<void> {
    const response = await invoke<ApiResponse<void>>('delete_document', { id })

    if (response.success) {
      if (moduleId) {
        dataEvents.emit('document:deleted', { moduleId, documentId: id })
      }
      return
    }

    throw new Error(response.error || `Failed to delete document ${id}`)
  }

  /**
   * Search documents in a campaign
   */
  async search(campaignId: string, query: string): Promise<DocumentSearchResult[]> {
    const response = await invoke<ApiResponse<DocumentSearchResult[]>>('search_documents', {
      campaignId,
      query
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to search documents')
  }

  /**
   * Search documents within a specific module
   */
  async searchInModule(moduleId: string, query: string): Promise<DocumentSearchResult[]> {
    const response = await invoke<ApiResponse<DocumentSearchResult[]>>('search_module_documents', {
      moduleId,
      query
    })

    if (response.success && response.data) {
      return response.data
    }

    throw new Error(response.error || 'Failed to search module documents')
  }

  /**
   * Swap sort order between two documents
   */
  async reorder(documentId: string, swapWithId: string): Promise<Document[]> {
    const response = await invoke<ApiResponse<Document[]>>('reorder_document', {
      documentId,
      swapWithId
    })

    if (response.success && response.data) {
      dataEvents.emit('document:reordered', { documentId })
      return response.data
    }

    throw new Error(response.error || `Failed to reorder document ${documentId}`)
  }

  /**
   * Update only the content of a document
   */
  async updateContent(id: string, content: string): Promise<Document> {
    return this.update(id, { content })
  }

  /**
   * Update only the title of a document
   */
  async updateTitle(id: string, title: string): Promise<Document> {
    return this.update(id, { title })
  }

  /**
   * Get documents by type from a module
   */
  async getByType(moduleId: string, docType: string): Promise<Document[]> {
    const documents = await this.listForModule(moduleId)
    return documents.filter(d => d.doc_type === docType)
  }
}

export const DocumentService = new DocumentServiceClass()

// Re-export Document type for backwards compatibility
export type { Document } from '@/types/api'
