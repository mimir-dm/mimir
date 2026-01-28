<template>
  <div class="world-tab">
    <!-- Sidebar -->
    <div class="sidebar-panel">
      <DocumentSidebar
        v-if="campaign"
        :campaign-id="campaign.id"
        :campaign-name="campaign.name"
        @select-document="handleSelectDocument"
        @select-asset="handleSelectAsset"
      />

      <!-- Maps List -->
      <div v-if="campaign" class="maps-section">
        <div class="maps-header">
          <h4>Maps</h4>
          <button class="btn-add" @click="showUploadModal = true" title="Upload Map">+</button>
        </div>
        <div v-if="loadingMaps" class="maps-loading">Loading...</div>
        <div v-else-if="maps.length === 0" class="maps-empty">No maps yet</div>
        <div v-else class="maps-list">
          <div
            v-for="map in maps"
            :key="map.id"
            class="map-item"
            :class="{ selected: selectedMap?.id === map.id }"
            @click="selectMap(map)"
          >
            {{ map.name }}
          </div>
        </div>
      </div>
    </div>

    <!-- Main Panel -->
    <div class="editor-panel">
      <!-- Document Editor -->
      <DocumentEditor
        v-if="selectedDocument"
        :document="selectedDocument"
        :campaign-id="campaign?.id || ''"
        @close="selectedDocument = null"
        @updated="handleDocumentUpdated"
        @stage-transitioned="handleStageTransitioned"
      />

      <!-- Map Preview -->
      <div v-else-if="selectedMap" class="map-preview">
        <div class="map-preview-header">
          <h3>{{ selectedMap.name }}</h3>
          <div class="map-actions">
            <button class="btn-action" @click="printMap" title="Export PDF">
              PDF
            </button>
            <button class="btn-action btn-danger" @click="deleteMap" title="Delete Map">
              Delete
            </button>
            <button class="btn-close" @click="selectedMap = null">Close</button>
          </div>
        </div>
        <div class="map-preview-content">
          <img v-if="mapImageUrl" :src="mapImageUrl" :alt="selectedMap.name" class="map-image" />
          <div v-else class="map-loading">Loading map...</div>
        </div>
        <div class="map-info">
          <span>{{ selectedMap.width_px }} x {{ selectedMap.height_px }}px</span>
          <span v-if="selectedMap.grid_type !== 'none'">{{ selectedMap.grid_type }} grid</span>
        </div>
      </div>

      <!-- Asset Preview -->
      <div v-else-if="selectedAsset" class="asset-preview">
        <div class="asset-preview-header">
          <h3>{{ selectedAsset.filename }}</h3>
          <button class="btn-close" @click="selectedAsset = null; assetImageUrl = null">Close</button>
        </div>
        <div class="asset-preview-content">
          <img v-if="assetImageUrl" :src="assetImageUrl" :alt="selectedAsset.filename" class="asset-image" />
          <div v-else class="asset-loading">Loading image...</div>
        </div>
        <div class="asset-info">
          <span>{{ selectedAsset.mime_type }}</span>
          <span v-if="selectedAsset.file_size">{{ Math.round(selectedAsset.file_size / 1024) }} KB</span>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="empty-state">
        <div class="empty-icon">📄</div>
        <h3>No Document Selected</h3>
        <p>Select a document or map from the sidebar.</p>
      </div>
    </div>

    <!-- Upload Modal -->
    <MapUploadModal
      :visible="showUploadModal"
      :campaign-id="campaign?.id || ''"
      @close="showUploadModal = false"
      @uploaded="handleMapUploaded"
    />

    <!-- Print Dialog -->
    <MapPrintDialog
      v-if="selectedMap && showPrintDialog"
      :visible="showPrintDialog"
      :map-id="selectedMap.id"
      :map-name="selectedMap.name"
      :map-dimensions="{ width: selectedMap.width_px, height: selectedMap.height_px }"
      :grid-size-px="selectedMap.grid_size_px || 70"
      @close="showPrintDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DocumentSidebar from '../DocumentSidebar.vue'
import DocumentEditor from '../DocumentEditor.vue'
import MapUploadModal from '../StageLanding/MapUploadModal.vue'
import MapPrintDialog from '@/components/print/MapPrintDialog.vue'
import type { Campaign } from '@/types'

interface MapData {
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
  campaign?: Campaign
  documents?: any[]
}>()

const emit = defineEmits<{
  refresh: []
}>()

// Helper to derive status from campaign data
function getCampaignStatus(campaign: Campaign): string {
  if (campaign.archived_at) return 'archived'
  return 'active'
}

// Document state
const selectedDocument = ref<any>(null)

// Asset state
interface CampaignAsset {
  id: string
  campaign_id: string | null
  module_id: string | null
  filename: string
  description: string | null
  mime_type: string
  blob_path: string
  file_size: number | null
  uploaded_at: string
}
const selectedAsset = ref<CampaignAsset | null>(null)
const assetImageUrl = ref<string | null>(null)

// Map state
const maps = ref<MapData[]>([])
const selectedMap = ref<MapData | null>(null)
const mapImageUrl = ref<string | null>(null)
const loadingMaps = ref(false)
const showUploadModal = ref(false)
const showPrintDialog = ref(false)

// Load campaign maps
async function loadMaps() {
  if (!props.campaign) return
  loadingMaps.value = true
  try {
    const response = await invoke<{ success: boolean; data?: MapData[] }>('list_campaign_maps', {
      campaignId: props.campaign.id
    })
    if (response.success && response.data) {
      maps.value = response.data
    }
  } catch (e) {
    console.error('Failed to load maps:', e)
  } finally {
    loadingMaps.value = false
  }
}

// Load map image
async function loadMapImage(mapId: string) {
  try {
    const response = await invoke<{ success: boolean; data?: string }>('read_map_uvtt', { mapId })
    if (response.success && response.data) {
      // The response is base64-encoded UVTT data - need to parse and extract image
      try {
        const uvttJson = JSON.parse(atob(response.data))
        let imageData = uvttJson.image || ''
        if (!imageData.startsWith('data:')) {
          imageData = `data:image/png;base64,${imageData}`
        }
        mapImageUrl.value = imageData
      } catch {
        // If not valid JSON, assume it's raw image data
        mapImageUrl.value = `data:image/png;base64,${response.data}`
      }
    }
  } catch (e) {
    console.error('Failed to load map image:', e)
  }
}

// Select a map
function selectMap(map: MapData) {
  selectedDocument.value = null
  selectedAsset.value = null
  assetImageUrl.value = null
  selectedMap.value = map
  mapImageUrl.value = null
  loadMapImage(map.id)
}

// Asset handlers
async function handleSelectAsset(asset: CampaignAsset) {
  selectedDocument.value = null
  selectedMap.value = null
  mapImageUrl.value = null
  selectedAsset.value = asset
  assetImageUrl.value = null

  // Load asset image
  try {
    const response = await invoke<{ success: boolean; data?: string }>('read_asset_file', { id: asset.id })
    if (response.success && response.data) {
      assetImageUrl.value = `data:${asset.mime_type};base64,${response.data}`
    }
  } catch (e) {
    console.error('Failed to load asset:', e)
  }
}

// Document handlers
function handleSelectDocument(document: any) {
  selectedMap.value = null
  mapImageUrl.value = null
  selectedAsset.value = null
  assetImageUrl.value = null
  selectedDocument.value = document
}

function handleCreateDocument() {
  console.warn('Document creation not yet implemented')
}

function handleDocumentCompletionChanged() {
  emit('refresh')
}

function handleDocumentUpdated(document: any) {
  selectedDocument.value = document
}

function handleStageTransitioned() {
  emit('refresh')
}

// Map handlers
function handleMapUploaded() {
  showUploadModal.value = false
  loadMaps()
}

function printMap() {
  showPrintDialog.value = true
}

async function deleteMap() {
  if (!selectedMap.value) return
  if (!confirm(`Delete map "${selectedMap.value.name}"?`)) return

  try {
    const response = await invoke<{ success: boolean }>('delete_map', { id: selectedMap.value.id })
    if (response.success) {
      selectedMap.value = null
      mapImageUrl.value = null
      loadMaps()
    }
  } catch (e) {
    console.error('Failed to delete map:', e)
  }
}

watch(() => props.campaign?.id, () => {
  loadMaps()
})

onMounted(() => {
  loadMaps()
})
</script>

<style scoped>
.world-tab {
  display: flex;
  height: 100%;
  overflow: hidden;
  align-items: stretch;
}

.sidebar-panel {
  width: 320px;
  min-width: 280px;
  max-width: 400px;
  border-right: 1px solid var(--color-border, #333);
  overflow-y: auto;
  background: var(--color-surface, #1a1a1a);
}

.sidebar-panel :deep(.document-sidebar) {
  height: auto !important;
  min-height: 0 !important;
  flex: none !important;
}

.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Maps section */
.maps-section {
  border-top: 1px solid var(--color-border, #333);
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
}

.maps-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px) 0;
  margin-bottom: 0;
}

.maps-header h4 {
  margin: 0;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--color-text-muted, #666);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.btn-add {
  width: 20px;
  height: 20px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-surface);
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}

.btn-add:hover {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.maps-loading,
.maps-empty {
  padding: var(--spacing-sm, 8px);
  font-size: 0.8rem;
  color: var(--color-text-muted);
}

.maps-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.map-item {
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
  border-radius: 4px;
}

.map-item:hover {
  background: var(--color-surface-variant, #252525);
}

.map-item.selected {
  background: var(--color-primary-900, #1e3a5f);
}

/* Map preview */
.map-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.map-preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.map-preview-header h3 {
  margin: 0;
  font-size: 1.125rem;
}

.map-actions {
  display: flex;
  gap: var(--spacing-sm, 8px);
}

.btn-action {
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  font-size: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
}

.btn-action:hover {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.btn-action.btn-danger:hover {
  background: var(--color-error);
  border-color: var(--color-error);
}

.btn-close {
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  font-size: 0.75rem;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
}

.btn-close:hover {
  color: var(--color-text);
}

.map-preview-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: auto;
  padding: var(--spacing-md);
  background: var(--color-background);
}

.map-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.map-loading {
  color: var(--color-text-muted);
}

.map-info {
  display: flex;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-lg);
  font-size: 0.75rem;
  color: var(--color-text-muted);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

/* Asset preview */
.asset-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.asset-preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.asset-preview-header h3 {
  margin: 0;
  font-size: 1.125rem;
}

.asset-preview-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: auto;
  padding: var(--spacing-md);
  background: var(--color-background);
}

.asset-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.asset-loading {
  color: var(--color-text-muted);
}

.asset-info {
  display: flex;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-lg);
  font-size: 0.75rem;
  color: var(--color-text-muted);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

/* Empty state */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  padding: var(--spacing-xl, 24px);
  color: var(--color-text-muted, #888);
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: var(--spacing-md, 12px);
  opacity: 0.5;
}

.empty-state h3 {
  margin: 0 0 var(--spacing-sm, 8px) 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.empty-state p {
  margin: 0;
  font-size: 0.875rem;
}
</style>
