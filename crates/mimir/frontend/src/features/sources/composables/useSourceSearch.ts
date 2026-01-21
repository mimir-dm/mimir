import { ref, computed } from 'vue'
import { useSpells, useItems, useMonsters } from './catalog'
import type {
  SpellSummary,
  ItemSummary,
  MonsterSummary
} from './catalog'

/**
 * Composable for searching across source content
 * This is a transitional implementation that wraps the existing catalog
 * Eventually this will directly interface with a unified source API
 */
export function useSourceSearch() {
  // Search state
  const searchQuery = ref('')
  const selectedContentType = ref<'all' | 'spells' | 'items' | 'monsters' | 'magic-items'>('all')
  const hasSearched = ref(false)

  // Results
  const spellResults = ref<SpellSummary[]>([])
  const itemResults = ref<ItemSummary[]>([])
  const monsterResults = ref<MonsterSummary[]>([])

  // Use individual composables
  const spells = useSpells()
  const items = useItems()
  const monsters = useMonsters()
  
  // Initialize catalogs based on content type
  async function initializeForContentType() {
    switch (selectedContentType.value) {
      case 'spells':
        await spells.initializeCatalog()
        break
      case 'items':
      case 'magic-items':
        await items.initializeItemCatalog()
        break
      case 'monsters':
        await monsters.initializeMonsterCatalog()
        break
      case 'all':
        // Initialize all catalogs for combined search
        await Promise.all([
          spells.initializeCatalog(),
          items.initializeItemCatalog(),
          monsters.initializeMonsterCatalog()
        ])
        break
    }
  }
  
  // Unified search function
  async function search(sources: string[] = []) {
    hasSearched.value = true
    
    // Initialize required catalogs
    await initializeForContentType()
    
    // Build filters based on selected content type
    const query = searchQuery.value || undefined
    
    // Clear previous results
    spellResults.value = []
    itemResults.value = []
    monsterResults.value = []
    
    // Perform searches based on content type
    switch (selectedContentType.value) {
      case 'spells':
        spellResults.value = await spells.searchSpells({
          query,
          sources: sources.length > 0 ? sources : undefined
        })
        break

      case 'items':
        itemResults.value = await items.searchItems({
          query,
          sources: sources.length > 0 ? sources : undefined,
          rarities: ['none', 'common'] // Non-magic items
        })
        break

      case 'magic-items':
        itemResults.value = await items.searchItems({
          query,
          sources: sources.length > 0 ? sources : undefined,
          rarities: ['uncommon', 'rare', 'very rare', 'legendary', 'artifact']
        })
        break

      case 'monsters':
        monsterResults.value = await monsters.searchMonsters({
          query,
          sources: sources.length > 0 ? sources : undefined
        })
        break

      case 'all':
        // Search all content types in parallel
        const [spellsResults, itemsResults, monstersResults] = await Promise.all([
          spells.searchSpells({ query, sources: sources.length > 0 ? sources : undefined }),
          items.searchItems({ query, sources: sources.length > 0 ? sources : undefined }),
          monsters.searchMonsters({ query, sources: sources.length > 0 ? sources : undefined })
        ])
        spellResults.value = spellsResults
        itemResults.value = itemsResults
        monsterResults.value = monstersResults
        break
    }
  }
  
  // Clear search
  function clearSearch() {
    searchQuery.value = ''
    hasSearched.value = false
    spellResults.value = []
    itemResults.value = []
    monsterResults.value = []
  }
  
  // Get specific content details (for modal display)
  async function getContentDetails(type: string, name: string, source: string) {
    switch (type) {
      case 'spell':
        return await spells.getSpellDetails(name, source)
      case 'item':
        return await items.getItemDetails(name, source)
      case 'monster':
        return await monsters.getMonsterDetails(name, source)
      default:
        return null
    }
  }
  
  return {
    // State
    searchQuery,
    selectedContentType,
    hasSearched,
    
    // Results
    spellResults: computed(() => spellResults.value),
    itemResults: computed(() => itemResults.value),
    monsterResults: computed(() => monsterResults.value),
    
    // Actions
    search,
    clearSearch,
    getContentDetails
  }
}