import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ObjectSummary {
  name: string
  source: string
  object_type?: string
}

export interface ObjectFilters {
  query?: string
  sources?: string[]
  object_type?: string
}

export interface DndObject {
  name: string
  source: string
  page?: number
  objectType?: string
  size?: string[]
  ac?: any
  hp?: number
  immune?: string[]
  resist?: string[]
  vulnerable?: string[]
  actionEntries?: any[]
  entries?: any[]
  hasToken?: boolean
  tokenCredit?: string
  srd?: boolean
  hasFluff?: boolean
  hasFluffImages?: boolean
}

export function useObjects() {
  const isObjectsInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const objects = ref<ObjectSummary[]>([])

  async function initializeObjectCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchObjects(filters: ObjectFilters = {}): Promise<ObjectSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend ObjectFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
        object_type: filters.object_type || null,
      }

      const response = await invoke<{ success: boolean; data?: ObjectSummary[]; error?: string }>('search_objects', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        objects.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Search failed'
        return []
      }
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getObjectDetails(name: string, source: string): Promise<DndObject | null> {
    try {
      const response = await invoke<{ success: boolean; data?: DndObject; error?: string }>('get_object_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get object details:', e)
      return null
    }
  }

  return {
    isObjectsInitialized,
    isLoading,
    error,
    objects,
    initializeObjectCatalog,
    searchObjects,
    getObjectDetails,
  }
}
