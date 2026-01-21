import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface OptionalFeatureSummary {
  name: string
  source: string
  feature_types: string[]
  feature_type_full: string
  prerequisite_text: string
  grants_spells: boolean
}

export interface OptionalFeatureFilters {
  query?: string
  sources?: string[]
  feature_types?: string[]
}

export function useOptionalFeatures() {
  const isOptionalFeaturesInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const optionalFeatures = ref<OptionalFeatureSummary[]>([])

  async function initializeOptionalFeatureCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchOptionalFeatures(filters: OptionalFeatureFilters): Promise<OptionalFeatureSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      const results = await invoke<OptionalFeatureSummary[]>('search_optional_features', {
        name: filters.query || null,
        feature_types: filters.feature_types?.length ? filters.feature_types : null,
        sources: filters.sources?.length ? filters.sources : null,
        grants_spells: null,
      })

      optionalFeatures.value = results
      return results
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getOptionalFeatureDetails(name: string, source: string): Promise<unknown | null> {
    try {
      const feature = await invoke<unknown>('get_optional_feature_details', { name, source })
      return feature
    } catch (e) {
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
