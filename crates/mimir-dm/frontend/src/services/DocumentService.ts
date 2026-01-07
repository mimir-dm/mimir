import { invoke } from '@tauri-apps/api/core'

export interface DocumentData {
  title: string
  content: string
  documentType: 'vision' | 'strategy' | 'initiative' | 'task' | 'adr'
  parentId?: string
  templateId?: string
  moduleId?: number
  campaignId?: number
}

export interface Document {
  id: string | number
  title: string
  content: string
  phase?: 'draft' | 'review' | 'published' | 'archived'
  documentType?: 'vision' | 'strategy' | 'initiative' | 'task' | 'adr'
  document_type?: string
  parent_id?: string
  parentId?: string
  template_id?: string
  templateId?: string
  created_at?: string
  createdAt?: Date
  updated_at?: string
  updatedAt?: Date
  completed_at?: string | null
  completedAt?: Date | null
  blocked_by?: string[]
  blockedBy?: string[]
  exit_criteria?: any[]
  exitCriteria?: any[]
  // Additional fields for compatibility
  campaign_id?: number
  module_id?: number | null
  session_id?: number | null
  file_path?: string
  file_type?: string
  is_user_created?: boolean
}

class DocumentServiceClass {
  private cache = new Map<string, Document[]>()
  
  async create(data: DocumentData): Promise<Document> {
    try {
      const response = await invoke<{ data: Document }>('create_document', {
        newDocument: data
      })

      this.clearCache()
      return response.data
    } catch (error) {
      throw new Error(`Failed to create document "${data.title}": ${error}`)
    }
  }
  
  async update(id: string | number, content: string): Promise<Document> {
    try {
      const response = await invoke<{ data: Document }>('update_document', {
        documentId: Number(id),
        update: {
          content
        }
      })

      this.clearCache()
      return response.data
    } catch (error) {
      throw new Error(`Failed to update document ${id}: ${error}`)
    }
  }
  
  async updateMetadata(id: string | number, metadata: Partial<Document>): Promise<Document> {
    try {
      const response = await invoke<{ data: Document }>('update_document_metadata', {
        documentId: Number(id),
        ...metadata
      })

      this.clearCache()
      return response.data
    } catch (error) {
      throw new Error(`Failed to update metadata for document ${id}: ${error}`)
    }
  }
  
  async delete(id: string | number): Promise<void> {
    try {
      await invoke('delete_document', {
        documentId: Number(id)
      })

      this.clearCache()
    } catch (error) {
      throw new Error(`Failed to delete document ${id}: ${error}`)
    }
  }
  
  async transition(id: string | number, phase: string): Promise<Document> {
    try {
      const response = await invoke<{ data: Document }>('transition_document_phase', {
        documentId: Number(id),
        phase: phase
      })

      this.clearCache()
      return response.data
    } catch (error) {
      throw new Error(`Failed to transition document ${id} to ${phase}: ${error}`)
    }
  }
  
  async complete(id: string | number): Promise<Document> {
    try {
      const response = await invoke<{ data: Document }>('complete_document', {
        documentId: Number(id)
      })

      this.clearCache()
      return response.data
    } catch (error) {
      throw new Error(`Failed to complete document ${id}: ${error}`)
    }
  }

  async uncomplete(id: string | number): Promise<Document> {
    try {
      // There's no uncomplete_document command, so we use update_document
      // to set completed_at to null
      const response = await invoke<{ data: Document }>('update_document', {
        documentId: Number(id),
        update: {
          completed_at: null
        }
      })

      this.clearCache()
      return response.data
    } catch (error) {
      throw new Error(`Failed to uncomplete document ${id}: ${error}`)
    }
  }
  
  async validateExitCriteria(id: string | number): Promise<boolean> {
    try {
      const response = await invoke<{ data: boolean }>('validate_exit_criteria', {
        documentId: Number(id)
      })

      return response.data
    } catch (error) {
      throw new Error(`Failed to validate exit criteria for document ${id}: ${error}`)
    }
  }
  
  async list(moduleId?: number, campaignId?: number): Promise<Document[]> {
    const cacheKey = `${moduleId || ''}-${campaignId || ''}`

    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)!
    }

    try {
      let response: { data: Document[] }

      if (campaignId && !moduleId) {
        // Use get_campaign_documents for campaign-level documents
        response = await invoke<{ data: Document[] }>('get_campaign_documents', {
          campaignId: campaignId
        })
      } else if (moduleId) {
        // Use get_documents_by_level for module documents
        response = await invoke<{ data: Document[] }>('get_documents_by_level', {
          campaignId: campaignId,
          level: 'module',
          moduleId: moduleId,
          sessionId: null
        })
      } else {
        // No campaignId or moduleId - return empty array
        response = { data: [] }
      }

      const documents = response.data || []
      this.cache.set(cacheKey, documents)
      return documents
    } catch (error) {
      const context = moduleId
        ? `module ${moduleId}`
        : campaignId
        ? `campaign ${campaignId}`
        : 'unknown context'
      throw new Error(`Failed to list documents for ${context}: ${error}`)
    }
  }
  
  async getByType(moduleId: number | undefined, type: string): Promise<Document[]> {
    const all = await this.list(moduleId)
    return all.filter(d => 
      d.documentType === type || d.document_type === type
    )
  }
  
  async getByTemplate(moduleId: number, templateId: string): Promise<Document | undefined> {
    const all = await this.list(moduleId)
    return all.find(d => 
      d.templateId === templateId || d.template_id === templateId
    )
  }
  
  async batchUpdate(updates: Array<{ id: string | number; content: string }>): Promise<Document[]> {
    const results = await Promise.all(
      updates.map(({ id, content }) => this.update(id, content))
    )
    
    this.clearCache()
    return results
  }
  
  clearCache() {
    this.cache.clear()
  }
}

export const DocumentService = new DocumentServiceClass()