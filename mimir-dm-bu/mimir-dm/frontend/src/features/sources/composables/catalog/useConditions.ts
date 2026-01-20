import { useCatalogSearch } from './useCatalogSearch'

export interface ConditionSummary {
  name: string
  source: string
  item_type: 'Condition' | 'Disease'
  description: string
}

export interface ConditionWithDetails {
  item: {
    type: 'Condition' | 'Disease'
    Condition?: any
    Disease?: any
  }
  fluff?: any
}

export interface ConditionFilters {
  query?: string
  sources?: string[]
  type_filter?: string
}

export function useConditions() {
  const catalog = useCatalogSearch<ConditionSummary, ConditionWithDetails, ConditionFilters>({
    name: 'condition',
    searchCommand: 'search_conditions',
    detailsCommand: 'get_condition',
    transformFilters: (filters) => ({
      query: filters.query,
      sources: filters.sources,
      typeFilter: filters.type_filter
    }),
  })

  return {
    isConditionsInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    conditions: catalog.results,
    initializeConditionCatalog: catalog.initialize,
    searchConditions: catalog.search,
    getConditionDetails: catalog.getDetails,
  }
}
