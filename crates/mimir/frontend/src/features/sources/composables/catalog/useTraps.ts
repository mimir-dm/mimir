import { invoke } from '@tauri-apps/api/core'
import { useCatalogSearch } from './useCatalogSearch'

export interface TrapSummary {
  name: string
  source: string
  trap_type: string
  category: string
}

export interface TrapFilters {
  query?: string
  sources?: string[]
  categories?: string[]
  trap_types?: string[]
}

export interface TrapOrHazard {
  name: string
  source: string
  page?: number
  trap_haz_type?: string
  entries?: any[]
  srd?: boolean
  has_fluff?: boolean
  has_fluff_images?: boolean
}

export function useTraps() {
  const catalog = useCatalogSearch<TrapSummary, TrapOrHazard, TrapFilters>({
    name: 'trap',
    initializeCommand: 'init_trap_catalog',
    searchCommand: 'search_traps',
    detailsCommand: 'get_trap_details',
    transformFilters: (filters) => ({
      query: filters.query || null,
      sources: filters.sources || null,
      categories: filters.categories || null,
      trap_types: filters.trap_types || null
    }),
  })

  async function getTrapTypes(): Promise<string[]> {
    try {
      const types = await invoke<string[]>('get_trap_types')
      return types || []
    } catch (e) {
      return []
    }
  }

  return {
    isLoading: catalog.isLoading,
    error: catalog.error,
    initializeTrapCatalog: catalog.initialize,
    searchTraps: catalog.search,
    getTrapDetails: catalog.getDetails,
    getTrapTypes,
  }
}
