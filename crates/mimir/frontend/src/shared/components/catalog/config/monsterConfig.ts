import { invoke } from '@tauri-apps/api/core'
import type { CatalogConfig } from './types'
import { formatMonsterDetails } from '../../../../features/sources/formatters/monsterFormatterEnhanced'

// Challenge Rating mapping for proper sorting
const crToNumeric = (cr: string): number => {
  switch (cr) {
    case '1/8': return 0.125
    case '1/4': return 0.25  
    case '1/2': return 0.5
    default: return parseFloat(cr) || 0
  }
}

// Size formatting for display
const formatSize = (size: string): string => {
  const sizeMap: Record<string, string> = {
    'F': 'Fine',
    'D': 'Diminutive', 
    'T': 'Tiny',
    'S': 'Small',
    'M': 'Medium',
    'L': 'Large',
    'H': 'Huge',
    'G': 'Gargantuan',
    'C': 'Colossal'
  }
  return sizeMap[size] || size
}

// Alignment formatting for display
const formatAlignment = (alignment: string): string => {
  const alignmentMap: Record<string, string> = {
    'LG': 'Lawful Good',
    'LN': 'Lawful Neutral',
    'LE': 'Lawful Evil',
    'NG': 'Neutral Good',
    'N': 'Neutral',
    'NE': 'Neutral Evil', 
    'CG': 'Chaotic Good',
    'CN': 'Chaotic Neutral',
    'CE': 'Chaotic Evil',
    'U': 'Unaligned',
    'A': 'Any Alignment'
  }
  return alignmentMap[alignment] || alignment
}

export const monsterConfig: CatalogConfig = {
  name: 'monsters',
  title: 'Monsters',
  
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      width: '200px'
    },
    {
      key: 'size',
      label: 'Size',
      sortable: true,
      width: '80px',
      formatter: (value: string) => formatSize(value)
    },
    {
      key: 'type',
      label: 'Type',
      sortable: true,
      width: '120px',
      formatter: (value: string) => value?.charAt(0).toUpperCase() + value?.slice(1) || 'Unknown'
    },
    {
      key: 'cr',
      label: 'CR',
      sortable: true,
      width: '60px',
      formatter: (value: string) => value || '0'
    },
    {
      key: 'hp',
      label: 'HP',
      sortable: true,
      width: '60px',
      formatter: (value: number) => value?.toString() || '1'
    },
    {
      key: 'ac',
      label: 'AC',
      sortable: true,
      width: '60px',
      formatter: (value: number) => value?.toString() || '10'
    },
    {
      key: 'alignment',
      label: 'Alignment',
      sortable: true,
      width: '120px',
      formatter: (value: string) => formatAlignment(value)
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      width: '80px'
    }
  ],

  filters: [
    {
      type: 'multiselect',
      key: 'sizes',
      label: 'Size',
      options: ['F', 'D', 'T', 'S', 'M', 'L', 'H', 'G', 'C']
    },
    {
      type: 'multiselect',
      key: 'creature_types',
      label: 'Type',
      options: ['aberration', 'beast', 'celestial', 'construct', 'dragon', 'elemental', 'fey', 'fiend', 'giant', 'humanoid', 'monstrosity', 'ooze', 'plant', 'undead']
    },
    {
      type: 'multiselect',
      key: 'alignments', 
      label: 'Alignment',
      options: ['LG', 'LN', 'LE', 'NG', 'N', 'NE', 'CG', 'CN', 'CE', 'U', 'A']
    }
  ],

  searchPlaceholder: 'Search monsters...',
  emptyMessage: {
    title: 'No monsters found',
    subtitle: 'Search for monsters',
    noResults: 'No monsters found matching your criteria'
  }
}