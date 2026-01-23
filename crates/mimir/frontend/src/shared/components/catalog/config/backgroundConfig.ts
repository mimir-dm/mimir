import type { CatalogConfig } from './types'

// Format skill proficiencies from 5etools array
function formatSkills(skillProficiencies: unknown): string {
  if (!Array.isArray(skillProficiencies) || skillProficiencies.length === 0) return '—'

  const skills: string[] = []
  for (const prof of skillProficiencies) {
    if (typeof prof === 'object' && prof !== null) {
      // Handle object format like {insight: true, religion: true}
      for (const [skill, value] of Object.entries(prof)) {
        if (skill === 'choose') continue // Skip choice objects
        if (value === true) {
          skills.push(skill.charAt(0).toUpperCase() + skill.slice(1))
        }
      }
      // Handle choose format like {choose: {from: ["arcana", "history"], count: 1}}
      if ('choose' in prof && typeof prof.choose === 'object' && prof.choose !== null) {
        const choose = prof.choose as { from?: string[]; count?: number }
        if (choose.from) {
          const count = choose.count || 1
          skills.push(`Choose ${count} from ${choose.from.join(', ')}`)
        }
      }
    }
  }
  return skills.length > 0 ? skills.join(', ') : '—'
}

// Format tool proficiencies from 5etools array
function formatTools(toolProficiencies: unknown): string {
  if (!Array.isArray(toolProficiencies) || toolProficiencies.length === 0) return '—'

  const tools: string[] = []
  for (const prof of toolProficiencies) {
    if (typeof prof === 'string') {
      tools.push(prof)
    } else if (typeof prof === 'object' && prof !== null) {
      for (const [tool, value] of Object.entries(prof)) {
        if (tool === 'choose') continue
        if (value === true) {
          tools.push(tool)
        }
      }
    }
  }
  return tools.length > 0 ? tools.join(', ') : '—'
}

// Format language proficiencies from 5etools array
function formatLanguages(languageProficiencies: unknown): string {
  if (!Array.isArray(languageProficiencies) || languageProficiencies.length === 0) return '—'

  const languages: string[] = []
  for (const prof of languageProficiencies) {
    if (typeof prof === 'object' && prof !== null) {
      for (const [lang, value] of Object.entries(prof)) {
        if (lang === 'anyStandard') {
          languages.push(`Any ${value} standard`)
        } else if (lang === 'any') {
          languages.push(`Any ${value}`)
        } else if (value === true) {
          languages.push(lang.charAt(0).toUpperCase() + lang.slice(1))
        }
      }
    }
  }
  return languages.length > 0 ? languages.join(', ') : '—'
}

// Extract feature name from entries
function formatFeature(entries: unknown): string {
  if (!Array.isArray(entries) || entries.length === 0) return '—'

  // Look for the first entry with type "entries" which is typically the feature
  for (const entry of entries) {
    if (typeof entry === 'object' && entry !== null && 'type' in entry && 'name' in entry) {
      if (entry.type === 'entries' && typeof entry.name === 'string') {
        return entry.name
      }
    }
  }
  return '—'
}

export const backgroundConfig: CatalogConfig = {
  name: 'backgrounds',
  title: 'Backgrounds',
  searchCommands: {
    search: 'search_backgrounds',
    details: 'get_background_details',
    sources: 'get_background_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'skillProficiencies',
      label: 'Skills',
      type: 'text',
      className: 'catalog-table__cell-skills',
      formatter: formatSkills
    },
    {
      key: 'languageProficiencies',
      label: 'Languages',
      type: 'text',
      className: 'catalog-table__cell-languages',
      formatter: formatLanguages
    },
    {
      key: 'toolProficiencies',
      label: 'Tools',
      type: 'text',
      className: 'catalog-table__cell-tools',
      formatter: formatTools
    },
    {
      key: 'entries',
      label: 'Feature',
      type: 'text',
      className: 'catalog-table__cell-feature',
      formatter: formatFeature
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
      placeholder: 'Search backgrounds...'
    }
  ],
  emptyMessage: {
    title: 'No backgrounds found',
    subtitle: 'Search for backgrounds to see results',
    noResults: 'No backgrounds found matching your criteria'
  }
}
