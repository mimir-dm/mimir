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
    initializeCommand: 'init_background_catalog',
    searchCommand: 'search_backgrounds',
    detailsCommand: 'get_background_details',
    transformFilters: (filters) => ({
      query: filters.query || null,
      sources: filters.sources && filters.sources.length > 0 ? filters.sources : null,
      hasTools: filters.has_tools !== undefined ? filters.has_tools : null
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
