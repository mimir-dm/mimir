<template>
  <section class="dashboard-section maps-section">
    <div class="section-header">
      <h3>Maps</h3>
      <button class="btn-add" @click="$emit('upload')" title="Upload Map">+</button>
    </div>
    <div v-if="loading" class="section-loading">Loading...</div>
    <div v-else-if="maps.length === 0" class="section-empty">
      No maps uploaded
    </div>
    <div v-else class="map-cards">
      <div
        v-for="map in maps"
        :key="map.id"
        class="map-card"
        @click="$emit('select', map)"
      >
        <span class="map-name">{{ map.name }}</span>
        <span class="map-size">{{ map.width_px }}x{{ map.height_px }}</span>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
export interface MapData {
  id: string
  campaign_id: string
  module_id: string | null
  name: string
  image_path: string
  width_px: number
  height_px: number
  grid_type: string
  grid_size_px: number | null
  grid_offset_x: number
  grid_offset_y: number
  original_width_px: number | null
  original_height_px: number | null
}

defineProps<{
  maps: MapData[]
  loading?: boolean
}>()

defineEmits<{
  upload: []
  select: [map: MapData]
}>()
</script>

<style scoped>
.dashboard-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-sm);
  padding-bottom: var(--spacing-xs);
  border-bottom: 1px solid var(--color-border);
}

.section-header h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
}

.btn-add {
  width: 20px;
  height: 20px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}

.btn-add:hover {
  background: var(--color-primary-500);
  color: var(--color-background);
  border-color: var(--color-primary-500);
}

.section-empty,
.section-loading {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-align: center;
  padding: var(--spacing-md);
}

.map-cards {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}

.map-card {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.map-card:hover {
  border-color: var(--color-primary-500);
}

.map-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text);
}

.map-size {
  font-size: 0.65rem;
  color: var(--color-text-secondary);
}
</style>
