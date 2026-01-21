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
  caption: string
  columns: number
  rows: number
  category: string
}

export interface TableFilters {
  query?: string
  sources?: string[]
  categories?: string[]
}

export function useTables() {
  async function initializeTableCatalog() {
    try {
      await invoke('init_table_catalog')
    } catch (e) {
      // Initialization failed silently
    }
  }

  async function searchTables(filters: TableFilters = {}): Promise<TableSummary[]> {
    try {
      const results = await invoke<TableSummary[]>('search_tables', {
        query: filters.query || null,
        sources: filters.sources || null,
        categories: filters.categories || null
      })
      return results || []
    } catch (e) {
      return []
    }
  }

  async function getTableDetails(name: string, source: string): Promise<Table | null> {
    try {
      const details = await invoke<Table>('get_table_details', { name, source })
      return details
    } catch (e) {
      return null
    }
  }

  async function getTableCategories(): Promise<string[]> {
    try {
      const categories = await invoke<string[]>('get_table_categories')
      return categories || []
    } catch (e) {
      return []
    }
  }

  async function getTableSources(): Promise<string[]> {
    try {
      const sources = await invoke<string[]>('get_table_sources')
      return sources || []
    } catch (e) {
      return []
    }
  }

  return {
    initializeTableCatalog,
    searchTables,
    getTableDetails,
    getTableCategories,
    getTableSources,
  }
}
