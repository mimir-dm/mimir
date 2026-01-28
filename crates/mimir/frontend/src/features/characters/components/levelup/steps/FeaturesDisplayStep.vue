<template>
  <div class="features-display-step">
    <h3 class="step-heading">Features Gained at Level {{ levelUp.newClassLevel.value }}</h3>
    <p class="step-description">
      As a {{ levelUp.selectedClass.value?.class_name }} reaching level
      {{ levelUp.newClassLevel.value }}, you gain the following features.
    </p>

    <!-- Loading State -->
    <div v-if="isLoading" class="loading-indicator">Loading class features...</div>

    <!-- Features List -->
    <div v-else-if="featuresAtLevel.length > 0" class="features-list">
      <div v-for="feature in featuresAtLevel" :key="feature.name" class="feature-card">
        <div class="feature-header">
          <h4 class="feature-name">{{ feature.name }}</h4>
          <span class="feature-source">{{ feature.source }}</span>
        </div>
        <div
          v-if="feature.description"
          class="feature-description"
          v-html="formatText(feature.description)"
        ></div>
        <div v-if="feature.entries && feature.entries.length > 0" class="feature-entries">
          <div v-for="(entry, idx) in feature.entries" :key="idx" class="feature-entry">
            <template v-if="typeof entry === 'string'">
              <span v-html="formatText(entry)"></span>
            </template>
            <template v-else-if="entry.type === 'list'">
              <ul class="entry-list">
                <li
                  v-for="(item, itemIdx) in entry.items"
                  :key="itemIdx"
                  v-html="formatText(item)"
                ></li>
              </ul>
            </template>
            <template v-else-if="entry.type === 'table'">
              <div class="entry-table-note">See rulebook for table details.</div>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- No Features Message -->
    <div v-else class="no-features">
      <p>No specific features are gained at this level.</p>
      <p class="note">
        You may still gain hit points, spell slots, or other class progression benefits.
      </p>
    </div>

    <!-- Subclass Features -->
    <div v-if="subclassFeaturesAtLevel.length > 0" class="subclass-section">
      <h4 class="section-title">
        {{ levelUp.subclass.value?.name || existingSubclassName }} Features
      </h4>
      <div class="features-list">
        <div v-for="feature in subclassFeaturesAtLevel" :key="feature.name" class="feature-card">
          <div class="feature-header">
            <h4 class="feature-name">{{ feature.name }}</h4>
            <span class="feature-source">{{ feature.source }}</span>
          </div>
          <div v-if="feature.description" class="feature-description">
            {{ feature.description }}
          </div>
        </div>
      </div>
    </div>

    <!-- Summary Section -->
    <div class="summary-section">
      <h4 class="summary-title">Level {{ levelUp.newClassLevel.value }} Summary</h4>
      <ul class="summary-list">
        <li v-if="proficiencyBonusIncreases">
          <strong>Proficiency Bonus:</strong> Increases to +{{ newProficiencyBonus }}
        </li>
        <li v-if="isAsiLevel">
          <strong>Ability Score Improvement:</strong> You can increase ability scores or take a feat
        </li>
        <li v-if="cantripsKnown > 0">
          <strong>Cantrips Known:</strong> {{ cantripsKnown }}
        </li>
        <li v-if="spellsKnown !== null">
          <strong>Spells Known:</strong> {{ spellsKnown }}
        </li>
        <li v-if="maxSpellLevel > 0">
          <strong>Max Spell Level:</strong> {{ maxSpellLevel }}
        </li>
        <li v-if="extraAttack">
          <strong>Extra Attack:</strong> You can attack {{ extraAttackCount }} times
        </li>
        <li v-if="channelDivinityUses > 0">
          <strong>Channel Divinity Uses:</strong> {{ channelDivinityUses }}
        </li>
        <li v-if="rageDamage > 0">
          <strong>Rage Damage:</strong> +{{ rageDamage }}
        </li>
        <li v-if="sneakAttackDice > 0">
          <strong>Sneak Attack:</strong> {{ sneakAttackDice }}d6
        </li>
        <li v-if="martialArtsDie">
          <strong>Martial Arts Die:</strong> {{ martialArtsDie }}
        </li>
        <li v-if="sorceryPoints > 0">
          <strong>Sorcery Points:</strong> {{ sorceryPoints }}
        </li>
        <li v-if="kiPoints > 0">
          <strong>Ki Points:</strong> {{ kiPoints }}
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import type { Character } from '@/types/character'
import type { LevelUpComposable } from '@/features/characters/composables/useLevelUp'
import { invoke } from '@tauri-apps/api/core'
import { processFormattingTags } from '@/features/sources/utils/textFormatting'

interface ClassFeature {
  name: string
  source: string
  level: number
  description?: string
  entries?: Array<string | { type: string; items?: string[] }>
}

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

// Helper to format 5etools markup
function formatText(text: string | unknown): string {
  if (!text || typeof text !== 'string') return ''
  return processFormattingTags(text)
}

// Loading state
const isLoading = ref(false)

// Features data
const classFeatures = ref<ClassFeature[]>([])
const subclassFeatures = ref<ClassFeature[]>([])

// Computed values
const className = computed(() => props.levelUp.selectedClass.value?.class_name ?? '')
const classSource = computed(() => props.levelUp.selectedClass.value?.class_source ?? 'PHB')
const classLevel = computed(() => props.levelUp.newClassLevel.value)
const totalLevel = computed(() => {
  const otherLevels = props.character.classes
    .filter((c) => c.class_name.toLowerCase() !== className.value.toLowerCase())
    .reduce((sum, c) => sum + c.level, 0)
  return otherLevels + classLevel.value
})

const existingSubclassName = computed(() => {
  const charClass = props.character.classes.find(
    (c) => c.class_name.toLowerCase() === className.value.toLowerCase()
  )
  return charClass?.subclass_name || ''
})

// Features at current level
const featuresAtLevel = computed(() => {
  return classFeatures.value.filter((f) => f.level === classLevel.value)
})

const subclassFeaturesAtLevel = computed(() => {
  return subclassFeatures.value.filter((f) => f.level === classLevel.value)
})

// Proficiency bonus
const newProficiencyBonus = computed(() => {
  return Math.floor((totalLevel.value - 1) / 4) + 2
})

const oldProficiencyBonus = computed(() => {
  const oldTotal = totalLevel.value - 1
  if (oldTotal <= 0) return 2
  return Math.floor((oldTotal - 1) / 4) + 2
})

const proficiencyBonusIncreases = computed(() => {
  return newProficiencyBonus.value > oldProficiencyBonus.value
})

// ASI levels by class
const isAsiLevel = computed(() => {
  const asiLevels: Record<string, number[]> = {
    fighter: [4, 6, 8, 12, 14, 16, 19],
    rogue: [4, 8, 10, 12, 16, 19],
    default: [4, 8, 12, 16, 19]
  }
  const levels = asiLevels[className.value.toLowerCase()] || asiLevels.default
  return levels.includes(classLevel.value)
})

// Cantrips known by class and level
const cantripsKnown = computed(() => {
  const cantripsTable: Record<string, number[]> = {
    bard: [0, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
    cleric: [0, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
    druid: [0, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
    sorcerer: [0, 4, 4, 4, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6],
    warlock: [0, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
    wizard: [0, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5]
  }
  const table = cantripsTable[className.value.toLowerCase()]
  if (!table) return 0
  return table[classLevel.value] || 0
})

// Spells known by class and level (for known casters)
const spellsKnown = computed(() => {
  const spellsTable: Record<string, number[]> = {
    bard: [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 15, 16, 18, 19, 19, 20, 22, 22],
    ranger: [0, 0, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11],
    sorcerer: [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 12, 13, 13, 14, 14, 15, 15, 15],
    warlock: [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15]
  }
  const table = spellsTable[className.value.toLowerCase()]
  if (!table) return null
  return table[classLevel.value] || 0
})

// Max spell level
const maxSpellLevel = computed(() => {
  const casterType = getCasterType(className.value.toLowerCase())
  if (!casterType) return 0

  if (casterType === 'full') {
    return Math.min(9, Math.ceil(classLevel.value / 2))
  } else if (casterType === 'half') {
    if (classLevel.value < 2) return 0
    return Math.min(5, Math.ceil((classLevel.value - 1) / 4) + 1)
  } else if (casterType === 'third') {
    if (classLevel.value < 3) return 0
    return Math.min(4, Math.ceil((classLevel.value - 2) / 4) + 1)
  } else if (casterType === 'pact') {
    return Math.min(5, Math.ceil(classLevel.value / 4) + (classLevel.value >= 9 ? 1 : 0))
  }
  return 0
})

// Extra Attack
const extraAttack = computed(() => {
  const extraAttackClasses = ['fighter', 'barbarian', 'monk', 'paladin', 'ranger']
  if (!extraAttackClasses.includes(className.value.toLowerCase())) return false
  if (className.value.toLowerCase() === 'fighter') {
    return [5, 11, 20].includes(classLevel.value)
  }
  return classLevel.value === 5
})

const extraAttackCount = computed(() => {
  if (className.value.toLowerCase() === 'fighter') {
    if (classLevel.value >= 20) return 4
    if (classLevel.value >= 11) return 3
    if (classLevel.value >= 5) return 2
  }
  if (classLevel.value >= 5) return 2
  return 1
})

// Class-specific resources
const channelDivinityUses = computed(() => {
  if (!['cleric', 'paladin'].includes(className.value.toLowerCase())) return 0
  if (className.value.toLowerCase() === 'cleric') {
    if (classLevel.value >= 18) return 3
    if (classLevel.value >= 6) return 2
    if (classLevel.value >= 2) return 1
  }
  if (className.value.toLowerCase() === 'paladin') {
    if (classLevel.value >= 3) return 1
  }
  return 0
})

const rageDamage = computed(() => {
  if (className.value.toLowerCase() !== 'barbarian') return 0
  if (classLevel.value >= 16) return 4
  if (classLevel.value >= 9) return 3
  if (classLevel.value >= 1) return 2
  return 0
})

const sneakAttackDice = computed(() => {
  if (className.value.toLowerCase() !== 'rogue') return 0
  return Math.ceil(classLevel.value / 2)
})

const martialArtsDie = computed(() => {
  if (className.value.toLowerCase() !== 'monk') return ''
  if (classLevel.value >= 17) return '1d10'
  if (classLevel.value >= 11) return '1d8'
  if (classLevel.value >= 5) return '1d6'
  return '1d4'
})

const sorceryPoints = computed(() => {
  if (className.value.toLowerCase() !== 'sorcerer') return 0
  if (classLevel.value >= 2) return classLevel.value
  return 0
})

const kiPoints = computed(() => {
  if (className.value.toLowerCase() !== 'monk') return 0
  if (classLevel.value >= 2) return classLevel.value
  return 0
})

// Helper functions
function getCasterType(cls: string): 'full' | 'half' | 'third' | 'pact' | null {
  const fullCasters = ['bard', 'cleric', 'druid', 'sorcerer', 'wizard']
  const halfCasters = ['paladin', 'ranger']
  const thirdCasters = ['eldritch knight', 'arcane trickster'] // subclasses
  const pactCasters = ['warlock']

  if (fullCasters.includes(cls)) return 'full'
  if (halfCasters.includes(cls)) return 'half'
  if (thirdCasters.includes(cls)) return 'third'
  if (pactCasters.includes(cls)) return 'pact'
  return null
}

// Load features from catalog
async function loadClassFeatures() {
  if (!className.value) return

  isLoading.value = true
  try {
    const result = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        level: number
        entries?: unknown[]
      }>
    }>('list_class_features', {
      className: className.value,
      classSource: classSource.value
    })

    if (result.success && result.data) {
      classFeatures.value = result.data.map((f) => ({
        name: f.name,
        source: f.source,
        level: f.level,
        description: extractFirstEntry(f.entries),
        entries: f.entries as ClassFeature['entries']
      }))
    }
  } catch (e) {
    console.error('Error loading class features:', e)
  } finally {
    isLoading.value = false
  }
}

async function loadSubclassFeatures() {
  const subclassName = props.levelUp.subclass.value?.name || existingSubclassName.value
  if (!subclassName || !className.value) return

  try {
    const result = await invoke<{
      success: boolean
      data: {
        name: string
        source: string
        subclassFeatures?: Array<{ classFeature?: string; subclassFeature?: string }>
      }
    }>('get_subclass_by_name', {
      name: subclassName,
      className: className.value,
      source: props.levelUp.subclass.value?.source || 'PHB'
    })

    if (result.success && result.data?.subclassFeatures) {
      // Parse subclass features and filter by level
      const features: ClassFeature[] = []
      for (const feat of result.data.subclassFeatures) {
        const ref = feat.subclassFeature || feat.classFeature || ''
        const parts = ref.split('|')
        if (parts.length >= 4) {
          const name = parts[0]
          const level = parseInt(parts[3], 10)
          if (!isNaN(level)) {
            features.push({
              name,
              source: result.data.source,
              level
            })
          }
        }
      }
      subclassFeatures.value = features
    }
  } catch (e) {
    console.error('Error loading subclass features:', e)
  }
}

function extractFirstEntry(entries: unknown[] | undefined): string | undefined {
  if (!entries || !Array.isArray(entries)) return undefined
  const firstString = entries.find((e) => typeof e === 'string')
  return firstString as string | undefined
}

// Watch for class changes
watch(
  () => props.levelUp.selectedClass.value,
  () => {
    loadClassFeatures()
  },
  { immediate: true }
)

watch(
  () => props.levelUp.subclass.value,
  () => {
    loadSubclassFeatures()
  },
  { immediate: true }
)

onMounted(() => {
  loadClassFeatures()
  loadSubclassFeatures()
})
</script>

<style scoped>
.features-display-step {
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

.features-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.feature-card {
  padding: var(--spacing-md);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  border-left: 4px solid var(--color-primary-500);
}

.feature-header {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

.feature-name {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.feature-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.feature-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.feature-entries {
  margin-top: var(--spacing-sm);
}

.feature-entry {
  font-size: 0.875rem;
  color: var(--color-text);
  line-height: 1.5;
  margin-bottom: var(--spacing-sm);
}

.entry-list {
  margin: var(--spacing-xs) 0;
  padding-left: var(--spacing-lg);
}

.entry-list li {
  margin-bottom: var(--spacing-xs);
}

.entry-table-note {
  font-style: italic;
  color: var(--color-text-secondary);
}

.no-features {
  padding: var(--spacing-lg);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  text-align: center;
}

.no-features p {
  margin: 0;
  color: var(--color-text);
}

.no-features .note {
  margin-top: var(--spacing-sm);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.subclass-section {
  margin-top: var(--spacing-md);
}

.section-title {
  margin: 0 0 var(--spacing-md);
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-primary-600);
}

.summary-section {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.summary-title {
  margin: 0 0 var(--spacing-md);
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.summary-list {
  margin: 0;
  padding-left: var(--spacing-lg);
}

.summary-list li {
  margin-bottom: var(--spacing-sm);
  font-size: 0.875rem;
  color: var(--color-text);
}

.summary-list li strong {
  color: var(--color-text);
}

/* 5etools formatted content styles */
.feature-description :deep(.book-ref),
.feature-entry :deep(.book-ref) {
  font-style: italic;
  color: var(--color-text-secondary);
}

.feature-description :deep(.filter-ref),
.feature-entry :deep(.filter-ref) {
  color: var(--color-primary-500);
}

.feature-description :deep(.spell-ref),
.feature-description :deep(.condition-ref),
.feature-description :deep(.item-ref),
.feature-entry :deep(.spell-ref),
.feature-entry :deep(.condition-ref),
.feature-entry :deep(.item-ref) {
  color: var(--color-primary-500);
  text-decoration: none;
}

.feature-description :deep(.dice-roll),
.feature-entry :deep(.dice-roll) {
  font-weight: 600;
  color: var(--color-text);
}

.feature-description :deep(.dc-check),
.feature-entry :deep(.dc-check) {
  font-weight: 600;
}
</style>
