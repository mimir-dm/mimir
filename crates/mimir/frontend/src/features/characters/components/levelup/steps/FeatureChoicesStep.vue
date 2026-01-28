<template>
  <div class="features-step">
    <h3 class="step-heading">Class Feature Choices</h3>
    <p class="step-description">
      At level {{ levelUp.newClassLevel.value }}, you gain new class features that require choices.
    </p>

    <!-- Loading State -->
    <div v-if="isLoading" class="loading-indicator">Loading feature options...</div>

    <!-- No choices needed message -->
    <div v-else-if="!hasAnySection" class="no-choices">
      <p>No feature choices required at this level.</p>
      <p class="note">You may proceed to the next step.</p>
    </div>

    <!-- Fighting Style (Fighter 1, Paladin 2, Ranger 2) -->
    <FeatureGridSection
      v-if="showFightingStyle"
      title="Fighting Style"
      description="Choose a fighting style specialty. You can't take the same Fighting Style option more than once."
      :items="fightingStyleItems"
      :selected-items="selectedFightingStyle ? [selectedFightingStyle] : []"
      :max-slots="1"
      :show-slot-count="false"
      :is-item-selected="(item) => selectedFightingStyle?.name === item.name"
      :is-item-disabled="() => false"
      @select="selectFightingStyle"
    >
      <template #empty>
        <div class="manual-fallback">
          <div class="form-group">
            <label class="form-label" for="fighting-style">Fighting Style</label>
            <select id="fighting-style" v-model="manualFightingStyle" class="form-input">
              <option value="">Select a Fighting Style</option>
              <option value="Archery">Archery</option>
              <option value="Defense">Defense</option>
              <option value="Dueling">Dueling</option>
              <option value="Great Weapon Fighting">Great Weapon Fighting</option>
              <option value="Protection">Protection</option>
              <option value="Two-Weapon Fighting">Two-Weapon Fighting</option>
            </select>
          </div>
        </div>
      </template>
    </FeatureGridSection>

    <!-- Metamagic (Sorcerer 3, 10, 17) -->
    <FeatureGridSection
      v-if="showMetamagic"
      title="Metamagic"
      description="Choose Metamagic options that allow you to twist your spells to suit your needs."
      :items="metamagicItems"
      :selected-items="metamagic.selected.value"
      :max-slots="metamagicSlots"
      :compact="true"
      :is-item-selected="metamagic.isSelected"
      :is-item-disabled="(item) => !metamagic.isSelected(item) && metamagic.isAtLimit.value"
      @select="(item) => metamagic.toggle(item)"
    />

    <!-- Maneuvers (Battle Master Fighter) -->
    <FeatureGridSection
      v-if="showManeuvers"
      title="Maneuvers"
      description="Choose maneuvers that fuel your martial techniques with superiority dice."
      :items="filteredManeuverItems"
      :selected-items="maneuvers.selected.value"
      :max-slots="maneuverSlots"
      :compact="true"
      :searchable="true"
      :search-query="maneuverSearch"
      search-placeholder="Search maneuvers..."
      :is-item-selected="maneuvers.isSelected"
      :is-item-disabled="(item) => !maneuvers.isSelected(item) && maneuvers.isAtLimit.value"
      @select="(item) => maneuvers.toggle(item)"
      @update:search-query="maneuverSearch = $event"
    />

    <!-- Eldritch Invocations (Warlock 2+) -->
    <FeatureGridSection
      v-if="showInvocations"
      title="Eldritch Invocations"
      description="Choose invocations that grant you supernatural abilities."
      :items="filteredInvocationItems"
      :selected-items="invocations.selected.value"
      :max-slots="invocationSlots"
      :compact="true"
      :searchable="true"
      :search-query="invocationSearch"
      search-placeholder="Search invocations..."
      :is-item-selected="invocations.isSelected"
      :is-item-disabled="(item) => !invocations.isSelected(item) && invocations.isAtLimit.value"
      @select="handleInvocationSelect"
      @update:search-query="invocationSearch = $event"
    />

    <!-- Pact Boon (Warlock 3) -->
    <FeatureGridSection
      v-if="showPactBoon"
      title="Pact Boon"
      description="Your otherworldly patron bestows a gift upon you for your loyal service."
      :items="pactBoonItems"
      :selected-items="selectedPactBoon ? [selectedPactBoon] : []"
      :max-slots="1"
      :show-slot-count="false"
      :is-item-selected="(item) => selectedPactBoon?.name === item.name"
      :is-item-disabled="() => false"
      @select="selectPactBoon"
    />

    <!-- Expertise (Rogue 1, 6; Bard 3, 10) -->
    <FeatureGridSection
      v-if="showExpertise"
      title="Expertise"
      description="Choose skills in which you have proficiency to gain expertise (double proficiency bonus)."
      :items="expertiseItems"
      :selected-items="expertise.selected.value"
      :max-slots="expertiseSlots"
      :compact="true"
      :allow-remove="true"
      :is-item-selected="(item) => expertise.isSelected(item.name)"
      :is-item-disabled="(item) => !expertise.isSelected(item.name) && expertise.isAtLimit.value"
      @select="(item) => expertise.toggle(item.name)"
      @remove="(item) => expertise.deselect(typeof item === 'string' ? item : item.name)"
    >
      <template #empty>
        <div class="no-proficiencies">
          <p>No skill proficiencies found. Add skills manually:</p>
          <div class="manual-entry">
            <input
              v-model="manualExpertise"
              type="text"
              class="form-input"
              placeholder="Skill name"
            />
            <button
              type="button"
              class="btn btn-secondary"
              :disabled="!manualExpertise.trim() || expertise.isAtLimit.value"
              @click="addManualExpertise"
            >
              Add
            </button>
          </div>
        </div>
      </template>
    </FeatureGridSection>

    <!-- Current Selection Summary -->
    <div v-if="hasAnyChoices" class="current-selection">
      <span class="selection-label">Current Choices:</span>
      <span v-if="levelUp.featureChoices.value?.fighting_style" class="selection-value">
        {{ levelUp.featureChoices.value.fighting_style.name }}
      </span>
      <span v-if="levelUp.featureChoices.value?.metamagic?.length" class="selection-value">
        {{ levelUp.featureChoices.value.metamagic.length }} Metamagic
      </span>
      <span
        v-if="levelUp.featureChoices.value?.maneuvers?.new_maneuvers?.length"
        class="selection-value"
      >
        {{ levelUp.featureChoices.value.maneuvers.new_maneuvers.length }} Maneuvers
      </span>
      <span
        v-if="levelUp.featureChoices.value?.invocations?.new_invocations?.length"
        class="selection-value"
      >
        {{ levelUp.featureChoices.value.invocations.new_invocations.length }} Invocations
      </span>
      <span v-if="levelUp.featureChoices.value?.pact_boon" class="selection-value">
        {{ levelUp.featureChoices.value.pact_boon.name }}
      </span>
      <span v-if="levelUp.featureChoices.value?.expertise_skills?.length" class="selection-value">
        {{ levelUp.featureChoices.value.expertise_skills.length }} Expertise
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import type { Character } from '@/types/character'
import type { LevelUpComposable } from '@/features/characters/composables/useLevelUp'
import {
  useFeatureSelection,
  useStringFeatureSelection,
  type FeatureRef,
  type FeatureItem
} from '@/features/characters/composables/useFeatureSelection'
import FeatureGridSection from '../FeatureGridSection.vue'
import { invoke } from '@tauri-apps/api/core'

interface FightingStyleOption extends FeatureRef {
  description?: string
  availableToClasses?: string[]
}

interface MetamagicOption extends FeatureRef {
  cost?: number
}

interface ManeuverOption extends FeatureRef {
  description?: string
}

interface InvocationOption extends FeatureRef {
  levelPrereq?: number
  pactPrereq?: string
  spellPrereq?: string
}

interface PactBoon extends FeatureRef {
  description: string
}

const props = defineProps<{
  levelUp: LevelUpComposable
  character: Character
}>()

// Loading state
const isLoading = ref(false)

// Catalog data
const fightingStyles = ref<FightingStyleOption[]>([])
const metamagicOptions = ref<MetamagicOption[]>([])
const maneuverOptions = ref<ManeuverOption[]>([])
const invocationOptions = ref<InvocationOption[]>([])

// Search filters
const maneuverSearch = ref('')
const invocationSearch = ref('')

// Single-select state (fighting style, pact boon)
const selectedFightingStyle = ref<FeatureRef | null>(null)
const selectedPactBoon = ref<PactBoon | null>(null)

// Manual fallbacks
const manualFightingStyle = ref('')
const manualExpertise = ref('')

// Pact boons (static)
const pactBoons: PactBoon[] = [
  {
    name: 'Pact of the Chain',
    source: 'PHB',
    description: 'Learn find familiar spell, and your familiar can take special forms like imp or pseudodragon.'
  },
  {
    name: 'Pact of the Blade',
    source: 'PHB',
    description: 'Create a pact weapon that you are proficient with and counts as magical for resistances.'
  },
  {
    name: 'Pact of the Tome',
    source: 'PHB',
    description: 'Receive a Book of Shadows with three cantrips from any class spell list.'
  },
  {
    name: 'Pact of the Talisman',
    source: 'TCE',
    description: "Receive an amulet that adds d4 to failed ability checks when the wearer lacks proficiency."
  }
]

// Feature visibility based on class and level
const className = computed(() => props.levelUp.selectedClass.value?.class_name?.toLowerCase() ?? '')
const classLevel = computed(() => props.levelUp.newClassLevel.value)
const subclassName = computed(
  () => props.levelUp.subclass.value?.name?.toLowerCase() ?? props.character.classes.find(
    (c) => c.class_name.toLowerCase() === className.value
  )?.subclass_name?.toLowerCase() ?? ''
)

const showFightingStyle = computed(() => {
  if (className.value === 'fighter' && classLevel.value === 1) return true
  if (className.value === 'paladin' && classLevel.value === 2) return true
  if (className.value === 'ranger' && classLevel.value === 2) return true
  return false
})

const showMetamagic = computed(() => className.value === 'sorcerer' && [3, 10, 17].includes(classLevel.value))
const metamagicSlots = computed(() => classLevel.value === 3 ? 2 : 1)

const showManeuvers = computed(() => {
  if (className.value !== 'fighter') return false
  if (classLevel.value < 3) return false
  return subclassName.value.includes('battle master')
})
const maneuverSlots = computed(() => {
  if (classLevel.value === 3) return 3
  if ([7, 10, 15].includes(classLevel.value)) return 2
  return 0
})

const showInvocations = computed(() => className.value === 'warlock' && classLevel.value >= 2)
const invocationSlots = computed(() => {
  if (classLevel.value === 2) return 2
  if ([5, 7, 9, 12, 15, 18].includes(classLevel.value)) return 1
  return 0
})

const showPactBoon = computed(() => className.value === 'warlock' && classLevel.value === 3)

const showExpertise = computed(() => {
  if (className.value === 'rogue' && [1, 6].includes(classLevel.value)) return true
  if (className.value === 'bard' && [3, 10].includes(classLevel.value)) return true
  return false
})
const expertiseSlots = computed(() => 2)

const hasAnySection = computed(() => {
  return showFightingStyle.value || showMetamagic.value || showManeuvers.value ||
         showInvocations.value || showPactBoon.value || showExpertise.value
})

const hasAnyChoices = computed(() => {
  const fc = props.levelUp.featureChoices.value
  if (!fc) return false
  return fc.fighting_style || (fc.metamagic && fc.metamagic.length > 0) ||
         (fc.maneuvers?.new_maneuvers && fc.maneuvers.new_maneuvers.length > 0) ||
         (fc.invocations?.new_invocations && fc.invocations.new_invocations.length > 0) ||
         fc.pact_boon || (fc.expertise_skills && fc.expertise_skills.length > 0)
})

// Multi-select composables
const metamagic = useFeatureSelection<FeatureRef>({
  maxSlots: metamagicSlots,
  onSelectionChange: updateFeatureChoices
})

const maneuvers = useFeatureSelection<FeatureRef>({
  maxSlots: maneuverSlots,
  onSelectionChange: updateFeatureChoices
})

const invocations = useFeatureSelection<FeatureRef>({
  maxSlots: invocationSlots,
  onSelectionChange: updateFeatureChoices
})

const expertise = useStringFeatureSelection({
  maxSlots: expertiseSlots,
  onSelectionChange: updateFeatureChoices
})

// Convert catalog data to FeatureItem format
const fightingStyleItems = computed<FeatureItem[]>(() => {
  const currentClass = className.value.charAt(0).toUpperCase() + className.value.slice(1)
  return fightingStyles.value
    .filter((style) => !style.availableToClasses?.length || style.availableToClasses.includes(currentClass))
    .map((style) => ({
      name: style.name,
      source: style.source,
      description: style.description
    }))
})

const metamagicItems = computed<FeatureItem[]>(() =>
  metamagicOptions.value.map((m) => ({
    name: m.name,
    source: m.source,
    cost: m.cost ? `${m.cost} SP` : undefined
  }))
)

const filteredManeuverItems = computed<FeatureItem[]>(() => {
  let list = maneuverOptions.value
  if (maneuverSearch.value.trim()) {
    const search = maneuverSearch.value.toLowerCase()
    list = list.filter((m) => m.name.toLowerCase().includes(search))
  }
  return list.map((m) => ({ name: m.name, source: m.source, description: m.description }))
})

const filteredInvocationItems = computed<FeatureItem[]>(() => {
  let list = invocationOptions.value.filter((inv) => meetsInvocationPrereqs(inv))
  if (invocationSearch.value.trim()) {
    const search = invocationSearch.value.toLowerCase()
    list = list.filter((i) => i.name.toLowerCase().includes(search))
  }
  return list.map((inv) => ({
    name: inv.name,
    source: inv.source,
    prereqs: formatInvocationPrereqs(inv)
  }))
})

const pactBoonItems = computed<FeatureItem[]>(() =>
  pactBoons.map((p) => ({ name: p.name, source: p.source, description: p.description }))
)

const expertiseItems = computed<FeatureItem[]>(() =>
  props.character.proficiencies
    .filter((p) => p.proficiency_type === 'skill')
    .map((p) => ({ name: p.name, source: 'Character' }))
)

// Helper functions
function meetsInvocationPrereqs(inv: InvocationOption): boolean {
  if (inv.levelPrereq && classLevel.value < inv.levelPrereq) return false
  if (inv.pactPrereq) {
    const hasPact = selectedPactBoon.value?.name.toLowerCase().includes(inv.pactPrereq.toLowerCase()) ||
                    props.levelUp.featureChoices.value?.pact_boon?.name.toLowerCase().includes(inv.pactPrereq.toLowerCase())
    if (!hasPact) return false
  }
  return true
}

function formatInvocationPrereqs(inv: InvocationOption): string | undefined {
  const parts: string[] = []
  if (inv.levelPrereq) parts.push(`Level ${inv.levelPrereq}`)
  if (inv.pactPrereq) parts.push(inv.pactPrereq)
  return parts.length > 0 ? parts.join(', ') : undefined
}

function extractSorceryPointCost(name: string): number | undefined {
  const costs: Record<string, number> = {
    'Careful Spell': 1, 'Distant Spell': 1, 'Empowered Spell': 1, 'Extended Spell': 1,
    'Heightened Spell': 3, 'Quickened Spell': 2, 'Seeking Spell': 2, 'Subtle Spell': 1,
    'Transmuted Spell': 1, 'Twinned Spell': 1
  }
  return costs[name]
}

function extractDescription(entries: unknown[] | undefined): string | undefined {
  if (!entries || !Array.isArray(entries)) return undefined
  return entries.find((e) => typeof e === 'string') as string | undefined
}

// Selection handlers
function selectFightingStyle(item: FeatureItem) {
  selectedFightingStyle.value = { name: item.name, source: item.source }
  updateFeatureChoices()
}

function selectPactBoon(item: FeatureItem) {
  const pact = pactBoons.find((p) => p.name === item.name)
  if (pact) {
    selectedPactBoon.value = pact
    updateFeatureChoices()
  }
}

function handleInvocationSelect(item: FeatureItem) {
  const inv = invocationOptions.value.find((i) => i.name === item.name)
  if (inv && meetsInvocationPrereqs(inv)) {
    invocations.toggle({ name: inv.name, source: inv.source })
  }
}

function addManualExpertise() {
  if (!manualExpertise.value.trim() || expertise.isAtLimit.value) return
  expertise.toggle(manualExpertise.value.trim())
  manualExpertise.value = ''
}

function updateFeatureChoices() {
  const choices: NonNullable<typeof props.levelUp.featureChoices.value> = {}

  if (selectedFightingStyle.value) {
    choices.fighting_style = selectedFightingStyle.value
  } else if (manualFightingStyle.value) {
    choices.fighting_style = { name: manualFightingStyle.value, source: 'PHB' }
  }

  if (metamagic.selected.value.length > 0) {
    choices.metamagic = metamagic.selected.value
  }

  if (maneuvers.selected.value.length > 0) {
    choices.maneuvers = { new_maneuvers: maneuvers.selected.value }
  }

  if (invocations.selected.value.length > 0) {
    choices.invocations = { new_invocations: invocations.selected.value }
  }

  if (selectedPactBoon.value) {
    choices.pact_boon = { name: selectedPactBoon.value.name, source: selectedPactBoon.value.source }
  }

  if (expertise.selected.value.length > 0) {
    choices.expertise_skills = expertise.selected.value
  }

  props.levelUp.featureChoices.value = Object.keys(choices).length > 0 ? choices : null
}

// Data loading
async function loadFightingStyles() {
  try {
    const result = await invoke<{ success: boolean; data: Array<{ name: string; source: string; entries?: unknown[]; available_to_classes?: string[] }> }>('list_fighting_styles')
    if (result.success && result.data) {
      fightingStyles.value = result.data.map((style) => ({
        name: style.name,
        source: style.source,
        description: extractDescription(style.entries),
        availableToClasses: style.available_to_classes || []
      }))
    }
  } catch (e) {
    console.error('Error loading fighting styles:', e)
  }
}

async function loadMetamagic() {
  try {
    const result = await invoke<{ success: boolean; data: Array<{ name: string; source: string }> }>('list_metamagic')
    if (result.success && result.data) {
      metamagicOptions.value = result.data.map((meta) => ({
        name: meta.name,
        source: meta.source,
        cost: extractSorceryPointCost(meta.name)
      }))
    }
  } catch (e) {
    console.error('Error loading metamagic:', e)
  }
}

async function loadManeuvers() {
  try {
    const result = await invoke<{ success: boolean; data: Array<{ name: string; source: string; entries?: unknown[] }> }>('list_maneuvers')
    if (result.success && result.data) {
      maneuverOptions.value = result.data.map((m) => ({
        name: m.name,
        source: m.source,
        description: extractDescription(m.entries)
      }))
    }
  } catch (e) {
    console.error('Error loading maneuvers:', e)
  }
}

async function loadInvocations() {
  try {
    const result = await invoke<{ success: boolean; data: Array<{ name: string; source: string; level_prereq?: number; pact_prereq?: string; spell_prereq?: string }> }>('list_invocations')
    if (result.success && result.data) {
      invocationOptions.value = result.data.map((inv) => ({
        name: inv.name,
        source: inv.source,
        levelPrereq: inv.level_prereq,
        pactPrereq: inv.pact_prereq,
        spellPrereq: inv.spell_prereq
      }))
    }
  } catch (e) {
    console.error('Error loading invocations:', e)
  }
}

// Initialize from existing state
watch(
  () => props.levelUp.featureChoices.value,
  (features) => {
    if (features) {
      if (features.fighting_style) selectedFightingStyle.value = features.fighting_style
      if (features.pact_boon) {
        selectedPactBoon.value = {
          ...features.pact_boon,
          description: pactBoons.find((p) => p.name === features.pact_boon?.name)?.description || ''
        }
      }
      if (features.metamagic) metamagic.selected.value = [...features.metamagic]
      if (features.maneuvers?.new_maneuvers) maneuvers.selected.value = [...features.maneuvers.new_maneuvers]
      if (features.invocations?.new_invocations) invocations.selected.value = [...features.invocations.new_invocations]
      if (features.expertise_skills) expertise.selected.value = [...features.expertise_skills]
    }
  },
  { immediate: true }
)

watch(manualFightingStyle, () => {
  if (manualFightingStyle.value) {
    selectedFightingStyle.value = null
    updateFeatureChoices()
  }
})

onMounted(async () => {
  isLoading.value = true
  try {
    await Promise.all([loadFightingStyles(), loadMetamagic(), loadManeuvers(), loadInvocations()])
  } finally {
    isLoading.value = false
  }
})
</script>

<style scoped>
.features-step {
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

.no-choices {
  padding: var(--spacing-lg);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  text-align: center;
}

.no-choices p {
  margin: 0;
  color: var(--color-text);
}

.no-choices .note {
  margin-top: var(--spacing-sm);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.manual-fallback,
.manual-entry {
  display: flex;
  gap: var(--spacing-md);
  align-items: flex-end;
}

.no-proficiencies {
  text-align: center;
  padding: var(--spacing-md);
}

.no-proficiencies p {
  margin: 0 0 var(--spacing-md);
  color: var(--color-text-secondary);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  flex: 1;
  min-width: 200px;
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

.current-selection {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
  padding: var(--spacing-md);
  background: var(--color-success-bg, #f0fdf4);
  border: 1px solid var(--color-success, #22c55e);
  border-radius: var(--radius-md);
}

.selection-label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.selection-value {
  font-weight: 600;
  color: var(--color-success, #22c55e);
}

.btn {
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all var(--transition-base);
}

.btn-secondary {
  background: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--color-surface-hover);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
