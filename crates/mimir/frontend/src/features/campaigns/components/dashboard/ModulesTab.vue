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
          v-for="mod in modules"
          :key="mod.id"
          class="module-item"
          :class="{ selected: selectedModule?.id === mod.id }"
          @click="selectModule(mod)"
        >
          <span class="module-number">#{{ mod.module_number }}</span>
          <span class="module-name">{{ mod.name }}</span>
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
                Print
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
              <section class="dashboard-section dangers-section">
                <div class="section-header">
                  <h3>Dangers</h3>
                </div>
                <div v-if="loadingMonsters || loadingTraps" class="section-loading">Loading...</div>
                <div v-else-if="moduleMonsters.length === 0 && moduleTraps.length === 0" class="section-empty">
                  No dangers added
                </div>
                <div v-else class="dangers-list">
                  <!-- Monsters Section -->
                  <div v-if="moduleMonsters.length > 0" class="danger-category">
                    <div class="danger-category-header">Monsters</div>
                    <!-- Grouped by encounter tag -->
                    <div
                      v-for="group in encounterGroups"
                      :key="group.encounter_tag || 'untagged'"
                      class="monster-group"
                    >
                      <div v-if="group.encounter_tag" class="monster-group-header">
                        {{ group.encounter_tag }}
                      </div>
                      <div v-else-if="encounterGroups.length > 1" class="monster-group-header untagged">
                        Other
                      </div>
                      <div class="monster-group-items">
                        <div
                          v-for="monster in group.monsters"
                          :key="monster.id"
                          class="monster-row"
                          :class="{ active: selectedMonster?.id === monster.id }"
                          @click="handleSelectMonster(monster)"
                        >
                          <span class="monster-qty">{{ monster.quantity }}×</span>
                          <div class="monster-info">
                            <span class="monster-name">{{ monster.display_name || monster.monster_name }}</span>
                            <span v-if="monster.display_name" class="monster-original">({{ monster.monster_name }})</span>
                            <span v-if="monster.notes" class="monster-has-notes" title="Has DM notes">*</span>
                          </div>
                          <button
                            v-if="monster.id"
                            class="monster-edit-btn"
                            @click.stop="openMonsterEditModal(monster)"
                            title="Customize monster"
                          >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" />
                            </svg>
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- Traps/Hazards Section -->
                  <div v-if="moduleTraps.length > 0" class="danger-category">
                    <div class="danger-category-header">Traps & Hazards</div>
                    <div class="trap-list">
                      <div
                        v-for="trap in moduleTraps"
                        :key="trap.name"
                        class="trap-row"
                        :class="{ active: selectedTrap?.name === trap.name }"
                        @click="selectTrapForDetails(trap)"
                      >
                        <span class="trap-qty" v-if="trap.count > 1">{{ trap.count }}×</span>
                        <span class="trap-name">{{ trap.name }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </section>

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
            </div>
          </div>

          <!-- Play Notes Panel -->
          <aside class="notes-panel" :class="{ collapsed: notesCollapsed }">
            <button class="notes-toggle" @click="toggleNotes">
              <span class="notes-toggle-icon">{{ notesCollapsed ? '▲' : '▼' }}</span>
              <span class="notes-toggle-label">Play Notes</span>
              <span v-if="notesSaving" class="notes-saving">Saving...</span>
              <span v-else-if="notesLastSaved" class="notes-saved">Saved</span>
            </button>

            <div class="notes-content" v-show="!notesCollapsed">
              <textarea
                v-model="notesContent"
                class="notes-textarea"
                placeholder="Type your play notes here... (auto-saves)"
                @input="handleNotesInput"
              ></textarea>
            </div>
          </aside>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { ModuleService } from '@/services/ModuleService'
import { DocumentService } from '@/services/DocumentService'
import { useModuleMonsters } from '@/features/modules/composables/useModuleMonsters'
import { usePlayNotes, buildNotesFilePath } from '@/features/modules/composables/usePlayNotes'
import { openSourcesReference } from '@/shared/utils/windows'
import { dataEvents } from '@/shared/utils/dataEvents'
import CreateModuleModal from '../StageLanding/CreateModuleModal.vue'
import MapUploadModal from '../StageLanding/MapUploadModal.vue'
import MapTokenSetupModal from '@/components/tokens/MapTokenSetupModal.vue'
import DocumentEditor from '../DocumentEditor.vue'
import NpcSelectorModal from '@/features/modules/components/NpcSelectorModal.vue'
import MonsterStatsPanel from '@/features/modules/components/MonsterStatsPanel.vue'
import TrapDetailsPanel from '@/features/modules/components/TrapDetailsPanel.vue'
import ModuleExportDialog from '@/components/print/ModuleExportDialog.vue'
import CreateDocumentModal from '@/components/CreateDocumentModal.vue'
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
const showCreateModal = ref(false)
const showDeleteModuleModal = ref(false)

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

// Trap state
interface ModuleTrap {
  name: string
  source: string
  count: number
}
const moduleTraps = ref<ModuleTrap[]>([])
const loadingTraps = ref(false)
const selectedTrap = ref<ModuleTrap | null>(null)
const trapPanelOpen = ref(true)

// Play notes (from composable)
const {
  notesCollapsed,
  notesContent,
  notesSaving,
  notesLastSaved,
  toggleNotes,
  setNotesFilePath,
  loadNotes,
  handleNotesInput
} = usePlayNotes()

// Document state
const moduleDocuments = ref<Document[]>([])
const selectedDocument = ref<Document | null>(null)

// Map state
const moduleMaps = ref<MapData[]>([])
const loadingMaps = ref(false)
const showMapUploadModal = ref(false)
const showTokenSetupModal = ref(false)
const selectedMapForTokens = ref<MapData | null>(null)

// Document creation state
const showCreateDocModal = ref(false)
const showDeleteDocModal = ref(false)
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
const showNpcSelector = ref(false)
const showExportDialog = ref(false)
const moduleNpcs = ref<ModuleNpc[]>([])
const loadingNpcs = ref(false)

// Get NPC IDs that are already in the module
const existingNpcCharacterIds = computed(() => {
  return moduleNpcs.value.map(npc => npc.id)
})

// Monster customization modal state
const showMonsterEditModal = ref(false)
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

// Select a module
async function selectModule(mod: Module) {
  selectedModule.value = mod
  selectedDocument.value = null
  selectedTrap.value = null

  // Note: Play notes functionality currently disabled - directory_path not available
  // TODO: Re-enable once campaign directory structure is implemented

  await Promise.all([
    loadModuleDocuments(),
    loadModuleMaps(),
    loadModuleMonsters(),
    loadModuleTraps(),
    loadNpcs()
  ])
}

// Load traps from module maps (trap tokens)
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

    // Get tokens from all maps and filter for traps
    const trapCounts = new Map<string, ModuleTrap>()

    for (const map of mapsResponse.data) {
      const tokensResponse = await invoke<{ success: boolean; data?: any[] }>('list_tokens', {
        mapId: map.id
      })

      if (tokensResponse.success && tokensResponse.data) {
        for (const token of tokensResponse.data) {
          if (token.token_type === 'trap' && token.name) {
            const existing = trapCounts.get(token.name)
            if (existing) {
              existing.count++
            } else {
              trapCounts.set(token.name, {
                name: token.name,
                source: 'DMG', // Default source for trap tokens
                count: 1
              })
            }
          }
        }
      }
    }

    moduleTraps.value = Array.from(trapCounts.values()).sort((a, b) => a.name.localeCompare(b.name))
  } catch (e) {
    console.error('Failed to load module traps:', e)
    moduleTraps.value = []
  } finally {
    loadingTraps.value = false
  }
}

// Select trap for details view
function selectTrapForDetails(trap: ModuleTrap) {
  // Clear monster selection when selecting a trap
  clearSelectedMonster()
  selectedTrap.value = trap
  trapPanelOpen.value = true
}

// Clear selected trap
function clearSelectedTrap() {
  selectedTrap.value = null
}

// Wrapper to clear trap when selecting monster
function handleSelectMonster(monster: any) {
  clearSelectedTrap()
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

// Play module
function handlePlayModule() {
  if (selectedModule.value && props.campaign) {
    router.push(`/campaigns/${props.campaign.id}/dashboard/modules/${selectedModule.value.id}/play`)
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
  // Module NPCs are custom DM-created characters, not campaign characters
  // For now, just log the NPC details (could open a detail modal later)
  console.log('Selected NPC:', npc)
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

/* Monster List (grouped by encounter tag) */
.monster-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  overflow-y: auto;
  flex: 1;
}

.monster-group {
  display: flex;
  flex-direction: column;
}

.monster-group-header {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-error);
  padding: var(--spacing-xs) 0;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-xs);
}

.monster-group-header.untagged {
  color: var(--color-text-secondary);
}

.monster-group-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.monster-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.monster-row:hover {
  background: var(--color-primary-100);
}

.monster-row.active {
  background: var(--color-primary-100);
  border-left: 3px solid var(--color-error);
  padding-left: calc(var(--spacing-sm) - 3px);
}

.monster-qty {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--color-error);
  min-width: 24px;
}

.monster-info {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-xs);
  flex: 1;
  min-width: 0;
}

.monster-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text);
}

.monster-original {
  font-size: 0.7rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

.monster-has-notes {
  font-size: 0.8rem;
  color: var(--color-primary-500);
  font-weight: bold;
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

/* Dangers Section (combined monsters + traps) */
.dangers-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dangers-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  overflow-y: auto;
  flex: 1;
}

.danger-category {
  display: flex;
  flex-direction: column;
}

.danger-category-header {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-text-secondary);
  padding: var(--spacing-xs) 0;
  margin-bottom: var(--spacing-xs);
}

/* Trap List */
.trap-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.trap-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.trap-row:hover {
  background: var(--color-primary-100);
}

.trap-row.active {
  background: var(--color-primary-100);
  border-left: 3px solid var(--color-warning);
  padding-left: calc(var(--spacing-sm) - 3px);
}

.trap-qty {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--color-warning);
  min-width: 24px;
}

.trap-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text);
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

/* Play Notes Panel */
.notes-panel {
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  transition: height var(--transition-slow);
  height: 200px;
  min-height: 36px;
  flex-shrink: 0;
}

.notes-panel.collapsed {
  height: 36px;
}

.notes-toggle {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-md);
  background: var(--color-surface-variant);
  border: none;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--color-text);
  text-align: left;
  width: 100%;
}

.notes-toggle:hover {
  background: var(--color-surface-hover);
}

.notes-toggle-icon {
  font-size: 0.65rem;
  color: var(--color-text-secondary);
}

.notes-toggle-label {
  flex: 1;
}

.notes-saving {
  font-size: 0.7rem;
  color: var(--color-warning);
  font-style: italic;
}

.notes-saved {
  font-size: 0.7rem;
  color: var(--color-success);
}

.notes-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.notes-textarea {
  flex: 1;
  padding: var(--spacing-sm) var(--spacing-md);
  border: none;
  resize: none;
  font-family: inherit;
  font-size: 0.85rem;
  line-height: 1.5;
  background: var(--color-surface);
  color: var(--color-text);
  overflow-y: auto;
}

.notes-textarea:focus {
  outline: none;
}

.notes-textarea::placeholder {
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Monster Edit Button */
.monster-edit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.monster-row:hover .monster-edit-btn {
  opacity: 1;
}

.monster-edit-btn:hover {
  background: var(--color-primary-100);
  color: var(--color-primary-600);
}

.monster-edit-btn svg {
  width: 14px;
  height: 14px;
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
</style>
