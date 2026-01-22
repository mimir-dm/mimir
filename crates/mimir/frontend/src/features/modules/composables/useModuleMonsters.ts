import { ref, watch, onMounted, onUnmounted, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { processFormattingTags } from '@/features/sources/utils/textFormatting'
import { dataEvents } from '@/shared/utils/dataEvents'

// Types
export interface MonsterWithData {
  id: string
  module_id: string
  monster_name: string
  monster_source: string
  quantity: number
  encounter_tag: string | null
  /** Custom display name (e.g., "Frost Wight" when using goblin stats) */
  display_name: string | null
  /** DM notes about customizations or thematic changes */
  notes: string | null
  monster_data: any | null
}

export interface EncounterGroup {
  encounter_tag: string | null
  monsters: MonsterWithData[]
}

/**
 * Composable for managing module monsters and encounters
 * Handles loading, selection, and formatting of monster data
 */
export function useModuleMonsters(moduleId: Ref<string>) {
  // State
  const encounterGroups = ref<EncounterGroup[]>([])
  const allMonsters = ref<MonsterWithData[]>([])
  const selectedEncounter = ref<string | null>(null)
  const selectedMonster = ref<MonsterWithData | null>(null)
  const encountersLoading = ref(true)

  // Load encounters/monsters for this module (including monster tokens from maps)
  async function loadEncounters(campaignId?: string) {
    encountersLoading.value = true
    try {
      // Load module monsters
      const response = await invoke<{ data: MonsterWithData[] }>('list_module_monsters_with_data', {
        moduleId: moduleId.value
      })

      const monsters = response.data || []

      // Also load monster tokens from module maps
      const mapMonsters = await loadMapMonsterTokens(campaignId)

      // Combine both sources
      const allMonstersData = [...monsters, ...mapMonsters]
      allMonsters.value = allMonstersData

      // Group monsters by encounter_tag
      const groups = new Map<string | null, MonsterWithData[]>()
      for (const monster of allMonstersData) {
        const tag = monster.encounter_tag
        if (!groups.has(tag)) {
          groups.set(tag, [])
        }
        groups.get(tag)!.push(monster)
      }

      // Convert to array, putting tagged encounters first
      const groupArray: EncounterGroup[] = []
      for (const [tag, groupMonsters] of groups) {
        if (tag !== null) {
          groupArray.push({ encounter_tag: tag, monsters: groupMonsters })
        }
      }
      // Add untagged at the end if any
      if (groups.has(null)) {
        groupArray.push({ encounter_tag: null, monsters: groups.get(null)! })
      }

      encounterGroups.value = groupArray
    } catch (error) {
      console.error('Failed to load encounters:', error)
      encounterGroups.value = []
      allMonsters.value = []
    } finally {
      encountersLoading.value = false
    }
  }

  // Load monster tokens from all maps in this module
  async function loadMapMonsterTokens(campaignId?: string): Promise<MonsterWithData[]> {
    if (!campaignId) return []

    try {
      // Get maps for this module
      const mapsResponse = await invoke<{ success: boolean; data?: any[] }>('list_maps', {
        request: { campaign_id: campaignId, module_id: moduleId.value }
      })

      if (!mapsResponse.success || !mapsResponse.data) return []

      const mapMonsters: MonsterWithData[] = []
      const seenMonsters = new Set<string>() // Track unique monsters by name

      for (const map of mapsResponse.data) {
        // Get tokens for this map
        const tokensResponse = await invoke<{ success: boolean; data?: any[] }>('list_tokens', {
          mapId: map.id
        })

        if (!tokensResponse.success || !tokensResponse.data) continue

        // Filter for monster tokens and convert to MonsterWithData format
        for (const token of tokensResponse.data) {
          if (token.token_type === 'monster') {
            const key = `${token.name}-${token.monster_source || 'MM'}`
            if (!seenMonsters.has(key)) {
              seenMonsters.add(key)
              mapMonsters.push({
                id: `map-token-${token.id}`, // Prefix to distinguish from module_monsters
                module_id: moduleId.value,
                monster_name: token.name,
                monster_source: token.monster_source || 'MM',
                quantity: 1,
                encounter_tag: 'On Map',
                display_name: null,
                notes: null,
                monster_data: null // Will be looked up separately if needed
              })
            }
          }
        }
      }

      return mapMonsters
    } catch (error) {
      console.error('Failed to load map monster tokens:', error)
      return []
    }
  }

  // Select an encounter group to expand
  function selectEncounter(group: EncounterGroup) {
    if (selectedEncounter.value === group.encounter_tag) {
      // Toggle off if clicking same group
      selectedEncounter.value = null
      selectedMonster.value = null
    } else {
      selectedEncounter.value = group.encounter_tag
      // Auto-select first monster in group
      if (group.monsters.length > 0) {
        selectedMonster.value = group.monsters[0]
      }
    }
  }

  // Select a monster to show details
  function selectMonster(monster: MonsterWithData) {
    selectedMonster.value = monster
  }

  // Clear selected monster
  function clearSelectedMonster() {
    selectedMonster.value = null
  }

  // Subscribe to monster change events for automatic refresh
  let unsubscribe: (() => void) | null = null

  onMounted(() => {
    unsubscribe = dataEvents.on('module:monsters:changed', (payload) => {
      if (payload.moduleId === moduleId.value) {
        // Refetch monsters when our module's monsters change
        loadEncounters()
      }
    })
  })

  onUnmounted(() => {
    if (unsubscribe) {
      unsubscribe()
      unsubscribe = null
    }
  })

  return {
    // State
    encounterGroups,
    allMonsters,
    selectedEncounter,
    selectedMonster,
    encountersLoading,
    // Actions
    loadEncounters,
    selectEncounter,
    selectMonster,
    clearSelectedMonster
  }
}

// ============================================
// Monster Formatting Functions
// ============================================

const SIZE_MAP: Record<string, string> = {
  'T': 'Tiny', 'S': 'Small', 'M': 'Medium', 'L': 'Large', 'H': 'Huge', 'G': 'Gargantuan'
}

const ALIGNMENT_MAP: Record<string, string> = {
  'L': 'lawful', 'N': 'neutral', 'C': 'chaotic', 'G': 'good', 'E': 'evil', 'U': 'unaligned', 'A': 'any alignment'
}

const ABILITY_NAMES: Record<string, string> = {
  str: 'Str', dex: 'Dex', con: 'Con', int: 'Int', wis: 'Wis', cha: 'Cha'
}

const SKILL_NAMES: Record<string, string> = {
  acrobatics: 'Acrobatics', athletics: 'Athletics', arcana: 'Arcana',
  deception: 'Deception', history: 'History', insight: 'Insight',
  intimidation: 'Intimidation', investigation: 'Investigation', medicine: 'Medicine',
  nature: 'Nature', perception: 'Perception', performance: 'Performance',
  persuasion: 'Persuasion', religion: 'Religion', sleight_of_hand: 'Sleight of Hand',
  stealth: 'Stealth', survival: 'Survival'
}

const XP_BY_CR: Record<string, string> = {
  '0': '0 or 10', '1/8': '25', '1/4': '50', '1/2': '100',
  '1': '200', '2': '450', '3': '700', '4': '1,100', '5': '1,800',
  '6': '2,300', '7': '2,900', '8': '3,900', '9': '5,000', '10': '5,900',
  '11': '7,200', '12': '8,400', '13': '10,000', '14': '11,500', '15': '13,000',
  '16': '15,000', '17': '18,000', '18': '20,000', '19': '22,000', '20': '25,000',
  '21': '33,000', '22': '41,000', '23': '50,000', '24': '62,000', '25': '75,000',
  '26': '90,000', '27': '105,000', '28': '120,000', '29': '135,000', '30': '155,000'
}

/**
 * Format creature type line (e.g., "Medium humanoid (any race), any alignment")
 */
export function formatCreatureType(monsterData: any): string {
  if (!monsterData) return ''

  const parts: string[] = []

  // Size
  const size = Array.isArray(monsterData.size) ? monsterData.size[0] : monsterData.size
  parts.push(SIZE_MAP[size] || size || 'Medium')

  // Type
  if (monsterData.type) {
    if (typeof monsterData.type === 'string') {
      parts.push(monsterData.type)
    } else if (typeof monsterData.type === 'object') {
      let typeStr = monsterData.type.type || ''
      if (monsterData.type.tags?.length) {
        typeStr += ` (${monsterData.type.tags.join(', ')})`
      }
      parts.push(typeStr)
    }
  }

  // Alignment
  if (monsterData.alignment) {
    const alignment = Array.isArray(monsterData.alignment)
      ? monsterData.alignment.map((a: string) => ALIGNMENT_MAP[a] || a).join(' ')
      : ALIGNMENT_MAP[monsterData.alignment] || monsterData.alignment
    parts.push(`, ${alignment}`)
  }

  return parts.join(' ')
}

/**
 * Format speed (e.g., "30 ft., fly 60 ft., swim 30 ft.")
 */
export function formatSpeed(monsterData: any): string {
  if (!monsterData?.speed) return '30 ft.'

  const speed = monsterData.speed
  const parts: string[] = []

  if (typeof speed === 'number') {
    return `${speed} ft.`
  }

  if (speed.walk) parts.push(`${speed.walk} ft.`)
  if (speed.burrow) parts.push(`burrow ${speed.burrow} ft.`)
  if (speed.climb) parts.push(`climb ${speed.climb} ft.`)
  if (speed.fly) {
    let flyStr = `fly ${speed.fly} ft.`
    if (speed.canHover) flyStr += ' (hover)'
    parts.push(flyStr)
  }
  if (speed.swim) parts.push(`swim ${speed.swim} ft.`)

  return parts.length > 0 ? parts.join(', ') : '30 ft.'
}

/**
 * Format ability modifier (e.g., "+3" or "-1")
 */
export function formatModifier(score: number): string {
  const mod = Math.floor((score - 10) / 2)
  return mod >= 0 ? `+${mod}` : `${mod}`
}

/**
 * Format saving throws
 */
export function formatSaves(monsterData: any): string {
  if (!monsterData?.save) return ''

  const saves: string[] = []
  for (const [ability, bonus] of Object.entries(monsterData.save)) {
    if (bonus) {
      saves.push(`${ABILITY_NAMES[ability] || ability} ${bonus}`)
    }
  }

  return saves.join(', ')
}

/**
 * Format skills
 */
export function formatSkills(monsterData: any): string {
  if (!monsterData?.skill) return ''

  const skills: string[] = []
  for (const [skill, bonus] of Object.entries(monsterData.skill)) {
    if (bonus) {
      skills.push(`${SKILL_NAMES[skill] || skill} ${bonus}`)
    }
  }

  return skills.join(', ')
}

/**
 * Format senses
 */
export function formatSenses(monsterData: any): string {
  if (!monsterData) return ''

  const parts: string[] = []

  if (monsterData.senses) {
    if (Array.isArray(monsterData.senses)) {
      parts.push(...monsterData.senses)
    } else {
      parts.push(monsterData.senses)
    }
  }

  if (monsterData.passive) {
    parts.push(`passive Perception ${monsterData.passive}`)
  }

  return parts.join(', ')
}

/**
 * Format languages
 */
export function formatLanguages(monsterData: any): string {
  if (!monsterData?.languages) return '—'

  if (Array.isArray(monsterData.languages)) {
    return monsterData.languages.join(', ') || '—'
  }

  return monsterData.languages || '—'
}

/**
 * Format damage vulnerabilities
 */
export function formatDamageVulnerabilities(monsterData: any): string {
  if (!monsterData?.vulnerable) return ''
  if (Array.isArray(monsterData.vulnerable)) {
    return monsterData.vulnerable.join(', ')
  }
  return monsterData.vulnerable
}

/**
 * Format damage resistances
 */
export function formatDamageResistances(monsterData: any): string {
  if (!monsterData?.resist) return ''
  if (Array.isArray(monsterData.resist)) {
    return monsterData.resist.map((r: any) => {
      if (typeof r === 'string') return r
      if (r.resist) return r.resist.join(', ') + (r.note ? ` ${r.note}` : '')
      return ''
    }).filter(Boolean).join('; ')
  }
  return monsterData.resist
}

/**
 * Format damage immunities
 */
export function formatDamageImmunities(monsterData: any): string {
  if (!monsterData?.immune) return ''
  if (Array.isArray(monsterData.immune)) {
    return monsterData.immune.map((i: any) => {
      if (typeof i === 'string') return i
      if (i.immune) return i.immune.join(', ') + (i.note ? ` ${i.note}` : '')
      return ''
    }).filter(Boolean).join('; ')
  }
  return monsterData.immune
}

/**
 * Format condition immunities
 */
export function formatConditionImmunities(monsterData: any): string {
  if (!monsterData?.conditionImmune) return ''
  if (Array.isArray(monsterData.conditionImmune)) {
    return monsterData.conditionImmune.join(', ')
  }
  return monsterData.conditionImmune
}

/**
 * Format CR with XP
 */
export function formatCR(monsterData: any): string {
  if (!monsterData?.cr) return '?'

  const cr = monsterData.cr
  const xp = XP_BY_CR[String(cr)] || '?'
  return `${cr} (${xp} XP)`
}

/**
 * Format AC from 5etools data format
 */
export function formatAC(monsterData: any): string {
  if (!monsterData?.ac) return '?'

  const ac = monsterData.ac
  let result = ''
  if (Array.isArray(ac)) {
    // 5etools format: ac is an array of AC objects or numbers
    const first = ac[0]
    if (typeof first === 'number') {
      result = String(first)
    } else if (typeof first === 'object') {
      const base = first.ac || first
      const from = first.from ? ` (${first.from.join(', ')})` : ''
      result = `${base}${from}`
    }
  } else {
    result = String(ac)
  }
  // Process any 5etools formatting tags like {@item}
  return processFormattingTags(result)
}

/**
 * Format HP from 5etools data format
 */
export function formatHP(monsterData: any): string {
  if (!monsterData?.hp) return '?'

  const hp = monsterData.hp
  if (typeof hp === 'object') {
    const avg = hp.average || hp.special || '?'
    const formula = hp.formula ? ` (${hp.formula})` : ''
    return `${avg}${formula}`
  }
  return String(hp)
}

/**
 * Format action/trait entries (array of strings/objects) into HTML
 */
export function formatActionEntries(entries: any[]): string {
  if (!entries || !Array.isArray(entries)) return ''

  return entries.map(entry => {
    if (typeof entry === 'string') {
      return processFormattingTags(entry)
    } else if (entry && typeof entry === 'object') {
      // Handle nested entries objects
      if (entry.entries) {
        return formatActionEntries(entry.entries)
      }
      return ''
    }
    return ''
  }).join(' ')
}

/**
 * Helper for ordinal numbers (1st, 2nd, 3rd, etc.)
 */
export function getOrdinal(n: number): string {
  const suffixes = ['th', 'st', 'nd', 'rd']
  const v = n % 100
  return n + (suffixes[(v - 20) % 10] || suffixes[v] || suffixes[0])
}

/**
 * Extract spellcasting info from monster data (can be in different formats)
 */
export function getSpellcasting(monsterData: any): string | null {
  if (!monsterData) return null

  // Check for spellcasting array (5etools format)
  if (monsterData.spellcasting && Array.isArray(monsterData.spellcasting)) {
    return monsterData.spellcasting.map((sc: any) => {
      let html = ''
      if (sc.headerEntries) {
        html += sc.headerEntries.map((e: string) => `<p>${processFormattingTags(e)}</p>`).join('')
      }
      if (sc.spells) {
        html += '<div class="spell-slots">'
        for (const [level, spellData] of Object.entries(sc.spells as Record<string, any>)) {
          const levelName = level === '0' ? 'Cantrips (at will)' : `${getOrdinal(parseInt(level))} level (${spellData.slots || '?'} slots)`
          const spellList = (spellData.spells || []).map((s: string) => processFormattingTags(s)).join(', ')
          html += `<p><strong>${levelName}:</strong> ${spellList}</p>`
        }
        html += '</div>'
      }
      if (sc.daily) {
        html += '<div class="innate-spells">'
        for (const [uses, spells] of Object.entries(sc.daily as Record<string, string[]>)) {
          const usesText = uses === '1' ? '1/day' : `${uses}/day each`
          const spellList = spells.map((s: string) => processFormattingTags(s)).join(', ')
          html += `<p><strong>${usesText}:</strong> ${spellList}</p>`
        }
        html += '</div>'
      }
      return html
    }).join('')
  }

  // Check for spellcasting in traits
  if (monsterData.trait && Array.isArray(monsterData.trait)) {
    const spellTrait = monsterData.trait.find((t: any) =>
      t.name?.toLowerCase().includes('spellcasting') ||
      t.name?.toLowerCase().includes('innate spellcasting')
    )
    if (spellTrait) {
      return formatActionEntries(spellTrait.entries)
    }
  }

  return null
}
