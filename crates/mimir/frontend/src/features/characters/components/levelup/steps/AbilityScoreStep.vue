<template>
  <div class="asi-step">
    <h3 class="step-heading">Ability Score Improvement</h3>
    <p class="step-description">
      At level {{ levelUp.newClassLevel.value }}, you gain an Ability Score Improvement. Choose to
      increase your ability scores or take a feat.
    </p>

    <!-- Choice: ASI or Feat -->
    <div class="choice-buttons">
      <button
        type="button"
        class="choice-button"
        :class="{ active: choiceType === 'asi' }"
        @click="choiceType = 'asi'"
      >
        Ability Score Improvement
      </button>
      <button
        type="button"
        class="choice-button"
        :class="{ active: choiceType === 'feat' }"
        @click="choiceType = 'feat'"
      >
        Feat
      </button>
    </div>

    <!-- ASI Selection -->
    <div v-if="choiceType === 'asi'" class="asi-section">
      <p class="section-note">Increase your ability scores by a total of 2 points (max 20 each).</p>

      <div class="ability-grid">
        <div v-for="ability in abilities" :key="ability.name" class="ability-row">
          <span class="ability-name">{{ ability.label }}</span>
          <span class="ability-current">{{ ability.score }}</span>
          <div class="ability-controls">
            <button
              type="button"
              class="control-btn"
              :disabled="getIncrease(ability.name) === 0"
              @click="decreaseAbility(ability.name)"
            >
              -
            </button>
            <span class="increase-value">+{{ getIncrease(ability.name) }}</span>
            <button
              type="button"
              class="control-btn"
              :disabled="!canIncreaseAbility(ability.name)"
              @click="increaseAbility(ability.name)"
            >
              +
            </button>
          </div>
          <span class="ability-new">{{ ability.score + getIncrease(ability.name) }}</span>
        </div>
      </div>

      <div class="points-remaining">
        Points remaining: <strong>{{ 2 - totalIncrease }}</strong>
      </div>
    </div>

    <!-- Feat Selection -->
    <div v-if="choiceType === 'feat'" class="feat-section">
      <!-- Search Box -->
      <div class="search-box">
        <input
          v-model="featSearch"
          type="text"
          class="form-input search-input"
          placeholder="Search feats..."
          @input="debouncedSearch"
        />
      </div>

      <!-- Loading State -->
      <div v-if="isLoadingFeats" class="loading-indicator">Loading feats...</div>

      <!-- Feat Grid -->
      <div v-else-if="filteredFeats.length > 0" class="feat-grid">
        <button
          v-for="feat in filteredFeats"
          :key="`${feat.name}-${feat.source}`"
          type="button"
          class="feat-card"
          :class="{
            selected: isFeatSelected(feat),
            disabled: !feat.meetsPrereqs
          }"
          :disabled="!feat.meetsPrereqs"
          @click="selectFeat(feat)"
        >
          <div class="feat-name">{{ feat.name }}</div>
          <div class="feat-source">{{ feat.source }}</div>
          <div v-if="feat.prereqDisplay" class="feat-prereq" :class="{ unmet: !feat.meetsPrereqs }">
            {{ feat.prereqDisplay }}
          </div>
        </button>
      </div>

      <!-- No Results -->
      <div v-else-if="featSearch && !isLoadingFeats" class="no-results">
        No feats found matching "{{ featSearch }}"
      </div>

      <!-- Manual Entry Fallback -->
      <div v-if="feats.length === 0 && !isLoadingFeats" class="manual-entry">
        <p class="fallback-note">
          Enter a feat manually if the catalog is unavailable:
        </p>
        <div class="manual-row">
          <div class="form-group">
            <label class="form-label" for="feat-name">Feat Name</label>
            <input
              id="feat-name"
              v-model="manualFeatName"
              type="text"
              class="form-input"
              placeholder="e.g., Great Weapon Master, Sentinel"
            />
          </div>
          <div class="form-group source-group">
            <label class="form-label" for="feat-source">Source</label>
            <select id="feat-source" v-model="manualFeatSource" class="form-input">
              <option value="PHB">Player's Handbook</option>
              <option value="XGE">Xanathar's Guide</option>
              <option value="TCE">Tasha's Cauldron</option>
            </select>
          </div>
          <button
            type="button"
            class="btn btn-secondary"
            :disabled="!manualFeatName.trim()"
            @click="selectManualFeat"
          >
            Select
          </button>
        </div>
      </div>
    </div>

    <!-- Apply Button -->
    <button
      type="button"
      class="btn btn-primary apply-btn"
      :disabled="!canApply"
      @click="applyChoice"
    >
      Apply {{ choiceType === 'asi' ? 'Ability Score Improvement' : 'Feat' }}
    </button>

    <!-- Current Selection Display -->
    <div v-if="levelUp.asiOrFeat.value" class="current-selection">
      <span class="selection-label">Selected:</span>
      <template v-if="levelUp.asiOrFeat.value.type === 'AbilityScoreImprovement'">
        <span class="selection-value">
          {{ formatAbility(levelUp.asiOrFeat.value.ability1) }} +{{
            levelUp.asiOrFeat.value.increase1
          }}
          <template v-if="levelUp.asiOrFeat.value.ability2">
            , {{ formatAbility(levelUp.asiOrFeat.value.ability2) }} +{{
              levelUp.asiOrFeat.value.increase2
            }}
          </template>
        </span>
      </template>
      <template v-else>
        <span class="selection-value">{{ levelUp.asiOrFeat.value.name }}</span>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive, onMounted } from 'vue'
import type { Character } from '@/types/character'
import type { LevelUpComposable } from '@/features/characters/composables/useLevelUp'
import { invoke } from '@tauri-apps/api/core'

interface FeatOption {
  name: string
  source: string
  prereqDisplay: string | null
  meetsPrereqs: boolean
}

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

const choiceType = ref<'asi' | 'feat'>('asi')
const featSearch = ref('')
const manualFeatName = ref('')
const manualFeatSource = ref('PHB')
const isLoadingFeats = ref(false)
const feats = ref<FeatOption[]>([])
let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Ability score increases
const increases = reactive<Record<string, number>>({
  strength: 0,
  dexterity: 0,
  constitution: 0,
  intelligence: 0,
  wisdom: 0,
  charisma: 0
})

const abilities = computed(() => [
  { name: 'strength', label: 'Strength', score: props.character.strength },
  { name: 'dexterity', label: 'Dexterity', score: props.character.dexterity },
  { name: 'constitution', label: 'Constitution', score: props.character.constitution },
  { name: 'intelligence', label: 'Intelligence', score: props.character.intelligence },
  { name: 'wisdom', label: 'Wisdom', score: props.character.wisdom },
  { name: 'charisma', label: 'Charisma', score: props.character.charisma }
])

const totalIncrease = computed(() => {
  return Object.values(increases).reduce((sum, val) => sum + val, 0)
})

const filteredFeats = computed(() => {
  if (!featSearch.value.trim()) return feats.value
  const search = featSearch.value.toLowerCase()
  return feats.value.filter((f) => f.name.toLowerCase().includes(search))
})

function getIncrease(ability: string): number {
  return increases[ability] || 0
}

function canIncreaseAbility(ability: string): boolean {
  if (totalIncrease.value >= 2) return false
  const currentScore = abilities.value.find((a) => a.name === ability)?.score ?? 10
  const increase = getIncrease(ability)
  // Can't go above 20, can't increase more than 2 for one ability
  return currentScore + increase < 20 && increase < 2
}

function increaseAbility(ability: string) {
  if (canIncreaseAbility(ability)) {
    increases[ability] = (increases[ability] || 0) + 1
  }
}

function decreaseAbility(ability: string) {
  if (increases[ability] > 0) {
    increases[ability]--
  }
}

function formatAbility(ability: string): string {
  return ability.charAt(0).toUpperCase() + ability.slice(1, 3).toUpperCase()
}

// Feat functions
async function loadFeats() {
  isLoadingFeats.value = true
  try {
    const result = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        parsed_prereqs?: string[]
        prerequisite?: unknown
      }>
      error?: string
    }>('list_feats_with_prereqs', { filter: null, limit: 100, offset: 0 })

    if (result.success && result.data) {
      feats.value = result.data.map((feat) => {
        const prereqs = feat.parsed_prereqs || []
        const prereqDisplay = prereqs.length > 0 ? prereqs.join(', ') : null

        // Check if character meets prerequisites
        const meetsPrereqs = checkFeatPrereqs(prereqs)

        return {
          name: feat.name,
          source: feat.source,
          prereqDisplay,
          meetsPrereqs
        }
      })
    }
  } catch (e) {
    console.error('Error loading feats:', e)
  } finally {
    isLoadingFeats.value = false
  }
}

function checkFeatPrereqs(prereqs: string[]): boolean {
  if (prereqs.length === 0) return true

  // Simple prerequisite checking for common patterns
  for (const prereq of prereqs) {
    const lower = prereq.toLowerCase()

    // Check ability score prerequisites (e.g., "Strength 13")
    const abilityMatch = lower.match(/(strength|dexterity|constitution|intelligence|wisdom|charisma)\s+(\d+)/)
    if (abilityMatch) {
      const ability = abilityMatch[1]
      const required = parseInt(abilityMatch[2], 10)
      const charScore = props.character[ability as keyof Character] as number
      if (charScore < required) return false
    }

    // Check spellcasting prerequisite
    if (lower.includes('spellcasting') || lower.includes('ability to cast')) {
      // Would need to check if character has spellcasting - simplified check
      const hasSpellcasting = props.character.classes.some((c) =>
        ['wizard', 'sorcerer', 'cleric', 'druid', 'bard', 'warlock', 'paladin', 'ranger'].includes(
          c.class_name.toLowerCase()
        )
      )
      if (!hasSpellcasting) return false
    }

    // Check proficiency prerequisites (simplified)
    if (lower.includes('proficiency with') && lower.includes('armor')) {
      // Would need detailed proficiency checking
      // For now, allow if they have a martial class
      const hasMartial = props.character.classes.some((c) =>
        ['fighter', 'paladin', 'ranger', 'barbarian', 'cleric'].includes(c.class_name.toLowerCase())
      )
      if (lower.includes('heavy') && !hasMartial) return false
    }
  }

  return true
}

function isFeatSelected(feat: FeatOption): boolean {
  const selected = props.levelUp.asiOrFeat.value
  if (!selected || selected.type !== 'Feat') return false
  return selected.name === feat.name && selected.source === feat.source
}

function selectFeat(feat: FeatOption) {
  if (!feat.meetsPrereqs) return
  props.levelUp.asiOrFeat.value = {
    type: 'Feat',
    name: feat.name,
    source: feat.source
  }
}

function selectManualFeat() {
  if (!manualFeatName.value.trim()) return
  props.levelUp.asiOrFeat.value = {
    type: 'Feat',
    name: manualFeatName.value.trim(),
    source: manualFeatSource.value
  }
}

function debouncedSearch() {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    // Filtering is done reactively via filteredFeats
  }, 200)
}

const canApply = computed(() => {
  if (choiceType.value === 'asi') {
    return totalIncrease.value === 2
  } else {
    // For feat, check if a feat is already selected or manual entry is filled
    const selected = props.levelUp.asiOrFeat.value
    if (selected && selected.type === 'Feat') return true
    return manualFeatName.value.trim().length > 0
  }
})

function applyChoice() {
  if (!canApply.value) return

  if (choiceType.value === 'asi') {
    // Find which abilities were increased
    const increased = Object.entries(increases).filter(([, v]) => v > 0)

    if (increased.length === 1) {
      props.levelUp.asiOrFeat.value = {
        type: 'AbilityScoreImprovement',
        ability1: increased[0][0],
        increase1: increased[0][1]
      }
    } else if (increased.length === 2) {
      props.levelUp.asiOrFeat.value = {
        type: 'AbilityScoreImprovement',
        ability1: increased[0][0],
        increase1: increased[0][1],
        ability2: increased[1][0],
        increase2: increased[1][1]
      }
    }
  } else {
    // If no feat selected from catalog, use manual entry
    const selected = props.levelUp.asiOrFeat.value
    if (!selected || selected.type !== 'Feat') {
      selectManualFeat()
    }
  }
}

// Initialize from existing state
watch(
  () => props.levelUp.asiOrFeat.value,
  (asi) => {
    if (asi) {
      if (asi.type === 'AbilityScoreImprovement') {
        choiceType.value = 'asi'
        // Reset and apply
        Object.keys(increases).forEach((k) => (increases[k] = 0))
        increases[asi.ability1] = asi.increase1
        if (asi.ability2 && asi.increase2) {
          increases[asi.ability2] = asi.increase2
        }
      } else {
        choiceType.value = 'feat'
        manualFeatName.value = asi.name
        manualFeatSource.value = asi.source
      }
    }
  },
  { immediate: true }
)

onMounted(() => {
  loadFeats()
})
</script>

<style scoped>
.asi-step {
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

.choice-buttons {
  display: flex;
  gap: var(--spacing-sm);
}

.choice-button {
  flex: 1;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-base);
  font-weight: 500;
}

.choice-button:hover {
  border-color: var(--color-primary-300);
}

.choice-button.active {
  border-color: var(--color-primary-500);
  background: var(--color-surface-hover);
}

.asi-section,
.feat-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.section-note {
  margin: 0;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.ability-grid {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.ability-row {
  display: grid;
  grid-template-columns: 120px 60px 1fr 60px;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.ability-name {
  font-weight: 500;
  color: var(--color-text);
}

.ability-current {
  text-align: center;
  color: var(--color-text-secondary);
}

.ability-controls {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
}

.control-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
  font-weight: bold;
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.increase-value {
  min-width: 30px;
  text-align: center;
  font-weight: 600;
  color: var(--color-primary-500);
}

.ability-new {
  text-align: center;
  font-weight: 600;
  color: var(--color-text);
}

.points-remaining {
  text-align: center;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

/* Feat Section Styles */
.search-box {
  margin-bottom: var(--spacing-sm);
}

.search-input {
  width: 100%;
}

.loading-indicator {
  padding: var(--spacing-lg);
  text-align: center;
  color: var(--color-text-secondary);
}

.feat-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--spacing-md);
  max-height: 400px;
  overflow-y: auto;
  padding: var(--spacing-xs);
}

.feat-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: left;
}

.feat-card:hover:not(.disabled) {
  border-color: var(--color-primary-300);
  background: var(--color-surface-variant);
}

.feat-card.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-hover);
}

.feat-card.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.feat-name {
  font-weight: 600;
  color: var(--color-text);
}

.feat-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.feat-prereq {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.feat-prereq.unmet {
  color: var(--color-error, #dc2626);
}

.no-results {
  padding: var(--spacing-lg);
  text-align: center;
  color: var(--color-text-secondary);
}

.manual-entry {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.fallback-note {
  margin: 0 0 var(--spacing-md);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.manual-row {
  display: flex;
  gap: var(--spacing-md);
  align-items: flex-end;
  flex-wrap: wrap;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  flex: 1;
  min-width: 150px;
}

.source-group {
  flex: 0 0 150px;
  min-width: 120px;
}

.form-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
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

.apply-btn {
  align-self: flex-start;
}

.current-selection {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
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
