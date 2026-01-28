<template>
  <div class="spells-step">
    <h3 class="step-heading">Spells</h3>
    <p class="step-description">
      At level {{ levelUp.newClassLevel.value }}, you may learn new spells or swap existing ones.
    </p>

    <!-- Spellcasting Info -->
    <div v-if="spellcastingInfo" class="spellcasting-info">
      <div class="info-item">
        <span class="info-label">Spellcasting Ability:</span>
        <span class="info-value">{{ formatAbility(spellcastingInfo.ability) }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">Max Spell Level:</span>
        <span class="info-value">{{ maxSpellLevel }}</span>
      </div>
    </div>

    <!-- New Cantrips Section -->
    <div v-if="cantripSlots > 0" class="spell-section">
      <h4 class="section-title">
        New Cantrips ({{ selectedCantrips.length }}/{{ cantripSlots }})
      </h4>

      <!-- Cantrip Search -->
      <div class="search-box">
        <input
          v-model="cantripSearch"
          type="text"
          class="form-input"
          placeholder="Search cantrips..."
        />
      </div>

      <!-- Cantrip Grid -->
      <div v-if="isLoadingCantrips" class="loading-indicator">Loading cantrips...</div>
      <div v-else-if="filteredCantrips.length > 0" class="spell-grid">
        <button
          v-for="spell in filteredCantrips"
          :key="`${spell.name}-${spell.source}`"
          type="button"
          class="spell-card"
          :class="{ selected: isCantripSelected(spell) }"
          :disabled="!isCantripSelected(spell) && selectedCantrips.length >= cantripSlots"
          @click="toggleCantrip(spell)"
        >
          <div class="spell-name">{{ spell.name }}</div>
          <div class="spell-meta">
            <span class="spell-school">{{ spell.school }}</span>
          </div>
        </button>
      </div>

      <!-- Selected Cantrips -->
      <div v-if="selectedCantrips.length > 0" class="selected-list">
        <span class="selected-label">Selected:</span>
        <span v-for="c in selectedCantrips" :key="c.name" class="selected-tag">
          {{ c.name }}
          <button type="button" class="remove-btn" @click="removeCantrip(c)">&times;</button>
        </span>
      </div>
    </div>

    <!-- New Spells Section -->
    <div v-if="spellSlots > 0" class="spell-section">
      <h4 class="section-title">
        New Spells ({{ selectedSpells.length }}/{{ spellSlots }})
      </h4>

      <!-- Spell Search and Filter -->
      <div class="search-row">
        <input
          v-model="spellSearch"
          type="text"
          class="form-input search-input"
          placeholder="Search spells..."
        />
        <select v-model="spellLevelFilter" class="form-input level-filter">
          <option :value="null">All Levels</option>
          <option v-for="lvl in availableSpellLevels" :key="lvl" :value="lvl">
            Level {{ lvl }}
          </option>
        </select>
      </div>

      <!-- Spell Grid -->
      <div v-if="isLoadingSpells" class="loading-indicator">Loading spells...</div>
      <div v-else-if="filteredSpells.length > 0" class="spell-grid">
        <button
          v-for="spell in filteredSpells"
          :key="`${spell.name}-${spell.source}`"
          type="button"
          class="spell-card"
          :class="{ selected: isSpellSelected(spell) }"
          :disabled="!isSpellSelected(spell) && selectedSpells.length >= spellSlots"
          @click="toggleSpell(spell)"
        >
          <div class="spell-name">{{ spell.name }}</div>
          <div class="spell-meta">
            <span class="spell-level">Lvl {{ spell.level }}</span>
            <span class="spell-school">{{ spell.school }}</span>
            <span v-if="spell.concentration" class="spell-tag conc">C</span>
            <span v-if="spell.ritual" class="spell-tag ritual">R</span>
          </div>
        </button>
      </div>
      <div v-else-if="!isLoadingSpells" class="no-results">
        No spells found{{ spellSearch ? ` matching "${spellSearch}"` : '' }}
      </div>

      <!-- Selected Spells -->
      <div v-if="selectedSpells.length > 0" class="selected-list">
        <span class="selected-label">Selected:</span>
        <span v-for="s in selectedSpells" :key="s.name" class="selected-tag">
          {{ s.name }} ({{ s.level }})
          <button type="button" class="remove-btn" @click="removeSpell(s)">&times;</button>
        </span>
      </div>
    </div>

    <!-- Spell Swap Section (for Known casters) -->
    <div v-if="canSwapSpells" class="spell-section swap-section">
      <h4 class="section-title">Spell Swap (Optional)</h4>
      <p class="section-note">
        You may replace one spell you know with a different spell of the same level.
      </p>

      <div class="swap-row">
        <div class="form-group">
          <label class="form-label">Spell to Forget</label>
          <input
            v-model="swapOutName"
            type="text"
            class="form-input"
            placeholder="Current spell name"
          />
        </div>
        <span class="swap-arrow">&rarr;</span>
        <div class="form-group">
          <label class="form-label">New Spell</label>
          <input
            v-model="swapInName"
            type="text"
            class="form-input"
            placeholder="Replacement spell name"
          />
        </div>
      </div>

      <div v-if="swapOutName && swapInName" class="swap-preview">
        <span class="swap-out">{{ swapOutName }}</span>
        <span class="swap-arrow">&rarr;</span>
        <span class="swap-in">{{ swapInName }}</span>
      </div>
    </div>

    <!-- Apply Button -->
    <button type="button" class="btn btn-primary apply-btn" @click="applySpellChanges">
      Apply Spell Choices
    </button>

    <!-- Current Selection Summary -->
    <div v-if="hasAnyChanges" class="current-selection">
      <span class="selection-label">Changes:</span>
      <span v-if="selectedCantrips.length > 0" class="selection-value">
        {{ selectedCantrips.length }} cantrip(s)
      </span>
      <span v-if="selectedSpells.length > 0" class="selection-value">
        {{ selectedSpells.length }} spell(s)
      </span>
      <span v-if="swapOutName && swapInName" class="selection-value"> 1 swap </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import type { Character } from '@/types/character'
import type { SpellReference } from '@/types/character'
import type { LevelUpComposable } from '@/features/characters/composables/useLevelUp'
import { invoke } from '@tauri-apps/api/core'

interface SpellOption {
  name: string
  source: string
  level: number
  school: string
  concentration: boolean
  ritual: boolean
}

interface SpellcastingInfo {
  ability: string
  type: string // full, half, third, pact
}

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

// State
const isLoadingCantrips = ref(false)
const isLoadingSpells = ref(false)
const cantrips = ref<SpellOption[]>([])
const spells = ref<SpellOption[]>([])
const cantripSearch = ref('')
const spellSearch = ref('')
const spellLevelFilter = ref<number | null>(null)
const selectedCantrips = ref<SpellReference[]>([])
const selectedSpells = ref<SpellReference[]>([])
const swapOutName = ref('')
const swapInName = ref('')

// Spellcasting info from class
const spellcastingInfo = computed<SpellcastingInfo | null>(() => {
  const classInfo = props.levelUp.classInfo.value
  if (!classInfo?.spellcasting_ability) return null
  return {
    ability: classInfo.spellcasting_ability,
    type: classInfo.caster_type || 'full'
  }
})

// Calculate max spell level based on class level and caster type
const maxSpellLevel = computed(() => {
  const level = props.levelUp.newClassLevel.value
  const type = spellcastingInfo.value?.type || 'full'

  if (type === 'full' || type === 'Full') {
    // Full casters: level 1-2 = 1st, 3-4 = 2nd, etc.
    return Math.min(9, Math.ceil(level / 2))
  } else if (type === 'half' || type === 'Half' || type === '1/2') {
    // Half casters: start at level 2, slower progression
    if (level < 2) return 0
    return Math.min(5, Math.ceil((level - 1) / 2))
  } else if (type === 'third' || type === 'Third' || type === '1/3') {
    // Third casters (Eldritch Knight, Arcane Trickster)
    if (level < 3) return 0
    return Math.min(4, Math.ceil((level - 2) / 3))
  } else if (type === 'pact' || type === 'PactMagic') {
    // Warlock pact magic
    if (level < 1) return 0
    if (level <= 2) return 1
    if (level <= 4) return 2
    if (level <= 6) return 3
    if (level <= 8) return 4
    return 5
  }
  return 1
})

// Calculate cantrip slots gained this level
const cantripSlots = computed(() => {
  const classInfo = props.levelUp.classInfo.value
  if (!classInfo?.spellcasting_ability) return 0

  // Simplified - would use cantrip_progression from class info
  const level = props.levelUp.newClassLevel.value
  if (level === 1) return 2 // Most casters get 2 cantrips at level 1
  if (level === 4 || level === 10) return 1 // Some classes get additional at 4 and 10
  return 0
})

// Calculate spell slots gained this level
const spellSlots = computed(() => {
  const classInfo = props.levelUp.classInfo.value
  if (!classInfo?.spellcasting_ability) return 0

  const level = props.levelUp.newClassLevel.value
  const className = props.levelUp.selectedClass.value?.class_name?.toLowerCase()

  // Simplified spells known progression
  if (['wizard'].includes(className || '')) {
    // Wizards add 2 spells to spellbook per level
    return 2
  } else if (['sorcerer', 'bard', 'ranger', 'warlock'].includes(className || '')) {
    // Known casters typically get 1 new spell per level (varies)
    if (level === 1) return 2
    return 1
  } else if (['cleric', 'druid', 'paladin'].includes(className || '')) {
    // Prepared casters don't gain specific spells
    return 0
  }

  return 0
})

// Can this class swap spells on level up?
const canSwapSpells = computed(() => {
  const className = props.levelUp.selectedClass.value?.class_name?.toLowerCase()
  // Known casters can swap
  return ['sorcerer', 'bard', 'ranger', 'warlock'].includes(className || '')
})

const availableSpellLevels = computed(() => {
  return Array.from({ length: maxSpellLevel.value }, (_, i) => i + 1)
})

const filteredCantrips = computed(() => {
  if (!cantripSearch.value.trim()) return cantrips.value
  const search = cantripSearch.value.toLowerCase()
  return cantrips.value.filter((s) => s.name.toLowerCase().includes(search))
})

const filteredSpells = computed(() => {
  let result = spells.value

  if (spellLevelFilter.value !== null) {
    result = result.filter((s) => s.level === spellLevelFilter.value)
  }

  if (spellSearch.value.trim()) {
    const search = spellSearch.value.toLowerCase()
    result = result.filter((s) => s.name.toLowerCase().includes(search))
  }

  return result.slice(0, 50) // Limit display for performance
})

const hasAnyChanges = computed(() => {
  return (
    selectedCantrips.value.length > 0 ||
    selectedSpells.value.length > 0 ||
    (swapOutName.value.trim() && swapInName.value.trim())
  )
})

// Load spells from catalog
async function loadSpells() {
  const className = props.levelUp.selectedClass.value?.class_name
  if (!className || !spellcastingInfo.value) return

  isLoadingCantrips.value = true
  isLoadingSpells.value = true

  // Fetch character's allowed sources for filtering
  let allowedSources: Set<string> | null = null
  try {
    const sourcesResult = await invoke<{ success: boolean; data?: string[] }>('list_character_sources', {
      characterId: props.character.id
    })
    if (sourcesResult.success && sourcesResult.data && sourcesResult.data.length > 0) {
      allowedSources = new Set(sourcesResult.data)
    }
    // If no sources configured (empty array), allowedSources stays null = show all
  } catch (e) {
    console.warn('Could not load character sources, showing all spells:', e)
  }

  try {
    // Load cantrips (level 0)
    const cantripResult = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        level: number
        school?: string
        duration?: { concentration?: boolean }
        meta?: { ritual?: boolean }
      }>
    }>('get_spells_by_class', { className, level: 0 })

    if (cantripResult.success && cantripResult.data) {
      cantrips.value = cantripResult.data
        .filter((s) => !allowedSources || allowedSources.has(s.source))
        .map((s) => ({
          name: s.name,
          source: s.source,
          level: 0,
          school: formatSchool(s.school),
          concentration: false,
          ritual: false
        }))
    }
  } catch (e) {
    console.error('Error loading cantrips:', e)
  } finally {
    isLoadingCantrips.value = false
  }

  try {
    // Load leveled spells
    const spellResult = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        level: number
        school?: string
        duration?: { concentration?: boolean }
        meta?: { ritual?: boolean }
      }>
    }>('get_spells_by_class', { className, level: null })

    if (spellResult.success && spellResult.data) {
      spells.value = spellResult.data
        .filter((s) => s.level > 0 && s.level <= maxSpellLevel.value)
        .filter((s) => !allowedSources || allowedSources.has(s.source))
        .map((s) => ({
          name: s.name,
          source: s.source,
          level: s.level,
          school: formatSchool(s.school),
          concentration: s.duration?.concentration ?? false,
          ritual: s.meta?.ritual ?? false
        }))
    }
  } catch (e) {
    console.error('Error loading spells:', e)
  } finally {
    isLoadingSpells.value = false
  }
}

function formatSchool(school: string | undefined): string {
  if (!school) return ''
  const schools: Record<string, string> = {
    A: 'Abjuration',
    C: 'Conjuration',
    D: 'Divination',
    E: 'Enchantment',
    V: 'Evocation',
    I: 'Illusion',
    N: 'Necromancy',
    T: 'Transmutation'
  }
  return schools[school] || school
}

function formatAbility(ability: string): string {
  const abilities: Record<string, string> = {
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma'
  }
  return abilities[ability.toLowerCase()] || ability
}

function isCantripSelected(spell: SpellOption): boolean {
  return selectedCantrips.value.some((c) => c.name === spell.name)
}

function isSpellSelected(spell: SpellOption): boolean {
  return selectedSpells.value.some((s) => s.name === spell.name)
}

function toggleCantrip(spell: SpellOption) {
  const idx = selectedCantrips.value.findIndex((c) => c.name === spell.name)
  if (idx >= 0) {
    selectedCantrips.value.splice(idx, 1)
  } else if (selectedCantrips.value.length < cantripSlots.value) {
    selectedCantrips.value.push({ name: spell.name, source: spell.source, level: 0 })
  }
}

function toggleSpell(spell: SpellOption) {
  const idx = selectedSpells.value.findIndex((s) => s.name === spell.name)
  if (idx >= 0) {
    selectedSpells.value.splice(idx, 1)
  } else if (selectedSpells.value.length < spellSlots.value) {
    selectedSpells.value.push({ name: spell.name, source: spell.source, level: spell.level })
  }
}

function removeCantrip(spell: SpellReference) {
  const idx = selectedCantrips.value.findIndex((c) => c.name === spell.name)
  if (idx >= 0) selectedCantrips.value.splice(idx, 1)
}

function removeSpell(spell: SpellReference) {
  const idx = selectedSpells.value.findIndex((s) => s.name === spell.name)
  if (idx >= 0) selectedSpells.value.splice(idx, 1)
}

function applySpellChanges() {
  props.levelUp.spellChanges.value = {
    new_cantrips: selectedCantrips.value.length > 0 ? selectedCantrips.value : undefined,
    new_spells: selectedSpells.value.length > 0 ? selectedSpells.value : undefined,
    swap_out:
      swapOutName.value.trim() && swapInName.value.trim()
        ? { name: swapOutName.value.trim(), source: 'PHB' }
        : undefined,
    swap_in:
      swapOutName.value.trim() && swapInName.value.trim()
        ? { name: swapInName.value.trim(), source: 'PHB' }
        : undefined
  }
}

// Watch for class changes
watch(
  () => props.levelUp.selectedClass.value,
  () => {
    loadSpells()
  }
)

// Initialize from existing state
watch(
  () => props.levelUp.spellChanges.value,
  (changes) => {
    if (changes) {
      if (changes.new_cantrips) selectedCantrips.value = [...changes.new_cantrips]
      if (changes.new_spells) selectedSpells.value = [...changes.new_spells]
      if (changes.swap_out) swapOutName.value = changes.swap_out.name
      if (changes.swap_in) swapInName.value = changes.swap_in.name
    }
  },
  { immediate: true }
)

onMounted(() => {
  loadSpells()
})
</script>

<style scoped>
.spells-step {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.step-heading {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
}

.step-description {
  margin: 0;
  color: var(--color-text-secondary);
}

.spellcasting-info {
  display: flex;
  gap: var(--spacing-lg);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.info-item {
  display: flex;
  gap: var(--spacing-sm);
}

.info-label {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.info-value {
  font-weight: 600;
  color: var(--color-text);
}

.spell-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.section-note {
  margin: 0;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.search-box,
.search-row {
  display: flex;
  gap: var(--spacing-md);
}

.search-input {
  flex: 1;
}

.level-filter {
  width: 120px;
}

.form-input {
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 1rem;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.loading-indicator {
  padding: var(--spacing-md);
  text-align: center;
  color: var(--color-text-secondary);
}

.spell-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--spacing-sm);
  max-height: 300px;
  overflow-y: auto;
  padding: var(--spacing-xs);
}

.spell-card {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--spacing-sm);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: left;
}

.spell-card:hover:not(:disabled) {
  border-color: var(--color-primary-300);
}

.spell-card.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-hover);
}

.spell-card:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.spell-name {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--color-text);
}

.spell-meta {
  display: flex;
  gap: var(--spacing-xs);
  flex-wrap: wrap;
  font-size: 0.7rem;
  color: var(--color-text-secondary);
}

.spell-level {
  font-weight: 500;
  color: var(--color-primary-500);
}

.spell-school {
  color: var(--color-text-secondary);
}

.spell-tag {
  padding: 1px 4px;
  border-radius: 2px;
  font-weight: 600;
}

.spell-tag.conc {
  background: var(--color-warning-bg, #fef9c3);
  color: var(--color-warning, #ca8a04);
}

.spell-tag.ritual {
  background: var(--color-info);
  color: white;
}

.no-results {
  padding: var(--spacing-md);
  text-align: center;
  color: var(--color-text-secondary);
}

.selected-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
  align-items: center;
}

.selected-label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.selected-tag {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-hover);
  color: var(--color-text);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  border: 1px solid var(--color-primary-500);
}

.remove-btn {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  font-size: 1rem;
  line-height: 1;
  padding: 0;
}

.remove-btn:hover {
  color: var(--color-error, #dc2626);
}

.swap-section {
  border-left: 4px solid var(--color-primary-300);
}

.swap-row {
  display: flex;
  gap: var(--spacing-md);
  align-items: flex-end;
}

.swap-row .form-group {
  flex: 1;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.form-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.swap-arrow {
  color: var(--color-text-secondary);
  font-size: 1.25rem;
  padding-bottom: var(--spacing-sm);
}

.swap-preview {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  background: var(--color-surface);
  border-radius: var(--radius-sm);
}

.swap-out {
  color: var(--color-error, #dc2626);
  text-decoration: line-through;
}

.swap-in {
  color: var(--color-success, #22c55e);
  font-weight: 500;
}

.apply-btn {
  align-self: flex-start;
}

.current-selection {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
  padding: var(--spacing-md);
  background: var(--color-success-bg, #f0fdf4);
  border: 1px solid var(--color-success, #22c55e);
  border-radius: var(--radius-md);
}

.selection-label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.selection-value {
  font-weight: 600;
  color: var(--color-success, #22c55e);
}
</style>
