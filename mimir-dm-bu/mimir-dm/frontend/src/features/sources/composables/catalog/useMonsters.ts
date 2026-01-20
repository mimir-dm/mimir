import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface MonsterSummary {
  name: string
  size: string
  type: string
  alignment: string
  cr: string
  hp: string
  ac: string
  speed: string
  source: string
  str: number
  dex: number
  con: number
  int: number
  wis: number
  cha: number
  senses?: string
  languages?: string
  description?: string
  creature_type?: string
  environment?: string[]
}

export interface MonsterFilters {
  query?: string
  sources?: string[]
  types?: string[]
  sizes?: string[]
  min_cr?: number
  max_cr?: number
}

export interface Monster {
  name: string
  source: string
  size: string[]
  type: unknown
  alignment?: unknown[]
  ac: unknown[]
  hp: unknown
  speed: unknown
  str: number
  dex: number
  con: number
  int: number
  wis: number
  cha: number
  save?: unknown
  skill?: unknown
  senses?: string[]
  languages?: string[]
  cr: string
  trait?: unknown[]
  action?: unknown[]
  legendary?: unknown[]
  immune?: unknown[]
  resist?: unknown[]
  vulnerable?: unknown[]
  conditionImmune?: string[]
  spellcasting?: unknown[]
  entries?: string[]
  fluffEntries?: unknown[]
  fluffImages?: unknown[]
  fluff_images?: unknown[]
}

export function useMonsters() {
  const isMonstersInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const monsters = ref<MonsterSummary[]>([])

  async function initializeMonsterCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchMonsters(filters: MonsterFilters): Promise<MonsterSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend MonsterFilters format
      const backendFilters = {
        name: filters.query || null,
        sources: filters.sources?.length ? filters.sources : null,
        creature_types: filters.types?.length ? filters.types : null,
        sizes: filters.sizes?.length ? filters.sizes : null,
        min_cr: filters.min_cr ?? null,
        max_cr: filters.max_cr ?? null,
        alignments: null,
        min_hp: null,
        max_hp: null,
        environment: null,
      }

      const results = await invoke<MonsterSummary[]>('search_monsters', {
        filters: backendFilters
      })

      monsters.value = results
      return results
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getMonsterDetails(name: string, source: string): Promise<Monster | null> {
    try {
      const monster = await invoke<Monster>('get_monster_details', {
        monsterName: name,
        monsterSource: source
      })
      return monster
    } catch (e) {
      return null
    }
  }

  return {
    isMonstersInitialized,
    isLoading,
    error,
    monsters,
    initializeMonsterCatalog,
    searchMonsters,
    getMonsterDetails,
  }
}
