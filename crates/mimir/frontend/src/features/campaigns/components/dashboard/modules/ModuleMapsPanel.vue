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
        <div class="map-card-content">
          <span class="map-name">{{ map.name }}</span>
          <span class="map-size">{{ map.width_px }}x{{ map.height_px }}</span>
        </div>
        <button
          class="btn-delete"
          title="Delete Map"
          @click.stop="$emit('delete', map)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" width="14" height="14">
            <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
          </svg>
        </button>
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
  delete: [map: MapData]
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
  align-items: center;
  gap: var(--spacing-xs);
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

.map-card-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
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

.btn-delete {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
}

.map-card:hover .btn-delete {
  opacity: 1;
}

.btn-delete:hover {
  background: var(--color-error, #ef4444);
  color: white;
}
</style>
