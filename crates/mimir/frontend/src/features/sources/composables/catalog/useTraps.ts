import { invoke } from '@tauri-apps/api/core'
import { useCatalogSearch } from './useCatalogSearch'
import { useCampaignStore } from '@/stores/campaigns'

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
  // Get effective sources: explicit filter sources, or campaign sources if configured
  // Note: Store access is lazy to avoid Pinia initialization issues
  const getEffectiveSources = (filterSources?: string[]): string[] | null => {
    if (filterSources && filterSources.length > 0) {
      return filterSources
    }
    try {
      const campaignStore = useCampaignStore()
      if (campaignStore.currentCampaignSources.length > 0) {
        return campaignStore.currentCampaignSources
      }
    } catch {
      // Store not available yet, use no filter
    }
    return null
  }

  const catalog = useCatalogSearch<TrapSummary, TrapOrHazard, TrapFilters>({
    name: 'trap',
    // No initialization needed - database-backed
    searchCommand: 'search_traps',
    detailsCommand: 'get_trap_by_name',
    transformFilters: (filters) => ({
      name_contains: filters.query || null,
      sources: getEffectiveSources(filters.sources),
    }),
  })

  async function getTrapTypes(): Promise<string[]> {
    try {
      const response = await invoke<{ success: boolean; data?: string[]; error?: string }>('list_trap_sources')
      if (response.success && response.data) {
        return response.data
      }
      return []
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
