<template>
  <div class="review-step">
    <h3 class="step-heading">Review Level Up</h3>
    <p class="step-description">Review your choices before confirming the level up.</p>

    <div class="review-sections">
      <!-- Class Info -->
      <div class="review-section">
        <h4 class="section-title">Class</h4>
        <div class="review-item">
          <span class="item-label">Class:</span>
          <span class="item-value">{{ levelUp.selectedClass.value?.class_name }}</span>
        </div>
        <div class="review-item">
          <span class="item-label">New Level:</span>
          <span class="item-value">{{ levelUp.newClassLevel.value }}</span>
        </div>
        <div class="review-item">
          <span class="item-label">Total Level:</span>
          <span class="item-value highlight">{{ levelUp.newTotalLevel.value }}</span>
        </div>
        <div v-if="levelUp.isNewClass.value" class="review-badge multiclass">Multiclass</div>
      </div>

      <!-- Subclass (if chosen) -->
      <div v-if="levelUp.subclass.value" class="review-section">
        <h4 class="section-title">Subclass</h4>
        <div class="review-item">
          <span class="item-value">{{ levelUp.subclass.value.name }}</span>
        </div>
      </div>

      <!-- Hit Points -->
      <div class="review-section">
        <h4 class="section-title">Hit Points</h4>
        <div class="review-item">
          <span class="item-label">Method:</span>
          <span class="item-value">{{ hpMethodDisplay }}</span>
        </div>
        <div class="review-item">
          <span class="item-label">HP Gained:</span>
          <span class="item-value highlight">+{{ hpGained }}</span>
        </div>
      </div>

      <!-- ASI or Feat (if chosen) -->
      <div v-if="levelUp.asiOrFeat.value" class="review-section">
        <h4 class="section-title">Ability Score / Feat</h4>
        <template v-if="levelUp.asiOrFeat.value.type === 'AbilityScoreImprovement'">
          <div class="review-item">
            <span class="item-label">{{ levelUp.asiOrFeat.value.ability1 }}:</span>
            <span class="item-value">+{{ levelUp.asiOrFeat.value.increase1 }}</span>
          </div>
          <div v-if="levelUp.asiOrFeat.value.ability2" class="review-item">
            <span class="item-label">{{ levelUp.asiOrFeat.value.ability2 }}:</span>
            <span class="item-value">+{{ levelUp.asiOrFeat.value.increase2 }}</span>
          </div>
        </template>
        <template v-else>
          <div class="review-item">
            <span class="item-label">Feat:</span>
            <span class="item-value">{{ levelUp.asiOrFeat.value.name }}</span>
          </div>
        </template>
      </div>

      <!-- Spells (if any) -->
      <div v-if="hasSpellChanges" class="review-section">
        <h4 class="section-title">Spells</h4>
        <div v-if="levelUp.spellChanges.value?.new_cantrips?.length" class="review-item">
          <span class="item-label">New Cantrips:</span>
          <span class="item-value">{{
            levelUp.spellChanges.value.new_cantrips.map((s) => s.name).join(', ')
          }}</span>
        </div>
        <div v-if="levelUp.spellChanges.value?.new_spells?.length" class="review-item">
          <span class="item-label">New Spells:</span>
          <span class="item-value">{{
            levelUp.spellChanges.value.new_spells.map((s) => s.name).join(', ')
          }}</span>
        </div>
        <div v-if="levelUp.spellChanges.value?.swap_out" class="review-item">
          <span class="item-label">Spell Swap:</span>
          <span class="item-value">
            {{ levelUp.spellChanges.value.swap_out.name }} &rarr;
            {{ levelUp.spellChanges.value.swap_in?.name }}
          </span>
        </div>
      </div>

      <!-- Features (if any) -->
      <div v-if="hasFeatureChoices" class="review-section">
        <h4 class="section-title">Class Features</h4>
        <div v-if="levelUp.featureChoices.value?.fighting_style" class="review-item">
          <span class="item-label">Fighting Style:</span>
          <span class="item-value">{{ levelUp.featureChoices.value.fighting_style.name }}</span>
        </div>
        <div v-if="levelUp.featureChoices.value?.metamagic?.length" class="review-item">
          <span class="item-label">Metamagic:</span>
          <span class="item-value">{{
            levelUp.featureChoices.value.metamagic.map((m) => m.name).join(', ')
          }}</span>
        </div>
        <div v-if="levelUp.featureChoices.value?.maneuvers?.new_maneuvers?.length" class="review-item">
          <span class="item-label">Maneuvers:</span>
          <span class="item-value">{{
            levelUp.featureChoices.value.maneuvers.new_maneuvers.map((m) => m.name).join(', ')
          }}</span>
        </div>
        <div v-if="levelUp.featureChoices.value?.invocations?.new_invocations?.length" class="review-item">
          <span class="item-label">Invocations:</span>
          <span class="item-value">{{
            levelUp.featureChoices.value.invocations.new_invocations.map((i) => i.name).join(', ')
          }}</span>
        </div>
        <div v-if="levelUp.featureChoices.value?.pact_boon" class="review-item">
          <span class="item-label">Pact Boon:</span>
          <span class="item-value">{{ levelUp.featureChoices.value.pact_boon.name }}</span>
        </div>
        <div v-if="levelUp.featureChoices.value?.expertise_skills?.length" class="review-item">
          <span class="item-label">Expertise:</span>
          <span class="item-value">{{
            levelUp.featureChoices.value.expertise_skills.join(', ')
          }}</span>
        </div>
      </div>
    </div>

    <!-- Confirmation -->
    <div class="confirmation-note">
      <p>Click <strong>Confirm Level Up</strong> to apply these changes to your character.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Character } from '@/types/character'
import { abilityModifier } from '@/types/character'
import type { LevelUpComposable } from '@/features/characters/composables/useLevelUp'

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

const hitDie = computed(() => {
  return props.levelUp.classInfo.value?.hit_die ?? 8
})

const conMod = computed(() => {
  return abilityModifier(props.character.constitution)
})

const hpMethodDisplay = computed(() => {
  const method = props.levelUp.hpMethod.value
  if (!method) return 'Not selected'

  if (method.type === 'Average') {
    return 'Average'
  } else if (method.type === 'Roll') {
    return `Rolled d${hitDie.value}`
  } else {
    return 'Manual'
  }
})

const hpGained = computed(() => {
  const method = props.levelUp.hpMethod.value
  if (!method) return 0

  if (method.type === 'Average') {
    return Math.floor(hitDie.value / 2) + 1 + conMod.value
  } else if (method.type === 'Roll') {
    return method.value + conMod.value
  } else {
    return method.value
  }
})

const hasSpellChanges = computed(() => {
  const changes = props.levelUp.spellChanges.value
  if (!changes) return false
  return (
    (changes.new_cantrips?.length ?? 0) > 0 ||
    (changes.new_spells?.length ?? 0) > 0 ||
    changes.swap_out !== undefined
  )
})

const hasFeatureChoices = computed(() => {
  const features = props.levelUp.featureChoices.value
  if (!features) return false
  return (
    features.fighting_style !== undefined ||
    (features.metamagic?.length ?? 0) > 0 ||
    (features.maneuvers?.new_maneuvers?.length ?? 0) > 0 ||
    (features.invocations?.new_invocations?.length ?? 0) > 0 ||
    features.pact_boon !== undefined ||
    (features.expertise_skills?.length ?? 0) > 0
  )
})
</script>

<style scoped>
.review-step {
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

.review-sections {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: var(--spacing-md);
}

.review-section {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.section-title {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.review-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.875rem;
}

.item-label {
  color: var(--color-text-secondary);
}

.item-value {
  font-weight: 500;
  color: var(--color-text);
}

.item-value.highlight {
  color: var(--color-primary-500);
  font-weight: 600;
}

.review-badge {
  display: inline-block;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  width: fit-content;
}

.review-badge.multiclass {
  background: var(--color-warning-bg, #fef9c3);
  color: var(--color-warning, #ca8a04);
}

.confirmation-note {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  text-align: center;
}

.confirmation-note p {
  margin: 0;
  font-size: 0.875rem;
  color: var(--color-text);
}
</style>
