<template>
  <div class="poi-panel poi-panel-content">
      <!-- POI Header -->
      <header class="poi-header">
        <div class="poi-title">
          <span class="poi-icon" :style="{ backgroundColor: poi.color || '#3b82f6' }">
            {{ getPoiIcon(poi.icon) }}
          </span>
          <h2>{{ poi.name }}</h2>
        </div>
      </header>

      <div class="poi-body">
        <!-- Description -->
        <div v-if="poi.description" class="poi-description">
          <p>{{ poi.description }}</p>
        </div>

        <!-- Empty state -->
        <div v-else class="poi-empty">
          <p>No description added for this point of interest.</p>
        </div>

        <!-- Location info -->
        <div class="poi-location">
          <span class="location-label">Location:</span>
          <span class="location-value">Grid {{ poi.grid_x }}, {{ poi.grid_y }}</span>
        </div>

        <!-- Visibility status -->
        <div class="poi-visibility">
          <span class="visibility-label">Visibility:</span>
          <span class="visibility-value" :class="poi.visible === 1 ? 'visible' : 'hidden'">
            {{ poi.visible === 1 ? 'Visible to players' : 'Hidden from players' }}
          </span>
        </div>
      </div>

      <!-- Footer with count -->
      <footer class="poi-footer" v-if="poi.count > 1">
        <span class="count-tag">{{ poi.count }} instances on maps</span>
      </footer>
  </div>
</template>

<script setup lang="ts">
import type { ModulePoi } from '../types'

const props = defineProps<{
  poi: ModulePoi
}>()

// Map icon names to emoji/symbols (matches PoiEditModal icons)
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

.poi-panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* POI Header */
.poi-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--color-surface);
  border-bottom: 2px solid var(--color-primary-500);
}

.poi-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.poi-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  color: white;
  flex-shrink: 0;
}

.poi-title h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--color-text);
  line-height: 1.2;
}

/* POI Body */
.poi-body {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.poi-description p {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.6;
  color: var(--color-text);
}

.poi-empty p {
  margin: 0;
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Location info */
.poi-location,
.poi-visibility {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
}

.location-label,
.visibility-label {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.location-value {
  color: var(--color-text);
  font-family: monospace;
}

.visibility-value {
  font-weight: 500;
}

.visibility-value.visible {
  color: var(--color-success);
}

.visibility-value.hidden {
  color: var(--color-text-secondary);
}

/* POI Footer */
.poi-footer {
  padding: 0.5rem 0.75rem;
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.count-tag {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  font-style: italic;
}
</style>
