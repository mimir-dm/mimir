<template>
  <div class="subclass-step">
    <h3 class="step-heading">Choose Your Subclass</h3>
    <p class="step-description">
      At level {{ levelUp.newClassLevel.value }}, you can choose a subclass for your
      {{ levelUp.selectedClass.value?.class_name }}.
    </p>

    <!-- Loading State -->
    <div v-if="isLoading" class="loading-indicator">Loading subclasses...</div>

    <!-- Subclass Grid from Catalog -->
    <div v-else-if="subclasses.length > 0" class="subclass-grid">
      <button
        v-for="sub in subclasses"
        :key="`${sub.name}-${sub.source}`"
        type="button"
        class="subclass-card"
        :class="{ selected: isSelected(sub) }"
        @click="selectSubclass(sub)"
      >
        <div class="subclass-name">{{ sub.name }}</div>
        <div class="subclass-source">{{ sub.source }}</div>
        <div v-if="sub.shortDescription" class="subclass-description">
          {{ sub.shortDescription }}
        </div>
      </button>
    </div>

    <!-- Manual Entry Fallback -->
    <div v-else class="manual-section">
      <div class="placeholder-content">
        <p class="placeholder-text">
          No subclasses found in catalog for {{ levelUp.selectedClass.value?.class_name }}.
        </p>
        <p class="placeholder-note">You can enter a subclass manually below.</p>
      </div>

      <div class="manual-entry">
        <div class="form-group">
          <label class="form-label" for="subclass-name">Subclass Name</label>
          <input
            id="subclass-name"
            v-model="manualName"
            type="text"
            class="form-input"
            placeholder="e.g., Champion, School of Evocation"
          />
        </div>
        <div class="form-group">
          <label class="form-label" for="subclass-source">Source</label>
          <select id="subclass-source" v-model="manualSource" class="form-input">
            <option value="PHB">Player's Handbook</option>
            <option value="XGE">Xanathar's Guide</option>
            <option value="TCE">Tasha's Cauldron</option>
            <option value="SCAG">Sword Coast Adventurer's Guide</option>
          </select>
        </div>
        <button
          type="button"
          class="btn btn-primary"
          :disabled="!manualName.trim()"
          @click="selectManualSubclass"
        >
          Select Subclass
        </button>
      </div>
    </div>

    <!-- Selected subclass display -->
    <div v-if="levelUp.subclass.value" class="selected-subclass">
      <span class="selected-label">Selected:</span>
      <span class="selected-name">{{ levelUp.subclass.value.name }}</span>
      <span class="selected-source">({{ levelUp.subclass.value.source }})</span>
    </div>

    <!-- Subclass Description Panel -->
    <div v-if="selectedSubclassDetails" class="details-panel">
      <h4 class="details-title">{{ selectedSubclassDetails.name }}</h4>
      <div v-if="selectedSubclassDetails.flavorText" class="details-flavor">
        {{ selectedSubclassDetails.flavorText }}
      </div>
      <div v-if="selectedSubclassDetails.features.length > 0" class="details-features">
        <h5 class="features-heading">Features at this level:</h5>
        <ul class="features-list">
          <li v-for="feature in selectedSubclassDetails.features" :key="feature">
            {{ feature }}
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import type { Character } from '@/types/character'
import type { LevelUpComposable } from '../../../composables/useLevelUp'
import { invoke } from '@tauri-apps/api/core'

interface SubclassOption {
  name: string
  source: string
  shortDescription: string | null
  flavorText: string | null
  features: string[]
}

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

const isLoading = ref(false)
const subclasses = ref<SubclassOption[]>([])
const manualName = ref('')
const manualSource = ref('PHB')

// Selected subclass details for preview panel
const selectedSubclassDetails = computed<SubclassOption | null>(() => {
  if (!props.levelUp.subclass.value) return null
  return (
    subclasses.value.find(
      (s) =>
        s.name === props.levelUp.subclass.value?.name &&
        s.source === props.levelUp.subclass.value?.source
    ) || null
  )
})

// Load subclasses from catalog
async function loadSubclasses() {
  const className = props.levelUp.selectedClass.value?.class_name
  if (!className) return

  isLoading.value = true
  try {
    const result = await invoke<{
      success: boolean
      data: Array<{ name: string; source: string; data: string }>
      error?: string
    }>('list_subclasses_by_class', { className })

    if (result.success && result.data) {
      subclasses.value = result.data.map((sub) => {
        let data: {
          shortName?: string
          subclassFeatures?: Array<{ classFeature?: string; subclassFeature?: string }>
        } = {}
        try {
          data = JSON.parse(sub.data)
        } catch {
          // Ignore parse errors
        }

        // Extract short description (often the first part of flavor text)
        const shortDescription = data.shortName || null

        // Extract features at current level
        const currentLevel = props.levelUp.newClassLevel.value
        const features: string[] = []

        if (data.subclassFeatures && Array.isArray(data.subclassFeatures)) {
          for (const feature of data.subclassFeatures) {
            // Parse feature reference to get level
            const featureRef = feature.subclassFeature || feature.classFeature || ''
            const parts = featureRef.split('|')
            if (parts.length >= 4) {
              const featureLevel = parseInt(parts[3], 10)
              if (featureLevel === currentLevel) {
                features.push(parts[0]) // Feature name
              }
            }
          }
        }

        return {
          name: sub.name,
          source: sub.source,
          shortDescription,
          flavorText: null, // Would need to parse from data if available
          features
        }
      })
    }
  } catch (e) {
    console.error('Error loading subclasses:', e)
  } finally {
    isLoading.value = false
  }
}

function isSelected(sub: SubclassOption): boolean {
  const selected = props.levelUp.subclass.value
  if (!selected) return false
  return selected.name === sub.name && selected.source === sub.source
}

function selectSubclass(sub: SubclassOption) {
  props.levelUp.subclass.value = {
    name: sub.name,
    source: sub.source
  }
}

function selectManualSubclass() {
  if (!manualName.value.trim()) return
  props.levelUp.subclass.value = {
    name: manualName.value.trim(),
    source: manualSource.value
  }
}

// Reload subclasses when class changes
watch(
  () => props.levelUp.selectedClass.value,
  () => {
    loadSubclasses()
  }
)

// Initialize from existing state
watch(
  () => props.levelUp.subclass.value,
  (subclass) => {
    if (subclass) {
      manualName.value = subclass.name
      manualSource.value = subclass.source
    }
  },
  { immediate: true }
)

onMounted(() => {
  loadSubclasses()
})
</script>

<style scoped>
.subclass-step {
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

.loading-indicator {
  padding: var(--spacing-lg);
  text-align: center;
  color: var(--color-text-secondary);
}

.subclass-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.subclass-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--spacing-xs);
  padding: var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-surface);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: left;
}

.subclass-card:hover {
  border-color: var(--color-primary-300);
  background: var(--color-surface-variant);
}

.subclass-card.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-hover);
}

.subclass-name {
  font-weight: 600;
  font-size: 1rem;
  color: var(--color-text);
}

.subclass-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.subclass-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.manual-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.placeholder-content {
  padding: var(--spacing-lg);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  text-align: center;
}

.placeholder-text {
  margin: 0 0 var(--spacing-sm);
  color: var(--color-text);
}

.placeholder-note {
  margin: 0;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.manual-entry {
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

.selected-subclass {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-md);
  background: var(--color-success-bg, #f0fdf4);
  border: 1px solid var(--color-success, #22c55e);
  border-radius: var(--radius-md);
}

.selected-label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.selected-name {
  font-weight: 600;
  color: var(--color-success, #22c55e);
}

.selected-source {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.details-panel {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  border-left: 4px solid var(--color-primary-500);
}

.details-title {
  margin: 0 0 var(--spacing-sm);
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.details-flavor {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  font-style: italic;
  margin-bottom: var(--spacing-md);
}

.details-features {
  margin-top: var(--spacing-md);
}

.features-heading {
  margin: 0 0 var(--spacing-xs);
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
}

.features-list {
  margin: 0;
  padding-left: var(--spacing-lg);
}

.features-list li {
  font-size: 0.875rem;
  color: var(--color-text);
  margin-bottom: var(--spacing-xs);
}
</style>
