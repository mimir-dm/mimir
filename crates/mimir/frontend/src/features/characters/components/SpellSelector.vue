<template>
  <div class="spell-selector">
    <div v-if="loading" class="loading">Loading spells...</div>

    <div v-else>
      <!-- Spell lists by level -->
      <div v-for="(spells, level) in spellsByLevel" :key="level" class="spell-level-group">
        <h4 class="spell-level-header">
          {{ Number(level) === 0 ? 'Cantrips' : `Level ${level} Spells` }}
          <span class="level-limit">
            ({{ getSelectedCount(Number(level)) }} / {{ spellsAllowed[Number(level)] || 0 }})
          </span>
        </h4>
        <div class="spell-grid">
          <div
            v-for="spell in spells"
            :key="`${spell.name}:${spell.source}`"
            class="spell-option"
            :class="{ selected: isSelected(spell.name) }"
            @click="toggleSpell(spell, Number(level))"
          >
            <input type="checkbox" :checked="isSelected(spell.name)" />
            <span class="spell-name">{{ spell.name }}</span>
            <span class="spell-school">{{ spell.school }}</span>
          </div>
        </div>
      </div>

      <div v-if="Object.keys(spellsByLevel).length === 0" class="no-spells">
        No spells available for this class.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SpellReferenceInput } from '@/types/character'

interface SpellOption {
  name: string
  level: number
  school: string
  source: string
  classes: string[]
}

const props = defineProps<{
  className: string
  maxSpellLevel: number
  spellsAllowed: Record<number, number>  // { 0: 2, 1: 4 } = 2 cantrips, 4 level 1 spells
  initialSelection?: SpellReferenceInput[]
}>()

const emit = defineEmits<{
  'update:selected': [spells: SpellReferenceInput[]]
  'update:selectedGrouped': [grouped: Record<number, SpellReferenceInput[]>]
}>()

const loading = ref(false)
const availableSpells = ref<SpellOption[]>([])
const selectedSpells = ref<SpellReferenceInput[]>([])

// Group spells by level
const spellsByLevel = computed(() => {
  const grouped: Record<number, SpellOption[]> = {}
  const hasAllowances = Object.keys(props.spellsAllowed).length > 0

  for (const spell of availableSpells.value) {
    // Include if no allowances defined (show all) or if level has an allowance
    if (!hasAllowances || props.spellsAllowed[spell.level] !== undefined) {
      if (!grouped[spell.level]) {
        grouped[spell.level] = []
      }
      grouped[spell.level].push(spell)
    }
  }

  // Sort spells within each level
  for (const level in grouped) {
    grouped[level].sort((a, b) => a.name.localeCompare(b.name))
  }

  return grouped
})

// Get count of selected spells at a specific level
const getSelectedCount = (level: number): number => {
  return selectedSpells.value.filter(ref => {
    const spell = availableSpells.value.find(s => s.name === ref.name && s.source === ref.source)
    return spell && spell.level === level
  }).length
}

// Check if spell is selected
const isSelected = (spellName: string): boolean => {
  return selectedSpells.value.some(ref => ref.name === spellName)
}

// Get selected spells grouped by level
const getSelectedGrouped = (): Record<number, SpellReferenceInput[]> => {
  const grouped: Record<number, SpellReferenceInput[]> = {}
  for (const ref of selectedSpells.value) {
    const spell = availableSpells.value.find(s => s.name === ref.name && s.source === ref.source)
    if (spell) {
      if (!grouped[spell.level]) {
        grouped[spell.level] = []
      }
      grouped[spell.level].push(ref)
    }
  }
  return grouped
}

// Emit both flat and grouped selection
const emitSelection = () => {
  emit('update:selected', [...selectedSpells.value])
  emit('update:selectedGrouped', getSelectedGrouped())
}

// Toggle spell selection
const toggleSpell = (spell: SpellOption, level: number) => {
  const index = selectedSpells.value.findIndex(
    ref => ref.name === spell.name && ref.source === spell.source
  )

  if (index === -1) {
    // Adding - check limit (unlimited if not defined)
    const currentCount = getSelectedCount(level)
    const allowed = props.spellsAllowed[level]
    const hasLimit = allowed !== undefined

    if (!hasLimit || currentCount < allowed) {
      selectedSpells.value.push({ name: spell.name, source: spell.source })
      emitSelection()
    }
  } else {
    // Removing
    selectedSpells.value.splice(index, 1)
    emitSelection()
  }
}

// Load spells from catalog
const loadSpells = async () => {
  loading.value = true

  try {
    const results = await invoke<SpellOption[]>('search_spells', {
      query: null,
      sources: null,
      levels: null,
      schools: null,
      tags: null,
      limit: null,
      offset: null
    })

    console.log('SpellSelector: Loaded', results.length, 'total spells')
    console.log('SpellSelector: Filtering for class:', props.className, 'maxLevel:', props.maxSpellLevel)
    console.log('SpellSelector: spellsAllowed:', props.spellsAllowed)

    // Sample first spell to see structure
    if (results.length > 0) {
      console.log('SpellSelector: Sample spell keys:', Object.keys(results[0]))
      console.log('SpellSelector: Sample spell classes:', results[0].classes)
    }

    // Filter to max spell level and class
    const classNameLower = props.className.toLowerCase()
    availableSpells.value = results.filter(spell => {
      // Check level
      if (spell.level > props.maxSpellLevel) return false

      // Check class - if no classes defined, include all for now
      if (spell.classes && spell.classes.length > 0) {
        return spell.classes.some(c => c.toLowerCase() === classNameLower)
      }
      // Fallback: include spell if no class data (will be filtered out when data is populated)
      return true
    })

    console.log('SpellSelector: After filtering:', availableSpells.value.length, 'spells for', props.className)
  } catch (e) {
    console.error('Failed to load spells:', e)
    availableSpells.value = []
  } finally {
    loading.value = false
  }
}

// Initialize
onMounted(async () => {
  if (props.initialSelection) {
    selectedSpells.value = [...props.initialSelection]
  }
  await loadSpells()
})

// Watch for class changes
watch(() => props.className, async () => {
  selectedSpells.value = []
  await loadSpells()
})

// Watch for initial selection changes
watch(() => props.initialSelection, (newVal) => {
  if (newVal) {
    selectedSpells.value = [...newVal]
  }
})
</script>

<style scoped>
.spell-selector {
  @apply space-y-4;
}

.loading {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
}

.selection-summary {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.level-count {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 0.875rem;
}

.level-label {
  color: var(--color-text-secondary);
}

.count-value {
  font-weight: 600;
  color: var(--color-text);
}

.count-value.at-limit {
  color: var(--color-success);
}

.count-value.over-limit {
  color: var(--color-error);
}

.spell-level-group {
  @apply space-y-2;
}

.spell-level-header {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  padding-bottom: var(--spacing-xs);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.level-limit {
  font-size: 0.75rem;
  font-weight: normal;
  color: var(--color-text-secondary);
}

.spell-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-sm);
}

.spell-option {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-size: 0.875rem;
}

.spell-option:hover {
  border-color: var(--color-primary-300);
}

.spell-option.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-variant);
}

.spell-option input[type="checkbox"] {
  flex-shrink: 0;
}

.spell-name {
  flex: 1;
  color: var(--color-text);
}

.spell-school {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.no-spells {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-style: italic;
}
</style>
