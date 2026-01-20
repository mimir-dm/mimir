import { invoke } from '@tauri-apps/api/core'
import { useCatalogSearch } from './useCatalogSearch'

export interface DeitySummary {
  name: string
  source: string
  title: string
  pantheon: string
  alignment: string
  domains: string[]
  symbol: string
}

export interface Deity {
  name: string
  source: string
  page?: number
  title?: string
  pantheon?: string
  alignment?: string[] | string
  domains?: string[]
  symbol?: string
  additionalSources?: any[]
  entries?: any[]
  srd?: boolean
  hasFluff?: boolean
  hasFluffImages?: boolean
}

export interface DeityFilters {
  query?: string
  sources?: string[]
  pantheons?: string[]
  domains?: string[]
}

export function useDeities() {
  const catalog = useCatalogSearch<DeitySummary, Deity, DeityFilters>({
    name: 'deity',
    searchCommand: 'search_deities',
    detailsCommand: 'get_deity_details',
    transformFilters: (filters) => ({
      filters: {
        name: filters.query || null,
        sources: filters.sources || null,
        pantheons: filters.pantheons || null,
        domains: filters.domains || null,
        alignments: null
      }
    }),
  })

  // Custom getDetails with different parameter names
  async function getDeityDetails(name: string, source: string): Promise<Deity | null> {
    try {
      const deity = await invoke<Deity>('get_deity_details', { deityName: name, deitySource: source })
      return deity
    } catch (e) {
      return null
    }
  }

  return {
    isDeitiesInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    initializeDeityCatalog: catalog.initialize,
    searchDeities: catalog.search,
    getDeityDetails,
  }
}
