<template>
  <div class="play-mode-layout">
    <!-- Play Mode Header -->
    <header class="play-header">
      <div class="header-left">
        <button class="back-to-prep-button" @click="handleBackToPrep">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="icon">
            <path fill-rule="evenodd" d="M17 10a.75.75 0 01-.75.75H5.612l4.158 3.96a.75.75 0 11-1.04 1.08l-5.5-5.25a.75.75 0 010-1.08l5.5-5.25a.75.75 0 111.04 1.08L5.612 9.25H16.25A.75.75 0 0117 10z" clip-rule="evenodd" />
          </svg>
          Back to Prep
        </button>
      </div>

      <div class="header-center">
        <h1 class="module-name">{{ module?.name || 'Loading...' }}</h1>
        <div class="play-mode-badge">PLAY MODE</div>
      </div>

      <div class="header-right">
        <!-- Player Display Controls -->
        <div class="display-controls">
          <button
            class="display-button"
            :class="{ active: isDisplayOpen }"
            @click="togglePlayerDisplay"
            title="Open/Close Player Display"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="icon">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 17.25v1.007a3 3 0 01-.879 2.122L7.5 21h9l-.621-.621A3 3 0 0115 18.257V17.25m6-12V15a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 15V5.25m18 0A2.25 2.25 0 0018.75 3H5.25A2.25 2.25 0 003 5.25m18 0V12a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 12V5.25" />
            </svg>
            <span>{{ isDisplayOpen ? 'Display Open' : 'Player Display' }}</span>
          </button>
          <button
            v-if="isDisplayOpen"
            class="blackout-button"
            :class="{ active: isBlackout }"
            @click="handleBlackoutToggle"
            title="Toggle Blackout"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="icon">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
            </svg>
          </button>
        </div>
        <button class="end-session-button" @click="handleEndSession">
          End Session
        </button>
      </div>
    </header>

    <!-- Main Play Area -->
    <div class="play-content">
      <!-- Collapsible Sidebar -->
      <aside class="play-sidebar" :class="{ collapsed: sidebarCollapsed }">
        <button class="sidebar-toggle" @click="toggleSidebar">
          {{ sidebarCollapsed ? '&raquo;' : '&laquo;' }}
        </button>

        <div class="sidebar-content" v-show="!sidebarCollapsed">
          <!-- Monsters Section -->
          <div class="sidebar-section">
            <h3>Monsters{{ !encountersLoading ? ` (${allMonsters.length})` : '' }}</h3>
            <div v-if="encountersLoading" class="loading-text">Loading...</div>
            <div v-else-if="allMonsters.length === 0" class="empty-text">No monsters added</div>
            <div v-else class="monster-quick-list">
              <div
                v-for="monster in allMonsters"
                :key="monster.id"
                class="monster-quick-item"
                :class="{ active: selectedMonster?.id === monster.id }"
                @click="selectMonsterAndShowTab(monster)"
              >
                <span class="monster-qty">{{ monster.quantity }}×</span>
                <span class="monster-name-text">{{ monster.monster_name }}</span>
                <span v-if="monster.encounter_tag" class="monster-tag">{{ monster.encounter_tag }}</span>
              </div>
            </div>
          </div>

          <!-- Maps Section -->
          <div class="sidebar-section">
            <h3>Maps</h3>
            <div v-if="mapsLoading" class="loading-text">Loading...</div>
            <div v-else-if="allMaps.length === 0" class="empty-text">No maps available</div>
            <div v-else class="map-list">
              <div
                v-for="map in allMaps"
                :key="map.id"
                class="map-item"
                :class="{ active: activeMapId === map.id }"
                @click="sendMapToDisplay(map)"
              >
                <div class="map-item-info">
                  <span class="map-item-name">{{ map.name }}</span>
                  <span class="map-item-meta">{{ map.module_name || 'Campaign' }}</span>
                </div>
                <div v-if="activeMapId === map.id" class="map-active-indicator">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="active-icon">
                    <path d="M6.3 2.841A1.5 1.5 0 004 4.11V15.89a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
                  </svg>
                </div>
              </div>
            </div>
          </div>
        </div>
      </aside>

      <!-- Main Content Area with Notes Panel -->
      <div class="main-wrapper">
        <main class="play-main" :class="{ 'notes-expanded': !notesCollapsed }">
          <!-- Combat Layout - Map + Monster Panel -->
          <div class="combat-layout" :class="{ 'monster-panel-open': selectedMonster && monsterPanelOpen }">
              <!-- Map Area -->
              <div class="map-area">
                <DmMapViewer
                  :map-id="activeMapId"
                  :grid-type="activeMap?.grid_type"
                  :grid-size-px="activeMap?.grid_size_px"
                  :grid-offset-x="activeMap?.grid_offset_x"
                  :grid-offset-y="activeMap?.grid_offset_y"
                  :show-grid="true"
                  :campaign-id="campaign?.id"
                  :module-id="activeMap?.module_id"
                  :uvtt-file-path="activeMap?.image_path"
                />
              </div>

              <!-- Monster Panel (slides in from right when monster selected) -->
              <MonsterStatsPanel
                v-if="selectedMonster"
                :monster="selectedMonster"
                v-model:panelOpen="monsterPanelOpen"
                @close="clearSelectedMonster"
              />
            </div>
        </main>

        <!-- Collapsible Notes Panel -->
        <aside class="notes-panel" :class="{ collapsed: notesCollapsed }">
          <button class="notes-toggle" @click="toggleNotes">
            <span class="notes-toggle-icon">{{ notesCollapsed ? '&#9650;' : '&#9660;' }}</span>
            <span class="notes-toggle-label">Session Notes</span>
            <span v-if="notesSaving" class="notes-saving">Saving...</span>
            <span v-else-if="notesLastSaved" class="notes-saved">Saved</span>
          </button>

          <div class="notes-content" v-show="!notesCollapsed">
            <textarea
              v-model="notesContent"
              class="notes-textarea"
              placeholder="Type your session notes here... (auto-saves)"
              @input="handleNotesInput"
            ></textarea>
          </div>
        </aside>
      </div>
    </div>

    <!-- Cross-reference tooltip -->
    <div
      v-if="tooltipVisible"
      class="cross-ref-tooltip"
      :style="{ left: `${tooltipPosition.x + 10}px`, top: `${tooltipPosition.y + 10}px` }"
      v-html="tooltipContent"
    />

    <!-- Cross-reference modal -->
    <AppModal
      :visible="modalContent.visible"
      :title="modalContent.title"
      size="md"
      @close="closeModal"
    >
      <div class="dnd-content" v-html="modalContent.content"></div>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { usePlayerDisplay } from '@/composables/usePlayerDisplay'
import DmMapViewer from '@/components/DmMapViewer.vue'
import AppModal from '@/components/shared/AppModal.vue'
import MonsterStatsPanel from '../components/MonsterStatsPanel.vue'
import type { Module, Campaign } from '@/types'
import { useCrossReferences } from '@/features/sources/composables/useCrossReferences'
import { useModuleMonsters } from '../composables/useModuleMonsters'
import { useModuleMaps } from '../composables/useModuleMaps'
import { useSessionNotes, buildNotesFilePath } from '../composables/useSessionNotes'

const route = useRoute()
const router = useRouter()

// Player display state
const {
  isDisplayOpen,
  isBlackout,
  toggleDisplay,
  toggleBlackout,
  checkDisplayOpen
} = usePlayerDisplay()

// Cross-reference handling for clickable links in monster stats
const {
  tooltipContent,
  tooltipVisible,
  tooltipPosition,
  modalContent,
  handleCrossRefHover,
  handleCrossRefClick,
  hideTooltip,
  closeModal
} = useCrossReferences()

// Toggle player display window
async function togglePlayerDisplay() {
  try {
    const wasOpen = isDisplayOpen.value
    await toggleDisplay()

    // If we just opened the display and have an active map, send it after a short delay
    // (the display window needs time to mount and set up event listeners)
    if (!wasOpen && isDisplayOpen.value && activeMapId.value) {
      const map = activeMap.value
      console.log('ModulePlayView: Display just opened, will send map after delay:', map)
      if (map) {
        setTimeout(async () => {
          try {
            console.log('ModulePlayView: Sending map to display:', map.id)
            await invoke('send_map_to_display', {
              mapId: map.id,
              gridType: map.grid_type,
              gridSizePx: map.grid_size_px,
              gridOffsetX: map.grid_offset_x,
              gridOffsetY: map.grid_offset_y
            })
            console.log('ModulePlayView: Map sent successfully')
          } catch (err) {
            console.error('Failed to send map after display open:', err)
          }
        }, 500) // Give the window time to initialize
      }
    }
  } catch (err) {
    console.error('Failed to toggle player display:', err)
  }
}

// Toggle blackout mode
async function handleBlackoutToggle() {
  try {
    await toggleBlackout()
  } catch (err) {
    console.error('Failed to toggle blackout:', err)
  }
}

const moduleId = computed(() => parseInt(route.params.id as string))
const module = ref<Module | null>(null)
const campaign = ref<Campaign | null>(null)
const sidebarCollapsed = ref(false)

// Monster/Encounter state (from composable)
const {
  encounterGroups,
  allMonsters,
  selectedEncounter,
  selectedMonster,
  encountersLoading,
  loadEncounters,
  selectEncounter,
  selectMonster,
  clearSelectedMonster
} = useModuleMonsters(moduleId)

// Monster panel state
const monsterPanelOpen = ref(true)

// Campaign ID computed for maps composable
const campaignId = computed(() => campaign.value?.id)

// Map state (from composable)
const {
  allMaps,
  mapsLoading,
  activeMapId,
  activeMap,
  loadMaps,
  sendMapToPlayerDisplay
} = useModuleMaps({
  moduleId,
  campaignId,
  isDisplayOpen
})

// Select a map and send to display
async function sendMapToDisplay(map: any) {
  await sendMapToPlayerDisplay(map)
}

// Select a monster and open the stats panel
function selectMonsterAndShowTab(monster: any) {
  selectMonster(monster)
  monsterPanelOpen.value = true
}

// Session notes (from composable)
const {
  notesCollapsed,
  notesContent,
  notesSaving,
  notesLastSaved,
  toggleNotes,
  setNotesFilePath,
  loadNotes,
  handleNotesInput
} = useSessionNotes()

// Load module and campaign data
async function loadModule() {
  try {
    const response = await invoke<{ data: Module }>('get_module', {
      id: moduleId.value
    })
    module.value = response.data

    // Load campaign to get directory path
    if (module.value?.campaign_id) {
      const campaignResponse = await invoke<{ data: Campaign }>('get_campaign', {
        id: module.value.campaign_id
      })
      campaign.value = campaignResponse.data

      // Build notes file path and load notes
      if (campaign.value?.directory_path && module.value) {
        const moduleNumber = (module.value as any).module_number || 1
        setNotesFilePath(buildNotesFilePath(campaign.value.directory_path, moduleNumber))
        await loadNotes()
      }
    }
  } catch (error) {
    console.error('Failed to load module:', error)
  }
}

// Check if we're in dashboard context
const isInDashboard = computed(() => {
  return route.path.includes('/dashboard/')
})

// Navigation
function handleBackToPrep() {
  // Navigate back to module prep view
  if (isInDashboard.value && campaign.value?.id) {
    // Dashboard context: go to modules tab (with query param to auto-select)
    router.push({
      path: `/campaigns/${campaign.value.id}/dashboard/modules`,
      query: { select: moduleId.value }
    })
  } else {
    // Legacy: go to standalone module board
    router.push({ name: 'module-board', params: { id: moduleId.value } })
  }
}

function handleEndSession() {
  // Navigate back to modules tab or campaign board
  if (isInDashboard.value && campaign.value?.id) {
    // Dashboard context: go back to modules tab
    router.push({
      path: `/campaigns/${campaign.value.id}/dashboard/modules`,
      query: { select: moduleId.value }
    })
  } else if (campaign.value?.id) {
    // Legacy: go to campaign board
    router.push({ name: 'campaign-board', params: { id: campaign.value.id } })
  } else {
    // Fallback to campaigns list
    router.push({ name: 'campaigns' })
  }
}

// Sidebar
function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

// Cleanup
onBeforeUnmount(() => {
  // Note: Session notes cleanup is handled by useSessionNotes composable
  // Clean up cross-reference handlers
  document.removeEventListener('click', handleCrossRefClick as any)
  document.removeEventListener('mouseover', handleCrossRefHover as any)
  document.removeEventListener('mouseout', hideTooltip)
})

onMounted(async () => {
  await loadModule()
  await Promise.all([
    loadEncounters(),
    loadMaps()
  ])

  // Set up cross-reference click handlers for monster stats
  document.addEventListener('click', handleCrossRefClick as any)
  document.addEventListener('mouseover', handleCrossRefHover as any)
  document.addEventListener('mouseout', (e) => {
    const target = e.target as HTMLElement
    if (target.classList?.contains('cross-ref-link')) {
      hideTooltip()
    }
  })
})
</script>

<style scoped>
.play-mode-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
  overflow: hidden;
}

/* Header Styles */
.play-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1.5rem;
  background: var(--color-surface);
  border-bottom: 2px solid var(--color-warning);
  box-shadow: var(--shadow-sm);
}

.header-left,
.header-right {
  flex: 1;
}

.header-left {
  display: flex;
  align-items: center;
}

.header-right {
  display: flex;
  justify-content: flex-end;
}

.back-to-prep-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-base);
}

.back-to-prep-button:hover {
  background: var(--color-surface-variant);
  color: var(--color-text);
  border-color: var(--color-text-secondary);
}

.back-to-prep-button .icon {
  width: 16px;
  height: 16px;
}

.header-center {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.module-name {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0;
  color: var(--color-text);
}

.play-mode-badge {
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  padding: 0.2rem 0.6rem;
  background: var(--color-warning);
  color: var(--color-background);
  border-radius: var(--radius-sm);
}

/* Display Controls */
.display-controls {
  display: flex;
  gap: 0.5rem;
  margin-right: 1rem;
}

.display-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-base);
}

.display-button:hover {
  background: var(--color-surface-variant);
  border-color: var(--color-primary-500);
}

.display-button.active {
  background: var(--color-success);
  border-color: var(--color-success);
  color: var(--color-background);
}

.display-button .icon {
  width: 1.25rem;
  height: 1.25rem;
}

.blackout-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-base);
}

.blackout-button:hover {
  background: var(--color-surface-variant);
  color: var(--color-text);
}

.blackout-button.active {
  background: var(--color-warning);
  border-color: var(--color-warning);
  color: var(--color-background);
}

.blackout-button .icon {
  width: 1.25rem;
  height: 1.25rem;
}

.end-session-button {
  padding: 0.5rem 1rem;
  background: var(--color-error);
  color: var(--color-background);
  border: none;
  border-radius: var(--radius-md);
  font-weight: 500;
  cursor: pointer;
  transition: background var(--transition-base);
}

.end-session-button:hover {
  background: var(--color-error-dark);
}

/* Content Area */
.play-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.main-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Sidebar Styles */
.play-sidebar {
  width: 280px;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  transition: width var(--transition-slow);
  position: relative;
}

.play-sidebar.collapsed {
  width: 40px;
}

.sidebar-toggle {
  position: absolute;
  right: -12px;
  top: 1rem;
  width: 24px;
  height: 24px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.sidebar-toggle:hover {
  background: var(--color-surface-variant);
  border-color: var(--color-primary-500);
}

.sidebar-content {
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
}

.sidebar-section {
  margin-bottom: 1.5rem;
}

.sidebar-section h3 {
  font-size: 0.875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-secondary);
  margin-bottom: 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--color-border);
}

/* Main Content */
.play-main {
  flex: 1;
  padding: 1.5rem;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-height: 0;
}

/* Content Panel */
.content-panel {
  background: var(--color-surface);
  border-radius: 0.5rem;
  padding: 1.5rem;
  border: 1px solid var(--color-border);
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.content-panel h2 {
  font-size: 1.1rem;
  font-weight: 600;
  margin: 0 0 1rem 0;
  color: var(--color-text);
}

/* Notes Panel */
.notes-panel {
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  transition: height var(--transition-slow);
  height: 250px;
  min-height: 40px;
}

.notes-panel.collapsed {
  height: 40px;
}

.notes-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: var(--color-surface-variant);
  border: none;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
  text-align: left;
  width: 100%;
}

.notes-toggle:hover {
  background: var(--color-surface-hover);
}

.notes-toggle-icon {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.notes-toggle-label {
  flex: 1;
}

.notes-saving {
  font-size: 0.75rem;
  color: var(--color-warning);
  font-style: italic;
}

.notes-saved {
  font-size: 0.75rem;
  color: var(--color-success);
}

.notes-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.notes-textarea {
  flex: 1;
  padding: 1rem;
  border: none;
  resize: none;
  font-family: inherit;
  font-size: 0.9rem;
  line-height: 1.6;
  background: var(--color-surface);
  color: var(--color-text);
  overflow-y: scroll;
}

.notes-textarea:focus {
  outline: none;
}

.notes-textarea::placeholder {
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Notes textarea scrollbar */
.notes-textarea::-webkit-scrollbar {
  width: 8px;
}

.notes-textarea::-webkit-scrollbar-track {
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.notes-textarea::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: var(--radius-sm);
}

.notes-textarea::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-secondary);
}

/* Adjust main area when notes expanded */
.play-main.notes-expanded {
  flex: 1;
  min-height: 0;
}

/* Encounter List Styles */
.loading-text,
.empty-text {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

.monster-qty {
  font-weight: 600;
  color: var(--color-warning);
  min-width: 2rem;
}

.monster-name {
  color: var(--color-text);
}

/* Monster Card Styles */
.monster-card {
  background: var(--color-surface-variant);
  border-radius: var(--radius-lg);
  padding: 1rem;
}

.monster-card-header {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  margin-bottom: 0.75rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--color-border);
}

.monster-card-header strong {
  font-size: 1rem;
  color: var(--color-text);
}

.monster-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.monster-stats {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.875rem;
}

.stat-label {
  font-weight: 600;
  color: var(--color-text-secondary);
}

.stat-value {
  color: var(--color-text);
}

/* Ability Scores Grid */
.ability-scores {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.5rem;
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--color-border);
}

.ability {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.125rem;
}

.ability-name {
  font-size: 0.65rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.ability-value {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text);
}

.no-data-text {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  font-style: italic;
  text-align: center;
  padding: 0.5rem;
}

/* Monster Traits, Actions, Reactions */
.monster-traits,
.monster-actions,
.monster-reactions {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--color-border);
}

.monster-traits h4,
.monster-actions h4,
.monster-reactions h4 {
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--color-warning);
  margin: 0 0 0.5rem 0;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.monster-action {
  font-size: 0.8rem;
  line-height: 1.4;
  margin-bottom: 0.5rem;
  color: var(--color-text);
}

.monster-action strong {
  color: var(--color-text);
}

.monster-action :deep(.hit-bonus),
.monster-action :deep(.damage-roll),
.monster-action :deep(.dice-roll) {
  color: var(--color-primary-500);
  font-weight: 600;
}

.monster-action :deep(.cross-ref-link) {
  color: var(--color-primary-500);
  cursor: pointer;
}

/* Legendary Actions */
.monster-legendary {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--color-border);
}

.monster-legendary h4 {
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--color-secondary); /* Purple for legendary */
  margin: 0 0 0.5rem 0;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.legendary-intro {
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-secondary);
  margin: 0 0 0.5rem 0;
  line-height: 1.3;
}

/* Spellcasting */
.monster-spellcasting {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--color-border);
}

.monster-spellcasting h4 {
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--color-info); /* Blue for spellcasting */
  margin: 0 0 0.5rem 0;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.monster-spellcasting p {
  font-size: 0.8rem;
  margin: 0.25rem 0;
  line-height: 1.4;
}

.monster-spellcasting .spell-slots,
.monster-spellcasting .innate-spells {
  margin-top: 0.5rem;
}

/* Monster Quick List - Sidebar */
.monster-quick-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.monster-quick-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  color: var(--color-text);
  background: var(--color-surface-variant);
  transition: all var(--transition-fast);
  border-left: 3px solid transparent;
}

.monster-quick-item:hover {
  background: var(--color-surface-hover);
  border-left-color: var(--color-warning);
}

.monster-quick-item.active {
  background: var(--color-surface-hover);
  border-left-color: var(--color-primary-500);
}

.monster-quick-item .monster-qty {
  font-weight: 700;
  color: var(--color-warning);
  min-width: 1.5rem;
}

.monster-quick-item .monster-name-text {
  flex: 1;
  font-weight: 500;
}

.monster-quick-item .monster-tag {
  font-size: 0.65rem;
  padding: 0.1rem 0.4rem;
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

/* Map List */
.map-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.map-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  border: 1px solid transparent;
}

.map-item:hover {
  background: var(--color-surface-variant);
}

.map-item.active {
  background: var(--color-primary-100);
  border-color: var(--color-primary-500);
}

.map-item-info {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
  flex: 1;
}

.map-item-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.map-item-meta {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.map-active-indicator {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary-500);
}

.active-icon {
  width: 16px;
  height: 16px;
}

/* Map Viewer Container */
.map-viewer-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

/* Cross-reference tooltip */
.cross-ref-tooltip {
  position: fixed;
  z-index: 9999;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: 0.5rem 0.75rem;
  font-size: 0.85rem;
  color: var(--color-text);
  max-width: 300px;
  box-shadow: var(--shadow-lg);
  pointer-events: none;
}


/* ============================================
   COMBAT LAYOUT - Map + Monster Panel
   ============================================ */

.combat-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.map-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
  transition: margin-right var(--transition-slow);
}

.combat-layout.monster-panel-open .map-area {
  margin-right: 0;
}
</style>
