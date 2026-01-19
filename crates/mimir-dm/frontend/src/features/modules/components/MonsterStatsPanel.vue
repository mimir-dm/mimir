<template>
  <aside class="monster-panel" :class="{ collapsed: !panelOpen }">
    <button class="monster-panel-toggle" @click="$emit('update:panelOpen', !panelOpen)">
      <span>{{ panelOpen ? '›' : '‹' }}</span>
    </button>

    <div class="monster-panel-content" v-show="panelOpen">
      <!-- Monster Header -->
      <header class="monster-header">
        <div class="monster-title">
          <h2>{{ monster.display_name || monster.monster_name }}</h2>
          <p v-if="monster.display_name" class="monster-alias">({{ monster.monster_name }})</p>
          <p class="monster-type">{{ formatCreatureType(monster.monster_data) }}</p>
        </div>
        <button class="close-monster" @click="$emit('close')" title="Close">×</button>
      </header>

      <!-- DM Notes -->
      <div v-if="monster.notes" class="dm-notes">
        <div class="dm-notes-label">DM Notes</div>
        <div class="dm-notes-content">{{ monster.notes }}</div>
      </div>

      <div class="monster-body" v-if="monster.monster_data">
        <!-- Quick Stats Bar -->
        <div class="quick-stats">
          <div class="quick-stat">
            <span class="stat-label">AC</span>
            <span class="stat-value" v-html="formatAC(monster.monster_data)"></span>
          </div>
          <div class="quick-stat">
            <span class="stat-label">HP</span>
            <span class="stat-value">{{ formatHP(monster.monster_data) }}</span>
          </div>
          <div class="quick-stat">
            <span class="stat-label">Speed</span>
            <span class="stat-value">{{ formatSpeed(monster.monster_data) }}</span>
          </div>
        </div>

        <!-- Ability Scores -->
        <div class="ability-row">
          <div class="ability-item" v-for="ability in ['str', 'dex', 'con', 'int', 'wis', 'cha']" :key="ability">
            <span class="ability-name">{{ ability.toUpperCase() }}</span>
            <span class="ability-value">{{ monster.monster_data[ability] || 10 }}</span>
            <span class="ability-mod">{{ formatModifier(monster.monster_data[ability] || 10) }}</span>
          </div>
        </div>

        <!-- Secondary Properties (collapsible) -->
        <details class="stat-section" open>
          <summary>Properties</summary>
          <div class="properties-list">
            <div v-if="formatSaves(monster.monster_data)" class="prop-line">
              <span class="prop-name">Saves</span>
              <span>{{ formatSaves(monster.monster_data) }}</span>
            </div>
            <div v-if="formatSkills(monster.monster_data)" class="prop-line">
              <span class="prop-name">Skills</span>
              <span>{{ formatSkills(monster.monster_data) }}</span>
            </div>
            <div v-if="formatDamageResistances(monster.monster_data)" class="prop-line">
              <span class="prop-name">Resist</span>
              <span>{{ formatDamageResistances(monster.monster_data) }}</span>
            </div>
            <div v-if="formatDamageImmunities(monster.monster_data)" class="prop-line">
              <span class="prop-name">Immune</span>
              <span>{{ formatDamageImmunities(monster.monster_data) }}</span>
            </div>
            <div v-if="formatConditionImmunities(monster.monster_data)" class="prop-line">
              <span class="prop-name">Cond. Immune</span>
              <span>{{ formatConditionImmunities(monster.monster_data) }}</span>
            </div>
            <div v-if="formatSenses(monster.monster_data)" class="prop-line">
              <span class="prop-name">Senses</span>
              <span>{{ formatSenses(monster.monster_data) }}</span>
            </div>
            <div class="prop-line">
              <span class="prop-name">CR</span>
              <span>{{ formatCR(monster.monster_data) }}</span>
            </div>
          </div>
        </details>

        <!-- Traits -->
        <details v-if="monster.monster_data.trait?.length" class="stat-section">
          <summary>Traits</summary>
          <div class="action-list">
            <div v-for="(trait, idx) in monster.monster_data.trait" :key="'trait-' + idx" class="action-item">
              <strong>{{ trait.name }}.</strong>
              <span v-html="formatActionEntries(trait.entries)"></span>
            </div>
          </div>
        </details>

        <!-- Actions -->
        <details v-if="monster.monster_data.action?.length" class="stat-section actions" open>
          <summary>Actions</summary>
          <div class="action-list">
            <div v-for="(action, idx) in monster.monster_data.action" :key="'action-' + idx" class="action-item">
              <strong>{{ action.name }}.</strong>
              <span v-html="formatActionEntries(action.entries)"></span>
            </div>
          </div>
        </details>

        <!-- Bonus Actions -->
        <details v-if="monster.monster_data.bonus?.length" class="stat-section">
          <summary>Bonus Actions</summary>
          <div class="action-list">
            <div v-for="(bonus, idx) in monster.monster_data.bonus" :key="'bonus-' + idx" class="action-item">
              <strong>{{ bonus.name }}.</strong>
              <span v-html="formatActionEntries(bonus.entries)"></span>
            </div>
          </div>
        </details>

        <!-- Reactions -->
        <details v-if="monster.monster_data.reaction?.length" class="stat-section">
          <summary>Reactions</summary>
          <div class="action-list">
            <div v-for="(reaction, idx) in monster.monster_data.reaction" :key="'reaction-' + idx" class="action-item">
              <strong>{{ reaction.name }}.</strong>
              <span v-html="formatActionEntries(reaction.entries)"></span>
            </div>
          </div>
        </details>

        <!-- Legendary Actions -->
        <details v-if="monster.monster_data.legendary?.length" class="stat-section legendary">
          <summary>Legendary Actions</summary>
          <p class="legendary-intro">3 actions per round, at end of other creature's turn.</p>
          <div class="action-list">
            <div v-for="(legendary, idx) in monster.monster_data.legendary" :key="'legendary-' + idx" class="action-item">
              <strong>{{ legendary.name }}.</strong>
              <span v-html="formatActionEntries(legendary.entries)"></span>
            </div>
          </div>
        </details>
      </div>

      <!-- Source -->
      <footer class="monster-footer">
        <span class="source-tag">{{ monster.monster_source }}</span>
      </footer>
    </div>
  </aside>
</template>

<script setup lang="ts">
import {
  type MonsterWithData,
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
  formatActionEntries
} from '../composables/useModuleMonsters'

defineProps<{
  monster: MonsterWithData
  panelOpen: boolean
}>()

defineEmits<{
  'update:panelOpen': [value: boolean]
  close: []
}>()
</script>

<style scoped>
/* Monster Panel - Slides in from right */
.monster-panel {
  width: 380px;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  position: relative;
  transition: width 0.3s ease, opacity 0.3s ease;
  overflow: hidden;
}

.monster-panel.collapsed {
  width: 32px;
}

.monster-panel-toggle {
  position: absolute;
  left: -1px;
  top: 50%;
  transform: translateY(-50%);
  width: 24px;
  height: 48px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-right: none;
  border-radius: 6px 0 0 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  color: var(--color-text-muted);
  z-index: 10;
}

.monster-panel-toggle:hover {
  background: var(--color-base-200);
  color: var(--color-text);
}

.monster-panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Monster Header */
.monster-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--color-base-200);
  border-bottom: 2px solid var(--color-dnd-creature, #ff9f43);
}

.monster-title h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--color-text);
  line-height: 1.2;
}

.monster-alias {
  margin: 0.1rem 0 0 0;
  font-size: 0.7rem;
  color: var(--color-text-secondary);
}

.monster-type {
  margin: 0.15rem 0 0 0;
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-muted);
}

/* DM Notes Section */
.dm-notes {
  padding: 0.5rem 1rem;
  background: var(--color-primary-100);
  border-bottom: 1px solid var(--color-border);
}

.dm-notes-label {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-primary-600);
  margin-bottom: 0.25rem;
}

.dm-notes-content {
  font-size: 0.8rem;
  color: var(--color-text);
  line-height: 1.4;
  white-space: pre-wrap;
}

.close-monster {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.close-monster:hover {
  color: var(--color-text);
}

/* Monster Body */
.monster-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
}

/* Quick Stats */
.quick-stats {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.quick-stat {
  flex: 1;
  text-align: left;
  padding: 0.5rem;
  background: var(--color-base-200);
  border-radius: 0.375rem;
  border: 1px solid var(--color-border);
}

.quick-stat .stat-label {
  display: block;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
  margin-bottom: 0.15rem;
}

.quick-stat .stat-value {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
}

/* Ability Row */
.ability-row {
  display: flex;
  justify-content: space-between;
  gap: 0.25rem;
  margin-bottom: 0.75rem;
  padding: 0.5rem;
  background: var(--color-base-200);
  border-radius: 0.375rem;
}

.ability-item {
  flex: 1;
  text-align: left;
}

.ability-item .ability-name {
  display: block;
  font-size: 0.6rem;
  font-weight: 700;
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.ability-item .ability-value {
  display: block;
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--color-text);
}

.ability-item .ability-mod {
  display: block;
  font-size: 0.7rem;
  color: var(--color-dnd-creature, #ff9f43);
  font-weight: 600;
}

/* Stat Sections (using native details/summary) */
.stat-section {
  margin-bottom: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  overflow: hidden;
  text-align: left;
}

.stat-section summary {
  padding: 0.5rem 0.75rem;
  background: var(--color-base-200);
  font-size: 0.8rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: var(--color-text);
  cursor: pointer;
  user-select: none;
  text-align: left;
}

.stat-section summary:hover {
  background: var(--color-base-300);
}

.stat-section.actions summary {
  color: var(--color-dnd-damage, #ff6b6b);
}

.stat-section.legendary summary {
  color: #9333ea;
}

/* Properties List */
.properties-list {
  padding: 0.5rem 0.75rem;
  text-align: left;
}

.prop-line {
  display: flex;
  gap: 0.5rem;
  font-size: 0.8rem;
  line-height: 1.4;
  margin-bottom: 0.25rem;
}

.prop-line:last-child {
  margin-bottom: 0;
}

.prop-name {
  font-weight: 600;
  color: var(--color-text-muted);
  min-width: 5rem;
  flex-shrink: 0;
}

/* Action List */
.action-list {
  padding: 0.5rem 0.75rem;
  text-align: left;
}

.action-item {
  font-size: 0.8rem;
  line-height: 1.5;
  margin-bottom: 0.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--color-border);
}

.action-item:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

.action-item strong {
  color: var(--color-text);
}

.legendary-intro {
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-muted);
  margin: 0 0.75rem 0.5rem;
  padding-top: 0.5rem;
}

/* Cross-ref styling within monster panel */
.monster-panel :deep(.cross-ref-link),
.monster-panel :deep(.spell-ref),
.monster-panel :deep(.item-ref),
.monster-panel :deep(.condition-ref) {
  color: var(--color-primary, #4a9eff);
  text-decoration: underline;
  text-decoration-style: dotted;
  cursor: pointer;
}

.monster-panel :deep(.dice-roll),
.monster-panel :deep(.damage-roll) {
  font-family: monospace;
  font-weight: 700;
  color: var(--color-dnd-damage, #ff6b6b);
}

.monster-panel :deep(.hit-bonus) {
  font-weight: 700;
  color: var(--color-success, #34d399);
}

/* Monster Footer */
.monster-footer {
  padding: 0.5rem 0.75rem;
  border-top: 1px solid var(--color-border);
  background: var(--color-base-200);
}

.source-tag {
  font-size: 0.7rem;
  color: var(--color-text-muted);
  font-style: italic;
}
</style>
