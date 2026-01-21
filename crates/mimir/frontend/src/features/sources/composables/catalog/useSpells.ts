import { useCatalogSearch } from './useCatalogSearch'

export interface SpellSummary {
  name: string
  level: number
  school: string
  source: string
  concentration: boolean
  ritual: boolean
  casting_time: string
  range: string
  components: string
  classes: string[]
  description: string
}

export interface SpellFilters {
  query?: string
  sources?: string[]
  levels?: number[]
  schools?: string[]
  classes?: string[]
  ritual?: boolean
  concentration?: boolean
}

export interface Spell {
  name: string
  source: string
  level: number
  school: string
  time: unknown[]
  range: unknown
  components: unknown
  duration: unknown[]
  entries: string[]
  scalingLevelDice?: unknown
  damageInflict?: string[]
  conditionInflict?: string[]
  savingThrow?: string[]
  miscTags?: string[]
  areaTags?: string[]
  classes?: unknown
}

export function useSpells() {
  const catalog = useCatalogSearch<SpellSummary, Spell, SpellFilters>({
    name: 'spell',
    initializeCommand: 'initialize_spell_catalog',
    searchCommand: 'search_spells',
    detailsCommand: 'get_spell_details',
  })

  return {
    isInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    spells: catalog.results,
    initializeCatalog: catalog.initialize,
    searchSpells: catalog.search,
    getSpellDetails: catalog.getDetails,
  }
}
