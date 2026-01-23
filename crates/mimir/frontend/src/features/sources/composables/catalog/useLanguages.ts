import { invoke } from '@tauri-apps/api/core'

export interface LanguageSummary {
  name: string
  source: string
  language_type: string
  script: string
  typical_speakers: string
}

export interface LanguageFilters {
  query?: string
  sources?: string[]
  types?: string[]
  scripts?: string[]
}

export interface Language {
  name: string
  source: string
  page?: number
  language_type?: string
  script?: string
  typical_speakers?: string[]
  entries?: any[]
  basic_rules?: boolean
  srd?: boolean
  has_fluff?: boolean
  has_fluff_images?: boolean
  fonts?: string[]
  dialects?: string[]
}

export function useLanguages() {
  async function initializeLanguageCatalog() {
    // No initialization needed - database-backed
  }

  async function searchLanguages(filters: LanguageFilters = {}): Promise<LanguageSummary[]> {
    try {
      const response = await invoke<{ success: boolean; data?: LanguageSummary[]; error?: string }>('search_languages', {
        filter: {
          name_contains: filters.query || null,
          sources: filters.sources ?? null,
          language_type: filters.types?.length ? filters.types[0] : null,  // Backend expects single type
        },
        limit: 10000,
        offset: 0
      })
      if (response.success && response.data) {
        return response.data
      }
      return []
    } catch (e) {
      console.error('Failed to search languages:', e)
      return []
    }
  }

  async function getLanguageDetails(name: string, source: string): Promise<Language | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Language; error?: string }>('get_language_by_name', { name, source })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get language details:', e)
      return null
    }
  }

  async function getLanguageTypes(): Promise<string[]> {
    try {
      const response = await invoke<{ success: boolean; data?: string[]; error?: string }>('list_language_sources')
      if (response.success && response.data) {
        return response.data
      }
      return []
    } catch (e) {
      return []
    }
  }

  async function getLanguageScripts(): Promise<string[]> {
    // Scripts not currently supported in backend
    return []
  }

  return {
    initializeLanguageCatalog,
    searchLanguages,
    getLanguageDetails,
    getLanguageTypes,
    getLanguageScripts,
  }
}
