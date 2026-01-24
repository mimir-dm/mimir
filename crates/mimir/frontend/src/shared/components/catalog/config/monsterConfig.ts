import type { CatalogConfig } from './types'

// Size formatting for display - handles both string and array formats from 5etools
const formatSize = (size: unknown): string => {
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
  // Handle array format from 5etools (e.g., ["M"])
  const sizeStr = Array.isArray(size) ? size[0] : size
  if (typeof sizeStr === 'string') {
    return sizeMap[sizeStr] || sizeStr
  }
  return 'Unknown'
}

// Type formatting - handles both string and object formats from 5etools
const formatType = (type: unknown): string => {
  if (typeof type === 'string') {
    return type.charAt(0).toUpperCase() + type.slice(1)
  }
  if (type && typeof type === 'object' && 'type' in type) {
    const typeStr = (type as { type: string }).type
    return typeStr.charAt(0).toUpperCase() + typeStr.slice(1)
  }
  return 'Unknown'
}

// CR formatting - handles both string and object formats from 5etools
const formatCR = (cr: unknown): string => {
  if (typeof cr === 'string') {
    return cr
  }
  if (typeof cr === 'number') {
    return cr.toString()
  }
  if (cr && typeof cr === 'object' && 'cr' in cr) {
    return String((cr as { cr: string | number }).cr)
  }
  return '0'
}

// HP formatting - handles object format from 5etools
const formatHP = (hp: unknown): string => {
  if (typeof hp === 'number') {
    return hp.toString()
  }
  if (hp && typeof hp === 'object' && 'average' in hp) {
    return String((hp as { average: number }).average)
  }
  return '—'
}

// AC formatting - handles array format from 5etools
const formatAC = (ac: unknown): string => {
  if (typeof ac === 'number') {
    return ac.toString()
  }
  if (Array.isArray(ac) && ac.length > 0) {
    const firstAC = ac[0]
    if (typeof firstAC === 'number') {
      return firstAC.toString()
    }
    if (firstAC && typeof firstAC === 'object' && 'ac' in firstAC) {
      return String((firstAC as { ac: number }).ac)
    }
  }
  return '10'
}

// Alignment formatting - handles array format from 5etools
const formatAlignment = (alignment: unknown): string => {
  const alignmentMap: Record<string, string> = {
    'L': 'Lawful',
    'N': 'Neutral',
    'NX': 'Neutral',
    'NY': 'Neutral',
    'C': 'Chaotic',
    'G': 'Good',
    'E': 'Evil',
    'U': 'Unaligned',
    'A': 'Any',
    'LG': 'Lawful Good',
    'LN': 'Lawful Neutral',
    'LE': 'Lawful Evil',
    'NG': 'Neutral Good',
    'NE': 'Neutral Evil',
    'CG': 'Chaotic Good',
    'CN': 'Chaotic Neutral',
    'CE': 'Chaotic Evil'
  }

  if (typeof alignment === 'string') {
    return alignmentMap[alignment] || alignment
  }

  if (Array.isArray(alignment)) {
    // Handle 5etools format like ["C", "E"] -> "Chaotic Evil"
    const mapped = alignment
      .filter((a): a is string => typeof a === 'string')
      .map(a => alignmentMap[a] || a)
    if (mapped.length === 2) {
      // Combine like ["Chaotic", "Evil"] -> "Chaotic Evil"
      return mapped.join(' ')
    }
    return mapped.join(', ')
  }

  return 'Unknown'
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
      formatter: formatSize
    },
    {
      key: 'type',
      label: 'Type',
      sortable: true,
      width: '120px',
      formatter: formatType
    },
    {
      key: 'cr',
      label: 'CR',
      sortable: true,
      width: '60px',
      formatter: formatCR
    },
    {
      key: 'hp',
      label: 'HP',
      sortable: true,
      width: '60px',
      formatter: formatHP
    },
    {
      key: 'ac',
      label: 'AC',
      sortable: true,
      width: '60px',
      formatter: formatAC
    },
    {
      key: 'alignment',
      label: 'Alignment',
      sortable: true,
      width: '120px',
      formatter: formatAlignment
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
      type: 'checkbox-group',
      key: 'sizes',
      label: 'Size:',
      options: [
        { value: 'T', label: 'Tiny' },
        { value: 'S', label: 'Small' },
        { value: 'M', label: 'Med' },
        { value: 'L', label: 'Large' },
        { value: 'H', label: 'Huge' },
        { value: 'G', label: 'Garg' }
      ]
    },
    {
      type: 'multiselect',
      key: 'creature_types',
      label: 'Type',
      options: ['aberration', 'beast', 'celestial', 'construct', 'dragon', 'elemental', 'fey', 'fiend', 'giant', 'humanoid', 'monstrosity', 'ooze', 'plant', 'undead']
    },
    {
      type: 'checkbox-group',
      key: 'alignments',
      label: 'Alignment:',
      options: [
        { value: 'LG', label: 'LG' },
        { value: 'NG', label: 'NG' },
        { value: 'CG', label: 'CG' },
        { value: 'LN', label: 'LN' },
        { value: 'N', label: 'N' },
        { value: 'CN', label: 'CN' },
        { value: 'LE', label: 'LE' },
        { value: 'NE', label: 'NE' },
        { value: 'CE', label: 'CE' },
        { value: 'U', label: 'Un' },
        { value: 'A', label: 'Any' }
      ]
    }
  ],

  searchPlaceholder: 'Search monsters...',
  emptyMessage: {
    title: 'No monsters found',
    subtitle: 'Search for monsters',
    noResults: 'No monsters found matching your criteria'
  }
}