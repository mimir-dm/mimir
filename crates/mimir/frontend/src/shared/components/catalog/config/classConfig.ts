import type { CatalogConfig } from './types'

// Format hit dice from 5etools object format
function formatHitDice(hd: unknown): string {
  if (typeof hd === 'object' && hd !== null && 'faces' in hd) {
    const faces = (hd as { faces: number }).faces
    return `d${faces}`
  }
  return '—'
}

// Format primary ability from 5etools array format
function formatPrimaryAbility(primaryAbility: unknown): string {
  if (!primaryAbility) return '—'

  const statNames: Record<string, string> = {
    str: 'Strength',
    dex: 'Dexterity',
    con: 'Constitution',
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma'
  }

  // Handle array format (newer 5etools)
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
    return abilities.length > 0 ? abilities.join(' or ') : '—'
  }

  // Handle object format
  if (typeof primaryAbility === 'object' && primaryAbility !== null) {
    const abilities: string[] = []
    for (const [stat, value] of Object.entries(primaryAbility)) {
      if (value === true) {
        abilities.push(statNames[stat] || stat)
      }
    }
    return abilities.length > 0 ? abilities.join(' or ') : '—'
  }

  return '—'
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
        if (typeof a === 'string') return a
        if (typeof a === 'object' && a !== null && 'proficiency' in a) {
          return (a as { proficiency: string }).proficiency
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
        if (typeof w === 'string') return w
        if (typeof w === 'object' && w !== null && 'proficiency' in w) {
          return (w as { proficiency: string }).proficiency
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
  if (typeof spellcastingAbility !== 'string') return '—'

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
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name'
    },
    {
      key: 'hd',
      label: 'Hit Dice',
      type: 'text',
      className: 'text-center',
      formatter: formatHitDice
    },
    {
      key: 'primaryAbility',
      label: 'Primary Ability',
      type: 'text',
      formatter: formatPrimaryAbility
    },
    {
      key: 'startingProficiencies',
      label: 'Proficiencies',
      type: 'text',
      formatter: formatProficiencies
    },
    {
      key: 'spellcastingAbility',
      label: 'Spellcasting',
      type: 'badge',
      formatter: formatSpellcasting
    },
    {
      key: 'subclassTitle',
      label: 'Subclass',
      type: 'text',
      formatter: (value: unknown) => {
        if (typeof value === 'string') return value
        return '—'
      }
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
