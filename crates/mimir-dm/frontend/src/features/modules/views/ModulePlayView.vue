<template>
  <div class="play-mode-layout">
    <!-- Play Mode Header -->
    <header class="play-header">
      <div class="header-left">
        <!-- Empty left section for layout balance -->
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
          <!-- Encounters Section -->
          <div class="sidebar-section">
            <h3>Encounters</h3>
            <div v-if="encountersLoading" class="loading-text">Loading...</div>
            <div v-else-if="encounterGroups.length === 0" class="empty-text">No encounters tagged</div>
            <div v-else class="encounter-list">
              <div
                v-for="group in encounterGroups"
                :key="group.encounter_tag || 'untagged'"
                class="encounter-group"
                :class="{ active: selectedEncounter === group.encounter_tag }"
                @click="selectEncounter(group)"
              >
                <div class="encounter-header">
                  <span class="encounter-name">{{ group.encounter_tag || 'Untagged' }}</span>
                  <span class="encounter-count">{{ group.monsters.length }}</span>
                </div>
                <div class="encounter-monsters" v-if="selectedEncounter === group.encounter_tag">
                  <div
                    v-for="monster in group.monsters"
                    :key="monster.id"
                    class="monster-item"
                    @click.stop="selectMonster(monster)"
                  >
                    <span class="monster-qty">{{ monster.quantity }}x</span>
                    <span class="monster-name">{{ monster.monster_name }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- All Monsters List (sidebar stays simple) -->
          <div class="sidebar-section" v-if="allMonsters.length > 0">
            <h3>Monsters ({{ allMonsters.length }})</h3>
            <div class="monster-quick-list">
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
          <!-- View Mode Tabs -->
          <div class="view-mode-tabs">
            <button
              class="view-mode-tab"
              :class="{ active: viewMode === 'documents' }"
              @click="viewMode = 'documents'"
            >
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="tab-icon">
                <path fill-rule="evenodd" d="M4.5 2A1.5 1.5 0 003 3.5v13A1.5 1.5 0 004.5 18h11a1.5 1.5 0 001.5-1.5V7.621a1.5 1.5 0 00-.44-1.06l-4.12-4.122A1.5 1.5 0 0011.378 2H4.5zm2.25 8.5a.75.75 0 000 1.5h6.5a.75.75 0 000-1.5h-6.5zm0 3a.75.75 0 000 1.5h6.5a.75.75 0 000-1.5h-6.5z" clip-rule="evenodd" />
              </svg>
              Documents
            </button>
            <button
              class="view-mode-tab"
              :class="{ active: viewMode === 'map', 'has-active': activeMapId !== null }"
              @click="viewMode = 'map'"
            >
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="tab-icon">
                <path fill-rule="evenodd" d="M8.157 2.175a1.5 1.5 0 00-1.147 0l-4.084 1.69A1.5 1.5 0 002 5.251v10.877a1.5 1.5 0 002.074 1.386l3.51-1.453 4.26 1.763a1.5 1.5 0 001.146 0l4.083-1.69A1.5 1.5 0 0018 14.748V3.873a1.5 1.5 0 00-2.073-1.386l-3.51 1.452-4.26-1.763zM7.58 5a.75.75 0 01.75.75v6.5a.75.75 0 01-1.5 0v-6.5A.75.75 0 017.58 5zm5.59 2.75a.75.75 0 00-1.5 0v6.5a.75.75 0 001.5 0v-6.5z" clip-rule="evenodd" />
              </svg>
              Combat
              <span v-if="activeMapId" class="active-indicator"></span>
            </button>
          </div>

          <!-- Documents View Mode -->
          <template v-if="viewMode === 'documents'">
            <!-- Document Tabs -->
            <div class="document-tabs" v-if="documents.length > 0">
              <button
                v-for="doc in documents"
                :key="doc.id"
                class="doc-tab"
                :class="{ active: selectedDocument?.id === doc.id }"
                @click="selectDocument(doc)"
              >
                {{ doc.title }}
              </button>
            </div>

            <!-- Document Viewer -->
            <div class="content-panel document-panel" v-if="selectedDocument">
              <div class="document-header">
                <h2>{{ selectedDocument.title }}</h2>
              </div>
              <div class="document-content">
                <div v-if="documentLoading" class="loading-state">
                  Loading document...
                </div>
                <div v-else-if="editor" class="prose-content">
                  <EditorContent :editor="editor" />
                </div>
              </div>
            </div>

            <!-- Fallback when no documents -->
            <div class="content-panel" v-else-if="!documentsLoading && documents.length === 0">
              <h2>Module Narrative</h2>
              <p class="placeholder-text">
                No module documents found. Create documents in the module prep view.
              </p>
            </div>

            <!-- Loading state -->
            <div class="content-panel" v-else-if="documentsLoading">
              <p class="placeholder-text">Loading documents...</p>
            </div>
          </template>

          <!-- Combat View Mode - Map + Monster Panel Side by Side -->
          <template v-else-if="viewMode === 'map'">
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
              <aside class="monster-panel" v-if="selectedMonster" :class="{ collapsed: !monsterPanelOpen }">
                <button class="monster-panel-toggle" @click="monsterPanelOpen = !monsterPanelOpen">
                  <span>{{ monsterPanelOpen ? '›' : '‹' }}</span>
                </button>

                <div class="monster-panel-content" v-show="monsterPanelOpen">
                  <!-- Monster Header -->
                  <header class="monster-header">
                    <div class="monster-title">
                      <h2>{{ selectedMonster.monster_name }}</h2>
                      <p class="monster-type">{{ formatCreatureType(selectedMonster.monster_data) }}</p>
                    </div>
                    <button class="close-monster" @click="selectedMonster = null" title="Close">×</button>
                  </header>

                  <div class="monster-body" v-if="selectedMonster.monster_data">
                    <!-- Quick Stats Bar -->
                    <div class="quick-stats">
                      <div class="quick-stat">
                        <span class="stat-label">AC</span>
                        <span class="stat-value" v-html="formatAC(selectedMonster.monster_data)"></span>
                      </div>
                      <div class="quick-stat">
                        <span class="stat-label">HP</span>
                        <span class="stat-value">{{ formatHP(selectedMonster.monster_data) }}</span>
                      </div>
                      <div class="quick-stat">
                        <span class="stat-label">Speed</span>
                        <span class="stat-value">{{ formatSpeed(selectedMonster.monster_data) }}</span>
                      </div>
                    </div>

                    <!-- Ability Scores -->
                    <div class="ability-row">
                      <div class="ability-item" v-for="ability in ['str', 'dex', 'con', 'int', 'wis', 'cha']" :key="ability">
                        <span class="ability-name">{{ ability.toUpperCase() }}</span>
                        <span class="ability-value">{{ selectedMonster.monster_data[ability] || 10 }}</span>
                        <span class="ability-mod">{{ formatModifier(selectedMonster.monster_data[ability] || 10) }}</span>
                      </div>
                    </div>

                    <!-- Secondary Properties (collapsible) -->
                    <details class="stat-section" open>
                      <summary>Properties</summary>
                      <div class="properties-list">
                        <div v-if="formatSaves(selectedMonster.monster_data)" class="prop-line">
                          <span class="prop-name">Saves</span>
                          <span>{{ formatSaves(selectedMonster.monster_data) }}</span>
                        </div>
                        <div v-if="formatSkills(selectedMonster.monster_data)" class="prop-line">
                          <span class="prop-name">Skills</span>
                          <span>{{ formatSkills(selectedMonster.monster_data) }}</span>
                        </div>
                        <div v-if="formatDamageResistances(selectedMonster.monster_data)" class="prop-line">
                          <span class="prop-name">Resist</span>
                          <span>{{ formatDamageResistances(selectedMonster.monster_data) }}</span>
                        </div>
                        <div v-if="formatDamageImmunities(selectedMonster.monster_data)" class="prop-line">
                          <span class="prop-name">Immune</span>
                          <span>{{ formatDamageImmunities(selectedMonster.monster_data) }}</span>
                        </div>
                        <div v-if="formatConditionImmunities(selectedMonster.monster_data)" class="prop-line">
                          <span class="prop-name">Cond. Immune</span>
                          <span>{{ formatConditionImmunities(selectedMonster.monster_data) }}</span>
                        </div>
                        <div v-if="formatSenses(selectedMonster.monster_data)" class="prop-line">
                          <span class="prop-name">Senses</span>
                          <span>{{ formatSenses(selectedMonster.monster_data) }}</span>
                        </div>
                        <div class="prop-line">
                          <span class="prop-name">CR</span>
                          <span>{{ formatCR(selectedMonster.monster_data) }}</span>
                        </div>
                      </div>
                    </details>

                    <!-- Traits -->
                    <details v-if="selectedMonster.monster_data.trait?.length" class="stat-section">
                      <summary>Traits</summary>
                      <div class="action-list">
                        <div v-for="(trait, idx) in selectedMonster.monster_data.trait" :key="'trait-' + idx" class="action-item">
                          <strong>{{ trait.name }}.</strong>
                          <span v-html="formatActionEntries(trait.entries)"></span>
                        </div>
                      </div>
                    </details>

                    <!-- Actions -->
                    <details v-if="selectedMonster.monster_data.action?.length" class="stat-section actions" open>
                      <summary>Actions</summary>
                      <div class="action-list">
                        <div v-for="(action, idx) in selectedMonster.monster_data.action" :key="'action-' + idx" class="action-item">
                          <strong>{{ action.name }}.</strong>
                          <span v-html="formatActionEntries(action.entries)"></span>
                        </div>
                      </div>
                    </details>

                    <!-- Bonus Actions -->
                    <details v-if="selectedMonster.monster_data.bonus?.length" class="stat-section">
                      <summary>Bonus Actions</summary>
                      <div class="action-list">
                        <div v-for="(bonus, idx) in selectedMonster.monster_data.bonus" :key="'bonus-' + idx" class="action-item">
                          <strong>{{ bonus.name }}.</strong>
                          <span v-html="formatActionEntries(bonus.entries)"></span>
                        </div>
                      </div>
                    </details>

                    <!-- Reactions -->
                    <details v-if="selectedMonster.monster_data.reaction?.length" class="stat-section">
                      <summary>Reactions</summary>
                      <div class="action-list">
                        <div v-for="(reaction, idx) in selectedMonster.monster_data.reaction" :key="'reaction-' + idx" class="action-item">
                          <strong>{{ reaction.name }}.</strong>
                          <span v-html="formatActionEntries(reaction.entries)"></span>
                        </div>
                      </div>
                    </details>

                    <!-- Legendary Actions -->
                    <details v-if="selectedMonster.monster_data.legendary?.length" class="stat-section legendary">
                      <summary>Legendary Actions</summary>
                      <p class="legendary-intro">3 actions per round, at end of other creature's turn.</p>
                      <div class="action-list">
                        <div v-for="(legendary, idx) in selectedMonster.monster_data.legendary" :key="'legendary-' + idx" class="action-item">
                          <strong>{{ legendary.name }}.</strong>
                          <span v-html="formatActionEntries(legendary.entries)"></span>
                        </div>
                      </div>
                    </details>
                  </div>

                  <!-- Source -->
                  <footer class="monster-footer">
                    <span class="source-tag">{{ selectedMonster.monster_source }}</span>
                  </footer>
                </div>
              </aside>
            </div>
          </template>
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
    <div v-if="modalContent.visible" class="modal-overlay" @click="closeModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>{{ modalContent.title }}</h2>
          <button class="modal-close" @click="closeModal">×</button>
        </div>
        <div class="modal-body dnd-content" v-html="modalContent.content"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import { usePlayerDisplay } from '@/composables/usePlayerDisplay'
import StarterKit from '@tiptap/starter-kit'
import { Markdown } from '@tiptap/markdown'
import { Table } from '@tiptap/extension-table'
import { TableRow } from '@tiptap/extension-table-row'
import { TableCell } from '@tiptap/extension-table-cell'
import { TableHeader } from '@tiptap/extension-table-header'
import DmMapViewer from '@/components/DmMapViewer.vue'
import type { Module, Document, Campaign } from '@/types'
import { processFormattingTags } from '@/features/sources/utils/textFormatting'
import { useCrossReferences } from '@/features/sources/composables/useCrossReferences'

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
const documents = ref<Document[]>([])
const selectedDocument = ref<Document | null>(null)
const sidebarCollapsed = ref(false)
const documentsLoading = ref(true)
const documentLoading = ref(false)

// Monster/Encounter state
interface MonsterWithData {
  id: number
  module_id: number
  monster_name: string
  monster_source: string
  quantity: number
  encounter_tag: string | null
  monster_data: any | null
}

interface EncounterGroup {
  encounter_tag: string | null
  monsters: MonsterWithData[]
}

// Map state
interface MapSummary {
  id: number
  campaign_id: number
  module_id: number | null
  name: string
  grid_type: string
  grid_size_px: number | null
  grid_offset_x: number
  grid_offset_y: number
  module_name: string | null
  width_px: number
  height_px: number
  ambient_light: string
  image_path: string
}

const allMaps = ref<MapSummary[]>([])
const mapsLoading = ref(false)
const activeMapId = ref<number | null>(null)
const viewMode = ref<'documents' | 'map'>('documents')
const monsterPanelOpen = ref(true)

// Get the active map details for the DmMapViewer
const activeMap = computed(() => {
  if (!activeMapId.value) return null
  return allMaps.value.find(m => m.id === activeMapId.value) || null
})

// Load maps for this module (campaign-level maps + this module's maps only)
async function loadMaps() {
  if (!campaign.value?.id) return

  mapsLoading.value = true
  try {
    const response = await invoke<{ success: boolean; data?: MapSummary[] }>('list_map_summaries', {
      campaignId: campaign.value.id
    })

    if (response.success && response.data) {
      // Filter to show only:
      // 1. Campaign-level maps (module_id is null)
      // 2. Maps for the current module
      allMaps.value = response.data.filter(map =>
        map.module_id === null || map.module_id === moduleId.value
      )
    }
  } catch (e) {
    console.error('Failed to load maps:', e)
  } finally {
    mapsLoading.value = false
  }
}

// Select a map and optionally send to player display
async function sendMapToDisplay(map: MapSummary) {
  // Always set the active map and switch to map view (DM viewer works independently)
  activeMapId.value = map.id
  viewMode.value = 'map'

  // If display is open, send the map to it
  if (isDisplayOpen.value) {
    try {
      await invoke('send_map_to_display', {
        mapId: map.id,
        gridType: map.grid_type,
        gridSizePx: map.grid_size_px,
        gridOffsetX: map.grid_offset_x,
        gridOffsetY: map.grid_offset_y,
        ambientLight: map.ambient_light,
        mapWidth: map.width_px,
        mapHeight: map.height_px
      })
    } catch (err) {
      console.error('Failed to send map to display:', err)
    }
  }
}

const encounterGroups = ref<EncounterGroup[]>([])
const allMonsters = ref<MonsterWithData[]>([])
const selectedEncounter = ref<string | null>(null)
const selectedMonster = ref<MonsterWithData | null>(null)
const encountersLoading = ref(true)

// Load encounters/monsters for this module
async function loadEncounters() {
  encountersLoading.value = true
  try {
    const response = await invoke<{ data: MonsterWithData[] }>('list_module_monsters_with_data', {
      moduleId: moduleId.value
    })

    const monsters = response.data || []
    allMonsters.value = monsters

    // Group monsters by encounter_tag
    const groups = new Map<string | null, MonsterWithData[]>()
    for (const monster of monsters) {
      const tag = monster.encounter_tag
      if (!groups.has(tag)) {
        groups.set(tag, [])
      }
      groups.get(tag)!.push(monster)
    }

    // Convert to array, putting tagged encounters first
    const groupArray: EncounterGroup[] = []
    for (const [tag, groupMonsters] of groups) {
      if (tag !== null) {
        groupArray.push({ encounter_tag: tag, monsters: groupMonsters })
      }
    }
    // Add untagged at the end if any
    if (groups.has(null)) {
      groupArray.push({ encounter_tag: null, monsters: groups.get(null)! })
    }

    encounterGroups.value = groupArray
  } catch (error) {
    console.error('Failed to load encounters:', error)
    encounterGroups.value = []
    allMonsters.value = []
  } finally {
    encountersLoading.value = false
  }
}

// Select an encounter group to expand
function selectEncounter(group: EncounterGroup) {
  if (selectedEncounter.value === group.encounter_tag) {
    // Toggle off if clicking same group
    selectedEncounter.value = null
    selectedMonster.value = null
  } else {
    selectedEncounter.value = group.encounter_tag
    // Auto-select first monster in group
    if (group.monsters.length > 0) {
      selectedMonster.value = group.monsters[0]
    }
  }
}

// Select a monster to show details
function selectMonster(monster: MonsterWithData) {
  selectedMonster.value = monster
}

// Select a monster and switch to combat view with panel open
function selectMonsterAndShowTab(monster: MonsterWithData) {
  selectedMonster.value = monster
  viewMode.value = 'map'
  monsterPanelOpen.value = true
}

// Format creature type line (e.g., "Medium humanoid (any race), any alignment")
function formatCreatureType(monsterData: any): string {
  if (!monsterData) return ''

  const parts: string[] = []

  // Size
  const size = Array.isArray(monsterData.size) ? monsterData.size[0] : monsterData.size
  const sizeMap: Record<string, string> = {
    'T': 'Tiny', 'S': 'Small', 'M': 'Medium', 'L': 'Large', 'H': 'Huge', 'G': 'Gargantuan'
  }
  parts.push(sizeMap[size] || size || 'Medium')

  // Type
  if (monsterData.type) {
    if (typeof monsterData.type === 'string') {
      parts.push(monsterData.type)
    } else if (typeof monsterData.type === 'object') {
      let typeStr = monsterData.type.type || ''
      if (monsterData.type.tags?.length) {
        typeStr += ` (${monsterData.type.tags.join(', ')})`
      }
      parts.push(typeStr)
    }
  }

  // Alignment
  if (monsterData.alignment) {
    const alignmentMap: Record<string, string> = {
      'L': 'lawful', 'N': 'neutral', 'C': 'chaotic', 'G': 'good', 'E': 'evil', 'U': 'unaligned', 'A': 'any alignment'
    }
    const alignment = Array.isArray(monsterData.alignment)
      ? monsterData.alignment.map((a: string) => alignmentMap[a] || a).join(' ')
      : alignmentMap[monsterData.alignment] || monsterData.alignment
    parts.push(`, ${alignment}`)
  }

  return parts.join(' ')
}

// Format speed (e.g., "30 ft., fly 60 ft., swim 30 ft.")
function formatSpeed(monsterData: any): string {
  if (!monsterData?.speed) return '30 ft.'

  const speed = monsterData.speed
  const parts: string[] = []

  if (typeof speed === 'number') {
    return `${speed} ft.`
  }

  if (speed.walk) parts.push(`${speed.walk} ft.`)
  if (speed.burrow) parts.push(`burrow ${speed.burrow} ft.`)
  if (speed.climb) parts.push(`climb ${speed.climb} ft.`)
  if (speed.fly) {
    let flyStr = `fly ${speed.fly} ft.`
    if (speed.canHover) flyStr += ' (hover)'
    parts.push(flyStr)
  }
  if (speed.swim) parts.push(`swim ${speed.swim} ft.`)

  return parts.length > 0 ? parts.join(', ') : '30 ft.'
}

// Format ability modifier (e.g., "+3" or "-1")
function formatModifier(score: number): string {
  const mod = Math.floor((score - 10) / 2)
  return mod >= 0 ? `+${mod}` : `${mod}`
}

// Format saving throws
function formatSaves(monsterData: any): string {
  if (!monsterData?.save) return ''

  const saves: string[] = []
  const abilityNames: Record<string, string> = {
    str: 'Str', dex: 'Dex', con: 'Con', int: 'Int', wis: 'Wis', cha: 'Cha'
  }

  for (const [ability, bonus] of Object.entries(monsterData.save)) {
    if (bonus) {
      saves.push(`${abilityNames[ability] || ability} ${bonus}`)
    }
  }

  return saves.join(', ')
}

// Format skills
function formatSkills(monsterData: any): string {
  if (!monsterData?.skill) return ''

  const skills: string[] = []
  const skillNames: Record<string, string> = {
    acrobatics: 'Acrobatics', athletics: 'Athletics', arcana: 'Arcana',
    deception: 'Deception', history: 'History', insight: 'Insight',
    intimidation: 'Intimidation', investigation: 'Investigation', medicine: 'Medicine',
    nature: 'Nature', perception: 'Perception', performance: 'Performance',
    persuasion: 'Persuasion', religion: 'Religion', sleight_of_hand: 'Sleight of Hand',
    stealth: 'Stealth', survival: 'Survival'
  }

  for (const [skill, bonus] of Object.entries(monsterData.skill)) {
    if (bonus) {
      skills.push(`${skillNames[skill] || skill} ${bonus}`)
    }
  }

  return skills.join(', ')
}

// Format senses
function formatSenses(monsterData: any): string {
  if (!monsterData) return ''

  const parts: string[] = []

  if (monsterData.senses) {
    if (Array.isArray(monsterData.senses)) {
      parts.push(...monsterData.senses)
    } else {
      parts.push(monsterData.senses)
    }
  }

  if (monsterData.passive) {
    parts.push(`passive Perception ${monsterData.passive}`)
  }

  return parts.join(', ')
}

// Format languages
function formatLanguages(monsterData: any): string {
  if (!monsterData?.languages) return '—'

  if (Array.isArray(monsterData.languages)) {
    return monsterData.languages.join(', ') || '—'
  }

  return monsterData.languages || '—'
}

// Format damage vulnerabilities
function formatDamageVulnerabilities(monsterData: any): string {
  if (!monsterData?.vulnerable) return ''
  if (Array.isArray(monsterData.vulnerable)) {
    return monsterData.vulnerable.join(', ')
  }
  return monsterData.vulnerable
}

// Format damage resistances
function formatDamageResistances(monsterData: any): string {
  if (!monsterData?.resist) return ''
  if (Array.isArray(monsterData.resist)) {
    return monsterData.resist.map((r: any) => {
      if (typeof r === 'string') return r
      if (r.resist) return r.resist.join(', ') + (r.note ? ` ${r.note}` : '')
      return ''
    }).filter(Boolean).join('; ')
  }
  return monsterData.resist
}

// Format damage immunities
function formatDamageImmunities(monsterData: any): string {
  if (!monsterData?.immune) return ''
  if (Array.isArray(monsterData.immune)) {
    return monsterData.immune.map((i: any) => {
      if (typeof i === 'string') return i
      if (i.immune) return i.immune.join(', ') + (i.note ? ` ${i.note}` : '')
      return ''
    }).filter(Boolean).join('; ')
  }
  return monsterData.immune
}

// Format condition immunities
function formatConditionImmunities(monsterData: any): string {
  if (!monsterData?.conditionImmune) return ''
  if (Array.isArray(monsterData.conditionImmune)) {
    return monsterData.conditionImmune.join(', ')
  }
  return monsterData.conditionImmune
}

// Format CR with XP
function formatCR(monsterData: any): string {
  if (!monsterData?.cr) return '?'

  const cr = monsterData.cr
  const xpByCR: Record<string, string> = {
    '0': '0 or 10', '1/8': '25', '1/4': '50', '1/2': '100',
    '1': '200', '2': '450', '3': '700', '4': '1,100', '5': '1,800',
    '6': '2,300', '7': '2,900', '8': '3,900', '9': '5,000', '10': '5,900',
    '11': '7,200', '12': '8,400', '13': '10,000', '14': '11,500', '15': '13,000',
    '16': '15,000', '17': '18,000', '18': '20,000', '19': '22,000', '20': '25,000',
    '21': '33,000', '22': '41,000', '23': '50,000', '24': '62,000', '25': '75,000',
    '26': '90,000', '27': '105,000', '28': '120,000', '29': '135,000', '30': '155,000'
  }

  const xp = xpByCR[String(cr)] || '?'
  return `${cr} (${xp} XP)`
}

// Format AC from 5etools data format
function formatAC(monsterData: any): string {
  if (!monsterData?.ac) return '?'

  const ac = monsterData.ac
  let result = ''
  if (Array.isArray(ac)) {
    // 5etools format: ac is an array of AC objects or numbers
    const first = ac[0]
    if (typeof first === 'number') {
      result = String(first)
    } else if (typeof first === 'object') {
      const base = first.ac || first
      const from = first.from ? ` (${first.from.join(', ')})` : ''
      result = `${base}${from}`
    }
  } else {
    result = String(ac)
  }
  // Process any 5etools formatting tags like {@item}
  return processFormattingTags(result)
}

// Format HP from 5etools data format
function formatHP(monsterData: any): string {
  if (!monsterData?.hp) return '?'

  const hp = monsterData.hp
  if (typeof hp === 'object') {
    const avg = hp.average || hp.special || '?'
    const formula = hp.formula ? ` (${hp.formula})` : ''
    return `${avg}${formula}`
  }
  return String(hp)
}

// Format action/trait entries (array of strings/objects) into HTML
function formatActionEntries(entries: any[]): string {
  if (!entries || !Array.isArray(entries)) return ''

  return entries.map(entry => {
    if (typeof entry === 'string') {
      return processFormattingTags(entry)
    } else if (entry && typeof entry === 'object') {
      // Handle nested entries objects
      if (entry.entries) {
        return formatActionEntries(entry.entries)
      }
      return ''
    }
    return ''
  }).join(' ')
}

// Extract spellcasting info from monster data (can be in different formats)
function getSpellcasting(monsterData: any): string | null {
  if (!monsterData) return null

  // Check for spellcasting array (5etools format)
  if (monsterData.spellcasting && Array.isArray(monsterData.spellcasting)) {
    return monsterData.spellcasting.map((sc: any) => {
      let html = ''
      if (sc.headerEntries) {
        html += sc.headerEntries.map((e: string) => `<p>${processFormattingTags(e)}</p>`).join('')
      }
      if (sc.spells) {
        html += '<div class="spell-slots">'
        for (const [level, spellData] of Object.entries(sc.spells as Record<string, any>)) {
          const levelName = level === '0' ? 'Cantrips (at will)' : `${getOrdinal(parseInt(level))} level (${spellData.slots || '?'} slots)`
          const spellList = (spellData.spells || []).map((s: string) => processFormattingTags(s)).join(', ')
          html += `<p><strong>${levelName}:</strong> ${spellList}</p>`
        }
        html += '</div>'
      }
      if (sc.daily) {
        html += '<div class="innate-spells">'
        for (const [uses, spells] of Object.entries(sc.daily as Record<string, string[]>)) {
          const usesText = uses === '1' ? '1/day' : `${uses}/day each`
          const spellList = spells.map((s: string) => processFormattingTags(s)).join(', ')
          html += `<p><strong>${usesText}:</strong> ${spellList}</p>`
        }
        html += '</div>'
      }
      return html
    }).join('')
  }

  // Check for spellcasting in traits
  if (monsterData.trait && Array.isArray(monsterData.trait)) {
    const spellTrait = monsterData.trait.find((t: any) =>
      t.name?.toLowerCase().includes('spellcasting') ||
      t.name?.toLowerCase().includes('innate spellcasting')
    )
    if (spellTrait) {
      return formatActionEntries(spellTrait.entries)
    }
  }

  return null
}

// Helper for ordinal numbers (1st, 2nd, 3rd, etc.)
function getOrdinal(n: number): string {
  const suffixes = ['th', 'st', 'nd', 'rd']
  const v = n % 100
  return n + (suffixes[(v - 20) % 10] || suffixes[v] || suffixes[0])
}

// Notes state
const notesCollapsed = ref(true)
const notesContent = ref('')
const notesFilePath = ref('')
const notesSaving = ref(false)
const notesLastSaved = ref(false)
let saveTimeout: ReturnType<typeof setTimeout> | null = null

// TipTap editor for read-only document viewing
const editor = useEditor({
  content: '',
  editable: false,
  extensions: [
    StarterKit.configure({
      heading: {
        levels: [1, 2, 3, 4, 5, 6]
      }
    }),
    Markdown,
    Table.configure({
      resizable: false
    }),
    TableRow,
    TableCell,
    TableHeader
  ]
})

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

      // Build notes file path
      if (campaign.value?.directory_path && module.value) {
        const moduleNumber = (module.value as any).module_number || 1
        const paddedNumber = String(moduleNumber).padStart(2, '0')
        notesFilePath.value = `${campaign.value.directory_path}/modules/module_${paddedNumber}/play-notes.md`

        // Load existing notes
        await loadNotes()
      }
    }
  } catch (error) {
    console.error('Failed to load module:', error)
  }
}

// Load module documents
async function loadDocuments() {
  documentsLoading.value = true
  try {
    const response = await invoke<{ data: Document[] }>('get_module_documents', {
      request: {
        module_id: moduleId.value
      }
    })
    documents.value = response.data || []

    // Auto-select the first document (usually module overview)
    if (documents.value.length > 0) {
      // Try to find module_overview first, otherwise use first document
      const overview = documents.value.find(d => d.template_id === 'module_overview')
      selectDocument(overview || documents.value[0])
    }
  } catch (error) {
    console.error('Failed to load documents:', error)
  } finally {
    documentsLoading.value = false
  }
}

// Select and load a document
async function selectDocument(doc: Document) {
  selectedDocument.value = doc
  await loadDocumentContent(doc)
}

// Strip YAML frontmatter from markdown content
function stripFrontmatter(content: string): string {
  const frontmatterRegex = /^---\r?\n[\s\S]*?\r?\n---\r?\n?/
  return content.replace(frontmatterRegex, '')
}

// Load document content
async function loadDocumentContent(doc: Document) {
  if (!doc.file_path) return

  documentLoading.value = true
  try {
    const response = await invoke<{ data: string }>('read_document_file', {
      filePath: doc.file_path
    })

    if (response.data && editor.value) {
      const content = stripFrontmatter(response.data)
      editor.value.commands.setContent(content, { contentType: 'markdown' })
    }
  } catch (error) {
    console.error('Failed to load document content:', error)
    if (editor.value) {
      editor.value.commands.setContent('*Failed to load document content*', { contentType: 'markdown' })
    }
  } finally {
    documentLoading.value = false
  }
}

// Navigation
function handleEndSession() {
  // Navigate to campaign board view
  if (campaign.value?.id) {
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

// Notes panel
function toggleNotes() {
  notesCollapsed.value = !notesCollapsed.value
}

// Load notes from file
async function loadNotes() {
  if (!notesFilePath.value) return

  try {
    const response = await invoke<{ data: string }>('read_document_file', {
      filePath: notesFilePath.value
    })
    if (response.data) {
      notesContent.value = response.data
    }
  } catch (error) {
    // File might not exist yet, that's OK
    console.log('Notes file not found, will create on first save')
    notesContent.value = ''
  }
}

// Save notes to file
async function saveNotes() {
  if (!notesFilePath.value) return

  notesSaving.value = true
  notesLastSaved.value = false

  try {
    await invoke('save_document_file', {
      filePath: notesFilePath.value,
      content: notesContent.value
    })
    notesLastSaved.value = true
    // Clear the "Saved" indicator after 2 seconds
    setTimeout(() => {
      notesLastSaved.value = false
    }, 2000)
  } catch (error) {
    console.error('Failed to save notes:', error)
  } finally {
    notesSaving.value = false
  }
}

// Handle notes input with debounced auto-save
function handleNotesInput() {
  // Clear any pending save
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }

  // Schedule save after 1 second of inactivity
  saveTimeout = setTimeout(() => {
    saveNotes()
  }, 1000)
}

// Cleanup
onBeforeUnmount(() => {
  editor.value?.destroy()
  // Save any pending notes before unmount
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveNotes()
  }
  // Clean up cross-reference handlers
  document.removeEventListener('click', handleCrossRefClick as any)
  document.removeEventListener('mouseover', handleCrossRefHover as any)
  document.removeEventListener('mouseout', hideTooltip)
})

onMounted(async () => {
  await loadModule()
  await Promise.all([
    loadDocuments(),
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
  background: var(--color-base-200);
  overflow: hidden;
}

/* Header Styles */
.play-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1.5rem;
  background: var(--color-base-300);
  border-bottom: 2px solid var(--color-accent, #e67e22);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.header-left,
.header-right {
  flex: 1;
}

.header-right {
  display: flex;
  justify-content: flex-end;
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
  background: var(--color-accent, #e67e22);
  color: white;
  border-radius: 0.25rem;
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
  border-radius: 0.375rem;
  color: var(--color-text);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.display-button:hover {
  background: var(--color-base-200);
  border-color: var(--color-primary);
}

.display-button.active {
  background: var(--color-success, #10b981);
  border-color: var(--color-success, #10b981);
  color: white;
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
  border-radius: 0.375rem;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.2s;
}

.blackout-button:hover {
  background: var(--color-base-200);
  color: var(--color-text);
}

.blackout-button.active {
  background: var(--color-warning, #f59e0b);
  border-color: var(--color-warning, #f59e0b);
  color: white;
}

.blackout-button .icon {
  width: 1.25rem;
  height: 1.25rem;
}

.end-session-button {
  padding: 0.5rem 1rem;
  background: var(--color-error, #dc3545);
  color: white;
  border: none;
  border-radius: 0.375rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.end-session-button:hover {
  background: var(--color-error-dark, #c82333);
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
  transition: width 0.3s ease;
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
  color: var(--color-text-muted);
}

.sidebar-toggle:hover {
  background: var(--color-base-200);
  border-color: var(--color-primary);
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
  color: var(--color-text-muted);
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

/* Document Tabs */
.document-tabs {
  display: flex;
  gap: 0.25rem;
  flex-wrap: wrap;
}

.doc-tab {
  padding: 0.5rem 1rem;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-bottom: none;
  border-radius: 0.375rem 0.375rem 0 0;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--color-text-muted);
  transition: all 0.2s;
}

.doc-tab:hover {
  background: var(--color-base-200);
  color: var(--color-text);
}

.doc-tab.active {
  background: var(--color-surface);
  color: var(--color-text);
  border-color: var(--color-accent, #e67e22);
  border-bottom: 2px solid var(--color-surface);
  margin-bottom: -1px;
  font-weight: 500;
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

.document-panel {
  border-top-left-radius: 0;
}

.document-header {
  border-bottom: 1px solid var(--color-border);
  padding-bottom: 0.75rem;
  margin-bottom: 1rem;
}

.document-content {
  flex: 1;
  overflow-y: scroll;
  min-height: 0;
}

/* Always show scrollbars */
.document-content::-webkit-scrollbar {
  width: 8px;
}

.document-content::-webkit-scrollbar-track {
  background: var(--color-base-200);
  border-radius: 4px;
}

.document-content::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 4px;
}

.document-content::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-muted);
}

.loading-state {
  color: var(--color-text-muted);
  font-style: italic;
}

.placeholder-text {
  color: var(--color-text-muted);
  font-style: italic;
}

/* Prose styling for document content */
.prose-content {
  line-height: 1.7;
}

.prose-content :deep(h1) {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 1.5rem 0 1rem 0;
  color: var(--color-text);
}

.prose-content :deep(h2) {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 1.25rem 0 0.75rem 0;
  color: var(--color-text);
}

.prose-content :deep(h3) {
  font-size: 1.1rem;
  font-weight: 600;
  margin: 1rem 0 0.5rem 0;
  color: var(--color-text);
}

.prose-content :deep(p) {
  margin: 0.75rem 0;
}

.prose-content :deep(ul),
.prose-content :deep(ol) {
  margin: 0.75rem 0;
  padding-left: 1.5rem;
}

.prose-content :deep(li) {
  margin: 0.25rem 0;
}

.prose-content :deep(blockquote) {
  border-left: 3px solid var(--color-accent, #e67e22);
  margin: 1rem 0;
  padding-left: 1rem;
  color: var(--color-text-muted);
  font-style: italic;
}

.prose-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--color-border);
  margin: 1.5rem 0;
}

.prose-content :deep(strong) {
  font-weight: 600;
}

.prose-content :deep(em) {
  font-style: italic;
}

/* Notes Panel */
.notes-panel {
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  transition: height 0.3s ease;
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
  background: var(--color-base-300);
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
  background: var(--color-base-200);
}

.notes-toggle-icon {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.notes-toggle-label {
  flex: 1;
}

.notes-saving {
  font-size: 0.75rem;
  color: var(--color-warning, #f59e0b);
  font-style: italic;
}

.notes-saved {
  font-size: 0.75rem;
  color: var(--color-success, #10b981);
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
  color: var(--color-text-muted);
  font-style: italic;
}

/* Notes textarea scrollbar */
.notes-textarea::-webkit-scrollbar {
  width: 8px;
}

.notes-textarea::-webkit-scrollbar-track {
  background: var(--color-base-200);
  border-radius: 4px;
}

.notes-textarea::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 4px;
}

.notes-textarea::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-muted);
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
  color: var(--color-text-muted);
  font-style: italic;
}

.encounter-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.encounter-group {
  background: var(--color-base-200);
  border-radius: 0.375rem;
  cursor: pointer;
  transition: all 0.2s;
}

.encounter-group:hover {
  background: var(--color-base-300);
}

.encounter-group.active {
  background: var(--color-base-300);
  border-left: 3px solid var(--color-accent, #e67e22);
}

.encounter-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
}

.encounter-name {
  font-weight: 500;
  font-size: 0.875rem;
}

.encounter-count {
  font-size: 0.75rem;
  background: var(--color-surface);
  padding: 0.125rem 0.5rem;
  border-radius: 999px;
  color: var(--color-text-muted);
}

.encounter-monsters {
  padding: 0.25rem 0.75rem 0.5rem 0.75rem;
  border-top: 1px solid var(--color-border);
}

.monster-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.5rem;
  font-size: 0.8rem;
  border-radius: 0.25rem;
  cursor: pointer;
}

.monster-item:hover {
  background: var(--color-surface);
}

.monster-qty {
  font-weight: 600;
  color: var(--color-accent, #e67e22);
  min-width: 2rem;
}

.monster-name {
  color: var(--color-text);
}

/* Monster Card Styles */
.monster-card {
  background: var(--color-base-200);
  border-radius: 0.5rem;
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
  color: var(--color-text-muted);
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
  color: var(--color-text-muted);
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
  color: var(--color-text-muted);
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
  color: var(--color-text-muted);
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
  color: var(--color-accent, #e67e22);
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
  color: var(--color-primary, #4a9eff);
  font-weight: 600;
}

.monster-action :deep(.cross-ref-link) {
  color: var(--color-primary, #4a9eff);
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
  color: #9333ea; /* Purple for legendary */
  margin: 0 0 0.5rem 0;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.legendary-intro {
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-muted);
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
  color: #2563eb; /* Blue for spellcasting */
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
  border-radius: 0.375rem;
  cursor: pointer;
  color: var(--color-text);
  background: var(--color-base-200);
  transition: all 0.15s ease;
  border-left: 3px solid transparent;
}

.monster-quick-item:hover {
  background: var(--color-base-300);
  border-left-color: var(--color-accent, #e67e22);
}

.monster-quick-item.active {
  background: var(--color-base-300);
  border-left-color: var(--color-primary, #4a9eff);
}

.monster-quick-item .monster-qty {
  font-weight: 700;
  color: var(--color-accent, #e67e22);
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
  border-radius: 0.25rem;
  color: var(--color-text-muted);
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
  background: var(--color-base-200);
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
  color: var(--color-text-muted);
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

/* View Mode Tabs */
.view-mode-tabs {
  display: flex;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
}

.view-mode-tab {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  color: var(--color-text-muted);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.view-mode-tab:hover {
  background: var(--color-base-200);
  color: var(--color-text);
}

.view-mode-tab.active {
  background: var(--color-primary-100);
  border-color: var(--color-primary-500);
  color: var(--color-primary-700);
}

.view-mode-tab .tab-icon {
  width: 16px;
  height: 16px;
}

.view-mode-tab .active-indicator {
  position: absolute;
  top: -2px;
  right: -2px;
  width: 8px;
  height: 8px;
  background: var(--color-success);
  border-radius: 50%;
  border: 2px solid var(--color-surface);
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
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  padding: 0.5rem 0.75rem;
  font-size: 0.85rem;
  color: var(--color-text);
  max-width: 300px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  pointer-events: none;
}

/* Cross-reference modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.modal-content {
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: 8px;
  max-width: 600px;
  max-height: 80vh;
  width: 90%;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--color-border, #333);
}

.modal-header h2 {
  margin: 0;
  font-size: 1.25rem;
  color: var(--color-text);
}

.modal-close {
  background: none;
  border: none;
  font-size: 1.5rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.modal-close:hover {
  color: var(--color-text);
}

.modal-body {
  padding: 1.25rem;
  overflow-y: auto;
  flex: 1;
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
  transition: margin-right 0.3s ease;
}

.combat-layout.monster-panel-open .map-area {
  margin-right: 0;
}

/* Monster Panel - Slides in from right */
.monster-panel {
  width: 380px;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  position: relative;
  transition: width 0.3s ease, opacity 0.3s ease;
  overflow: hidden;
}

.monster-panel.collapsed {
  width: 32px;
}

.monster-panel-toggle {
  position: absolute;
  left: -1px;
  top: 50%;
  transform: translateY(-50%);
  width: 24px;
  height: 48px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-right: none;
  border-radius: 6px 0 0 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  color: var(--color-text-muted);
  z-index: 10;
}

.monster-panel-toggle:hover {
  background: var(--color-base-200);
  color: var(--color-text);
}

.monster-panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Monster Header */
.monster-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--color-base-200);
  border-bottom: 2px solid var(--color-dnd-creature, #ff9f43);
}

.monster-title h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--color-text);
  line-height: 1.2;
}

.monster-type {
  margin: 0.15rem 0 0 0;
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-muted);
}

.close-monster {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.close-monster:hover {
  color: var(--color-text);
}

/* Monster Body */
.monster-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
}

/* Quick Stats */
.quick-stats {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.quick-stat {
  flex: 1;
  text-align: center;
  padding: 0.5rem;
  background: var(--color-base-200);
  border-radius: 0.375rem;
  border: 1px solid var(--color-border);
}

.quick-stat .stat-label {
  display: block;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
  margin-bottom: 0.15rem;
}

.quick-stat .stat-value {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
}

/* Ability Row */
.ability-row {
  display: flex;
  justify-content: space-between;
  gap: 0.25rem;
  margin-bottom: 0.75rem;
  padding: 0.5rem;
  background: var(--color-base-200);
  border-radius: 0.375rem;
}

.ability-item {
  flex: 1;
  text-align: center;
}

.ability-item .ability-name {
  display: block;
  font-size: 0.6rem;
  font-weight: 700;
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.ability-item .ability-value {
  display: block;
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--color-text);
}

.ability-item .ability-mod {
  display: block;
  font-size: 0.7rem;
  color: var(--color-dnd-creature, #ff9f43);
  font-weight: 600;
}

/* Stat Sections (using native details/summary) */
.stat-section {
  margin-bottom: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  overflow: hidden;
}

.stat-section summary {
  padding: 0.5rem 0.75rem;
  background: var(--color-base-200);
  font-size: 0.8rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: var(--color-text);
  cursor: pointer;
  user-select: none;
}

.stat-section summary:hover {
  background: var(--color-base-300);
}

.stat-section.actions summary {
  color: var(--color-dnd-damage, #ff6b6b);
}

.stat-section.legendary summary {
  color: #9333ea;
}

/* Properties List */
.properties-list {
  padding: 0.5rem 0.75rem;
}

.prop-line {
  display: flex;
  gap: 0.5rem;
  font-size: 0.8rem;
  line-height: 1.4;
  margin-bottom: 0.25rem;
}

.prop-line:last-child {
  margin-bottom: 0;
}

.prop-name {
  font-weight: 600;
  color: var(--color-text-muted);
  min-width: 5rem;
  flex-shrink: 0;
}

/* Action List */
.action-list {
  padding: 0.5rem 0.75rem;
}

.action-item {
  font-size: 0.8rem;
  line-height: 1.5;
  margin-bottom: 0.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--color-border);
}

.action-item:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

.action-item strong {
  color: var(--color-text);
}

.legendary-intro {
  font-size: 0.75rem;
  font-style: italic;
  color: var(--color-text-muted);
  margin: 0 0.75rem 0.5rem;
  padding-top: 0.5rem;
}

/* Cross-ref styling within monster panel */
.monster-panel :deep(.cross-ref-link),
.monster-panel :deep(.spell-ref),
.monster-panel :deep(.item-ref),
.monster-panel :deep(.condition-ref) {
  color: var(--color-primary, #4a9eff);
  text-decoration: underline;
  text-decoration-style: dotted;
  cursor: pointer;
}

.monster-panel :deep(.dice-roll),
.monster-panel :deep(.damage-roll) {
  font-family: monospace;
  font-weight: 700;
  color: var(--color-dnd-damage, #ff6b6b);
}

.monster-panel :deep(.hit-bonus) {
  font-weight: 700;
  color: var(--color-success, #34d399);
}

/* Monster Footer */
.monster-footer {
  padding: 0.5rem 0.75rem;
  border-top: 1px solid var(--color-border);
  background: var(--color-base-200);
}

.source-tag {
  font-size: 0.7rem;
  color: var(--color-text-muted);
  font-style: italic;
}
</style>
