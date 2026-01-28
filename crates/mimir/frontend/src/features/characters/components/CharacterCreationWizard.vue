<template>
  <AppModal
    :visible="visible"
    :title="isNpc ? 'Create NPC' : 'Create Character'"
    :size="isNpc ? 'md' : 'lg'"
    :closable="!creating"
    :close-on-overlay="!creating"
    :close-on-escape="!creating"
    @close="handleClose"
  >
    <template v-if="!isNpc && steps.length > 1" #header>
      <h2 class="modal-title">Create Character</h2>
      <div class="wizard-progress">
        <div
          v-for="(step, index) in steps"
          :key="step.id"
          class="progress-step"
          :class="{ active: currentStep === index, completed: currentStep > index }"
        >
          <div class="step-number">{{ index + 1 }}</div>
          <div class="step-label">{{ step.label }}</div>
        </div>
      </div>
    </template>

    <div class="wizard-body">
      <!-- Step: Basics (always first for PCs, only step for NPCs) -->
      <div v-if="isNpc || currentStepId === 'basics'" class="wizard-step">
        <template v-if="!isNpc">
          <h3>Basic Information</h3>
          <p class="step-description">Character name and player details</p>
        </template>

        <!-- Character Type Selection (hidden when pcOnly or npcOnly) -->
        <div v-if="!pcOnly && !npcOnly" class="form-group">
          <label class="form-label">Character Type</label>
          <div class="type-buttons">
            <button
              type="button"
              class="type-button"
              :class="{ active: !isNpc }"
              @click="setCharacterType(false)"
            >
              Player Character
            </button>
            <button
              type="button"
              class="type-button"
              :class="{ active: isNpc }"
              @click="setCharacterType(true)"
            >
              NPC
            </button>
          </div>
        </div>

        <!-- Name -->
        <div class="form-group">
          <label class="form-label" for="name">Name *</label>
          <input
            id="name"
            v-model="formData.name"
            type="text"
            class="form-input"
            placeholder="Character name"
            required
          />
        </div>

        <!-- Player Name (PC only) -->
        <div v-if="!isNpc" class="form-group">
          <label class="form-label" for="player_name">Player Name *</label>
          <input
            id="player_name"
            v-model="formData.player_name"
            type="text"
            class="form-input"
            placeholder="Player's name"
            required
          />
        </div>

        <!-- Source Selection (PC only) -->
        <div v-if="!isNpc" class="form-group">
          <label class="form-label">Sources</label>
          <div class="source-chips">
            <button
              v-for="source in characterSources"
              :key="source.id"
              type="button"
              class="source-chip"
              :class="{ selected: selectedSources.includes(source.id) }"
              @click="toggleSource(source.id)"
            >
              <span class="source-chip-check">{{ selectedSources.includes(source.id) ? '✓' : '' }}</span>
              <span class="source-chip-text">
                <span class="source-chip-name">{{ source.name }}</span>
                <span class="source-chip-tags">
                  <span v-if="source.has.races" class="source-tag">R</span>
                  <span v-if="source.has.classes" class="source-tag">C</span>
                  <span v-if="source.has.backgrounds" class="source-tag">B</span>
                </span>
              </span>
            </button>
          </div>
          <div class="source-footer">
            <p v-if="selectedSources.length === 0" class="field-hint">All sources enabled</p>
            <p v-else-if="!sourceCoverage.complete" class="source-warning">
              Missing:
              <span v-if="!sourceCoverage.races">races</span>
              <span v-if="!sourceCoverage.races && (!sourceCoverage.classes || !sourceCoverage.backgrounds)">, </span>
              <span v-if="!sourceCoverage.classes">classes</span>
              <span v-if="!sourceCoverage.classes && !sourceCoverage.backgrounds">, </span>
              <span v-if="!sourceCoverage.backgrounds">backgrounds</span>
              &mdash; select more sources
            </p>
            <div class="source-legend">
              <span class="source-legend-item"><span class="source-tag legend">R</span> Races</span>
              <span class="source-legend-item"><span class="source-tag legend">C</span> Classes</span>
              <span class="source-legend-item"><span class="source-tag legend">B</span> Backgrounds</span>
            </div>
          </div>
        </div>

        <!-- NPC-specific fields (shown inline for NPCs) -->
        <template v-if="isNpc">
          <div class="form-group">
            <label class="form-label" for="race">Race</label>
            <input
              id="race"
              v-model="formData.race_name"
              type="text"
              class="form-input"
              placeholder="e.g., Human, Elf, Dwarf"
            />
          </div>
          <div class="form-group">
            <label class="form-label" for="role">Role</label>
            <input
              id="role"
              v-model="formData.role"
              type="text"
              class="form-input"
              placeholder="e.g., Merchant, Guard, Wizard"
            />
          </div>
          <div class="form-group">
            <label class="form-label" for="location">Location</label>
            <input
              id="location"
              v-model="formData.location"
              type="text"
              class="form-input"
              placeholder="e.g., Tavern, Castle, Forest"
            />
          </div>
          <div class="form-group">
            <label class="form-label" for="faction">Faction</label>
            <input
              id="faction"
              v-model="formData.faction"
              type="text"
              class="form-input"
              placeholder="e.g., Thieves Guild, Royal Guard"
            />
          </div>
        </template>
      </div>

      <!-- Step: Race Selection (PC only) -->
      <div v-if="!isNpc && currentStepId === 'race'" class="wizard-step">
        <h3>Race</h3>
        <p class="step-description">Choose your character's race</p>
        <div class="form-group">
          <label class="form-label">Race *</label>
          <select v-model="formData.race_name" class="form-select">
            <option value="">-- Select a Race --</option>
            <option v-for="race in catalogRaces" :key="`${race.name}-${race.source}`" :value="race.name">
              {{ race.name }} ({{ race.source }})
            </option>
          </select>
        </div>
        <div v-if="selectedRaceSource" class="selection-detail">
          Selected: <strong>{{ formData.race_name }}</strong> ({{ selectedRaceSource }})
        </div>
      </div>

      <!-- Step: Class Selection (PC only) -->
      <div v-if="!isNpc && currentStepId === 'class'" class="wizard-step">
        <h3>Class</h3>
        <p class="step-description">Choose your character's class</p>
        <div class="form-group">
          <label class="form-label">Class *</label>
          <select v-model="formData.class_name" class="form-select">
            <option value="">-- Select a Class --</option>
            <option v-for="cls in catalogClasses" :key="`${cls.name}-${cls.source}`" :value="cls.name">
              {{ cls.name }} ({{ cls.source }})
            </option>
          </select>
        </div>
        <div v-if="selectedClassSource" class="selection-detail">
          Selected: <strong>{{ formData.class_name }}</strong> ({{ selectedClassSource }})
        </div>
      </div>

      <!-- Step: Background Selection (PC only) -->
      <div v-if="!isNpc && currentStepId === 'background'" class="wizard-step">
        <h3>Background</h3>
        <p class="step-description">Choose your character's background</p>
        <div class="form-group">
          <label class="form-label">Background *</label>
          <select v-model="formData.background_name" class="form-select">
            <option value="">-- Select a Background --</option>
            <option v-for="bg in catalogBackgrounds" :key="`${bg.name}-${bg.source}`" :value="bg.name">
              {{ bg.name }} ({{ bg.source }})
            </option>
          </select>
        </div>
        <div v-if="selectedBackgroundSource" class="selection-detail">
          Selected: <strong>{{ formData.background_name }}</strong> ({{ selectedBackgroundSource }})
        </div>
      </div>

      <!-- Step: Ability Scores (PC only) -->
      <div v-if="!isNpc && currentStepId === 'abilities'" class="wizard-step">
        <h3>Ability Scores</h3>
        <p class="step-description">Assign your character's ability scores</p>

        <div class="ability-controls">
          <div class="form-group">
            <label class="form-label">Method</label>
            <select v-model="abilityScoreMethod" class="form-select">
              <option value="standard">Standard Array (15, 14, 13, 12, 10, 8)</option>
              <option value="point-buy">Point Buy (27 points)</option>
              <option value="manual">Manual Entry</option>
            </select>
          </div>
        </div>

        <table class="ability-scores-table">
          <thead>
            <tr>
              <th>Ability</th>
              <th>Score</th>
              <th>Modifier</th>
              <th v-if="abilityScoreMethod === 'point-buy'">Cost</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="ability in abilityNames" :key="ability.key">
              <td class="ability-name">{{ ability.label }}</td>
              <td class="ability-score">
                <!-- Standard Array: dropdown -->
                <select
                  v-if="abilityScoreMethod === 'standard'"
                  v-model.number="formData.abilities[ability.key]"
                  class="form-select form-select-sm"
                >
                  <option :value="0">-</option>
                  <option
                    v-for="score in availableStandardScores(ability.key)"
                    :key="score"
                    :value="score"
                  >{{ score }}</option>
                </select>

                <!-- Point Buy / Manual: increment/decrement -->
                <div v-else class="ability-incrementer">
                  <button
                    type="button"
                    class="increment-btn"
                    :disabled="!canDecrement(ability.key)"
                    @click="decrementAbility(ability.key)"
                  >-</button>
                  <span class="ability-value">{{ formData.abilities[ability.key] }}</span>
                  <button
                    type="button"
                    class="increment-btn"
                    :disabled="!canIncrement(ability.key)"
                    @click="incrementAbility(ability.key)"
                  >+</button>
                </div>
              </td>
              <td class="ability-modifier">
                {{ formatModifier(getModifier(formData.abilities[ability.key])) }}
              </td>
              <td v-if="abilityScoreMethod === 'point-buy'" class="ability-cost">
                {{ pointBuyCost(formData.abilities[ability.key]) }}
              </td>
            </tr>
          </tbody>
        </table>

        <div v-if="abilityScoreMethod === 'point-buy'" class="point-buy-remaining">
          Points remaining: <strong>{{ pointBuyRemaining }}</strong> / 27
        </div>
      </div>

      <!-- Step: Skills (PC only) -->
      <div v-if="!isNpc && currentStepId === 'skills'" class="wizard-step">
        <h3>Skill Proficiencies</h3>
        <p class="step-description">Choose your skill proficiencies</p>

        <!-- Background skills (locked) -->
        <div v-if="backgroundSkills.length > 0" class="skills-section">
          <h4 class="skills-section-header">From Background ({{ formData.background_name }})</h4>
          <div class="skill-chips">
            <span v-for="skill in backgroundSkills" :key="skill" class="skill-chip locked">
              {{ skill }}
            </span>
          </div>
        </div>

        <!-- Class skill choices -->
        <div v-if="classSkillChoices.count > 0" class="skills-section">
          <h4 class="skills-section-header">
            Choose {{ classSkillChoices.count }} from {{ formData.class_name }}
            <span class="skills-counter">({{ selectedClassSkills.length }}/{{ classSkillChoices.count }})</span>
          </h4>

          <div v-for="group in availableSkillGroups" :key="group.ability" class="skill-group">
            <h5 class="ability-group-header">{{ group.ability }}</h5>
            <div class="skill-chips">
              <button
                v-for="skill in group.skills"
                :key="skill.name"
                type="button"
                class="skill-chip selectable"
                :class="{
                  selected: isClassSkillSelected(skill.name),
                  disabled: atClassSkillLimit && !isClassSkillSelected(skill.name),
                  'from-background': isBackgroundSkill(skill.name)
                }"
                :disabled="isBackgroundSkill(skill.name)"
                @click="toggleClassSkill(skill.name)"
              >
                <span class="skill-chip-name">{{ skill.name }}</span>
                <span class="skill-chip-desc">{{ skill.description }}</span>
                <span v-if="isBackgroundSkill(skill.name)" class="skill-badge">Background</span>
              </button>
            </div>
          </div>
        </div>

        <div v-if="classSkillChoices.count === 0 && backgroundSkills.length === 0" class="empty-skills">
          Select a class and background first to see available skills.
        </div>
      </div>

      <!-- Step: Review (PC only) -->
      <div v-if="!isNpc && currentStepId === 'review'" class="wizard-step">
        <h3>Review</h3>
        <p class="step-description">Review your character before creating</p>

        <div class="review-section">
          <div class="review-item">
            <span class="review-label">Name:</span>
            <span class="review-value">{{ formData.name }}</span>
          </div>
          <div class="review-item">
            <span class="review-label">Player:</span>
            <span class="review-value">{{ formData.player_name }}</span>
          </div>
          <div v-if="formData.race_name" class="review-item">
            <span class="review-label">Race:</span>
            <span class="review-value">{{ formData.race_name }}</span>
          </div>
          <div v-if="formData.class_name" class="review-item">
            <span class="review-label">Class:</span>
            <span class="review-value">{{ formData.class_name }}</span>
          </div>
          <div v-if="formData.background_name" class="review-item">
            <span class="review-label">Background:</span>
            <span class="review-value">{{ formData.background_name }}</span>
          </div>
          <div class="review-item">
            <span class="review-label">Abilities:</span>
            <span class="review-value">
              STR {{ formData.abilities.strength }} |
              DEX {{ formData.abilities.dexterity }} |
              CON {{ formData.abilities.constitution }} |
              INT {{ formData.abilities.intelligence }} |
              WIS {{ formData.abilities.wisdom }} |
              CHA {{ formData.abilities.charisma }}
            </span>
          </div>
          <div v-if="allSelectedSkills.length > 0" class="review-item">
            <span class="review-label">Skills:</span>
            <span class="review-value">
              <template v-if="backgroundSkills.length > 0">
                <span class="review-skill-source">Background:</span> {{ backgroundSkills.join(', ') }}
                <template v-if="selectedClassSkills.length > 0">
                  <br><span class="review-skill-source">Class:</span> {{ selectedClassSkills.join(', ') }}
                </template>
              </template>
              <template v-else>
                {{ allSelectedSkills.join(', ') }}
              </template>
            </span>
          </div>
        </div>
      </div>

      <!-- Error display -->
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <template #footer>
      <!-- NPC: simple Cancel/Create -->
      <template v-if="isNpc">
        <button type="button" @click="handleClose" class="btn btn-secondary">Cancel</button>
        <button
          type="button"
          @click="createCharacter"
          class="btn btn-primary"
          :disabled="!canCreate || creating"
        >
          {{ creating ? 'Creating...' : 'Create NPC' }}
        </button>
      </template>

      <!-- PC: wizard navigation -->
      <template v-else>
        <button
          v-if="currentStep > 0"
          type="button"
          @click="previousStep"
          class="btn btn-secondary"
          :disabled="creating"
        >
          Back
        </button>
        <div class="footer-spacer"></div>
        <button
          v-if="currentStep < steps.length - 1"
          type="button"
          @click="nextStep"
          class="btn btn-primary"
          :disabled="!canProceed || creating"
        >
          Next
        </button>
        <button
          v-if="currentStep === steps.length - 1"
          type="button"
          @click="createCharacter"
          class="btn btn-primary"
          :disabled="!canCreate || creating"
        >
          {{ creating ? 'Creating...' : 'Create Character' }}
        </button>
      </template>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import { useCharacterStore } from '@/stores/characters'
import type { ApiResponse } from '@/types/api'

// --- Props & Emits ---

const props = defineProps<{
  visible: boolean
  campaignId?: string
  startAsNpc?: boolean
  pcOnly?: boolean
  npcOnly?: boolean
}>()

const emit = defineEmits<{
  close: []
  created: [characterId: string]
}>()

// --- Stores ---

const characterStore = useCharacterStore()

// --- State ---

const isNpc = ref(props.startAsNpc ?? false)
const currentStep = ref(0)
const creating = ref(false)
const error = ref<string | null>(null)
const abilityScoreMethod = ref<'standard' | 'point-buy' | 'manual'>('standard')

// Catalog data
interface CatalogItem {
  name: string
  source: string
}

const catalogRacesAll = ref<CatalogItem[]>([])
const catalogClassesAll = ref<CatalogItem[]>([])
const catalogBackgroundsAll = ref<CatalogItem[]>([])

// Filtered by selected sources
const catalogRaces = computed(() =>
  selectedSources.value.length > 0
    ? catalogRacesAll.value.filter(r => selectedSources.value.includes(r.source))
    : catalogRacesAll.value
)
const catalogClasses = computed(() =>
  selectedSources.value.length > 0
    ? catalogClassesAll.value.filter(c => selectedSources.value.includes(c.source))
    : catalogClassesAll.value
)
const catalogBackgrounds = computed(() =>
  selectedSources.value.length > 0
    ? catalogBackgroundsAll.value.filter(b => selectedSources.value.includes(b.source))
    : catalogBackgroundsAll.value
)

// --- Source Selection ---

interface SourceOption {
  id: string
  name: string
  has: { races: boolean; classes: boolean; backgrounds: boolean }
}

const characterSources = ref<SourceOption[]>([])
const selectedSources = ref<string[]>([])

// Check that selected sources collectively cover races, classes, and backgrounds
const sourceCoverage = computed(() => {
  if (selectedSources.value.length === 0) return { races: true, classes: true, backgrounds: true, complete: true }
  const selected = characterSources.value.filter(s => selectedSources.value.includes(s.id))
  const races = selected.some(s => s.has.races)
  const classes = selected.some(s => s.has.classes)
  const backgrounds = selected.some(s => s.has.backgrounds)
  return { races, classes, backgrounds, complete: races && classes && backgrounds }
})

function toggleSource(sourceId: string) {
  const idx = selectedSources.value.indexOf(sourceId)
  if (idx === -1) {
    selectedSources.value.push(sourceId)
  } else {
    selectedSources.value.splice(idx, 1)
  }
  clearCatalogSelections()
}

function clearCatalogSelections() {
  formData.value.race_name = ''
  formData.value.race_source = ''
  formData.value.class_name = ''
  formData.value.class_source = ''
  formData.value.background_name = ''
  formData.value.background_source = ''
  formData.value.skills = []
  selectedClassDetails.value = null
  selectedBackgroundDetails.value = null
}


type AbilityKey = 'strength' | 'dexterity' | 'constitution' | 'intelligence' | 'wisdom' | 'charisma'

const abilityNames: { key: AbilityKey; label: string }[] = [
  { key: 'strength', label: 'STR' },
  { key: 'dexterity', label: 'DEX' },
  { key: 'constitution', label: 'CON' },
  { key: 'intelligence', label: 'INT' },
  { key: 'wisdom', label: 'WIS' },
  { key: 'charisma', label: 'CHA' },
]

const standardArray = [15, 14, 13, 12, 10, 8]

const formData = ref({
  name: '',
  player_name: '',
  race_name: '',
  race_source: '',
  class_name: '',
  class_source: '',
  background_name: '',
  background_source: '',
  abilities: {
    strength: 10,
    dexterity: 10,
    constitution: 10,
    intelligence: 10,
    wisdom: 10,
    charisma: 10,
  } as Record<AbilityKey, number>,
  skills: [] as string[],
  // NPC fields
  role: '',
  location: '',
  faction: '',
})

// --- Steps ---

interface WizardStep {
  id: string
  label: string
}

const steps = computed((): WizardStep[] => {
  if (isNpc.value) {
    return [{ id: 'basics', label: 'Basics' }]
  }
  const s: WizardStep[] = [
    { id: 'basics', label: 'Basics' },
    { id: 'race', label: 'Race' },
    { id: 'class', label: 'Class' },
    { id: 'background', label: 'Background' },
    { id: 'abilities', label: 'Abilities' },
    { id: 'skills', label: 'Skills' },
    { id: 'review', label: 'Review' },
  ]
  return s
})

const currentStepId = computed(() => steps.value[currentStep.value]?.id ?? 'basics')

// --- Navigation ---

const canProceed = computed(() => {
  const stepId = currentStepId.value
  switch (stepId) {
    case 'basics':
      if (!formData.value.name.trim()) return false
      if (!isNpc.value && !formData.value.player_name.trim()) return false
      if (!isNpc.value && !sourceCoverage.value.complete) return false
      return true
    case 'race':
      return !!formData.value.race_name
    case 'class':
      return !!formData.value.class_name
    case 'background':
      return !!formData.value.background_name
    case 'abilities':
      return validateAbilities()
    case 'skills':
      return classSkillChoices.value.count === 0 || selectedClassSkills.value.length === classSkillChoices.value.count
    case 'spells':
      return true // TODO: validate spell selections
    case 'review':
      return true
    default:
      return true
  }
})

const canCreate = computed(() => {
  if (isNpc.value) {
    return !!formData.value.name.trim()
  }
  // PC: must be on review step and have basics filled
  return currentStepId.value === 'review'
    && !!formData.value.name.trim()
    && !!formData.value.player_name.trim()
})

const nextStep = () => {
  if (canProceed.value && currentStep.value < steps.value.length - 1) {
    currentStep.value++
  }
}

const previousStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

// --- Character type switching ---

const setCharacterType = (npc: boolean) => {
  isNpc.value = npc
  currentStep.value = 0
}

// When switching to NPC, reset step
watch(isNpc, () => {
  currentStep.value = 0
})

// --- Catalog sources for selected items ---

const selectedRaceSource = computed(() => {
  const race = catalogRaces.value.find(r => r.name === formData.value.race_name)
  if (race) {
    formData.value.race_source = race.source
    return race.source
  }
  return ''
})

const selectedClassSource = computed(() => {
  const cls = catalogClasses.value.find(c => c.name === formData.value.class_name)
  if (cls) {
    formData.value.class_source = cls.source
    return cls.source
  }
  return ''
})

const selectedBackgroundSource = computed(() => {
  const bg = catalogBackgrounds.value.find(b => b.name === formData.value.background_name)
  if (bg) {
    formData.value.background_source = bg.source
    return bg.source
  }
  return ''
})

// --- Ability Score Logic ---

function validateAbilities(): boolean {
  const abilities = formData.value.abilities
  if (abilityScoreMethod.value === 'standard') {
    // All 6 must be assigned (non-zero) and distinct
    const values = Object.values(abilities)
    return values.every(v => v > 0) && new Set(values).size === 6
  }
  if (abilityScoreMethod.value === 'point-buy') {
    return pointBuyRemaining.value === 0
  }
  // Manual: just check 3-18 range
  return Object.values(abilities).every(v => v >= 3 && v <= 18)
}

function availableStandardScores(currentKey: AbilityKey): number[] {
  const used = new Set<number>()
  for (const ability of abilityNames) {
    if (ability.key !== currentKey && formData.value.abilities[ability.key] > 0) {
      used.add(formData.value.abilities[ability.key])
    }
  }
  return standardArray.filter(s => !used.has(s) || formData.value.abilities[currentKey] === s)
}

function pointBuyCost(score: number): number {
  if (score <= 8) return 0
  if (score === 9) return 1
  if (score === 10) return 2
  if (score === 11) return 3
  if (score === 12) return 4
  if (score === 13) return 5
  if (score === 14) return 7
  if (score === 15) return 9
  return 0
}

const pointBuyRemaining = computed(() => {
  const total = Object.values(formData.value.abilities).reduce((sum, v) => sum + pointBuyCost(v), 0)
  return 27 - total
})

function canIncrement(key: AbilityKey): boolean {
  const val = formData.value.abilities[key]
  if (abilityScoreMethod.value === 'point-buy') {
    return val < 15 && pointBuyRemaining.value > 0
  }
  return val < 18
}

function canDecrement(key: AbilityKey): boolean {
  const val = formData.value.abilities[key]
  if (abilityScoreMethod.value === 'point-buy') {
    return val > 8
  }
  return val > 3
}

function incrementAbility(key: AbilityKey) {
  if (canIncrement(key)) {
    formData.value.abilities[key]++
  }
}

function decrementAbility(key: AbilityKey) {
  if (canDecrement(key)) {
    formData.value.abilities[key]--
  }
}

function getModifier(score: number): number {
  return Math.floor((score - 10) / 2)
}

function formatModifier(mod: number): string {
  return mod >= 0 ? `+${mod}` : `${mod}`
}

// Reset abilities when method changes
watch(abilityScoreMethod, (method) => {
  if (method === 'standard') {
    for (const a of abilityNames) {
      formData.value.abilities[a.key] = 0
    }
  } else if (method === 'point-buy') {
    for (const a of abilityNames) {
      formData.value.abilities[a.key] = 8
    }
  } else {
    for (const a of abilityNames) {
      formData.value.abilities[a.key] = 10
    }
  }
})

// --- Skill Proficiency Logic ---

interface SkillInfo {
  name: string
  description: string
}

interface SkillGroup {
  ability: string
  skills: SkillInfo[]
}

const skillsByAbility: SkillGroup[] = [
  {
    ability: 'Strength',
    skills: [
      { name: 'Athletics', description: 'Climbing, jumping, swimming' },
    ],
  },
  {
    ability: 'Dexterity',
    skills: [
      { name: 'Acrobatics', description: 'Balance, tumbling, aerial maneuvers' },
      { name: 'Sleight of Hand', description: 'Pickpocketing, concealing objects' },
      { name: 'Stealth', description: 'Moving silently and hiding' },
    ],
  },
  {
    ability: 'Intelligence',
    skills: [
      { name: 'Arcana', description: 'Knowledge of spells and magic' },
      { name: 'History', description: 'Lore about events and civilizations' },
      { name: 'Investigation', description: 'Searching for clues, deductions' },
      { name: 'Nature', description: 'Terrain, plants, animals, weather' },
      { name: 'Religion', description: 'Deities, rites, religious traditions' },
    ],
  },
  {
    ability: 'Wisdom',
    skills: [
      { name: 'Animal Handling', description: 'Calming and controlling animals' },
      { name: 'Insight', description: 'Detecting lies, reading intentions' },
      { name: 'Medicine', description: 'Stabilizing the dying, diagnosis' },
      { name: 'Perception', description: 'Spotting, hearing, detecting' },
      { name: 'Survival', description: 'Tracking, hunting, navigating' },
    ],
  },
  {
    ability: 'Charisma',
    skills: [
      { name: 'Deception', description: 'Lying and misleading' },
      { name: 'Intimidation', description: 'Threats and hostility' },
      { name: 'Performance', description: 'Music, dance, acting' },
      { name: 'Persuasion', description: 'Tact and social grace' },
    ],
  },
]

// Class/background detail data for skill proficiencies
const selectedClassDetails = ref<any>(null)
const selectedBackgroundDetails = ref<any>(null)

// Capitalize a skill name from lowercase 5etools format
function capitalizeSkill(s: string): string {
  return s.split(' ').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
}

// Background skills (locked, auto-granted)
const backgroundSkills = computed<string[]>(() => {
  if (!selectedBackgroundDetails.value?.skillProficiencies) return []
  const skills: string[] = []
  for (const profGroup of selectedBackgroundDetails.value.skillProficiencies) {
    for (const [skill, granted] of Object.entries(profGroup)) {
      if (granted === true) {
        skills.push(capitalizeSkill(skill))
      }
    }
  }
  return skills
})

// Class skill choices (count + available list)
const classSkillChoices = computed(() => {
  if (!selectedClassDetails.value?.startingProficiencies?.skills) {
    return { count: 0, from: [] as string[] }
  }
  const skillDef = selectedClassDetails.value.startingProficiencies.skills[0]
  if (skillDef?.choose) {
    return {
      count: skillDef.choose.count,
      from: skillDef.choose.from.map((s: string) => capitalizeSkill(s)),
    }
  }
  return { count: 0, from: [] as string[] }
})

// Only show skill groups that have available class skills
const availableSkillGroups = computed(() => {
  return skillsByAbility
    .map(group => ({
      ...group,
      skills: group.skills.filter(s => classSkillChoices.value.from.includes(s.name)),
    }))
    .filter(group => group.skills.length > 0)
})

const selectedClassSkills = computed(() => {
  return formData.value.skills.filter(s => !backgroundSkills.value.includes(s))
})

const allSelectedSkills = computed(() => {
  return [...backgroundSkills.value, ...selectedClassSkills.value]
})

const atClassSkillLimit = computed(() => {
  return selectedClassSkills.value.length >= classSkillChoices.value.count
})

function isBackgroundSkill(name: string): boolean {
  return backgroundSkills.value.includes(name)
}

function isClassSkillSelected(name: string): boolean {
  return formData.value.skills.includes(name)
}

function toggleClassSkill(name: string) {
  if (isBackgroundSkill(name)) return
  const idx = formData.value.skills.indexOf(name)
  if (idx === -1) {
    if (!atClassSkillLimit.value) {
      formData.value.skills.push(name)
    }
  } else {
    formData.value.skills.splice(idx, 1)
  }
}

// Fetch class details when class changes
watch(() => formData.value.class_name, async (className) => {
  if (!className) {
    selectedClassDetails.value = null
    return
  }
  const source = catalogClasses.value.find(c => c.name === className)?.source
  if (!source) return
  try {
    const res = await invoke<ApiResponse<any>>('get_class_by_name', { name: className, source })
    if (res.success && res.data) {
      selectedClassDetails.value = res.data
    }
  } catch (e) {
    console.error('Failed to fetch class details:', e)
  }
})

// Fetch background details when background changes
watch(() => formData.value.background_name, async (bgName) => {
  if (!bgName) {
    selectedBackgroundDetails.value = null
    return
  }
  const source = catalogBackgrounds.value.find(b => b.name === bgName)?.source
  if (!source) return
  try {
    const res = await invoke<ApiResponse<any>>('get_background_by_name', { name: bgName, source })
    if (res.success && res.data) {
      selectedBackgroundDetails.value = res.data
    }
  } catch (e) {
    console.error('Failed to fetch background details:', e)
  }
})

// Clear skill selections when class/background changes
watch([() => formData.value.class_name, () => formData.value.background_name], () => {
  formData.value.skills = []
})

// --- Catalog loading ---

async function loadCatalogs() {
  try {
    const [racesRes, classesRes, backgroundsRes, raceSourcesRes, classSourcesRes, bgSourcesRes, allSourcesRes] = await Promise.all([
      invoke<ApiResponse<CatalogItem[]>>('search_races', { filter: {}, limit: 500, offset: 0 }),
      invoke<ApiResponse<CatalogItem[]>>('search_classes', { filter: {}, limit: 500, offset: 0 }),
      invoke<ApiResponse<CatalogItem[]>>('search_backgrounds', { filter: {}, limit: 500, offset: 0 }),
      invoke<ApiResponse<string[]>>('list_race_sources'),
      invoke<ApiResponse<string[]>>('list_class_sources'),
      invoke<ApiResponse<string[]>>('list_background_sources'),
      invoke<ApiResponse<SourceOption[]>>('list_catalog_sources'),
    ])

    if (racesRes.success && racesRes.data) {
      catalogRacesAll.value = racesRes.data
        .map(r => ({ name: r.name, source: r.source }))
        .sort((a, b) => a.name.localeCompare(b.name))
    }
    if (classesRes.success && classesRes.data) {
      // Filter to base classes only (no subclasses)
      catalogClassesAll.value = classesRes.data
        .filter((c: any) => !c.subclass_name && !c.subclassName)
        .map(c => ({ name: c.name, source: c.source }))
        .sort((a, b) => a.name.localeCompare(b.name))
    }
    if (backgroundsRes.success && backgroundsRes.data) {
      catalogBackgroundsAll.value = backgroundsRes.data
        .map(b => ({ name: b.name, source: b.source }))
        .sort((a, b) => a.name.localeCompare(b.name))
    }

    // Build per-source capability map
    const raceSources = new Set(raceSourcesRes.success && raceSourcesRes.data ? raceSourcesRes.data : [])
    const classSources = new Set(classSourcesRes.success && classSourcesRes.data ? classSourcesRes.data : [])
    const bgSources = new Set(bgSourcesRes.success && bgSourcesRes.data ? bgSourcesRes.data : [])
    const allCodes = new Set([...raceSources, ...classSources, ...bgSources])

    // Cross-reference with catalog sources to get display names
    if (allSourcesRes.success && allSourcesRes.data) {
      const sourceMap = new Map<string, string>()
      for (const s of allSourcesRes.data as any[]) {
        const id = s.id || s.code
        sourceMap.set(id, s.name)
      }
      characterSources.value = Array.from(allCodes)
        .map(code => ({
          id: code,
          name: sourceMap.get(code) || code,
          has: {
            races: raceSources.has(code),
            classes: classSources.has(code),
            backgrounds: bgSources.has(code),
          },
        }))
        .sort((a, b) => a.name.localeCompare(b.name))
    }
  } catch (e) {
    console.error('Failed to load catalogs:', e)
  }
}

// Load catalogs when modal opens
watch(() => props.visible, (visible) => {
  if (visible) {
    loadCatalogs()
  }
}, { immediate: true })

// --- Form management ---

const handleClose = () => {
  if (!creating.value) {
    resetForm()
    emit('close')
  }
}

const resetForm = () => {
  isNpc.value = props.startAsNpc ?? false
  currentStep.value = 0
  abilityScoreMethod.value = 'standard'
  formData.value = {
    name: '',
    player_name: '',
    race_name: '',
    race_source: '',
    class_name: '',
    class_source: '',
    background_name: '',
    background_source: '',
    abilities: {
      strength: 0,
      dexterity: 0,
      constitution: 0,
      intelligence: 0,
      wisdom: 0,
      charisma: 0,
    },
    skills: [],
    role: '',
    location: '',
    faction: '',
  }
  selectedClassDetails.value = null
  selectedBackgroundDetails.value = null
  error.value = null
}

const createCharacter = async () => {
  if (isNpc.value && !formData.value.name.trim()) return
  if (!isNpc.value && !canCreate.value) return

  creating.value = true
  error.value = null

  try {
    let character
    if (isNpc.value) {
      character = await characterStore.createNpc({
        campaign_id: props.campaignId,
        name: formData.value.name.trim(),
        race_name: formData.value.race_name.trim() || undefined,
        role: formData.value.role.trim() || undefined,
        location: formData.value.location.trim() || undefined,
        faction: formData.value.faction.trim() || undefined,
      })
    } else {
      character = await characterStore.createPc({
        campaign_id: props.campaignId,
        name: formData.value.name.trim(),
        player_name: formData.value.player_name.trim(),
        race_name: formData.value.race_name.trim() || undefined,
        race_source: formData.value.race_source.trim() || undefined,
        background_name: formData.value.background_name.trim() || undefined,
        background_source: formData.value.background_source.trim() || undefined,
        ability_scores: [
          formData.value.abilities.strength,
          formData.value.abilities.dexterity,
          formData.value.abilities.constitution,
          formData.value.abilities.intelligence,
          formData.value.abilities.wisdom,
          formData.value.abilities.charisma,
        ],
      })
    }

    if (character) {
      emit('created', character.id)
      resetForm()
      emit('close')
    } else {
      error.value = characterStore.error || 'Failed to create character'
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to create character'
  } finally {
    creating.value = false
  }
}
</script>

<style scoped>
.wizard-body {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  min-height: 300px;
}

.wizard-step h3 {
  margin: 0 0 var(--spacing-xs) 0;
  font-size: 1.1rem;
}

.step-description {
  color: var(--color-text-muted);
  font-size: 0.875rem;
  margin: 0 0 var(--spacing-md) 0;
}

/* Progress bar */
.wizard-progress {
  display: flex;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) 0;
  overflow-x: auto;
}

.progress-step {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-md);
  font-size: 0.75rem;
  white-space: nowrap;
  color: var(--color-text-muted);
}

.progress-step.active {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  font-weight: 600;
}

.progress-step.completed {
  color: var(--color-success, #16a34a);
}

.step-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 1.5px solid currentColor;
  font-size: 0.7rem;
  font-weight: 600;
  flex-shrink: 0;
}

.progress-step.completed .step-number {
  background: var(--color-success, #16a34a);
  color: white;
  border-color: var(--color-success, #16a34a);
}

.progress-step.active .step-number {
  background: var(--color-primary-500);
  color: white;
  border-color: var(--color-primary-500);
}

/* Form elements */
.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.form-label {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--color-text);
}

.form-input,
.form-select {
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 1rem;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}

.form-select-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
}

/* Type toggle */
.type-buttons {
  display: flex;
  gap: var(--spacing-sm);
}

.type-button {
  flex: 1;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-base);
}

.type-button:hover {
  background: var(--color-surface-variant);
}

.type-button.active {
  background: var(--color-primary-500);
  color: white;
  border-color: var(--color-primary-500);
}

/* Source selection */
.field-hint {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin: 0;
}

.source-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.source-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-full, 999px);
  background: var(--color-surface);
  color: var(--color-text-muted);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s ease;
  line-height: 1.3;
}

.source-chip:hover {
  border-color: var(--color-primary-400);
  color: var(--color-text);
}

.source-chip.selected {
  background: var(--color-primary-500);
  border-color: var(--color-primary-500);
  color: white;
}

.source-chip-check {
  width: 10px;
  font-size: 0.65rem;
  font-weight: 700;
}

.source-chip-text {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.source-chip-name {
  font-weight: 500;
}

.source-chip-tags {
  display: inline-flex;
  gap: 2px;
}

.source-tag {
  display: inline-block;
  width: 14px;
  height: 14px;
  line-height: 14px;
  text-align: center;
  border-radius: 3px;
  font-size: 0.55rem;
  font-weight: 700;
  background: var(--color-surface-variant);
  color: var(--color-text-muted);
}

.source-chip.selected .source-tag {
  background: rgba(255, 255, 255, 0.25);
  color: white;
}

.source-footer {
  margin-top: 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
}

.source-warning {
  color: var(--color-warning, #c47a00);
  font-size: 0.75rem;
  margin: 0;
}

.source-legend {
  display: flex;
  gap: var(--spacing-sm);
  font-size: 0.65rem;
  color: var(--color-text-muted);
}

.source-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}

.source-tag.legend {
  opacity: 0.7;
}

/* Selection detail */
.selection-detail {
  margin-top: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

/* Ability scores table */
.ability-scores-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

.ability-scores-table th {
  text-align: left;
  padding: var(--spacing-sm);
  border-bottom: 2px solid var(--color-border);
  font-size: 0.8rem;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.ability-scores-table td {
  padding: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.ability-name {
  font-weight: 600;
}

.ability-modifier {
  color: var(--color-text-muted);
}

.ability-incrementer {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.increment-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
  font-size: 1rem;
  font-weight: 600;
}

.increment-btn:hover:not(:disabled) {
  background: var(--color-surface-variant);
}

.increment-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.ability-value {
  min-width: 24px;
  text-align: center;
  font-weight: 600;
}

.point-buy-remaining {
  margin-top: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  text-align: center;
  font-size: 0.9rem;
}

/* Review */
.review-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.review-item {
  display: flex;
  gap: var(--spacing-md);
  padding: var(--spacing-xs) 0;
  border-bottom: 1px solid var(--color-border);
}

.review-label {
  font-weight: 600;
  min-width: 100px;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.review-value {
  font-size: 0.875rem;
}

/* Footer */
.footer-spacer {
  flex: 1;
}

/* Error */
.error-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-bg, #fef2f2);
  color: var(--color-error, #dc2626);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
}

/* Skills */
.skills-section {
  margin-bottom: var(--spacing-md);
}

.skills-section-header {
  font-size: 0.9rem;
  font-weight: 600;
  margin: 0 0 var(--spacing-sm) 0;
}

.skills-counter {
  font-weight: 400;
  color: var(--color-text-muted);
  margin-left: var(--spacing-xs);
}

.skill-group {
  margin-bottom: var(--spacing-sm);
}

.ability-group-header {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  margin: 0 0 var(--spacing-xs) 0;
}

.skill-chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}

.skill-chip {
  display: inline-flex;
  flex-direction: column;
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: 0.8rem;
  background: var(--color-surface);
  transition: all var(--transition-base);
}

.skill-chip.locked {
  background: var(--color-surface-variant);
  color: var(--color-text-muted);
  font-style: italic;
}

.skill-chip.selectable {
  cursor: pointer;
}

.skill-chip.selectable:hover:not(.disabled):not(.from-background) {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50, rgba(99, 102, 241, 0.05));
}

.skill-chip.selected {
  background: var(--color-primary-500);
  color: white;
  border-color: var(--color-primary-500);
}

.skill-chip.disabled:not(.selected) {
  opacity: 0.4;
  cursor: not-allowed;
}

.skill-chip.from-background {
  opacity: 0.5;
  cursor: not-allowed;
}

.skill-chip-name {
  font-weight: 600;
}

.skill-chip-desc {
  font-size: 0.7rem;
  opacity: 0.8;
}

.skill-badge {
  font-size: 0.65rem;
  background: var(--color-surface-variant);
  padding: 1px 4px;
  border-radius: var(--radius-sm);
  margin-top: 2px;
}

.empty-skills {
  color: var(--color-text-muted);
  font-style: italic;
  padding: var(--spacing-md);
  text-align: center;
}

.review-skill-source {
  font-weight: 500;
  color: var(--color-text-muted);
}
</style>
