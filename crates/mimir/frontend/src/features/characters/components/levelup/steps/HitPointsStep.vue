<template>
  <div class="hitpoints-step">
    <h3 class="step-heading">Hit Points</h3>
    <p class="step-description">
      Choose how to determine your hit points for this level. Your hit die is
      <strong>d{{ hitDie }}</strong
      >.
    </p>

    <!-- HP Method Selection -->
    <div class="hp-methods">
      <button
        type="button"
        class="method-card"
        :class="{ selected: selectedMethod === 'average' }"
        @click="selectAverage"
      >
        <div class="method-name">Take Average</div>
        <div class="method-value">+{{ averageHp }}</div>
        <div class="method-description">
          Standard choice: {{ Math.floor(hitDie / 2) + 1 }} + {{ conMod }} (CON)
        </div>
      </button>

      <button
        type="button"
        class="method-card"
        :class="{ selected: selectedMethod === 'roll' }"
        @click="showRollInput"
      >
        <div class="method-name">Roll Hit Die</div>
        <div class="method-value">
          <span v-if="rollValue !== null">+{{ rollValue + conMod }}</span>
          <span v-else>d{{ hitDie }}</span>
        </div>
        <div class="method-description">Roll and enter the result (1-{{ hitDie }})</div>
      </button>

      <button
        type="button"
        class="method-card"
        :class="{ selected: selectedMethod === 'manual' }"
        @click="showManualInput"
      >
        <div class="method-name">Manual Entry</div>
        <div class="method-value">
          <span v-if="manualValue !== null">+{{ manualValue }}</span>
          <span v-else>?</span>
        </div>
        <div class="method-description">Enter a custom HP value</div>
      </button>
    </div>

    <!-- Roll Input -->
    <div v-if="selectedMethod === 'roll'" class="input-section">
      <label class="form-label">Enter your d{{ hitDie }} roll result:</label>
      <div class="roll-input-row">
        <input
          v-model.number="rollInputValue"
          type="number"
          :min="1"
          :max="hitDie"
          class="form-input roll-input"
          placeholder="1-{{ hitDie }}"
        />
        <button type="button" class="btn btn-primary" :disabled="!isValidRoll" @click="confirmRoll">
          Confirm Roll
        </button>
      </div>
      <div v-if="rollInputValue && !isValidRoll" class="input-error">
        Roll must be between 1 and {{ hitDie }}
      </div>
    </div>

    <!-- Manual Input -->
    <div v-if="selectedMethod === 'manual'" class="input-section">
      <label class="form-label">Enter HP gained:</label>
      <div class="roll-input-row">
        <input
          v-model.number="manualInputValue"
          type="number"
          :min="1"
          class="form-input roll-input"
          placeholder="HP gained"
        />
        <button
          type="button"
          class="btn btn-primary"
          :disabled="!isValidManual"
          @click="confirmManual"
        >
          Confirm
        </button>
      </div>
      <div v-if="manualInputValue && !isValidManual" class="input-error">
        HP must be at least 1
      </div>
    </div>

    <!-- HP Summary -->
    <div v-if="levelUp.hpMethod.value" class="hp-summary">
      <div class="summary-row">
        <span class="summary-label">HP Gained This Level:</span>
        <span class="summary-value">+{{ totalHpGained }}</span>
      </div>
      <div class="summary-breakdown">
        <span v-if="selectedMethod === 'average'"> Average ({{ Math.floor(hitDie / 2) + 1 }}) </span>
        <span v-else-if="selectedMethod === 'roll'"> Roll ({{ rollValue }}) </span>
        <span v-else> Manual Entry </span>
        <span v-if="conMod !== 0">
          {{ conMod >= 0 ? '+' : '' }}{{ conMod }} CON
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { Character } from '@/types/character'
import { abilityModifier } from '@/types/character'
import type { LevelUpComposable } from '@/features/characters/composables/useLevelUp'

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

// Local state
const selectedMethod = ref<'average' | 'roll' | 'manual' | null>(null)
const rollInputValue = ref<number | null>(null)
const rollValue = ref<number | null>(null)
const manualInputValue = ref<number | null>(null)
const manualValue = ref<number | null>(null)

// Computed values
const hitDie = computed(() => {
  return props.levelUp.classInfo.value?.hit_die ?? 8
})

const conMod = computed(() => {
  return abilityModifier(props.character.constitution)
})

const averageHp = computed(() => {
  // Average is (hit_die / 2) + 1 + CON mod
  return Math.floor(hitDie.value / 2) + 1 + conMod.value
})

const isValidRoll = computed(() => {
  if (rollInputValue.value === null) return false
  return rollInputValue.value >= 1 && rollInputValue.value <= hitDie.value
})

const isValidManual = computed(() => {
  if (manualInputValue.value === null) return false
  return manualInputValue.value >= 1
})

const totalHpGained = computed(() => {
  if (selectedMethod.value === 'average') {
    return averageHp.value
  } else if (selectedMethod.value === 'roll' && rollValue.value !== null) {
    return rollValue.value + conMod.value
  } else if (selectedMethod.value === 'manual' && manualValue.value !== null) {
    return manualValue.value
  }
  return 0
})

// Methods
function selectAverage() {
  selectedMethod.value = 'average'
  rollValue.value = null
  manualValue.value = null
  props.levelUp.hpMethod.value = { type: 'Average' }
}

function showRollInput() {
  selectedMethod.value = 'roll'
  manualValue.value = null
  // Don't set hpMethod until roll is confirmed
  if (rollValue.value === null) {
    props.levelUp.hpMethod.value = null
  }
}

function showManualInput() {
  selectedMethod.value = 'manual'
  rollValue.value = null
  // Don't set hpMethod until manual is confirmed
  if (manualValue.value === null) {
    props.levelUp.hpMethod.value = null
  }
}

function confirmRoll() {
  if (!isValidRoll.value || rollInputValue.value === null) return
  rollValue.value = rollInputValue.value
  props.levelUp.hpMethod.value = { type: 'Roll', value: rollInputValue.value }
}

function confirmManual() {
  if (!isValidManual.value || manualInputValue.value === null) return
  manualValue.value = manualInputValue.value
  props.levelUp.hpMethod.value = { type: 'Manual', value: manualInputValue.value }
}

// Initialize from existing state
watch(
  () => props.levelUp.hpMethod.value,
  (method) => {
    if (method) {
      if (method.type === 'Average') {
        selectedMethod.value = 'average'
      } else if (method.type === 'Roll') {
        selectedMethod.value = 'roll'
        rollValue.value = method.value
        rollInputValue.value = method.value
      } else if (method.type === 'Manual') {
        selectedMethod.value = 'manual'
        manualValue.value = method.value
        manualInputValue.value = method.value
      }
    }
  },
  { immediate: true }
)
</script>

<style scoped>
.hitpoints-step {
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

.hp-methods {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: var(--spacing-md);
}

.method-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-lg);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-surface);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: center;
}

.method-card:hover {
  border-color: var(--color-primary-300);
  background: var(--color-surface-variant);
}

.method-card.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-hover);
}

.method-name {
  font-weight: 600;
  font-size: 1rem;
  color: var(--color-text);
}

.method-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-primary-500);
}

.method-description {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.input-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.form-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.roll-input-row {
  display: flex;
  gap: var(--spacing-md);
  align-items: center;
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

.roll-input {
  width: 120px;
}

.input-error {
  font-size: 0.75rem;
  color: var(--color-error, #dc2626);
}

.hp-summary {
  padding: var(--spacing-md);
  background: var(--color-success-bg, #f0fdf4);
  border: 1px solid var(--color-success, #22c55e);
  border-radius: var(--radius-md);
}

.summary-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.summary-label {
  font-size: 0.875rem;
  color: var(--color-text);
}

.summary-value {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-success, #22c55e);
}

.summary-breakdown {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-top: var(--spacing-xs);
}
</style>
