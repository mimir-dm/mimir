<template>
  <AppModal
    :visible="visible"
    :title="`Level Up: ${characterData.name}`"
    size="lg"
    :closable="!levelUp.isSubmitting.value"
    :close-on-overlay="!levelUp.isSubmitting.value"
    @close="handleClose"
  >
    <div class="levelup-wizard">
      <!-- Progress Indicator -->
      <div class="wizard-progress">
        <div
          v-for="(step, index) in levelUp.visibleSteps.value"
          :key="step.id"
          class="progress-step"
          :class="{
            active: index === levelUp.currentStepIndex.value,
            completed: index < levelUp.currentStepIndex.value,
            clickable: index < levelUp.currentStepIndex.value
          }"
          @click="handleStepClick(index)"
        >
          <div class="step-indicator">
            <span v-if="index < levelUp.currentStepIndex.value" class="step-check">&#10003;</span>
            <span v-else>{{ index + 1 }}</span>
          </div>
          <span class="step-title">{{ step.title }}</span>
        </div>
      </div>

      <!-- Step Content -->
      <div class="wizard-content">
        <component
          :is="currentStepComponent"
          v-if="currentStepComponent"
          :level-up="levelUp"
          :character="characterData"
        />
      </div>

      <!-- Error Display -->
      <div v-if="levelUp.error.value" class="wizard-error">
        {{ levelUp.error.value }}
      </div>
    </div>

    <template #footer>
      <div class="wizard-footer">
        <button
          type="button"
          class="btn btn-secondary"
          :disabled="levelUp.isSubmitting.value"
          @click="handleClose"
        >
          Cancel
        </button>

        <div class="wizard-nav">
          <button
            v-if="levelUp.canGoBack.value"
            type="button"
            class="btn btn-secondary"
            :disabled="levelUp.isSubmitting.value"
            @click="levelUp.goToPreviousStep()"
          >
            Back
          </button>

          <button
            v-if="!levelUp.isLastStep.value"
            type="button"
            class="btn btn-primary"
            :disabled="!levelUp.canGoForward.value || levelUp.isSubmitting.value"
            @click="levelUp.goToNextStep()"
          >
            Next
          </button>

          <button
            v-else
            type="button"
            class="btn btn-primary"
            :disabled="!levelUp.canGoForward.value || levelUp.isSubmitting.value"
            @click="handleSubmit"
          >
            {{ levelUp.isSubmitting.value ? 'Leveling Up...' : 'Confirm Level Up' }}
          </button>
        </div>
      </div>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { computed, ref, watch, type Component } from 'vue'
import AppModal from '@/components/shared/AppModal.vue'
import type { Character } from '@/types/character'
import { useLevelUp } from '../../composables/useLevelUp'

// Step components
import ClassSelectionStep from './steps/ClassSelectionStep.vue'
import SubclassStep from './steps/SubclassStep.vue'
import HitPointsStep from './steps/HitPointsStep.vue'
import AbilityScoreStep from './steps/AbilityScoreStep.vue'
import SpellsStep from './steps/SpellsStep.vue'
import FeatureChoicesStep from './steps/FeatureChoicesStep.vue'
import FeaturesDisplayStep from './steps/FeaturesDisplayStep.vue'
import ReviewStep from './steps/ReviewStep.vue'

const props = defineProps<{
  visible: boolean
  characterId: string
  characterData: Character
}>()

const emit = defineEmits<{
  close: []
  completed: []
}>()

// Create character ref for the composable
const characterRef = ref(props.characterData)

// Watch for character data changes
watch(
  () => props.characterData,
  (newData) => {
    characterRef.value = newData
  }
)

// Initialize level up composable
const levelUp = useLevelUp(characterRef)

// Map step IDs to components
const stepComponents: Record<string, Component> = {
  class: ClassSelectionStep,
  subclass: SubclassStep,
  hp: HitPointsStep,
  asi: AbilityScoreStep,
  spells: SpellsStep,
  features: FeatureChoicesStep,
  featuresDisplay: FeaturesDisplayStep,
  review: ReviewStep
}

// Current step component
const currentStepComponent = computed(() => {
  const step = levelUp.currentStep.value
  if (!step) return null
  return stepComponents[step.id] || null
})

// Handle step click (for going back to previous steps)
function handleStepClick(index: number) {
  if (index < levelUp.currentStepIndex.value) {
    levelUp.goToStep(index)
  }
}

// Handle close
function handleClose() {
  if (!levelUp.isSubmitting.value) {
    levelUp.reset()
    emit('close')
  }
}

// Handle submit
async function handleSubmit() {
  const result = await levelUp.submit()
  if (result) {
    levelUp.reset()
    emit('completed')
    emit('close')
  }
}

// Reset when dialog opens
watch(
  () => props.visible,
  (newVisible) => {
    if (newVisible) {
      characterRef.value = props.characterData
      levelUp.reset()
    }
  }
)
</script>

<style scoped>
.levelup-wizard {
  display: flex;
  flex-direction: column;
  min-height: 400px;
}

/* Progress Indicator */
.wizard-progress {
  display: flex;
  justify-content: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
  flex-wrap: wrap;
}

.progress-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  min-width: 80px;
}

.progress-step.clickable {
  cursor: pointer;
}

.progress-step.clickable:hover .step-indicator {
  background: var(--color-primary-100);
}

.step-indicator {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 0.875rem;
  background: var(--color-surface-variant);
  color: var(--color-text-secondary);
  transition: all var(--transition-base);
}

.progress-step.active .step-indicator {
  background: var(--color-primary-500);
  color: white;
}

.progress-step.completed .step-indicator {
  background: var(--color-success, #22c55e);
  color: white;
}

.step-check {
  font-size: 1rem;
}

.step-title {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-align: center;
  white-space: nowrap;
}

.progress-step.active .step-title {
  color: var(--color-primary-500);
  font-weight: 500;
}

/* Content Area */
.wizard-content {
  flex: 1;
  padding: var(--spacing-lg);
  overflow-y: auto;
}

/* Error Display */
.wizard-error {
  margin: 0 var(--spacing-lg);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-bg, #fef2f2);
  color: var(--color-error, #dc2626);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
}

/* Footer */
.wizard-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.wizard-nav {
  display: flex;
  gap: var(--spacing-sm);
}
</style>
