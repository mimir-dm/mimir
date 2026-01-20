// Book-related type definitions

export interface BookInfo {
  id: string
  name: string
  image_count?: number
  // Add other properties as needed based on actual data structure
}

export interface BookContent {
  success: boolean
  data: BookSection[] | null
  message?: string
}

export interface BookSection {
  type?: string
  name?: string
  entries?: BookEntry[]
  page?: number
  source?: string
  id?: string
  [key: string]: any // Allow for additional properties
}

export type BookEntry = string | ComplexEntry

export interface ComplexEntry {
  type: string
  name?: string
  entries?: BookEntry[]
  items?: any[]
  page?: number
  source?: string
  id?: string
  caption?: string
  colLabels?: string[]
  colStyles?: string[]
  rows?: any[][]
  href?: {
    path?: string
    type?: string
  }
  title?: string
  style?: string
  by?: string
  roll?: {
    min?: number
    max?: number
  }
  [key: string]: any
}

export interface SubEntry {
  id: string
  name: string
  level: number
  children?: SubEntry[]
}