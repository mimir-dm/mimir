import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useCatalogSearch } from './useCatalogSearch'

export interface RaceSummary {
  name: string
  source: string
  size: string
  speed: number
  abilityBonuses: string
  traitsCount: number
  isSubrace: boolean
  parentRace?: string
}

export interface RaceFilters {
  query?: string
  sources?: string[]
  sizes?: string[]
  has_darkvision?: boolean
  has_flight?: boolean
}

export interface Race {
  name: string
  source: string
  page?: number
  size?: string[]
  speed?: any
  ability?: any[]
  age?: any
  darkvision?: number
  traitTags?: string[]
  languageProficiencies?: any[]
  skillProficiencies?: any[]
  weaponProficiencies?: any[]
  armorProficiencies?: any[]
  toolProficiencies?: any[]
  resist?: string[]
  immune?: string[]
  vulnerable?: string[]
  conditionImmune?: string[]
  entries: any[]
  soundClip?: any
  lineage?: string
  raceName?: string
  raceSource?: string
}

export interface Subrace {
  name: string
  source: string
  page?: number
  raceName: string
  raceSource: string
  ability?: any[]
  speed?: any
  darkvision?: number
  resist?: string[]
  traitTags?: string[]
  languageProficiencies?: any[]
  skillProficiencies?: any[]
  weaponProficiencies?: any[]
  armorProficiencies?: any[]
  toolProficiencies?: any[]
  entries: any[]
  overwrite?: any
}

export interface RaceWithDetails {
  race?: Race
  subrace?: Subrace
  relatedSubraces: Subrace[]
  fluff?: any
}

export function useRaces() {
  const catalog = useCatalogSearch<RaceSummary, Race, RaceFilters>({
    name: 'race',
    // No initialization needed - database-backed
    searchCommand: 'search_races',
    detailsCommand: 'get_race_by_name',
    transformFilters: (filters) => ({
      name_contains: filters.query || null,
      sources: filters.sources ?? null,
      sizes: filters.sizes && filters.sizes.length > 0 ? filters.sizes : null,
    }),
  })

  // Custom getDetails that wraps the result
  async function getRaceDetails(name: string, source: string): Promise<RaceWithDetails | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Race; error?: string }>('get_race_by_name', { name, source })
      if (!response.success || !response.data) {
        return null
      }

      const raceData = response.data

      return {
        race: raceData,
        subrace: undefined,
        relatedSubraces: [],
        fluff: null
      } as RaceWithDetails
    } catch (e) {
      console.error('Failed to get race details:', e)
      return null
    }
  }

  return {
    isRacesInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    races: catalog.results,
    initializeRaceCatalog: catalog.initialize,
    searchRaces: catalog.search,
    getRaceDetails,
  }
}
