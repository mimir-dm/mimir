import type { CatalogConfig } from './types'

// Format prerequisites from 5etools array format
function formatPrerequisites(prerequisite: unknown): string {
  if (!Array.isArray(prerequisite) || prerequisite.length === 0) return '—'

  const prereqs: string[] = []

  for (const prereq of prerequisite) {
    if (typeof prereq !== 'object' || prereq === null) continue

    // Handle ability score requirements
    if ('ability' in prereq && Array.isArray(prereq.ability)) {
      for (const ability of prereq.ability) {
        if (typeof ability === 'object' && ability !== null) {
          const entries = Object.entries(ability)
          for (const [stat, value] of entries) {
            const statName = stat.charAt(0).toUpperCase() + stat.slice(1)
            prereqs.push(`${statName} ${value}+`)
          }
        }
      }
    }

    // Handle level requirements
    if ('level' in prereq) {
      const level = prereq.level
      if (typeof level === 'number') {
        prereqs.push(`Level ${level}`)
      } else if (typeof level === 'object' && level !== null && 'level' in level) {
        prereqs.push(`Level ${level.level}`)
      }
    }

    // Handle race requirements
    if ('race' in prereq && Array.isArray(prereq.race)) {
      const races = prereq.race
        .map((r: unknown) => {
          if (typeof r === 'string') return r
          if (typeof r === 'object' && r !== null && 'name' in r) return r.name
          return null
        })
        .filter(Boolean)
      if (races.length > 0) {
        prereqs.push(races.join(' or '))
      }
    }

    // Handle spellcasting requirement
    if ('spellcasting' in prereq && prereq.spellcasting === true) {
      prereqs.push('Spellcasting')
    }
    if ('spellcastingFeature' in prereq && prereq.spellcastingFeature === true) {
      prereqs.push('Spellcasting feature')
    }

    // Handle proficiency requirements
    if ('proficiency' in prereq && Array.isArray(prereq.proficiency)) {
      for (const prof of prereq.proficiency) {
        if (typeof prof === 'object' && prof !== null) {
          if ('armor' in prof) prereqs.push(`${prof.armor} armor proficiency`)
          if ('weapon' in prof) prereqs.push(`${prof.weapon} weapon proficiency`)
        }
      }
    }

    // Handle feat requirements
    if ('feat' in prereq && Array.isArray(prereq.feat)) {
      const feats = prereq.feat.map((f: string) => f.split('|')[0])
      prereqs.push(`Feat: ${feats.join(', ')}`)
    }

    // Handle other text-based prereqs
    if ('other' in prereq && typeof prereq.other === 'string') {
      prereqs.push(prereq.other)
    }
  }

  return prereqs.length > 0 ? prereqs.join('; ') : '—'
}

export const featConfig: CatalogConfig = {
  name: 'feats',
  title: 'Feats',
  searchCommands: {
    search: 'search_feats',
    details: 'get_feat_details',
    sources: 'get_feat_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'prerequisite',
      label: 'Prerequisites',
      type: 'text',
      className: 'catalog-table__cell-prerequisites',
      formatter: formatPrerequisites
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text',
      className: 'catalog-table__cell-source'
    }
  ],
  filters: [
    {
      type: 'text',
      key: 'search',
      label: 'Search',
      placeholder: 'Search feats...'
    }
  ],
  emptyMessage: {
    title: 'No feats found',
    subtitle: 'Search for feats to see results',
    noResults: 'No feats found matching your criteria'
  }
}
