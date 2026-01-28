import { ref, type Ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useCampaignStore } from '@/stores/campaigns'

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

  // Get effective sources: explicit filter sources, or campaign sources if configured
  // Note: Store access is lazy to avoid Pinia initialization issues
  const getEffectiveSources = (filterSources?: string[]): string[] | null => {
    // If explicit sources provided in filter, use those
    if (filterSources && filterSources.length > 0) {
      return filterSources
    }
    // If campaign has sources configured, use those
    try {
      const campaignStore = useCampaignStore()
      if (campaignStore.currentCampaignSources.length > 0) {
        return campaignStore.currentCampaignSources
      }
    } catch {
      // Store not available yet, use no filter
    }
    // No filtering - return null to show all
    return null
  }

  async function initializeMonsterCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchMonsters(filters: MonsterFilters): Promise<MonsterSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend MonsterFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: getEffectiveSources(filters.sources),
        creature_type: filters.types?.length ? filters.types[0] : null,  // Backend expects single type
        size: filters.sizes?.length ? filters.sizes[0] : null,  // Backend expects single size
        cr: null,  // Using cr directly instead of min/max for now
      }

      const response = await invoke<{ success: boolean; data?: MonsterSummary[]; error?: string }>('search_monsters', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        monsters.value = response.data
        return response.data
      } else {
        error.value = response.error || 'Search failed'
        return []
      }
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getMonsterDetails(name: string, source: string): Promise<Monster | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Monster; error?: string }>('get_monster_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get monster details:', e)
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
