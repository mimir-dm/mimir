import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface CultSummary {
  name: string
  source: string
}

export interface CultFilters {
  query?: string
  sources?: string[]
}

export interface Cult {
  name: string
  source: string
  page?: number
  entries?: any[]
  cultists?: { entry: string }
  goal?: { entry: string }
  signature_spells?: { entry: string }
}

export function useCults() {
  const isCultsInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const cults = ref<CultSummary[]>([])

  async function initializeCultCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchCults(filters: CultFilters = {}): Promise<CultSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend CultFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
      }

      const response = await invoke<{ success: boolean; data?: CultSummary[]; error?: string }>('search_cults', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        cults.value = response.data
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

  async function getCultDetails(name: string, source: string): Promise<Cult | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Cult; error?: string }>('get_cult_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get cult details:', e)
      return null
    }
  }

  return {
    isCultsInitialized,
    isLoading,
    error,
    cults,
    initializeCultCatalog,
    searchCults,
    getCultDetails,
  }
}
