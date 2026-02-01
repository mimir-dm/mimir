<template>
  <div class="modules-tab">
    <!-- Sidebar - Just module list -->
    <div class="sidebar-panel">
      <div class="sidebar-header">
        <h3>Modules</h3>
        <button class="btn-add" @click="showCreateModal = true" title="Create Module">+</button>
      </div>

      <div v-if="loading" class="modules-loading">Loading...</div>
      <div v-else-if="modules.length === 0" class="modules-empty">No modules yet</div>
      <div v-else class="modules-list">
        <div
          v-for="(mod, index) in modules"
          :key="mod.id"
          class="module-item"
          :class="{ selected: selectedModule?.id === mod.id }"
          @click="selectModule(mod)"
        >
          <span class="module-number">#{{ mod.module_number }}</span>
          <span class="module-name">{{ mod.name }}</span>
          <span class="module-reorder-buttons">
            <button
              class="btn-reorder"
              :disabled="index === 0"
              title="Move up"
              @click.stop="moveModule(mod.id, mod.module_number - 1)"
            >&#9650;</button>
            <button
              class="btn-reorder"
              :disabled="index === modules.length - 1"
              title="Move down"
              @click.stop="moveModule(mod.id, mod.module_number + 1)"
            >&#9660;</button>
          </span>
        </div>
      </div>
    </div>

    <!-- Main Panel -->
    <div class="main-panel">
      <!-- Module Selected -->
      <template v-if="selectedModule">
        <!-- Document Editor -->
        <DocumentEditor
          v-if="selectedDocument"
          :document="selectedDocument"
          :campaign-id="campaign?.id || ''"
          :module-id="selectedModule.id"
          @close="selectedDocument = null"
          @updated="handleDocumentUpdated"
        />

        <!-- Module Dashboard (default) -->
        <div v-else class="module-dashboard">
          <!-- Module Header -->
          <div class="module-header">
            <div class="module-title">
              <template v-if="isEditingTitle">
                <input
                  ref="titleInput"
                  v-model="editingTitleValue"
                  class="module-title-input"
                  @keyup.enter="saveModuleTitle"
                  @keyup.escape="cancelEditTitle"
                  @blur="saveModuleTitle"
                />
              </template>
              <template v-else>
                <h2>{{ selectedModule.name }}</h2>
                <button class="btn-icon btn-edit-title" @click="startEditTitle" title="Edit title">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" />
                  </svg>
                </button>
              </template>
            </div>
            <div class="module-actions">
              <button class="btn btn-primary" @click="handlePlayModule">
                Play
              </button>
              <button class="btn btn-secondary" @click="showExportDialog = true">
                PDF
              </button>
              <button class="btn btn-danger" @click="confirmDeleteModule" title="Delete module">
                Delete
              </button>
            </div>
          </div>

          <!-- Dashboard Grid - Two Column Layout -->
          <div class="dashboard-grid">
            <!-- Left Column: Documents, NPCs, Maps -->
            <div class="dashboard-left">
              <!-- Documents Section -->
              <section class="dashboard-section documents-section">
                <div class="section-header">
                  <h3>Documents</h3>
                  <button class="btn-add" @click="showCreateDocModal = true" title="Create Document">+</button>
                </div>
                <div v-if="moduleDocuments.length === 0" class="section-empty">
                  No documents yet
                </div>
                <div v-else class="document-cards">
                  <div
                    v-for="doc in moduleDocuments"
                    :key="doc.id"
                    class="document-card"
                    @click="selectedDocument = doc"
                  >
                    <span class="doc-title">{{ formatDocumentTitle(doc.title || 'Untitled') }}</span>
                    <button
                      class="doc-delete-btn"
                      @click="confirmDeleteDocument(doc, $event)"
                      title="Delete document"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                      </svg>
                    </button>
                  </div>
                </div>
              </section>

              <!-- NPCs Section -->
              <section class="dashboard-section npcs-section">
                <div class="section-header">
                  <h3>NPCs</h3>
                  <button class="btn-add" @click="showNpcSelector = true" title="Add NPC">+</button>
                </div>
                <div v-if="moduleNpcs.length === 0" class="section-empty">
                  No NPCs assigned
                </div>
                <div v-else class="npc-cards">
                  <div
                    v-for="npc in moduleNpcs"
                    :key="npc.id"
                    class="npc-card"
                    @click="viewModuleNpc(npc)"
                  >
                    <span class="npc-name">{{ npc.name }}</span>
                    <span class="npc-role">{{ npc.role || 'NPC' }}</span>
                  </div>
                </div>
              </section>

              <!-- Maps Section -->
              <section class="dashboard-section maps-section">
                <div class="section-header">
                  <h3>Maps</h3>
                  <button class="btn-add" @click="showMapUploadModal = true" title="Upload Map">+</button>
                </div>
                <div v-if="loadingMaps" class="section-loading">Loading...</div>
                <div v-else-if="moduleMaps.length === 0" class="section-empty">
                  No maps uploaded
                </div>
                <div v-else class="map-cards">
                  <div
                    v-for="map in moduleMaps"
                    :key="map.id"
                    class="map-card"
                    @click="selectMap(map)"
                  >
                    <span class="map-name">{{ map.name }}</span>
                    <span class="map-size">{{ map.width_px }}x{{ map.height_px }}</span>
                  </div>
                </div>
              </section>
            </div>

            <!-- Right Column: Dangers (monsters + traps/hazards) -->
            <div class="dashboard-right">
              <DangersList
                :monsters="moduleMonsters"
                :encounter-groups="encounterGroups"
                :traps="moduleTraps"
                :pois="modulePois"
                :selected-monster-id="selectedMonster?.id"
                :selected-trap-name="selectedTrap?.name"
                :selected-poi-name="selectedPoi?.name"
                :loading-monsters="loadingMonsters"
                :loading-traps="loadingTraps"
                :loading-pois="loadingPois"
                @select-monster="handleSelectMonster"
                @edit-monster="openMonsterEditModal"
                @select-trap="selectTrapForDetails"
                @select-poi="selectPoiForDetails"
              />

              <!-- Monster Stats Panel -->
              <MonsterStatsPanel
                v-if="selectedMonster"
                :monster="selectedMonster"
                v-model:panelOpen="monsterPanelOpen"
                @close="clearSelectedMonster"
                class="module-monster-panel"
              />

              <!-- Trap Details Panel -->
              <TrapDetailsPanel
                v-if="selectedTrap"
                :trap="selectedTrap"
                v-model:panelOpen="trapPanelOpen"
                @close="clearSelectedTrap"
                class="module-trap-panel"
              />

              <!-- POI Details Panel -->
              <PoiDetailsPanel
                v-if="selectedPoi"
                :poi="selectedPoi"
                v-model:panelOpen="poiPanelOpen"
                @close="clearSelectedPoi"
                class="module-poi-panel"
              />
            </div>
          </div>

        </div>
      </template>

      <!-- No Module Selected -->
      <div v-else class="empty-state">
        <div class="empty-icon">+</div>
        <h3>No Module Selected</h3>
        <p>Select a module from the sidebar or create a new one.</p>
        <button v-if="modules.length === 0" class="btn btn-primary" @click="showCreateModal = true">
          Create First Module
        </button>
      </div>
    </div>

    <!-- Create Module Modal -->
    <CreateModuleModal
      :show="showCreateModal"
      @close="showCreateModal = false"
      @create="handleCreateModule"
    />

    <!-- Map Upload Modal -->
    <MapUploadModal
      :visible="showMapUploadModal"
      :campaign-id="campaign?.id || ''"
      :module-id="selectedModule?.id"
      @close="showMapUploadModal = false"
      @uploaded="handleMapUploaded"
    />

    <!-- Module Export Dialog -->
    <ModuleExportDialog
      :visible="showExportDialog"
      :module-id="selectedModule?.id || null"
      :module-name="selectedModule?.name"
      :module-number="selectedModule?.module_number"
      :campaign-id="campaign?.id"
      @close="showExportDialog = false"
    />

    <!-- Token Setup Modal -->
    <MapTokenSetupModal
      v-if="selectedMapForTokens"
      :visible="showTokenSetupModal"
      :map="selectedMapForTokens"
      @close="closeTokenSetup"
    />

    <!-- NPC Selector Modal -->
    <NpcSelectorModal
      :visible="showNpcSelector"
      :module-id="selectedModule?.id || ''"
      :campaign-id="campaign?.id || ''"
      :existing-npc-ids="existingNpcCharacterIds"
      @close="showNpcSelector = false"
      @added="handleNpcsAdded"
    />

    <!-- Create Document Modal -->
    <CreateDocumentModal
      :visible="showCreateDocModal"
      :campaign-id="campaign?.id || ''"
      :module-id="selectedModule?.id"
      @close="showCreateDocModal = false"
      @created="handleDocumentCreated"
    />

    <!-- Delete Document Confirmation Modal -->
    <AppModal
      :visible="showDeleteDocModal"
      title="Delete Document"
      size="sm"
      @close="showDeleteDocModal = false"
    >
      <p>Are you sure you want to delete "{{ documentToDelete?.title }}"?</p>
      <p class="delete-warning">This will permanently remove the document and its file from disk.</p>
      <template #footer>
        <button class="btn btn-secondary" @click="showDeleteDocModal = false">Cancel</button>
        <button class="btn btn-danger" @click="handleDeleteDocument">Delete</button>
      </template>
    </AppModal>

    <!-- Delete Module Confirmation Modal -->
    <AppModal
      :visible="showDeleteModuleModal"
      title="Delete Module"
      size="sm"
      @close="showDeleteModuleModal = false"
    >
      <p>Are you sure you want to delete "{{ selectedModule?.name }}"?</p>
      <p class="delete-warning">This will permanently delete the module and all its associated documents, maps, and assignments.</p>
      <template #footer>
        <button class="btn btn-secondary" @click="showDeleteModuleModal = false">Cancel</button>
        <button class="btn btn-danger" @click="handleDeleteModule">Delete Module</button>
      </template>
    </AppModal>

    <!-- Monster Customization Modal -->
    <AppModal
      :visible="showMonsterEditModal"
      title="Customize Monster"
      size="md"
      @close="closeMonsterEditModal"
    >
      <div v-if="monsterToEdit" class="monster-edit-form">
        <div class="form-header">
          <span class="base-monster-label">Base Monster:</span>
          <span class="base-monster-name">{{ monsterToEdit.monster_name }}</span>
          <span class="base-monster-source">({{ monsterToEdit.monster_source }})</span>
        </div>

        <div class="form-group">
          <label for="display-name">Display Name</label>
          <input
            id="display-name"
            v-model="monsterEditForm.display_name"
            type="text"
            class="form-input"
            :placeholder="monsterToEdit.monster_name"
          />
          <p class="form-help">Custom name to display instead of the base monster name (e.g., "Frost Wight" for a reskinned Goblin)</p>
        </div>

        <div class="form-group">
          <label for="monster-notes">DM Notes</label>
          <textarea
            id="monster-notes"
            v-model="monsterEditForm.notes"
            class="form-textarea"
            rows="5"
            placeholder="Notes about stat modifications, thematic changes, or encounter context..."
          ></textarea>
          <p class="form-help">Private notes about customizations or how this monster differs from the base stat block</p>
        </div>
      </div>
      <template #footer>
        <button class="btn btn-secondary" @click="closeMonsterEditModal">Cancel</button>
        <button class="btn btn-primary" @click="saveMonsterCustomization" :disabled="savingMonster">
          {{ savingMonster ? 'Saving...' : 'Save' }}
        </button>
      </template>
    </AppModal>

    <!-- NPC Detail Modal -->
    <AppModal
      :visible="showNpcDetailModal"
      :title="selectedNpc?.name || 'NPC Details'"
      size="md"
      @close="closeNpcDetailModal"
    >
      <div v-if="selectedNpc" class="npc-detail-content">
        <!-- Role badge -->
        <div v-if="selectedNpc.role" class="npc-role-badge">{{ selectedNpc.role }}</div>

        <!-- Description -->
        <div v-if="selectedNpc.description" class="npc-section">
          <p>{{ selectedNpc.description }}</p>
        </div>

        <!-- Appearance -->
        <div v-if="selectedNpc.appearance" class="npc-section">
          <h4>Appearance</h4>
          <p>{{ selectedNpc.appearance }}</p>
        </div>

        <!-- Personality -->
        <div v-if="selectedNpc.personality" class="npc-section">
          <h4>Personality</h4>
          <p>{{ selectedNpc.personality }}</p>
        </div>

        <!-- Motivation -->
        <div v-if="selectedNpc.motivation" class="npc-section">
          <h4>Motivation</h4>
          <p>{{ selectedNpc.motivation }}</p>
        </div>

        <!-- Secrets (DM only) -->
        <div v-if="selectedNpc.secrets" class="npc-section npc-secrets">
          <h4>Secrets <span class="dm-only-badge">DM Only</span></h4>
          <p>{{ selectedNpc.secrets }}</p>
        </div>

        <!-- Empty state -->
        <div v-if="!selectedNpc.description && !selectedNpc.appearance && !selectedNpc.personality && !selectedNpc.motivation && !selectedNpc.secrets" class="npc-empty">
          <p>No details have been added for this NPC yet.</p>
        </div>
      </div>
      <template #footer>
        <button class="btn btn-secondary" @click="closeNpcDetailModal">Close</button>
      </template>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { ModuleService } from '@/services/ModuleService'
import { DocumentService } from '@/services/DocumentService'
import { useModuleMonsters } from '@/features/modules/composables/useModuleMonsters'
import { useModalsState } from '@/features/campaigns/composables/useModalsState'
import { useDmMapWindow } from '@/composables/windows/useDmMapWindow'
import { useDashboardLink } from '@/composables/useDashboardLink'
import { openSourcesReference } from '@/utils/windows'
import { dataEvents } from '@/utils/dataEvents'
import CreateModuleModal from '../StageLanding/CreateModuleModal.vue'
import MapUploadModal from '../StageLanding/MapUploadModal.vue'
import MapTokenSetupModal from '@/components/tokens/MapTokenSetupModal.vue'
import DocumentEditor from '../DocumentEditor.vue'
import NpcSelectorModal from '@/features/modules/components/NpcSelectorModal.vue'
import MonsterStatsPanel from '@/features/modules/components/MonsterStatsPanel.vue'
import TrapDetailsPanel from '@/features/modules/components/TrapDetailsPanel.vue'
import PoiDetailsPanel from '@/features/modules/components/PoiDetailsPanel.vue'
import DangersList from './DangersList.vue'
import ModuleExportDialog from '@/components/print/ModuleExportDialog.vue'
import CreateDocumentModal from '@/components/dialogs/CreateDocumentModal.vue'
import AppModal from '@/components/shared/AppModal.vue'
import type { Campaign, Module, Document } from '@/types'

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

const router = useRouter()
const route = useRoute()

// Module state
const modules = ref<Module[]>([])
const selectedModule = ref<Module | null>(null)
const loading = ref(false)

// Modal state (from composable)
const {
  showCreateModal,
  showDeleteModuleModal,
  showMapUploadModal,
  showTokenSetupModal,
  showCreateDocModal,
  showDeleteDocModal,
  showNpcSelector,
  showExportDialog,
  showNpcDetailModal,
  showMonsterEditModal
} = useModalsState()

// Title editing state
const isEditingTitle = ref(false)
const editingTitleValue = ref('')
const titleInput = ref<HTMLInputElement | null>(null)

// Computed moduleId for monsters composable
const selectedModuleId = computed(() => selectedModule.value?.id || '')

// Monster state (from composable)
const {
  allMonsters: moduleMonsters,
  encounterGroups,
  selectedMonster,
  encountersLoading: loadingMonsters,
  loadEncounters: loadModuleMonsters,
  selectMonster,
  clearSelectedMonster
} = useModuleMonsters(selectedModuleId)

// Monster panel state
const monsterPanelOpen = ref(true)

// Trap state - references catalog traps
interface ModuleTrap {
  id: string
  name: string
  source: string  // Catalog source (e.g., "DMG")
  count: number   // How many of this trap type across all maps
}
const moduleTraps = ref<ModuleTrap[]>([])
const loadingTraps = ref(false)
const selectedTrap = ref<ModuleTrap | null>(null)
const trapPanelOpen = ref(true)

// POI state - points of interest on maps
interface ModulePoi {
  id: string
  name: string
  description: string | null
  icon: string
  color: string | null
  visible: number
  grid_x: number
  grid_y: number
  count: number   // How many of this POI type across all maps
}
const modulePois = ref<ModulePoi[]>([])
const loadingPois = ref(false)
const selectedPoi = ref<ModulePoi | null>(null)
const poiPanelOpen = ref(true)

// Document state
const moduleDocuments = ref<Document[]>([])
const selectedDocument = ref<Document | null>(null)

// Map state
const moduleMaps = ref<MapData[]>([])
const loadingMaps = ref(false)
const selectedMapForTokens = ref<MapData | null>(null)

// Document deletion state
const documentToDelete = ref<Document | null>(null)

// NPC state - Module NPCs are custom DM-created characters
interface ModuleNpc {
  id: string
  module_id: string
  name: string
  role: string | null
  description: string | null
  appearance: string | null
  personality: string | null
  motivation: string | null
  secrets: string | null
}
const moduleNpcs = ref<ModuleNpc[]>([])
const loadingNpcs = ref(false)

// NPC detail state
const selectedNpc = ref<ModuleNpc | null>(null)

// Get NPC IDs that are already in the module
const existingNpcCharacterIds = computed(() => {
  return moduleNpcs.value.map(npc => npc.id)
})

// Monster customization state
const monsterToEdit = ref<any>(null)
const monsterEditForm = ref({
  display_name: '',
  notes: ''
})
const savingMonster = ref(false)

// Load modules
async function loadModules() {
  if (!props.campaign?.id) return

  loading.value = true
  try {
    modules.value = await ModuleService.list(props.campaign.id)
  } catch (e) {
    console.error('Failed to load modules:', e)
  } finally {
    loading.value = false
  }
}

// Move a module to a new position
async function moveModule(moduleId: string, newPosition: number) {
  try {
    const updatedModules = await ModuleService.reorder(moduleId, newPosition)
    modules.value = updatedModules
    // Update selectedModule ref if it was the moved module
    if (selectedModule.value?.id === moduleId) {
      selectedModule.value = updatedModules.find(m => m.id === moduleId) || null
    }
  } catch (e) {
    console.error('Failed to reorder module:', e)
  }
}

// Select a module
async function selectModule(mod: Module) {
  selectedModule.value = mod
  selectedDocument.value = null
  selectedTrap.value = null
  selectedPoi.value = null

  await Promise.all([
    loadModuleDocuments(),
    loadModuleMaps(),
    loadModuleMonsters(),
    loadModuleTraps(),
    loadModulePois(),
    loadNpcs()
  ])
}

// Load traps from module maps (from map_traps table)
async function loadModuleTraps() {
  if (!selectedModule.value || !props.campaign) return

  loadingTraps.value = true
  try {
    // Get all maps for this module
    const mapsResponse = await invoke<{ success: boolean; data?: MapData[] }>('list_module_maps', {
      moduleId: selectedModule.value.id
    })

    if (!mapsResponse.success || !mapsResponse.data) {
      moduleTraps.value = []
      return
    }

    // Get traps from all maps (from map_traps table)
    // Group by name for catalog lookup
    const trapsByName = new Map<string, ModuleTrap>()

    for (const map of mapsResponse.data) {
      const trapsResponse = await invoke<{ success: boolean; data?: any[] }>('list_map_traps', {
        mapId: map.id
      })

      if (trapsResponse.success && trapsResponse.data) {
        for (const trap of trapsResponse.data) {
          if (trap.name) {
            const existing = trapsByName.get(trap.name)
            if (existing) {
              existing.count++
            } else {
              trapsByName.set(trap.name, {
                id: trap.id,
                name: trap.name,
                source: 'DMG',  // Default source for catalog lookup
                count: 1
              })
            }
          }
        }
      }
    }

    moduleTraps.value = Array.from(trapsByName.values()).sort((a, b) => a.name.localeCompare(b.name))
  } catch (e) {
    console.error('Failed to load module traps:', e)
    moduleTraps.value = []
  } finally {
    loadingTraps.value = false
  }
}

// Load POIs from module maps (from map_pois table)
async function loadModulePois() {
  if (!selectedModule.value || !props.campaign) return

  loadingPois.value = true
  try {
    // Get all maps for this module
    const mapsResponse = await invoke<{ success: boolean; data?: MapData[] }>('list_module_maps', {
      moduleId: selectedModule.value.id
    })

    if (!mapsResponse.success || !mapsResponse.data) {
      modulePois.value = []
      return
    }

    // Get POIs from all maps (from map_pois table)
    // Group by name
    const poisByName = new Map<string, ModulePoi>()

    for (const map of mapsResponse.data) {
      const poisResponse = await invoke<{ success: boolean; data?: any[] }>('list_map_pois', {
        mapId: map.id
      })

      if (poisResponse.success && poisResponse.data) {
        for (const poi of poisResponse.data) {
          if (poi.name) {
            const existing = poisByName.get(poi.name)
            if (existing) {
              existing.count++
            } else {
              poisByName.set(poi.name, {
                id: poi.id,
                name: poi.name,
                description: poi.description,
                icon: poi.icon || 'marker',
                color: poi.color,
                visible: poi.visible,
                grid_x: poi.grid_x,
                grid_y: poi.grid_y,
                count: 1
              })
            }
          }
        }
      }
    }

    modulePois.value = Array.from(poisByName.values()).sort((a, b) => a.name.localeCompare(b.name))
  } catch (e) {
    console.error('Failed to load module POIs:', e)
    modulePois.value = []
  } finally {
    loadingPois.value = false
  }
}

// Select trap for details view
function selectTrapForDetails(trap: ModuleTrap) {
  // Clear other selections when selecting a trap
  clearSelectedMonster()
  clearSelectedPoi()
  selectedTrap.value = trap
  trapPanelOpen.value = true
}

// Clear selected trap
function clearSelectedTrap() {
  selectedTrap.value = null
}

// Select POI for details view
function selectPoiForDetails(poi: ModulePoi) {
  // Clear other selections when selecting a POI
  clearSelectedMonster()
  clearSelectedTrap()
  selectedPoi.value = poi
  poiPanelOpen.value = true
}

// Clear selected POI
function clearSelectedPoi() {
  selectedPoi.value = null
}

// Wrapper to clear trap/POI when selecting monster
function handleSelectMonster(monster: any) {
  clearSelectedTrap()
  clearSelectedPoi()
  selectMonster(monster)
}

// Load documents for selected module
async function loadModuleDocuments() {
  if (!selectedModule.value) return

  try {
    moduleDocuments.value = await DocumentService.listForModule(selectedModule.value.id)
  } catch (e) {
    console.error('Failed to load module documents:', e)
  }
}

// Load maps for selected module
async function loadModuleMaps() {
  if (!selectedModule.value) return

  loadingMaps.value = true
  try {
    const response = await invoke<{ success: boolean; data?: MapData[] }>('list_module_maps', {
      moduleId: selectedModule.value.id
    })
    if (response.success && response.data) {
      moduleMaps.value = response.data
    }
  } catch (e) {
    console.error('Failed to load maps:', e)
  } finally {
    loadingMaps.value = false
  }
}

// Select a map - open token setup modal
function selectMap(map: MapData) {
  selectedMapForTokens.value = map
  showTokenSetupModal.value = true
}

// Close token setup modal
function closeTokenSetup() {
  showTokenSetupModal.value = false
  selectedMapForTokens.value = null
}

// Create module
async function handleCreateModule(data: { name: string; type: string; description?: string }) {
  if (!props.campaign?.id) return

  try {
    const newModule = await ModuleService.create({
      campaign_id: props.campaign.id,
      name: data.name,
      description: data.description,
      module_type: data.type
    })

    if (newModule) {
      showCreateModal.value = false
      await loadModules()
      selectModule(newModule)
    }
  } catch (e) {
    console.error('Failed to create module:', e)
  }
}

// Title editing functions
function startEditTitle() {
  if (!selectedModule.value) return
  editingTitleValue.value = selectedModule.value.name
  isEditingTitle.value = true
  nextTick(() => {
    titleInput.value?.focus()
    titleInput.value?.select()
  })
}

function cancelEditTitle() {
  isEditingTitle.value = false
  editingTitleValue.value = ''
}

async function saveModuleTitle() {
  if (!selectedModule.value || !editingTitleValue.value.trim()) {
    cancelEditTitle()
    return
  }

  const newName = editingTitleValue.value.trim()
  if (newName === selectedModule.value.name) {
    cancelEditTitle()
    return
  }

  try {
    await ModuleService.update(selectedModule.value.id, { name: newName })
    selectedModule.value.name = newName
    // Update in modules list
    const idx = modules.value.findIndex(m => m.id === selectedModule.value!.id)
    if (idx !== -1) {
      modules.value[idx].name = newName
    }
  } catch (e) {
    console.error('Failed to update module title:', e)
  } finally {
    cancelEditTitle()
  }
}

// Delete module functions
function confirmDeleteModule() {
  showDeleteModuleModal.value = true
}

async function handleDeleteModule() {
  if (!selectedModule.value) return

  try {
    await ModuleService.delete(selectedModule.value.id)
    showDeleteModuleModal.value = false
    selectedModule.value = null
    await loadModules()
  } catch (e) {
    console.error('Failed to delete module:', e)
  }
}

// Handle document updated
function handleDocumentUpdated(doc: Document) {
  const index = moduleDocuments.value.findIndex(d => d.id === doc.id)
  if (index !== -1) {
    moduleDocuments.value[index] = doc
  }
}

// Handle document created
async function handleDocumentCreated() {
  showCreateDocModal.value = false
  await loadModuleDocuments()
}

// Confirm delete document
function confirmDeleteDocument(doc: Document, event: Event) {
  event.stopPropagation()
  documentToDelete.value = doc
  showDeleteDocModal.value = true
}

// Delete document
async function handleDeleteDocument() {
  if (!documentToDelete.value) return

  try {
    await invoke('delete_document', {
      documentId: documentToDelete.value.id
    })

    // Remove from documents list
    moduleDocuments.value = moduleDocuments.value.filter(d => d.id !== documentToDelete.value!.id)

    // Clear selection if deleted doc was selected
    if (selectedDocument.value?.id === documentToDelete.value.id) {
      selectedDocument.value = null
    }

    showDeleteDocModal.value = false
    documentToDelete.value = null
  } catch (e) {
    console.error('Failed to delete document:', e)
  }
}

// Format document title
function formatDocumentTitle(templateId: string): string {
  return templateId
    .replace(/[-_]/g, ' ')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

// DM Map window
const { openWindow: openDmMapWindow } = useDmMapWindow()

// Dashboard link - handles entity focus events from DM Map window
useDashboardLink({
  onFocusMonster: (monsterId, tokenName) => {
    // Find the monster by its module_monster ID
    const monster = moduleMonsters.value.find(m => m.id === monsterId)
    if (monster) {
      handleSelectMonster(monster)
    } else {
      console.warn(`Monster with ID ${monsterId} not found in module (token: ${tokenName})`)
    }
  },
  onFocusNpc: (characterId, tokenName) => {
    // Find the NPC by its character ID
    const npc = moduleNpcs.value.find(n => n.id === characterId)
    if (npc) {
      viewModuleNpc(npc)
    } else {
      console.warn(`NPC with ID ${characterId} not found in module (token: ${tokenName})`)
    }
  },
  onFocusPc: (characterId, tokenName) => {
    // PCs don't have a detail modal in the module dashboard, log for now
    console.log(`PC focus requested: ${tokenName} (ID: ${characterId})`)
  },
  onFocusTrap: (trapId, trapName) => {
    // Find the trap by name (traps are grouped by name in the dashboard)
    const trap = moduleTraps.value.find(t => t.name === trapName)
    if (trap) {
      selectTrapForDetails(trap)
    } else {
      console.warn(`Trap "${trapName}" not found in module`)
    }
  },
  onFocusPoi: (poiId, poiName) => {
    // Find the POI by name (POIs are grouped by name in the dashboard)
    const poi = modulePois.value.find(p => p.name === poiName)
    if (poi) {
      selectPoiForDetails(poi)
    } else {
      console.warn(`POI "${poiName}" not found in module`)
    }
  }
})

// Play module - opens DM Map window
async function handlePlayModule() {
  if (selectedModule.value && props.campaign) {
    try {
      await openDmMapWindow(selectedModule.value.id, props.campaign.id)
    } catch (e) {
      console.error('Failed to open DM Map window:', e)
    }
  }
}

// Map handlers
function handleMapUploaded() {
  showMapUploadModal.value = false
  loadModuleMaps()
}

// Open monster reference window
async function openMonsterReference() {
  try {
    await openSourcesReference()
  } catch (e) {
    console.error('Failed to open reference:', e)
  }
}

// View module NPC detail
function viewModuleNpc(npc: ModuleNpc) {
  selectedNpc.value = npc
  showNpcDetailModal.value = true
}

// Close NPC detail modal
function closeNpcDetailModal() {
  showNpcDetailModal.value = false
  selectedNpc.value = null
}

// Handle NPCs added from selector
async function handleNpcsAdded() {
  showNpcSelector.value = false
  await loadNpcs()
}

// Load NPCs for the selected module
async function loadNpcs() {
  if (!selectedModule.value) return

  loadingNpcs.value = true
  try {
    const response = await invoke<{ success: boolean; data?: ModuleNpc[] }>('list_module_npcs', {
      moduleId: selectedModule.value.id
    })
    if (response.success && response.data) {
      moduleNpcs.value = response.data
    } else {
      moduleNpcs.value = []
    }
  } catch (e) {
    console.error('Failed to load module NPCs:', e)
    moduleNpcs.value = []
  } finally {
    loadingNpcs.value = false
  }
}

// Monster customization modal handlers
function openMonsterEditModal(monster: any) {
  monsterToEdit.value = monster
  monsterEditForm.value = {
    display_name: monster.display_name || '',
    notes: monster.notes || ''
  }
  showMonsterEditModal.value = true
}

function closeMonsterEditModal() {
  showMonsterEditModal.value = false
  monsterToEdit.value = null
  monsterEditForm.value = { display_name: '', notes: '' }
}

async function saveMonsterCustomization() {
  if (!monsterToEdit.value || !selectedModule.value) return

  savingMonster.value = true
  try {
    const displayName = monsterEditForm.value.display_name.trim() || null
    const notes = monsterEditForm.value.notes.trim() || null

    const response = await invoke<{ success: boolean; data?: any }>('update_module_monster', {
      monsterId: monsterToEdit.value.id,
      request: {
        display_name: displayName,
        notes: notes
      }
    })

    if (response.success) {
      // Emit event to trigger refresh in composable
      dataEvents.emit('module:monsters:changed', { moduleId: selectedModule.value.id })
      closeMonsterEditModal()
    }
  } catch (e) {
    console.error('Failed to update monster:', e)
  } finally {
    savingMonster.value = false
  }
}

// Watch for campaign changes
watch(() => props.campaign?.id, () => {
  selectedModule.value = null
  selectedDocument.value = null
  loadModules()
})

// Watch for query param to auto-select module (e.g., from play view back navigation)
watch(() => route.query.select, async (selectId) => {
  if (selectId && modules.value.length > 0) {
    const moduleToSelect = modules.value.find(m => m.id === String(selectId))
    if (moduleToSelect && selectedModule.value?.id !== moduleToSelect.id) {
      await selectModule(moduleToSelect)
    }
    // Clear the query param after selecting
    router.replace({ query: {} })
  }
}, { immediate: true })

onMounted(async () => {
  await loadModules()
  // Check for select query param after modules are loaded
  const selectId = route.query.select
  if (selectId && modules.value.length > 0) {
    const moduleToSelect = modules.value.find(m => m.id === String(selectId))
    if (moduleToSelect) {
      await selectModule(moduleToSelect)
      router.replace({ query: {} })
    }
  }
})
</script>

<style scoped>
.modules-tab {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.sidebar-panel {
  width: 280px;
  min-width: 240px;
  max-width: 320px;
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
  background: var(--color-surface);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-header h3 {
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

.modules-loading,
.modules-empty,
.maps-loading,
.maps-empty {
  padding: var(--spacing-sm) var(--spacing-md);
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.modules-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--spacing-xs);
}

.module-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.module-item:hover {
  background: var(--color-surface-variant);
}

.module-item.selected {
  background: var(--color-primary-100);
}

.module-number {
  font-weight: 600;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.module-reorder-buttons {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-left: auto;
  opacity: 0;
  transition: opacity 0.15s;
}

.module-item:hover .module-reorder-buttons {
  opacity: 1;
}

.btn-reorder {
  background: none;
  border: none;
  padding: 0 2px;
  font-size: 0.5rem;
  line-height: 1;
  color: var(--color-text-secondary);
  cursor: pointer;
  border-radius: 2px;
}

.btn-reorder:hover:not(:disabled) {
  color: var(--color-primary);
  background: var(--color-surface-variant);
}

.btn-reorder:disabled {
  opacity: 0.2;
  cursor: default;
}

.module-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Maps section */
.maps-section {
  border-top: 1px solid var(--color-border);
  padding: var(--spacing-xs) var(--spacing-sm);
}

.maps-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs) var(--spacing-sm) 0;
}

.maps-header h4 {
  margin: 0;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.maps-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.map-item {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.map-item:hover {
  background: var(--color-surface-variant);
}

.map-item.selected {
  background: var(--color-primary-100);
}

/* Main panel */
.main-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
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
  padding: var(--spacing-md) var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.map-preview-header h3 {
  margin: 0;
  font-size: 1.125rem;
}

.map-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.btn-action {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
}

.btn-action:hover {
  background: var(--color-primary-500);
  color: var(--color-background);
  border-color: var(--color-primary-500);
}

.btn-action.btn-danger:hover {
  background: var(--color-error);
  border-color: var(--color-error);
}

.btn-close {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
  border: none;
  background: none;
  color: var(--color-text-secondary);
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
  color: var(--color-text-secondary);
}

.map-info {
  display: flex;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-lg);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

/* Module Dashboard */
.module-dashboard {
  padding: var(--spacing-lg);
  overflow-y: auto;
  height: 100%;
}

.module-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.module-title {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.module-title h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.module-title-input {
  font-size: 1.25rem;
  font-weight: 600;
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 2px solid var(--color-primary-500);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  min-width: 200px;
}

.module-title-input:focus {
  outline: none;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-icon:hover {
  background: var(--color-surface-variant);
  color: var(--color-text);
}

.btn-edit-title svg {
  width: 16px;
  height: 16px;
}

.module-actions {
  display: flex;
  gap: var(--spacing-sm);
}

/* Dashboard Grid - Two Column Layout */
.dashboard-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.dashboard-left {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  overflow-y: auto;
}

.dashboard-right {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.dashboard-right .dashboard-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

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

.section-empty,
.section-loading {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-align: center;
  padding: var(--spacing-md);
}

/* NPC Cards */
.npc-cards {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}

.npc-card {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-left: 3px solid var(--color-warning);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.npc-card:hover {
  border-color: var(--color-primary-500);
  border-left-color: var(--color-warning);
}

.npc-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text);
}

.npc-role {
  font-size: 0.65rem;
  color: var(--color-text-secondary);
}

/* Monster Stats Panel in Module Dashboard */
.module-monster-panel {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  z-index: 10;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.3);
}

/* Trap Details Panel in Module Dashboard */
.module-trap-panel {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  z-index: 10;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.3);
}

/* Map Cards */
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

/* Document Cards */
.document-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.document-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.document-card:hover {
  border-color: var(--color-primary-500);
}

.doc-title {
  font-size: 0.8rem;
  color: var(--color-text);
}

.doc-status {
  font-size: 0.6rem;
  text-transform: uppercase;
  padding: 2px 4px;
  border-radius: 2px;
}

.doc-status.complete {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.doc-status.user {
  background: var(--color-primary-100);
  color: var(--color-primary-600);
}

.document-card.user-created {
  border-left: 3px solid var(--color-primary-500);
}

/* Document delete button */
.doc-delete-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  padding: 0;
  background: transparent;
  color: var(--color-text-muted);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  flex-shrink: 0;
  margin-left: var(--spacing-xs);
}

.document-card:hover .doc-delete-btn {
  opacity: 1;
}

.doc-delete-btn:hover {
  background: var(--color-error-100, rgba(239, 68, 68, 0.1));
  color: var(--color-error);
}

.doc-delete-btn svg {
  width: 14px;
  height: 14px;
}

/* Delete modal styles */
.delete-warning {
  font-size: 0.875rem;
  color: var(--color-error);
  margin-top: var(--spacing-sm);
}

.btn {
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary {
  background: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  background: var(--color-surface-variant);
}

.btn-danger {
  background: var(--color-error);
  color: white;
  border: none;
}

.btn-danger:hover {
  background: var(--color-error-dark, #dc2626);
}

/* Empty state */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: var(--spacing-md);
  opacity: 0.5;
}

.empty-state h3 {
  margin: 0 0 var(--spacing-sm) 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
}

.empty-state p {
  margin: 0 0 var(--spacing-md) 0;
  font-size: 0.875rem;
}

/* Monster Edit Form */
.monster-edit-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.form-header {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-sm);
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-sm);
}

.base-monster-label {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.base-monster-name {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text);
}

.base-monster-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.form-group label {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--color-text);
}

.form-input {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.form-textarea {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
  font-family: inherit;
  resize: vertical;
  min-height: 80px;
}

.form-textarea:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.form-help {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.4;
}

/* NPC Detail Modal */
.npc-detail-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.npc-role-badge {
  display: inline-block;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-warning-100, rgba(245, 158, 11, 0.1));
  color: var(--color-warning, #f59e0b);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-radius: var(--radius-sm);
  align-self: flex-start;
}

.npc-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.npc-section h4 {
  margin: 0;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-primary-500);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.npc-section p {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.6;
  color: var(--color-text);
}

.npc-secrets {
  padding: var(--spacing-sm);
  background: var(--color-error-100, rgba(239, 68, 68, 0.05));
  border: 1px dashed var(--color-error, #ef4444);
  border-radius: var(--radius-sm);
}

.npc-secrets h4 {
  color: var(--color-error, #ef4444);
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.dm-only-badge {
  font-size: 0.6rem;
  padding: 2px 6px;
  background: var(--color-error, #ef4444);
  color: white;
  border-radius: 2px;
  text-transform: uppercase;
  font-weight: 700;
}

.npc-empty {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-muted);
  font-style: italic;
}

.npc-empty p {
  margin: 0;
}
</style>
