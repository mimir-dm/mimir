<template>
  <div class="sheet-content three-columns">
    <!-- Left Column: Abilities & Combat -->
    <div class="sheet-column">
      <!-- Ability Scores -->
      <section class="sheet-section">
        <h2>Ability Scores</h2>
        <div class="ability-grid">
          <div v-for="ability in ABILITIES" :key="ability" class="ability-box">
            <div class="ability-name">{{ ability.slice(0, 3).toUpperCase() }}</div>
            <div class="ability-value">{{ character[ability] }}</div>
            <div class="ability-modifier">
              {{ formatMod(getModifier(character[ability])) }}
            </div>
          </div>
        </div>
      </section>

      <!-- Combat Stats -->
      <section class="sheet-section">
        <h2>Combat</h2>
        <div class="combat-grid">
          <div class="combat-stat">
            <span class="stat-label">Armor Class</span>
            <span class="stat-value">{{ baseAC }}</span>
            <span v-if="equippedArmor" class="stat-note">{{ equippedArmor.item_name }}</span>
          </div>
          <div class="combat-stat">
            <span class="stat-label">Initiative</span>
            <span class="stat-value">{{ formatMod(getModifier(character.dexterity)) }}</span>
          </div>
          <div class="combat-stat">
            <span class="stat-label">Speed</span>
            <span class="stat-value">{{ speed }} ft</span>
          </div>
          <div class="combat-stat">
            <span class="stat-label">Passive Perception</span>
            <span class="stat-value">{{ passivePerception }}</span>
          </div>
          <div class="combat-stat">
            <span class="stat-label">Hit Dice</span>
            <span class="stat-value">{{ hitDice }}</span>
          </div>
          <div class="combat-stat">
            <span class="stat-label">Proficiency</span>
            <span class="stat-value">{{ formatMod(profBonus) }}</span>
          </div>
        </div>
      </section>

      <!-- Saving Throws -->
      <section class="sheet-section">
        <h2>Saving Throws</h2>
        <div class="saves-list">
          <div v-for="ability in ABILITIES" :key="ability" class="save-item">
            <span class="save-proficient" :class="{ active: isSaveProficient(ability) }">
              *
            </span>
            <span class="save-name">{{ ability }}</span>
            <span class="save-bonus">
              {{ formatMod(getSaveBonus(character, ability, character[ability])) }}
            </span>
          </div>
        </div>
      </section>

      <!-- Attacks -->
      <section v-if="attacks.length > 0" class="sheet-section">
        <h2>Attacks</h2>
        <div class="attacks-list">
          <div v-for="attack in attacks" :key="attack.name" class="attack-item">
            <span class="attack-name">{{ attack.name }}</span>
            <span class="attack-bonus">{{ formatMod(attack.attackBonus) }}</span>
            <span class="attack-damage">{{ attack.damage }}</span>
          </div>
        </div>
      </section>
    </div>

    <!-- Middle Column: Skills -->
    <div class="sheet-column">
      <section class="sheet-section skills-section">
        <h2>Skills</h2>
        <div class="skills-list">
          <div v-for="skill in ALL_SKILLS" :key="skill.name" class="skill-item">
            <span
              class="skill-proficient"
              :class="{
                active: isSkillProficient(skill.name),
                expertise: hasExpertise(skill.name),
              }"
            >
              {{ hasExpertise(skill.name) ? '**' : '*' }}
            </span>
            <span class="skill-name">{{ skill.name }}</span>
            <span class="skill-ability">({{ skill.ability.slice(0, 3) }})</span>
            <span class="skill-bonus">
              {{ formatMod(getSkillBonus(character, skill.name, character[skill.ability])) }}
            </span>
          </div>
        </div>
      </section>
    </div>

    <!-- Right Column: Features & Proficiencies -->
    <div class="sheet-column">
      <!-- Proficiencies -->
      <section class="sheet-section">
        <h2>Proficiencies</h2>
        <div v-if="armorProficiencies.length" class="proficiency-group">
          <strong>Armor:</strong> {{ armorProficiencies.map((p) => p.name).join(', ') }}
        </div>
        <div v-if="weaponProficiencies.length" class="proficiency-group">
          <strong>Weapons:</strong> {{ weaponProficiencies.map((p) => p.name).join(', ') }}
        </div>
        <div v-if="toolProficiencies.length" class="proficiency-group">
          <strong>Tools:</strong> {{ toolProficiencies.map((p) => p.name).join(', ') }}
        </div>
        <div v-if="languages.length" class="proficiency-group">
          <strong>Languages:</strong> {{ languages.map((p) => p.name).join(', ') }}
        </div>
        <div v-if="!hasProficiencies" class="empty-proficiencies">
          No proficiencies recorded
        </div>
      </section>

      <!-- Class Features -->
      <section v-if="classFeatures.length > 0" class="sheet-section">
        <h2>Class Features</h2>
        <div class="features-list">
          <div
            v-for="feature in classFeatures"
            :key="`${feature.class_name}-${feature.name}-${feature.level}`"
            class="feature-item"
            :class="{ expanded: isFeatureExpanded(feature) }"
          >
            <div class="feature-header" @click="toggleFeatureExpansion(feature)">
              <span class="feature-name">
                <span class="expand-icon">{{ isFeatureExpanded(feature) ? '▼' : '▶' }}</span>
                {{ feature.name }}
              </span>
              <span class="feature-meta">
                <span v-if="feature.subclass_name" class="subclass-badge">{{ feature.subclass_name }}</span>
                {{ feature.class_name }} {{ feature.level }}
              </span>
            </div>
            <div v-if="isFeatureExpanded(feature)" class="feature-details">
              <div v-if="isFeatureLoading(feature)" class="feature-loading">Loading...</div>
              <div v-else-if="getFeatureDescription(feature)" class="feature-description" v-html="getFeatureDescription(feature)"></div>
              <div v-else class="feature-no-desc">No description available</div>
            </div>
          </div>
        </div>
      </section>

      <!-- Spellcasting Summary -->
      <section v-if="characterIsSpellcaster" class="sheet-section">
        <h2>Spellcasting</h2>
        <div class="spell-stats">
          <div class="spell-stat">
            <span class="stat-label">Spell Save DC</span>
            <span class="stat-value">{{ spellSaveDC }}</span>
          </div>
          <div class="spell-stat">
            <span class="stat-label">Spell Attack</span>
            <span class="stat-value">{{ formatMod(spellAttackBonus || 0) }}</span>
          </div>
          <div class="spell-stat">
            <span class="stat-label">Ability</span>
            <span class="stat-value spell-ability">
              {{ spellcastingAbility?.toUpperCase().slice(0, 3) }}
            </span>
          </div>
        </div>
        <p class="spell-note">See Spells tab for full spell list</p>
      </section>

      <!-- Personality -->
      <section v-if="hasPersonality" class="sheet-section">
        <h2>Personality</h2>
        <div v-if="character.traits" class="personality-item">
          <strong>Traits:</strong> {{ character.traits }}
        </div>
        <div v-if="character.ideals" class="personality-item">
          <strong>Ideals:</strong> {{ character.ideals }}
        </div>
        <div v-if="character.bonds" class="personality-item">
          <strong>Bonds:</strong> {{ character.bonds }}
        </div>
        <div v-if="character.flaws" class="personality-item">
          <strong>Flaws:</strong> {{ character.flaws }}
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Character, CharacterInventory } from '@/types/character'
import {
  ALL_SKILLS,
  ABILITIES,
  getModifier,
  formatModifier,
  getProficiencyBonus,
  getTotalLevel,
  getProficienciesByType,
  isProficientInSkill,
  hasSkillExpertise,
  isProficientInSave,
  getSkillBonus,
  getSaveBonus,
  getPassivePerception,
  getArmorAC,
  getWeaponDamage,
  isFinesse,
  isRanged,
  getHitDiceString,
} from '@/utils/characterUtils'
import { processFormattingTags } from '../../../sources/utils/textFormatting'

// Class feature from catalog
interface ClassFeature {
  name: string
  source: string
  class_name: string
  class_source: string
  level: number
  data: string
  subclass_name?: string
  subclass_short_name?: string
  subclass_source?: string
}

const props = defineProps<{
  character: Character
  inventory: CharacterInventory[]
  classFeatures: ClassFeature[]
  speed: number
  characterIsSpellcaster: boolean
  spellcastingAbility: string | null
  spellSaveDC: number | null
  spellAttackBonus: number | null
}>()

// Local state for feature expansion
const expandedFeatures = ref<Set<string>>(new Set())
const featureDetails = ref<Record<string, Record<string, unknown>>>({})
const loadingFeature = ref<string | null>(null)

// Computed properties
const totalLevel = computed(() => getTotalLevel(props.character))
const profBonus = computed(() => getProficiencyBonus(totalLevel.value))
const hitDice = computed(() => getHitDiceString(props.character))
const passivePerception = computed(() => getPassivePerception(props.character))

// Proficiency helpers
const armorProficiencies = computed(() => getProficienciesByType(props.character, 'armor'))
const weaponProficiencies = computed(() => getProficienciesByType(props.character, 'weapon'))
const toolProficiencies = computed(() => getProficienciesByType(props.character, 'tool'))
const languages = computed(() => getProficienciesByType(props.character, 'language'))
const hasProficiencies = computed(
  () =>
    armorProficiencies.value.length > 0 ||
    weaponProficiencies.value.length > 0 ||
    toolProficiencies.value.length > 0 ||
    languages.value.length > 0
)

// Skill/save proficiency checks
const isSkillProficient = (skillName: string) => isProficientInSkill(props.character, skillName)
const hasExpertise = (skillName: string) => hasSkillExpertise(props.character, skillName)
const isSaveProficient = (ability: string) => isProficientInSave(props.character, ability)

// Equipment
const equippedItems = computed(() => props.inventory.filter((i) => i.equipped !== 0))
const equippedArmor = computed(() =>
  equippedItems.value.find((i) => {
    const name = i.item_name.toLowerCase()
    return (
      name.includes('armor') ||
      name.includes('mail') ||
      name.includes('hide') ||
      name.includes('leather') ||
      name.includes('plate') ||
      name.includes('robe')
    )
  })
)
const equippedWeapons = computed(() =>
  equippedItems.value.filter((i) => {
    const name = i.item_name.toLowerCase()
    return (
      name.includes('sword') ||
      name.includes('axe') ||
      name.includes('bow') ||
      name.includes('dagger') ||
      name.includes('mace') ||
      name.includes('staff') ||
      name.includes('crossbow') ||
      name.includes('spear') ||
      name.includes('hammer')
    )
  })
)

// AC calculation
const baseAC = computed(() => {
  const dexMod = getModifier(props.character.dexterity)
  if (equippedArmor.value) {
    return getArmorAC(equippedArmor.value.item_name, dexMod)
  }
  return 10 + dexMod
})

// Attacks from equipped weapons
const attacks = computed(() => {
  if (equippedWeapons.value.length === 0) return []

  const strMod = getModifier(props.character.strength)
  const dexMod = getModifier(props.character.dexterity)
  const prof = profBonus.value

  return equippedWeapons.value.map((weapon) => {
    let abilityMod = strMod
    if (isRanged(weapon.item_name)) {
      abilityMod = dexMod
    } else if (isFinesse(weapon.item_name) && dexMod > strMod) {
      abilityMod = dexMod
    }

    return {
      name: weapon.item_name,
      attackBonus: prof + abilityMod,
      damage: getWeaponDamage(weapon.item_name, abilityMod),
    }
  })
})

// Personality check
const hasPersonality = computed(() => {
  return (
    props.character.traits ||
    props.character.ideals ||
    props.character.bonds ||
    props.character.flaws
  )
})

// Methods
const formatMod = (mod: number) => formatModifier(mod)

// Class Feature helpers
const toggleFeatureExpansion = async (feature: ClassFeature) => {
  const key = `${feature.name}|${feature.class_name}`

  if (expandedFeatures.value.has(key)) {
    expandedFeatures.value.delete(key)
    expandedFeatures.value = new Set(expandedFeatures.value)
    return
  }

  if (!featureDetails.value[key]) {
    loadingFeature.value = key
    try {
      const result = await invoke<{ success: boolean; data?: Record<string, unknown>; error?: string }>(
        'get_class_feature',
        { name: feature.name, className: feature.class_name }
      )
      if (result.success && result.data) {
        featureDetails.value = { ...featureDetails.value, [key]: result.data }
      }
    } catch (e) {
      console.error('Failed to load feature details:', e)
    } finally {
      loadingFeature.value = null
    }
  }

  expandedFeatures.value.add(key)
  expandedFeatures.value = new Set(expandedFeatures.value)
}

const isFeatureExpanded = (feature: ClassFeature): boolean => {
  return expandedFeatures.value.has(`${feature.name}|${feature.class_name}`)
}

const getFeatureDescription = (feature: ClassFeature): string => {
  const key = `${feature.name}|${feature.class_name}`
  const details = featureDetails.value[key]
  if (!details) return ''

  const entries = details.entries as unknown[] | undefined
  if (!entries) return ''

  return entries
    .map((entry) => {
      if (typeof entry === 'string') return processFormattingTags(entry)
      if (typeof entry === 'object' && entry !== null) {
        const e = entry as Record<string, unknown>
        if (e.type === 'entries' && Array.isArray(e.entries)) {
          return (e.entries as unknown[])
            .filter((sub) => typeof sub === 'string')
            .map((s) => processFormattingTags(s as string))
            .join(' ')
        }
        if (e.type === 'list' && Array.isArray(e.items)) {
          return (e.items as unknown[])
            .filter((sub) => typeof sub === 'string')
            .map((s) => `• ${processFormattingTags(s as string)}`)
            .join('\n')
        }
      }
      return ''
    })
    .filter(Boolean)
    .join('\n\n')
}

const isFeatureLoading = (feature: ClassFeature): boolean => {
  return loadingFeature.value === `${feature.name}|${feature.class_name}`
}
</script>

<style scoped>
/* Content Layouts */
.sheet-content.three-columns {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--spacing-lg);
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
}
</style>
