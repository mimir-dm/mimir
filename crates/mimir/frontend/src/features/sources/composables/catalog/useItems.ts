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

      const results = await invoke<ItemSummary[]>('search_items', {
        name: filters.query || null,
        item_types: filters.types?.length ? filters.types : null,
        rarities: filters.rarities?.length ? filters.rarities : null,
        sources: filters.sources?.length ? filters.sources : null,
        min_value: filters.min_value ?? null,
        max_value: filters.max_value ?? null,
      })

      items.value = results
      return results
    } catch (e) {
      error.value = `Search failed: ${e}`
      return []
    } finally {
      isLoading.value = false
    }
  }

  async function getItemDetails(name: string, source: string): Promise<Item | null> {
    try {
      const item = await invoke<Item>('get_item_details', {
        item_name: name,
        item_source: source
      })
      return item
    } catch (e) {
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
