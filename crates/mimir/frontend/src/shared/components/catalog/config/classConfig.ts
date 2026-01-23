import type { CatalogConfig } from './types'
import { processFormattingTags } from '@/features/sources/utils/textFormatting'

// Helper to check if this is a subclass row (should blank out class-level columns)
function isSubclassRow(item: unknown): boolean {
  const rowType = (item as any)?.rowType
  return rowType === 'class-subclass'
}

// Wrap a formatter to return empty string for subclass rows (class info shown on base row)
function skipOnSubclassRow<T>(formatter: (value: unknown, item?: unknown) => T): (value: unknown, item?: unknown) => T | string {
  return (value: unknown, item?: unknown) => {
    if (isSubclassRow(item)) return ''
    return formatter(value, item)
  }
}

// Format hit dice from 5etools object format
function formatHitDice(hd: unknown): string {
  if (typeof hd === 'object' && hd !== null && 'faces' in hd) {
    const faces = (hd as { faces: number }).faces
    return `d${faces}`
  }
  return '—'
}

// Format primary ability from 5etools array format
// Note: This receives the full item, not just the primaryAbility field
function formatPrimaryAbility(item: unknown): string {
  const data = item as Record<string, unknown>
  const primaryAbility = data?.primaryAbility

  const statNames: Record<string, string> = {
    str: 'Strength',
    dex: 'Dexterity',
    con: 'Constitution',
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma'
  }

  // Handle array format (newer 5etools / XPHB)
  if (Array.isArray(primaryAbility)) {
    const abilities: string[] = []
    for (const ability of primaryAbility) {
      if (typeof ability === 'object' && ability !== null) {
        for (const [stat, value] of Object.entries(ability)) {
          if (value === true) {
            abilities.push(statNames[stat] || stat)
          }
        }
      }
    }
    if (abilities.length > 0) {
      return abilities.join(' or ')
    }
  }

  // Handle object format
  if (typeof primaryAbility === 'object' && primaryAbility !== null) {
    const abilities: string[] = []
    for (const [stat, value] of Object.entries(primaryAbility)) {
      if (value === true) {
        abilities.push(statNames[stat] || stat)
      }
    }
    if (abilities.length > 0) {
      return abilities.join(' or ')
    }
  }

  // Fallback: For casters without primaryAbility (PHB classes), infer from spellcasting
  const spellcastingAbility = data?.spellcastingAbility
  if (typeof spellcastingAbility === 'string') {
    return statNames[spellcastingAbility] || spellcastingAbility.toUpperCase()
  }

  // No primary ability info available
  return ''
}

// Format starting proficiencies from 5etools object format
function formatProficiencies(startingProficiencies: unknown): string {
  if (typeof startingProficiencies !== 'object' || startingProficiencies === null) {
    return '—'
  }

  const profs: string[] = []
  const sp = startingProficiencies as Record<string, unknown>

  // Armor
  if (Array.isArray(sp.armor)) {
    const armorTypes = sp.armor
      .map((a: unknown) => {
        if (typeof a === 'string') return processFormattingTags(a)
        if (typeof a === 'object' && a !== null && 'proficiency' in a) {
          return processFormattingTags((a as { proficiency: string }).proficiency)
        }
        return null
      })
      .filter(Boolean)
    if (armorTypes.length > 0) {
      profs.push(`Armor: ${armorTypes.join(', ')}`)
    }
  }

  // Weapons
  if (Array.isArray(sp.weapons)) {
    const weapons = sp.weapons
      .map((w: unknown) => {
        if (typeof w === 'string') return processFormattingTags(w)
        if (typeof w === 'object' && w !== null && 'proficiency' in w) {
          return processFormattingTags((w as { proficiency: string }).proficiency)
        }
        return null
      })
      .filter(Boolean)
    if (weapons.length > 0) {
      profs.push(`Weapons: ${weapons.join(', ')}`)
    }
  }

  // Tools
  if (Array.isArray(sp.tools)) {
    profs.push(`Tools: ${sp.tools.length}`)
  }

  // Skills
  if (Array.isArray(sp.skills)) {
    const skillChoices = sp.skills.map((s: unknown) => {
      if (typeof s === 'object' && s !== null && 'choose' in s) {
        const choose = s as { choose: { from: string[]; count: number } }
        return `Choose ${choose.choose.count} skills`
      }
      return null
    }).filter(Boolean)
    if (skillChoices.length > 0) {
      profs.push(skillChoices.join(', '))
    }
  }

  return profs.length > 0 ? profs.join('; ') : '—'
}

// Format spellcasting ability from 5etools string
// Note: badge type columns receive the full item, not just the field value
function formatSpellcasting(item: unknown): { text: string; variant: string } | string {
  const spellcastingAbility = (item as any)?.spellcastingAbility
  // Non-casters don't have spellcasting ability - return empty string (not a badge)
  if (typeof spellcastingAbility !== 'string') return ''

  const abilityMap: Record<string, string> = {
    int: 'INT',
    wis: 'WIS',
    cha: 'CHA'
  }

  const text = abilityMap[spellcastingAbility] || spellcastingAbility.toUpperCase()
  return { text, variant: 'primary' }
}

export const classConfig: CatalogConfig = {
  name: 'classes',
  title: 'Classes',
  columns: [
    {
      key: 'name',
      label: 'Class',
      sortable: true,
      className: 'catalog-table__cell-name',
      formatter: skipOnSubclassRow((value: unknown) => String(value || ''))
    },
    {
      key: 'subclassName',
      label: 'Subclass',
      type: 'text',
      className: 'catalog-table__cell-name',
      formatter: (value: unknown) => {
        if (typeof value === 'string' && value.trim() && value !== '—') return value
        return '—'
      }
    },
    {
      key: 'hd',
      label: 'Hit Dice',
      type: 'text',
      className: 'text-center',
      formatter: skipOnSubclassRow(formatHitDice)
    },
    {
      key: 'primaryAbility',
      label: 'Primary Ability',
      type: 'text',
      formatter: skipOnSubclassRow((_value: unknown, item: unknown) => formatPrimaryAbility(item))
    },
    {
      key: 'spellcastingAbility',
      label: 'Spellcasting',
      type: 'badge',
      formatter: skipOnSubclassRow(formatSpellcasting)
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
      type: 'text',
      key: 'search',
      label: 'Search',
      placeholder: 'Search classes...'
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
