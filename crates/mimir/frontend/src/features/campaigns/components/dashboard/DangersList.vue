<template>
  <section class="dashboard-section dangers-section">
    <div class="section-header">
      <h3>Dangers</h3>
    </div>
    <div v-if="loading" class="section-loading">Loading...</div>
    <div v-else-if="isEmpty" class="section-empty">
      No dangers or points of interest added
    </div>
    <div v-else class="dangers-list">
      <!-- Monsters Section -->
      <div v-if="monsters.length > 0" class="danger-category">
        <div class="danger-category-header">Monsters</div>
        <!-- Grouped by encounter tag -->
        <div
          v-for="group in encounterGroups"
          :key="group.encounter_tag || 'untagged'"
          class="monster-group"
        >
          <div v-if="group.encounter_tag" class="monster-group-header">
            {{ group.encounter_tag }}
          </div>
          <div v-else-if="encounterGroups.length > 1" class="monster-group-header untagged">
            Other
          </div>
          <div class="monster-group-items">
            <div
              v-for="monster in group.monsters"
              :key="monster.id"
              class="monster-row"
              :class="{ active: selectedMonsterId === monster.id }"
              @click="$emit('select-monster', monster)"
            >
              <span class="monster-qty">{{ monster.quantity }}×</span>
              <div class="monster-info">
                <span class="monster-name">{{ monster.display_name || monster.monster_name }}</span>
                <span v-if="monster.display_name" class="monster-original">({{ monster.monster_name }})</span>
                <span v-if="monster.notes" class="monster-has-notes" title="Has DM notes">*</span>
              </div>
              <button
                v-if="monster.id"
                class="monster-edit-btn"
                @click.stop="$emit('edit-monster', monster)"
                title="Customize monster"
              >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Traps/Hazards Section -->
      <div v-if="traps.length > 0" class="danger-category">
        <div class="danger-category-header">Traps & Hazards</div>
        <div class="trap-list">
          <div
            v-for="trap in traps"
            :key="trap.name"
            class="trap-row"
            :class="{ active: selectedTrapName === trap.name }"
            @click="$emit('select-trap', trap)"
          >
            <span class="trap-qty" v-if="trap.count > 1">{{ trap.count }}×</span>
            <span class="trap-name">{{ trap.name }}</span>
          </div>
        </div>
      </div>

      <!-- Points of Interest Section -->
      <div v-if="pois.length > 0" class="danger-category">
        <div class="danger-category-header">Points of Interest</div>
        <div class="poi-list">
          <div
            v-for="poi in pois"
            :key="poi.name"
            class="poi-row"
            :class="{ active: selectedPoiName === poi.name }"
            @click="$emit('select-poi', poi)"
          >
            <span class="poi-icon-small" :style="{ backgroundColor: poi.color || '#3b82f6' }">
              {{ getPoiIcon(poi.icon) }}
            </span>
            <span class="poi-name">{{ poi.name }}</span>
            <span class="poi-qty" v-if="poi.count > 1">{{ poi.count }}×</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'

/**
 * Module trap data
 */
export interface ModuleTrap {
  id: string
  name: string
  source: string
  count: number
}

/**
 * Module POI data
 */
export interface ModulePoi {
  id: string
  name: string
  description: string | null
  icon: string
  color: string | null
  visible: number
  grid_x: number
  grid_y: number
  count: number
}

/**
 * Encounter group with monsters
 */
export interface EncounterGroup {
  encounter_tag: string | null
  monsters: Array<{
    id: string
    quantity: number
    display_name: string | null
    monster_name: string
    notes: string | null
  }>
}

const props = defineProps<{
  monsters: Array<any>
  encounterGroups: EncounterGroup[]
  traps: ModuleTrap[]
  pois: ModulePoi[]
  selectedMonsterId?: string | null
  selectedTrapName?: string | null
  selectedPoiName?: string | null
  loadingMonsters?: boolean
  loadingTraps?: boolean
  loadingPois?: boolean
}>()

defineEmits<{
  (e: 'select-monster', monster: any): void
  (e: 'edit-monster', monster: any): void
  (e: 'select-trap', trap: ModuleTrap): void
  (e: 'select-poi', poi: ModulePoi): void
}>()

const loading = computed(() => {
  return props.loadingMonsters || props.loadingTraps || props.loadingPois
})

const isEmpty = computed(() => {
  return props.monsters.length === 0 && props.traps.length === 0 && props.pois.length === 0
})

/**
 * Get emoji icon for POI type
 */
function getPoiIcon(iconName: string): string {
  const iconMap: Record<string, string> = {
    'pin': '📍',
    'star': '⭐',
    'skull': '💀',
    'chest': '📦',
    'door': '🚪',
    'secret': '🔮',
    'question': '❓',
    'exclamation': '❗'
  }
  return iconMap[iconName] || '📍'
}
</script>

<style scoped>
/* Dangers Section */
.dangers-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--spacing-sm);
}

.section-header h3 {
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-secondary);
  margin: 0;
}

.section-loading,
.section-empty {
  font-size: 0.85rem;
  color: var(--color-text-muted);
  padding: var(--spacing-md);
  text-align: center;
}

.dangers-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  overflow-y: auto;
  flex: 1;
}

.danger-category {
  display: flex;
  flex-direction: column;
}

.danger-category-header {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-text-secondary);
  padding: var(--spacing-xs) 0;
  margin-bottom: var(--spacing-xs);
}

/* Monster Groups */
.monster-group {
  display: flex;
  flex-direction: column;
}

.monster-group-header {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-error);
  padding: var(--spacing-xs) 0;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-xs);
}

.monster-group-header.untagged {
  color: var(--color-text-secondary);
}

.monster-group-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.monster-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.monster-row:hover {
  background: var(--color-primary-100);
}

.monster-row.active {
  background: var(--color-primary-100);
  border-left: 3px solid var(--color-error);
  padding-left: calc(var(--spacing-sm) - 3px);
}

.monster-qty {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--color-error);
  min-width: 24px;
}

.monster-info {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-xs);
  flex: 1;
  min-width: 0;
}

.monster-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text);
}

.monster-original {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

.monster-has-notes {
  font-size: 0.8rem;
  color: var(--color-primary-500);
  font-weight: bold;
}

.monster-edit-btn {
  padding: 2px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.monster-edit-btn svg {
  width: 14px;
  height: 14px;
  color: var(--color-text-secondary);
}

.monster-edit-btn:hover svg {
  color: var(--color-primary-500);
}

.monster-row:hover .monster-edit-btn {
  opacity: 1;
}

/* Trap List */
.trap-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.trap-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.trap-row:hover {
  background: var(--color-primary-100);
}

.trap-row.active {
  background: var(--color-primary-100);
  border-left: 3px solid var(--color-warning);
  padding-left: calc(var(--spacing-sm) - 3px);
}

.trap-qty {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--color-warning);
  min-width: 24px;
}

.trap-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text);
}

/* POI List */
.poi-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.poi-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.poi-row:hover {
  background: var(--color-primary-100);
}

.poi-row.active {
  background: var(--color-primary-100);
  border-left: 3px solid var(--color-primary);
  padding-left: calc(var(--spacing-sm) - 3px);
}

.poi-icon-small {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.65rem;
  color: white;
  flex-shrink: 0;
}

.poi-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text);
  flex: 1;
}

.poi-qty {
  font-size: 0.7rem;
  color: var(--color-text-muted);
}
</style>
