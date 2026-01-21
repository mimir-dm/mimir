import { useCatalogSearch } from './useCatalogSearch'

export interface ActionSummary {
  name: string
  source: string
  time: string
  description: string
  see_also: string[]
}

export interface ActionFilters {
  query?: string
  sources?: string[]
  time_filter?: string
}

export function useActions() {
  const catalog = useCatalogSearch<ActionSummary, any, ActionFilters>({
    name: 'action',
    searchCommand: 'search_actions',
    detailsCommand: 'get_action',
    transformFilters: (filters) => ({
      query: filters.query || null,
      sources: filters.sources && filters.sources.length > 0 ? filters.sources : null,
      timeFilter: filters.time_filter || null
    }),
  })

  return {
    isActionsInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    actions: catalog.results,
    initializeActionCatalog: catalog.initialize,
    searchActions: catalog.search,
    getActionDetails: catalog.getDetails,
  }
}
