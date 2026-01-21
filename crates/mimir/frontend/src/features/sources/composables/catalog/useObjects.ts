import { invoke } from '@tauri-apps/api/core'
import { useCatalogSearch } from './useCatalogSearch'

export interface ObjectSummary {
  name: string
  source: string
  object_type: string
  size: string
  ac: string
  hp: string
}

export interface ObjectFilters {
  query?: string
  sources?: string[]
  object_types?: string[]
  sizes?: string[]
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
  const catalog = useCatalogSearch<ObjectSummary, string | null, ObjectFilters>({
    name: 'object',
    initializeCommand: 'init_object_catalog',
    searchCommand: 'search_objects',
    detailsCommand: 'get_object_details',
    transformFilters: (filters) => ({
      search: filters.query || null,
      sources: filters.sources && filters.sources.length > 0 ? filters.sources : null,
      object_types: filters.object_types && filters.object_types.length > 0 ? filters.object_types : null,
      sizes: filters.sizes && filters.sizes.length > 0 ? filters.sizes : null
    }),
  })

  // Custom getDetails that parses JSON
  async function getObjectDetails(name: string, source: string): Promise<DndObject | null> {
    try {
      const jsonString = await invoke<string | null>('get_object_details', { name, source })
      if (!jsonString) {
        return null
      }

      const objectData = JSON.parse(jsonString)
      return objectData as DndObject
    } catch (e) {
      return null
    }
  }

  return {
    isObjectsInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    initializeObjectCatalog: catalog.initialize,
    searchObjects: catalog.search,
    getObjectDetails,
  }
}
