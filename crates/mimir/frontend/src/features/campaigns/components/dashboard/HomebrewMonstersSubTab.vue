<template>
  <div class="homebrew-monsters">
    <!-- Header -->
    <div class="homebrew-tab-header">
      <h2>Homebrew Monsters</h2>
      <div class="homebrew-header-actions">
        <button @click="openCloneFromCatalog" class="btn btn-secondary btn-sm">
          Clone from Catalog
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="homebrew-loading-state">Loading homebrew monsters...</div>

    <!-- Empty state -->
    <div v-else-if="monsters.length === 0" class="homebrew-empty-state">
      <div class="homebrew-empty-icon">&#128009;</div>
      <h3>No homebrew monsters yet</h3>
      <p>Clone a monster from the catalog and customize it.</p>
      <button @click="openCloneFromCatalog" class="btn btn-primary">Clone from Catalog</button>
    </div>

    <!-- Monster list + detail -->
    <div v-else class="homebrew-layout">
      <div class="homebrew-list">
        <div
          v-for="monster in monsters"
          :key="monster.id"
          class="homebrew-card"
          :class="{ selected: selectedMonster?.id === monster.id }"
          @click="selectMonster(monster)"
        >
          <div class="homebrew-card-header">
            <span class="homebrew-card-name">{{ monster.name }}</span>
            <span v-if="monster.cr" class="homebrew-card-badge">CR {{ monster.cr }}</span>
          </div>
          <div class="homebrew-card-meta">
            <span v-if="monster.size" class="homebrew-card-size">{{ sizeLabel(monster.size) }}</span>
            <span v-if="monster.creature_type" class="homebrew-card-type">{{ monster.creature_type }}</span>
            <span v-if="monster.cloned_from_name" class="homebrew-card-cloned">
              Based on {{ monster.cloned_from_name }}
            </span>
          </div>
        </div>
      </div>

      <!-- Detail pane -->
      <div v-if="selectedMonster" class="homebrew-detail">
        <div class="homebrew-detail-header">
          <h3>{{ selectedMonster.name }}</h3>
          <div class="homebrew-detail-actions">
            <button @click="startEditing" class="btn btn-secondary btn-sm">Edit</button>
            <button @click="confirmDelete" class="btn btn-danger btn-sm">Delete</button>
          </div>
        </div>
        <div v-if="selectedMonster.cloned_from_name" class="homebrew-detail-cloned">
          Based on {{ selectedMonster.cloned_from_name }} ({{ selectedMonster.cloned_from_source }})
        </div>
        <!-- Stat block preview -->
        <MonsterStatBlock v-if="parsedMonsterData" :data="parsedMonsterData" :name="selectedMonster.name" />
        <details class="homebrew-raw-json-toggle">
          <summary>Raw JSON</summary>
          <pre class="homebrew-data-json">{{ formatData(selectedMonster.data) }}</pre>
        </details>
      </div>
      <div v-else class="homebrew-detail empty-detail">
        <p>Select a monster to view details</p>
      </div>
    </div>

    <!-- Edit Modal -->
    <div v-if="editingMonster" class="modal-overlay" @click.self="closeForm">
      <div class="modal-content homebrew-modal-lg">
        <h3>Edit Monster</h3>
        <form @submit.prevent="saveMonster">
          <div class="form-group">
            <label class="form-label required">Name</label>
            <input v-model="form.name" class="form-input" type="text" required placeholder="e.g. Frost Wight" />
          </div>
          <div class="form-group">
            <label class="form-label">Monster Data (JSON)</label>
            <textarea
              v-model="form.data"
              class="form-textarea homebrew-json-editor"
              rows="20"
              placeholder='{"str": 16, "dex": 12, ...}'
            ></textarea>
          </div>
          <div v-if="jsonError" class="form-help is-invalid">{{ jsonError }}</div>
          <div v-if="formError" class="form-help is-invalid">{{ formError }}</div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" @click="closeForm">Cancel</button>
            <button type="submit" class="btn btn-primary" :disabled="saving">
              {{ saving ? 'Saving...' : 'Update' }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Clone from Catalog Modal -->
    <div v-if="showCloneModal" class="modal-overlay" @click.self="showCloneModal = false">
      <div class="modal-content">
        <h3>Clone Monster from Catalog</h3>
        <p class="homebrew-clone-hint">Search the monster catalog, then edit the cloned monster's JSON.</p>
        <div class="form-group">
          <input
            v-model="cloneSearch"
            class="form-input"
            type="text"
            placeholder="Search monsters by name..."
            @input="debouncedCatalogSearch"
          />
        </div>
        <div v-if="cloneSearching" class="homebrew-clone-status">Searching...</div>
        <div v-else-if="cloneResults.length > 0" class="homebrew-clone-results">
          <div
            v-for="result in cloneResults"
            :key="`${result.name}-${result.source}`"
            class="homebrew-clone-result-card"
            @click="selectCloneResult(result)"
          >
            <div class="homebrew-card-header">
              <span class="homebrew-card-name">{{ result.name }}</span>
              <span v-if="result.cr" class="homebrew-card-badge">CR {{ result.cr }}</span>
            </div>
            <div class="homebrew-card-meta">
              <span v-if="result.size" class="homebrew-card-size">{{ sizeLabel(result.size) }}</span>
              <span v-if="result.type" class="homebrew-card-type">{{ result.type }}</span>
              <span class="homebrew-card-source">{{ result.source }}</span>
            </div>
          </div>
        </div>
        <div v-else-if="cloneSearch.length >= 2" class="homebrew-clone-status">No results found</div>
        <div class="form-actions">
          <button type="button" class="btn btn-secondary" @click="showCloneModal = false">Cancel</button>
        </div>
      </div>
    </div>

    <!-- Delete confirmation -->
    <div v-if="showDeleteConfirm" class="modal-overlay" @click.self="cancelDelete">
      <div class="modal-content modal-sm">
        <h3>Delete Monster</h3>
        <div v-if="deleteWarningModules.length > 0" class="homebrew-delete-warning">
          <p>This monster is referenced in the following modules:</p>
          <ul>
            <li v-for="name in deleteWarningModules" :key="name"><strong>{{ name }}</strong></li>
          </ul>
          <p>Deleting this homebrew monster will <strong>not</strong> remove module references, but the custom data will no longer be available.</p>
        </div>
        <p>Are you sure you want to delete <strong>{{ selectedMonster?.name }}</strong>? This cannot be undone.</p>
        <div class="form-actions">
          <button class="btn btn-secondary" @click="cancelDelete">Cancel</button>
          <button class="btn btn-danger" @click="deleteMonster" :disabled="saving">
            {{ saving ? 'Deleting...' : 'Delete' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { HomebrewMonsterService, type HomebrewMonster } from '@/services/HomebrewMonsterService'
import MonsterStatBlock from './MonsterStatBlock.vue'
import { dataEvents } from '@/utils/dataEvents'
import type { Campaign } from '@/types'
import type { ApiResponse } from '@/types/api'

const props = defineProps<{
  campaign?: Campaign
}>()

const loading = ref(false)
const saving = ref(false)
const monsters = ref<HomebrewMonster[]>([])
const selectedMonster = ref<HomebrewMonster | null>(null)
const editingMonster = ref<HomebrewMonster | null>(null)
const showDeleteConfirm = ref(false)
const formError = ref('')

interface FormState {
  name: string
  data: string
}

const form = ref<FormState>({ name: '', data: '{}' })

// Clone from catalog state
const showCloneModal = ref(false)
const cloneSearch = ref('')
const cloneSearching = ref(false)
const cloneResults = ref<Array<{ name: string; source: string; cr?: string; size?: string; type?: string; data: Record<string, unknown> }>>([])
let cloneSearchTimer: ReturnType<typeof setTimeout> | null = null

// Delete warning state
const deleteWarningModules = ref<string[]>([])

const parsedMonsterData = computed(() => {
  if (!selectedMonster.value) return null
  try {
    return JSON.parse(selectedMonster.value.data)
  } catch {
    return null
  }
})

const jsonError = computed(() => {
  try {
    JSON.parse(form.value.data)
    return ''
  } catch (e: any) {
    return `Invalid JSON: ${e.message}`
  }
})

const SIZE_LABELS: Record<string, string> = {
  T: 'Tiny', S: 'Small', M: 'Medium', L: 'Large', H: 'Huge', G: 'Gargantuan',
}

function sizeLabel(size: string): string {
  return SIZE_LABELS[size] || size
}

function formatData(data: string): string {
  try {
    return JSON.stringify(JSON.parse(data), null, 2)
  } catch {
    return data
  }
}

async function loadMonsters() {
  if (!props.campaign?.id) return
  loading.value = true
  try {
    monsters.value = await HomebrewMonsterService.list(props.campaign.id)
  } catch (e) {
    console.error('Failed to load homebrew monsters:', e)
  } finally {
    loading.value = false
  }
}

function selectMonster(monster: HomebrewMonster) {
  selectedMonster.value = monster
}

function startEditing() {
  if (!selectedMonster.value) return
  editingMonster.value = selectedMonster.value
  form.value = {
    name: selectedMonster.value.name,
    data: formatData(selectedMonster.value.data),
  }
}

function closeForm() {
  editingMonster.value = null
  formError.value = ''
  form.value = { name: '', data: '{}' }
}

// Clone from catalog
function openCloneFromCatalog() {
  cloneSearch.value = ''
  cloneResults.value = []
  showCloneModal.value = true
}

function debouncedCatalogSearch() {
  if (cloneSearchTimer) clearTimeout(cloneSearchTimer)
  if (cloneSearch.value.length < 2) {
    cloneResults.value = []
    return
  }
  cloneSearchTimer = setTimeout(() => searchCatalog(), 300)
}

async function searchCatalog() {
  cloneSearching.value = true
  try {
    const response = await invoke<ApiResponse<Array<Record<string, unknown>>>>('search_monsters', {
      filter: { nameContains: cloneSearch.value },
      limit: 20,
      offset: 0,
    })
    if (response.success && response.data) {
      cloneResults.value = response.data.map((m: Record<string, unknown>) => {
        // Extract CR from the data
        let cr: string | undefined
        const crVal = m.cr
        if (typeof crVal === 'string') cr = crVal
        else if (crVal && typeof crVal === 'object' && typeof (crVal as any).cr === 'string') cr = (crVal as any).cr

        // Extract creature type
        let type: string | undefined
        const typeVal = m.type
        if (typeof typeVal === 'string') type = typeVal
        else if (typeVal && typeof typeVal === 'object' && typeof (typeVal as any).type === 'string') type = (typeVal as any).type

        // Extract size
        let size: string | undefined
        const sizeVal = m.size
        if (Array.isArray(sizeVal) && sizeVal.length > 0) size = sizeVal[0] as string
        else if (typeof sizeVal === 'string') size = sizeVal

        return {
          name: (m.name as string) || '',
          source: (m.source as string) || '',
          cr,
          size,
          type,
          data: m,
        }
      })
    }
  } catch (e) {
    console.error('Monster catalog search failed:', e)
    cloneResults.value = []
  } finally {
    cloneSearching.value = false
  }
}

function extractCrString(data: Record<string, unknown>): string | undefined {
  const cr = data.cr
  if (typeof cr === 'string') return cr
  if (cr && typeof cr === 'object' && typeof (cr as any).cr === 'string') return (cr as any).cr
  return undefined
}

function extractCreatureType(data: Record<string, unknown>): string | undefined {
  const t = data.type
  if (typeof t === 'string') return t
  if (t && typeof t === 'object' && typeof (t as any).type === 'string') return (t as any).type
  return undefined
}

function extractSize(data: Record<string, unknown>): string | undefined {
  const s = data.size
  if (Array.isArray(s) && s.length > 0) return s[0] as string
  if (typeof s === 'string') return s
  return undefined
}

async function selectCloneResult(result: { name: string; source: string; cr?: string; size?: string; type?: string; data: Record<string, unknown> }) {
  showCloneModal.value = false

  const dataJson = JSON.stringify(result.data, null, 2)

  try {
    const monster = await HomebrewMonsterService.create({
      campaign_id: props.campaign!.id,
      name: result.name,
      cr: extractCrString(result.data),
      creature_type: extractCreatureType(result.data),
      size: extractSize(result.data),
      data: JSON.stringify(result.data),
      cloned_from_name: result.name,
      cloned_from_source: result.source,
    })
    await loadMonsters()
    selectedMonster.value = monster
  } catch (e: any) {
    console.error('Failed to clone monster:', e)
    // If name conflict, try with suffix
    if (e.message?.includes('UNIQUE')) {
      try {
        const monster = await HomebrewMonsterService.create({
          campaign_id: props.campaign!.id,
          name: `${result.name} (Custom)`,
          cr: extractCrString(result.data),
          creature_type: extractCreatureType(result.data),
          size: extractSize(result.data),
          data: JSON.stringify(result.data),
          cloned_from_name: result.name,
          cloned_from_source: result.source,
        })
        await loadMonsters()
        selectedMonster.value = monster
      } catch (e2) {
        console.error('Failed to clone monster with suffix:', e2)
      }
    }
  }
}

// Delete
function cancelDelete() {
  showDeleteConfirm.value = false
  deleteWarningModules.value = []
}

async function confirmDelete() {
  if (!selectedMonster.value || !props.campaign?.id) return
  deleteWarningModules.value = []

  try {
    // Check if any modules reference this monster
    const modResponse = await invoke<ApiResponse<Array<{ id: string; name: string }>>>('list_modules', {
      campaignId: props.campaign.id,
    })
    if (modResponse.success && modResponse.data) {
      const affected: string[] = []
      for (const mod of modResponse.data) {
        const monstersResponse = await invoke<ApiResponse<Array<{ monster_name: string; monster_source: string }>>>('list_module_monsters_with_data', {
          moduleId: mod.id,
        })
        if (monstersResponse.success && monstersResponse.data) {
          const hasMonster = monstersResponse.data.some(
            (mm) => mm.monster_name === selectedMonster.value!.name && mm.monster_source === 'HB'
          )
          if (hasMonster) affected.push(mod.name)
        }
      }
      deleteWarningModules.value = affected
    }
  } catch (e) {
    console.error('Failed to check module usage:', e)
  }

  showDeleteConfirm.value = true
}

async function deleteMonster() {
  if (!selectedMonster.value) return
  saving.value = true
  try {
    await HomebrewMonsterService.delete(selectedMonster.value.id)
    selectedMonster.value = null
    showDeleteConfirm.value = false
    await loadMonsters()
  } catch (e) {
    console.error('Failed to delete homebrew monster:', e)
  } finally {
    saving.value = false
  }
}

async function saveMonster() {
  formError.value = ''
  if (jsonError.value) return
  saving.value = true

  try {
    const parsed = JSON.parse(form.value.data)
    await HomebrewMonsterService.update(editingMonster.value!.id, {
      name: form.value.name,
      cr: extractCrString(parsed),
      creature_type: extractCreatureType(parsed),
      size: extractSize(parsed),
      data: JSON.stringify(parsed),
    })
    closeForm()
    await loadMonsters()
    // Re-select the updated monster
    const updated = monsters.value.find(m => m.id === editingMonster.value?.id)
    if (updated) selectedMonster.value = updated
  } catch (e: any) {
    console.error('Failed to save homebrew monster:', e)
    formError.value = e.message || 'Failed to save'
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadMonsters()

  const unsub1 = dataEvents.on('homebrew-monster:created', () => loadMonsters())
  const unsub2 = dataEvents.on('homebrew-monster:updated', () => loadMonsters())
  const unsub3 = dataEvents.on('homebrew-monster:deleted', () => loadMonsters())

  onUnmounted(() => {
    unsub1()
    unsub2()
    unsub3()
  })
})

watch(() => props.campaign?.id, () => {
  selectedMonster.value = null
  loadMonsters()
})
</script>

<style scoped>
/* Container */
.homebrew-monsters {
  display: flex;
  flex-direction: column;
  height: 100%;
}

/* Monster-specific: form actions */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-md);
}
</style>
