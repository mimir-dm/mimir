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
          :campaign-id="campaign?.id || 0"
          :module-id="selectedModule.id"
          @close="selectedDocument = null"
          @updated="handleDocumentUpdated"
        />

        <!-- Module Dashboard (default) -->
        <div v-else class="module-dashboard">
          <!-- Module Header -->
          <div class="module-header">
            <div class="module-title">
              <h2>{{ selectedModule.name }}</h2>
            </div>
            <div class="module-actions">
              <button class="btn btn-primary" @click="handlePlayModule">
                Play
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
                    <span class="doc-title">{{ formatDocumentTitle(doc.template_id || doc.title || 'Untitled') }}</span>
                    <span v-if="doc.completed_at" class="doc-status complete">Done</span>
                  </div>
                </div>
              </section>

              <!-- NPCs Section -->
              <section class="dashboard-section npcs-section">
                <div class="section-header">
                  <h3>NPCs</h3>
                  <button class="btn-add" @click="showCreateNpcWizard = true" title="Add NPC">+</button>
                </div>
                <div v-if="moduleNpcs.length === 0" class="section-empty">
                  No NPCs assigned
                </div>
                <div v-else class="npc-cards">
                  <div
                    v-for="npc in moduleNpcs"
                    :key="npc.id"
                    class="npc-card"
                    @click="viewCharacter(npc)"
                  >
                    <span class="npc-name">{{ npc.character_name }}</span>
                    <span class="npc-role">{{ npc.class || 'NPC' }}</span>
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

            <!-- Right Column: Monsters (grouped by encounter tag) -->
            <div class="dashboard-right">
              <section class="dashboard-section monsters-section">
                <div class="section-header">
                  <h3>Monsters</h3>
                  <button class="btn-add" @click="openMonsterReference" title="Add Monster">+</button>
                </div>
                <div v-if="loadingMonsters" class="section-loading">Loading...</div>
                <div v-else-if="moduleMonsters.length === 0" class="section-empty">
                  No monsters added
                </div>
                <div v-else class="monster-list">
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
                        @click="selectMonster(monster)"
                      >
                        <span class="monster-qty">{{ monster.quantity }}×</span>
                        <span class="monster-name">{{ monster.monster_name }}</span>
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
            </div>
          </div>

          <!-- Session Notes Panel -->
          <aside class="notes-panel" :class="{ collapsed: notesCollapsed }">
            <button class="notes-toggle" @click="toggleNotes">
              <span class="notes-toggle-icon">{{ notesCollapsed ? '▲' : '▼' }}</span>
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
      :campaign-id="campaign?.id || 0"
      :module-id="selectedModule?.id"
      @close="showMapUploadModal = false"
      @uploaded="handleMapUploaded"
    />

    <!-- Token Setup Modal -->
    <MapTokenSetupModal
      v-if="selectedMapForTokens"
      :visible="showTokenSetupModal"
      :map="selectedMapForTokens"
      @close="closeTokenSetup"
    />

    <!-- NPC Creation Wizard -->
    <CharacterCreationWizard
      :visible="showCreateNpcWizard"
      :campaign-id="campaign?.id"
      :start-as-npc="true"
      @close="showCreateNpcWizard = false"
      @created="handleNpcCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { ModuleService } from '@/services/ModuleService'
import { DocumentService } from '@/services/DocumentService'
import { useCharacterStore } from '@/stores/characters'
import { useModuleMonsters } from '@/features/modules/composables/useModuleMonsters'
import { useSessionNotes, buildNotesFilePath } from '@/features/modules/composables/useSessionNotes'
import { openSourcesReference } from '@/shared/utils/windows'
import CreateModuleModal from '../StageLanding/CreateModuleModal.vue'
import MapUploadModal from '../StageLanding/MapUploadModal.vue'
import MapTokenSetupModal from '@/components/tokens/MapTokenSetupModal.vue'
import DocumentEditor from '../DocumentEditor.vue'
import CharacterCreationWizard from '@/features/characters/components/CharacterCreationWizard.vue'
import MonsterStatsPanel from '@/features/modules/components/MonsterStatsPanel.vue'
import type { Campaign, BoardConfig, Module, Document } from '@/types'
import type { Character } from '@/types/character'

interface MapData {
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
  campaign?: Campaign
  boardConfig?: BoardConfig
  documents?: any[]
}>()

const router = useRouter()
const route = useRoute()
const characterStore = useCharacterStore()

// Module state
const modules = ref<Module[]>([])
const selectedModule = ref<Module | null>(null)
const loading = ref(false)
const showCreateModal = ref(false)

// Computed moduleId for monsters composable
const selectedModuleId = computed(() => selectedModule.value?.id || 0)

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

// Document state
const moduleDocuments = ref<Document[]>([])
const selectedDocument = ref<Document | null>(null)

// Map state
const moduleMaps = ref<MapData[]>([])
const loadingMaps = ref(false)
const showMapUploadModal = ref(false)
const showTokenSetupModal = ref(false)
const selectedMapForTokens = ref<MapData | null>(null)

// NPC state
const showCreateNpcWizard = ref(false)

// NPCs for the current campaign (filter to show relevant ones)
const moduleNpcs = computed(() => {
  if (!props.campaign?.id) return []
  return characterStore.characters.filter(c =>
    c.campaign_id === props.campaign!.id && c.is_npc === 1
  )
})

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

  // Set up session notes path
  if (props.campaign?.directory_path && mod.module_number) {
    setNotesFilePath(buildNotesFilePath(props.campaign.directory_path, mod.module_number))
  }

  await Promise.all([
    loadModuleDocuments(),
    loadModuleMaps(),
    loadModuleMonsters(),
    loadNpcs(),
    loadNotes()
  ])
}

// Load documents for selected module
async function loadModuleDocuments() {
  if (!selectedModule.value || !props.campaign?.id) return

  try {
    moduleDocuments.value = await DocumentService.list(selectedModule.value.id, props.campaign.id)
  } catch (e) {
    console.error('Failed to load module documents:', e)
  }
}

// Load maps for selected module
async function loadModuleMaps() {
  if (!selectedModule.value || !props.campaign) return

  loadingMaps.value = true
  try {
    const response = await invoke<{ success: boolean; data?: MapData[] }>('list_maps', {
      request: { campaign_id: props.campaign.id, module_id: selectedModule.value.id }
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
async function handleCreateModule(data: { name: string; type: string; sessions: number }) {
  if (!props.campaign?.id) return

  try {
    const newModule = await ModuleService.create({
      campaign_id: props.campaign.id,
      name: data.name,
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

// Handle document updated
function handleDocumentUpdated(doc: Document) {
  const index = moduleDocuments.value.findIndex(d => d.id === doc.id)
  if (index !== -1) {
    moduleDocuments.value[index] = doc
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

// View character detail
function viewCharacter(character: Character) {
  router.push(`/characters/${character.id}`)
}

// Handle NPC created
async function handleNpcCreated() {
  showCreateNpcWizard.value = false
  await characterStore.fetchAllCharacters()
}

// Load NPCs when selecting module
async function loadNpcs() {
  await characterStore.fetchAllCharacters()
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
    const moduleToSelect = modules.value.find(m => m.id === Number(selectId))
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
    const moduleToSelect = modules.value.find(m => m.id === Number(selectId))
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
  border-right: 1px solid var(--color-border, #333);
  overflow-y: auto;
  background: var(--color-surface, #1a1a1a);
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

.modules-loading,
.modules-empty,
.maps-loading,
.maps-empty {
  padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
  font-size: 0.8rem;
  color: var(--color-text-muted);
}

.modules-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--spacing-xs, 4px);
}

.module-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
  border-radius: 4px;
}

.module-item:hover {
  background: var(--color-surface-variant, #252525);
}

.module-item.selected {
  background: var(--color-primary-900, #1e3a5f);
}

.module-number {
  font-weight: 600;
  color: var(--color-text-muted);
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
  border-top: 1px solid var(--color-border, #333);
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
}

.maps-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px) 0;
}

.maps-header h4 {
  margin: 0;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--color-text-muted, #666);
  text-transform: uppercase;
  letter-spacing: 0.05em;
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

/* Module Dashboard */
.module-dashboard {
  padding: var(--spacing-lg, 16px);
  overflow-y: auto;
  height: 100%;
}

.module-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-lg, 16px);
  padding-bottom: var(--spacing-md, 12px);
  border-bottom: 1px solid var(--color-border);
}

.module-title h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.module-actions {
  display: flex;
  gap: var(--spacing-sm, 8px);
}

/* Dashboard Grid - Two Column Layout */
.dashboard-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md, 12px);
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.dashboard-left {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md, 12px);
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
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-md, 8px);
  padding: var(--spacing-md, 12px);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-sm, 8px);
  padding-bottom: var(--spacing-xs, 4px);
  border-bottom: 1px solid var(--color-border, #333);
}

.section-header h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.section-empty,
.section-loading {
  font-size: 0.75rem;
  color: var(--color-text-muted, #888);
  text-align: center;
  padding: var(--spacing-md, 12px);
}

/* NPC Cards */
.npc-cards {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs, 4px);
}

.npc-card {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  background: var(--color-surface-variant, #252525);
  border: 1px solid var(--color-border, #333);
  border-left: 3px solid var(--color-warning, #f59e0b);
  border-radius: var(--radius-sm, 4px);
  cursor: pointer;
  transition: all 0.15s;
}

.npc-card:hover {
  border-color: var(--color-primary, #4a9eff);
  border-left-color: var(--color-warning, #f59e0b);
}

.npc-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.npc-role {
  font-size: 0.65rem;
  color: var(--color-text-muted, #888);
}

/* Monster List (grouped by encounter tag) */
.monster-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm, 8px);
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
  color: var(--color-error, #dc3545);
  padding: var(--spacing-xs, 4px) 0;
  border-bottom: 1px solid var(--color-border, #333);
  margin-bottom: var(--spacing-xs, 4px);
}

.monster-group-header.untagged {
  color: var(--color-text-muted, #888);
}

.monster-group-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.monster-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  background: var(--color-surface-variant, #252525);
  border-radius: var(--radius-sm, 4px);
  cursor: pointer;
  transition: all 0.15s;
}

.monster-row:hover {
  background: var(--color-primary-900, #1e3a5f);
}

.monster-row.active {
  background: var(--color-primary-900, #1e3a5f);
  border-left: 3px solid var(--color-error, #dc3545);
  padding-left: calc(var(--spacing-sm, 8px) - 3px);
}

.monster-qty {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--color-error, #dc3545);
  min-width: 24px;
}

.monster-name {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text, #e0e0e0);
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

/* Map Cards */
.map-cards {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs, 4px);
}

.map-card {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  background: var(--color-surface-variant, #252525);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-sm, 4px);
  cursor: pointer;
  transition: all 0.15s;
}

.map-card:hover {
  border-color: var(--color-primary, #4a9eff);
}

.map-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.map-size {
  font-size: 0.65rem;
  color: var(--color-text-muted, #888);
}

/* Document Cards */
.document-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs, 4px);
}

.document-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  background: var(--color-surface-variant, #252525);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-sm, 4px);
  cursor: pointer;
  transition: all 0.15s;
}

.document-card:hover {
  border-color: var(--color-primary, #4a9eff);
}

.doc-title {
  font-size: 0.8rem;
  color: var(--color-text, #e0e0e0);
}

.doc-status {
  font-size: 0.6rem;
  text-transform: uppercase;
  padding: 2px 4px;
  border-radius: 2px;
}

.doc-status.complete {
  background: var(--color-success-900, rgba(16, 185, 129, 0.2));
  color: var(--color-success, #10b981);
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
  margin: 0 0 var(--spacing-md, 12px) 0;
  font-size: 0.875rem;
}

/* Session Notes Panel */
.notes-panel {
  background: var(--color-surface, #1a1a1a);
  border-top: 1px solid var(--color-border, #333);
  display: flex;
  flex-direction: column;
  transition: height 0.3s ease;
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
  gap: var(--spacing-sm, 8px);
  padding: var(--spacing-xs, 4px) var(--spacing-md, 12px);
  background: var(--color-surface-variant, #252525);
  border: none;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--color-text, #e0e0e0);
  text-align: left;
  width: 100%;
}

.notes-toggle:hover {
  background: var(--color-base-300, #333);
}

.notes-toggle-icon {
  font-size: 0.65rem;
  color: var(--color-text-muted, #888);
}

.notes-toggle-label {
  flex: 1;
}

.notes-saving {
  font-size: 0.7rem;
  color: var(--color-warning, #f59e0b);
  font-style: italic;
}

.notes-saved {
  font-size: 0.7rem;
  color: var(--color-success, #10b981);
}

.notes-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.notes-textarea {
  flex: 1;
  padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
  border: none;
  resize: none;
  font-family: inherit;
  font-size: 0.85rem;
  line-height: 1.5;
  background: var(--color-surface, #1a1a1a);
  color: var(--color-text, #e0e0e0);
  overflow-y: auto;
}

.notes-textarea:focus {
  outline: none;
}

.notes-textarea::placeholder {
  color: var(--color-text-muted, #888);
  font-style: italic;
}
</style>
