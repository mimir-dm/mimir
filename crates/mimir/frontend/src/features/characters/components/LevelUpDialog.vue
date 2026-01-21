<template>
  <AppModal
    :visible="visible"
    :title="`Level Up to ${newLevel}`"
    size="md"
    @close="closeDialog"
  >
    <template #header>
      <h2>Level Up to {{ newLevel }}</h2>
      <div class="dialog-progress">
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

    <div class="dialog-body">
        <!-- Step: Class Selection -->
        <div v-if="steps[currentStep] === 'Class'" class="dialog-step">
          <h3>Choose Class</h3>
          <p class="step-description">Select which class to level up in</p>

          <div class="class-options">
            <!-- Current classes -->
            <div class="class-section">
              <h4>Current Classes</h4>
              <div
                v-for="cls in characterData.classes"
                :key="cls.class_name"
                class="class-option"
                :class="{ selected: selectedClassName === cls.class_name }"
                @click="selectedClassName = cls.class_name"
              >
                <input type="radio" :checked="selectedClassName === cls.class_name" />
                <span class="class-name">{{ cls.class_name }}</span>
                <span class="class-level">Level {{ cls.level }} → {{ cls.level + 1 }}</span>
              </div>
            </div>

            <!-- Multiclass options -->
            <div class="class-section">
              <h4>Add New Class (Multiclass)</h4>
              <div
                v-for="cls in availableMulticlasses"
                :key="cls.name"
                class="class-option"
                :class="{ selected: selectedClassName === cls.name, disabled: !cls.meetsPrereqs }"
                @click="cls.meetsPrereqs && (selectedClassName = cls.name)"
              >
                <input type="radio" :checked="selectedClassName === cls.name" :disabled="!cls.meetsPrereqs" />
                <span class="class-name">{{ cls.name }}</span>
                <span v-if="!cls.meetsPrereqs" class="prereq-warning">Prerequisites not met</span>
                <span v-else class="class-level">Level 1</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Step: Hit Points -->
        <div v-if="steps[currentStep] === 'Hit Points'" class="dialog-step">
          <h3>Hit Points</h3>
          <p class="step-description">Choose how to determine your new hit points</p>

          <div class="hp-options">
            <div class="hp-option" :class="{ selected: hpMethod === 'average' }" @click="hpMethod = 'average'">
              <div class="option-header">
                <input type="radio" :checked="hpMethod === 'average'" />
                <span class="option-title">Take Average</span>
              </div>
              <p class="option-description">
                Gain {{ averageHp }} HP ({{ hitDice }}/2 + 1 + {{ conModifier }} CON)
              </p>
            </div>

            <div class="hp-option" :class="{ selected: hpMethod === 'roll' }" @click="hpMethod = 'roll'">
              <div class="option-header">
                <input type="radio" :checked="hpMethod === 'roll'" />
                <span class="option-title">Roll for HP</span>
              </div>
              <div v-if="hpMethod === 'roll'" class="roll-section">
                <button @click="rollHitPoints" class="btn-roll">
                  Roll {{ hitDice }}
                </button>
                <div v-if="hpRoll !== null" class="roll-result">
                  <span class="roll-value">{{ hpRoll }}</span>
                  <span class="roll-total">+ {{ conModifier }} CON = {{ hpRoll + conModifier }} HP</span>
                </div>
              </div>
            </div>

            <div class="hp-option" :class="{ selected: hpMethod === 'manual' }" @click="hpMethod = 'manual'">
              <div class="option-header">
                <input type="radio" :checked="hpMethod === 'manual'" />
                <span class="option-title">Manual Entry</span>
              </div>
              <p class="option-description">Enter a roll result from external dice</p>
              <div v-if="hpMethod === 'manual'" class="manual-section">
                <label>{{ hitDice }} roll result:</label>
                <input
                  type="number"
                  v-model.number="manualHpEntry"
                  :min="1"
                  :max="hitDiceMax"
                  class="form-input manual-input"
                  placeholder="Enter roll"
                />
                <div v-if="manualHpEntry" class="roll-result">
                  <span class="roll-total">+ {{ conModifier }} CON = {{ manualHpEntry + conModifier }} HP</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 2: Ability Score Improvement (if applicable) -->
        <div v-if="steps[currentStep] === 'Abilities'" class="dialog-step">
          <h3>Ability Score Improvement</h3>
          <template v-if="hasAsi">
            <p class="step-description">Choose how to improve your abilities</p>

            <div class="asi-options">
              <div class="asi-option" :class="{ selected: asiMode === 'asi' }" @click="asiMode = 'asi'">
                <div class="option-header">
                  <input type="radio" :checked="asiMode === 'asi'" />
                  <span class="option-title">Ability Score Improvement</span>
                </div>
                <p class="option-description">Increase one ability by 2, or two abilities by 1 each</p>
              </div>

              <div class="asi-option" :class="{ selected: asiMode === 'feat' }" @click="asiMode = 'feat'">
                <div class="option-header">
                  <input type="radio" :checked="asiMode === 'feat'" />
                  <span class="option-title">Feat</span>
                </div>
                <p class="option-description">Gain a feat instead of ability improvements</p>
              </div>
            </div>

            <!-- ASI Selection -->
            <div v-if="asiMode === 'asi'" class="asi-selection">
              <div class="asi-method">
                <label>
                  <input type="radio" v-model="asiMethod" value="single" />
                  +2 to one ability
                </label>
                <label>
                  <input type="radio" v-model="asiMethod" value="split" />
                  +1 to two abilities
                </label>
              </div>

              <div v-if="asiMethod === 'single'" class="ability-select">
                <label>Choose ability to increase by 2:</label>
                <select v-model="asiPrimary" class="form-select">
                  <option value="">-- Select Ability --</option>
                  <option v-for="ability in abilities" :key="ability" :value="ability">
                    {{ ability }} ({{ currentAbilityScore(ability) }} → {{ currentAbilityScore(ability) + 2 }})
                  </option>
                </select>
              </div>

              <div v-if="asiMethod === 'split'" class="ability-select-split">
                <div class="ability-select">
                  <label>First ability (+1):</label>
                  <select v-model="asiPrimary" class="form-select">
                    <option value="">-- Select Ability --</option>
                    <option v-for="ability in abilities" :key="ability" :value="ability">
                      {{ ability }} ({{ currentAbilityScore(ability) }} → {{ currentAbilityScore(ability) + 1 }})
                    </option>
                  </select>
                </div>
                <div class="ability-select">
                  <label>Second ability (+1):</label>
                  <select v-model="asiSecondary" class="form-select">
                    <option value="">-- Select Ability --</option>
                    <option v-for="ability in availableSecondaryAbilities" :key="ability" :value="ability">
                      {{ ability }} ({{ currentAbilityScore(ability) }} → {{ currentAbilityScore(ability) + 1 }})
                    </option>
                  </select>
                </div>
              </div>
            </div>

            <!-- Feat Selection (placeholder for now) -->
            <div v-if="asiMode === 'feat'" class="feat-selection">
              <label>Choose a feat:</label>
              <select v-model="selectedFeat" class="form-select">
                <option value="">-- Select Feat --</option>
                <option v-for="feat in availableFeats" :key="feat" :value="feat">
                  {{ feat }}
                </option>
              </select>
            </div>
          </template>
          <template v-else>
            <p class="no-asi-message">No ability score improvement at this level.</p>
          </template>
        </div>

        <!-- Step 3: Spells (for Spells Known casters) -->
        <div v-if="steps[currentStep] === 'Spells'" class="dialog-step">
          <h3>Spells</h3>
          <p class="step-description">Select your known spells (you can change any spells when leveling)</p>

          <SpellSelector
            v-if="classDetails"
            :class-name="primaryClassName"
            :max-spell-level="maxSpellLevel"
            :spells-allowed="spellsAllowed"
            :initial-selection="currentKnownSpells"
            @update:selected="handleSpellUpdate"
            @update:selectedGrouped="handleSpellGroupedUpdate"
          />
          <div v-else class="loading">Loading spell options...</div>
        </div>

        <!-- Step: New Features -->
        <div v-if="steps[currentStep] === 'Features'" class="dialog-step">
          <h3>New Features</h3>
          <p class="step-description">You gain the following features at level {{ newLevel }}</p>

          <div v-if="newFeatures.length > 0" class="features-list">
            <div v-for="feature in newFeatures" :key="feature" class="feature-item">
              <span class="feature-name">{{ feature }}</span>
            </div>
          </div>
          <div v-else class="no-features">
            <p>No new class features at this level.</p>
          </div>
        </div>

        <!-- Step: Review -->
        <div v-if="steps[currentStep] === 'Review'" class="dialog-step">
          <h3>Review Changes</h3>
          <p class="step-description">Confirm your level up choices</p>

          <div class="review-summary">
            <div class="review-item">
              <span class="review-label">New Level:</span>
              <span class="review-value">{{ newLevel }}</span>
            </div>
            <div class="review-item">
              <span class="review-label">Hit Points:</span>
              <span class="review-value">
                +{{ totalHpGain }} HP
                <span class="review-detail">
                  ({{ hpMethod === 'average' ? 'average' : hpMethod === 'roll' ? 'rolled ' + hpRoll : 'manual ' + manualHpEntry }})
                </span>
              </span>
            </div>
            <div v-if="hasAsi" class="review-item">
              <span class="review-label">Ability Improvement:</span>
              <span class="review-value">
                <template v-if="asiMode === 'asi'">
                  <template v-if="asiMethod === 'single'">
                    {{ asiPrimary }} +2
                  </template>
                  <template v-else>
                    {{ asiPrimary }} +1, {{ asiSecondary }} +1
                  </template>
                </template>
                <template v-else>
                  Feat: {{ selectedFeat }}
                </template>
              </span>
            </div>
            <div v-if="isSpellsKnownCaster && selectedSpells.length > 0" class="review-section">
              <div class="review-section-header">Spells</div>
              <div v-if="selectedSpellsGrouped[0]?.length" class="review-item">
                <span class="review-label">Cantrips:</span>
                <span class="review-value">{{ selectedSpellsGrouped[0].map(s => s.name).join(', ') }}</span>
              </div>
              <template v-for="level in [1, 2, 3, 4, 5, 6, 7, 8, 9]" :key="level">
                <div v-if="selectedSpellsGrouped[level]?.length" class="review-item">
                  <span class="review-label">Level {{ level }}:</span>
                  <span class="review-value">{{ selectedSpellsGrouped[level].map(s => s.name).join(', ') }}</span>
                </div>
              </template>
            </div>
            <div v-if="newFeatures.length > 0" class="review-item">
              <span class="review-label">New Features:</span>
              <span class="review-value">{{ newFeatures.join(', ') }}</span>
            </div>
          </div>
        </div>
      </div>

    <template #footer>
      <button v-if="currentStep > 0" @click="prevStep" class="btn btn-secondary">
        Back
      </button>
      <div class="footer-spacer"></div>
      <button @click="closeDialog" class="btn btn-secondary">
        Cancel
      </button>
      <button v-if="currentStep < steps.length - 1" @click="nextStep" class="btn btn-primary" :disabled="!canProceed">
        Next
      </button>
      <button v-else @click="confirmLevelUp" class="btn btn-primary" :disabled="isSubmitting">
        {{ isSubmitting ? 'Leveling Up...' : 'Confirm Level Up' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import SpellSelector from './SpellSelector.vue'
import { useCharacterStore } from '../../../stores/characters'
import type { CharacterData, LevelUpRequest, SpellReferenceInput, SpellReference } from '../../../types/character'

interface ClassDetails {
  cantripProgression?: number[]
  spellsKnownProgression?: number[]
  spellcastingAbility?: string
}

const props = defineProps<{
  visible: boolean
  characterId: number
  characterData: CharacterData
}>()

const emit = defineEmits<{
  close: []
  completed: []
}>()

const characterStore = useCharacterStore()

// Helper to get primary class name
const primaryClassName = computed(() => {
  if (!props.characterData.classes.length) return ''
  return props.characterData.classes[0].class_name
})

// Helper to get primary subclass
const primarySubclass = computed(() => {
  if (!props.characterData.classes.length) return null
  return props.characterData.classes[0].subclass
})

// Class selection state
const selectedClassName = ref('')

// Spells Known classes
const spellsKnownClasses = ['Bard', 'Ranger', 'Sorcerer', 'Warlock']

const isSpellsKnownCaster = computed(() => {
  return spellsKnownClasses.includes(selectedClassName.value)
})

// All available classes for multiclassing
const allClasses = [
  { name: 'Barbarian', prereqs: { strength: 13 } },
  { name: 'Bard', prereqs: { charisma: 13 } },
  { name: 'Cleric', prereqs: { wisdom: 13 } },
  { name: 'Druid', prereqs: { wisdom: 13 } },
  { name: 'Fighter', prereqs: { strength: 13, dexterity: 13, or: true } },
  { name: 'Monk', prereqs: { dexterity: 13, wisdom: 13 } },
  { name: 'Paladin', prereqs: { strength: 13, charisma: 13 } },
  { name: 'Ranger', prereqs: { dexterity: 13, wisdom: 13 } },
  { name: 'Rogue', prereqs: { dexterity: 13 } },
  { name: 'Sorcerer', prereqs: { charisma: 13 } },
  { name: 'Warlock', prereqs: { charisma: 13 } },
  { name: 'Wizard', prereqs: { intelligence: 13 } },
]

// Check if character meets multiclass prerequisites
const meetsPrereqs = (prereqs: any): boolean => {
  const abilities = props.characterData.abilities
  if (prereqs.or) {
    // Fighter: STR 13 OR DEX 13
    return abilities.strength >= (prereqs.strength || 0) || abilities.dexterity >= (prereqs.dexterity || 0)
  }
  // All prereqs must be met
  if (prereqs.strength && abilities.strength < prereqs.strength) return false
  if (prereqs.dexterity && abilities.dexterity < prereqs.dexterity) return false
  if (prereqs.constitution && abilities.constitution < prereqs.constitution) return false
  if (prereqs.intelligence && abilities.intelligence < prereqs.intelligence) return false
  if (prereqs.wisdom && abilities.wisdom < prereqs.wisdom) return false
  if (prereqs.charisma && abilities.charisma < prereqs.charisma) return false
  return true
}

// Available classes for multiclassing (excluding current classes)
const availableMulticlasses = computed(() => {
  const currentClassNames = props.characterData.classes.map(c => c.class_name)
  return allClasses
    .filter(c => !currentClassNames.includes(c.name))
    .map(c => ({
      name: c.name,
      meetsPrereqs: meetsPrereqs(c.prereqs)
    }))
})

// Steps - include Class and Spells steps as needed
const steps = computed(() => {
  const baseSteps = ['Class', 'Hit Points', 'Abilities', 'Features', 'Review']
  if (isSpellsKnownCaster.value) {
    // Insert Spells before Features
    return ['Class', 'Hit Points', 'Abilities', 'Spells', 'Features', 'Review']
  }
  return baseSteps
})

const currentStep = ref(0)

// HP state
const hpMethod = ref<'average' | 'roll' | 'manual'>('average')
const hpRoll = ref<number | null>(null)
const manualHpEntry = ref<number | null>(null)

// ASI state
const asiMode = ref<'asi' | 'feat'>('asi')
const asiMethod = ref<'single' | 'split'>('single')
const asiPrimary = ref('')
const asiSecondary = ref('')
const selectedFeat = ref('')

// Spell state
const classDetails = ref<ClassDetails | null>(null)
const selectedSpells = ref<SpellReferenceInput[]>([])
const selectedSpellsGrouped = ref<Record<number, SpellReferenceInput[]>>({})

// Submission state
const isSubmitting = ref(false)

// Computed values
const newLevel = computed(() => props.characterData.level + 1)

const hitDice = computed(() => {
  // Get hit dice based on selected class
  const classHitDice: Record<string, string> = {
    'Barbarian': 'd12',
    'Fighter': 'd10',
    'Paladin': 'd10',
    'Ranger': 'd10',
    'Bard': 'd8',
    'Cleric': 'd8',
    'Druid': 'd8',
    'Monk': 'd8',
    'Rogue': 'd8',
    'Warlock': 'd8',
    'Sorcerer': 'd6',
    'Wizard': 'd6',
  }
  return classHitDice[selectedClassName.value] || 'd8'
})

const hitDiceMax = computed(() => {
  return parseInt(hitDice.value.substring(1))
})

const conModifier = computed(() => {
  return Math.floor((props.characterData.abilities.constitution - 10) / 2)
})

const averageHp = computed(() => {
  return Math.floor(hitDiceMax.value / 2) + 1 + conModifier.value
})

const totalHpGain = computed(() => {
  if (hpMethod.value === 'average') {
    return averageHp.value
  } else if (hpMethod.value === 'roll') {
    return (hpRoll.value || 0) + conModifier.value
  } else {
    return (manualHpEntry.value || 0) + conModifier.value
  }
})

const actualHpRoll = computed(() => {
  if (hpMethod.value === 'roll') return hpRoll.value
  if (hpMethod.value === 'manual') return manualHpEntry.value
  return null
})

// ASI levels vary by class, but standard is 4, 8, 12, 16, 19
// Fighter gets extra at 6 and 14, Rogue at 10
// For multiclass, ASI is based on class level, not total level
const hasAsi = computed(() => {
  // Get the level in the selected class after leveling
  const existingClass = props.characterData.classes.find(c => c.class_name === selectedClassName.value)
  const classLevel = existingClass ? existingClass.level + 1 : 1
  const className = selectedClassName.value

  // Standard ASI levels
  const standardAsiLevels = [4, 8, 12, 16, 19]

  // Class-specific extra ASI levels
  if (className === 'Fighter' && (classLevel === 6 || classLevel === 14)) {
    return true
  }
  if (className === 'Rogue' && classLevel === 10) {
    return true
  }

  return standardAsiLevels.includes(classLevel)
})

const abilities = ['Strength', 'Dexterity', 'Constitution', 'Intelligence', 'Wisdom', 'Charisma']

const availableSecondaryAbilities = computed(() => {
  return abilities.filter(a => a !== asiPrimary.value)
})

// Placeholder feats - in full implementation, load from catalog
const availableFeats = computed(() => {
  return [
    'Alert',
    'Athlete',
    'Actor',
    'Charger',
    'Crossbow Expert',
    'Defensive Duelist',
    'Dual Wielder',
    'Dungeon Delver',
    'Durable',
    'Elemental Adept',
    'Grappler',
    'Great Weapon Master',
    'Healer',
    'Heavily Armored',
    'Heavy Armor Master',
    'Inspiring Leader',
    'Keen Mind',
    'Lightly Armored',
    'Linguist',
    'Lucky',
    'Mage Slayer',
    'Magic Initiate',
    'Martial Adept',
    'Medium Armor Master',
    'Mobile',
    'Moderately Armored',
    'Mounted Combatant',
    'Observant',
    'Polearm Master',
    'Resilient',
    'Ritual Caster',
    'Savage Attacker',
    'Sentinel',
    'Sharpshooter',
    'Shield Master',
    'Skilled',
    'Skulker',
    'Spell Sniper',
    'Tavern Brawler',
    'Tough',
    'War Caster',
    'Weapon Master',
  ]
})

// Placeholder features - in full implementation, load from class data
const newFeatures = computed(() => {
  // Get the level in the selected class after leveling
  const existingClass = props.characterData.classes.find(c => c.class_name === selectedClassName.value)
  const classLevel = existingClass ? existingClass.level + 1 : 1
  const className = selectedClassName.value

  // This is a simplified placeholder
  // In full implementation, fetch from catalog_class_features
  const features: string[] = []

  // Generic features based on class level
  if (classLevel === 5) {
    features.push('Extra Attack')
  }
  if (classLevel === 9 && className === 'Fighter') {
    features.push('Indomitable')
  }

  return features
})

// Max spell level by class level (half-casters get spells slower)
const maxSpellLevel = computed(() => {
  // Get the level in the selected class after leveling
  const existingClass = props.characterData.classes.find(c => c.class_name === selectedClassName.value)
  const classLevel = existingClass ? existingClass.level + 1 : 1
  const className = selectedClassName.value

  // Half-casters (Ranger, Paladin)
  if (className === 'Ranger' || className === 'Paladin') {
    if (classLevel < 2) return 0
    return Math.min(5, Math.ceil((classLevel - 1) / 4) + 1)
  }

  // Full casters and pact magic (Warlock)
  if (classLevel < 1) return 0
  return Math.min(9, Math.ceil(classLevel / 2))
})

// Current known spells from character data
const currentKnownSpells = computed(() => {
  return props.characterData.spells?.known_spells || []
})

// Spells allowed at the new level based on class progression
const spellsAllowed = computed((): Record<number, number> => {
  if (!classDetails.value) return {}

  const result: Record<number, number> = {}
  const levelIndex = newLevel.value - 1 // Arrays are 0-indexed

  // Cantrips (level 0)
  if (classDetails.value.cantripProgression && classDetails.value.cantripProgression[levelIndex] !== undefined) {
    result[0] = classDetails.value.cantripProgression[levelIndex]
  }

  // Spells known (total for all levels 1+)
  if (classDetails.value.spellsKnownProgression && classDetails.value.spellsKnownProgression[levelIndex] !== undefined) {
    // For now, put all spell slots under level 1 - this is a simplification
    // A more accurate approach would track by spell level
    result[1] = classDetails.value.spellsKnownProgression[levelIndex]
  }

  return result
})

// Load class details for spell progressions
const loadClassDetails = async () => {
  if (!isSpellsKnownCaster.value) return

  try {
    const details = await invoke<ClassDetails>('get_class_details', {
      className: selectedClassName.value,
      classSource: 'PHB' // TODO: Get actual source
    })
    classDetails.value = details
  } catch (e) {
    console.error('Failed to load class details:', e)
    classDetails.value = null
  }
}

// Handle spell selection updates from SpellSelector
const handleSpellUpdate = (spells: SpellReferenceInput[]) => {
  selectedSpells.value = spells
}

const handleSpellGroupedUpdate = (grouped: Record<number, SpellReferenceInput[]>) => {
  selectedSpellsGrouped.value = grouped
}

const currentAbilityScore = (ability: string): number => {
  const key = ability.toLowerCase() as keyof typeof props.characterData.abilities
  return props.characterData.abilities[key]
}

const canProceed = computed(() => {
  const currentStepName = steps.value[currentStep.value]

  switch (currentStepName) {
    case 'Class':
      return selectedClassName.value !== ''
    case 'Hit Points':
      if (hpMethod.value === 'average') return true
      if (hpMethod.value === 'roll') return hpRoll.value !== null
      if (hpMethod.value === 'manual') return manualHpEntry.value !== null && manualHpEntry.value > 0
      return false
    case 'Abilities':
      if (!hasAsi.value) return true
      if (asiMode.value === 'asi') {
        if (asiMethod.value === 'single') {
          return asiPrimary.value !== ''
        } else {
          return asiPrimary.value !== '' && asiSecondary.value !== ''
        }
      } else {
        return selectedFeat.value !== ''
      }
    case 'Spells':
      // For spells, just need at least one selected (can be permissive)
      return selectedSpells.value.length > 0
    case 'Features':
      return true
    case 'Review':
      return true
    default:
      return true
  }
})

// Methods
const closeDialog = () => {
  emit('close')
}

const rollHitPoints = () => {
  const max = hitDiceMax.value
  hpRoll.value = Math.floor(Math.random() * max) + 1
}

const prevStep = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

const nextStep = () => {
  if (currentStep.value < steps.value.length - 1) {
    currentStep.value++
  }
}

const confirmLevelUp = async () => {
  isSubmitting.value = true

  try {
    // Build ASI data in the format expected by backend AsiOrFeat enum
    let asiData: string | null = null
    if (hasAsi.value && asiMode.value === 'asi') {
      if (asiMethod.value === 'single') {
        asiData = JSON.stringify({
          AbilityScoreImprovement: {
            ability1: asiPrimary.value.toLowerCase(),
            increase1: 2,
            ability2: null,
            increase2: null
          }
        })
      } else {
        asiData = JSON.stringify({
          AbilityScoreImprovement: {
            ability1: asiPrimary.value.toLowerCase(),
            increase1: 1,
            ability2: asiSecondary.value.toLowerCase(),
            increase2: 1
          }
        })
      }
    }

    // Get subclass for the selected class
    const existingClass = props.characterData.classes.find(c => c.class_name === selectedClassName.value)
    const subclass = existingClass?.subclass || null

    // Extract cantrips and known spells from selection
    const cantrips: SpellReferenceInput[] = selectedSpellsGrouped.value[0] || []
    const knownSpells: SpellReferenceInput[] = []
    for (const level in selectedSpellsGrouped.value) {
      if (parseInt(level) > 0) {
        knownSpells.push(...selectedSpellsGrouped.value[parseInt(level)])
      }
    }

    const request: LevelUpRequest = {
      class_name: selectedClassName.value,
      class_source: 'PHB', // TODO: Get actual source
      hit_points_roll: hpMethod.value === 'average' ? null : actualHpRoll.value,
      take_average_hp: hpMethod.value === 'average',
      subclass: subclass,
      ability_score_improvement: asiData,
      feat: asiMode.value === 'feat' ? selectedFeat.value : null,
      new_spell_slots: null, // TODO: Implement spell slot updates
      new_known_spells: isSpellsKnownCaster.value && knownSpells.length > 0 ? knownSpells : null,
      new_cantrips: isSpellsKnownCaster.value && cantrips.length > 0 ? cantrips : null,
    }

    await characterStore.levelUpCharacter(props.characterId, request)
    emit('completed')
    closeDialog()
  } catch (e) {
    console.error('Failed to level up character:', e)
  } finally {
    isSubmitting.value = false
  }
}

// Reset state when dialog opens
watch(() => props.visible, async (visible) => {
  if (visible) {
    // Initialize selected class to primary class
    selectedClassName.value = primaryClassName.value
    console.log('LevelUpDialog opened for class:', selectedClassName.value)
    console.log('isSpellsKnownCaster:', isSpellsKnownCaster.value)
    currentStep.value = 0
    hpMethod.value = 'average'
    hpRoll.value = null
    manualHpEntry.value = null
    asiMode.value = 'asi'
    asiMethod.value = 'single'
    asiPrimary.value = ''
    asiSecondary.value = ''
    selectedFeat.value = ''
    isSubmitting.value = false

    // Initialize spell selection with current known spells
    selectedSpells.value = [...currentKnownSpells.value]

    // Load class details for spell casters
    if (isSpellsKnownCaster.value) {
      await loadClassDetails()
    }
  }
})

// Reload class details when selected class changes (for spell casters)
watch(selectedClassName, async () => {
  if (props.visible && isSpellsKnownCaster.value) {
    await loadClassDetails()
  }
})
</script>

<style scoped>
/* Domain-specific styles for Level Up Dialog */

.dialog-progress {
  display: flex;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-sm);
}

.progress-step {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  opacity: 0.5;
}

.progress-step.active,
.progress-step.completed {
  opacity: 1;
}

.step-number {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--color-surface-variant);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  font-weight: 600;
}

.progress-step.active .step-number {
  background: var(--color-primary-500);
  color: white;
}

.progress-step.completed .step-number {
  background: var(--color-success);
  color: white;
}

.step-label {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.dialog-step h3 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
  color: var(--color-text);
}

.step-description {
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-lg);
}

/* HP Options */
.hp-options {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.hp-option {
  padding: var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.hp-option:hover {
  border-color: var(--color-primary-300);
}

.hp-option.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-variant);
}

.option-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xs);
}

.option-title {
  font-weight: 600;
  color: var(--color-text);
}

.option-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  margin: 0;
}

.roll-section,
.manual-section {
  margin-top: var(--spacing-md);
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  flex-wrap: wrap;
}

.manual-section label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.manual-input {
  width: 80px;
}

.form-input {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.btn-roll {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
}

.btn-roll:hover {
  background: var(--color-primary-600);
}

.roll-result {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.roll-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-primary-500);
}

.roll-total {
  color: var(--color-text-secondary);
}

/* ASI Options */
.asi-options {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.asi-option {
  padding: var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.asi-option:hover {
  border-color: var(--color-primary-300);
}

.asi-option.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-variant);
}

.asi-selection,
.feat-selection {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.asi-method {
  display: flex;
  gap: var(--spacing-lg);
  margin-bottom: var(--spacing-md);
}

.asi-method label {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  cursor: pointer;
}

.ability-select,
.ability-select-split {
  margin-top: var(--spacing-md);
}

.ability-select-split {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.ability-select label {
  display: block;
  margin-bottom: var(--spacing-xs);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.form-select {
  width: 100%;
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.no-asi-message {
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Features List */
.features-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.feature-item {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.feature-name {
  font-weight: 500;
  color: var(--color-text);
}

.no-features {
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Class Selection */
.class-options {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.class-section h4 {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.class-option {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.class-option:hover:not(.disabled) {
  border-color: var(--color-primary-300);
}

.class-option.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-variant);
}

.class-option.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.class-option .class-name {
  font-weight: 500;
  color: var(--color-text);
  flex: 1;
}

.class-option .class-level {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.class-option .prereq-warning {
  font-size: 0.75rem;
  color: var(--color-error);
  font-style: italic;
}

/* Review Summary */
.review-summary {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.review-item {
  display: flex;
  justify-content: space-between;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.review-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.review-section-header {
  font-weight: 600;
  font-size: 0.875rem;
  color: var(--color-text);
  padding-bottom: var(--spacing-xs);
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-xs);
}

.review-label {
  font-weight: 500;
  color: var(--color-text-secondary);
  flex-shrink: 0;
  white-space: nowrap;
  margin-right: var(--spacing-sm);
}

.review-value {
  color: var(--color-text);
}

.review-detail {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

/* Footer spacer for wizard layout */
.footer-spacer {
  flex: 1;
}

.loading {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
}
</style>
