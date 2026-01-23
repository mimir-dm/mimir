import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface VariantRule {
  name: string
  source: string
  rule_type?: string
  page?: number
  entries?: any[]
}

export interface VariantRuleSummary {
  name: string
  source: string
  rule_type?: string
}

export interface VariantRuleFilters {
  query?: string
  sources?: string[]
  rule_type?: string
}

export function useVariantRules() {
  const isVariantRulesInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const variantRules = ref<VariantRuleSummary[]>([])

  async function initializeVariantRuleCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchVariantRules(filters: VariantRuleFilters = {}): Promise<VariantRuleSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend VariantRuleFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
        rule_type: filters.rule_type || null,
      }

      const response = await invoke<{ success: boolean; data?: VariantRuleSummary[]; error?: string }>('search_variant_rules', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        variantRules.value = response.data
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

  async function getVariantRuleDetails(name: string, source: string): Promise<VariantRule | null> {
    try {
      const response = await invoke<{ success: boolean; data?: VariantRule; error?: string }>('get_variant_rule_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get variant rule details:', e)
      return null
    }
  }

  return {
    isVariantRulesInitialized,
    isLoading,
    error,
    variantRules,
    initializeVariantRuleCatalog,
    searchVariantRules,
    getVariantRuleDetails,
  }
}
