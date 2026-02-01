<template>
  <div class="stat-block">
    <div class="stat-block-header">
      <h4>{{ name }}</h4>
      <p class="creature-meta">{{ creatureMeta }}</p>
    </div>

    <div class="stat-divider" />

    <!-- Quick Stats -->
    <div class="quick-stats">
      <div class="quick-stat">
        <span class="stat-label">AC</span>
        <span class="stat-value" v-html="acDisplay"></span>
      </div>
      <div class="quick-stat">
        <span class="stat-label">HP</span>
        <span class="stat-value">{{ hpDisplay }}</span>
      </div>
      <div class="quick-stat">
        <span class="stat-label">Speed</span>
        <span class="stat-value">{{ speedDisplay }}</span>
      </div>
    </div>

    <div class="stat-divider" />

    <!-- Ability Scores -->
    <div class="ability-row">
      <div class="ability-item" v-for="ability in abilities" :key="ability">
        <span class="ability-name">{{ ability.toUpperCase() }}</span>
        <span class="ability-value">{{ data[ability] || 10 }}</span>
        <span class="ability-mod">({{ modStr(data[ability] || 10) }})</span>
      </div>
    </div>

    <div class="stat-divider" />

    <!-- Properties -->
    <div class="properties">
      <div v-if="savesDisplay" class="prop-line"><strong>Saving Throws</strong> {{ savesDisplay }}</div>
      <div v-if="skillsDisplay" class="prop-line"><strong>Skills</strong> {{ skillsDisplay }}</div>
      <div v-if="resistDisplay" class="prop-line"><strong>Damage Resistances</strong> {{ resistDisplay }}</div>
      <div v-if="immuneDisplay" class="prop-line"><strong>Damage Immunities</strong> {{ immuneDisplay }}</div>
      <div v-if="condImmune" class="prop-line"><strong>Condition Immunities</strong> {{ condImmune }}</div>
      <div v-if="sensesDisplay" class="prop-line"><strong>Senses</strong> {{ sensesDisplay }}</div>
      <div v-if="languagesDisplay" class="prop-line"><strong>Languages</strong> {{ languagesDisplay }}</div>
      <div class="prop-line"><strong>Challenge</strong> {{ crDisplay }}</div>
    </div>

    <div class="stat-divider" />

    <!-- Traits -->
    <div v-if="data.trait?.length" class="action-section">
      <div v-for="(t, i) in data.trait" :key="'trait-' + i" class="action-item">
        <strong>{{ t.name }}.</strong>
        <span v-html="formatEntries(t.entries)"></span>
      </div>
    </div>

    <!-- Actions -->
    <div v-if="data.action?.length" class="action-section">
      <h5>Actions</h5>
      <div v-for="(a, i) in data.action" :key="'action-' + i" class="action-item">
        <strong>{{ a.name }}.</strong>
        <span v-html="formatEntries(a.entries)"></span>
      </div>
    </div>

    <!-- Bonus Actions -->
    <div v-if="data.bonus?.length" class="action-section">
      <h5>Bonus Actions</h5>
      <div v-for="(b, i) in data.bonus" :key="'bonus-' + i" class="action-item">
        <strong>{{ b.name }}.</strong>
        <span v-html="formatEntries(b.entries)"></span>
      </div>
    </div>

    <!-- Reactions -->
    <div v-if="data.reaction?.length" class="action-section">
      <h5>Reactions</h5>
      <div v-for="(r, i) in data.reaction" :key="'reaction-' + i" class="action-item">
        <strong>{{ r.name }}.</strong>
        <span v-html="formatEntries(r.entries)"></span>
      </div>
    </div>

    <!-- Legendary Actions -->
    <div v-if="data.legendary?.length" class="action-section">
      <h5>Legendary Actions</h5>
      <div v-for="(l, i) in data.legendary" :key="'legendary-' + i" class="action-item">
        <strong>{{ l.name }}.</strong>
        <span v-html="formatEntries(l.entries)"></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  formatCreatureType,
  formatSpeed,
  formatModifier,
  formatSaves,
  formatSkills,
  formatSenses,
  formatDamageResistances,
  formatDamageImmunities,
  formatConditionImmunities,
  formatCR,
  formatAC,
  formatHP,
  formatActionEntries,
} from '@/features/modules/composables/useModuleMonsters'

const props = defineProps<{
  data: Record<string, any>
  name: string
}>()

const abilities = ['str', 'dex', 'con', 'int', 'wis', 'cha']

function modStr(score: number): string {
  return formatModifier(score)
}

const creatureMeta = computed(() => formatCreatureType(props.data))
const acDisplay = computed(() => formatAC(props.data))
const hpDisplay = computed(() => formatHP(props.data))
const speedDisplay = computed(() => formatSpeed(props.data))
const savesDisplay = computed(() => formatSaves(props.data))
const skillsDisplay = computed(() => formatSkills(props.data))
const resistDisplay = computed(() => formatDamageResistances(props.data))
const immuneDisplay = computed(() => formatDamageImmunities(props.data))
const sensesDisplay = computed(() => formatSenses(props.data))
const crDisplay = computed(() => formatCR(props.data))

const condImmune = computed(() => formatConditionImmunities(props.data))

const languagesDisplay = computed(() => {
  if (!props.data.languages) return ''
  if (Array.isArray(props.data.languages)) return props.data.languages.join(', ')
  return props.data.languages
})

function formatEntries(entries: unknown[]): string {
  return formatActionEntries(entries)
}
</script>

<style scoped>
.stat-block {
  background: var(--color-surface-variant);
  border: 2px solid var(--color-primary-300);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  font-size: 0.9rem;
  line-height: 1.5;
}

.stat-block-header h4 {
  margin: 0;
  font-size: 1.3rem;
  color: var(--color-primary-500);
}

.creature-meta {
  margin: 2px 0 0;
  font-size: 0.85rem;
  font-style: italic;
  color: var(--color-text-secondary);
}

.stat-divider {
  height: 2px;
  background: linear-gradient(to right, var(--color-primary-300), transparent);
  margin: var(--spacing-sm) 0;
}

.quick-stats {
  display: flex;
  gap: var(--spacing-lg);
}

.quick-stat {
  display: flex;
  gap: var(--spacing-xs);
}

.stat-label {
  font-weight: 700;
  color: var(--color-primary-500);
}

.ability-row {
  display: flex;
  justify-content: space-between;
  text-align: center;
}

.ability-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
}

.ability-name {
  font-weight: 700;
  font-size: 0.75rem;
  color: var(--color-primary-500);
}

.ability-value {
  font-weight: 600;
}

.ability-mod {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.properties {
  font-size: 0.85rem;
}

.prop-line {
  margin: 2px 0;
}

.prop-line strong {
  color: var(--color-primary-500);
}

.action-section {
  margin-top: var(--spacing-sm);
}

.action-section h5 {
  margin: 0 0 var(--spacing-xs);
  font-size: 1rem;
  color: var(--color-primary-500);
  border-bottom: 1px solid var(--color-primary-300);
  padding-bottom: 2px;
}

.action-item {
  margin: var(--spacing-xs) 0;
  font-size: 0.85rem;
}

.action-item strong {
  font-style: italic;
}
</style>
