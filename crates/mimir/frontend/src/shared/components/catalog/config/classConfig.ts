import type { CatalogConfig } from './types'

// Format spellcasting ability as uppercase badge
function formatSpellcasting(ability: string | null): { text: string; variant: string } | string {
  if (!ability) return '—'
  
  return {
    text: ability.toUpperCase(),
    variant: 'primary'
  }
}

export const classConfig: CatalogConfig = {
  name: 'classes',
  title: 'Classes',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'hitDice',
      label: 'Hit Dice',
      type: 'text',
      className: 'text-center',
      formatter: (value: any) => {
        return value || '1d6'
      }
    },
    {
      key: 'primaryAbility',
      label: 'Primary Ability',
      type: 'text',
      formatter: (value: any) => {
        return value || 'Various'
      }
    },
    {
      key: 'proficiency',
      label: 'Proficiencies',
      type: 'text',
      formatter: (value: any) => {
        return value || 'None'
      }
    },
    {
      key: 'spellcastingAbility',
      label: 'Spellcasting',
      type: 'badge',
      formatter: (value: any) => {
        return formatSpellcasting(typeof value === 'string' ? value : null)
      }
    },
    {
      key: 'subclassName',
      label: 'Subclass',
      type: 'text',
      formatter: (value: any) => {
        return value || '—'
      }
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text',
      formatter: (value: any) => {
        if (!value || typeof value !== 'string') return '—'
        return value
      }
    }
  ],
  filters: [
    {
      type: 'multiselect',
      key: 'sources',
      label: 'Source',
      apiSource: 'get_class_sources'
    },
    {
      type: 'multiselect',
      key: 'primary_abilities',
      label: 'Primary Ability',
      apiSource: 'get_class_primary_abilities'
    },
    {
      type: 'checkbox',
      key: 'has_spellcasting',
      label: 'Has Spellcasting'
    }
  ],
  searchCommands: {
    search: 'search_classes',
    details: 'get_class_details',
    sources: 'get_class_statistics'
  },
  emptyMessage: {
    title: 'No classes found',
    subtitle: 'Search for character classes to see results',
    noResults: 'No classes found matching your criteria'
  },
  searchPlaceholder: 'Search classes...'
}