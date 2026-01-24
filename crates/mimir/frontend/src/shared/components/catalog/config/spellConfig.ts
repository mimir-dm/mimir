import type { CatalogConfig } from './types'
import { formatSpellLevel, formatSpellTags } from '../../../utils/formatters'

// School code to full name mapping
const schoolMap: Record<string, string> = {
  'A': 'Abjuration',
  'C': 'Conjuration',
  'D': 'Divination',
  'E': 'Enchantment',
  'V': 'Evocation',
  'I': 'Illusion',
  'N': 'Necromancy',
  'T': 'Transmutation'
}

// Format school from 5etools single-char code
function formatSchool(school: unknown): string {
  if (typeof school === 'string') {
    return schoolMap[school] || school
  }
  return '—'
}

// Format casting time from 5etools time array
function formatCastingTime(time: unknown): string {
  if (Array.isArray(time) && time.length > 0) {
    const t = time[0]
    if (t && typeof t === 'object') {
      const num = (t as { number?: number }).number || 1
      const unit = (t as { unit?: string }).unit || 'action'
      return num === 1 ? unit : `${num} ${unit}s`
    }
  }
  return '—'
}

// Format range from 5etools range object
function formatRange(range: unknown): string {
  if (!range || typeof range !== 'object') return '—'

  const r = range as { type?: string; distance?: { type?: string; amount?: number } }

  if (r.type === 'point') {
    if (r.distance?.type === 'self') return 'Self'
    if (r.distance?.type === 'touch') return 'Touch'
    if (r.distance?.type === 'sight') return 'Sight'
    if (r.distance?.type === 'unlimited') return 'Unlimited'
    if (r.distance?.amount) return `${r.distance.amount} ${r.distance.type || 'ft'}`
  }
  if (r.type === 'special') return 'Special'

  return '—'
}

// Format components from 5etools components object
function formatComponents(components: unknown): string {
  if (!components || typeof components !== 'object') return '—'

  const c = components as { v?: boolean; s?: boolean; m?: unknown }
  const parts: string[] = []

  if (c.v) parts.push('V')
  if (c.s) parts.push('S')
  if (c.m) parts.push('M')

  return parts.length > 0 ? parts.join(', ') : '—'
}

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
      type: 'text',
      formatter: formatSchool
    },
    {
      key: 'time',
      label: 'Cast Time',
      type: 'text',
      formatter: formatCastingTime
    },
    {
      key: 'range',
      label: 'Range',
      type: 'text',
      formatter: formatRange
    },
    {
      key: 'components',
      label: 'Components',
      type: 'text',
      formatter: formatComponents
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
      type: 'checkbox-group',
      key: 'level',
      label: 'Level:',
      options: [
        { value: '0', label: 'C' },
        { value: '1', label: '1' },
        { value: '2', label: '2' },
        { value: '3', label: '3' },
        { value: '4', label: '4' },
        { value: '5', label: '5' },
        { value: '6', label: '6' },
        { value: '7', label: '7' },
        { value: '8', label: '8' },
        { value: '9', label: '9' }
      ]
    },
    {
      type: 'checkbox-group',
      key: 'school',
      label: 'School:',
      options: [
        { value: 'Abjuration', label: 'Abj' },
        { value: 'Conjuration', label: 'Con' },
        { value: 'Divination', label: 'Div' },
        { value: 'Enchantment', label: 'Enc' },
        { value: 'Evocation', label: 'Evo' },
        { value: 'Illusion', label: 'Ill' },
        { value: 'Necromancy', label: 'Nec' },
        { value: 'Transmutation', label: 'Tra' }
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