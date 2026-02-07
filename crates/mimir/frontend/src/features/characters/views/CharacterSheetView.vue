<template>
  <MainLayout>
    <div class="character-sheet">
      <div v-if="loading" class="loading-state">Loading character...</div>

      <div v-else-if="error" class="error-state">
        <p>{{ error }}</p>
        <button @click="loadCharacter" class="btn btn-primary">Retry</button>
      </div>

      <template v-else-if="character">
        <!-- Header -->
        <div class="sheet-header">
          <div class="header-content">
            <button @click="goBack" class="btn-back">Back</button>
            <h1 class="character-name">{{ character.name }}</h1>
            <div class="character-subtitle">
              Level {{ totalLevel }} {{ character.race_name || '' }} {{ classString }}
            </div>
            <div v-if="character.background_name" class="character-background">
              {{ character.background_name }}
            </div>
          </div>
          <div class="header-actions">
            <span v-if="character.is_npc === 1" class="npc-badge">NPC</span>
            <span v-else-if="character.player_name" class="player-name">
              Player: {{ character.player_name }}
            </span>
            <button @click="showSourcesModal = true" class="btn btn-secondary">Sources</button>
            <button @click="printCharacter" class="btn btn-secondary">Print PDF</button>
          </div>
        </div>

        <!-- Tab Navigation -->
        <div class="tab-navigation">
          <button
            @click="activeTab = 'character'"
            :class="['tab-button', { active: activeTab === 'character' }]"
          >
            Character
          </button>
          <button
            @click="activeTab = 'equipment'"
            :class="['tab-button', { active: activeTab === 'equipment' }]"
          >
            Equipment
          </button>
          <button
            v-if="characterIsSpellcaster"
            @click="activeTab = 'spells'"
            :class="['tab-button', { active: activeTab === 'spells' }]"
          >
            Spells
          </button>
          <button
            @click="activeTab = 'details'"
            :class="['tab-button', { active: activeTab === 'details' }]"
          >
            Details
          </button>
        </div>

        <!-- Character Tab -->
        <CharacterStatsTab
          v-if="activeTab === 'character'"
          :character="character"
          :inventory="inventory"
          :classFeatures="classFeatures"
          :speed="speed"
          :characterIsSpellcaster="characterIsSpellcaster"
          :spellcastingAbility="spellcastingAbility"
          :spellSaveDC="spellSaveDC"
          :spellAttackBonus="spellAttackBonus"
        />

        <!-- Equipment Tab -->
        <EquipmentSection
          v-else-if="activeTab === 'equipment'"
          :character="character"
          :inventory="inventory"
          :loading-inventory="loadingInventory"
          @open-inventory="openInventory"
        />

        <!-- Spells Tab -->
        <SpellsSection
          v-else-if="activeTab === 'spells'"
          :character="character"
          :format-mod="formatMod"
        />

        <!-- Details Tab -->
        <CharacterDetailsTab
          v-else-if="activeTab === 'details'"
          :character="character"
          :class-features="classFeatures"
          :class-data="classData"
          :background-details="backgroundDetails"
          :subclass-details="subclassDetails"
          @open-feature-modal="openFeatureModal"
        />
      </template>
    </div>

    <!-- Inventory Manager Dialog -->
    <InventoryManager
      v-if="character"
      :visible="showInventory"
      :character-id="characterId"
      :character-data="character"
      @close="showInventory = false"
      @updated="loadInventory"
    />

    <!-- Print Dialog -->
    <CharacterPrintDialog
      v-if="character"
      :visible="showPrintDialog"
      :character-id="characterId"
      :character-name="character.name"
      @close="showPrintDialog = false"
    />

    <!-- Character Sources Modal -->
    <CharacterSourcesModal
      v-if="character"
      :visible="showSourcesModal"
      :character-id="characterId"
      @close="showSourcesModal = false"
      @saved="loadCharacter"
    />

    <!-- Cross-Reference Modal -->
    <AppModal
      :visible="modalContent.visible"
      :title="modalContent.title"
      size="md"
      @close="closeModal"
    >
      <div class="dnd-content" v-html="modalContent.content"></div>
    </AppModal>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import MainLayout from '@/shared/components/layout/MainLayout.vue'
import InventoryManager from '../components/InventoryManager.vue'
import SpellsSection from '../components/sheet/SpellsSection.vue'
import EquipmentSection from '../components/sheet/EquipmentSection.vue'
import CharacterStatsTab from '../components/sheet/CharacterStatsTab.vue'
import CharacterDetailsTab from '../components/sheet/CharacterDetailsTab.vue'
import { CharacterPrintDialog } from '@/components/print'
import { CharacterSourcesModal } from '@/components/characters'
import AppModal from '@/components/shared/AppModal.vue'
import { useCharacterStore } from '@/stores/characters'
import { useCrossReferences } from '../../sources/composables/useCrossReferences'
import { renderModalContent } from '../../sources/formatters/modalFormatters'
import type { Character, CharacterInventory } from '@/types/character'

// Class feature from catalog
interface ClassFeature {
  name: string
  source: string
  class_name: string
  class_source: string
  level: number
  data: string
  // For subclass features
  subclass_name?: string
  subclass_short_name?: string
  subclass_source?: string
}

// Background detail from catalog
interface BackgroundDetail {
  name: string
  source: string
  data: Record<string, unknown>
  fluff: string | null
}

// Subclass detail from catalog
interface SubclassDetail {
  name: string
  source: string
  class_name: string
  class_source: string
  data: Record<string, unknown>
}
import {
  getTotalLevel,
  formatClassString,
  formatModifier,
} from '@/utils/characterUtils'
import { useSpellManagement } from '../composables/useSpellManagement'

const route = useRoute()
const router = useRouter()
const characterStore = useCharacterStore()

// Cross-reference support
const {
  modalContent,
  lookupReference,
  handleCrossRefHover,
  handleCrossRefClick,
  hideTooltip,
  closeModal
} = useCrossReferences()

// Open modal for a class/subclass feature
const openFeatureModal = async (feature: ClassFeature) => {
  const refType = feature.subclass_name ? 'subclassFeature' : 'classFeature'
  const className = feature.subclass_name ? feature.subclass_short_name : feature.class_name

  // Show loading modal
  modalContent.value = {
    title: feature.name,
    content: '<p>Loading...</p>',
    visible: true
  }

  // Lookup the feature
  const refData = await lookupReference(refType, feature.name, feature.source, className)

  if (refData) {
    const contentData = refData.data || refData
    // Add ref_type for the renderer
    const contentWithType = { ...contentData, ref_type: refType }
    modalContent.value = {
      title: refData.name || feature.name,
      content: await renderModalContent(contentWithType),
      visible: true
    }
  } else {
    modalContent.value = {
      title: feature.name,
      content: '<p>No data available for this feature.</p>',
      visible: true
    }
  }
}

// State
const characterId = computed(() => route.params.id as string)
const character = ref<Character | null>(null)
const inventory = ref<CharacterInventory[]>([])
const raceData = ref<Record<string, unknown> | null>(null)
const classData = ref<Record<string, Record<string, unknown>>>({}) // keyed by class name
const classFeatures = ref<ClassFeature[]>([])
const backgroundDetails = ref<BackgroundDetail | null>(null)
const subclassDetails = ref<Record<string, SubclassDetail>>({}) // keyed by "className|subclassName"
const loading = ref(true)
const loadingInventory = ref(false)
const error = ref<string | null>(null)
const showInventory = ref(false)
const showPrintDialog = ref(false)
const showSourcesModal = ref(false)
const activeTab = ref<'character' | 'equipment' | 'spells' | 'details'>('character')

// Computed properties
const totalLevel = computed(() => (character.value ? getTotalLevel(character.value) : 0))
const classString = computed(() => (character.value ? formatClassString(character.value) : ''))

// Speed - from race catalog data, default 30ft
const speed = computed(() => {
  if (raceData.value && raceData.value.speed) {
    const speedData = raceData.value.speed
    // 5etools format: speed can be number or { walk: number, fly?: number, swim?: number, ... }
    if (typeof speedData === 'number') {
      return speedData
    }
    if (typeof speedData === 'object' && speedData !== null) {
      const walkSpeed = (speedData as Record<string, unknown>).walk
      if (typeof walkSpeed === 'number') return walkSpeed
    }
  }
  return 30 // Default
})

// Spellcasting - using composable (passed to CharacterStatsTab)
const {
  characterIsSpellcaster,
  spellcastingAbility,
  spellSaveDC,
  spellAttackBonus,
} = useSpellManagement(character, characterId)

// Methods
const formatMod = (mod: number) => formatModifier(mod)

const loadCharacter = async () => {
  loading.value = true
  error.value = null

  try {
    character.value = await characterStore.getCharacter(characterId.value)
    if (!character.value) {
      error.value = 'Character not found'
    } else {
      // Load inventory and catalog data in parallel
      // Note: Spells are loaded by SpellsSection component when it mounts
      await Promise.all([
        loadInventory(),
        loadRaceData(),
        loadClassData(),
        loadBackgroundDetails(),
        loadSubclassDetails(),
      ])
      // Load features after class data is ready (it parses from classData)
      loadClassFeatures()
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load character'
  } finally {
    loading.value = false
  }
}

const loadRaceData = async () => {
  if (!character.value?.race_name) return

  try {
    const source = character.value.race_source || 'PHB'
    const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
      'get_race_by_name',
      { name: character.value.race_name, source }
    )
    if (result.success && result.data) {
      raceData.value = result.data
    }
  } catch (e) {
    console.error('Failed to load race data:', e)
  }
}

const loadClassData = async () => {
  if (!character.value?.classes?.length) return

  try {
    const results = await Promise.all(
      character.value.classes.map(async (cls) => {
        const source = cls.class_source || 'PHB'
        const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
          'get_class_by_name',
          { name: cls.class_name, source }
        )
        return { className: cls.class_name, data: result.success ? result.data : null }
      })
    )

    const newClassData: Record<string, Record<string, unknown>> = {}
    for (const { className, data } of results) {
      if (data) {
        newClassData[className.toLowerCase()] = data
      }
    }
    classData.value = newClassData
  } catch (e) {
    console.error('Failed to load class data:', e)
  }
}

const loadClassFeatures = () => {
  if (!character.value?.classes?.length) return

  const allFeatures: ClassFeature[] = []

  for (const cls of character.value.classes) {
    const data = classData.value[cls.class_name.toLowerCase()]
    if (!data) continue

    // Parse classFeatures from the class data (format: "FeatureName|ClassName|ClassSource|Level" or object)
    const rawFeatures = data.classFeatures as Array<string | { classFeature: string }> | undefined
    if (rawFeatures) {
      for (const feature of rawFeatures) {
        let featureStr = ''
        if (typeof feature === 'string') {
          featureStr = feature
        } else if (typeof feature === 'object' && feature.classFeature) {
          featureStr = feature.classFeature
        }

        if (featureStr) {
          const parts = featureStr.split('|')
          if (parts.length >= 4) {
            const featureName = parts[0]
            const className = parts[1] || cls.class_name
            const classSource = parts[2] || cls.class_source || 'PHB'
            const level = parseInt(parts[3]) || 1

            // Only include features up to character's current level in this class
            if (level <= cls.level) {
              allFeatures.push({
                name: featureName,
                source: classSource,
                class_name: className,
                class_source: classSource,
                level: level,
                data: '',
              })
            }
          }
        }
      }
    }

    // Also load subclass features if character has a subclass
    if (cls.subclass_name) {
      const subclassKey = `${cls.class_name}|${cls.subclass_name}`
      const subclass = subclassDetails.value[subclassKey]
      if (subclass?.data) {
        // Subclass features format: "FeatureName|ClassName|ClassSource|SubclassShortName|SubclassSource|Level" (6 parts)
        const rawSubFeatures = (subclass.data as Record<string, unknown>).subclassFeatures as string[] | undefined
        if (rawSubFeatures) {
          for (const featureRef of rawSubFeatures) {
            if (typeof featureRef !== 'string') continue
            const parts = featureRef.split('|')
            if (parts.length >= 6) {
              const featureName = parts[0]
              const className = parts[1] || cls.class_name
              const classSource = parts[2] || cls.class_source || 'PHB'
              const subclassShortName = parts[3]
              const subclassSource = parts[4]
              const level = parseInt(parts[5]) || 1

              // Only include features up to character's current level
              if (level <= cls.level) {
                allFeatures.push({
                  name: featureName,
                  source: subclassSource || classSource,
                  class_name: className,
                  class_source: classSource,
                  level: level,
                  data: '',
                  subclass_name: cls.subclass_name,
                  subclass_short_name: subclassShortName,
                  subclass_source: subclassSource,
                })
              }
            }
          }
        }
      }
    }
  }

  // Sort by level, then by subclass (class features first), then by name
  allFeatures.sort((a, b) => {
    if (a.level !== b.level) return a.level - b.level
    // Class features before subclass features at same level
    if (a.subclass_name && !b.subclass_name) return 1
    if (!a.subclass_name && b.subclass_name) return -1
    return a.name.localeCompare(b.name)
  })

  classFeatures.value = allFeatures
}

const loadBackgroundDetails = async () => {
  if (!character.value?.background_name) return

  try {
    const source = character.value.background_source || 'PHB'
    const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
      'get_background_by_name',
      { name: character.value.background_name, source }
    )

    if (result.success && result.data) {
      const rawBg = result.data as unknown as {
        name: string
        source: string
        data: string | Record<string, unknown>
        fluff: string | null
      }
      backgroundDetails.value = {
        name: rawBg.name,
        source: rawBg.source,
        data: typeof rawBg.data === 'string' ? JSON.parse(rawBg.data) : rawBg.data,
        fluff: rawBg.fluff,
      }
    }
  } catch (e) {
    console.error('Failed to load background details:', e)
  }
}

const loadSubclassDetails = async () => {
  if (!character.value?.classes?.length) return

  try {
    for (const cls of character.value.classes) {
      if (!cls.subclass_name) continue

      const source = cls.subclass_source || 'PHB'
      const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
        'get_subclass_by_name',
        { name: cls.subclass_name, className: cls.class_name, source }
      )

      if (result.success && result.data) {
        // result.data IS the parsed 5etools data (entity_to_json merges it)
        const subclassData = result.data as Record<string, unknown>
        const key = `${cls.class_name}|${cls.subclass_name}`
        subclassDetails.value[key] = {
          name: (subclassData.name as string) || cls.subclass_name,
          source: (subclassData.source as string) || cls.subclass_source || 'PHB',
          class_name: (subclassData.className as string) || cls.class_name,
          class_source: (subclassData.classSource as string) || 'PHB',
          data: subclassData, // The full 5etools data is already here
        }
      }
    }
  } catch (e) {
    console.error('Failed to load subclass details:', e)
  }
}

const loadInventory = async () => {
  if (!characterId.value) return

  loadingInventory.value = true
  try {
    const result = await invoke<{ data: CharacterInventory[] }>('get_character_inventory', {
      characterId: characterId.value,
    })
    inventory.value = result.data || []
  } catch (e) {
    console.error('Failed to load inventory:', e)
    inventory.value = []
  } finally {
    loadingInventory.value = false
  }
}

const goBack = () => {
  router.back()
}

const openInventory = () => {
  showInventory.value = true
}

const printCharacter = () => {
  showPrintDialog.value = true
}

onMounted(() => {
  loadCharacter()

  // Set up cross-reference event listeners
  document.addEventListener('mouseover', handleCrossRefHover as any)
  document.addEventListener('mouseout', (e) => {
    const target = e.target as HTMLElement
    if (target.classList?.contains('cross-ref-link')) {
      hideTooltip()
    }
  })
  document.addEventListener('click', handleCrossRefClick as any)
})

onUnmounted(() => {
  // Clean up cross-reference event listeners
  document.removeEventListener('mouseover', handleCrossRefHover as any)
  document.removeEventListener('click', handleCrossRefClick as any)
})
</script>

<style scoped>
.character-sheet {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--spacing-lg);
}

.loading-state,
.error-state {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

/* Header */
.sheet-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-lg);
  border-bottom: 2px solid var(--color-border);
}

.header-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.btn-back {
  align-self: flex-start;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: transparent;
  border: 1px solid #ccc;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  margin-bottom: var(--spacing-sm);
}

.btn-back:hover {
  background: var(--color-surface-variant);
}

.character-name {
  font-size: 2rem;
  font-weight: bold;
  color: var(--color-text);
  margin: 0;
}

.character-subtitle {
  color: var(--color-text-secondary);
  font-size: 1.1rem;
}

.character-background {
  color: var(--color-text-secondary);
  font-size: 0.9rem;
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
  align-items: center;
}

.npc-badge {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-warning-bg, #fef3c7);
  color: var(--color-warning, #d97706);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.player-name {
  font-style: italic;
  color: var(--color-text-secondary);
}

/* Tab Navigation */
.tab-navigation {
  display: flex;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
}

.tab-button {
  padding: var(--spacing-sm) var(--spacing-lg);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 1rem;
  transition: all 0.2s ease;
}

.tab-button:hover {
  color: var(--color-text);
}

.tab-button.active {
  color: var(--color-primary-600);
  border-bottom-color: var(--color-primary-600);
}

/* Content Layouts */
.sheet-content.three-columns {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--spacing-lg);
}

.sheet-content.single-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  max-width: 700px;
}

.sheet-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

/* Sections */
.sheet-section {
  background: var(--color-surface);
  border: 1px solid #ccc;
  border-radius: var(--radius-lg);
  padding: var(--spacing-md);
}

.sheet-section h2 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--spacing-md) 0;
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.section-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.section-header-row h2 {
  margin: 0;
  padding: 0;
  border: none;
}

/* Ability Scores */
.ability-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-sm);
}

.ability-box {
  text-align: center;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.ability-name {
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.ability-value {
  font-size: 1.25rem;
  font-weight: bold;
  color: var(--color-text);
}

.ability-modifier {
  font-size: 0.9rem;
  color: var(--color-primary-600);
  font-weight: 500;
}

/* Combat Stats */
.combat-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-sm);
}

.combat-stat {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.stat-label {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.stat-value {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text);
}

.stat-note {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

/* Saves */
.saves-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.save-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs);
  border-radius: var(--radius-sm);
}

.save-item:hover {
  background: var(--color-surface-variant);
}

.save-proficient {
  width: 16px;
  color: var(--color-text-secondary);
  opacity: 0.3;
}

.save-proficient.active {
  color: var(--color-primary-600);
  opacity: 1;
}

.save-name {
  flex: 1;
  text-transform: capitalize;
  font-size: 0.9rem;
}

.save-bonus {
  font-weight: 600;
  color: var(--color-text);
}

/* Skills */
.skills-section {
  max-height: 500px;
  overflow-y: auto;
}

.skills-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.skill-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs);
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
}

.skill-item:hover {
  background: var(--color-surface-variant);
}

.skill-proficient {
  width: 20px;
  color: var(--color-text-secondary);
  opacity: 0.3;
}

.skill-proficient.active {
  color: var(--color-primary-600);
  opacity: 1;
}

.skill-proficient.expertise {
  color: var(--color-success, #059669);
}

.skill-name {
  flex: 1;
}

.skill-ability {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.skill-bonus {
  font-weight: 600;
  min-width: 30px;
  text-align: right;
}

/* Attacks */
.attacks-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.attack-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.attack-name {
  flex: 1;
  font-weight: 500;
}

.attack-bonus {
  font-weight: 600;
  color: var(--color-primary-600);
}

.attack-damage {
  font-size: 0.9rem;
  color: var(--color-text-secondary);
}

/* Proficiencies */
.proficiency-group {
  margin-bottom: var(--spacing-sm);
  font-size: 0.9rem;
}

.proficiency-group:last-child {
  margin-bottom: 0;
}

.empty-proficiencies {
  color: var(--color-text-secondary);
  font-style: italic;
  font-size: 0.9rem;
}

/* Class Features */
.features-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  max-height: 400px;
  overflow-y: auto;
}

.feature-item {
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
  overflow: hidden;
}

.feature-item.expanded {
  background: var(--color-surface);
  border: 1px solid #ccc;
}

.feature-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs) var(--spacing-sm);
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.feature-header:hover {
  background: var(--color-surface-hover);
}

.feature-name {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-weight: 500;
}

.feature-name .expand-icon {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  width: 12px;
}

.feature-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--color-text-secondary);
  font-size: 0.8rem;
}

.subclass-badge {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.7rem;
  font-weight: 600;
}

.theme-dark .subclass-badge {
  background: var(--color-primary-900);
  color: var(--color-primary-300);
}

.feature-details {
  padding: var(--spacing-sm) var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.feature-loading {
  color: var(--color-text-secondary);
  font-style: italic;
}

.feature-description {
  line-height: 1.5;
  color: var(--color-text);
  white-space: pre-wrap;
}

.feature-no-desc {
  color: var(--color-text-secondary);
  font-style: italic;
}

.feature-name {
  font-weight: 500;
}

.feature-meta {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

/* Spellcasting */
.spell-stats {
  display: flex;
  gap: var(--spacing-md);
}

.spell-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  flex: 1;
}

.spell-ability {
  font-size: 0.9rem;
}

.spell-note {
  margin-top: var(--spacing-sm);
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Personality */
.personality-item {
  margin-bottom: var(--spacing-sm);
  font-size: 0.9rem;
}

.personality-item:last-child {
  margin-bottom: 0;
}

/* Item Cards */
.item-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.item-card {
  background: var(--color-surface-variant);
  border: 1px solid #ccc;
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: all 0.2s ease;
}

.item-card.expanded {
  border-color: var(--color-primary-300);
}

.item-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background 0.15s ease;
}

.item-card-header:hover {
  background: var(--color-surface-hover);
}

.item-card-header .item-name {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.item-card-header .item-qty {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-weight: normal;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 0.85rem;
}

.item-source {
  color: var(--color-text-secondary);
}

.item-equipped-badge {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
}

.item-attuned {
  background: var(--color-warning-100);
  color: var(--color-warning-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
}

.expand-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  font-weight: bold;
  color: var(--color-text-secondary);
}

.item-card-details {
  padding: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
  font-size: 0.9rem;
}

.item-detail-row {
  display: flex;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xs);
}

.detail-label {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.detail-value.rarity {
  text-transform: capitalize;
}

.detail-value.rarity.common {
  color: var(--color-text-secondary);
}

.detail-value.rarity.uncommon {
  color: #16a34a;
}

.detail-value.rarity.rare {
  color: #2563eb;
}

.detail-value.rarity.very.rare {
  color: #7c3aed;
}

.detail-value.rarity.legendary {
  color: #ea580c;
}

.detail-value.rarity.artifact {
  color: #dc2626;
}

.item-description {
  margin-top: var(--spacing-sm);
  color: var(--color-text);
  line-height: 1.5;
}

.item-notes {
  margin-top: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px dashed var(--color-border);
  font-style: italic;
  color: var(--color-text-secondary);
}

.loading-details {
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Currency */
.currency-display {
  display: flex;
  gap: var(--spacing-lg);
  justify-content: center;
}

.currency-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

.currency-item.large .currency-value {
  font-size: 1.5rem;
}

.currency-icon {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  background: var(--color-surface-variant);
}

.currency-icon.pp {
  background: #e0e7ff;
  color: #4338ca;
}

.currency-icon.gp {
  background: #fef3c7;
  color: #d97706;
}

.currency-icon.ep,
.currency-icon.sp {
  background: #f3f4f6;
  color: #6b7280;
}

.currency-icon.cp {
  background: #fef2f2;
  color: #dc2626;
}

.currency-value {
  font-size: 1.1rem;
  font-weight: bold;
}

/* Details Tab */
.details-card {
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.details-card h3 {
  font-size: 0.95rem;
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
  color: var(--color-primary-600);
}

.details-card.feature-card {
  border-left: 3px solid var(--color-primary-500);
}

.proficiency-grid {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.prof-item {
  display: flex;
  gap: var(--spacing-sm);
}

.prof-label {
  font-weight: 500;
  color: var(--color-text-secondary);
  min-width: 80px;
}

.loading-text {
  color: var(--color-text-secondary);
  font-style: italic;
}

.npc-details-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: var(--spacing-md);
}

.npc-detail-item {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.npc-label {
  font-size: 0.8rem;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--color-text-secondary);
}

.npc-value {
  font-size: 1rem;
  font-weight: 500;
}

.class-details-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.class-detail-card {
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  padding: var(--spacing-lg);
  border: 1px solid #ccc;
}

.class-detail-card .class-header {
  margin-bottom: var(--spacing-md);
}

.class-detail-card .class-header h3 {
  font-size: 1.1rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.class-level-badge {
  font-size: 0.8rem;
  font-weight: 500;
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}

.class-stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-md);
}

.class-stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--spacing-sm);
  background: var(--color-surface);
  border-radius: var(--radius-sm);
}

.class-stat-label {
  font-size: 0.7rem;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--color-text-secondary);
  letter-spacing: 0.5px;
}

.class-stat-value {
  font-size: 0.9rem;
  font-weight: 600;
}

.class-stat-value.spellcaster {
  color: var(--color-primary-500);
}

.class-proficiencies {
  background: var(--color-surface);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.class-proficiencies h4 {
  font-size: 0.85rem;
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
  color: var(--color-text-secondary);
}

/* Links in proficiencies */
.class-proficiencies :deep(a) {
  color: var(--color-primary-500);
  text-decoration: none;
}

.class-proficiencies :deep(a:hover) {
  text-decoration: underline;
}

.subclass-section {
  background: var(--color-surface);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin-bottom: var(--spacing-md);
  border-left: 3px solid var(--color-secondary-500);
}

.subclass-header {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--color-secondary-600);
  margin-bottom: var(--spacing-sm);
}

.subclass-description {
  color: var(--color-text-secondary);
  font-size: 0.85rem;
  line-height: 1.5;
}

.class-features-section {
  margin-top: var(--spacing-md);
}

.class-features-section h4 {
  font-size: 0.9rem;
  font-weight: 600;
  margin-bottom: var(--spacing-sm);
  color: var(--color-text-secondary);
}

.features-by-level {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.feature-level-group {
  display: flex;
  gap: var(--spacing-sm);
  font-size: 0.85rem;
  line-height: 1.4;
}

.level-label {
  font-weight: 600;
  min-width: 85px;
  color: var(--color-text-secondary);
}

.feature-links {
  flex: 1;
}

.feature-links a {
  color: var(--color-primary-500);
  text-decoration: none;
}

.feature-links a:hover {
  text-decoration: underline;
}

.feature-links a.subclass-feature {
  color: var(--color-secondary-500, #9c27b0);
  font-style: italic;
}

.feature-links a.subclass-feature:hover {
  color: var(--color-secondary-600, #7b1fa2);
}

/* Classes */
.class-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.class-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.class-name {
  font-weight: 600;
}

.class-subclass {
  color: var(--color-text-secondary);
}

.class-level {
  margin-left: auto;
  font-size: 0.9rem;
  color: var(--color-primary-600);
}

/* Details */
.detail-item {
  margin-bottom: var(--spacing-sm);
}

.detail-item:last-child {
  margin-bottom: 0;
}

/* Empty states */
.empty-state {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-style: italic;
}

.loading-inventory {
  text-align: center;
  padding: var(--spacing-md);
  color: var(--color-text-secondary);
}

/* Responsive */
@media (max-width: 900px) {
  .sheet-content.three-columns {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 600px) {
  .sheet-content.three-columns {
    grid-template-columns: 1fr;
  }

  .ability-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .combat-grid {
    grid-template-columns: 1fr;
  }

  .currency-display {
    flex-wrap: wrap;
    gap: var(--spacing-md);
  }

  .sheet-header {
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .header-actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
