import {
  useSpells,
  useItems,
  useMonsters,
  useClasses,
  useRaces,
  useFeats,
  useBackgrounds,
  useActions,
  useConditions,
  useOptionalFeatures,
  useDeities,
  useObjects,
  useTraps,
  useLanguages,
  useRewards,
  useTables,
  useVariantRules,
  useVehicles,
  useCults,
  usePsionics,
  type SpellSummary,
  type ItemSummary,
  type MonsterSummary,
  type ClassSummary,
  type FeatSummary,
  type RaceSummary,
  type BackgroundSummary,
  type ActionSummary,
  type ConditionSummary,
  type ConditionWithDetails,
  type OptionalFeatureSummary,
  type DeitySummary,
  type ObjectSummary,
  type TrapSummary,
  type TrapOrHazard,
  type LanguageSummary,
  type Language,
  type RewardSummary,
  type Reward,
  type TableSummary,
  type Table
} from '../composables/catalog'

export type { BackgroundSummary, ActionSummary, ConditionSummary, ConditionWithDetails, OptionalFeatureSummary }

export interface SearchFilters {
  spells: {
    school: string
    level: string
    class: string
    ritual: boolean
    concentration: boolean
  }
  equipment: {
    type: string
    rarity: string
  }
  monsters: {
    sizes: string[]
    types: string[]
    minCr?: number
    maxCr?: number
  }
  magicItems: {
    rarity: string
  }
}

export interface SearchParams {
  query: string
  sources: string[]
  category: string
  filters: Partial<SearchFilters>
}

export interface DetailFetchParams {
  name: string
  source: string
  type: 'spell' | 'item' | 'monster' | 'class' | 'subclass' | 'feat' | 'race' | 'background' | 'action' | 'condition' | 'option' | 'deity' | 'object' | 'trap' | 'language' | 'reward' | 'table' | 'variantrule' | 'vehicle' | 'cult' | 'boon' | 'psionic'
  subclassName?: string
}

class SearchServiceClass {
  private spells = useSpells()
  private items = useItems()
  private monsters = useMonsters()
  private classes = useClasses()
  private races = useRaces()
  private feats = useFeats()
  private backgrounds = useBackgrounds()
  private actions = useActions()
  private conditions = useConditions()
  private optionalFeatures = useOptionalFeatures()
  private deities = useDeities()
  private objects = useObjects()
  private traps = useTraps()
  private languages = useLanguages()
  private rewards = useRewards()
  private tables = useTables()
  private variantRules = useVariantRules()
  private vehicles = useVehicles()
  private cults = useCults()
  private psionics = usePsionics()

  async initialize(category: string): Promise<void> {
    try {
      switch (category) {
        case 'Spells':
          await this.spells.initializeCatalog()
          break
        case 'Equipment':
        case 'Magic Items':
          await this.items.initializeItemCatalog()
          break
        case 'Monsters':
          await this.monsters.initializeMonsterCatalog()
          break
        case 'Classes':
          await this.classes.initializeClassCatalog()
          break
        case 'Races':
          console.log('Races now use database-backed service (no initialization needed)')
          break
        case 'Backgrounds':
          await this.backgrounds.initializeBackgroundCatalog()
          break
        case 'Actions':
          await this.actions.initializeActionCatalog()
          break
        case 'Conditions':
          await this.conditions.initializeConditionCatalog()
          break
        case 'Options':
        case 'Other Options & Features':
          console.log('Optional features now use database-backed service (no initialization needed)')
          break
        case 'Deities':
          await this.deities.initializeDeityCatalog()
          break
        case 'Objects':
          console.log('Objects now use database-backed service (no initialization needed)')
          break
        case 'Traps & Hazards':
          await this.traps.initializeTrapCatalog()
          break
        case 'Languages':
          await this.languages.initializeLanguageCatalog()
          break
        case 'Rewards':
          await this.rewards.initializeRewardCatalog()
          break
        case 'Tables':
          await this.tables.initializeTableCatalog()
          break
        case 'Variant Rules':
          // No initialization needed - loaded from database
          break
        case 'Vehicles':
          await this.vehicles.initializeVehicleCatalog()
          break
        case 'Cults & Boons':
          await this.cults.initializeCultCatalog()
          break
        case 'Psionics':
          // No initialization needed - loaded from single file
          break
        case 'Feats':
          // No initialization needed - uses database-backed service
          break
      }
    } catch (error) {
      throw new Error(`Failed to initialize ${category} catalog: ${error}`)
    }
  }
  
  async search(params: Partial<SearchParams>): Promise<any[]> {
    const { query, sources, category, filters = {} } = params

    try {
      switch (category) {
        case 'Spells':
          return await this.searchSpells(query, sources, filters.spells)
        case 'Equipment':
          return await this.searchEquipment(query, sources, filters.equipment)
        case 'Magic Items':
          return await this.searchMagicItems(query, sources, filters.magicItems)
        case 'Monsters':
          return await this.searchMonsters(query, sources, filters.monsters)
        case 'Classes':
          return await this.searchClasses(query)
        case 'Races':
          return await this.searchRaces(query, sources)
        case 'Backgrounds':
          return await this.searchBackgrounds(query, sources)
        case 'Actions':
          return await this.searchActions(query, sources)
        case 'Conditions':
          return await this.searchConditions(query, sources)
        case 'Options':
        case 'Other Options & Features':
          return await this.searchOptionalFeatures(query, sources)
        case 'Deities':
          return await this.searchDeities(query, sources)
        case 'Objects':
          return await this.searchObjects(query, sources)
        case 'Traps & Hazards':
          return await this.searchTraps({ query, sources })
        case 'Languages':
          return await this.searchLanguages({ query, sources })
        case 'Rewards':
          return await this.searchRewards({ query, sources })
        case 'Tables':
          return await this.searchTables({ query, sources })
        case 'Variant Rules':
          return await this.searchVariantRules({ query, sources })
        case 'Vehicles':
          return await this.searchVehicles({ query, sources })
        case 'Cults & Boons':
          return await this.searchCults({ query, sources })
        case 'Psionics':
          return await this.searchPsionics({ query, sources })
        case 'Feats':
          return await this.searchFeats(query, sources)
        default:
          return []
      }
    } catch (error) {
      const searchContext = query ? `"${query}" in ${category}` : category || 'catalog'
      throw new Error(`Failed to search ${searchContext}: ${error}`)
    }
  }
  
  private async searchSpells(
    query?: string,
    sources?: string[],
    filters?: SearchFilters['spells']
  ): Promise<SpellSummary[]> {
    return await this.spells.searchSpells({
      query: query || undefined,
      sources,
      schools: filters?.school ? [filters.school] : undefined,
      levels: filters?.level ? [parseInt(filters.level)] : undefined,
      classes: filters?.class ? [filters.class] : undefined,
      ritual: filters?.ritual || undefined,
      concentration: filters?.concentration || undefined
    })
  }
  
  private async searchEquipment(
    query?: string,
    sources?: string[],
    filters?: SearchFilters['equipment']
  ): Promise<ItemSummary[]> {
    const { invoke } = await import('@tauri-apps/api/core')
    const results = await invoke<ItemSummary[]>('search_items', {
      name: query,
      sources: sources,
      item_types: filters?.type ? [filters.type] : undefined,
      rarities: undefined,
      min_value: undefined,
      max_value: undefined
    })
    
    // Filter out magical items (those with rarities other than 'none')
    return results.filter(item => 
      !item.rarity || item.rarity === 'none'
    )
  }
  
  private async searchMagicItems(
    query?: string,
    sources?: string[],
    filters?: SearchFilters['magicItems']
  ): Promise<ItemSummary[]> {
    const { invoke } = await import('@tauri-apps/api/core')
    const results = await invoke<ItemSummary[]>('search_items', {
      name: query,
      sources: sources,
      item_types: undefined,
      rarities: filters?.rarity ? [filters.rarity] : undefined,
      min_value: undefined,
      max_value: undefined
    })
    
    // Filter out non-magical items (those with 'none' rarity)
    return results.filter(item => 
      item.rarity && item.rarity !== 'none'
    )
  }
  
  private async searchMonsters(
    query?: string,
    sources?: string[],
    filters?: SearchFilters['monsters']
  ): Promise<MonsterSummary[]> {
    return await this.monsters.searchMonsters({
      query: query || undefined,
      sources,
      sizes: filters?.sizes?.length ? filters.sizes : undefined,
      types: filters?.types?.length ? filters.types : undefined,
      min_cr: filters?.minCr,
      max_cr: filters?.maxCr
    })
  }
  
  private async searchClasses(query?: string): Promise<ClassSummary[]> {
    return await this.classes.searchClasses({
      name: query || undefined
    })
  }
  
  private async searchFeats(query?: string, sources?: string[]): Promise<FeatSummary[]> {
    return await this.feats.searchFeats({
      query: query || undefined,
      sources: sources || undefined
    })
  }
  
  private async searchRaces(query?: string, sources?: string[]): Promise<RaceSummary[]> {
    console.log('SearchService.searchRaces called with:', { query, sources })
    const results = await this.races.searchRaces({
      query: query || undefined,
      sources: sources || undefined
    })
    console.log('SearchService.searchRaces results:', results)
    return results
  }
  
  private async searchBackgrounds(query?: string, sources?: string[]): Promise<BackgroundSummary[]> {
    const results = await this.backgrounds.searchBackgrounds({
      query: query || undefined,
      sources: sources || undefined
    })
    return results
  }
  
  private async searchActions(query?: string, sources?: string[]): Promise<ActionSummary[]> {
    const results = await this.actions.searchActions({
      query: query || undefined,
      sources: sources || undefined
    })
    return results
  }
  
  private async searchConditions(query?: string, sources?: string[]): Promise<ConditionSummary[]> {
    const results = await this.conditions.searchConditions({
      query: query || undefined,
      sources: sources || undefined
    })
    return results
  }
  
  private async searchOptionalFeatures(query?: string, sources?: string[]): Promise<OptionalFeatureSummary[]> {
    const { invoke } = await import('@tauri-apps/api/core')
    const results = await invoke<OptionalFeatureSummary[]>('search_optional_features', {
      name: query,
      sources: sources,
      feature_types: undefined,
      grants_spells: undefined
    })
    return results
  }
  
  private async searchDeities(query?: string, sources?: string[]): Promise<DeitySummary[]> {
    const results = await this.deities.searchDeities({
      query: query || undefined,
      sources: sources || undefined
    })
    return results
  }
  
  private async searchObjects(query?: string, sources?: string[]): Promise<ObjectSummary[]> {
    const results = await this.objects.searchObjects({
      query: query || undefined,
      sources: sources || undefined
    })
    return results
  }
  
  async searchTraps(params: {
    query?: string
    sources?: string[]
    categories?: string[]
    trap_types?: string[]
  }): Promise<TrapSummary[]> {
    const results = await this.traps.searchTraps(params)
    return results
  }

  async getTrapDetails(name: string, source: string): Promise<TrapOrHazard | null> {
    return await this.traps.getTrapDetails(name, source)
  }

  async getTrapTypes(): Promise<string[]> {
    return await this.traps.getTrapTypes()
  }

  async searchLanguages(params: {
    query?: string
    sources?: string[]
    types?: string[]
    scripts?: string[]
  }): Promise<LanguageSummary[]> {
    const results = await this.languages.searchLanguages(params)
    return results
  }

  async getLanguageDetails(name: string, source: string): Promise<Language | null> {
    return await this.languages.getLanguageDetails(name, source)
  }

  async getLanguageTypes(): Promise<string[]> {
    return await this.languages.getLanguageTypes()
  }

  async getLanguageScripts(): Promise<string[]> {
    return await this.languages.getLanguageScripts()
  }

  async searchRewards(params: {
    query?: string
    sources?: string[]
    reward_types?: string[]
    has_prerequisites?: boolean
  }): Promise<RewardSummary[]> {
    const results = await this.rewards.searchRewards(params)
    return results
  }

  async getRewardDetails(name: string, source: string): Promise<Reward | null> {
    return await this.rewards.getRewardDetails(name, source)
  }

  async getRewardTypes(): Promise<string[]> {
    return await this.rewards.getRewardTypes()
  }

  async getRewardSources(): Promise<string[]> {
    return await this.rewards.getRewardSources()
  }

  async searchTables(params: {
    query?: string
    sources?: string[]
    categories?: string[]
    min_rows?: number
    max_rows?: number
  }): Promise<TableSummary[]> {
    const results = await this.tables.searchTables(params)
    return results
  }

  async getTableDetails(name: string, source: string): Promise<Table | null> {
    return await this.tables.getTableDetails(name, source)
  }

  async getTableCategories(): Promise<string[]> {
    return await this.tables.getTableCategories()
  }

  async getTableSources(): Promise<string[]> {
    return await this.tables.getTableSources()
  }

  async searchVariantRules(params: {
    query?: string
    types?: string[]
    sources?: string[]
  }): Promise<any[]> {
    const results = await this.variantRules.searchVariantRules(params)
    return results
  }

  async getVariantRuleDetails(name: string, source: string): Promise<any> {
    return await this.variantRules.getVariantRuleDetails(name, source)
  }

  async getVariantRuleTypes(): Promise<string[]> {
    return await this.variantRules.getVariantRuleTypes()
  }

  async getVariantRuleSources(): Promise<string[]> {
    return await this.variantRules.getVariantRuleSources()
  }

  async searchVehicles(params: {
    query?: string
    types?: string[]
    sources?: string[]
    terrains?: string[]
    sizes?: string[]
  }): Promise<any[]> {
    const results = await this.vehicles.searchVehicles(params)
    return results
  }

  async getVehicleDetails(name: string, source: string): Promise<any> {
    return await this.vehicles.getVehicleDetails(name, source)
  }

  async getVehicleTypes(): Promise<string[]> {
    return await this.vehicles.getVehicleTypes()
  }

  async getVehicleTerrains(): Promise<string[]> {
    return await this.vehicles.getVehicleTerrains()
  }

  async getVehicleSources(): Promise<string[]> {
    return await this.vehicles.getVehicleSources()
  }

  async searchCults(params: {
    query?: string
    item_types?: string[]
    subtypes?: string[]
    sources?: string[]
  }): Promise<any[]> {
    const results = await this.cults.searchCults(params)
    return results
  }

  async getCultDetails(name: string, source: string): Promise<any> {
    return await this.cults.getCultDetails(name, source)
  }

  async getBoonDetails(name: string, source: string): Promise<any> {
    return await this.cults.getBoonDetails(name, source)
  }

  async getCultTypes(): Promise<string[]> {
    return await this.cults.getCultTypes()
  }

  async getCultSources(): Promise<string[]> {
    return await this.cults.getCultSources()
  }

  async searchPsionics(params: {
    query?: string
    psionic_types?: string[]
    orders?: string[]
    sources?: string[]
  }): Promise<any[]> {
    const results = await this.psionics.searchPsionics(params)
    return results
  }

  async getPsionicDetails(name: string, source: string): Promise<any> {
    return await this.psionics.getPsionicDetails(name, source)
  }

  async getPsionicOrders(): Promise<string[]> {
    return await this.psionics.getPsionicOrders()
  }

  async getPsionicSources(): Promise<string[]> {
    return await this.psionics.getPsionicSources()
  }
  
  async getDetails(params: DetailFetchParams): Promise<any> {
    const { name, source, type, subclassName } = params

    try {
      switch (type) {
        case 'spell':
          return await this.spells.getSpellDetails(name, source)
        case 'item':
          const { invoke } = await import('@tauri-apps/api/core')
          return await invoke('get_item_details', { itemName: name, itemSource: source })
        case 'monster':
          return await this.monsters.getMonsterDetails(name, source)
        case 'class':
          return await this.classes.getClassDetails(name, source)
        case 'subclass':
          if (!subclassName) {
            throw new Error('subclassName is required for subclass details')
          }
          return await this.classes.getSubclassDetails(subclassName, name, source)
        case 'feat':
          return await this.feats.getFeatDetails(name, source)
        case 'race':
          return await this.races.getRaceDetails(name, source)
        case 'background':
          return await this.backgrounds.getBackgroundDetails(name, source)
        case 'action':
          return await this.actions.getActionDetails(name, source)
        case 'condition':
          return await this.conditions.getConditionDetails(name, source)
        case 'option':
          return await this.optionalFeatures.getOptionalFeatureDetails(name, source)
        case 'deity':
          return await this.deities.getDeityDetails(name, source)
        case 'object':
          return await this.objects.getObjectDetails(name, source)
        case 'trap':
          return await this.traps.getTrapDetails(name, source)
        case 'language':
          return await this.languages.getLanguageDetails(name, source)
        case 'reward':
          return await this.rewards.getRewardDetails(name, source)
        case 'table':
          return await this.tables.getTableDetails(name, source)
        case 'variantrule':
          return await this.variantRules.getVariantRuleDetails(name, source)
        case 'vehicle':
          return await this.vehicles.getVehicleDetails(name, source)
        case 'cult':
          return await this.cults.getCultDetails(name, source)
        case 'boon':
          return await this.cults.getBoonDetails(name, source)
        case 'psionic':
          return await this.psionics.getPsionicDetails(name, source)
        default:
          return null
      }
    } catch (error) {
      throw new Error(`Failed to get ${type} details for "${name}" (${source}): ${error}`)
    }
  }

  mapBookIdsToSources(bookIds: string[]): string[] {
    return bookIds.map(id => {
      const parts = id.split('-')
      return parts[parts.length - 1].toUpperCase()
    })
  }

  getClassSources(): string[] {
    return this.classes.classSources.value
  }
}

export const SearchService = new SearchServiceClass()