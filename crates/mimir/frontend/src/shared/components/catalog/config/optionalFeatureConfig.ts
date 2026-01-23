import type { CatalogConfig } from './types'

// Map feature type codes to full names
const featureTypeMap: Record<string, string> = {
  'EI': 'Eldritch Invocation',
  'MM': 'Metamagic',
  'FS': 'Fighting Style',
  'MV': 'Maneuver',
  'MV:B': 'Maneuver',
  'MV:C2-UA': 'Maneuver',
  'AS': 'Arcane Shot',
  'AS:V1-UA': 'Arcane Shot',
  'AS:V2-UA': 'Arcane Shot',
  'PB': 'Pact Boon',
  'AI': 'Artificer Infusion',
  'SHP:G': 'Ship Upgrade',
  'SHP:H': 'Ship Upgrade',
  'SHP:M': 'Ship Upgrade',
  'SHP:W': 'Ship Upgrade',
  'SHP:F': 'Ship Upgrade',
  'SHP:O': 'Ship Upgrade',
  'IWM:W': 'Infernal War Machine',
  'IWM:A': 'Infernal War Machine',
  'IWM:G': 'Infernal War Machine',
  'OR': 'Onomancy Resonant',
  'RN': 'Rune Knight Rune',
  'AF': 'Alchemical Formula',
  'OB': 'Otherworldly Boon',
  'ED': 'Elemental Discipline',
  'OTH': 'Other'
}

// Map feature type codes to associated class(es)
const featureTypeClassMap: Record<string, string> = {
  'EI': 'Warlock',
  'MM': 'Sorcerer',
  'FS': 'Fighter, Paladin, Ranger',
  'FS:B': 'Bard',
  'FS:P': 'Paladin',
  'FS:R': 'Ranger',
  'FS:F': 'Fighter',
  'MV': 'Fighter',
  'MV:B': 'Fighter',
  'MV:C2-UA': 'Fighter',
  'AS': 'Fighter',
  'AS:V1-UA': 'Fighter',
  'AS:V2-UA': 'Fighter',
  'PB': 'Warlock',
  'AI': 'Artificer',
  'OR': 'Wizard',
  'RN': 'Fighter',
  'AF': 'Artificer',
  'ED': 'Monk',
  'OB': 'Warlock'
}

// Extract feature type code from the item (handles both array and string formats)
function getFeatureTypeCode(item: unknown): string | null {
  if (typeof item !== 'object' || item === null) return null

  const data = item as Record<string, unknown>

  // First check featureType array (5etools format from JSON data)
  const featureTypeArray = data.featureType
  if (Array.isArray(featureTypeArray) && featureTypeArray.length > 0) {
    const firstType = featureTypeArray[0]
    if (typeof firstType === 'string') return firstType
  }

  // Fall back to feature_type string (database column)
  const featureType = data.feature_type
  if (typeof featureType === 'string' && featureType) {
    return featureType
  }

  return null
}

// Format feature type from code to full name
function formatFeatureType(_value: unknown, item: unknown): string {
  const code = getFeatureTypeCode(item)
  if (code) {
    return featureTypeMap[code] || code
  }
  return '—'
}

// Format class from feature type
function formatClass(_value: unknown, item: unknown): string {
  const code = getFeatureTypeCode(item)
  if (code) {
    return featureTypeClassMap[code] || '—'
  }
  return '—'
}

// Format prerequisites from 5etools format (directly on item from parsed JSON)
function formatPrerequisites(_value: unknown, item: unknown): string {
  if (typeof item !== 'object' || item === null) return '—'

  const record = item as Record<string, unknown>
  const prerequisite = record.prerequisite

  if (!Array.isArray(prerequisite) || prerequisite.length === 0) return '—'

  const prereqs: string[] = []

  for (const prereq of prerequisite) {
    if (typeof prereq !== 'object' || prereq === null) continue

    // Handle level requirements
    if ('level' in prereq) {
      const level = prereq.level
      if (typeof level === 'number') {
        prereqs.push(`Level ${level}`)
      } else if (typeof level === 'object' && level !== null) {
        const lvlObj = level as Record<string, unknown>
        if ('level' in lvlObj) {
          prereqs.push(`Level ${lvlObj.level}`)
        }
      }
    }

    // Handle spell requirements
    if ('spell' in prereq && Array.isArray(prereq.spell)) {
      const spells = prereq.spell
        .map((s: unknown) => {
          if (typeof s === 'string') {
            // Extract spell name from "spell name|source" or "spell name#c" format
            return s.split(/[|#]/)[0]
          }
          return null
        })
        .filter(Boolean)
      if (spells.length > 0) {
        prereqs.push(`${spells.join(' or ')} spell`)
      }
    }

    // Handle pact requirements
    if ('pact' in prereq && typeof prereq.pact === 'string') {
      prereqs.push(`Pact of the ${prereq.pact}`)
    }

    // Handle patron requirements
    if ('patron' in prereq && typeof prereq.patron === 'string') {
      prereqs.push(`${prereq.patron} patron`)
    }

    // Handle invocation requirements
    if ('invocation' in prereq && Array.isArray(prereq.invocation)) {
      const invocations = prereq.invocation
        .map((i: unknown) => {
          if (typeof i === 'string') return i.split(/[|#]/)[0]
          return null
        })
        .filter(Boolean)
      if (invocations.length > 0) {
        prereqs.push(`${invocations.join(' or ')} invocation`)
      }
    }

    // Handle class feature requirements
    if ('feature' in prereq && Array.isArray(prereq.feature)) {
      const features = prereq.feature
        .map((f: unknown) => {
          if (typeof f === 'string') return f.split(/[|#]/)[0]
          return null
        })
        .filter(Boolean)
      if (features.length > 0) {
        prereqs.push(features.join(' or '))
      }
    }

    // Handle other text-based prereqs
    if ('other' in prereq && typeof prereq.other === 'string') {
      prereqs.push(prereq.other)
    }
  }

  return prereqs.length > 0 ? prereqs.join('; ') : '—'
}

export const optionalFeatureConfig: CatalogConfig = {
  name: 'optionalFeatures',
  title: 'Optional Features',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'featureType',
      label: 'Type',
      type: 'text',
      formatter: formatFeatureType
    },
    {
      key: 'featureType',
      label: 'Class',
      type: 'text',
      formatter: formatClass
    },
    {
      key: 'prerequisite',
      label: 'Prerequisites',
      type: 'text',
      formatter: formatPrerequisites
    },
    {
      key: 'source',
      label: 'Source',
      type: 'text',
      sortable: true
    }
  ],
  filters: [
    {
      type: 'multiselect',
      key: 'feature_types',
      label: 'Feature Type',
      apiSource: 'get_optional_feature_types'
    },
    {
      type: 'checkbox',
      key: 'grants_spells',
      label: 'Grants Spells',
      tooltip: 'Filter by features that grant additional spells'
    }
  ],
  searchCommands: {
    search: 'search_optional_features',
    details: 'get_optional_feature_details',
    sources: 'get_optional_feature_sources'
  },
  emptyMessage: {
    title: 'No optional features found',
    subtitle: 'Search for optional features and class variants',
    noResults: 'No optional features found matching your criteria'
  },
  searchPlaceholder: 'Search optional features...'
}