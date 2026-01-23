import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface PsionicSummary {
  name: string
  source: string
  psionic_type?: string
  psionic_order?: string
}

export interface PsionicFilters {
  query?: string
  sources?: string[]
  psionic_type?: string
  psionic_order?: string
}

export interface Psionic {
  name: string
  source: string
  psionic_type?: string
  psionic_order?: string
  page?: number
  entries?: any[]
  focus?: string
  modes?: PsionicMode[]
}

export interface PsionicMode {
  name: string
  cost: {
    min: number
    max?: number
  }
  entries: any[]
  concentration?: {
    duration: number
    unit: string
  }
}

export function usePsionics() {
  const isPsionicsInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const psionics = ref<PsionicSummary[]>([])

  async function initializePsionicCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchPsionics(filters: PsionicFilters = {}): Promise<PsionicSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend PsionicFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
        psionic_type: filters.psionic_type || null,
        psionic_order: filters.psionic_order || null,
      }

      const response = await invoke<{ success: boolean; data?: PsionicSummary[]; error?: string }>('search_psionics', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        psionics.value = response.data
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

  async function getPsionicDetails(name: string, source: string): Promise<Psionic | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Psionic; error?: string }>('get_psionic_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get psionic details:', e)
      return null
    }
  }

  return {
    isPsionicsInitialized,
    isLoading,
    error,
    psionics,
    initializePsionicCatalog,
    searchPsionics,
    getPsionicDetails,
  }
}
