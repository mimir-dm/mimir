import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface DeitySummary {
  name: string
  source: string
  title?: string
  pantheon?: string
  alignment?: string
  domains?: string[]
  symbol?: string
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
  pantheon?: string
}

export function useDeities() {
  const isDeitiesInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const deities = ref<DeitySummary[]>([])

  async function initializeDeityCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchDeities(filters: DeityFilters): Promise<DeitySummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend DeityFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
        pantheon: filters.pantheon || null,
      }

      const response = await invoke<{ success: boolean; data?: DeitySummary[]; error?: string }>('search_deities', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        deities.value = response.data
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

  async function getDeityDetails(name: string, source: string): Promise<Deity | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Deity; error?: string }>('get_deity_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get deity details:', e)
      return null
    }
  }

  return {
    isDeitiesInitialized,
    isLoading,
    error,
    deities,
    initializeDeityCatalog,
    searchDeities,
    getDeityDetails,
  }
}
