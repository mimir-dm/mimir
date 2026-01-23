import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Table {
  name: string
  source: string
  page?: number
  caption?: string
  col_labels?: string[]
  col_styles?: string[]
  rows: any[][]
  intro?: any[]
  outro?: any[]
  table_include?: any
  footnotes?: any[]
  srd?: boolean
  basic_rules?: boolean
  has_fluff?: boolean
  has_fluff_images?: boolean
}

export interface TableSummary {
  name: string
  source: string
  caption?: string
}

export interface TableFilters {
  query?: string
  sources?: string[]
}

export function useTables() {
  const isTablesInitialized = ref(true)
  const isLoading = ref(false)
  const error: Ref<string | null> = ref(null)
  const tables = ref<TableSummary[]>([])

  async function initializeTableCatalog() {
    // No initialization needed for DB-backed catalog
  }

  async function searchTables(filters: TableFilters = {}): Promise<TableSummary[]> {
    try {
      isLoading.value = true
      error.value = null

      // Transform to backend CatalogTableFilter format
      const backendFilter = {
        name_contains: filters.query || null,
        sources: filters.sources ?? null,
      }

      const response = await invoke<{ success: boolean; data?: TableSummary[]; error?: string }>('search_tables', {
        filter: backendFilter,
        limit: 10000,
        offset: 0
      })

      if (response.success && response.data) {
        tables.value = response.data
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

  async function getTableDetails(name: string, source: string): Promise<Table | null> {
    try {
      const response = await invoke<{ success: boolean; data?: Table; error?: string }>('get_table_by_name', {
        name,
        source
      })
      if (response.success && response.data) {
        return response.data
      }
      return null
    } catch (e) {
      console.error('Failed to get table details:', e)
      return null
    }
  }

  return {
    isTablesInitialized,
    isLoading,
    error,
    tables,
    initializeTableCatalog,
    searchTables,
    getTableDetails,
  }
}
