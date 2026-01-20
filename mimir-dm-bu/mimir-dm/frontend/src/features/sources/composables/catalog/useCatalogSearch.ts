import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface CatalogConfig<TSummary, TDetails, TFilters extends object> {
  /** Name of the catalog (for error messages) */
  name: string

  /** Tauri command to initialize the catalog (optional) */
  initializeCommand?: string

  /** Tauri command to search the catalog */
  searchCommand: string

  /** Tauri command to get item details */
  detailsCommand: string

  /** Transform filters before sending to backend (optional) */
  transformFilters?: (filters: TFilters) => Record<string, unknown>
}

export interface CatalogSearchResult<TSummary, TDetails, TFilters extends object> {
  isInitialized: Ref<boolean>
  isLoading: Ref<boolean>
  error: Ref<string | null>
  results: Ref<TSummary[]>
  initialize: () => Promise<void>
  search: (filters: TFilters) => Promise<TSummary[]>
  getDetails: (name: string, source: string) => Promise<TDetails | null>
}

/**
 * Generic composable for catalog search functionality.
 * Handles initialization, searching, and fetching details for any catalog type.
 */
export function useCatalogSearch<
  TSummary,
  TDetails,
  TFilters extends object = Record<string, unknown>
>(
  config: CatalogConfig<TSummary, TDetails, TFilters>
): CatalogSearchResult<TSummary, TDetails, TFilters> {
  const isInitialized = ref(false)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const results = ref<TSummary[]>([]) as Ref<TSummary[]>

  async function initialize(): Promise<void> {
    if (isInitialized.value) return
    if (!config.initializeCommand) {
      isInitialized.value = true
      return
    }

    try {
      isLoading.value = true
      error.value = null
      await invoke(config.initializeCommand)
      isInitialized.value = true
    } catch (e) {
      error.value = `Failed to initialize ${config.name} catalog: ${e}`
    } finally {
      isLoading.value = false
    }
  }

  async function search(filters: TFilters): Promise<TSummary[]> {
    if (!isInitialized.value) {
      await initialize()
    }

    try {
      isLoading.value = true
      error.value = null

      const transformedFilters = config.transformFilters
        ? config.transformFilters(filters)
        : transformDefaultFilters(filters as Record<string, unknown>)

      const searchResults = await invoke<TSummary[]>(
        config.searchCommand,
        transformedFilters
      )

      results.value = searchResults
      return searchResults
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getDetails(name: string, source: string): Promise<TDetails | null> {
    try {
      const details = await invoke<TDetails>(config.detailsCommand, { name, source })
      return details
    } catch (e) {
      return null
    }
  }

  return {
    isInitialized,
    isLoading,
    error,
    results,
    initialize,
    search,
    getDetails,
  }
}

/**
 * Default filter transformation:
 * - Converts empty arrays to null
 * - Converts empty strings to null
 * - Passes undefined values as null
 */
function transformDefaultFilters(filters: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = {}

  for (const [key, value] of Object.entries(filters)) {
    if (Array.isArray(value)) {
      result[key] = value.length > 0 ? value : null
    } else if (value === '' || value === undefined) {
      result[key] = null
    } else {
      result[key] = value
    }
  }

  return result
}
