<template>
  <div class="module-maps">
    <div class="section-header">
      <h3 class="section-title">Module Maps</h3>
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
      title="No maps for this module yet"
      description="Upload encounter maps, battle grids, or location maps specific to this module."
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
            <MapIcon class="placeholder-icon" :stroke-width="1.5" aria-hidden="true" />
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
            title="Place Tokens"
            @click.stop="setupTokens(map)"
          >
            <Users :stroke-width="1.5" aria-hidden="true" />
          </button>
          <button
            class="action-btn"
            title="Print Map"
            @click.stop="printMap(map)"
          >
            <Printer :stroke-width="1.5" aria-hidden="true" />
          </button>
          <button
            class="action-btn action-btn-danger"
            title="Delete Map"
            @click.stop="confirmDeleteMap(map)"
          >
            <Trash2 :stroke-width="1.5" aria-hidden="true" />
          </button>
        </div>
      </div>
    </div>

    <!-- Upload Modal -->
    <MapUploadModal
      :visible="showUploadModal"
      :campaign-id="campaignId"
      :module-id="moduleId"
      @close="showUploadModal = false"
      @uploaded="handleMapUploaded"
    />

    <!-- Token Setup Modal -->
    <MapTokenSetupModal
      v-if="selectedMapForTokens"
      :visible="showTokenSetupModal"
      :map="selectedMapForTokens"
      @close="closeTokenSetup"
    />

    <!-- Print Dialog -->
    <MapPrintDialog
      :visible="showPrintDialog"
      :map-id="selectedMapForPrint?.id ?? null"
      :map-name="selectedMapForPrint?.name"
      :map-dimensions="selectedMapForPrint ? { width: selectedMapForPrint.width_px, height: selectedMapForPrint.height_px } : undefined"
      :grid-size-px="selectedMapForPrint?.grid_size_px ?? 70"
      @close="closePrintDialog"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { Map as MapIcon, Printer, Trash2, Users } from '@lucide/vue'
import { alertDialog, confirmDialog } from '@/composables/useDialog'
import { invoke } from '@tauri-apps/api/core'
import MapUploadModal from '@/features/campaigns/components/StageLanding/MapUploadModal.vue'
import MapTokenSetupModal from '@/components/tokens/MapTokenSetupModal.vue'
import MapPrintDialog from '@/components/print/MapPrintDialog.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'

interface Map {
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

const props = defineProps<{
  moduleId: string
  campaignId: string
}>()

const emit = defineEmits<{
  selectMap: [map: Map]
}>()

const loading = ref(false)
const maps = ref<Map[]>([])
const mapThumbnails = ref<Record<string, string>>({})
const showUploadModal = ref(false)
const showTokenSetupModal = ref(false)
const selectedMapForTokens = ref<Map | null>(null)
const showPrintDialog = ref(false)
const selectedMapForPrint = ref<Map | null>(null)

// Load module maps
async function loadMaps() {
  loading.value = true
  try {
    const response = await invoke<{ success: boolean; data?: Map[]; error?: string }>('list_maps', {
      request: { campaign_id: props.campaignId, module_id: props.moduleId }
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
async function loadMapThumbnail(mapId: string) {
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

function setupTokens(map: Map) {
  selectedMapForTokens.value = map
  showTokenSetupModal.value = true
}

function closeTokenSetup() {
  showTokenSetupModal.value = false
  selectedMapForTokens.value = null
}

function printMap(map: Map) {
  selectedMapForPrint.value = map
  showPrintDialog.value = true
}

function closePrintDialog() {
  showPrintDialog.value = false
  selectedMapForPrint.value = null
}

async function confirmDeleteMap(map: Map) {
  const ok = await confirmDialog({
    title: 'Delete map?',
    message: `Delete "${map.name}"? Its tokens, lights, traps and points of interest go with it. This cannot be undone.`,
    confirmLabel: 'Delete map',
    danger: true,
  })
  if (!ok) return

  try {
    const response = await invoke<{ success: boolean; error?: string }>('delete_map', {
      id: map.id
    })

    if (response.success) {
      loadMaps()
    } else {
      await alertDialog({ title: 'Delete failed', message: `Failed to delete map: ${response.error}` })
    }
  } catch (e) {
    console.error('Failed to delete map:', e)
    await alertDialog({ title: 'Delete failed', message: 'Failed to delete map.' })
  }
}

function handleMapUploaded() {
  showUploadModal.value = false
  loadMaps()
}

watch(() => props.moduleId, () => {
  loadMaps()
})

onMounted(() => {
  loadMaps()
})
</script>

<style scoped>
.module-maps {
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
