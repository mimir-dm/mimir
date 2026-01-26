<template>
  <div class="dm-map-viewer">
    <!-- Toolbar -->
    <div class="viewer-toolbar">
      <div class="toolbar-group">
        <span class="toolbar-label">Zoom:</span>
        <button class="toolbar-btn" @click="zoomOut" :disabled="!mapImageUrl">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4 10a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H4.75A.75.75 0 014 10z" clip-rule="evenodd" />
          </svg>
        </button>
        <span class="zoom-level">{{ Math.round(zoom * 100) }}%</span>
        <button class="toolbar-btn" @click="zoomIn" :disabled="!mapImageUrl">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z" />
          </svg>
        </button>
        <button class="toolbar-btn" @click="resetView" :disabled="!mapImageUrl" title="Reset view">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M15.312 11.424a5.5 5.5 0 01-9.201 2.466l-.312-.311h2.433a.75.75 0 000-1.5H3.989a.75.75 0 00-.75.75v4.242a.75.75 0 001.5 0v-2.43l.31.31a7 7 0 0011.712-3.138.75.75 0 00-1.449-.39zm1.23-3.723a.75.75 0 00.219-.53V2.929a.75.75 0 00-1.5 0V5.36l-.31-.31A7 7 0 003.239 8.188a.75.75 0 101.448.389A5.5 5.5 0 0113.89 6.11l.311.31h-2.432a.75.75 0 000 1.5h4.243a.75.75 0 00.53-.219z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>

      <div class="toolbar-group">
        <button
          class="toolbar-btn add-pcs-btn"
          @click="addAllPCsToMap"
          :disabled="!mapImageUrl || !props.campaignId || addingPCs"
          title="Add all player characters to map"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 9a3 3 0 100-6 3 3 0 000 6zM6 8a2 2 0 11-4 0 2 2 0 014 0zM1.49 15.326a.78.78 0 01-.358-.442 3 3 0 014.308-3.516 6.484 6.484 0 00-1.905 3.959c-.023.222-.014.442.025.654a4.97 4.97 0 01-2.07-.655zM16.44 15.98a4.97 4.97 0 002.07-.654.78.78 0 00.357-.442 3 3 0 00-4.308-3.517 6.484 6.484 0 011.907 3.96 2.32 2.32 0 01-.026.654zM18 8a2 2 0 11-4 0 2 2 0 014 0zM5.304 16.19a.844.844 0 01-.277-.71 5 5 0 019.947 0 .843.843 0 01-.277.71A6.975 6.975 0 0110 18a6.974 6.974 0 01-4.696-1.81z" />
          </svg>
          <span>{{ addingPCs ? 'Adding...' : 'Add PCs' }}</span>
        </button>
      </div>

      <!-- Reveal Map Toggle (red eye icon) -->
      <div class="toolbar-group">
        <button
          class="reveal-btn"
          :class="{ active: revealMap }"
          @click="toggleRevealMap"
          :disabled="!mapImageUrl"
          :title="revealMap ? 'Hide map (restore fog)' : 'Reveal entire map to players'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 12.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z" />
            <path fill-rule="evenodd" d="M.664 10.59a1.651 1.651 0 010-1.186A10.004 10.004 0 0110 3c4.257 0 7.893 2.66 9.336 6.41.147.381.146.804 0 1.186A10.004 10.004 0 0110 17c-4.257 0-7.893-2.66-9.336-6.41zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>

      <!-- LOS Controls (only when UVTT data available) -->
      <div v-if="uvttLoaded" class="toolbar-group los-controls">
        <span class="toolbar-label">LOS:</span>
        <div class="btn-group">
          <button
            class="btn-group-item"
            :class="{ active: !tokenOnlyLos }"
            @click="tokenOnlyLos = false"
            :disabled="!mapImageUrl || revealMap"
            title="Fog mode: hide map outside token vision"
          >Fog</button>
          <button
            class="btn-group-item"
            :class="{ active: tokenOnlyLos }"
            @click="tokenOnlyLos = true"
            :disabled="!mapImageUrl || revealMap"
            title="Token mode: show map, hide tokens outside vision"
          >Token</button>
        </div>
        <button
          class="toolbar-btn"
          :class="{ active: showLosDebug }"
          @click="showLosDebug = !showLosDebug"
          :disabled="!mapImageUrl"
          title="Toggle LOS debug view"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 12.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z" />
            <path fill-rule="evenodd" d="M.664 10.59a1.651 1.651 0 010-1.186A10.004 10.004 0 0110 3c4.257 0 7.893 2.66 9.336 6.41.147.381.146.804 0 1.186A10.004 10.004 0 0110 17c-4.257 0-7.893-2.66-9.336-6.41zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
          </svg>
        </button>
        <!-- Ambient Light Dropdown -->
        <select
          class="ambient-select"
          :value="currentAmbientLight"
          @change="ambientLightOverride = ($event.target as HTMLSelectElement).value as 'bright' | 'dim' | 'darkness'"
          :disabled="!mapImageUrl"
          title="Ambient light level"
        >
          <option value="bright">☀️ Bright</option>
          <option value="dim">🌙 Dim</option>
          <option value="darkness">🌑 Dark</option>
        </select>
      </div>

      <div class="toolbar-group">
        <button
          class="toolbar-btn sync-btn"
          :class="{ active: autoSync }"
          @click="toggleAutoSync"
          title="Auto-sync viewport to player display"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path d="M12.232 4.232a2.5 2.5 0 013.536 3.536l-1.225 1.224a.75.75 0 001.061 1.06l1.224-1.224a4 4 0 00-5.656-5.656l-3 3a4 4 0 00.225 5.865.75.75 0 00.977-1.138 2.5 2.5 0 01-.142-3.667l3-3z" />
            <path d="M11.603 7.963a.75.75 0 00-.977 1.138 2.5 2.5 0 01.142 3.667l-3 3a2.5 2.5 0 01-3.536-3.536l1.225-1.224a.75.75 0 00-1.061-1.06l-1.224 1.224a4 4 0 105.656 5.656l3-3a4 4 0 00-.225-5.865z" />
          </svg>
          <span>{{ autoSync ? 'Synced' : 'Sync' }}</span>
        </button>
        <button
          v-if="!autoSync"
          class="toolbar-btn push-btn"
          @click="pushViewport"
          :disabled="!mapImageUrl || !isDisplayOpen"
          title="Push current view to player display"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 10-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z" clip-rule="evenodd" />
          </svg>
          <span>Push View</span>
        </button>
      </div>

      <!-- Print button -->
      <div class="toolbar-group">
        <button
          class="toolbar-btn print-btn"
          @click="showPrintDialog = true"
          :disabled="!mapImageUrl"
          title="Print map to PDF"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M5 4v3H4a2 2 0 00-2 2v3a2 2 0 002 2h1v2a2 2 0 002 2h6a2 2 0 002-2v-2h1a2 2 0 002-2V9a2 2 0 00-2-2h-1V4a2 2 0 00-2-2H7a2 2 0 00-2 2zm8 0H7v3h6V4zm0 8H7v4h6v-4z" clip-rule="evenodd" />
          </svg>
          <span>Print</span>
        </button>
      </div>
    </div>

    <!-- Map Viewport -->
    <div
      class="map-viewport"
      ref="viewport"
      @mousedown="startPan"
      @mousemove="onPan"
      @mouseup="endPan"
      @mouseleave="endPan"
      @wheel.prevent="onWheel"
    >
      <div v-if="loading" class="loading-state">
        Loading map...
      </div>

      <EmptyState
        v-else-if="!mapImageUrl"
        variant="campaigns"
        title="No map selected"
        description="Select a map from the sidebar to view and control it"
      />

      <div
        v-else
        class="map-container"
        :style="mapContainerStyle"
      >
        <img
          :src="mapImageUrl"
          :alt="mapName"
          class="map-image"
          @load="onImageLoad"
          ref="mapImage"
          draggable="false"
        />

        <!-- Grid Overlay -->
        <svg
          v-if="showGrid && gridType !== 'none' && imageLoaded"
          class="grid-overlay"
          :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
          :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
        >
          <defs>
            <pattern
              v-if="gridType === 'square'"
              id="dmGridPattern"
              :width="effectiveGridSize"
              :height="effectiveGridSize"
              patternUnits="userSpaceOnUse"
              :x="effectiveGridOffsetX"
              :y="effectiveGridOffsetY"
            >
              <rect
                :width="effectiveGridSize"
                :height="effectiveGridSize"
                fill="none"
                stroke="rgba(255, 255, 255, 0.4)"
                stroke-width="1"
              />
            </pattern>
            <pattern
              v-if="gridType === 'hex'"
              id="dmGridPattern"
              :width="effectiveGridSize * 1.5"
              :height="effectiveGridSize * 1.732"
              patternUnits="userSpaceOnUse"
              :x="effectiveGridOffsetX"
              :y="effectiveGridOffsetY"
            >
              <polygon
                :points="hexPoints"
                fill="none"
                stroke="rgba(255, 255, 255, 0.4)"
                stroke-width="1"
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#dmGridPattern)" />
        </svg>

        <!-- UVTT Map Lights (embedded in map file) -->
        <LightOverlay
          v-if="uvttLoaded && uvttLights.length > 0 && imageLoaded"
          :lights="uvttLights"
          :walls="blockingWalls"
          :map-width="mapWidth"
          :map-height="mapHeight"
          :show-debug="showLosDebug"
          blend-mode="soft-light"
        />

        <!-- Fog of War Overlay (DM view - semi-transparent) -->
        <!-- Uses visibility polygons when UVTT data available, otherwise circles -->
        <svg
          v-if="fogEnabled && imageLoaded"
          class="fog-overlay dm-fog"
          :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
          :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
        >
          <defs>
            <!-- Blur filter for soft vision edges -->
            <filter id="visionBlur" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur in="SourceGraphic" stdDeviation="20" />
            </filter>
            <mask id="dmFogMask">
              <!-- White = fogged, Black = revealed -->
              <rect width="100%" height="100%" fill="white" />
              <!-- Use visibility polygons when LOS blocking is enabled and UVTT data available -->
              <g v-if="uvttLoaded && useLosBlocking" filter="url(#visionBlur)">
                <path
                  v-for="vis in visibilityPolygons"
                  :key="'vis-' + vis.tokenId"
                  :d="vis.path"
                  fill="black"
                />
              </g>
              <!-- Fall back to circles when no UVTT data or LOS disabled -->
              <g v-else filter="url(#visionBlur)">
                <circle
                  v-for="token in playerTokensWithVision"
                  :key="'vision-' + token.id"
                  :cx="token.x"
                  :cy="token.y"
                  :r="getTokenVisionRadiusPx(token)"
                  fill="black"
                />
              </g>
              <!-- Map lights create visible pools in dim/dark conditions -->
              <g v-if="currentAmbientLight !== 'bright' && mapLightZones.length > 0" filter="url(#visionBlur)">
                <circle
                  v-for="zone in mapLightZones"
                  :key="'maplight-' + zone.id"
                  :cx="zone.x"
                  :cy="zone.y"
                  :r="zone.radiusPx"
                  fill="black"
                />
              </g>
            </mask>
          </defs>
          <!-- Semi-transparent fog for DM view -->
          <rect
            width="100%"
            height="100%"
            fill="rgba(0, 0, 0, 0.5)"
            mask="url(#dmFogMask)"
          />
        </svg>

        <!-- Door Interaction Overlay (DM only, when UVTT data available) -->
        <DoorInteractionOverlay
          v-if="uvttLoaded && uvttPortals.length > 0 && imageLoaded"
          :portals="uvttPortals"
          :map-width="mapWidth"
          :map-height="mapHeight"
          @toggle-door="handleDoorToggle"
        />

        <!-- LOS Debug Overlay -->
        <LosDebugOverlay
          v-if="showLosDebug && uvttLoaded && imageLoaded"
          :walls="uvttWalls"
          :portals="uvttPortals"
          :map-width="mapWidth"
          :map-height="mapHeight"
          :visible="showLosDebug"
          :show-legend="true"
        />

        <!-- Light Source Layer -->
        <LightSourceRenderer
          v-if="imageLoaded && lightSources.length > 0"
          :lights="lightSources"
          :tokens="tokens"
          :grid-size-px="effectiveGridSize"
          :map-width="mapWidth"
          :map-height="mapHeight"
          :show-inactive="true"
          :show-bright-border="true"
          :show-center-dot="true"
          :show-labels="false"
          @light-context="handleLightContext"
        />

        <!-- Token Layer -->
        <TokenRenderer
          v-if="imageLoaded && tokens.length > 0"
          ref="tokenRendererRef"
          :tokens="tokens"
          :grid-size-px="effectiveGridSize"
          :base-scale="1"
          :show-hidden="true"
          :selected-token-id="selectedTokenId"
          :dragging-token-id="draggingTokenId"
          :drag-offset="dragOffset"
          :interactive="true"
          :token-lights="tokenLightInfo"
          :dead-token-ids="deadTokenIds"
          :token-images="tokenImages"
          @token-click="handleTokenClick"
          @token-context="handleTokenContext"
          @token-drag-start="handleTokenDragStart"
        />

        <!-- Map Markers Layer (Traps & POIs) -->
        <svg
          v-if="imageLoaded && (mapTraps.length > 0 || mapPois.length > 0)"
          class="markers-overlay"
          :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
          :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
        >
          <!-- Traps -->
          <g class="trap-markers">
            <g
              v-for="trap in mapTraps"
              :key="'trap-' + trap.id"
              class="trap-marker"
              :class="{ selected: selectedTrapId === trap.id }"
              :transform="`translate(${trap.grid_x * effectiveGridSize + effectiveGridSize / 2}, ${trap.grid_y * effectiveGridSize + effectiveGridSize / 2})`"
              @click.stop="selectedTrapId = selectedTrapId === trap.id ? null : trap.id"
            >
              <!-- Trap icon (warning triangle) -->
              <polygon
                points="-12,-10 12,-10 0,12"
                :fill="trap.visible ? '#ef4444' : '#6b7280'"
                stroke="#fff"
                stroke-width="2"
              />
              <text
                y="2"
                text-anchor="middle"
                fill="#fff"
                font-size="12"
                font-weight="bold"
              >!</text>
              <!-- Label on hover/select -->
              <text
                v-if="selectedTrapId === trap.id"
                y="-18"
                text-anchor="middle"
                fill="#fff"
                font-size="11"
                class="marker-label"
              >{{ trap.name }}</text>
            </g>
          </g>

          <!-- POIs -->
          <g class="poi-markers">
            <g
              v-for="poi in mapPois"
              :key="'poi-' + poi.id"
              class="poi-marker"
              :class="{ selected: selectedPoiId === poi.id }"
              :transform="`translate(${poi.grid_x * effectiveGridSize + effectiveGridSize / 2}, ${poi.grid_y * effectiveGridSize + effectiveGridSize / 2})`"
              @click.stop="selectedPoiId = selectedPoiId === poi.id ? null : poi.id"
            >
              <!-- POI icon (circle with icon) -->
              <circle
                r="14"
                :fill="poi.color || '#3b82f6'"
                stroke="#fff"
                stroke-width="2"
              />
              <text
                y="5"
                text-anchor="middle"
                fill="#fff"
                font-size="14"
              >{{ getPoiIcon(poi.icon) }}</text>
              <!-- Label on hover/select -->
              <text
                v-if="selectedPoiId === poi.id"
                y="-20"
                text-anchor="middle"
                fill="#fff"
                font-size="11"
                class="marker-label"
              >{{ poi.name }}</text>
            </g>
          </g>
        </svg>
      </div>
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <span v-if="mapName">{{ mapName }}</span>
      <span v-if="mapWidth && mapHeight" class="dim">{{ mapWidth }}x{{ mapHeight }}</span>
      <span class="dim">Pan: {{ Math.round(panX) }}, {{ Math.round(panY) }}</span>
      <span v-if="isDisplayOpen" class="status-indicator connected">Display Connected</span>
      <span v-else class="status-indicator disconnected">Display Disconnected</span>
    </div>

    <!-- Token Context Menu -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <button @click="toggleSelectedTokenVisibility">
        {{ contextMenu.token?.visible_to_players ? 'Hide from Players' : 'Show to Players' }}
        <span class="shortcut">H</span>
      </button>
      <!-- Vision settings (PC tokens only) -->
      <button
        v-if="contextMenu.token?.token_type === 'pc'"
        @click="openVisionMenu"
      >
        Vision...
        <span class="shortcut">V</span>
      </button>
      <div class="context-menu-divider"></div>
      <!-- Light source options -->
      <button
        v-if="contextMenu.token && getTokenLightSource(contextMenu.token.id)"
        @click="toggleTokenLight"
        class="light-option"
      >
        {{ getTokenLightSource(contextMenu.token.id)?.is_active ? 'Extinguish Light' : 'Light Torch' }}
        <span class="shortcut">L</span>
      </button>
      <button
        v-else-if="contextMenu.token"
        @click="addTorchToToken"
        class="light-option"
      >
        Give Torch
        <span class="shortcut">L</span>
      </button>
      <div class="context-menu-divider"></div>
      <!-- Dead toggle -->
      <button
        v-if="contextMenu.token"
        @click="toggleTokenDead"
        :class="{ 'dead-option': isTokenDead(contextMenu.token.id) }"
      >
        {{ isTokenDead(contextMenu.token.id) ? 'Mark Alive' : 'Mark Dead' }}
        <span class="shortcut">D</span>
      </button>
    </div>

    <!-- Click outside to close context menu -->
    <div
      v-if="contextMenu.visible || lightContextMenu.visible || visionMenu.visible"
      class="context-menu-backdrop"
      @click="closeContextMenu"
    ></div>

    <!-- Token Vision Menu -->
    <TokenVisionMenu
      v-if="visionMenu.token"
      :visible="visionMenu.visible"
      :token="visionMenu.token"
      :x="visionMenu.x"
      :y="visionMenu.y"
      @close="closeVisionMenu"
      @updated="onVisionUpdated"
    />

    <!-- Light Source Context Menu -->
    <div
      v-if="lightContextMenu.visible"
      class="context-menu"
      :style="{ left: lightContextMenu.x + 'px', top: lightContextMenu.y + 'px' }"
      @click.stop
    >
      <button @click="toggleLightFromContext" class="light-option">
        {{ lightContextMenu.light?.is_active ? 'Extinguish' : 'Ignite' }}
      </button>
    </div>

    <!-- Map Print Dialog -->
    <MapPrintDialog
      :visible="showPrintDialog"
      :map-id="mapId"
      :map-name="mapName"
      :map-dimensions="mapWidth && mapHeight ? { width: mapWidth, height: mapHeight } : undefined"
      :grid-size-px="effectiveGridSize"
      @close="showPrintDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, toRef, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
import { usePlayerDisplay } from '@/composables/usePlayerDisplay'
import { useTokens } from '@/composables/useTokens'
import { useLightSources, type LightSourceSummary } from '@/composables/useLightSources'
import { useVisionCalculation, type AmbientLight } from '@/composables/useVisionCalculation'
import { useUvttMap } from '@/composables/useUvttMap'
import { useMultiTokenVisibility } from '@/composables/useVisibilityPolygon'
import TokenRenderer from '@/components/tokens/TokenRenderer.vue'
import LightSourceRenderer from '@/components/lighting/LightSourceRenderer.vue'
import LosDebugOverlay from '@/components/los/LosDebugOverlay.vue'
import DoorInteractionOverlay from '@/components/los/DoorInteractionOverlay.vue'
import LightOverlay from '@/components/los/LightOverlay.vue'
import MapPrintDialog from '@/components/print/MapPrintDialog.vue'
import TokenVisionMenu from '@/components/tokens/TokenVisionMenu.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import type { Token } from '@/types/api'
import { useCharacterStore } from '@/stores/characters'

// Throttle helper for smooth updates
function throttle<T extends (...args: any[]) => void>(fn: T, limit: number): T {
  let lastCall = 0
  let pendingCall: number | null = null

  return ((...args: any[]) => {
    const now = Date.now()
    const remaining = limit - (now - lastCall)

    if (remaining <= 0) {
      if (pendingCall) {
        cancelAnimationFrame(pendingCall)
        pendingCall = null
      }
      lastCall = now
      fn(...args)
    } else if (!pendingCall) {
      pendingCall = requestAnimationFrame(() => {
        lastCall = Date.now()
        pendingCall = null
        fn(...args)
      })
    }
  }) as T
}

interface Props {
  mapId: string | null
  gridType?: string
  gridSizePx?: number | null
  gridOffsetX?: number
  gridOffsetY?: number
  showGrid?: boolean
  /** Campaign ID for UVTT file loading */
  campaignId?: string | null
  /** Module ID for UVTT file loading (null for campaign-level maps) */
  moduleId?: string | null
  /** UVTT file path (e.g., "abc123.dd2vtt") */
  uvttFilePath?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  gridType: 'none',
  gridSizePx: null,
  gridOffsetX: 0,
  gridOffsetY: 0,
  showGrid: true,
  campaignId: null,
  moduleId: null,
  uvttFilePath: null
})

// Computed grid values (with defaults for null)
const effectiveGridSize = computed(() => props.gridSizePx ?? 70)
const effectiveGridOffsetX = computed(() => props.gridOffsetX ?? 0)
const effectiveGridOffsetY = computed(() => props.gridOffsetY ?? 0)

const { isDisplayOpen, updateViewport } = usePlayerDisplay()

// Token state - will be initialized when mapId is available
const tokens = ref<Token[]>([])
const selectedTokenId = ref<string | null>(null)

// Token images cache (token_id -> base64 data URL)
const tokenImages = ref<Map<string, string>>(new Map())

// Dead token state (frontend-only, not persisted)
const deadTokenIds = ref<string[]>([])

// Token drag state
const draggingTokenId = ref<string | null>(null)
const dragOffset = ref<{ x: number; y: number } | null>(null)
const dragStartPos = ref<{ x: number; y: number; tokenX: number; tokenY: number } | null>(null)
const tokenRendererRef = ref<InstanceType<typeof TokenRenderer> | null>(null)

// Token context menu state
const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
  token: Token | null
}>({
  visible: false,
  x: 0,
  y: 0,
  token: null
})

// Light source context menu state
const lightContextMenu = ref<{
  visible: boolean
  x: number
  y: number
  light: LightSourceSummary | null
}>({
  visible: false,
  x: 0,
  y: 0,
  light: null
})

// Token vision menu state
const visionMenu = ref<{
  visible: boolean
  x: number
  y: number
  token: Token | null
}>({
  visible: false,
  x: 0,
  y: 0,
  token: null
})

// Add PCs state
const characterStore = useCharacterStore()
const addingPCs = ref(false)

// Print dialog state
const showPrintDialog = ref(false)

// Fog of war state
const fogEnabled = ref(false)

// Light source state
const lightSources = ref<LightSourceSummary[]>([])

// Map trap and POI types
interface MapTrap {
  id: string
  map_id: string
  grid_x: number
  grid_y: number
  name: string
  description: string | null
  trigger_description: string | null
  effect_description: string | null
  dc: number | null
  visible: number
  created_at: string
  updated_at: string
}

interface MapPoi {
  id: string
  map_id: string
  grid_x: number
  grid_y: number
  name: string
  description: string | null
  icon: string
  color: string | null
  visible: number
  created_at: string
  updated_at: string
}

// Map traps and POIs state
const mapTraps = ref<MapTrap[]>([])
const mapPois = ref<MapPoi[]>([])
const selectedTrapId = ref<string | null>(null)
const selectedPoiId = ref<string | null>(null)

// Computed: token light info for TokenRenderer
const tokenLightInfo = computed(() => {
  return lightSources.value
    .filter(ls => ls.token_id !== null)
    .map(ls => ({
      tokenId: ls.token_id!,
      isActive: ls.is_active
    }))
})

// LOS (Line of Sight) state
const showLosDebug = ref(false)
const useLosBlocking = ref(true) // Always use wall-based LOS blocking when UVTT data available
const tokenOnlyLos = ref(false) // LOS mode: false = Fog (map hidden), true = Token (map visible, tokens filtered)
const revealMap = ref(false) // Master toggle: false = hiding active (safe default), true = everything revealed

// UVTT map data composable
const mapIdRef = toRef(props, 'mapId')
const campaignIdRef = toRef(props, 'campaignId')
const moduleIdRef = toRef(props, 'moduleId')
const uvttFilePathRef = toRef(props, 'uvttFilePath')
const {
  walls: uvttWalls,
  portals: uvttPortals,
  lights: uvttLights,
  ambientLight: uvttAmbientLight,
  blockingWalls,
  isLoaded: uvttLoaded,
  togglePortal,
  mapWidthPx: uvttMapWidth,
  mapHeightPx: uvttMapHeight
} = useUvttMap(mapIdRef, campaignIdRef, moduleIdRef, uvttFilePathRef)

// Ambient light - initialized from UVTT, can be overridden via UI
const ambientLightOverride = ref<'bright' | 'dim' | 'darkness' | null>(null)
const currentAmbientLight = computed(() =>
  ambientLightOverride.value ?? uvttAmbientLight.value ?? 'bright'
)

// Reset override when UVTT data changes
watch(uvttAmbientLight, () => {
  ambientLightOverride.value = null
})

// Send fog updates when ambient light changes
watch(currentAmbientLight, () => {
  if (!revealMap.value) {
    sendFogToDisplay()
  }
})

// Send fog updates when light sources change (token lights affect vision in darkness)
watch(lightSources, () => {
  if (!revealMap.value) {
    sendFogToDisplay()
  }
}, { deep: true })

// Send fog updates when visibility settings change
watch([tokenOnlyLos, revealMap], () => {
  console.log('Visibility changed - revealMap:', revealMap.value, 'tokenOnlyLos:', tokenOnlyLos.value)
  sendFogToDisplay()
})

// Vision calculation using the proper D&D 5e rules
// This considers map light sources when determining token vision
const ambientLightRef = computed<AmbientLight>(() => currentAmbientLight.value)
const mapWidthForVision = computed(() => mapWidth.value || uvttMapWidth.value)
const mapHeightForVision = computed(() => mapHeight.value || uvttMapHeight.value)

const {
  pcVision,
  allTokenVision,
  lightZones: visionLightZones
} = useVisionCalculation({
  tokens,
  lightSources,
  ambientLight: ambientLightRef,
  gridSizePx: effectiveGridSize,
  mapWidth: mapWidthForVision,
  mapHeight: mapHeightForVision
})

// Tokens with vision for visibility polygon calculation
// Uses pcVision from useVisionCalculation which considers map light sources
const tokensWithVision = computed(() => {
  return pcVision.value
    .filter(v => v.visionRadiusPx > 0) // Exclude tokens that can't see
    .map(v => ({
      id: v.tokenId,
      x: v.x,
      y: v.y,
      visionRadius: v.visionRadiusPx
    }))
})

// Map dimensions for visibility calculation
const mapWidthRef = computed(() => mapWidth.value || uvttMapWidth.value)
const mapHeightRef = computed(() => mapHeight.value || uvttMapHeight.value)

// Visibility polygon calculation
const {
  visibilityPolygons,
  combinedVisibilityPath
} = useMultiTokenVisibility(
  tokensWithVision,
  computed(() => useLosBlocking.value ? blockingWalls.value : []),
  uvttPortals,
  mapWidthRef,
  mapHeightRef
)

// Point-in-polygon check using ray casting algorithm
function isPointInPolygon(x: number, y: number, polygon: { x: number; y: number }[]): boolean {
  if (polygon.length < 3) return false

  let inside = false
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const xi = polygon[i].x, yi = polygon[i].y
    const xj = polygon[j].x, yj = polygon[j].y

    if (((yi > y) !== (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi)) {
      inside = !inside
    }
  }
  return inside
}

// Check if a point is visible to any PC (within their visibility polygon)
function isPointVisibleToAnyPC(x: number, y: number): boolean {
  // If LOS blocking is disabled, everything is visible
  if (!useLosBlocking.value) return true

  // Check each PC's visibility polygon
  for (const vis of visibilityPolygons.value) {
    if (isPointInPolygon(x, y, vis.polygon)) {
      return true
    }
  }
  return false
}

// Map light zones that contribute to visibility
// Only includes lights that at least one PC can see (has LOS to)
const mapLightZones = computed(() => {
  const zones: { id: string; x: number; y: number; radiusPx: number }[] = []
  const gridSize = effectiveGridSize.value

  // Add UVTT lights (already in pixel coordinates) - only if visible to a PC
  if (uvttLoaded.value) {
    for (const light of uvttLights.value) {
      if (isPointVisibleToAnyPC(light.position.x, light.position.y)) {
        zones.push({
          id: `uvtt-${light.id}`,
          x: light.position.x,
          y: light.position.y,
          radiusPx: light.range // Already in pixels
        })
      }
    }
  }

  // Add active database light sources (convert feet to pixels) - only if visible
  for (const light of lightSources.value) {
    if (!light.is_active) continue
    if (isPointVisibleToAnyPC(light.x, light.y)) {
      // Use dim radius for the full visible zone
      const radiusPx = (light.dim_radius_ft / 5) * gridSize
      zones.push({
        id: `db-${light.id}`,
        x: light.x,
        y: light.y,
        radiusPx
      })
    }
  }

  return zones
})

// Handle door toggle
function handleDoorToggle(portalId: string) {
  togglePortal(portalId)
  // Trigger fog update when door state changes
  if (!revealMap.value) {
    sendFogToDisplay()
  }
}

// Backend token response type (matches TokenResponse from backend)
interface BackendToken {
  id: string
  map_id: string
  name: string
  token_type: string
  size: string
  x: number
  y: number
  visible_to_players: boolean
  color: string | null
  image_path: string | null
  monster_id: string | null
  character_id: string | null
  notes: string | null
  vision_type: string
  vision_range_ft: number | null
  vision_bright_ft: number | null
  vision_dim_ft: number | null
  vision_dark_ft: number
  light_radius_ft: number
  created_at: string
  updated_at: string
}

// Transform backend token to frontend Token format
function transformToken(backendToken: BackendToken): Token {
  return {
    id: backendToken.id,
    map_id: backendToken.map_id,
    name: backendToken.name,
    token_type: backendToken.token_type as Token['token_type'],
    size: backendToken.size as Token['size'],
    x: backendToken.x,
    y: backendToken.y,
    visible_to_players: backendToken.visible_to_players,
    color: backendToken.color,
    image_path: backendToken.image_path,
    monster_id: backendToken.monster_id,
    character_id: backendToken.character_id,
    notes: backendToken.notes,
    vision_type: backendToken.vision_type as Token['vision_type'],
    vision_range_ft: backendToken.vision_range_ft,
    vision_bright_ft: backendToken.vision_bright_ft,
    vision_dim_ft: backendToken.vision_dim_ft,
    vision_dark_ft: backendToken.vision_dark_ft,
    light_radius_ft: backendToken.light_radius_ft,
    created_at: backendToken.created_at,
    updated_at: backendToken.updated_at
  }
}

// Load tokens when map changes
async function loadTokens(mapId: string) {
  try {
    const response = await invoke<{ success: boolean; data?: BackendToken[] }>('list_tokens', { mapId })
    if (response.success && response.data) {
      // Transform backend tokens to frontend format (backend already converts to pixel coordinates)
      tokens.value = response.data.map(t => transformToken(t))
      // Load token images for tokens that have image_path
      await loadTokenImages()
      // Send visible tokens to player display
      sendTokensToDisplay()
    }
  } catch (e) {
    console.error('Failed to load tokens:', e)
    tokens.value = []
  }
}

// Load images for all tokens that have image_path
async function loadTokenImages() {
  const tokensWithImages = tokens.value.filter(t => t.image_path)
  const loadPromises = tokensWithImages.map(async (token) => {
    // Skip if already cached
    if (tokenImages.value.has(token.id)) return

    try {
      const response = await invoke<{ success: boolean; data?: string; error?: string }>('serve_token_image', { tokenId: token.id })
      if (response.success && response.data) {
        tokenImages.value.set(token.id, response.data)
      }
    } catch (e) {
      console.error(`Failed to load image for token ${token.id}:`, e)
    }
  })
  await Promise.all(loadPromises)
}

// Send visible tokens to player display via IPC
async function sendTokensToDisplay() {
  if (!isDisplayOpen.value || !props.mapId) return

  const visibleTokens = tokens.value.filter(t => t.visible_to_players)
  try {
    await emit('player-display:tokens-update', {
      mapId: props.mapId,
      tokens: visibleTokens,
      deadTokenIds: deadTokenIds.value
    })
  } catch (e) {
    console.error('Failed to send tokens to display:', e)
  }
}

// Load fog state
async function loadFogState(mapId: string) {
  try {
    const response = await invoke<{ success: boolean; data?: { fog_enabled: boolean } }>('get_fog_state', { mapId })
    if (response.success && response.data) {
      fogEnabled.value = response.data.fog_enabled
      // Send fog state to player display
      sendFogToDisplay()
    }
  } catch (e) {
    console.error('Failed to load fog state:', e)
  }
}

// Load light sources for the map
async function loadLightSources(mapId: string) {
  try {
    const response = await invoke<{ success: boolean; data?: LightSourceSummary[] }>('list_light_sources', { mapId })
    if (response.success && response.data) {
      lightSources.value = response.data
      // Send light sources to player display
      sendLightSourcesToDisplay()
    }
  } catch (e) {
    console.error('Failed to load light sources:', e)
    lightSources.value = []
  }
}

// Load traps for the map
async function loadMapTraps(mapId: string) {
  try {
    const response = await invoke<{ success: boolean; data?: MapTrap[] }>('list_map_traps', { mapId })
    if (response.success && response.data) {
      mapTraps.value = response.data
    }
  } catch (e) {
    console.error('Failed to load map traps:', e)
    mapTraps.value = []
  }
}

// Load POIs for the map
async function loadMapPois(mapId: string) {
  try {
    const response = await invoke<{ success: boolean; data?: MapPoi[] }>('list_map_pois', { mapId })
    if (response.success && response.data) {
      mapPois.value = response.data
    }
  } catch (e) {
    console.error('Failed to load map POIs:', e)
    mapPois.value = []
  }
}

// Send light sources to player display
async function sendLightSourcesToDisplay() {
  if (!isDisplayOpen.value || !props.mapId) return

  // Only send active light sources to player display
  const activeLights = lightSources.value.filter(l => l.is_active)
  try {
    await emit('player-display:light-sources-update', {
      mapId: props.mapId,
      lightSources: activeLights
    })
  } catch (e) {
    console.error('Failed to send light sources to display:', e)
  }
}

// Toggle reveal map (master toggle)
function toggleRevealMap() {
  revealMap.value = !revealMap.value
  sendFogToDisplay()
}

// Send fog state to player display (vision-based)
async function sendFogToDisplay() {
  if (!isDisplayOpen.value || !props.mapId) return

  // Calculate vision circles for player tokens using proper D&D 5e vision rules
  // pcVision considers map light sources when determining token vision
  const visionCircles = pcVision.value.map(vision => ({
    tokenId: vision.tokenId,
    x: vision.x,
    y: vision.y,
    radiusPx: vision.visionRadiusPx
  }))

  // Include visibility polygon data when UVTT LOS is available
  const visibilityPaths = uvttLoaded.value && useLosBlocking.value
    ? visibilityPolygons.value.map(v => ({
        tokenId: v.tokenId,
        path: v.path,
        polygon: v.polygon  // Include polygon points for Token LOS mode
      }))
    : []

  const payload = {
    mapId: props.mapId,
    // revealMap controls whether anything is shown to players
    revealMap: revealMap.value,
    // tokenOnlyLos: false = Fog mode (map hidden), true = Token mode (map visible)
    tokenOnlyLos: tokenOnlyLos.value,
    visionCircles,
    // UVTT LOS data
    useLosBlocking: uvttLoaded.value && useLosBlocking.value,
    visibilityPaths,
    // Send blocking walls and lights for player display to render shadows
    blockingWalls: uvttLoaded.value ? blockingWalls.value : [],
    uvttLights: uvttLoaded.value ? uvttLights.value : [],
    // Ambient light level
    ambientLight: currentAmbientLight.value
  }

  console.log('DmMapViewer: Sending fog-update:', {
    revealMap: payload.revealMap,
    tokenOnlyLos: payload.tokenOnlyLos,
    los: payload.useLosBlocking,
    paths: payload.visibilityPaths.length,
    circles: payload.visionCircles.length
  })

  try {
    await emit('player-display:fog-update', payload)
  } catch (e) {
    console.error('Failed to send fog to display:', e)
  }
}

// Handle token click
function handleTokenClick(token: Token) {
  selectedTokenId.value = token.id === selectedTokenId.value ? null : token.id
}

// Handle token context menu
function handleTokenContext(event: MouseEvent, token: Token) {
  selectedTokenId.value = token.id
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    token
  }
}

// Close context menu
function closeContextMenu() {
  contextMenu.value.visible = false
  lightContextMenu.value.visible = false
  visionMenu.value.visible = false
}

// Open vision settings menu for current token
function openVisionMenu() {
  if (!contextMenu.value.token) return
  const token = contextMenu.value.token

  // Close the context menu and open vision menu at same position
  const x = contextMenu.value.x
  const y = contextMenu.value.y
  contextMenu.value.visible = false

  visionMenu.value = {
    visible: true,
    x,
    y,
    token
  }
}

// Close vision menu
function closeVisionMenu() {
  visionMenu.value.visible = false
}

// Handle vision settings updated
function onVisionUpdated(updatedToken: Token) {
  // Update the token in our local array
  const index = tokens.value.findIndex(t => t.id === updatedToken.id)
  if (index !== -1) {
    tokens.value[index] = updatedToken
  }

  // Update the vision menu's token reference
  if (visionMenu.value.token?.id === updatedToken.id) {
    visionMenu.value.token = updatedToken
  }

  // Trigger fog recalculation when vision changes
  if (!revealMap.value) {
    sendFogToDisplay()
  }
}

// Handle light source context menu (right-click on light dot)
function handleLightContext(event: MouseEvent, light: LightSourceSummary) {
  // Close any existing menus
  contextMenu.value.visible = false
  lightContextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    light
  }
}

// Toggle light from context menu
async function toggleLightFromContext() {
  const light = lightContextMenu.value.light
  if (!light) return

  try {
    const response = await invoke<{ success: boolean; data?: LightSourceSummary; error?: string }>('toggle_light_source', {
      id: light.id
    })

    if (response.success && response.data) {
      // Update local light source
      const index = lightSources.value.findIndex(ls => ls.id === light.id)
      if (index !== -1) {
        const newLights = [...lightSources.value]
        newLights[index] = response.data
        lightSources.value = newLights
      }
      // Sync to player display
      sendLightSourcesToDisplay()
      // Update fog when hiding is active
      if (!revealMap.value) {
        sendFogToDisplay()
      }
    }
  } catch (e) {
    console.error('Failed to toggle light source:', e)
  }

  closeContextMenu()
}

// Toggle visibility of selected token
async function toggleSelectedTokenVisibility() {
  const token = contextMenu.value.token || tokens.value.find(t => t.id === selectedTokenId.value)
  if (!token) return

  try {
    const response = await invoke<{ success: boolean; data?: Token; error?: string }>('toggle_token_visibility', {
      id: token.id
    })

    if (response.success && response.data) {
      // Update local token
      const index = tokens.value.findIndex(t => t.id === token.id)
      if (index !== -1) {
        tokens.value[index] = response.data
      }
      // Sync to player display
      sendTokensToDisplay()
    }
  } catch (e) {
    console.error('Failed to toggle token visibility:', e)
  }

  closeContextMenu()
}

// Get light source for a token
function getTokenLightSource(tokenId: string): LightSourceSummary | undefined {
  return lightSources.value.find(ls => ls.token_id === tokenId)
}

// Toggle light source on/off
async function toggleTokenLight() {
  const token = contextMenu.value.token
  if (!token) return

  const light = getTokenLightSource(token.id)
  if (!light) return

  try {
    const response = await invoke<{ success: boolean; data?: LightSourceSummary; error?: string }>('toggle_light_source', {
      id: light.id
    })

    if (response.success && response.data) {
      // Update local light source (create new array for reactivity)
      const index = lightSources.value.findIndex(ls => ls.id === light.id)
      if (index !== -1) {
        const newLights = [...lightSources.value]
        newLights[index] = response.data
        lightSources.value = newLights
      }
      // Sync to player display
      sendLightSourcesToDisplay()
      // Update fog when hiding is active (light affects visibility)
      if (!revealMap.value) {
        sendFogToDisplay()
      }
    }
  } catch (e) {
    console.error('Failed to toggle light source:', e)
  }

  closeContextMenu()
}

// Add a torch to a token
async function addTorchToToken() {
  const token = contextMenu.value.token
  if (!token || !props.mapId) return

  try {
    const response = await invoke<{ success: boolean; data?: LightSourceSummary; error?: string }>('create_light_source', {
      request: {
        map_id: props.mapId,
        token_id: token.id,
        name: `${token.name}'s Torch`,
        light_type: 'torch',
        x: token.x,
        y: token.y,
        bright_radius_ft: 20,
        dim_radius_ft: 40,
        color: '#ffaa44'
      }
    })

    if (response.success && response.data) {
      // Add to local light sources
      lightSources.value.push(response.data)
      // Sync to player display
      sendLightSourcesToDisplay()
      // Update fog when hiding is active
      if (!revealMap.value) {
        sendFogToDisplay()
      }
    }
  } catch (e) {
    console.error('Failed to add torch to token:', e)
  }

  closeContextMenu()
}

// Check if a token is dead
function isTokenDead(tokenId: string): boolean {
  return deadTokenIds.value.includes(tokenId)
}

// Toggle token dead state
function toggleTokenDead() {
  const token = contextMenu.value.token
  if (!token) return

  const index = deadTokenIds.value.indexOf(token.id)
  if (index === -1) {
    deadTokenIds.value.push(token.id)
  } else {
    deadTokenIds.value.splice(index, 1)
  }

  // Sync to player display
  sendTokensToDisplay()

  closeContextMenu()
}

// Handle quick-add token
// Add all player characters to map
async function addAllPCsToMap() {
  if (!props.mapId || !props.campaignId) return

  addingPCs.value = true

  try {
    // Fetch PCs for the campaign
    await characterStore.fetchPcs(props.campaignId)

    // Get PCs from store (is_npc === 0 means PC)
    const pcs = characterStore.characters.filter(c => c.campaign_id === props.campaignId && c.is_npc === 0)

    if (pcs.length === 0) {
      console.log('No player characters found for campaign')
      return
    }

    // Place PCs in upper left corner in a 2-column formation
    // Start at grid position (1, 1) to give a small margin
    const startGridX = 1
    const startGridY = 1
    const columns = 2

    for (let i = 0; i < pcs.length; i++) {
      const pc = pcs[i]
      const col = i % columns
      const row = Math.floor(i / columns)
      const gridX = startGridX + col
      const gridY = startGridY + row

      try {
        const response = await invoke<{ success: boolean; data?: BackendToken; error?: string }>('create_token', {
          request: {
            mapId: props.mapId,
            gridX,
            gridY,
            label: pc.name,
            factionColor: '#4CAF50', // Green for PCs
            hidden: false
          }
        })

        if (response.success && response.data) {
          tokens.value.push(transformToken(response.data))
        } else {
          console.error('Failed to create PC token:', response.error)
        }
      } catch (e) {
        console.error(`Failed to create token for ${pc.name}:`, e)
      }
    }

    // Sync all tokens to player display
    sendTokensToDisplay()
    // Update fog if hiding is active
    if (!revealMap.value) {
      sendFogToDisplay()
    }
  } catch (e) {
    console.error('Failed to add PCs to map:', e)
  } finally {
    addingPCs.value = false
  }
}

// Handle token drag start
function handleTokenDragStart(event: MouseEvent, token: Token) {
  draggingTokenId.value = token.id
  selectedTokenId.value = token.id
  dragOffset.value = { x: 0, y: 0 }
  dragStartPos.value = {
    x: event.clientX,
    y: event.clientY,
    tokenX: token.x,
    tokenY: token.y
  }

  // Add document-level listeners for drag
  document.addEventListener('mousemove', handleTokenDrag)
  document.addEventListener('mouseup', handleTokenDragEnd)
}

// Handle token drag movement
function handleTokenDrag(event: MouseEvent) {
  if (!draggingTokenId.value || !dragStartPos.value) return

  const deltaX = (event.clientX - dragStartPos.value.x) / zoom.value
  const deltaY = (event.clientY - dragStartPos.value.y) / zoom.value

  // Mark as moved if we've dragged more than 5px
  if (Math.abs(deltaX) > 5 || Math.abs(deltaY) > 5) {
    tokenRendererRef.value?.setHasMoved(true)
  }

  dragOffset.value = { x: deltaX, y: deltaY }

  // Send live position update to player display for visible tokens
  const token = tokens.value.find(t => t.id === draggingTokenId.value)
  if (token?.visible_to_players) {
    sendTokensToDisplayWithDragOffset()
  }
}

// Handle token drag end
async function handleTokenDragEnd(event: MouseEvent) {
  document.removeEventListener('mousemove', handleTokenDrag)
  document.removeEventListener('mouseup', handleTokenDragEnd)

  if (!draggingTokenId.value || !dragStartPos.value || !dragOffset.value) {
    draggingTokenId.value = null
    dragOffset.value = null
    dragStartPos.value = null
    return
  }

  const token = tokens.value.find(t => t.id === draggingTokenId.value)
  if (!token) {
    draggingTokenId.value = null
    dragOffset.value = null
    dragStartPos.value = null
    return
  }

  // Only process if we actually moved (more than 5px in any direction)
  const didMove = Math.abs(dragOffset.value.x) > 5 || Math.abs(dragOffset.value.y) > 5
  if (!didMove) {
    draggingTokenId.value = null
    dragOffset.value = null
    dragStartPos.value = null
    return
  }

  // Calculate new position with grid snapping
  const rawX = dragStartPos.value.tokenX + dragOffset.value.x
  const rawY = dragStartPos.value.tokenY + dragOffset.value.y
  const { x: snappedX, y: snappedY } = snapToGrid(rawX, rawY)

  // Only update if position changed
  if (snappedX !== token.x || snappedY !== token.y) {
    // Convert pixel coordinates to grid coordinates
    const gridSize = effectiveGridSize.value
    const gridX = Math.floor(snappedX / gridSize)
    const gridY = Math.floor(snappedY / gridSize)

    try {
      const response = await invoke<{ success: boolean; data?: BackendToken; error?: string }>('update_token_position', {
        id: token.id,
        gridX,
        gridY
      })

      if (response.success && response.data) {
        // Update local token position from response (backend returns pixel coords)
        const updated = transformToken(response.data)
        token.x = updated.x
        token.y = updated.y
        // Sync to player display
        sendTokensToDisplay()
        // Update fog/visibility when hiding is active (revealMap OFF)
        if (!revealMap.value) {
          sendFogToDisplay()
        }
      } else {
        console.error('Failed to update token position:', response.error)
      }
    } catch (e) {
      console.error('Failed to update token position:', e)
    }
  }

  // Clear drag state
  draggingTokenId.value = null
  dragOffset.value = null
  dragStartPos.value = null
}

// Send tokens with live drag offset for smooth player display updates
async function sendTokensToDisplayWithDragOffset() {
  if (!isDisplayOpen.value || !props.mapId) return

  const visibleTokens = tokens.value
    .filter(t => t.visible_to_players)
    .map(t => {
      if (t.id === draggingTokenId.value && dragStartPos.value && dragOffset.value) {
        // Apply drag offset to the dragging token
        return {
          ...t,
          x: dragStartPos.value.tokenX + dragOffset.value.x,
          y: dragStartPos.value.tokenY + dragOffset.value.y
        }
      }
      return t
    })

  try {
    await emit('player-display:tokens-update', {
      mapId: props.mapId,
      tokens: visibleTokens
    })
  } catch (e) {
    console.error('Failed to send tokens to display:', e)
  }
}

// Snap position to grid center
function snapToGrid(x: number, y: number): { x: number; y: number } {
  const gridSize = effectiveGridSize.value
  const offsetX = effectiveGridOffsetX.value
  const offsetY = effectiveGridOffsetY.value

  // Snap to nearest grid cell center
  const gridX = Math.round((x - offsetX) / gridSize) * gridSize + offsetX + gridSize / 2
  const gridY = Math.round((y - offsetY) / gridSize) * gridSize + offsetY + gridSize / 2

  return { x: gridX, y: gridY }
}

// Player tokens that contribute to vision (PCs only - NPCs don't reveal fog for players)
const playerTokensWithVision = computed(() => {
  return tokens.value.filter(t =>
    t.visible_to_players && t.token_type === 'pc'
  )
})

// Calculate vision radius in pixels for a token based on ambient light
// Uses the token's vision settings (D&D 5e rules)
function getTokenVisionRadiusPx(token: Token): number {
  const gridSize = effectiveGridSize.value
  const ambient = currentAmbientLight.value

  // Convert feet to pixels (1 grid = 5 feet)
  const feetToPixels = (feet: number) => (feet / 5) * gridSize
  const UNLIMITED = 100000

  let visionFt: number | null

  switch (ambient) {
    case 'bright':
      visionFt = token.vision_bright_ft
      break
    case 'dim':
      visionFt = token.vision_dim_ft
      break
    case 'darkness':
      // In darkness, use dark vision OR own light radius (whichever is greater)
      visionFt = Math.max(token.vision_dark_ft, token.light_radius_ft)
      break
    default:
      visionFt = null
  }

  return visionFt === null ? UNLIMITED : feetToPixels(visionFt)
}

// Map state
const loading = ref(false)
const mapImageUrl = ref<string | null>(null)
const mapName = ref('')
const mapWidth = ref(0)
const mapHeight = ref(0)
const imageLoaded = ref(false)

// View state
const panX = ref(0)
const panY = ref(0)
const zoom = ref(1)
const autoSync = ref(true)

// Pan/zoom interaction state
const isPanning = ref(false)
const isZooming = ref(false)
const lastMouseX = ref(0)
const lastMouseY = ref(0)
let zoomTimeout: number | null = null

// Refs
const viewport = ref<HTMLElement | null>(null)
const mapImage = ref<HTMLImageElement | null>(null)

// Computed styles - use translate3d/scale3d for GPU compositing
const isInteracting = computed(() => isPanning.value || isZooming.value)
const mapContainerStyle = computed(() => ({
  // Use 3D transforms to force GPU layer compositing
  transform: `translate3d(${panX.value}px, ${panY.value}px, 0) scale3d(${zoom.value}, ${zoom.value}, 1)`,
  transformOrigin: 'center center',
  transition: isInteracting.value ? 'none' : 'transform 0.1s ease-out',
  willChange: 'transform',
  backfaceVisibility: 'hidden' as const
}))

// Get icon character for POI type
function getPoiIcon(icon: string): string {
  const iconMap: Record<string, string> = {
    'star': '★',
    'flag': '⚑',
    'door': '🚪',
    'chest': '📦',
    'scroll': '📜',
    'key': '🔑',
    'skull': '💀',
    'fire': '🔥',
    'water': '💧',
    'tree': '🌲',
    'mountain': '⛰',
    'house': '🏠',
    'castle': '🏰',
    'eye': '👁',
    'question': '?',
    'info': 'ℹ',
  }
  return iconMap[icon] || '●'
}

// Hex grid points calculation
const hexPoints = computed(() => {
  const size = effectiveGridSize.value
  const h = size * 0.866
  return `${size * 0.5},0 ${size},${h * 0.5} ${size},${h * 1.5} ${size * 0.5},${h * 2} 0,${h * 1.5} 0,${h * 0.5}`
})

// Load map image and tokens when mapId changes
watch(() => props.mapId, async (newId) => {
  if (newId) {
    await loadMapImage(newId)
    await loadTokens(newId)
    await loadFogState(newId)
    await loadLightSources(newId)
    await loadMapTraps(newId)
    await loadMapPois(newId)
  } else {
    mapImageUrl.value = null
    mapName.value = ''
    mapWidth.value = 0
    mapHeight.value = 0
    imageLoaded.value = false
    tokens.value = []
    fogEnabled.value = false
    lightSources.value = []
    mapTraps.value = []
    mapPois.value = []
  }
}, { immediate: true })

// Listen for state request from player display (sent after map-update is received)
let unlistenStateRequest: UnlistenFn | null = null

async function setupStateRequestListener() {
  unlistenStateRequest = await listen<{ mapId: string }>('player-display:request-state', (event) => {
    console.log('DmMapViewer: Received state request for map', event.payload.mapId)
    // Only respond if this is our current map
    if (event.payload.mapId === props.mapId) {
      sendTokensToDisplay()
      sendFogToDisplay()
      sendLightSourcesToDisplay()
    }
  })
}

// Also send state when display first opens (backup for timing issues)
watch(isDisplayOpen, async (open) => {
  if (open && props.mapId) {
    // Small delay then send - the request-state event should also trigger this
    await new Promise(resolve => setTimeout(resolve, 100))
    sendTokensToDisplay()
    sendFogToDisplay()
    sendLightSourcesToDisplay()
  }
})

async function loadMapImage(id: string) {
  console.log('DmMapViewer: Loading map with id:', id)
  loading.value = true
  imageLoaded.value = false

  try {
    // Get map details
    const mapResponse = await invoke<{ success: boolean; data?: any }>('get_map', { id })
    console.log('DmMapViewer: get_map response:', mapResponse)
    if (mapResponse.success && mapResponse.data) {
      mapName.value = mapResponse.data.name
      mapWidth.value = mapResponse.data.width_px
      mapHeight.value = mapResponse.data.height_px
    }

    // Get map image
    const imageResponse = await invoke<{ success: boolean; data?: string }>('serve_map_image', { id })
    console.log('DmMapViewer: serve_map_image response success:', imageResponse.success, 'has data:', !!imageResponse.data)
    if (imageResponse.success && imageResponse.data) {
      mapImageUrl.value = imageResponse.data
    }
  } catch (e) {
    console.error('DmMapViewer: Failed to load map:', e)
  } finally {
    loading.value = false
  }
}

function onImageLoad() {
  imageLoaded.value = true
  resetView()
}

// Zoom controls
function zoomIn() {
  zoom.value = Math.min(zoom.value * 1.25, 5)
  syncViewportIfNeeded()
}

function zoomOut() {
  zoom.value = Math.max(zoom.value / 1.25, 0.1)
  syncViewportIfNeeded()
}

function onWheel(event: WheelEvent) {
  const delta = event.deltaY > 0 ? 0.9 : 1.1
  const newZoom = Math.max(0.1, Math.min(5, zoom.value * delta))

  // Mark as zooming for smooth updates
  isZooming.value = true
  if (zoomTimeout) clearTimeout(zoomTimeout)
  zoomTimeout = window.setTimeout(() => {
    isZooming.value = false
  }, 150)

  // Zoom toward mouse position
  if (viewport.value) {
    const rect = viewport.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left - rect.width / 2
    const mouseY = event.clientY - rect.top - rect.height / 2

    const zoomRatio = newZoom / zoom.value
    panX.value = mouseX - (mouseX - panX.value) * zoomRatio
    panY.value = mouseY - (mouseY - panY.value) * zoomRatio
  }

  zoom.value = newZoom
  throttledSync()
}

function resetView() {
  panX.value = 0
  panY.value = 0
  zoom.value = 1
  syncViewportIfNeeded()
}

// Convert screen coordinates to map coordinates
function screenToMapCoords(clientX: number, clientY: number): { x: number; y: number } {
  const rect = viewport.value?.getBoundingClientRect()
  if (!rect) return { x: 0, y: 0 }

  const screenX = clientX - rect.left - rect.width / 2
  const screenY = clientY - rect.top - rect.height / 2

  const mapX = (screenX - panX.value) / zoom.value
  const mapY = (screenY - panY.value) / zoom.value

  return { x: mapX, y: mapY }
}

// Pan controls
function startPan(event: MouseEvent) {
  if (event.button !== 0) return // Only left click

  isPanning.value = true
  lastMouseX.value = event.clientX
  lastMouseY.value = event.clientY
}

function onPan(event: MouseEvent) {
  if (!isPanning.value) return

  const deltaX = event.clientX - lastMouseX.value
  const deltaY = event.clientY - lastMouseY.value

  panX.value += deltaX
  panY.value += deltaY

  lastMouseX.value = event.clientX
  lastMouseY.value = event.clientY

  // Use throttled sync during panning for smoothness
  throttledSync()
}

function endPan() {
  if (isPanning.value) {
    isPanning.value = false
    // Final sync to ensure we capture the end position
    syncViewportIfNeeded()
  }
}

// Sync controls
function toggleAutoSync() {
  autoSync.value = !autoSync.value
  if (autoSync.value) {
    syncViewportIfNeeded()
  }
}

function syncViewportIfNeeded() {
  if (autoSync.value && isDisplayOpen.value) {
    pushViewport()
  }
}

// Throttled sync for smooth panning - only sync every 50ms during drag
const throttledSync = throttle(() => {
  syncViewportIfNeeded()
}, 50)

async function pushViewport() {
  if (!isDisplayOpen.value) return

  try {
    // Convert pan coordinates to normalized values
    // The player display expects x, y as offsets from center
    await updateViewport(panX.value, panY.value, zoom.value)
  } catch (e) {
    console.error('Failed to push viewport:', e)
  }
}

// Keyboard shortcuts
function handleKeydown(event: KeyboardEvent) {
  if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
    return
  }

  switch (event.key) {
    case '+':
    case '=':
      zoomIn()
      break
    case '-':
      zoomOut()
      break
    case '0':
      resetView()
      break
    case 'p':
    case 'P':
      if (!autoSync.value) {
        pushViewport()
      }
      break
    case 'h':
    case 'H':
      // Toggle visibility of selected token
      if (selectedTokenId.value) {
        toggleSelectedTokenVisibility()
      }
      break
    case 'l':
    case 'L':
      // Toggle light or add torch to selected token
      if (selectedTokenId.value) {
        const light = getTokenLightSource(selectedTokenId.value)
        if (light) {
          // Set context menu token for toggleTokenLight
          contextMenu.value.token = tokens.value.find(t => t.id === selectedTokenId.value) || null
          toggleTokenLight()
        } else {
          contextMenu.value.token = tokens.value.find(t => t.id === selectedTokenId.value) || null
          addTorchToToken()
        }
      }
      break
    case 'd':
    case 'D':
      // Toggle dead state for selected token
      if (selectedTokenId.value) {
        contextMenu.value.token = tokens.value.find(t => t.id === selectedTokenId.value) || null
        toggleTokenDead()
      }
      break
    case 'Escape':
      // Close context menu and deselect
      closeContextMenu()
      selectedTokenId.value = null
      break
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  await setupStateRequestListener()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  // Clean up any lingering drag listeners
  document.removeEventListener('mousemove', handleTokenDrag)
  document.removeEventListener('mouseup', handleTokenDragEnd)
  // Clean up event listener
  unlistenStateRequest?.()
})
</script>

<style scoped>
.dm-map-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-base-200);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.viewer-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.toolbar-label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  font-weight: 500;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-background);
  color: var(--color-text);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.toolbar-btn:hover:not(:disabled) {
  background: var(--color-base-200);
  border-color: var(--color-primary-500);
}

.toolbar-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toolbar-btn svg {
  width: 16px;
  height: 16px;
}

.ambient-select {
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-background);
  color: var(--color-text);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.ambient-select:hover:not(:disabled) {
  border-color: var(--color-primary-500);
}

.ambient-select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sync-btn.active {
  background: var(--color-primary-100);
  border-color: var(--color-primary-500);
  color: var(--color-primary-700);
}

.push-btn {
  background: var(--color-primary-500);
  border-color: var(--color-primary-500);
  color: white;
}

.push-btn:hover:not(:disabled) {
  background: var(--color-primary-600);
  border-color: var(--color-primary-600);
}

.print-btn {
  background: var(--color-base-100);
  border-color: var(--color-border);
}

.print-btn:hover:not(:disabled) {
  background: var(--color-base-200);
  border-color: var(--color-primary-400);
}

.zoom-level {
  font-size: 0.75rem;
  font-family: monospace;
  min-width: 40px;
  text-align: center;
  color: var(--color-text);
}

.toggle-switch {
  position: relative;
  width: 40px;
  height: 22px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--color-base-300);
  border-radius: 22px;
  transition: background-color 0.2s ease;
}

.toggle-slider::before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  border-radius: 50%;
  transition: transform 0.2s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.toggle-switch.active .toggle-slider {
  background-color: var(--color-primary-500);
}

.toggle-switch.active .toggle-slider::before {
  transform: translateX(18px);
}

.toggle-switch:hover .toggle-slider {
  background-color: var(--color-base-400);
}

.toggle-switch.active:hover .toggle-slider {
  background-color: var(--color-primary-600);
}

/* Reveal Map Button (red eye icon) */
.reveal-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid rgba(220, 38, 38, 0.4);
  border-radius: var(--radius-sm);
  background: rgba(220, 38, 38, 0.1);
  color: #dc2626;
  cursor: pointer;
  transition: all 0.15s ease;
}

.reveal-btn svg {
  width: 16px;
  height: 16px;
}

.reveal-btn:hover:not(:disabled) {
  background: rgba(220, 38, 38, 0.2);
  border-color: #dc2626;
}

.reveal-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.reveal-btn.active {
  background: #dc2626;
  border-color: #dc2626;
  color: white;
  animation: pulse-danger 1.5s ease-in-out infinite;
}

@keyframes pulse-danger {
  0%, 100% { box-shadow: 0 0 0 0 rgba(220, 38, 38, 0.4); }
  50% { box-shadow: 0 0 0 4px rgba(220, 38, 38, 0); }
}

/* Button Group (Fog/Token selector) */
.btn-group {
  display: flex;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.btn-group-item {
  padding: 4px 10px;
  border: none;
  background: var(--color-background);
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-group-item:not(:last-child) {
  border-right: 1px solid var(--color-border);
}

.btn-group-item:hover:not(:disabled):not(.active) {
  background: var(--color-base-200);
  color: var(--color-text);
}

.btn-group-item.active {
  background: var(--color-primary-500);
  color: white;
}

.btn-group-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.map-viewport {
  flex: 1;
  overflow: hidden;
  position: relative;
  cursor: grab;
  display: flex;
  align-items: center;
  justify-content: center;
}

.map-viewport:active {
  cursor: grabbing;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-muted);
  font-size: 0.875rem;
}

.map-container {
  position: relative;
  /* transition and will-change handled dynamically in computed style */
  /* Force GPU layer for the container */
  transform-style: preserve-3d;
  perspective: 1000px;
}

.map-image {
  display: block;
  max-width: none;
  user-select: none;
  -webkit-user-drag: none;
  /* GPU optimizations for large images */
  will-change: transform;
  backface-visibility: hidden;
  image-rendering: auto;
  /* Prevent layout recalculations */
  contain: layout style paint;
}

.grid-overlay {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  /* GPU layer for grid */
  will-change: transform;
  backface-visibility: hidden;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-xs) var(--spacing-md);
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  font-size: 0.75rem;
  color: var(--color-text);
  flex-shrink: 0;
}

.status-bar .dim {
  color: var(--color-text-muted);
}

.status-indicator {
  margin-left: auto;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: 0.625rem;
  font-weight: 600;
  text-transform: uppercase;
}

.status-indicator.connected {
  background: var(--color-success-100);
  color: var(--color-success);
}

.status-indicator.disconnected {
  background: var(--color-base-200);
  color: var(--color-text-muted);
}

/* Context Menu */
.context-menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

.context-menu {
  position: fixed;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  min-width: 180px;
  padding: var(--spacing-xs) 0;
}

.context-menu button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: none;
  background: none;
  color: var(--color-text);
  text-align: left;
  cursor: pointer;
  font-size: 0.875rem;
}

.context-menu button:hover {
  background: var(--color-base-200);
}

.context-menu button.danger {
  color: var(--color-error);
}

.context-menu button.danger:hover {
  background: var(--color-error-100);
}

.context-menu .shortcut {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  font-family: monospace;
  background: var(--color-base-200);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}

.context-menu-divider {
  height: 1px;
  background: var(--color-border);
  margin: var(--spacing-xs) 0;
}

.context-menu button.light-option {
  color: #fbbf24;
}

.context-menu button.light-option:hover {
  background: rgba(251, 191, 36, 0.1);
}

.context-menu button.dead-option {
  color: #dc2626;
}

.context-menu button.dead-option:hover {
  background: rgba(220, 38, 38, 0.1);
}

/* Fog of War Controls */
.fog-controls {
  border-left: 1px solid var(--color-border);
  padding-left: var(--spacing-md);
  margin-left: var(--spacing-sm);
}

/* LOS Controls */
.los-controls {
  border-left: 1px solid var(--color-border);
  padding-left: var(--spacing-md);
  margin-left: var(--spacing-sm);
}

.los-controls .toolbar-btn.active {
  background: var(--color-warning-100);
  border-color: var(--color-warning);
  color: var(--color-warning);
}

/* Fog Overlay */
.fog-overlay {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 10; /* Above map, below lights (15) */
  pointer-events: none;
  will-change: transform;
  backface-visibility: hidden;
}

.fog-overlay.dm-fog {
  /* DM view - semi-transparent so DM can see hidden areas */
  opacity: 1;
}

/* Map Markers Overlay (Traps & POIs) */
.markers-overlay {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  will-change: transform;
  backface-visibility: hidden;
}

.trap-marker,
.poi-marker {
  cursor: pointer;
  pointer-events: all;
  transition: transform 0.15s ease;
}

.trap-marker:hover,
.poi-marker:hover {
  transform: scale(1.15);
}

.trap-marker.selected,
.poi-marker.selected {
  transform: scale(1.2);
}

.marker-label {
  paint-order: stroke fill;
  stroke: rgba(0, 0, 0, 0.7);
  stroke-width: 3px;
  font-weight: 600;
}
</style>
