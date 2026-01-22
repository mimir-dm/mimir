import { useCatalogSearch } from './useCatalogSearch'

export interface BackgroundSummary {
  name: string
  source: string
  skills: string
  languages: string
  tools: string
  feature: string
}

export interface BackgroundFilters {
  query?: string
  sources?: string[]
  has_tools?: boolean
}

export function useBackgrounds() {
  const catalog = useCatalogSearch<BackgroundSummary, any, BackgroundFilters>({
    name: 'background',
    // No initialization needed - database-backed
    searchCommand: 'search_backgrounds',
    detailsCommand: 'get_background_by_name',
    transformFilters: (filters) => ({
      name: filters.query || null,
      sources: filters.sources && filters.sources.length > 0 ? filters.sources : null,
    }),
  })

  return {
    isBackgroundsInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    backgrounds: catalog.results,
    initializeBackgroundCatalog: catalog.initialize,
    searchBackgrounds: catalog.search,
    getBackgroundDetails: catalog.getDetails,
  }
}
