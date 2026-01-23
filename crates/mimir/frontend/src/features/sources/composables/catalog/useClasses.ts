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
  subclassSource?: string
  rowType?: string
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

  async function getSubclassDetails(subclassName: string, className: string, source: string): Promise<Subclass | null> {
    try {
      const response = await invoke<{ success: boolean; data?: any; error?: string }>('get_subclass_by_name', {
        name: subclassName,
        className,
        source
      })
      if (response.success && response.data) {
        return response.data as Subclass
      }
      return null
    } catch (e) {
      console.error('Failed to get subclass details:', e)
      return null
    }
  }

  async function getClassSubclasses(className: string): Promise<Subclass[]> {
    try {
      const response = await invoke<{ success: boolean; data?: any[]; error?: string }>('list_subclasses_by_class', {
        className
      })
      if (response.success && response.data) {
        return response.data as Subclass[]
      }
      return []
    } catch (e) {
      console.error('Failed to get class subclasses:', e)
      return []
    }
  }

  // Fetch classes with their subclasses merged into a flat list
  // Groups by source book, then shows each class with subclasses from that book
  async function searchClassesWithSubclasses(filters?: ClassFilters): Promise<ClassSummary[]> {
    // First get all classes
    await catalog.search(filters || {})
    const classes = catalog.results.value

    // Cache subclasses by class name to avoid duplicate fetches
    const subclassCache = new Map<string, Subclass[]>()

    // Build results: for each class (from each source), show its subclasses from that source
    const results: ClassSummary[] = []
    let prevClassKey = '' // Track class+source combo for grouping

    for (const cls of classes) {
      // Fetch subclasses if not cached
      if (!subclassCache.has(cls.name)) {
        const allSubclasses = await getClassSubclasses(cls.name)
        subclassCache.set(cls.name, allSubclasses)
      }

      const allSubclasses = subclassCache.get(cls.name) || []

      // Filter to subclasses from the same source as this class
      const subclasses = allSubclasses.filter(sc => sc.source === cls.source)

      const classKey = `${cls.name}|${cls.source}`

      // Always add the base class row first (no subclass selected)
      results.push({
        ...cls,
        subclassName: '—',
        rowType: 'class-base'
      })

      // Then add rows for each subclass from this source
      for (const subclass of subclasses) {
        results.push({
          ...cls,
          subclassName: subclass.name,
          rowType: 'class-subclass',
          subclassSource: subclass.source,
        } as ClassSummary & { subclassSource: string })
      }

      prevClassKey = classKey
    }

    return results
  }

  return {
    isClassesInitialized: catalog.isInitialized,
    isLoading: catalog.isLoading,
    error: catalog.error,
    classes: catalog.results,
    classSources,
    initializeClassCatalog: catalog.initialize,
    searchClasses: catalog.search,
    searchClassesWithSubclasses,
    getClassDetails,
    getSubclassDetails,
    getClassSubclasses,
  }
}
