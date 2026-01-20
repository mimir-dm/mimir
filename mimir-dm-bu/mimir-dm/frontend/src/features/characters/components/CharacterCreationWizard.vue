<template>
  <AppModal
    :visible="visible"
    title="Create New Character"
    size="lg"
    :closable="!creating"
    :close-on-overlay="!creating"
    :close-on-escape="!creating"
    @close="closeWizard"
  >
    <template #header>
      <h2>Create New Character</h2>
      <div class="wizard-progress">
        <div
          v-for="(step, index) in steps"
          :key="index"
          class="progress-step"
          :class="{ active: currentStep === index, completed: currentStep > index }"
        >
          <div class="step-number">{{ index + 1 }}</div>
          <div class="step-label">{{ step }}</div>
        </div>
      </div>
    </template>

    <div class="wizard-body">
        <!-- Step 1: Character Type & Player Selection -->
        <div v-if="steps[currentStep] === 'Player'" class="wizard-step">
          <h3>Character Type</h3>
          <p class="step-description">Is this a player character or an NPC?</p>

          <div class="character-type-toggle">
            <button
              type="button"
              class="type-toggle-btn"
              :class="{ active: !formData.is_npc }"
              @click="setCharacterType(false)"
            >
              Player Character
            </button>
            <button
              type="button"
              class="type-toggle-btn"
              :class="{ active: formData.is_npc }"
              @click="setCharacterType(true)"
            >
              NPC
            </button>
          </div>

          <!-- Player selection (for PCs only) -->
          <div v-if="!formData.is_npc" class="form-group">
            <label>Player</label>
            <select v-model="formData.player_id" class="form-select" required>
              <option :value="null">-- Select a Player --</option>
              <option v-for="player in players" :key="player.id" :value="player.id">
                {{ player.name }}
              </option>
            </select>
          </div>

          <!-- NPC fields (for NPCs only) -->
          <div v-else class="npc-fields">
            <div class="form-group">
              <label>Role <span class="optional-label">(optional)</span></label>
              <input
                v-model="formData.npc_role"
                type="text"
                class="form-input"
                placeholder="e.g., Tavern Owner, Town Guard, Villain"
              />
            </div>
            <div class="form-group">
              <label>Location <span class="optional-label">(optional)</span></label>
              <input
                v-model="formData.npc_location"
                type="text"
                class="form-input"
                placeholder="e.g., The Rusty Anchor Tavern, Waterdeep"
              />
            </div>
            <div class="form-group">
              <label>Faction <span class="optional-label">(optional)</span></label>
              <input
                v-model="formData.npc_faction"
                type="text"
                class="form-input"
                placeholder="e.g., Zhentarim, Harpers, City Watch"
              />
            </div>
            <div class="form-group">
              <label>Notes <span class="optional-label">(optional)</span></label>
              <textarea
                v-model="formData.npc_notes"
                class="form-textarea"
                rows="3"
                placeholder="Additional notes about this NPC..."
              ></textarea>
            </div>

            <!-- Legendary Actions (for boss NPCs) -->
            <div class="legendary-actions-section">
              <div class="legendary-header">
                <label>Legendary Actions <span class="optional-label">(optional, for boss NPCs)</span></label>
                <button type="button" class="btn-add-action" @click="addLegendaryAction">
                  + Add Action
                </button>
              </div>

              <div v-if="formData.legendary_actions.length > 0" class="legendary-count-row">
                <label>Actions per Round:</label>
                <select v-model.number="formData.legendary_action_count" class="form-select-sm">
                  <option :value="1">1</option>
                  <option :value="2">2</option>
                  <option :value="3">3</option>
                  <option :value="4">4</option>
                  <option :value="5">5</option>
                </select>
              </div>

              <div v-for="(action, index) in formData.legendary_actions" :key="index" class="legendary-action-card">
                <div class="action-header">
                  <input
                    v-model="action.name"
                    type="text"
                    class="form-input action-name-input"
                    placeholder="Action name"
                  />
                  <div class="action-cost">
                    <label>Cost:</label>
                    <select v-model.number="action.cost" class="form-select-sm">
                      <option :value="1">1</option>
                      <option :value="2">2</option>
                      <option :value="3">3</option>
                    </select>
                  </div>
                  <button type="button" class="btn-remove-action" @click="removeLegendaryAction(index)">
                    x
                  </button>
                </div>
                <textarea
                  v-model="action.description"
                  class="form-textarea action-description"
                  rows="2"
                  placeholder="Describe what this action does..."
                ></textarea>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 2: Basic Info -->
        <div v-if="steps[currentStep] === 'Basic Info'" class="wizard-step">
          <h3>Basic Information</h3>
          <p class="step-description">Character basics - we'll add more details after creation</p>

          <div class="form-group">
            <label>Character Name *</label>
            <input
              v-model="formData.character_name"
              type="text"
              class="form-input"
              placeholder="Enter character name"
              required
            />
          </div>

          <div class="form-group">
            <label>{{ formData.is_npc ? 'Race / Creature Type' : 'Race' }} *</label>
            <select v-model="formData.race" class="form-select" required>
              <option value="">-- Select a {{ formData.is_npc ? 'Race or Creature' : 'Race' }} --</option>
              <optgroup v-if="formData.is_npc" label="Standard Races">
                <option v-for="race in raceOptions.filter(r => !r.isMonster)" :key="`${race.name}-${race.source}`" :value="race.name">
                  {{ race.name }} ({{ race.source }})
                </option>
              </optgroup>
              <optgroup v-if="formData.is_npc" label="Monsters / Creatures">
                <option v-for="race in raceOptions.filter(r => r.isMonster)" :key="`monster-${race.name}-${race.source}`" :value="race.name">
                  {{ race.name }} ({{ race.source }}) - CR {{ race.cr }}
                </option>
              </optgroup>
              <template v-if="!formData.is_npc">
                <option v-for="race in raceOptions" :key="`${race.name}-${race.source}`" :value="race.name">
                  {{ race.name }} ({{ race.source }})
                </option>
              </template>
            </select>
          </div>

          <div class="form-group">
            <label>Class *</label>
            <select v-model="formData.class" class="form-select" required>
              <option value="">-- Select a Class --</option>
              <option v-for="cls in classes" :key="`${cls.name}-${cls.source}`" :value="cls.name">
                {{ cls.name }} ({{ cls.source }})
              </option>
            </select>
          </div>

          <div class="form-group">
            <label>Background *</label>
            <select v-model="formData.background" class="form-select" required>
              <option value="">-- Select a Background --</option>
              <option v-for="bg in backgrounds" :key="`${bg.name}-${bg.source}`" :value="bg.name">
                {{ bg.name }} ({{ bg.source }})
              </option>
            </select>
          </div>
        </div>

        <!-- Step 3: Ability Scores -->
        <div v-if="steps[currentStep] === 'Abilities'" class="wizard-step">
          <h3>Ability Scores</h3>
          <p class="step-description">Assign your character's ability scores</p>

          <div class="ability-controls">
            <div class="ability-method-selector">
              <label>Method</label>
              <select v-model="abilityScoreMethod" class="form-select">
                <option value="standard">Standard Array (15, 14, 13, 12, 10, 8)</option>
                <option value="point-buy">Point Buy (27 points)</option>
                <option value="manual">Manual Entry</option>
              </select>
            </div>
            <button
              type="button"
              class="btn-secondary apply-defaults-btn"
              @click="applyClassDefaults"
              :disabled="!selectedClassDetails"
            >
              Apply {{ formData.class }} Defaults
            </button>
          </div>

          <table class="ability-scores-table">
            <thead>
              <tr>
                <th>Attribute</th>
                <th>Base Score</th>
                <th>Racial Bonus</th>
                <th>Total</th>
                <th>Modifier</th>
                <th v-if="abilityScoreMethod === 'point-buy'">Cost</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="ability in abilityNames" :key="ability">
                <td class="ability-name">{{ ability.charAt(0).toUpperCase() + ability.slice(1, 3) }}</td>
                <td class="ability-base">
                  <!-- Standard Array: dropdown -->
                  <select v-if="abilityScoreMethod === 'standard'" v-model.number="formData.abilities[ability]" class="form-select-sm">
                    <option :value="null">-</option>
                    <option v-for="score in standardArray" :key="score" :value="score">{{ score }}</option>
                  </select>

                  <!-- Point Buy & Manual: incrementer -->
                  <div v-else class="ability-incrementer">
                    <button
                      type="button"
                      class="increment-btn decrement"
                      @click="decrementAbility(ability)"
                      :disabled="!canDecrement(ability)"
                    >-</button>
                    <span class="ability-value">{{ formData.abilities[ability] }}</span>
                    <button
                      type="button"
                      class="increment-btn increment"
                      @click="incrementAbility(ability)"
                      :disabled="!canIncrement(ability)"
                    >+</button>
                  </div>
                </td>
                <td class="racial-bonus" :class="{ 'has-bonus': racialBonuses[ability] > 0 }">
                  {{ racialBonuses[ability] > 0 ? '+' + racialBonuses[ability] : '-' }}
                </td>
                <td class="ability-total">
                  {{ formData.abilities[ability] + racialBonuses[ability] }}
                </td>
                <td class="ability-modifier">
                  {{ getAbilityModifier(formData.abilities[ability] + racialBonuses[ability]) >= 0 ? '+' : '' }}{{ getAbilityModifier(formData.abilities[ability] + racialBonuses[ability]) }}
                </td>
                <td v-if="abilityScoreMethod === 'point-buy'" class="ability-cost">
                  {{ pointBuyCost(formData.abilities[ability]) }}
                </td>
              </tr>
            </tbody>
          </table>

          <div v-if="abilityScoreMethod === 'point-buy'" class="point-buy-remaining">
            Points remaining: <strong>{{ calculatePointBuyRemaining() }}</strong> / 27
          </div>
        </div>

        <!-- Step 4: Skill Proficiencies -->
        <div v-if="steps[currentStep] === 'Skills'" class="wizard-step">
          <h3>Skill Proficiencies</h3>
          <p class="step-description">Choose your skill proficiencies</p>

          <!-- Background skills (fixed) -->
          <div v-if="backgroundSkills.length > 0" class="skills-section">
            <h4 class="skills-section-header">From Background ({{ formData.background }})</h4>
            <div class="background-skills">
              <span v-for="skill in backgroundSkills" :key="skill" class="skill-chip locked">
                <span class="skill-name">{{ skill }}</span>
              </span>
            </div>
          </div>

          <!-- Class skill choices -->
          <div class="skills-section">
            <h4 class="skills-section-header">
              Choose {{ classSkillChoices.count }} from {{ formData.class }}
              <span class="skills-counter-inline">({{ selectedClassSkills.length }}/{{ classSkillChoices.count }})</span>
            </h4>

            <div class="skills-by-ability">
              <div v-for="group in skillsByAbility" :key="group.ability" class="skill-group">
                <!-- Only show groups that have available class skills -->
                <template v-if="group.skills.some(s => isClassSkillAvailable(s.name))">
                  <h4 class="ability-header">{{ group.ability }}</h4>
                  <div class="skill-chips">
                    <div
                      v-for="skill in group.skills.filter(s => isClassSkillAvailable(s.name))"
                      :key="skill.name"
                      class="skill-chip"
                      :class="{
                        selected: isSkillSelected(skill.name),
                        disabled: atClassSkillLimit && !isSkillSelected(skill.name),
                        'from-background': isBackgroundSkill(skill.name)
                      }"
                      @click="toggleSkill(skill.name)"
                    >
                      <span class="skill-name">{{ skill.name }}</span>
                      <span class="skill-description">{{ skill.description }}</span>
                      <span v-if="isBackgroundSkill(skill.name)" class="skill-badge">From background</span>
                    </div>
                  </div>
                </template>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 5: Spells (for spellcasters) -->
        <div v-if="steps[currentStep] === 'Spells'" class="wizard-step">
          <h3>Spells</h3>
          <p class="step-description">Select your starting spells</p>

          <SpellSelector
            v-if="selectedClassDetails"
            :class-name="formData.class"
            :max-spell-level="1"
            :spells-allowed="spellsAllowed"
            @update:selected="handleSpellUpdate"
            @update:selectedGrouped="handleSpellGroupedUpdate"
          />
          <div v-else class="loading">Loading spell options...</div>
        </div>

        <!-- Step 6: Campaign Assignment -->
        <div v-if="steps[currentStep] === 'Campaign'" class="wizard-step">
          <h3>Campaign Assignment</h3>
          <p class="step-description">Assign to a campaign or leave unassigned</p>

          <select v-model="formData.campaign_id" class="form-select">
            <option :value="null">Unassigned (Character Pool)</option>
            <option v-for="campaign in campaigns" :key="campaign.id" :value="campaign.id">
              {{ campaign.name }}
            </option>
          </select>
        </div>

        <!-- Step 7: Review -->
        <div v-if="steps[currentStep] === 'Review'" class="wizard-step">
          <h3>Review</h3>
          <p class="step-description">Review your character before creating</p>

          <div class="review-section">
            <div class="review-item">
              <span class="review-label">Type:</span>
              <span class="review-value">{{ formData.is_npc ? 'NPC' : 'Player Character' }}</span>
            </div>
            <div v-if="!formData.is_npc" class="review-item">
              <span class="review-label">Player:</span>
              <span class="review-value">{{ getPlayerName(formData.player_id) }}</span>
            </div>
            <!-- NPC fields -->
            <template v-if="formData.is_npc">
              <div v-if="formData.npc_role" class="review-item">
                <span class="review-label">Role:</span>
                <span class="review-value">{{ formData.npc_role }}</span>
              </div>
              <div v-if="formData.npc_location" class="review-item">
                <span class="review-label">Location:</span>
                <span class="review-value">{{ formData.npc_location }}</span>
              </div>
              <div v-if="formData.npc_faction" class="review-item">
                <span class="review-label">Faction:</span>
                <span class="review-value">{{ formData.npc_faction }}</span>
              </div>
            </template>
            <div class="review-item">
              <span class="review-label">Character Name:</span>
              <span class="review-value">{{ formData.character_name }}</span>
            </div>
            <div class="review-item">
              <span class="review-label">Race:</span>
              <span class="review-value">{{ formData.race }}</span>
            </div>
            <div class="review-item">
              <span class="review-label">Class:</span>
              <span class="review-value">{{ formData.class }}</span>
            </div>
            <div class="review-item">
              <span class="review-label">Background:</span>
              <span class="review-value">{{ formData.background }}</span>
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
            <div class="review-item">
              <span class="review-label">Skills:</span>
              <span class="review-value">
                <template v-if="backgroundSkills.length > 0">
                  <span class="skill-source">Background:</span> {{ backgroundSkills.join(', ') }}
                  <template v-if="selectedClassSkills.length > 0">
                    <br /><span class="skill-source">Class:</span> {{ selectedClassSkills.join(', ') }}
                  </template>
                </template>
                <template v-else>
                  {{ formData.skills.join(', ') || 'None' }}
                </template>
              </span>
            </div>
            <div v-if="isSpellsKnownCaster" class="review-item">
              <span class="review-label">Spells:</span>
              <span class="review-value">{{ selectedSpells.length }} spells selected</span>
            </div>
            <div class="review-item">
              <span class="review-label">Campaign:</span>
              <span class="review-value">{{ getCampaignName(formData.campaign_id) }}</span>
            </div>
            <div v-if="formData.is_npc && formData.npc_notes" class="review-item">
              <span class="review-label">Notes:</span>
              <span class="review-value review-notes">{{ formData.npc_notes }}</span>
            </div>
            <div v-if="formData.is_npc && formData.legendary_actions.length > 0" class="review-item review-legendary">
              <span class="review-label">Legendary Actions:</span>
              <span class="review-value">
                {{ formData.legendary_actions.length }} action(s), {{ formData.legendary_action_count }}/round
                <ul class="legendary-review-list">
                  <li v-for="(action, i) in formData.legendary_actions" :key="i">
                    <strong>{{ action.name || '(unnamed)' }}</strong>
                    <span v-if="action.cost > 1"> ({{ action.cost }} actions)</span>
                  </li>
                </ul>
              </span>
            </div>
          </div>
        </div>

        <div v-if="error" class="error-message">{{ error }}</div>
    </div>

    <template #footer>
      <button
        v-if="currentStep > 0"
        @click="previousStep"
        class="btn btn-secondary"
        :disabled="creating"
      >
        Back
      </button>
      <button
        v-if="currentStep < steps.length - 1"
        @click="nextStep"
        class="btn btn-primary"
        :disabled="!canProceed || creating"
      >
        Next
      </button>
      <button
        v-if="currentStep === steps.length - 1"
        @click="createCharacter"
        class="btn btn-primary"
        :disabled="creating"
      >
        {{ creating ? 'Creating...' : 'Create Character' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlayerStore } from '../../../stores/players'
import { useCampaignStore } from '../../../stores/campaigns'
import AppModal from '@/components/shared/AppModal.vue'
import SpellSelector from './SpellSelector.vue'
import type { SpellReferenceInput, LegendaryAction } from '@/types/character'

const props = withDefaults(defineProps<{
  visible: boolean
  campaignId?: number | null
  startAsNpc?: boolean
}>(), {
  campaignId: null,
  startAsNpc: false
})

const emit = defineEmits<{
  close: []
  created: []
}>()

const playerStore = usePlayerStore()
const campaignStore = useCampaignStore()

// Spells Known classes that need spell selection at level 1
const spellsKnownClasses = ['Bard', 'Ranger', 'Sorcerer', 'Warlock']

const isSpellsKnownCaster = computed(() => {
  return spellsKnownClasses.includes(formData.value.class)
})

// Dynamic steps based on whether class is a spellcaster
const steps = computed(() => {
  if (isSpellsKnownCaster.value) {
    return ['Player', 'Basic Info', 'Abilities', 'Skills', 'Spells', 'Campaign', 'Review']
  }
  return ['Player', 'Basic Info', 'Abilities', 'Skills', 'Campaign', 'Review']
})

const currentStep = ref(0)
const creating = ref(false)
const error = ref<string | null>(null)

// Spell selection state
const selectedSpells = ref<SpellReferenceInput[]>([])
const selectedSpellsGrouped = ref<Record<number, SpellReferenceInput[]>>({})

// Ability score assignment methods
const abilityScoreMethod = ref<'manual' | 'standard' | 'point-buy'>('standard')
const standardArray = [15, 14, 13, 12, 10, 8]
const pointBuyPoints = ref(27)

// Typed ability names for iteration
type AbilityName = 'strength' | 'dexterity' | 'constitution' | 'intelligence' | 'wisdom' | 'charisma'
const abilityNames: AbilityName[] = ['strength', 'dexterity', 'constitution', 'intelligence', 'wisdom', 'charisma']

const formData = ref({
  player_id: null as number | null,
  character_name: '',
  race: '',
  class: '',
  background: '',
  campaign_id: props.campaignId as number | null,
  abilities: {
    strength: 10,
    dexterity: 10,
    constitution: 10,
    intelligence: 10,
    wisdom: 10,
    charisma: 10
  },
  skills: [] as string[],
  // NPC-specific fields
  is_npc: props.startAsNpc,
  npc_role: '',
  npc_location: '',
  npc_faction: '',
  npc_notes: '',
  // Boss NPC abilities
  legendary_actions: [] as LegendaryAction[],
  legendary_action_count: 3
})

const players = computed(() => playerStore.players)
const campaigns = computed(() => campaignStore.campaigns)

// Catalog data
interface CatalogItem {
  name: string
  source: string
  subclassName?: string | null
}

const races = ref<CatalogItem[]>([])
const classes = ref<CatalogItem[]>([])
const backgrounds = ref<CatalogItem[]>([])

// Monsters loaded for NPC race options
interface MonsterSummary {
  name: string
  type: string
  source: string
  cr: string
}
const monsters = ref<MonsterSummary[]>([])

// Selected class/background full details for skill proficiencies
interface ClassDetails {
  startingProficiencies?: {
    skills?: Array<{
      choose?: {
        count: number
        from: string[]
      }
    }>
  }
  multiclassing?: {
    requirements?: {
      str?: number
      dex?: number
      con?: number
      int?: number
      wis?: number
      cha?: number
      or?: Array<Record<string, number>>
    }
  }
  // Spell progression data (camelCase to match backend JSON)
  cantripProgression?: number[]
  spellsKnownProgression?: number[]
  spellcastingAbility?: string
}

interface BackgroundDetails {
  skillProficiencies?: Array<Record<string, boolean>>
}

const selectedClassDetails = ref<ClassDetails | null>(null)
const selectedBackgroundDetails = ref<BackgroundDetails | null>(null)
const selectedRaceDetails = ref<Record<string, unknown> | null>(null)

// Race options - includes monsters for NPCs
interface RaceOption {
  name: string
  source: string
  isMonster?: boolean
  type?: string
  cr?: string
}

const raceOptions = computed((): RaceOption[] => {
  // Start with standard races
  const options: RaceOption[] = races.value.map(r => ({
    name: r.name,
    source: r.source,
    isMonster: false
  }))

  // For NPCs, also include monsters as race options
  if (formData.value.is_npc) {
    const monsterOptions: RaceOption[] = monsters.value.map(m => ({
      name: m.name,
      source: m.source,
      isMonster: true,
      type: m.type,
      cr: m.cr
    }))
    options.push(...monsterOptions)
  }

  // Sort alphabetically by name
  return options.sort((a, b) => a.name.localeCompare(b.name))
})

// Computed racial ability bonuses
const racialBonuses = computed(() => {
  const bonuses: Record<string, number> = {
    strength: 0, dexterity: 0, constitution: 0,
    intelligence: 0, wisdom: 0, charisma: 0
  }

  if (!selectedRaceDetails.value) return bonuses

  const abilities = selectedRaceDetails.value.ability as Array<Record<string, number>> | undefined
  if (!abilities) return bonuses

  for (const abilityObj of abilities) {
    if (abilityObj.str) bonuses.strength += abilityObj.str
    if (abilityObj.dex) bonuses.dexterity += abilityObj.dex
    if (abilityObj.con) bonuses.constitution += abilityObj.con
    if (abilityObj.int) bonuses.intelligence += abilityObj.int
    if (abilityObj.wis) bonuses.wisdom += abilityObj.wis
    if (abilityObj.cha) bonuses.charisma += abilityObj.cha
  }

  return bonuses
})

// Computed: skills granted by background (fixed, can't change)
const backgroundSkills = computed<string[]>(() => {
  if (!selectedBackgroundDetails.value?.skillProficiencies) return []

  const skills: string[] = []
  for (const profGroup of selectedBackgroundDetails.value.skillProficiencies) {
    for (const [skill, granted] of Object.entries(profGroup)) {
      if (granted === true) {
        // Capitalize skill name properly
        skills.push(skill.split(' ').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' '))
      }
    }
  }
  return skills
})

// Computed: class skill choices
const classSkillChoices = computed(() => {
  if (!selectedClassDetails.value?.startingProficiencies?.skills) {
    return { count: 0, from: [] as string[] }
  }

  const skillDef = selectedClassDetails.value.startingProficiencies.skills[0]
  if (skillDef?.choose) {
    return {
      count: skillDef.choose.count,
      from: skillDef.choose.from.map(s =>
        s.split(' ').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
      )
    }
  }
  return { count: 0, from: [] as string[] }
})

// Computed: spells allowed at level 1
const spellsAllowed = computed((): Record<number, number> => {
  if (!selectedClassDetails.value || !isSpellsKnownCaster.value) return {}

  const result: Record<number, number> = {}

  // Level 1 is index 0
  if (selectedClassDetails.value.cantripProgression && selectedClassDetails.value.cantripProgression[0] !== undefined) {
    result[0] = selectedClassDetails.value.cantripProgression[0]
  }

  if (selectedClassDetails.value.spellsKnownProgression && selectedClassDetails.value.spellsKnownProgression[0] !== undefined) {
    result[1] = selectedClassDetails.value.spellsKnownProgression[0]
  }

  return result
})

// Handle spell selection updates
const handleSpellUpdate = (spells: SpellReferenceInput[]) => {
  selectedSpells.value = spells
}

const handleSpellGroupedUpdate = (grouped: Record<number, SpellReferenceInput[]>) => {
  selectedSpellsGrouped.value = grouped
}

// Fetch class details when class changes
const loadClassDetails = async (className: string, source: string) => {
  try {
    // Returns JSON Value directly, not a string
    const result = await invoke<ClassDetails>('get_class_details', { className, classSource: source })
    if (result) {
      selectedClassDetails.value = result
    }
  } catch (e) {
    console.error('Failed to load class details:', e)
    selectedClassDetails.value = null
  }
}

// Fetch background details when background changes
const loadBackgroundDetails = async (backgroundName: string, source: string) => {
  try {
    // Returns JSON Value directly, not a string
    const result = await invoke<BackgroundDetails>('get_background_details', { name: backgroundName, source })
    if (result) {
      selectedBackgroundDetails.value = result
    }
  } catch (e) {
    console.error('Failed to load background details:', e)
    selectedBackgroundDetails.value = null
  }
}

// Fetch race details when race changes
const loadRaceDetails = async (raceName: string, source: string) => {
  try {
    const result = await invoke<string>('get_race_details', { name: raceName, source })
    if (result) {
      selectedRaceDetails.value = JSON.parse(result)
    }
  } catch (e) {
    console.error('Failed to load race details:', e)
    selectedRaceDetails.value = null
  }
}

// D&D 5e skills organized by ability with descriptions
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
      { name: 'Athletics', description: 'Climbing, jumping, swimming, and other physical activities' }
    ]
  },
  {
    ability: 'Dexterity',
    skills: [
      { name: 'Acrobatics', description: 'Balance, tumbling, and aerial maneuvers' },
      { name: 'Sleight of Hand', description: 'Pickpocketing, concealing objects, and manual trickery' },
      { name: 'Stealth', description: 'Moving silently and hiding from enemies' }
    ]
  },
  {
    ability: 'Intelligence',
    skills: [
      { name: 'Arcana', description: 'Knowledge of spells, magic items, and magical traditions' },
      { name: 'History', description: 'Recalling lore about historical events and civilizations' },
      { name: 'Investigation', description: 'Searching for clues and making deductions' },
      { name: 'Nature', description: 'Knowledge of terrain, plants, animals, and weather' },
      { name: 'Religion', description: 'Knowledge of deities, rites, and religious traditions' }
    ]
  },
  {
    ability: 'Wisdom',
    skills: [
      { name: 'Animal Handling', description: 'Calming, controlling, or intuiting animal intentions' },
      { name: 'Insight', description: 'Determining true intentions and detecting lies' },
      { name: 'Medicine', description: 'Stabilizing the dying and diagnosing illnesses' },
      { name: 'Perception', description: 'Spotting, hearing, or detecting presences' },
      { name: 'Survival', description: 'Tracking, hunting, navigating, and avoiding hazards' }
    ]
  },
  {
    ability: 'Charisma',
    skills: [
      { name: 'Deception', description: 'Lying, misleading, and hiding the truth' },
      { name: 'Intimidation', description: 'Influencing others through threats and hostility' },
      { name: 'Performance', description: 'Entertaining through music, dance, or acting' },
      { name: 'Persuasion', description: 'Influencing others through tact and good nature' }
    ]
  }
]

// Selected class skills (user choices, not including background)
const selectedClassSkills = computed(() => {
  return formData.value.skills.filter(s => !backgroundSkills.value.includes(s))
})

// Check if at class skill limit
const atClassSkillLimit = computed(() => {
  return selectedClassSkills.value.length >= classSkillChoices.value.count
})

// Toggle skill selection (only for class skills, not background)
const toggleSkill = (skillName: string) => {
  // Can't toggle background skills
  if (backgroundSkills.value.includes(skillName)) return

  const index = formData.value.skills.indexOf(skillName)
  if (index === -1) {
    // Only add if under class skill limit
    if (selectedClassSkills.value.length < classSkillChoices.value.count) {
      formData.value.skills.push(skillName)
    }
  } else {
    formData.value.skills.splice(index, 1)
  }
}

// Check if skill is from background (locked)
const isBackgroundSkill = (skillName: string): boolean => {
  return backgroundSkills.value.includes(skillName)
}

// Check if skill is available for class selection
const isClassSkillAvailable = (skillName: string): boolean => {
  return classSkillChoices.value.from.includes(skillName)
}

// Check if skill is selected
const isSkillSelected = (skillName: string): boolean => {
  return formData.value.skills.includes(skillName)
}

// Calculate ability modifier
const getAbilityModifier = (score: number): number => {
  return Math.floor((score - 10) / 2)
}

// Ability score incrementer helpers
const getMinScore = (): number => {
  return abilityScoreMethod.value === 'manual' ? 3 : 8
}

const getMaxScore = (): number => {
  return abilityScoreMethod.value === 'manual' ? 18 : 15
}

const canDecrement = (ability: string): boolean => {
  const current = formData.value.abilities[ability as keyof typeof formData.value.abilities]
  return current > getMinScore()
}

const canIncrement = (ability: string): boolean => {
  const current = formData.value.abilities[ability as keyof typeof formData.value.abilities]
  if (current >= getMaxScore()) return false

  // For point buy, check if we have enough points
  if (abilityScoreMethod.value === 'point-buy') {
    const nextCost = pointBuyCost(current + 1)
    const currentCost = pointBuyCost(current)
    const costDiff = nextCost - currentCost
    return calculatePointBuyRemaining() >= costDiff
  }

  return true
}

const decrementAbility = (ability: string) => {
  const current = formData.value.abilities[ability as keyof typeof formData.value.abilities]
  if (current > getMinScore()) {
    formData.value.abilities[ability as keyof typeof formData.value.abilities] = current - 1
  }
}

const incrementAbility = (ability: string) => {
  const current = formData.value.abilities[ability as keyof typeof formData.value.abilities]
  if (canIncrement(ability)) {
    formData.value.abilities[ability as keyof typeof formData.value.abilities] = current + 1
  }
}

// Apply sensible defaults based on class
const applyClassDefaults = () => {
  if (!selectedClassDetails.value?.multiclassing?.requirements) {
    // No class data, use balanced defaults
    formData.value.abilities = {
      strength: 10, dexterity: 10, constitution: 14,
      intelligence: 10, wisdom: 10, charisma: 10
    }
    return
  }

  const reqs = selectedClassDetails.value.multiclassing.requirements

  // Start with base scores
  const scores = {
    strength: 8, dexterity: 8, constitution: 14,
    intelligence: 8, wisdom: 8, charisma: 8
  }

  // Identify primary abilities from multiclassing requirements
  const primaryAbilities: string[] = []

  if (reqs.str) primaryAbilities.push('strength')
  if (reqs.dex) primaryAbilities.push('dexterity')
  if (reqs.con) primaryAbilities.push('constitution')
  if (reqs.int) primaryAbilities.push('intelligence')
  if (reqs.wis) primaryAbilities.push('wisdom')
  if (reqs.cha) primaryAbilities.push('charisma')

  // Handle "or" requirements (like Fighter: str OR dex) - pick just the first option
  if (reqs.or && reqs.or.length > 0) {
    const firstOption = reqs.or[0]
    const firstKey = Object.keys(firstOption)[0]
    if (firstKey) {
      const fullName = firstKey === 'str' ? 'strength' : firstKey === 'dex' ? 'dexterity' :
                       firstKey === 'con' ? 'constitution' : firstKey === 'int' ? 'intelligence' :
                       firstKey === 'wis' ? 'wisdom' : 'charisma'
      if (!primaryAbilities.includes(fullName)) {
        primaryAbilities.push(fullName)
      }
    }
  }

  // Assign high scores to primary abilities (max 2)
  // Point buy: 15 costs 9, 14 costs 7, 13 costs 5
  primaryAbilities.slice(0, 2).forEach((ability, index) => {
    scores[ability as keyof typeof scores] = index === 0 ? 15 : 14
  })

  // CON is always important for HP
  if (!primaryAbilities.includes('constitution')) {
    // If we have 2 primary abilities, CON gets 14
    // If we have 1 primary ability, CON can also get 14
    scores.constitution = 14
  }

  // Fill remaining abilities to use all 27 points
  const calcUsed = () => Object.values(scores).reduce((sum, score) => sum + pointBuyCost(score), 0)

  // First pass: get all 8s to 10
  const abilityPriority = ['dexterity', 'wisdom', 'intelligence', 'charisma']
  for (const ability of abilityPriority) {
    if (scores[ability as keyof typeof scores] === 8 && (27 - calcUsed()) >= 2) {
      scores[ability as keyof typeof scores] = 10
    }
  }

  // Second pass: spend remaining points by bumping scores
  // Prioritize secondary combat stats
  let remaining = 27 - calcUsed()

  while (remaining > 0) {
    let spent = false

    // Try to bump a 10 to higher values
    for (const ability of abilityPriority) {
      const current = scores[ability as keyof typeof scores]
      if (current >= 8 && current < 13) {
        const nextScore = current + 1
        const costDiff = pointBuyCost(nextScore) - pointBuyCost(current)
        if (costDiff <= remaining) {
          scores[ability as keyof typeof scores] = nextScore
          remaining -= costDiff
          spent = true
          break
        }
      }
    }

    if (!spent) break // Can't spend any more points
  }

  // Distribute remaining points for point buy balance
  // This gives a reasonable starting point
  if (abilityScoreMethod.value === 'standard') {
    // For standard array, just assign sensibly
    const remaining = [13, 12, 10, 8].filter(s => !Object.values(scores).includes(s))
    const unassigned = Object.keys(scores).filter(k =>
      scores[k as keyof typeof scores] === 8 && k !== 'constitution'
    )
    unassigned.forEach((ability, i) => {
      if (remaining[i]) {
        scores[ability as keyof typeof scores] = remaining[i]
      }
    })
  }

  formData.value.abilities = scores
}

// Calculate point buy cost
const pointBuyCost = (score: number): number => {
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

// Calculate remaining point buy points
const calculatePointBuyRemaining = (): number => {
  if (abilityScoreMethod.value !== 'point-buy') return 0

  const total = pointBuyCost(formData.value.abilities.strength) +
                pointBuyCost(formData.value.abilities.dexterity) +
                pointBuyCost(formData.value.abilities.constitution) +
                pointBuyCost(formData.value.abilities.intelligence) +
                pointBuyCost(formData.value.abilities.wisdom) +
                pointBuyCost(formData.value.abilities.charisma)

  return 27 - total
}

// Load catalog data
const loadCatalogData = async () => {
  try {
    // Call catalog search commands with correct parameters
    const [raceResults, classResults, backgroundResults, monsterResults] = await Promise.all([
      invoke<CatalogItem[]>('search_races', {
        search: null,
        sources: null,
        sizes: null,
        has_darkvision: null,
        has_flight: null
      }),
      invoke<CatalogItem[]>('search_classes', {
        filters: {
          search_pattern: null,
          sources: null,
          caster_progression: null,
          primary_ability: null
        }
      }),
      invoke<CatalogItem[]>('search_backgrounds', {
        filters: {
          search_pattern: null,
          sources: null,
          has_tools: null
        }
      }),
      // Load monsters for NPC race options
      invoke<MonsterSummary[]>('search_monsters', {
        filters: {
          name: null,
          sources: null,
          creature_types: null,
          sizes: null,
          min_cr: null,
          max_cr: null,
          alignments: null,
          min_hp: null,
          max_hp: null,
          environment: null
        }
      })
    ])

    races.value = raceResults
    // Filter to only show base classes (subclassName is null for base classes)
    classes.value = classResults.filter(cls => !cls.subclassName)
    backgrounds.value = backgroundResults
    monsters.value = monsterResults
  } catch (e) {
    console.error('Failed to load catalog data:', e)
    error.value = 'Failed to load character options. Please try again.'
  }
}

const canProceed = computed(() => {
  const currentStepName = steps.value[currentStep.value]

  switch (currentStepName) {
    case 'Player':
      // NPCs don't need a player, PCs do
      return formData.value.is_npc || formData.value.player_id !== null
    case 'Basic Info':
      return !!(
        formData.value.character_name &&
        formData.value.race &&
        formData.value.class &&
        formData.value.background
      )
    case 'Abilities':
      // Ability scores - validate based on method
      if (abilityScoreMethod.value === 'standard') {
        // All 6 abilities must be assigned from standard array
        const abilities = Object.values(formData.value.abilities)
        return abilities.every(a => a && standardArray.includes(a)) &&
               new Set(abilities).size === 6 // All different
      } else if (abilityScoreMethod.value === 'point-buy') {
        // Must have exactly 0 points remaining
        return calculatePointBuyRemaining() === 0
      } else {
        // Manual - all abilities must be between 3-18
        return Object.values(formData.value.abilities).every(a => a >= 3 && a <= 18)
      }
    case 'Skills':
      // Skills - must have selected all class skills
      return selectedClassSkills.value.length === classSkillChoices.value.count
    case 'Spells':
      // Spells - must have selected the right number
      return selectedSpells.value.length > 0
    case 'Campaign':
      // Campaign is optional
      return true
    case 'Review':
      return true
    default:
      return false
  }
})

const closeWizard = () => {
  if (!creating.value) {
    resetForm()
    emit('close')
  }
}

const resetForm = () => {
  currentStep.value = 0
  abilityScoreMethod.value = 'standard'
  formData.value = {
    player_id: null,
    character_name: '',
    race: '',
    class: '',
    background: '',
    campaign_id: props.campaignId ?? null,
    abilities: {
      strength: 10,
      dexterity: 10,
      constitution: 10,
      intelligence: 10,
      wisdom: 10,
      charisma: 10
    },
    skills: [],
    is_npc: props.startAsNpc,
    npc_role: '',
    npc_location: '',
    npc_faction: '',
    npc_notes: '',
    legendary_actions: [],
    legendary_action_count: 3
  }
  selectedSpells.value = []
  selectedSpellsGrouped.value = {}
  error.value = null
}

const setCharacterType = (isNpc: boolean) => {
  formData.value.is_npc = isNpc
  // Clear player_id when switching to NPC
  if (isNpc) {
    formData.value.player_id = null
  }
  // Clear NPC fields when switching to PC
  if (!isNpc) {
    formData.value.npc_role = ''
    formData.value.npc_location = ''
    formData.value.npc_faction = ''
    formData.value.npc_notes = ''
    formData.value.legendary_actions = []
    formData.value.legendary_action_count = 3
  }
}

// Legendary action helpers
const addLegendaryAction = () => {
  formData.value.legendary_actions.push({
    name: '',
    cost: 1,
    description: ''
  })
}

const removeLegendaryAction = (index: number) => {
  formData.value.legendary_actions.splice(index, 1)
}

const nextStep = () => {
  if (canProceed.value && currentStep.value < steps.value.length - 1) {
    currentStep.value++
    error.value = null
  }
}

const previousStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--
    error.value = null
  }
}

const getPlayerName = (playerId: number | null): string => {
  if (!playerId) return 'None'
  const player = players.value.find(p => p.id === playerId)
  return player?.name || 'Unknown'
}

const getCampaignName = (campaignId: number | null): string => {
  if (!campaignId) return 'Unassigned (Character Pool)'
  const campaign = campaigns.value.find(c => c.id === campaignId)
  return campaign?.name || 'Unknown'
}

const createCharacter = async () => {
  // PCs require a player, NPCs don't
  if (!formData.value.is_npc && !formData.value.player_id) {
    error.value = 'Player is required for player characters'
    return
  }

  creating.value = true
  error.value = null

  try {
    // Get sources for race, class, background
    const raceItem = races.value.find(r => r.name === formData.value.race)
    const classItem = classes.value.find(c => c.name === formData.value.class)
    const bgItem = backgrounds.value.find(b => b.name === formData.value.background)

    // Map ability score method to backend format
    const methodMap: Record<string, string> = {
      'manual': 'manual',
      'standard': 'standard_array',
      'point-buy': 'point_buy'
    }

    const request = {
      character_name: formData.value.character_name,
      player_id: formData.value.is_npc ? null : formData.value.player_id,
      race: formData.value.race,
      race_source: raceItem?.source || 'PHB',
      subrace: null,
      class: formData.value.class,
      class_source: classItem?.source || 'PHB',
      subclass: null,
      background: formData.value.background,
      background_source: bgItem?.source || 'PHB',
      ability_score_method: methodMap[abilityScoreMethod.value] || 'standard_array',
      ability_scores: {
        strength: formData.value.abilities.strength,
        dexterity: formData.value.abilities.dexterity,
        constitution: formData.value.abilities.constitution,
        intelligence: formData.value.abilities.intelligence,
        wisdom: formData.value.abilities.wisdom,
        charisma: formData.value.abilities.charisma
      },
      alignment: null,
      personality: null,
      skill_proficiencies: formData.value.skills,
      equipment: null,
      cantrips: isSpellsKnownCaster.value && selectedSpellsGrouped.value[0]?.length
        ? selectedSpellsGrouped.value[0]
        : null,
      known_spells: isSpellsKnownCaster.value
        ? Object.entries(selectedSpellsGrouped.value)
            .filter(([level]) => parseInt(level) > 0)
            .flatMap(([, spells]) => spells)
        : null,
      // NPC fields
      is_npc: formData.value.is_npc || null,
      npc_role: formData.value.is_npc && formData.value.npc_role ? formData.value.npc_role : null,
      npc_location: formData.value.is_npc && formData.value.npc_location ? formData.value.npc_location : null,
      npc_faction: formData.value.is_npc && formData.value.npc_faction ? formData.value.npc_faction : null,
      npc_notes: formData.value.is_npc && formData.value.npc_notes ? formData.value.npc_notes : null,
      // Boss NPC abilities
      legendary_actions: formData.value.is_npc && formData.value.legendary_actions.length > 0
        ? formData.value.legendary_actions
        : null,
      legendary_action_count: formData.value.is_npc && formData.value.legendary_actions.length > 0
        ? formData.value.legendary_action_count
        : null
    }

    await invoke('create_character', { request })

    emit('created')
    closeWizard()
  } catch (e) {
    console.error('Failed to create character:', e)
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    creating.value = false
  }
}

// Load players, campaigns, and catalog data when wizard opens
watch(
  () => props.visible,
  async (isVisible) => {
    if (isVisible) {
      await Promise.all([
        playerStore.fetchPlayers(),
        campaignStore.fetchCampaigns(),
        loadCatalogData()
      ])
    }
  }
)

// Load class details when class selection changes
watch(
  () => formData.value.class,
  async (newClass) => {
    if (newClass) {
      // Find source from classes list
      const cls = classes.value.find(c => c.name === newClass)
      if (cls) {
        await loadClassDetails(newClass, cls.source)
        // Clear selected skills when class changes
        formData.value.skills = []
      }
    } else {
      selectedClassDetails.value = null
    }
  }
)

// Load race details when race selection changes
watch(
  () => formData.value.race,
  async (newRace) => {
    if (newRace) {
      const race = races.value.find(r => r.name === newRace)
      if (race) {
        await loadRaceDetails(newRace, race.source)
      }
    } else {
      selectedRaceDetails.value = null
    }
  }
)

// Load background details when background selection changes
watch(
  () => formData.value.background,
  async (newBackground) => {
    if (newBackground) {
      // Find source from backgrounds list
      const bg = backgrounds.value.find(b => b.name === newBackground)
      if (bg) {
        await loadBackgroundDetails(newBackground, bg.source)
        // Clear selected skills and add background skills
        formData.value.skills = []
      }
    } else {
      selectedBackgroundDetails.value = null
    }
  }
)

// Auto-add background skills when background details load
watch(
  backgroundSkills,
  (skills) => {
    // Add background skills to formData if not already there
    for (const skill of skills) {
      if (!formData.value.skills.includes(skill)) {
        formData.value.skills.push(skill)
      }
    }
  },
  { immediate: true }
)
</script>

<style scoped>
/* Domain-specific styles */
.wizard-progress {
  display: flex;
  padding: 20px;
  border-bottom: 1px solid var(--border);
  gap: 8px;
}

.progress-step {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.step-number {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--surface);
  border: 2px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.progress-step.active .step-number {
  background: var(--primary);
  border-color: var(--primary);
  color: white;
}

.progress-step.completed .step-number {
  background: var(--primary);
  border-color: var(--primary);
  color: white;
}

.step-label {
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
}

.progress-step.active .step-label {
  color: var(--text);
  font-weight: 500;
}

.wizard-body {
  flex: 1;
  overflow-y: auto;
  padding: 30px;
}

.wizard-step h3 {
  margin: 0 0 8px;
  font-size: 20px;
  color: var(--text);
}

.step-description {
  margin: 0 0 24px;
  color: var(--text-secondary);
  font-size: 14px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
}

.form-input,
.form-select {
  width: 100%;
  padding: 10px 12px;
  font-size: 14px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface);
  color: var(--text);
  font-family: inherit;
  transition: border-color 0.2s;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--primary);
}

.review-section {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 20px;
}

.review-item {
  display: flex;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid var(--border);
}

.review-item:last-child {
  border-bottom: none;
}

.review-label {
  font-weight: 500;
  color: var(--text-secondary);
}

.review-value {
  color: var(--text);
}

.skill-source {
  font-weight: 500;
  color: var(--text-secondary);
}

.error-message {
  margin-top: 16px;
  padding: 12px;
  background: rgba(220, 38, 38, 0.1);
  border: 1px solid var(--error);
  border-radius: 4px;
  color: var(--error);
  font-size: 14px;
}

/* Ability Score Controls */
.ability-controls {
  display: flex;
  gap: 16px;
  align-items: flex-end;
  margin-bottom: 24px;
}

.ability-method-selector {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ability-method-selector label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
}

.apply-defaults-btn {
  white-space: nowrap;
  height: fit-content;
}

/* Ability Scores Table */
.ability-scores-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 16px;
}

.ability-scores-table th,
.ability-scores-table td {
  padding: 8px 12px;
  text-align: center;
  border-bottom: 1px solid var(--border);
}

.ability-scores-table th {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.ability-scores-table tbody tr:hover {
  background: var(--surface);
}

.ability-name {
  font-weight: 600;
  text-align: left !important;
  color: var(--text);
}

.ability-base {
  min-width: 100px;
}

.racial-bonus {
  color: var(--text-secondary);
  font-size: 14px;
}

.racial-bonus.has-bonus {
  color: var(--success, #22c55e);
  font-weight: 600;
}

.ability-total {
  font-weight: 700;
  font-size: 16px;
  color: var(--text);
}

.ability-modifier {
  font-size: 14px;
  color: var(--primary);
  font-weight: 500;
}

.ability-cost {
  font-size: 12px;
  color: var(--text-secondary);
}

.form-select-sm {
  padding: 4px 8px;
  font-size: 14px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface);
  color: var(--text);
  min-width: 60px;
}

.form-select-sm:focus {
  outline: none;
  border-color: var(--primary);
}

.ability-incrementer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.increment-btn {
  width: 24px;
  height: 24px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.increment-btn:hover:not(:disabled) {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}

.increment-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.ability-value {
  width: 28px;
  text-align: center;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}

.point-buy-remaining {
  padding: 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 4px;
  text-align: center;
  font-weight: 500;
  color: var(--text);
}

/* Skills Selection */
.skills-section {
  margin-bottom: 24px;
}

.skills-section-header {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}

.skills-counter-inline {
  font-weight: 400;
  color: var(--text-secondary);
}

.background-skills {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}

.skills-by-ability {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.skill-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.ability-header {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.skill-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.skill-chip {
  display: flex;
  flex-direction: column;
  padding: 10px 14px;
  border: 2px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--color-surface);
  min-width: 180px;
  max-width: 280px;
  flex: 1;
}

.skill-chip:hover {
  border-color: var(--primary);
  background: var(--surface);
}

.skill-chip.selected {
  border-color: var(--primary);
  background: var(--primary);
  color: white;
}

.skill-chip.selected .skill-description {
  color: rgba(255, 255, 255, 0.85);
}

.skill-chip.locked {
  background: var(--surface);
  border-color: var(--primary);
  cursor: default;
  padding: 8px 12px;
  min-width: auto;
  max-width: none;
}

.skill-chip.locked .skill-name {
  color: var(--primary);
  margin-bottom: 0;
}

.skill-chip.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.skill-chip.disabled:hover {
  border-color: var(--border);
  background: var(--color-surface);
}

.skill-chip.from-background {
  border-color: var(--primary);
}

.skill-badge {
  font-size: 9px;
  padding: 2px 6px;
  background: var(--primary);
  color: white;
  border-radius: 3px;
  margin-top: 4px;
  align-self: flex-start;
}

.skill-name {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 4px;
}

.skill-description {
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.3;
}

.loading {
  text-align: center;
  padding: 20px;
  color: var(--text-secondary);
}

/* Character Type Toggle */
.character-type-toggle {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}

.type-toggle-btn {
  flex: 1;
  padding: 16px 20px;
  font-size: 14px;
  font-weight: 500;
  border: 2px solid var(--border);
  border-radius: 8px;
  background: var(--color-surface);
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s;
}

.type-toggle-btn:hover {
  border-color: var(--primary);
  background: var(--surface);
}

.type-toggle-btn.active {
  border-color: var(--primary);
  background: var(--primary);
  color: white;
}

/* NPC Fields */
.npc-fields {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.optional-label {
  font-weight: 400;
  color: var(--text-secondary);
  font-size: 12px;
}

.form-textarea {
  width: 100%;
  padding: 10px 12px;
  font-size: 14px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface);
  color: var(--text);
  font-family: inherit;
  resize: vertical;
  min-height: 80px;
  transition: border-color 0.2s;
}

.form-textarea:focus {
  outline: none;
  border-color: var(--primary);
}

/* Review notes styling */
.review-notes {
  white-space: pre-wrap;
  font-style: italic;
}

/* Legendary Actions Section */
.legendary-actions-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--border);
}

.legendary-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.legendary-header label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
}

.btn-add-action {
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 500;
  background: var(--surface);
  color: var(--primary);
  border: 1px solid var(--primary);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-add-action:hover {
  background: var(--primary);
  color: white;
}

.legendary-count-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 14px;
}

.legendary-count-row label {
  color: var(--text-secondary);
}

.legendary-action-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 12px;
}

.action-header {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 8px;
}

.action-name-input {
  flex: 1;
}

.action-cost {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.btn-remove-action {
  width: 24px;
  height: 24px;
  padding: 0;
  font-size: 14px;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-remove-action:hover {
  background: var(--error, #dc2626);
  color: white;
  border-color: var(--error, #dc2626);
}

.action-description {
  min-height: 50px;
}

/* Legendary actions in review */
.review-legendary {
  flex-direction: column;
  align-items: flex-start;
}

.legendary-review-list {
  margin: 8px 0 0 0;
  padding-left: 20px;
  font-size: 13px;
}

.legendary-review-list li {
  margin-bottom: 4px;
}
</style>
