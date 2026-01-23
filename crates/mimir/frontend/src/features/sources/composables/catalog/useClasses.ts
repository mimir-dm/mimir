import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useCatalogSearch } from './useCatalogSearch'

export interface ClassSummary {
  name: string
  source: string
  page?: number
  hitDice: string
  proficiency: string
  primaryAbility: string
  spellcastingAbility?: string
  tableGroups?: any[]
  subclassTitle?: string
  description: string
  subclassName?: string
  rowType: string
}

export interface ClassFilters {
  name?: string
  sources?: string[]
  has_spellcasting?: boolean
  primary_abilities?: string[]
}

export interface Subclass {
  name: string
  source: string
  className: string
  classSource: string
  shortName?: string
  page?: number
  spellcastingAbility?: string
  casterProgression?: string
  subclassFeatures?: any
  subclassTableGroups?: any[]
  fluff?: SubclassFluff
  introDescription?: string
}

export interface ClassFeature {
  name: string
  source: string
  class_name: string
  class_source: string
  level: number
  page?: number
  entries?: any[]
}

export interface SubclassFeature {
  name: string
  source: string
  class_name: string
  class_source: string
  subclass_short_name?: string
  subclass_source: string
  level: number
  page?: number
  entries?: any[]
}

export interface ClassFluff {
  name: string
  source: string
  entries: any[]
  images?: any[]
}

export interface SubclassFluff {
  name: string
  short_name?: string
  source: string
  class_name: string
  class_source: string
  entries: any[]
  images?: any[]
}

export interface ClassWithDetails {
  class: Class
  subclasses: Subclass[]
  features: ClassFeature[]
  subclass_features: SubclassFeature[]
  fluff?: ClassFluff
  subclass_fluff: SubclassFluff[]
}

export interface Class {
  name: string
  source: string
  page?: number
  hd?: any
  proficiency?: any
  startingProficiencies?: any
  spellcastingAbility?: string
  classTableGroups?: any[]
  subclassTitle?: string
  entries?: any[]
  classFeatures?: any[]
  multiclassing?: any
  casterProgression?: string
  fluff?: ClassFluff
}

export function useClasses() {
  const classSources = ref<string[]>([])

  const catalog = useCatalogSearch<ClassSummary, Class, ClassFilters>({
    name: 'class',
    searchCommand: 'search_classes',
    detailsCommand: 'get_class_by_name',
    transformFilters: (filters) => ({
      name_contains: filters.name || null,
      sources: filters.sources || null,
    }),
  })

  // Custom getDetails with ApiResponse handling
  async function getClassDetails(name: string, source: string): Promise<ClassWithDetails | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Class; error?: string }>('get_class_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        // Wrap in ClassWithDetails structure
        return {
          class: response.data,
          subclasses: [],
          features: [],
          subclass_features: [],
          fluff: response.data.fluff,
          subclass_fluff: []
        }
      }
      return null
    } catch (e) {
      console.error('Failed to get class details:', e)
      return null
    }
  }

  async function getSubclassDetails(subclassName: string, className: string, classSource: string): Promise<Subclass | null> {
    // Subclass details not currently supported in backend
    console.warn('getSubclassDetails not implemented in backend')
    return null
  }

  async function getClassSubclasses(className: string, classSource: string): Promise<Subclass[]> {
    // Class subclasses not currently supported in backend
    console.warn('getClassSubclasses not implemented in backend')
    return []
  }

  return {
    isClassesInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    classes: catalog.results,
    classSources,
    initializeClassCatalog: catalog.initialize,
    searchClasses: catalog.search,
    getClassDetails,
    getSubclassDetails,
    getClassSubclasses,
  }
}
