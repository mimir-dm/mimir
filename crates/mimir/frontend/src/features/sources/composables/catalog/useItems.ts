import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ItemSummary {
  name: string
  itemType: string
  typeName: string
  source: string
  rarity: string
  value?: number
  weight?: number
  ac?: number
  damage?: string
  reqAttune?: string
  description: string
}

export interface ItemFilters {
  query?: string
  sources?: string[]
  types?: string[]
  rarities?: string[]
  min_value?: number
  max_value?: number
}

export interface Item {
  name: string
  source: string
  type: string
  rarity: string
  weight?: number
  value?: number
  entries?: string[]
}

export function useItems() {
  const isItemsInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const items = ref<ItemSummary[]>([])

  async function initializeItemCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchItems(filters: ItemFilters): Promise<ItemSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      const response = await invoke<{ success: boolean; data?: ItemSummary[]; error?: string }>('search_items', {
        filter: {
          name_contains: filters.query || null,
          item_type: filters.types?.length ? filters.types[0] : null,  // Backend expects single type
          rarity: filters.rarities?.length ? filters.rarities[0] : null,  // Backend expects single rarity
          sources: filters.sources ?? null,
        },
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        items.value = response.data
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

  async function getItemDetails(name: string, source: string): Promise<Item | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Item; error?: string }>('get_item_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get item details:', e)
      return null
    }
  }

  return {
    isItemsInitialized,
    isLoading,
    error,
    items,
    initializeItemCatalog,
    searchItems,
    getItemDetails,
  }
}
