import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useCampaignStore } from '@/stores/campaigns'

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

      // Backend expects { filter, limit, offset } with ApiResponse wrapper
      // Fetch all results - pagination is handled in the UI layer
      const response = await invoke<{ success: boolean; data?: TSummary[]; error?: string }>(
        config.searchCommand,
        {
          filter: transformedFilters,
          limit: 10000,
          offset: 0
        }
      )

      if (response.success && response.data) {
        results.value = response.data
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

  async function getDetails(name: string, source: string): Promise<TDetails | null> {
    try {
      const response = await invoke<{ success: boolean; data?: TDetails; error?: string }>(
        config.detailsCommand,
        { name, source }
      )
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error(`Failed to get ${config.name} details:`, e)
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
 * Get effective sources for filtering:
 * - If explicit sources provided, use those
 * - If campaign has sources configured, use those
 * - Otherwise return null (no filtering)
 */
function getEffectiveSources(filterSources: unknown): string[] | null {
  // If explicit sources provided in filter, use those
  if (Array.isArray(filterSources) && filterSources.length > 0) {
    return filterSources as string[]
  }

  // If campaign has sources configured, use those
  // Note: Store access is lazy to avoid Pinia initialization issues
  try {
    const campaignStore = useCampaignStore()
    if (campaignStore.currentCampaignSources.length > 0) {
      return campaignStore.currentCampaignSources
    }
  } catch {
    // Store not available yet, use no filter
  }

  // No filtering - return null to show all
  return null
}

/**
 * Default filter transformation:
 * - Applies campaign source filtering when no explicit sources provided
 * - Converts other empty arrays to null
 * - Converts empty strings to null
 * - Passes undefined values as null
 * - Renames 'query' and 'name' to 'name_contains' for backend compatibility
 */
function transformDefaultFilters(filters: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = {}

  for (const [key, value] of Object.entries(filters)) {
    // Rename 'query' and 'name' to 'name_contains' for backend compatibility
    const outputKey = (key === 'query' || key === 'name') ? 'name_contains' : key

    if (key === 'sources') {
      // Apply campaign source filtering
      result[outputKey] = getEffectiveSources(value)
    } else if (Array.isArray(value)) {
      // For other arrays, convert empty to null (no filter)
      result[outputKey] = value.length > 0 ? value : null
    } else if (value === '' || value === undefined) {
      result[outputKey] = null
    } else {
      result[outputKey] = value
    }
  }

  // If sources wasn't in filters at all, still apply campaign filtering
  if (!('sources' in filters)) {
    result['sources'] = getEffectiveSources(undefined)
  }

  return result
}
