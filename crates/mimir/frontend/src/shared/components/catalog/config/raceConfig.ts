import type { CatalogConfig } from './types'

// Size mapping from 5etools codes to human-readable names
const sizeMap: Record<string, string> = {
  'F': 'Fine',
  'D': 'Diminutive',
  'T': 'Tiny',
  'S': 'Small',
  'M': 'Medium',
  'L': 'Large',
  'H': 'Huge',
  'G': 'Gargantuan',
  'V': 'Varies'
}

// Format size from 5etools array format
function formatSize(size: unknown): string {
  if (Array.isArray(size)) {
    const sizes = size
      .map((s: unknown) => {
        if (typeof s === 'string') return sizeMap[s] || s
        return null
      })
      .filter(Boolean)
    return sizes.length > 0 ? sizes.join('/') : '—'
  }
  if (typeof size === 'string') {
    return sizeMap[size] || size
  }
  return '—'
}

// Format speed from 5etools format (can be number or object)
function formatSpeed(speed: unknown): string {
  if (typeof speed === 'number') {
    return `${speed} ft.`
  }
  if (typeof speed === 'object' && speed !== null) {
    const speedObj = speed as Record<string, unknown>
    // Handle walk speed
    if ('walk' in speedObj) {
      const walk = speedObj.walk
      if (typeof walk === 'number') {
        return `${walk} ft.`
      }
      if (typeof walk === 'object' && walk !== null && 'number' in walk) {
        return `${(walk as { number: number }).number} ft.`
      }
    }
    // If no walk, return first available speed type
    for (const [type, value] of Object.entries(speedObj)) {
      if (typeof value === 'number') {
        return `${value} ft. (${type})`
      }
    }
  }
  return '—'
}

// Format ability score bonuses from 5etools format
function formatAbilityBonuses(ability: unknown): string {
  if (!Array.isArray(ability) || ability.length === 0) return '—'

  const bonuses: string[] = []
  const statNames: Record<string, string> = {
    str: 'STR',
    dex: 'DEX',
    con: 'CON',
    int: 'INT',
    wis: 'WIS',
    cha: 'CHA'
  }

  for (const abilitySet of ability) {
    if (typeof abilitySet !== 'object' || abilitySet === null) continue

    // Handle choose format
    if ('choose' in abilitySet) {
      const choose = abilitySet.choose as { from?: string[]; count?: number; amount?: number }
      const count = choose.count || 1
      const amount = choose.amount || 1
      if (choose.from) {
        bonuses.push(`+${amount} to ${count} from ${choose.from.map(s => statNames[s] || s).join('/')}`)
      } else {
        bonuses.push(`+${amount} to any ${count}`)
      }
      continue
    }

    // Handle direct ability bonuses
    for (const [stat, value] of Object.entries(abilitySet)) {
      if (stat === 'choose') continue
      if (typeof value === 'number') {
        const sign = value >= 0 ? '+' : ''
        bonuses.push(`${statNames[stat] || stat.toUpperCase()} ${sign}${value}`)
      }
    }
  }

  return bonuses.length > 0 ? bonuses.join(', ') : '—'
}

// Count traits from entries array
function countTraits(entries: unknown): string {
  if (!Array.isArray(entries)) return '—'

  // Count entries that have names (these are the traits)
  const traitCount = entries.filter((entry: unknown) => {
    if (typeof entry === 'object' && entry !== null && 'name' in entry) {
      return true
    }
    return false
  }).length

  return traitCount > 0 ? traitCount.toString() : '—'
}

// Format race name, combining subrace + parent race when applicable
function formatRaceName(name: unknown, row: unknown): string {
  if (typeof name !== 'string') return '—'

  // Check if this is a subrace (has raceName field)
  if (typeof row === 'object' && row !== null) {
    const data = row as Record<string, unknown>
    const parentRace = data.raceName
    if (typeof parentRace === 'string' && parentRace) {
      // Combine subrace name with parent race: "Wood" + "Elf" = "Wood Elf"
      return `${name} ${parentRace}`
    }
  }

  return name
}

export const raceConfig: CatalogConfig = {
  name: 'races',
  title: 'Races',
  searchCommands: {
    search: 'search_races',
    details: 'get_race_details',
    sources: 'get_race_sources'
  },
  columns: [
    {
      key: 'name',
      label: 'Name',
      sortable: true,
      className: 'catalog-table__cell-name',
      formatter: formatRaceName
    },
    {
      key: 'size',
      label: 'Size',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: formatSize
    },
    {
      key: 'speed',
      label: 'Speed',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: formatSpeed
    },
    {
      key: 'ability',
      label: 'Ability Bonuses',
      type: 'text',
      className: 'catalog-table__cell-secondary',
      formatter: formatAbilityBonuses
    },
    {
      key: 'entries',
      label: 'Traits',
      type: 'text',
      className: 'catalog-table__cell-center',
      formatter: countTraits
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
      placeholder: 'Search races...'
    }
  ],
  emptyMessage: {
    title: 'No races found',
    subtitle: 'Search for races to see results',
    noResults: 'No races found matching your criteria'
  }
}
