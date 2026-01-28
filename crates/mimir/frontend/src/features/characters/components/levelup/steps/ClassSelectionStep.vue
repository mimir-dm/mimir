<template>
  <div class="class-selection-step">
    <h3 class="step-heading">Choose a Class to Level Up</h3>
    <p class="step-description">
      Select a class to gain a level in. You can level up an existing class or multiclass into a new
      one.
    </p>

    <!-- Current Classes -->
    <div v-if="character.classes.length > 0" class="class-section">
      <h4 class="section-title">Continue in Current Class</h4>
      <div class="class-grid">
        <button
          v-for="charClass in character.classes"
          :key="charClass.id"
          type="button"
          class="class-card"
          :class="{ selected: isSelected(charClass) && !levelUp.isNewClass.value }"
          @click="selectExistingClass(charClass)"
        >
          <div class="class-name">{{ charClass.class_name }}</div>
          <div class="class-level">Level {{ charClass.level }}</div>
          <div v-if="charClass.subclass_name" class="class-subclass">
            {{ charClass.subclass_name }}
          </div>
          <div class="class-next">
            <span class="arrow">&rarr;</span> Level {{ charClass.level + 1 }}
          </div>
        </button>
      </div>
    </div>

    <!-- Multiclass Option -->
    <div class="class-section">
      <h4 class="section-title">
        {{ character.classes.length > 0 ? 'Multiclass into New Class' : 'Choose Starting Class' }}
      </h4>

      <!-- Available Classes from Catalog -->
      <div v-if="isLoadingClasses" class="loading-indicator">Loading classes...</div>

      <div v-else-if="availableClasses.length > 0" class="multiclass-grid">
        <button
          v-for="cls in availableClasses"
          :key="`${cls.name}-${cls.source}`"
          type="button"
          class="multiclass-card"
          :class="{
            selected: isMulticlassSelected(cls),
            disabled: !cls.meetsPrereqs && character.classes.length > 0
          }"
          :disabled="!cls.meetsPrereqs && character.classes.length > 0"
          @click="selectMulticlass(cls)"
        >
          <div class="multiclass-name">{{ cls.name }}</div>
          <div class="multiclass-source">{{ cls.source }}</div>
          <div class="multiclass-hitdie">d{{ cls.hit_die }}</div>

          <!-- Multiclass Prerequisites -->
          <div
            v-if="character.classes.length > 0 && cls.prereqDisplay"
            class="prereq-section"
            :class="{ met: cls.meetsPrereqs, unmet: !cls.meetsPrereqs }"
          >
            <span class="prereq-label">Requires:</span>
            <span class="prereq-value">{{ cls.prereqDisplay }}</span>
            <span v-if="!cls.meetsPrereqs && cls.missingDisplay" class="prereq-missing">
              (You have {{ cls.missingDisplay }})
            </span>
          </div>
        </button>
      </div>

      <!-- Manual Entry Fallback -->
      <div v-else class="multiclass-input">
        <div class="form-group">
          <label class="form-label" for="new-class-name">Class Name</label>
          <input
            id="new-class-name"
            v-model="newClassName"
            type="text"
            class="form-input"
            placeholder="e.g., Fighter, Wizard, Rogue"
          />
        </div>
        <div class="form-group">
          <label class="form-label" for="new-class-source">Source</label>
          <select id="new-class-source" v-model="newClassSource" class="form-input">
            <option value="PHB">Player's Handbook</option>
            <option value="XGE">Xanathar's Guide</option>
            <option value="TCE">Tasha's Cauldron</option>
          </select>
        </div>
        <button
          type="button"
          class="btn btn-secondary add-class-btn"
          :disabled="!canAddNewClass"
          @click="selectManualClass"
        >
          {{ character.classes.length > 0 ? 'Multiclass' : 'Select Class' }}
        </button>
      </div>

      <!-- New class preview -->
      <div v-if="levelUp.isNewClass.value && levelUp.selectedClass.value" class="new-class-preview">
        <span class="preview-label">Selected:</span>
        <span class="preview-class">{{ levelUp.selectedClass.value.class_name }}</span>
        <span class="preview-level">Level 1</span>
        <span v-if="levelUp.classInfo.value" class="preview-hitdie">
          (d{{ levelUp.classInfo.value.hit_die }} hit die)
        </span>
      </div>
    </div>

    <!-- Level Summary -->
    <div class="level-summary">
      <div class="summary-item">
        <span class="summary-label">Current Total Level:</span>
        <span class="summary-value">{{ currentTotalLevel }}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">New Total Level:</span>
        <span class="summary-value highlight">{{ levelUp.newTotalLevel.value }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { Character, CharacterClass } from '@/types/character'
import { totalLevel, abilityModifier } from '@/types/character'
import type { LevelUpComposable } from '@/features/characters/composables/useLevelUp'
import { invoke } from '@tauri-apps/api/core'

interface ClassOption {
  name: string
  source: string
  hit_die: number
  prereqs: MulticlassPrereqs | null
  prereqDisplay: string | null
  meetsPrereqs: boolean
  missingDisplay: string | null
}

interface MulticlassPrereqs {
  [key: string]: number | { [key: string]: number }[]
}

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

const newClassName = ref('')
const newClassSource = ref('PHB')
const availableClasses = ref<ClassOption[]>([])
const isLoadingClasses = ref(false)

const currentTotalLevel = computed(() => totalLevel(props.character))

const canAddNewClass = computed(() => {
  return newClassName.value.trim().length > 0
})

// Check if character meets multiclass prerequisites
function meetsPrereqs(prereqs: MulticlassPrereqs | null): {
  meets: boolean
  display: string | null
  missing: string | null
} {
  if (!prereqs) return { meets: true, display: null, missing: null }

  const abilityMap: Record<string, number> = {
    str: props.character.strength,
    dex: props.character.dexterity,
    con: props.character.constitution,
    int: props.character.intelligence,
    wis: props.character.wisdom,
    cha: props.character.charisma
  }

  const abilityNames: Record<string, string> = {
    str: 'STR',
    dex: 'DEX',
    con: 'CON',
    int: 'INT',
    wis: 'WIS',
    cha: 'CHA'
  }

  // Handle OR requirements (array)
  if (prereqs.or && Array.isArray(prereqs.or)) {
    const options = prereqs.or as { [key: string]: number }[]
    const displays: string[] = []
    const missingParts: string[] = []

    for (const option of options) {
      let optionMet = true
      const optionDisplay: string[] = []
      const optionMissing: string[] = []

      for (const [ability, required] of Object.entries(option)) {
        const abilityKey = ability.toLowerCase()
        const current = abilityMap[abilityKey] ?? 10
        optionDisplay.push(`${abilityNames[abilityKey] || ability} ${required}`)

        if (current < required) {
          optionMet = false
          optionMissing.push(`${abilityNames[abilityKey] || ability} ${current}`)
        }
      }

      displays.push(optionDisplay.join(' and '))

      if (optionMet) {
        return { meets: true, display: displays.join(' OR '), missing: null }
      }

      missingParts.push(optionMissing.join(', '))
    }

    return {
      meets: false,
      display: displays.join(' OR '),
      missing: missingParts.join(' / ')
    }
  }

  // Handle AND requirements (simple object)
  const displays: string[] = []
  const missingParts: string[] = []
  let allMet = true

  for (const [ability, required] of Object.entries(prereqs)) {
    if (ability === 'or') continue
    const abilityKey = ability.toLowerCase()
    const current = abilityMap[abilityKey] ?? 10
    const reqValue = typeof required === 'number' ? required : 13

    displays.push(`${abilityNames[abilityKey] || ability} ${reqValue}`)

    if (current < reqValue) {
      allMet = false
      missingParts.push(`${abilityNames[abilityKey] || ability} ${current}`)
    }
  }

  return {
    meets: allMet,
    display: displays.join(' and '),
    missing: allMet ? null : missingParts.join(', ')
  }
}

// Load available classes from catalog
async function loadAvailableClasses() {
  isLoadingClasses.value = true
  try {
    const result = await invoke<{
      success: boolean
      data: Array<{ name: string; source: string; data: string }>
      error?: string
    }>('search_classes', { filter: { sources: ['PHB'] }, limit: 20 })

    if (result.success && result.data) {
      availableClasses.value = result.data.map((cls) => {
        let data: { hd?: { faces?: number }; multiclassing?: { requirements?: MulticlassPrereqs } } =
          {}
        try {
          data = JSON.parse(cls.data)
        } catch {
          // Ignore parse errors
        }

        const hitDie = data?.hd?.faces ?? 8
        const prereqs = data?.multiclassing?.requirements ?? null
        const prereqResult = meetsPrereqs(prereqs)

        return {
          name: cls.name,
          source: cls.source,
          hit_die: hitDie,
          prereqs,
          prereqDisplay: prereqResult.display,
          meetsPrereqs: prereqResult.meets,
          missingDisplay: prereqResult.missing
        }
      })
    }
  } catch (e) {
    console.error('Error loading classes:', e)
  } finally {
    isLoadingClasses.value = false
  }
}

function isSelected(charClass: CharacterClass): boolean {
  const selected = props.levelUp.selectedClass.value
  if (!selected) return false
  return selected.id === charClass.id
}

function isMulticlassSelected(cls: ClassOption): boolean {
  if (!props.levelUp.isNewClass.value) return false
  const selected = props.levelUp.selectedClass.value
  if (!selected) return false
  return (
    selected.class_name.toLowerCase() === cls.name.toLowerCase() &&
    selected.class_source === cls.source
  )
}

function selectExistingClass(charClass: CharacterClass) {
  props.levelUp.selectClass(charClass, false)
}

function selectMulticlass(cls: ClassOption) {
  if (!cls.meetsPrereqs && props.character.classes.length > 0) return

  const newClass: CharacterClass = {
    id: 'new-class-temp',
    character_id: props.character.id,
    class_name: cls.name,
    class_source: cls.source,
    level: 0,
    subclass_name: null,
    subclass_source: null
  }

  props.levelUp.selectClass(newClass, true)
}

function selectManualClass() {
  if (!canAddNewClass.value) return

  const newClass: CharacterClass = {
    id: 'new-class-temp',
    character_id: props.character.id,
    class_name: newClassName.value.trim(),
    class_source: newClassSource.value,
    level: 0,
    subclass_name: null,
    subclass_source: null
  }

  props.levelUp.selectClass(newClass, true)
}

onMounted(() => {
  loadAvailableClasses()
})
</script>

<style scoped>
.class-selection-step {
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

.class-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 500;
  color: var(--color-text);
}

.class-grid,
.multiclass-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--spacing-md);
}

.class-card,
.multiclass-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-surface);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: center;
}

.class-card:hover,
.multiclass-card:hover:not(.disabled) {
  border-color: var(--color-primary-300);
  background: var(--color-surface-variant);
}

.class-card.selected,
.multiclass-card.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-hover);
}

.multiclass-card.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.class-name,
.multiclass-name {
  font-weight: 600;
  font-size: 1.125rem;
  color: var(--color-text);
}

.class-level,
.multiclass-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.multiclass-hitdie {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-primary-500);
}

.class-subclass {
  font-size: 0.75rem;
  color: var(--color-primary-500);
  font-style: italic;
}

.class-next {
  margin-top: var(--spacing-xs);
  font-size: 0.875rem;
  color: var(--color-success, #22c55e);
}

.arrow {
  font-weight: bold;
}

.prereq-section {
  margin-top: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  width: 100%;
}

.prereq-section.met {
  background: var(--color-success-bg, #f0fdf4);
  color: var(--color-success, #22c55e);
}

.prereq-section.unmet {
  background: var(--color-error-bg, #fef2f2);
  color: var(--color-error, #dc2626);
}

.prereq-label {
  font-weight: 500;
}

.prereq-value {
  margin-left: var(--spacing-xs);
}

.prereq-missing {
  display: block;
  font-size: 0.7rem;
  opacity: 0.8;
}

.loading-indicator {
  padding: var(--spacing-lg);
  text-align: center;
  color: var(--color-text-secondary);
}

.multiclass-input {
  display: flex;
  gap: var(--spacing-md);
  align-items: flex-end;
  flex-wrap: wrap;
}

.multiclass-input .form-group {
  flex: 1;
  min-width: 150px;
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

.add-class-btn {
  white-space: nowrap;
}

.new-class-preview {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.preview-label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.preview-class {
  font-weight: 600;
  color: var(--color-primary-500);
}

.preview-level,
.preview-hitdie {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.level-summary {
  display: flex;
  gap: var(--spacing-xl);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.summary-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.summary-label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.summary-value {
  font-weight: 600;
  font-size: 1.125rem;
}

.summary-value.highlight {
  color: var(--color-primary-500);
}
</style>
