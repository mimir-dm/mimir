<template>
  <div class="campaign-maps">
    <div class="section-header">
      <h3 class="section-title">Campaign Maps</h3>
      <button class="btn-secondary btn-sm" @click="showUploadModal = true">
        + Upload Map
      </button>
    </div>

    <div v-if="loading" class="loading-state">
      Loading maps...
    </div>

    <EmptyState
      v-else-if="maps.length === 0"
      variant="campaigns"
      title="No maps in this campaign yet"
      description="Upload battle maps, dungeon layouts, or regional maps to use with the player display."
    />

    <div v-else class="map-grid">
      <div
        v-for="map in maps"
        :key="map.id"
        class="map-card"
        @click="selectMap(map)"
      >
        <div class="map-thumbnail">
          <img
            v-if="mapThumbnails[map.id]"
            :src="mapThumbnails[map.id]"
            :alt="map.name"
            class="thumbnail-image"
          />
          <div v-else class="thumbnail-placeholder">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="placeholder-icon">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 6.75V15m6-6v8.25m.503 3.498l4.875-2.437c.381-.19.622-.58.622-1.006V4.82c0-.836-.88-1.38-1.628-1.006l-3.869 1.934c-.317.159-.69.159-1.006 0L9.503 3.252a1.125 1.125 0 00-1.006 0L3.622 5.689C3.24 5.88 3 6.27 3 6.695V19.18c0 .836.88 1.38 1.628 1.006l3.869-1.934c.317-.159.69-.159 1.006 0l4.994 2.497c.317.158.69.158 1.006 0z" />
            </svg>
          </div>
        </div>
        <div class="map-info">
          <h4 class="map-name">{{ map.name }}</h4>
          <div class="map-details">
            <span class="map-size">{{ map.width_px }}x{{ map.height_px }}</span>
            <span v-if="map.grid_type !== 'none'" class="map-grid-type">
              {{ map.grid_type }} grid
            </span>
          </div>
        </div>
        <div class="map-actions">
          <button
            class="action-btn"
            title="Configure Grid"
            @click.stop="configureGrid(map)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6A2.25 2.25 0 016 3.75h2.25A2.25 2.25 0 0110.5 6v2.25a2.25 2.25 0 01-2.25 2.25H6a2.25 2.25 0 01-2.25-2.25V6zM3.75 15.75A2.25 2.25 0 016 13.5h2.25a2.25 2.25 0 012.25 2.25V18a2.25 2.25 0 01-2.25 2.25H6A2.25 2.25 0 013.75 18v-2.25zM13.5 6a2.25 2.25 0 012.25-2.25H18A2.25 2.25 0 0120.25 6v2.25A2.25 2.25 0 0118 10.5h-2.25a2.25 2.25 0 01-2.25-2.25V6zM13.5 15.75a2.25 2.25 0 012.25-2.25H18a2.25 2.25 0 012.25 2.25V18A2.25 2.25 0 0118 20.25h-2.25A2.25 2.25 0 0113.5 18v-2.25z" />
            </svg>
          </button>
          <button
            class="action-btn action-btn-danger"
            title="Delete Map"
            @click.stop="confirmDeleteMap(map)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Upload Modal -->
    <MapUploadModal
      :visible="showUploadModal"
      :campaign-id="campaignId"
      @close="showUploadModal = false"
      @uploaded="handleMapUploaded"
    />

    <!-- Grid Config Modal -->
    <MapGridConfigModal
      v-if="selectedMapForGrid"
      :visible="showGridConfigModal"
      :map="selectedMapForGrid"
      @close="closeGridConfig"
      @saved="handleGridConfigSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import MapUploadModal from './MapUploadModal.vue'
import MapGridConfigModal from './MapGridConfigModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'

interface Map {
  id: number
  campaign_id: number
  module_id: number | null
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

const props = defineProps<{
  campaignId: number
}>()

const emit = defineEmits<{
  selectMap: [map: Map]
}>()

const loading = ref(false)
const maps = ref<Map[]>([])
const mapThumbnails = ref<Record<number, string>>({})
const showUploadModal = ref(false)
const showGridConfigModal = ref(false)
const selectedMapForGrid = ref<Map | null>(null)

// Load campaign maps (campaign-level only, not module maps)
async function loadMaps() {
  loading.value = true
  try {
    const response = await invoke<{ success: boolean; data?: Map[]; error?: string }>('list_maps', {
      request: { campaign_id: props.campaignId, module_id: null }
    })

    if (response.success && response.data) {
      maps.value = response.data
      // Load thumbnails for each map
      for (const map of maps.value) {
        loadMapThumbnail(map.id)
      }
    }
  } catch (e) {
    console.error('Failed to load maps:', e)
  } finally {
    loading.value = false
  }
}

// Load a map thumbnail
async function loadMapThumbnail(mapId: number) {
  try {
    const response = await invoke<{ success: boolean; data?: string }>('serve_map_image', {
      id: mapId
    })

    if (response.success && response.data) {
      mapThumbnails.value[mapId] = response.data
    }
  } catch (e) {
    console.error(`Failed to load thumbnail for map ${mapId}:`, e)
  }
}

function selectMap(map: Map) {
  emit('selectMap', map)
}

function configureGrid(map: Map) {
  selectedMapForGrid.value = map
  showGridConfigModal.value = true
}

function closeGridConfig() {
  showGridConfigModal.value = false
  selectedMapForGrid.value = null
}

function handleGridConfigSaved() {
  closeGridConfig()
  loadMaps()
}

async function confirmDeleteMap(map: Map) {
  if (!confirm(`Delete map "${map.name}"? This cannot be undone.`)) {
    return
  }

  try {
    const response = await invoke<{ success: boolean; error?: string }>('delete_map', {
      id: map.id
    })

    if (response.success) {
      loadMaps()
    } else {
      alert(`Failed to delete map: ${response.error}`)
    }
  } catch (e) {
    console.error('Failed to delete map:', e)
    alert('Failed to delete map')
  }
}

function handleMapUploaded() {
  showUploadModal.value = false
  loadMaps()
}

watch(() => props.campaignId, () => {
  loadMaps()
})

onMounted(() => {
  loadMaps()
})
</script>

<style scoped>
.campaign-maps {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.section-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.btn-secondary {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  font-weight: 500;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: var(--color-surface);
  border-color: var(--color-primary-500);
}

.btn-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
}

.loading-state {
  padding: var(--spacing-xl);
  text-align: center;
  color: var(--color-text-muted);
}

.map-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.map-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.map-card:hover {
  border-color: var(--color-primary-500);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.map-thumbnail {
  aspect-ratio: 16/10;
  background: var(--color-base-200);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.thumbnail-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumbnail-placeholder {
  color: var(--color-text-muted);
  opacity: 0.5;
}

.placeholder-icon {
  width: 48px;
  height: 48px;
}

.map-info {
  padding: var(--spacing-sm);
}

.map-name {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--spacing-xs) 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.map-details {
  display: flex;
  gap: var(--spacing-sm);
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.map-grid-type {
  text-transform: capitalize;
}

.map-actions {
  display: flex;
  gap: var(--spacing-xs);
  padding: 0 var(--spacing-sm) var(--spacing-sm);
}

.action-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background: var(--color-base-200);
  color: var(--color-text);
}

.action-btn-danger:hover {
  background: var(--color-error-100);
  border-color: var(--color-error);
  color: var(--color-error);
}

.action-btn svg {
  width: 16px;
  height: 16px;
}
</style>
