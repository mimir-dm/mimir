import type { CatalogConfig } from './types'
import { formatSpellLevel, formatSpellTags } from '../../../utils/formatters'

export const spellConfig: CatalogConfig = {
  name: 'spells',
  title: 'Spells',
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'level',
      label: 'Level',
      type: 'number',
      formatter: formatSpellLevel
    },
    {
      key: 'school',
      label: 'School',
      type: 'text'
    },
    {
      key: 'casting_time',
      label: 'Cast Time',
      type: 'text'
    },
    {
      key: 'range',
      label: 'Range',
      type: 'text'
    },
    {
      key: 'components',
      label: 'Components',
      type: 'text'
    },
    {
      key: 'tags',
      label: 'Tags',
      type: 'badges',
      formatter: (spell: any) => formatSpellTags(spell)
    },
    {
      key: 'source',
      label: 'Source',
      sortable: true,
      type: 'text'
    }
  ],
  filters: [
    {
      type: 'multiselect',
      key: 'level',
      label: 'Level',
      options: ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
    },
    {
      type: 'multiselect',
      key: 'school',
      label: 'School',
      options: [
        'Abjuration',
        'Conjuration',
        'Divination',
        'Enchantment',
        'Evocation',
        'Illusion',
        'Necromancy',
        'Transmutation'
      ]
    },
    {
      type: 'checkbox',
      key: 'concentration',
      label: 'Conc',
      tooltip: 'Filter by Concentration spells'
    },
    {
      type: 'checkbox',
      key: 'ritual',
      label: 'Ritual',
      tooltip: 'Filter by Ritual spells'
    }
  ],
  emptyMessage: {
    title: 'No spells found',
    subtitle: 'Search for spells',
    noResults: 'No spells found matching your criteria'
  },
  searchPlaceholder: 'Search spells...'
}