<template>
  <div class="sheet-content single-column">
    <!-- Background -->
    <section v-if="character.background_name" class="sheet-section">
      <h2>Background: {{ character.background_name }}</h2>

      <template v-if="backgroundDetails">
        <!-- Background Proficiencies -->
        <div class="details-card">
          <h3>Proficiencies</h3>
          <div class="proficiency-grid">
            <div v-if="backgroundSkillProficiencies.length > 0" class="prof-item">
              <span class="prof-label">Skills:</span>
              <span>{{ backgroundSkillProficiencies.join(', ') }}</span>
            </div>
            <div v-if="backgroundToolProficiencies.length > 0" class="prof-item">
              <span class="prof-label">Tools:</span>
              <span>{{ backgroundToolProficiencies.join(', ') }}</span>
            </div>
            <div v-if="backgroundLanguages" class="prof-item">
              <span class="prof-label">Languages:</span>
              <span>{{ backgroundLanguages }}</span>
            </div>
          </div>
        </div>

        <!-- Background Equipment -->
        <div v-if="backgroundEquipment" class="details-card">
          <h3>Starting Equipment</h3>
          <p>{{ backgroundEquipment }}</p>
        </div>

        <!-- Background Feature -->
        <div v-if="backgroundFeature" class="details-card feature-card">
          <h3>Feature: {{ backgroundFeature.name }}</h3>
          <p>{{ backgroundFeature.description }}</p>
        </div>
      </template>

      <p v-else class="loading-text">Loading background details...</p>
    </section>

    <!-- NPC Info -->
    <section v-if="character.is_npc === 1 && hasNpcInfo" class="sheet-section">
      <h2>NPC Details</h2>
      <div class="npc-details-grid">
        <div v-if="character.role" class="npc-detail-item">
          <span class="npc-label">Role</span>
          <span class="npc-value">{{ character.role }}</span>
        </div>
        <div v-if="character.location" class="npc-detail-item">
          <span class="npc-label">Location</span>
          <span class="npc-value">{{ character.location }}</span>
        </div>
        <div v-if="character.faction" class="npc-detail-item">
          <span class="npc-label">Faction</span>
          <span class="npc-value">{{ character.faction }}</span>
        </div>
      </div>
    </section>

    <!-- Classes -->
    <section class="sheet-section">
      <h2>Classes</h2>
      <div v-if="character.classes.length === 0" class="empty-state">No classes</div>
      <div v-else class="class-details-list">
        <div v-for="cls in character.classes" :key="cls.id" class="class-detail-card">
          <div class="class-header">
            <h3>
              {{ cls.class_name }}
              <span class="class-level-badge">Level {{ cls.level }}</span>
            </h3>
          </div>

          <!-- Class Mechanical Info -->
          <div class="class-stats-grid">
            <div class="class-stat">
              <span class="class-stat-label">Hit Die</span>
              <span class="class-stat-value">{{ getClassHitDice(cls.class_name) || '—' }}</span>
            </div>
            <div class="class-stat">
              <span class="class-stat-label">Primary Ability</span>
              <span class="class-stat-value">{{ getClassPrimaryAbility(cls.class_name) || '—' }}</span>
            </div>
            <div v-if="getClassSpellcastingAbility(cls.class_name)" class="class-stat">
              <span class="class-stat-label">Spellcasting</span>
              <span class="class-stat-value spellcaster">{{ getClassSpellcastingAbility(cls.class_name) }}</span>
            </div>
            <div class="class-stat">
              <span class="class-stat-label">Saving Throws</span>
              <span class="class-stat-value">{{ getClassSavingThrows(cls.class_name).join(', ') || '—' }}</span>
            </div>
          </div>

          <!-- Starting Proficiencies -->
          <div class="class-proficiencies">
            <h4>Starting Proficiencies</h4>
            <div class="proficiency-grid">
              <div v-if="getClassProficiencies(cls.class_name).armor" class="prof-item">
                <span class="prof-label">Armor:</span>
                <span v-html="getClassProficiencies(cls.class_name).armor"></span>
              </div>
              <div v-if="getClassProficiencies(cls.class_name).weapons" class="prof-item">
                <span class="prof-label">Weapons:</span>
                <span v-html="getClassProficiencies(cls.class_name).weapons"></span>
              </div>
              <div v-if="getClassProficiencies(cls.class_name).tools" class="prof-item">
                <span class="prof-label">Tools:</span>
                <span>{{ getClassProficiencies(cls.class_name).tools }}</span>
              </div>
              <div v-if="getClassProficiencies(cls.class_name).skills" class="prof-item">
                <span class="prof-label">Skills:</span>
                <span>{{ getClassProficiencies(cls.class_name).skills }}</span>
              </div>
            </div>
          </div>

          <!-- Subclass -->
          <div v-if="cls.subclass_name" class="subclass-section">
            <h4 class="subclass-header">
              Subclass: {{ cls.subclass_name }}
            </h4>
            <div v-if="getSubclassDescription(cls.class_name, cls.subclass_name)" class="subclass-description">
              {{ getSubclassDescription(cls.class_name, cls.subclass_name) }}
            </div>
          </div>

          <!-- Class Features for this class -->
          <div class="class-features-section">
            <h4>Features</h4>
            <div class="features-by-level">
              <template v-for="level in getFeatureLevels(cls.class_name, cls.subclass_name)" :key="level">
                <div class="feature-level-group">
                  <span class="level-label">{{ formatOrdinal(level) }} Level:</span>
                  <span class="feature-links">
                    <a
                      v-for="(feature, idx) in getFeaturesAtLevel(cls.class_name, cls.subclass_name, level)"
                      :key="feature.name + (feature.subclass_name || '')"
                      href="#"
                      :class="['cross-ref-link', 'feature-ref', { 'subclass-feature': feature.subclass_name }]"
                      @click.prevent="$emit('open-feature-modal', feature)"
                    >{{ feature.name }}{{ idx < getFeaturesAtLevel(cls.class_name, cls.subclass_name, level).length - 1 ? ', ' : '' }}</a>
                  </span>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Character } from '@/types/character'
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

const props = defineProps<{
  character: Character
  classFeatures: ClassFeature[]
  classData: Record<string, Record<string, unknown>>
  backgroundDetails: BackgroundDetail | null
  subclassDetails: Record<string, SubclassDetail>
}>()

defineEmits<{
  'open-feature-modal': [feature: ClassFeature]
}>()

// NPC check
const hasNpcInfo = computed(() => {
  return props.character.role || props.character.location || props.character.faction
})

// Background helpers
const backgroundSkillProficiencies = computed((): string[] => {
  if (!props.backgroundDetails?.data) return []
  const skillProfs = props.backgroundDetails.data.skillProficiencies as Array<Record<string, boolean>> | undefined
  if (!skillProfs) return []
  return skillProfs.flatMap((sp) => Object.keys(sp).filter((k) => sp[k] && k !== 'choose'))
})

const backgroundToolProficiencies = computed((): string[] => {
  if (!props.backgroundDetails?.data) return []
  const toolProfs = props.backgroundDetails.data.toolProficiencies as Array<Record<string, boolean | string>> | undefined
  if (!toolProfs) return []
  const tools: string[] = []
  for (const tp of toolProfs) {
    for (const [key, val] of Object.entries(tp)) {
      if (key !== 'choose' && val) {
        tools.push(typeof val === 'string' ? val : key)
      }
    }
  }
  return tools
})

const backgroundLanguages = computed((): string => {
  if (!props.backgroundDetails?.data) return ''
  const langs = props.backgroundDetails.data.languageProficiencies as Array<{ anyStandard?: number }> | undefined
  if (!langs || langs.length === 0) return ''
  const lang = langs[0]
  if (lang.anyStandard) return `${lang.anyStandard} of your choice`
  return ''
})

const backgroundEquipment = computed((): string => {
  if (!props.backgroundDetails?.data) return ''
  const entries = props.backgroundDetails.data.entries as unknown[] | undefined
  if (!entries) return ''

  for (const entry of entries) {
    if (typeof entry === 'object' && entry !== null) {
      const e = entry as Record<string, unknown>
      if (e.type === 'list' && e.name === 'Equipment') {
        const items = e.items as string[] | undefined
        if (items) return items.join(', ')
      }
    }
  }
  return ''
})

const backgroundFeature = computed((): { name: string; description: string } | null => {
  if (!props.backgroundDetails?.data) return null
  const entries = props.backgroundDetails.data.entries as unknown[] | undefined
  if (!entries) return null

  for (const entry of entries) {
    if (typeof entry === 'object' && entry !== null) {
      const e = entry as Record<string, unknown>
      if (e.type === 'entries' && e.name && typeof e.name === 'string') {
        const subEntries = e.entries as unknown[] | undefined
        if (subEntries) {
          const desc = subEntries
            .filter((se) => typeof se === 'string')
            .join(' ')
          if (desc) {
            return { name: e.name, description: desc }
          }
        }
      }
    }
  }
  return null
})

// Class mechanical info helpers
const getClassHitDice = (className: string): string => {
  const data = props.classData[className.toLowerCase()]
  if (!data) return ''
  const hd = data.hd as { faces?: number } | undefined
  return hd?.faces ? `d${hd.faces}` : ''
}

const getClassPrimaryAbility = (className: string): string => {
  const data = props.classData[className.toLowerCase()]
  if (!data) return ''

  const statNames: Record<string, string> = {
    str: 'Strength',
    dex: 'Dexterity',
    con: 'Constitution',
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma',
  }

  const primaryAbility = data.primaryAbility
  if (Array.isArray(primaryAbility)) {
    const abilities: string[] = []
    for (const ability of primaryAbility) {
      if (typeof ability === 'object' && ability !== null) {
        for (const [stat, value] of Object.entries(ability)) {
          if (value === true) abilities.push(statNames[stat] || stat.toUpperCase())
        }
      }
    }
    if (abilities.length > 0) return abilities.join(' or ')
  }
  if (typeof primaryAbility === 'object' && primaryAbility !== null) {
    const abilities: string[] = []
    for (const [stat, value] of Object.entries(primaryAbility)) {
      if (value === true) abilities.push(statNames[stat] || stat.toUpperCase())
    }
    if (abilities.length > 0) return abilities.join(' or ')
  }
  const spellcastingAbility = data.spellcastingAbility
  if (typeof spellcastingAbility === 'string') {
    return statNames[spellcastingAbility] || spellcastingAbility.toUpperCase()
  }
  return ''
}

const getClassSpellcastingAbility = (className: string): string => {
  const data = props.classData[className.toLowerCase()]
  if (!data) return ''
  const spellcastingAbility = data.spellcastingAbility as string | undefined
  if (!spellcastingAbility) return ''
  const abilityMap: Record<string, string> = { int: 'INT', wis: 'WIS', cha: 'CHA' }
  return abilityMap[spellcastingAbility] || spellcastingAbility.toUpperCase()
}

const getClassProficiencies = (className: string): { armor: string; weapons: string; tools: string; skills: string } => {
  const data = props.classData[className.toLowerCase()]
  const result = { armor: '', weapons: '', tools: '', skills: '' }
  if (!data) return result

  const sp = data.startingProficiencies as Record<string, unknown> | undefined
  if (!sp) return result

  if (Array.isArray(sp.armor)) {
    const armorItems = sp.armor
      .map((a: unknown) => {
        if (typeof a === 'string') return processFormattingTags(a)
        if (typeof a === 'object' && a !== null && 'proficiency' in a) {
          return processFormattingTags((a as { proficiency: string }).proficiency)
        }
        return null
      })
      .filter(Boolean)
    result.armor = armorItems.join(', ')
  }

  if (Array.isArray(sp.weapons)) {
    const weaponItems = sp.weapons
      .map((w: unknown) => {
        if (typeof w === 'string') return processFormattingTags(w)
        if (typeof w === 'object' && w !== null && 'proficiency' in w) {
          return processFormattingTags((w as { proficiency: string }).proficiency)
        }
        return null
      })
      .filter(Boolean)
    result.weapons = weaponItems.join(', ')
  }

  if (Array.isArray(sp.tools)) {
    result.tools = sp.tools.length > 0 ? `${sp.tools.length} of your choice` : ''
  }

  if (Array.isArray(sp.skills)) {
    const skillChoices = sp.skills.map((s: unknown) => {
      if (typeof s === 'object' && s !== null && 'choose' in s) {
        const choose = s as { choose: { from?: string[]; count: number } }
        return `Choose ${choose.choose.count}`
      }
      return null
    }).filter(Boolean)
    result.skills = skillChoices.join(', ')
  }

  return result
}

const getClassSavingThrows = (className: string): string[] => {
  const data = props.classData[className.toLowerCase()]
  if (!data) return []
  const profs = data.proficiency as Array<{ [key: string]: boolean }> | undefined
  if (!Array.isArray(profs)) return []

  const statNames: Record<string, string> = {
    str: 'Strength',
    dex: 'Dexterity',
    con: 'Constitution',
    int: 'Intelligence',
    wis: 'Wisdom',
    cha: 'Charisma',
  }

  const saves: string[] = []
  for (const prof of profs) {
    for (const [stat, value] of Object.entries(prof)) {
      if (value === true && statNames[stat]) {
        saves.push(statNames[stat])
      }
    }
  }
  return saves
}

// Subclass description
const getSubclassDescription = (className: string, subclassName: string): string => {
  const key = `${className}|${subclassName}`
  const subclass = props.subclassDetails[key]
  if (!subclass?.data) return ''

  const entries = subclass.data.entries as unknown[] | undefined
  if (!entries) return ''

  for (const entry of entries) {
    if (typeof entry === 'string') return entry
    if (typeof entry === 'object' && entry !== null) {
      const e = entry as Record<string, unknown>
      if (e.type === 'entries' && Array.isArray(e.entries)) {
        const firstText = (e.entries as unknown[]).find((sub) => typeof sub === 'string')
        if (firstText) return firstText as string
      }
    }
  }
  return ''
}

// Feature display helpers
const getFeatureLevels = (className: string, subclassName?: string | null): number[] => {
  const features = props.classFeatures.filter((f) => {
    const classMatch = f.class_name?.toLowerCase() === className.toLowerCase()
    if (!classMatch) return false
    if (f.subclass_name) {
      return f.subclass_name === subclassName
    }
    return true
  })
  const levels = [...new Set(features.map((f) => f.level))].sort((a, b) => a - b)
  return levels
}

const getFeaturesAtLevel = (className: string, subclassName: string | null | undefined, level: number): ClassFeature[] => {
  return props.classFeatures.filter((f) => {
    const classMatch = f.class_name?.toLowerCase() === className.toLowerCase()
    const levelMatch = f.level === level
    if (!classMatch || !levelMatch) return false
    if (f.subclass_name) {
      return f.subclass_name === subclassName
    }
    return true
  })
}

const formatOrdinal = (n: number): string => {
  const suffixes = ['th', 'st', 'nd', 'rd']
  const v = n % 100
  return n + (suffixes[(v - 20) % 10] || suffixes[v] || suffixes[0])
}
</script>

<style scoped>
/* Content Layouts */
.sheet-content.single-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  max-width: 700px;
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

/* Empty states */
.empty-state {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-style: italic;
}
</style>
