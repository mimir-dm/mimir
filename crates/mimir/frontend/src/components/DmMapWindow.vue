<template>
  <div class="dm-map-window">
    <!-- Toolbar -->
    <header class="dm-toolbar">
      <div class="toolbar-left">
        <!-- Map Selector -->
        <div class="map-selector">
          <label for="map-select">Map:</label>
          <select
            id="map-select"
            v-model="activeMapId"
            class="map-select"
            @change="onMapSelected"
          >
            <option value="" disabled>Select a map...</option>
            <option
              v-for="map in allMaps"
              :key="map.id"
              :value="map.id"
            >
              {{ map.name }}{{ map.module_id ? '' : ' (Campaign)' }}
            </option>
          </select>
        </div>
      </div>

      <div class="toolbar-center">
        <span class="module-name">{{ module?.name || 'Loading...' }}</span>
      </div>

      <div class="toolbar-right">
        <!-- Player Display Toggle -->
        <button
          class="toolbar-button"
          :class="{ active: isDisplayOpen }"
          @click="togglePlayerDisplay"
          title="Toggle Player Display"
        >
          <Monitor class="icon" :stroke-width="1.5" aria-hidden="true" />
          <span class="button-label">{{ isDisplayOpen ? 'Display On' : 'Display' }}</span>
        </button>

        <!-- Blackout Toggle (only when display is open) -->
        <button
          v-if="isDisplayOpen"
          class="toolbar-button"
          :class="{ active: isBlackout }"
          @click="handleBlackoutToggle"
          title="Toggle Blackout"
        >
          <EyeOff class="icon" :stroke-width="1.5" aria-hidden="true" />
        </button>

        <!-- Fullscreen Toggle -->
        <button
          class="toolbar-button"
          @click="handleFullscreen"
          title="Toggle Fullscreen"
        >
          <Maximize class="icon" :stroke-width="1.5" aria-hidden="true" />
        </button>
      </div>
    </header>

    <!-- Map Area -->
    <main class="map-container">
      <DmMapViewer
        v-if="activeMapId"
        class="map-viewer"
        :map-id="activeMapId"
        :grid-type="activeMap?.grid_type"
        :grid-size-px="activeMap?.grid_size_px"
        :grid-offset-x="activeMap?.grid_offset_x"
        :grid-offset-y="activeMap?.grid_offset_y"
        :show-grid="true"
        :campaign-id="campaignId"
        :module-id="activeMap?.module_id"
        :uvtt-file-path="activeMap?.image_path"
        :selected-token-id="selectedTokenId"
        :current-turn-token-id="combatMap.currentTurnTokenId"
        :down-token-ids="combatMap.downTokenIds"
        @token-selected="selectedTokenId = $event"
        @visible-tokens="visibleTokenIds = $event"
      />
      <div v-else class="no-map-selected">
        <p>Select a map from the dropdown above</p>
      </div>
      <InitiativeTracker
        v-if="moduleId"
        :module-id="moduleId"
        :campaign-id="campaignId"
        :map-id="activeMapId"
        :selected-token-id="selectedTokenId"
        :visible-token-ids="visibleTokenIds"
        @select-token="selectedTokenId = $event"
        @map-state="combatMap = $event"
      />
    </main>
    <DialogHost />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { EyeOff, Maximize, Monitor } from '@lucide/vue'
import DialogHost from '@/shared/components/ui/DialogHost.vue'
import { invoke } from '@tauri-apps/api/core'
import DmMapViewer from '@/components/DmMapViewer.vue'
import InitiativeTracker from '@/features/combat/components/InitiativeTracker.vue'
import type { CombatMapState } from '@/features/combat/mapLink'
import { usePlayerDisplay } from '@/composables/windows/usePlayerDisplay'
import { useDmMapWindow } from '@/composables/windows/useDmMapWindow'

interface MapSummary {
  id: string
  campaign_id: string
  module_id: string | null
  name: string
  description: string | null
  grid_type: string
  grid_size_px: number | null
  grid_offset_x: number
  grid_offset_y: number
  width_px: number
  height_px: number
  lighting_mode: string
  image_path: string
}

interface Module {
  id: string
  name: string
  campaign_id: string
}

// Get URL params
const urlParams = new URLSearchParams(window.location.search)
const moduleId = urlParams.get('moduleId') || ''
const campaignId = urlParams.get('campaignId') || ''

// Player display state
const {
  isDisplayOpen,
  isBlackout,
  toggleDisplay,
  toggleBlackout
} = usePlayerDisplay()

// DM map window controls
const { toggleFullscreen } = useDmMapWindow()

// Module data
const module = ref<Module | null>(null)

// Combat tracker <-> map (MIMIR-T-0680)
const selectedTokenId = ref<string | null>(null)
const visibleTokenIds = ref<string[]>([])
const combatMap = ref<CombatMapState>({ currentTurnTokenId: null, downTokenIds: [] })

// Map state
const allMaps = ref<MapSummary[]>([])
const activeMapId = ref<string | null>(null)
const mapsLoading = ref(false)

// Get active map details
const activeMap = computed(() => {
  if (!activeMapId.value) return null
  return allMaps.value.find(m => m.id === activeMapId.value) || null
})

// Load module info
async function loadModule() {
  if (!moduleId) return
  try {
    const response = await invoke<{ success: boolean; data?: Module }>('get_module', {
      id: moduleId
    })
    if (response.success && response.data) {
      module.value = response.data
    }
  } catch (e) {
    console.error('Failed to load module:', e)
  }
}

// Load maps
async function loadMaps() {
  if (!campaignId) return

  mapsLoading.value = true
  try {
    const response = await invoke<{ success: boolean; data?: MapSummary[] }>('list_campaign_maps', {
      campaignId
    })

    if (response.success && response.data) {
      // Filter to show campaign-level maps + this module's maps
      allMaps.value = response.data.filter(map =>
        map.module_id === null || map.module_id === moduleId
      )

      // Auto-select first module map (or first map if no module maps)
      if (!activeMapId.value && allMaps.value.length > 0) {
        const moduleMap = allMaps.value.find(m => m.module_id === moduleId)
        activeMapId.value = moduleMap?.id || allMaps.value[0].id
      }
    }
  } catch (e) {
    console.error('Failed to load maps:', e)
  } finally {
    mapsLoading.value = false
  }
}

// Handle map selection
async function onMapSelected() {
  const map = activeMap.value
  if (!map) return

  // If display is open, send the map to it
  if (isDisplayOpen.value) {
    try {
      await invoke('send_map_to_display', {
        mapId: map.id,
        gridType: map.grid_type,
        gridSizePx: map.grid_size_px,
        gridOffsetX: map.grid_offset_x,
        gridOffsetY: map.grid_offset_y,
        ambientLight: map.lighting_mode,
        mapWidth: map.width_px,
        mapHeight: map.height_px
      })
    } catch (err) {
      console.error('Failed to send map to display:', err)
    }
  }
}

// Toggle player display
async function togglePlayerDisplay() {
  try {
    const wasOpen = isDisplayOpen.value
    await toggleDisplay()

    // If we just opened the display and have an active map, send it
    if (!wasOpen && isDisplayOpen.value && activeMap.value) {
      setTimeout(async () => {
        try {
          const map = activeMap.value
          if (map) {
            await invoke('send_map_to_display', {
              mapId: map.id,
              gridType: map.grid_type,
              gridSizePx: map.grid_size_px,
              gridOffsetX: map.grid_offset_x,
              gridOffsetY: map.grid_offset_y,
              ambientLight: map.lighting_mode,
              mapWidth: map.width_px,
              mapHeight: map.height_px
            })
          }
        } catch (err) {
          console.error('Failed to send map after display open:', err)
        }
      }, 500)
    }
  } catch (err) {
    console.error('Failed to toggle player display:', err)
  }
}

// Toggle blackout
async function handleBlackoutToggle() {
  try {
    await toggleBlackout()
  } catch (err) {
    console.error('Failed to toggle blackout:', err)
  }
}

// Toggle fullscreen
async function handleFullscreen() {
  try {
    await toggleFullscreen()
  } catch (err) {
    console.error('Failed to toggle fullscreen:', err)
  }
}

// Watch for active map changes to send to display
watch(activeMapId, () => {
  onMapSelected()
})

onMounted(async () => {
  const t0 = performance.now()
  console.log('[perf] DmMapWindow: onMounted')

  const t1 = performance.now()
  await loadModule()
  console.log(`[perf]   loadModule: ${(performance.now() - t1).toFixed(0)}ms`)

  const t2 = performance.now()
  await loadMaps()
  console.log(`[perf]   loadMaps: ${(performance.now() - t2).toFixed(0)}ms`)

  console.log(`[perf] DmMapWindow: onMounted total: ${(performance.now() - t0).toFixed(0)}ms`)
})
</script>

<style scoped>
.dm-map-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
  overflow: hidden;
}

/* Toolbar */
.dm-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 1rem;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  gap: 1rem;
}

.toolbar-left,
.toolbar-center,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toolbar-left {
  flex: 1;
}

.toolbar-center {
  flex: 0 0 auto;
}

.toolbar-right {
  flex: 1;
  justify-content: flex-end;
}

/* Map Selector */
.map-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.map-selector label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.map-select {
  padding: 0.375rem 0.75rem;
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 0.875rem;
  min-width: 200px;
  cursor: pointer;
}

.map-select:hover {
  border-color: var(--color-primary-500);
}

.map-select:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}

/* Module Name */
.module-name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

/* Toolbar Buttons */
.toolbar-button {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.toolbar-button:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-primary-500);
}

.toolbar-button.active {
  background: var(--color-success);
  border-color: var(--color-success);
  color: white;
}

.toolbar-button .icon {
  width: 1rem;
  height: 1rem;
}

.toolbar-button .button-label {
  font-weight: 500;
}

/* Map Container */
.map-container {
  flex: 1;
  overflow: hidden;
  display: flex;
}

/* The map takes the width the initiative tracker leaves. */
.map-viewer {
  flex: 1;
  min-width: 0;
}

.no-map-selected {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  font-size: 1.125rem;
}
</style>
