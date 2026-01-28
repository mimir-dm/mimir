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
    <div v-if="showFightingStyle" class="feature-section">
      <h4 class="section-title">Fighting Style</h4>
      <p class="section-note">
        Choose a fighting style specialty. You can't take the same Fighting Style option more than
        once.
      </p>
      <div v-if="fightingStyles.length > 0" class="feature-grid">
        <button
          v-for="style in availableFightingStyles"
          :key="`${style.name}-${style.source}`"
          type="button"
          class="feature-card"
          :class="{ selected: isStyleSelected(style) }"
          @click="selectFightingStyle(style)"
        >
          <div class="feature-name">{{ style.name }}</div>
          <div class="feature-source">{{ style.source }}</div>
          <div v-if="style.description" class="feature-description">
            {{ truncateDescription(style.description) }}
          </div>
        </button>
      </div>
      <div v-else class="manual-fallback">
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
    </div>

    <!-- Metamagic (Sorcerer 3, 10, 17) -->
    <div v-if="showMetamagic" class="feature-section">
      <h4 class="section-title">Metamagic ({{ selectedMetamagic.length }}/{{ metamagicSlots }})</h4>
      <p class="section-note">
        Choose Metamagic options that allow you to twist your spells to suit your needs.
      </p>
      <div v-if="metamagicOptions.length > 0" class="feature-grid compact">
        <button
          v-for="meta in metamagicOptions"
          :key="`${meta.name}-${meta.source}`"
          type="button"
          class="feature-card compact"
          :class="{
            selected: isMetamagicSelected(meta),
            disabled: !isMetamagicSelected(meta) && selectedMetamagic.length >= metamagicSlots
          }"
          :disabled="!isMetamagicSelected(meta) && selectedMetamagic.length >= metamagicSlots"
          @click="toggleMetamagic(meta)"
        >
          <div class="feature-name">{{ meta.name }}</div>
          <div class="feature-source">{{ meta.source }}</div>
          <div v-if="meta.cost" class="feature-cost">{{ meta.cost }} SP</div>
        </button>
      </div>
      <div v-if="selectedMetamagic.length > 0" class="selected-list">
        <span class="selected-label">Selected:</span>
        <span v-for="meta in selectedMetamagic" :key="meta.name" class="selected-tag">
          {{ meta.name }}
        </span>
      </div>
    </div>

    <!-- Maneuvers (Battle Master Fighter) -->
    <div v-if="showManeuvers" class="feature-section">
      <h4 class="section-title">
        Maneuvers ({{ selectedManeuvers.length }}/{{ maneuverSlots }})
      </h4>
      <p class="section-note">
        Choose maneuvers that fuel your martial techniques with superiority dice.
      </p>
      <div class="search-box">
        <input
          v-model="maneuverSearch"
          type="text"
          class="search-input"
          placeholder="Search maneuvers..."
        />
      </div>
      <div v-if="filteredManeuvers.length > 0" class="feature-grid compact">
        <button
          v-for="maneuver in filteredManeuvers"
          :key="`${maneuver.name}-${maneuver.source}`"
          type="button"
          class="feature-card compact"
          :class="{
            selected: isManeuverSelected(maneuver),
            disabled: !isManeuverSelected(maneuver) && selectedManeuvers.length >= maneuverSlots
          }"
          :disabled="!isManeuverSelected(maneuver) && selectedManeuvers.length >= maneuverSlots"
          @click="toggleManeuver(maneuver)"
        >
          <div class="feature-name">{{ maneuver.name }}</div>
          <div class="feature-source">{{ maneuver.source }}</div>
        </button>
      </div>
      <div v-if="selectedManeuvers.length > 0" class="selected-list">
        <span class="selected-label">Selected:</span>
        <span v-for="m in selectedManeuvers" :key="m.name" class="selected-tag">
          {{ m.name }}
        </span>
      </div>
    </div>

    <!-- Eldritch Invocations (Warlock 2+) -->
    <div v-if="showInvocations" class="feature-section">
      <h4 class="section-title">
        Eldritch Invocations ({{ selectedInvocations.length }}/{{ invocationSlots }})
      </h4>
      <p class="section-note">
        Choose invocations that grant you supernatural abilities.
      </p>
      <div class="search-box">
        <input
          v-model="invocationSearch"
          type="text"
          class="search-input"
          placeholder="Search invocations..."
        />
      </div>
      <div v-if="availableInvocations.length > 0" class="feature-grid compact">
        <button
          v-for="inv in filteredInvocations"
          :key="`${inv.name}-${inv.source}`"
          type="button"
          class="feature-card compact"
          :class="{
            selected: isInvocationSelected(inv),
            disabled: !isInvocationSelected(inv) && selectedInvocations.length >= invocationSlots,
            'has-prereq': inv.levelPrereq || inv.pactPrereq
          }"
          :disabled="
            (!isInvocationSelected(inv) && selectedInvocations.length >= invocationSlots) ||
            !meetsInvocationPrereqs(inv)
          "
          @click="toggleInvocation(inv)"
        >
          <div class="feature-name">{{ inv.name }}</div>
          <div class="feature-source">{{ inv.source }}</div>
          <div v-if="inv.levelPrereq || inv.pactPrereq" class="feature-prereq">
            <span v-if="inv.levelPrereq" class="prereq-item">Level {{ inv.levelPrereq }}</span>
            <span v-if="inv.pactPrereq" class="prereq-item">{{ inv.pactPrereq }}</span>
          </div>
        </button>
      </div>
      <div v-if="selectedInvocations.length > 0" class="selected-list">
        <span class="selected-label">Selected:</span>
        <span v-for="inv in selectedInvocations" :key="inv.name" class="selected-tag">
          {{ inv.name }}
        </span>
      </div>
    </div>

    <!-- Pact Boon (Warlock 3) -->
    <div v-if="showPactBoon" class="feature-section">
      <h4 class="section-title">Pact Boon</h4>
      <p class="section-note">
        Your otherworldly patron bestows a gift upon you for your loyal service.
      </p>
      <div class="feature-grid">
        <button
          v-for="pact in pactBoons"
          :key="pact.name"
          type="button"
          class="feature-card"
          :class="{ selected: selectedPactBoon?.name === pact.name }"
          @click="selectPactBoon(pact)"
        >
          <div class="feature-name">{{ pact.name }}</div>
          <div class="feature-description">{{ pact.description }}</div>
        </button>
      </div>
    </div>

    <!-- Expertise (Rogue 1, 6; Bard 3, 10) -->
    <div v-if="showExpertise" class="feature-section">
      <h4 class="section-title">
        Expertise ({{ selectedExpertise.length }}/{{ expertiseSlots }})
      </h4>
      <p class="section-note">
        Choose skills in which you have proficiency to gain expertise (double proficiency bonus).
      </p>
      <div v-if="proficientSkills.length > 0" class="feature-grid compact">
        <button
          v-for="skill in proficientSkills"
          :key="skill"
          type="button"
          class="feature-card compact"
          :class="{
            selected: selectedExpertise.includes(skill),
            disabled: !selectedExpertise.includes(skill) && selectedExpertise.length >= expertiseSlots
          }"
          :disabled="!selectedExpertise.includes(skill) && selectedExpertise.length >= expertiseSlots"
          @click="toggleExpertise(skill)"
        >
          <div class="feature-name">{{ skill }}</div>
        </button>
      </div>
      <div v-else class="no-proficiencies">
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
            :disabled="!manualExpertise.trim() || selectedExpertise.length >= expertiseSlots"
            @click="addManualExpertise"
          >
            Add
          </button>
        </div>
      </div>
      <div v-if="selectedExpertise.length > 0" class="selected-list">
        <span class="selected-label">Selected:</span>
        <span v-for="skill in selectedExpertise" :key="skill" class="selected-tag">
          {{ skill }}
          <button type="button" class="remove-tag" @click="removeExpertise(skill)">&times;</button>
        </span>
      </div>
    </div>

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
import type { LevelUpComposable } from '../../../composables/useLevelUp'
import { invoke } from '@tauri-apps/api/core'

interface FeatureRef {
  name: string
  source: string
}

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

interface PactBoon {
  name: string
  source: string
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

// Selections
const selectedFightingStyle = ref<FeatureRef | null>(null)
const selectedMetamagic = ref<FeatureRef[]>([])
const selectedManeuvers = ref<FeatureRef[]>([])
const selectedInvocations = ref<FeatureRef[]>([])
const selectedPactBoon = ref<PactBoon | null>(null)
const selectedExpertise = ref<string[]>([])

// Manual fallbacks
const manualFightingStyle = ref('')
const manualExpertise = ref('')

// Pact boons (static since not in catalog)
const pactBoons: PactBoon[] = [
  {
    name: 'Pact of the Chain',
    source: 'PHB',
    description:
      'Learn find familiar spell, and your familiar can take special forms like imp or pseudodragon.'
  },
  {
    name: 'Pact of the Blade',
    source: 'PHB',
    description:
      'Create a pact weapon that you are proficient with and counts as magical for resistances.'
  },
  {
    name: 'Pact of the Tome',
    source: 'PHB',
    description: 'Receive a Book of Shadows with three cantrips from any class spell list.'
  },
  {
    name: 'Pact of the Talisman',
    source: 'TCE',
    description:
      'Receive an amulet that adds d4 to failed ability checks when the wearer lacks proficiency.'
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

const showMetamagic = computed(() => {
  return className.value === 'sorcerer' && [3, 10, 17].includes(classLevel.value)
})

const metamagicSlots = computed(() => {
  if (classLevel.value === 3) return 2
  return 1
})

const showManeuvers = computed(() => {
  // Battle Master Fighter or any Fighter at level 3+ with Battle Master subclass
  if (className.value !== 'fighter') return false
  if (classLevel.value < 3) return false
  // Check if Battle Master
  return subclassName.value.includes('battle master')
})

const maneuverSlots = computed(() => {
  if (classLevel.value === 3) return 3
  if ([7, 10, 15].includes(classLevel.value)) return 2
  return 0
})

const showInvocations = computed(() => {
  return className.value === 'warlock' && classLevel.value >= 2
})

const invocationSlots = computed(() => {
  if (classLevel.value === 2) return 2
  if ([5, 7, 9, 12, 15, 18].includes(classLevel.value)) return 1
  return 0
})

const showPactBoon = computed(() => {
  return className.value === 'warlock' && classLevel.value === 3
})

const showExpertise = computed(() => {
  if (className.value === 'rogue' && [1, 6].includes(classLevel.value)) return true
  if (className.value === 'bard' && [3, 10].includes(classLevel.value)) return true
  return false
})

const expertiseSlots = computed(() => 2)

const hasAnySection = computed(() => {
  return (
    showFightingStyle.value ||
    showMetamagic.value ||
    showManeuvers.value ||
    showInvocations.value ||
    showPactBoon.value ||
    showExpertise.value
  )
})

const hasAnyChoices = computed(() => {
  const fc = props.levelUp.featureChoices.value
  if (!fc) return false
  return (
    fc.fighting_style ||
    (fc.metamagic && fc.metamagic.length > 0) ||
    (fc.maneuvers?.new_maneuvers && fc.maneuvers.new_maneuvers.length > 0) ||
    (fc.invocations?.new_invocations && fc.invocations.new_invocations.length > 0) ||
    fc.pact_boon ||
    (fc.expertise_skills && fc.expertise_skills.length > 0)
  )
})

// Fighting styles available to current class
const availableFightingStyles = computed(() => {
  const currentClass = className.value.charAt(0).toUpperCase() + className.value.slice(1)
  return fightingStyles.value.filter((style) => {
    if (!style.availableToClasses || style.availableToClasses.length === 0) return true
    return style.availableToClasses.includes(currentClass)
  })
})

// Filtered maneuvers
const filteredManeuvers = computed(() => {
  if (!maneuverSearch.value.trim()) return maneuverOptions.value
  const search = maneuverSearch.value.toLowerCase()
  return maneuverOptions.value.filter((m) => m.name.toLowerCase().includes(search))
})

// Available invocations (filtered by prereqs)
const availableInvocations = computed(() => {
  return invocationOptions.value.filter((inv) => meetsInvocationPrereqs(inv))
})

// Filtered invocations
const filteredInvocations = computed(() => {
  let list = availableInvocations.value
  if (invocationSearch.value.trim()) {
    const search = invocationSearch.value.toLowerCase()
    list = list.filter((i) => i.name.toLowerCase().includes(search))
  }
  return list
})

// Proficient skills from character
const proficientSkills = computed(() => {
  return props.character.proficiencies
    .filter((p) => p.proficiency_type === 'skill')
    .map((p) => p.name)
})

// Load catalog data
async function loadFightingStyles() {
  try {
    const result = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        entries?: unknown[]
        available_to_classes?: string[]
      }>
    }>('list_fighting_styles')

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
    const result = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        entries?: unknown[]
      }>
    }>('list_metamagic')

    if (result.success && result.data) {
      metamagicOptions.value = result.data.map((meta) => {
        // Parse sorcery point cost from entries if available
        const cost = extractSorceryPointCost(meta.name)
        return {
          name: meta.name,
          source: meta.source,
          cost
        }
      })
    }
  } catch (e) {
    console.error('Error loading metamagic:', e)
  }
}

async function loadManeuvers() {
  try {
    const result = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        entries?: unknown[]
      }>
    }>('list_maneuvers')

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
    const result = await invoke<{
      success: boolean
      data: Array<{
        name: string
        source: string
        level_prereq?: number
        pact_prereq?: string
        spell_prereq?: string
      }>
    }>('list_invocations')

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

// Helper functions
function extractDescription(entries: unknown[] | undefined): string | undefined {
  if (!entries || !Array.isArray(entries)) return undefined
  const firstEntry = entries.find((e) => typeof e === 'string')
  return firstEntry as string | undefined
}

function extractSorceryPointCost(name: string): number | undefined {
  // Common metamagic costs
  const costs: Record<string, number> = {
    'Careful Spell': 1,
    'Distant Spell': 1,
    'Empowered Spell': 1,
    'Extended Spell': 1,
    'Heightened Spell': 3,
    'Quickened Spell': 2,
    'Seeking Spell': 2,
    'Subtle Spell': 1,
    'Transmuted Spell': 1,
    'Twinned Spell': 1 // Variable but minimum 1
  }
  return costs[name]
}

function truncateDescription(text: string | undefined, maxLength = 100): string {
  if (!text) return ''
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

function meetsInvocationPrereqs(inv: InvocationOption): boolean {
  // Check level
  if (inv.levelPrereq && classLevel.value < inv.levelPrereq) {
    return false
  }

  // Check pact boon
  if (inv.pactPrereq) {
    const hasPact =
      selectedPactBoon.value?.name.toLowerCase().includes(inv.pactPrereq.toLowerCase()) ||
      props.levelUp.featureChoices.value?.pact_boon?.name
        .toLowerCase()
        .includes(inv.pactPrereq.toLowerCase())
    // For existing characters, check their current pact
    // TODO: Check character's current pact boon if they have one
    if (!hasPact) return false
  }

  return true
}

// Selection handlers
function isStyleSelected(style: FightingStyleOption): boolean {
  return selectedFightingStyle.value?.name === style.name
}

function selectFightingStyle(style: FightingStyleOption) {
  selectedFightingStyle.value = { name: style.name, source: style.source }
  updateFeatureChoices()
}

function isMetamagicSelected(meta: MetamagicOption): boolean {
  return selectedMetamagic.value.some((m) => m.name === meta.name)
}

function toggleMetamagic(meta: MetamagicOption) {
  const index = selectedMetamagic.value.findIndex((m) => m.name === meta.name)
  if (index >= 0) {
    selectedMetamagic.value.splice(index, 1)
  } else if (selectedMetamagic.value.length < metamagicSlots.value) {
    selectedMetamagic.value.push({ name: meta.name, source: meta.source })
  }
  updateFeatureChoices()
}

function isManeuverSelected(maneuver: ManeuverOption): boolean {
  return selectedManeuvers.value.some((m) => m.name === maneuver.name)
}

function toggleManeuver(maneuver: ManeuverOption) {
  const index = selectedManeuvers.value.findIndex((m) => m.name === maneuver.name)
  if (index >= 0) {
    selectedManeuvers.value.splice(index, 1)
  } else if (selectedManeuvers.value.length < maneuverSlots.value) {
    selectedManeuvers.value.push({ name: maneuver.name, source: maneuver.source })
  }
  updateFeatureChoices()
}

function isInvocationSelected(inv: InvocationOption): boolean {
  return selectedInvocations.value.some((i) => i.name === inv.name)
}

function toggleInvocation(inv: InvocationOption) {
  if (!meetsInvocationPrereqs(inv)) return

  const index = selectedInvocations.value.findIndex((i) => i.name === inv.name)
  if (index >= 0) {
    selectedInvocations.value.splice(index, 1)
  } else if (selectedInvocations.value.length < invocationSlots.value) {
    selectedInvocations.value.push({ name: inv.name, source: inv.source })
  }
  updateFeatureChoices()
}

function selectPactBoon(pact: PactBoon) {
  selectedPactBoon.value = pact
  updateFeatureChoices()
}

function toggleExpertise(skill: string) {
  const index = selectedExpertise.value.indexOf(skill)
  if (index >= 0) {
    selectedExpertise.value.splice(index, 1)
  } else if (selectedExpertise.value.length < expertiseSlots.value) {
    selectedExpertise.value.push(skill)
  }
  updateFeatureChoices()
}

function addManualExpertise() {
  if (!manualExpertise.value.trim() || selectedExpertise.value.length >= expertiseSlots.value)
    return
  selectedExpertise.value.push(manualExpertise.value.trim())
  manualExpertise.value = ''
  updateFeatureChoices()
}

function removeExpertise(skill: string) {
  const index = selectedExpertise.value.indexOf(skill)
  if (index >= 0) {
    selectedExpertise.value.splice(index, 1)
    updateFeatureChoices()
  }
}

function updateFeatureChoices() {
  const choices: NonNullable<typeof props.levelUp.featureChoices.value> = {}

  // Fighting style
  if (selectedFightingStyle.value) {
    choices.fighting_style = selectedFightingStyle.value
  } else if (manualFightingStyle.value) {
    choices.fighting_style = { name: manualFightingStyle.value, source: 'PHB' }
  }

  // Metamagic
  if (selectedMetamagic.value.length > 0) {
    choices.metamagic = selectedMetamagic.value
  }

  // Maneuvers
  if (selectedManeuvers.value.length > 0) {
    choices.maneuvers = { new_maneuvers: selectedManeuvers.value }
  }

  // Invocations
  if (selectedInvocations.value.length > 0) {
    choices.invocations = { new_invocations: selectedInvocations.value }
  }

  // Pact boon
  if (selectedPactBoon.value) {
    choices.pact_boon = { name: selectedPactBoon.value.name, source: selectedPactBoon.value.source }
  }

  // Expertise
  if (selectedExpertise.value.length > 0) {
    choices.expertise_skills = selectedExpertise.value
  }

  props.levelUp.featureChoices.value = Object.keys(choices).length > 0 ? choices : null
}

// Initialize from existing state
watch(
  () => props.levelUp.featureChoices.value,
  (features) => {
    if (features) {
      if (features.fighting_style) {
        selectedFightingStyle.value = features.fighting_style
      }
      if (features.pact_boon) {
        selectedPactBoon.value = {
          ...features.pact_boon,
          description: pactBoons.find((p) => p.name === features.pact_boon?.name)?.description || ''
        }
      }
      if (features.metamagic) {
        selectedMetamagic.value = [...features.metamagic]
      }
      if (features.maneuvers?.new_maneuvers) {
        selectedManeuvers.value = [...features.maneuvers.new_maneuvers]
      }
      if (features.invocations?.new_invocations) {
        selectedInvocations.value = [...features.invocations.new_invocations]
      }
      if (features.expertise_skills) {
        selectedExpertise.value = [...features.expertise_skills]
      }
    }
  },
  { immediate: true }
)

// Watch manual fighting style for update
watch(manualFightingStyle, () => {
  if (manualFightingStyle.value) {
    selectedFightingStyle.value = null
    updateFeatureChoices()
  }
})

// Load data on mount
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

.feature-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.section-note {
  margin: 0;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.search-box {
  margin-bottom: var(--spacing-sm);
}

.search-input {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.feature-grid.compact {
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--spacing-sm);
}

.feature-card {
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

.feature-card.compact {
  padding: var(--spacing-sm) var(--spacing-md);
}

.feature-card:hover:not(.disabled) {
  border-color: var(--color-primary-300);
  background: var(--color-surface-variant);
}

.feature-card.selected {
  border-color: var(--color-primary-500);
  background: var(--color-surface-hover);
}

.feature-card.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.feature-card.has-prereq {
  border-style: dashed;
}

.feature-name {
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--color-text);
}

.feature-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.feature-description {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.feature-cost {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-primary-600);
}

.feature-prereq {
  display: flex;
  gap: var(--spacing-xs);
  flex-wrap: wrap;
}

.prereq-item {
  font-size: 0.7rem;
  padding: 2px 6px;
  background: var(--color-warning-bg, #fef3c7);
  color: var(--color-warning-text, #92400e);
  border-radius: var(--radius-sm);
}

.selected-list {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
  padding: var(--spacing-sm);
  background: var(--color-surface);
  border-radius: var(--radius-md);
}

.selected-label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.selected-tag {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-hover);
  color: var(--color-text);
  border: 1px solid var(--color-primary-500);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  font-weight: 500;
}

.remove-tag {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  font-size: 1rem;
  line-height: 1;
  padding: 0;
  opacity: 0.7;
}

.remove-tag:hover {
  opacity: 1;
}

.manual-fallback {
  display: flex;
  gap: var(--spacing-md);
  align-items: flex-end;
}

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
</style>
