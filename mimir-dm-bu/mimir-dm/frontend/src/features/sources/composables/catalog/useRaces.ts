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
  const catalog = useCatalogSearch<RaceSummary, string | null, RaceFilters>({
    name: 'race',
    initializeCommand: 'init_race_catalog',
    searchCommand: 'search_races',
    detailsCommand: 'get_race_details',
    transformFilters: (filters) => ({
      search: filters.query || null,
      sources: filters.sources && filters.sources.length > 0 ? filters.sources : null,
      sizes: filters.sizes && filters.sizes.length > 0 ? filters.sizes : null,
      has_darkvision: filters.has_darkvision !== undefined ? filters.has_darkvision : null,
      has_flight: filters.has_flight !== undefined ? filters.has_flight : null,
    }),
  })

  // Custom getDetails that parses JSON
  async function getRaceDetails(name: string, source: string): Promise<RaceWithDetails | null> {
    try {
      const jsonString = await invoke<string | null>('get_race_details', { name, source })
      if (!jsonString) {
        return null
      }

      const raceData = JSON.parse(jsonString)

      return {
        race: raceData.name ? raceData : null,
        subrace: raceData.race_name ? raceData : null,
        relatedSubraces: [],
        fluff: null
      } as RaceWithDetails
    } catch (e) {
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
