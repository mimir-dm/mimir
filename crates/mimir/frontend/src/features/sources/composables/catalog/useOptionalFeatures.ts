import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface OptionalFeatureSummary {
  name: string
  source: string
  feature_type?: string
}

export interface OptionalFeatureFilters {
  query?: string
  sources?: string[]
  feature_type?: string
}

export interface OptionalFeature {
  name: string
  source: string
  page?: number
  feature_type?: string
  entries?: any[]
  prerequisite?: any[]
  srd?: boolean
  basicRules?: boolean
  hasFluff?: boolean
  hasFluffImages?: boolean
}

export function useOptionalFeatures() {
  const isOptionalFeaturesInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const optionalFeatures = ref<OptionalFeatureSummary[]>([])

  async function initializeOptionalFeatureCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchOptionalFeatures(filters: OptionalFeatureFilters = {}): Promise<OptionalFeatureSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend OptionalFeatureFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
        feature_type: filters.feature_type || null,
      }

      const response = await invoke<{ success: boolean; data?: OptionalFeatureSummary[]; error?: string }>('search_optional_features', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        optionalFeatures.value = response.data
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

  async function getOptionalFeatureDetails(name: string, source: string): Promise<OptionalFeature | null> {
    try {
      const response = await invoke<{ success: boolean; data?: OptionalFeature; error?: string }>('get_optional_feature_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get optional feature details:', e)
      return null
    }
  }

  return {
    isOptionalFeaturesInitialized,
    isLoading,
    error,
    optionalFeatures,
    initializeOptionalFeatureCatalog,
    searchOptionalFeatures,
    getOptionalFeatureDetails,
  }
}
