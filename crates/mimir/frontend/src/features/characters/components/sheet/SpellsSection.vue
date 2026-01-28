<template>
  <div class="sheet-content single-column">
    <!-- Spellcasting Stats -->
    <section class="sheet-section">
      <h2>Spellcasting</h2>
      <!-- Multiclass: show stats for each spellcasting class -->
      <template v-if="isMulticlassSpellcaster">
        <div
          v-for="stats in allSpellcastingStats"
          :key="stats.className"
          class="spell-stats-row multiclass"
        >
          <div class="spell-class-label">{{ stats.className }}</div>
          <div class="spell-stat-box">
            <span class="stat-label">Spell Save DC</span>
            <span class="stat-value large">{{ stats.saveDC }}</span>
          </div>
          <div class="spell-stat-box">
            <span class="stat-label">Spell Attack</span>
            <span class="stat-value large">{{ formatMod(stats.attackBonus) }}</span>
          </div>
          <div class="spell-stat-box">
            <span class="stat-label">Ability</span>
            <span class="stat-value large">{{ stats.abilityAbbrev }}</span>
          </div>
        </div>
      </template>
      <!-- Single class: show simple row -->
      <div v-else class="spell-stats-row">
        <div class="spell-stat-box">
          <span class="stat-label">Spell Save DC</span>
          <span class="stat-value large">{{ spellSaveDC }}</span>
        </div>
        <div class="spell-stat-box">
          <span class="stat-label">Spell Attack</span>
          <span class="stat-value large">{{ formatMod(spellAttackBonus || 0) }}</span>
        </div>
        <div class="spell-stat-box">
          <span class="stat-label">Spellcasting Ability</span>
          <span class="stat-value large">{{ spellcastingAbility?.toUpperCase().slice(0, 3) }}</span>
        </div>
      </div>
    </section>

    <!-- Spell Slots -->
    <section v-if="spellSlots" class="sheet-section">
      <h2>Spell Slots</h2>
      <p class="spell-info-note">
        Maximum slots shown below. Track used slots on paper.
      </p>
      <div class="spell-slots-grid">
        <div class="spell-slot-row cantrip-row">
          <span class="slot-level">Cantrips</span>
          <span class="slot-unlimited">Unlimited</span>
        </div>
        <div v-for="level in 9" :key="level" class="spell-slot-row">
          <template v-if="spellSlots[level]">
            <span class="slot-level">Level {{ level }}</span>
            <div class="slot-boxes">
              <span
                v-for="n in spellSlots[level]"
                :key="n"
                class="slot-box"
              ></span>
            </div>
            <span class="slot-count">{{ spellSlots[level] }} slots</span>
          </template>
        </div>
      </div>
    </section>

    <!-- Available Spells -->
    <section class="sheet-section">
      <h2>Available Spells</h2>
      <p class="spell-info-note">
        These spells are available to your class. Track prepared/known spells on paper.
      </p>

      <div v-if="loadingSpells" class="loading-state">Loading spells...</div>

      <div v-else-if="classSpells.length === 0" class="empty-state">
        No spells available for your class configuration.
      </div>

      <div v-else class="spell-list-container">
        <div v-for="level in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]" :key="level">
          <div v-if="spellsByLevel[level]?.length" class="spell-level-group">
            <h3
              class="spell-level-header collapsible"
              :class="{ collapsed: isSpellLevelCollapsed(level) }"
              @click="toggleSpellLevel(level)"
            >
              <span class="collapse-icon">{{ isSpellLevelCollapsed(level) ? '▶' : '▼' }}</span>
              {{ getLevelDisplay(level) }}
              <span class="spell-count">({{ spellsByLevel[level].length }})</span>
            </h3>
            <div v-show="!isSpellLevelCollapsed(level)" class="spell-cards">
              <div
                v-for="spell in spellsByLevel[level]"
                :key="`${spell.name}|${spell.source}`"
                class="spell-card"
                :class="{ expanded: isSpellExpanded(spell.name, spell.source) }"
              >
                <div
                  class="spell-card-header"
                  @click="toggleSpellDetails(spell.name, spell.source)"
                >
                  <span class="spell-name">
                    {{ spell.name }}
                    <span v-if="spell.ritual" class="spell-tag ritual">R</span>
                    <span v-if="spell.concentration" class="spell-tag conc">C</span>
                  </span>
                  <span class="spell-meta">
                    <span class="spell-school">{{ getSchoolName(spell.school) }}</span>
                    <span class="expand-icon">{{ isSpellExpanded(spell.name, spell.source) ? '−' : '+' }}</span>
                  </span>
                </div>
                <div
                  v-if="isSpellExpanded(spell.name, spell.source)"
                  class="spell-card-details"
                >
                  <div class="spell-stats-mini">
                    <div class="spell-stat-mini">
                      <span class="label">Casting Time:</span>
                      <span>{{ getSpellCastingTime(spell) }}</span>
                    </div>
                    <div class="spell-stat-mini">
                      <span class="label">Range:</span>
                      <span>{{ getSpellRange(spell) }}</span>
                    </div>
                    <div class="spell-stat-mini">
                      <span class="label">Components:</span>
                      <span>{{ getSpellComponents(spell) }}</span>
                    </div>
                    <div class="spell-stat-mini">
                      <span class="label">Duration:</span>
                      <span>{{ getSpellDuration(spell) }}</span>
                    </div>
                  </div>
                  <div class="spell-description">
                    {{ getSpellDescription(spell) }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, toRef } from 'vue'
import type { Character } from '@/types/character'
import { useSpellManagement } from '../../composables/useSpellManagement'

const props = defineProps<{
  character: Character
  formatMod: (mod: number) => string
}>()

// Create refs for the composable
const characterRef = toRef(props, 'character')
const characterId = computed(() => props.character.id.toString())

// Use the spell management composable
const {
  classSpells,
  loadingSpells,
  spellcastingAbility,
  spellSaveDC,
  spellAttackBonus,
  allSpellcastingStats,
  isMulticlassSpellcaster,
  spellSlots,
  spellsByLevel,
  getSchoolName,
  getLevelDisplay,
  toggleSpellDetails,
  toggleSpellLevel,
  isSpellLevelCollapsed,
  isSpellExpanded,
  getSpellCastingTime,
  getSpellRange,
  getSpellComponents,
  getSpellDuration,
  getSpellDescription,
  loadClassSpells,
} = useSpellManagement(characterRef, characterId)

// Load spells when component mounts
loadClassSpells()
</script>

<style scoped>
.sheet-content.single-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  max-width: 700px;
}

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

.loading-state,
.empty-state {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Spell Stats */
.spell-stats-row {
  display: flex;
  gap: var(--spacing-lg);
  justify-content: center;
}

.spell-stats-row.multiclass {
  justify-content: flex-start;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  margin-bottom: var(--spacing-sm);
}

.spell-stats-row.multiclass .spell-stat-box {
  background: var(--color-surface);
  min-width: 80px;
  padding: var(--spacing-sm) var(--spacing-md);
}

.spell-stats-row.multiclass .spell-stat-box .stat-value.large {
  font-size: 1.25rem;
}

.spell-class-label {
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--color-primary-500);
  min-width: 80px;
  display: flex;
  align-items: center;
}

.spell-stat-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  min-width: 100px;
}

.spell-stat-box .stat-label {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.spell-stat-box .stat-value.large {
  font-size: 1.75rem;
  font-weight: 700;
}

/* Spell Slots */
.spell-info-note {
  color: var(--color-text-secondary);
  font-style: italic;
  margin-bottom: var(--spacing-md);
}

.spell-slots-grid {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.spell-slot-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) 0;
}

.slot-level {
  font-weight: 600;
  min-width: 70px;
  color: var(--color-text-secondary);
}

.cantrip-row {
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-xs);
  padding-bottom: var(--spacing-sm);
}

.slot-unlimited {
  font-size: 0.85rem;
  font-style: italic;
  color: var(--color-text-tertiary);
}

.slot-boxes {
  display: flex;
  gap: var(--spacing-xs);
}

.slot-box {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-primary-500);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
}

.slot-count {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
}

/* Spell List */
.spell-list-container {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.spell-level-group {
  margin-bottom: var(--spacing-md);
}

.spell-level-header {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-primary-600);
  margin-bottom: var(--spacing-sm);
  padding-bottom: var(--spacing-xs);
  border-bottom: 2px solid var(--color-primary-200);
}

.spell-level-header.collapsible {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  user-select: none;
  transition: color 0.15s ease;
}

.spell-level-header.collapsible:hover {
  color: var(--color-primary-700);
}

.spell-level-header .collapse-icon {
  font-size: 0.75rem;
  width: 1rem;
  text-align: center;
  transition: transform 0.15s ease;
}

.spell-level-header .spell-count {
  font-weight: 400;
  font-size: 0.875rem;
  color: var(--color-text-muted);
  margin-left: auto;
}

.spell-level-header.collapsed {
  margin-bottom: 0;
  border-bottom-color: var(--color-border);
}

.spell-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.spell-card {
  background: var(--color-surface-variant);
  border: 1px solid #ccc;
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: all 0.2s ease;
}

.spell-card.expanded {
  border-color: var(--color-primary-300);
}

.spell-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background 0.15s ease;
}

.spell-card-header:hover {
  background: var(--color-surface-hover);
}

.spell-card-header .spell-name {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.spell-tag {
  font-size: 0.65rem;
  font-weight: 600;
  padding: 1px 4px;
  border-radius: 3px;
  text-transform: uppercase;
}

.spell-tag.ritual {
  background: var(--color-success-100);
  color: var(--color-success-700);
}

.spell-tag.conc {
  background: var(--color-warning-100);
  color: var(--color-warning-700);
}

.spell-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 0.85rem;
}

.spell-school {
  color: var(--color-text-secondary);
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

.spell-card-details {
  padding: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
  font-size: 0.9rem;
}

.spell-stats-mini {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-xs) var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.spell-stat-mini {
  display: flex;
  gap: var(--spacing-xs);
}

.spell-stat-mini .label {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.spell-description {
  line-height: 1.5;
  white-space: pre-wrap;
  color: var(--color-text);
}
</style>
