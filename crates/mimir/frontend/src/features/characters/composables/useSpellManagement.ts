import { ref, computed, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Character } from '@/types/character'
import {
  isSpellcaster,
  getSpellcastingAbility,
  getSpellSaveDC,
  getSpellAttackBonus,
  getAllSpellcastingStats,
  getMulticlassCasterLevel,
} from '@/utils/characterUtils'

/**
 * Spell info from catalog
 */
export interface SpellInfo {
  name: string
  source: string
  level: number
  school: string | null
  ritual: boolean
  concentration: boolean
  data: Record<string, unknown>
}

/**
 * Spellcasting stats for a single class
 */
export interface SpellcastingStats {
  className: string
  ability: string
  saveDC: number
  attackBonus: number
}

/**
 * Composable for managing character spellcasting state and operations
 */
export function useSpellManagement(
  character: Ref<Character | null>,
  characterId: ComputedRef<string>
) {
  // State
  const classSpells = ref<SpellInfo[]>([])
  const loadingSpells = ref(false)
  const expandedSpells = ref<Set<string>>(new Set())
  const collapsedSpellLevels = ref<Set<number>>(new Set([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])) // start all collapsed

  // Computed properties
  const characterIsSpellcaster = computed(() =>
    character.value ? isSpellcaster(character.value) : false
  )

  const spellcastingAbility = computed(() =>
    character.value ? getSpellcastingAbility(character.value) : null
  )

  const spellSaveDC = computed(() =>
    character.value ? getSpellSaveDC(character.value) : null
  )

  const spellAttackBonus = computed(() =>
    character.value ? getSpellAttackBonus(character.value) : null
  )

  // All spellcasting stats for multiclass characters (one per spellcasting class)
  const allSpellcastingStats = computed(() =>
    character.value ? getAllSpellcastingStats(character.value) : []
  )

  // Whether this is a multiclass spellcaster (has 2+ spellcasting classes)
  const isMulticlassSpellcaster = computed(() => allSpellcastingStats.value.length > 1)

  // Spell slots using proper multiclass caster level calculation
  // Per D&D 5e rules: combine caster levels from all Spellcasting classes, then look up slots
  const spellSlots = computed(() => {
    if (!character.value || !characterIsSpellcaster.value) return null

    const slots: Record<number, number> = {}

    // Check for Warlock first (uses separate Pact Magic)
    const warlock = character.value.classes?.find(
      (c) => c.class_name.toLowerCase() === 'warlock'
    )

    // Get multiclass caster level (excludes Warlock)
    const casterLevel = getMulticlassCasterLevel(character.value)

    if (casterLevel > 0) {
      // Multiclass spell slot progression (same as full caster progression)
      const multiclassSlots: Record<number, number[]> = {
        1: [2],
        2: [3],
        3: [4, 2],
        4: [4, 3],
        5: [4, 3, 2],
        6: [4, 3, 3],
        7: [4, 3, 3, 1],
        8: [4, 3, 3, 2],
        9: [4, 3, 3, 3, 1],
        10: [4, 3, 3, 3, 2],
        11: [4, 3, 3, 3, 2, 1],
        12: [4, 3, 3, 3, 2, 1],
        13: [4, 3, 3, 3, 2, 1, 1],
        14: [4, 3, 3, 3, 2, 1, 1],
        15: [4, 3, 3, 3, 2, 1, 1, 1],
        16: [4, 3, 3, 3, 2, 1, 1, 1],
        17: [4, 3, 3, 3, 2, 1, 1, 1, 1],
        18: [4, 3, 3, 3, 3, 1, 1, 1, 1],
        19: [4, 3, 3, 3, 3, 2, 1, 1, 1],
        20: [4, 3, 3, 3, 3, 2, 2, 1, 1],
      }
      const slotArray = multiclassSlots[casterLevel] || []
      slotArray.forEach((count, idx) => {
        if (count > 0) slots[idx + 1] = count
      })
    }

    // Add Warlock Pact Magic slots separately (they're tracked independently)
    if (warlock) {
      const warlockSlots: Record<number, { count: number; level: number }> = {
        1: { count: 1, level: 1 },
        2: { count: 2, level: 1 },
        3: { count: 2, level: 2 },
        4: { count: 2, level: 2 },
        5: { count: 2, level: 3 },
        6: { count: 2, level: 3 },
        7: { count: 2, level: 4 },
        8: { count: 2, level: 4 },
        9: { count: 2, level: 5 },
        10: { count: 2, level: 5 },
        11: { count: 3, level: 5 },
        12: { count: 3, level: 5 },
        13: { count: 3, level: 5 },
        14: { count: 3, level: 5 },
        15: { count: 3, level: 5 },
        16: { count: 3, level: 5 },
        17: { count: 4, level: 5 },
        18: { count: 4, level: 5 },
        19: { count: 4, level: 5 },
        20: { count: 4, level: 5 },
      }
      const pactMagic = warlockSlots[warlock.level]
      if (pactMagic) {
        // Note: Warlock pact slots are separate from regular slots
        // For now we just add them to the display at their level
        slots[pactMagic.level] = (slots[pactMagic.level] || 0) + pactMagic.count
      }
    }

    return Object.keys(slots).length > 0 ? slots : null
  })

  // Spells grouped by level
  const spellsByLevel = computed(() => {
    const grouped: Record<number, SpellInfo[]> = {}
    for (const spell of classSpells.value) {
      if (!grouped[spell.level]) {
        grouped[spell.level] = []
      }
      grouped[spell.level].push(spell)
    }
    return grouped
  })

  // Helper functions
  const getSchoolName = (code: string | null): string => {
    if (!code) return 'Unknown'
    const schools: Record<string, string> = {
      A: 'Abjuration',
      C: 'Conjuration',
      D: 'Divination',
      E: 'Enchantment',
      V: 'Evocation',
      I: 'Illusion',
      N: 'Necromancy',
      T: 'Transmutation',
    }
    return schools[code] || code
  }

  const getLevelDisplay = (level: number): string => {
    if (level === 0) return 'Cantrip'
    if (level === 1) return '1st Level'
    if (level === 2) return '2nd Level'
    if (level === 3) return '3rd Level'
    return `${level}th Level`
  }

  const toggleSpellDetails = (name: string, source: string) => {
    const key = `${name}|${source}`
    if (expandedSpells.value.has(key)) {
      expandedSpells.value.delete(key)
    } else {
      expandedSpells.value.add(key)
    }
    expandedSpells.value = new Set(expandedSpells.value)
  }

  const toggleSpellLevel = (level: number) => {
    if (collapsedSpellLevels.value.has(level)) {
      collapsedSpellLevels.value.delete(level)
    } else {
      collapsedSpellLevels.value.add(level)
    }
    collapsedSpellLevels.value = new Set(collapsedSpellLevels.value)
  }

  const isSpellLevelCollapsed = (level: number): boolean => {
    return collapsedSpellLevels.value.has(level)
  }

  const isSpellExpanded = (name: string, source: string): boolean => {
    return expandedSpells.value.has(`${name}|${source}`)
  }

  const getSpellCastingTime = (spell: SpellInfo): string => {
    const time = spell.data.time as Array<{ number: number; unit: string }> | undefined
    if (!time || time.length === 0) return 'Unknown'
    const t = time[0]
    return `${t.number} ${t.unit}`
  }

  const getSpellRange = (spell: SpellInfo): string => {
    const range = spell.data.range as { type: string; distance?: { type: string; amount?: number } } | undefined
    if (!range) return 'Unknown'
    if (range.type === 'point') {
      if (range.distance?.type === 'self') return 'Self'
      if (range.distance?.type === 'touch') return 'Touch'
      if (range.distance?.amount) return `${range.distance.amount} ${range.distance.type}`
    }
    if (range.type === 'special') return 'Special'
    return range.type
  }

  const getSpellComponents = (spell: SpellInfo): string => {
    const comp = spell.data.components as { v?: boolean; s?: boolean; m?: unknown } | undefined
    if (!comp) return ''
    const parts: string[] = []
    if (comp.v) parts.push('V')
    if (comp.s) parts.push('S')
    if (comp.m) parts.push('M')
    return parts.join(', ')
  }

  const getSpellDuration = (spell: SpellInfo): string => {
    const duration = spell.data.duration as Array<{ type: string; duration?: { type: string; amount: number }; concentration?: boolean }> | undefined
    if (!duration || duration.length === 0) return 'Unknown'
    const d = duration[0]
    if (d.type === 'instant') return 'Instantaneous'
    if (d.type === 'permanent') return 'Permanent'
    if (d.duration) {
      const conc = d.concentration ? 'Concentration, ' : ''
      return `${conc}${d.duration.amount} ${d.duration.type}`
    }
    return d.type
  }

  const getSpellDescription = (spell: SpellInfo): string => {
    const entries = spell.data.entries as unknown[] | undefined
    if (!entries) return ''

    return entries
      .map((entry) => {
        if (typeof entry === 'string') return entry
        if (typeof entry === 'object' && entry !== null) {
          const e = entry as Record<string, unknown>
          if (e.type === 'entries' && Array.isArray(e.entries)) {
            return (e.entries as unknown[])
              .filter((sub) => typeof sub === 'string')
              .join(' ')
          }
          if (e.type === 'list' && Array.isArray(e.items)) {
            return (e.items as unknown[])
              .filter((sub) => typeof sub === 'string')
              .map((s) => `• ${s}`)
              .join('\n')
          }
        }
        return ''
      })
      .filter(Boolean)
      .join('\n\n')
  }

  // Load spells for the character
  const loadClassSpells = async () => {
    if (!character.value?.classes?.length) return

    // Find spellcasting classes
    const spellcastingClasses = ['bard', 'cleric', 'druid', 'paladin', 'ranger', 'sorcerer', 'warlock', 'wizard']
    const charSpellClasses = character.value.classes.filter((c) =>
      spellcastingClasses.includes(c.class_name.toLowerCase())
    )

    if (charSpellClasses.length === 0) return

    loadingSpells.value = true

    try {
      // Fetch character's allowed sources for filtering
      let allowedSources: Set<string> | null = null
      try {
        const sourcesResult = await invoke<{ success: boolean; data?: string[] }>('list_character_sources', {
          characterId: characterId.value
        })
        if (sourcesResult.success && sourcesResult.data && sourcesResult.data.length > 0) {
          allowedSources = new Set(sourcesResult.data)
        }
        // If no sources configured (empty array), allowedSources stays null = show all
      } catch (e) {
        console.warn('Could not load character sources, showing all spells:', e)
      }

      // Determine max spell level based on class and level
      const getMaxSpellLevel = (className: string, level: number): number => {
        const lowerName = className.toLowerCase()
        if (['paladin', 'ranger'].includes(lowerName)) {
          // Half casters - spell level = floor((level + 1) / 2) but max 5
          if (level < 2) return 0
          return Math.min(5, Math.ceil((level - 1) / 2))
        }
        if (lowerName === 'warlock') {
          // Warlock pact magic
          if (level < 1) return 0
          if (level < 3) return 1
          if (level < 5) return 2
          if (level < 7) return 3
          if (level < 9) return 4
          return 5
        }
        // Full casters
        return Math.min(9, Math.ceil(level / 2))
      }

      let maxLevel = 0
      for (const cls of charSpellClasses) {
        maxLevel = Math.max(maxLevel, getMaxSpellLevel(cls.class_name, cls.level))
      }

      // Use the get_spells_by_class command which joins with spell_classes table
      // Fetch spells for each spellcasting class the character has
      const spellsByClassName = new Map<string, SpellInfo[]>()

      for (const cls of charSpellClasses) {
        const result = await invoke<{ success: boolean; data?: Array<Record<string, unknown>>; error?: string }>(
          'get_spells_by_class',
          { className: cls.class_name }
        )

        if (result.success && result.data) {
          const classSpellList: SpellInfo[] = []
          for (const rawSpell of result.data) {
            // Filter by max spell level for this character
            const spellLevel = rawSpell.level as number
            if (spellLevel > maxLevel) continue

            // Filter by character's allowed sources (if configured)
            const spellSource = rawSpell.source as string
            if (allowedSources && !allowedSources.has(spellSource)) continue

            // Backend merges data at top level via entity_to_json
            // Store the whole rawSpell object as data for accessing time, range, etc.
            classSpellList.push({
              name: rawSpell.name as string,
              source: spellSource,
              level: spellLevel,
              school: rawSpell.school as string | null,
              ritual: (rawSpell.ritual as number) === 1 || rawSpell.ritual === true,
              concentration: (rawSpell.concentration as number) === 1 || rawSpell.concentration === true,
              data: rawSpell as Record<string, unknown>,
            })
          }
          spellsByClassName.set(cls.class_name, classSpellList)
        }
      }

      // Merge and deduplicate spells from all classes
      // Keep name|source as key since different sources may have different spell versions
      const seenSpells = new Set<string>()
      const allSpells: SpellInfo[] = []

      for (const spellList of spellsByClassName.values()) {
        for (const spell of spellList) {
          const key = `${spell.name}|${spell.source}`
          if (!seenSpells.has(key)) {
            seenSpells.add(key)
            allSpells.push(spell)
          }
        }
      }

      // Sort by level, then name
      allSpells.sort((a, b) => {
        if (a.level !== b.level) return a.level - b.level
        return a.name.localeCompare(b.name)
      })

      classSpells.value = allSpells
    } catch (e) {
      console.error('Failed to load class spells:', e)
      classSpells.value = []
    } finally {
      loadingSpells.value = false
    }
  }

  return {
    // State
    classSpells,
    loadingSpells,
    expandedSpells,
    collapsedSpellLevels,

    // Computed
    characterIsSpellcaster,
    spellcastingAbility,
    spellSaveDC,
    spellAttackBonus,
    allSpellcastingStats,
    isMulticlassSpellcaster,
    spellSlots,
    spellsByLevel,

    // Helper functions
    getSchoolName,
    getLevelDisplay,
    toggleSpellDetails,
    toggleSpellLevel,
    isSpellLevelCollapsed,
    isSpellExpanded,
    getSpellCastingTime,
    getSpellRange,
    getSpellComponents,
    getSpellDuration,
    getSpellDescription,

    // Data loading
    loadClassSpells,
  }
}
