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
    try {
      await invoke('init_language_catalog')
    } catch (e) {
      // Initialization failed silently
    }
  }

  async function searchLanguages(filters: LanguageFilters = {}): Promise<LanguageSummary[]> {
    try {
      const results = await invoke<LanguageSummary[]>('search_languages', {
        query: filters.query || null,
        sources: filters.sources || null,
        language_types: filters.types || null,
        scripts: filters.scripts || null
      })
      return results || []
    } catch (e) {
      return []
    }
  }

  async function getLanguageDetails(name: string, source: string): Promise<Language | null> {
    try {
      const details = await invoke<Language>('get_language_details', { name, source })
      return details
    } catch (e) {
      return null
    }
  }

  async function getLanguageTypes(): Promise<string[]> {
    try {
      const types = await invoke<string[]>('get_language_types')
      return types || []
    } catch (e) {
      return []
    }
  }

  async function getLanguageScripts(): Promise<string[]> {
    try {
      const scripts = await invoke<string[]>('get_language_scripts')
      return scripts || []
    } catch (e) {
      return []
    }
  }

  return {
    initializeLanguageCatalog,
    searchLanguages,
    getLanguageDetails,
    getLanguageTypes,
    getLanguageScripts,
  }
}
