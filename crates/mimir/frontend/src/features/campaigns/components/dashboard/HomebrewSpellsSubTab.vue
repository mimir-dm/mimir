<template>
  <div class="homebrew-spells">
    <!-- Header -->
    <div class="homebrew-tab-header">
      <h2>Homebrew Spells</h2>
      <div class="homebrew-header-actions">
        <button @click="openCloneFromCatalog" class="btn btn-secondary btn-sm">
          Clone from Catalog
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="homebrew-loading-state">Loading homebrew spells...</div>

    <!-- Empty state -->
    <div v-else-if="spells.length === 0" class="homebrew-empty-state">
      <div class="homebrew-empty-icon">&#10024;</div>
      <h3>No homebrew spells yet</h3>
      <p>Clone a spell from the catalog and customize it.</p>
      <button @click="openCloneFromCatalog" class="btn btn-primary">Clone from Catalog</button>
    </div>

    <!-- Spell list + detail -->
    <div v-else class="homebrew-layout">
      <div class="homebrew-list">
        <div
          v-for="spell in spells"
          :key="spell.id"
          class="homebrew-card"
          :class="{ selected: selectedSpell?.id === spell.id }"
          @click="selectSpell(spell)"
        >
          <div class="homebrew-card-header">
            <span class="homebrew-card-name">{{ spell.name }}</span>
            <span v-if="spell.level !== null" class="homebrew-card-badge">{{ levelLabel(spell.level) }}</span>
          </div>
          <div class="homebrew-card-meta">
            <span v-if="spell.school" class="homebrew-card-school">{{ spell.school }}</span>
            <span v-if="spell.cloned_from_name" class="homebrew-card-cloned">
              Based on {{ spell.cloned_from_name }}
            </span>
          </div>
        </div>
      </div>

      <!-- Detail pane -->
      <div v-if="selectedSpell" class="homebrew-detail">
        <div class="homebrew-detail-header">
          <h3>{{ selectedSpell.name }}</h3>
          <div class="homebrew-detail-actions">
            <button @click="startEditing" class="btn btn-secondary btn-sm">Edit</button>
            <button @click="confirmDelete" class="btn btn-danger btn-sm">Delete</button>
          </div>
        </div>
        <div v-if="selectedSpell.cloned_from_name" class="homebrew-detail-cloned">
          Based on {{ selectedSpell.cloned_from_name }} ({{ selectedSpell.cloned_from_source }})
        </div>
        <!-- Spell stat block preview -->
        <SpellStatBlock v-if="parsedSpellData" :data="parsedSpellData" :name="selectedSpell.name" />
        <details class="homebrew-raw-json-toggle">
          <summary>Raw JSON</summary>
          <pre class="homebrew-data-json">{{ formatData(selectedSpell.data) }}</pre>
        </details>
      </div>
      <div v-else class="homebrew-detail empty-detail">
        <p>Select a spell to view details</p>
      </div>
    </div>

    <!-- Edit Modal -->
    <div v-if="editingSpell" class="modal-overlay" @click.self="closeForm">
      <div class="modal-content homebrew-modal-lg">
        <h3>Edit Spell</h3>
        <form @submit.prevent="saveSpell">
          <div class="form-group">
            <label class="form-label required">Name</label>
            <input v-model="form.name" class="form-input" type="text" required placeholder="e.g. Arcane Burst" />
          </div>
          <div class="form-group">
            <label class="form-label">Spell Data (JSON)</label>
            <textarea
              v-model="form.data"
              class="form-textarea homebrew-json-editor"
              rows="20"
              placeholder='{"level": 3, "school": "V", ...}'
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
        <h3>Clone Spell from Catalog</h3>
        <p class="homebrew-clone-hint">Search the spell catalog, then edit the cloned spell's JSON.</p>
        <div class="form-group">
          <input
            v-model="cloneSearch"
            class="form-input"
            type="text"
            placeholder="Search spells by name..."
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
              <span v-if="result.level !== undefined" class="homebrew-card-badge">{{ levelLabel(result.level) }}</span>
            </div>
            <div class="homebrew-card-meta">
              <span v-if="result.school" class="homebrew-card-school">{{ result.school }}</span>
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
        <h3>Delete Spell</h3>
        <p>Are you sure you want to delete <strong>{{ selectedSpell?.name }}</strong>? This cannot be undone.</p>
        <div class="form-actions">
          <button class="btn btn-secondary" @click="cancelDelete">Cancel</button>
          <button class="btn btn-danger" @click="deleteSpell" :disabled="saving">
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
import { HomebrewSpellService, type HomebrewSpell } from '@/services/HomebrewSpellService'
import SpellStatBlock from './SpellStatBlock.vue'
import { dataEvents } from '@/utils/dataEvents'
import type { Campaign } from '@/types'
import type { ApiResponse } from '@/types/api'

const props = defineProps<{
  campaign?: Campaign
}>()

const loading = ref(false)
const saving = ref(false)
const spells = ref<HomebrewSpell[]>([])
const selectedSpell = ref<HomebrewSpell | null>(null)
const editingSpell = ref<HomebrewSpell | null>(null)
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
const cloneResults = ref<Array<{ name: string; source: string; level?: number; school?: string; data: Record<string, unknown> }>>([])
let cloneSearchTimer: ReturnType<typeof setTimeout> | null = null

const SCHOOL_LABELS: Record<string, string> = {
  A: 'Abjuration', C: 'Conjuration', D: 'Divination', E: 'Enchantment',
  V: 'Evocation', I: 'Illusion', N: 'Necromancy', T: 'Transmutation',
}

const parsedSpellData = computed(() => {
  if (!selectedSpell.value) return null
  try {
    return JSON.parse(selectedSpell.value.data)
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

function levelLabel(level: number | null): string {
  if (level === null) return ''
  if (level === 0) return 'Cantrip'
  return `Level ${level}`
}

function schoolLabel(code: string): string {
  return SCHOOL_LABELS[code] || code
}

function formatData(data: string): string {
  try {
    return JSON.stringify(JSON.parse(data), null, 2)
  } catch {
    return data
  }
}

async function loadSpells() {
  if (!props.campaign?.id) return
  loading.value = true
  try {
    spells.value = await HomebrewSpellService.list(props.campaign.id)
  } catch (e) {
    console.error('Failed to load homebrew spells:', e)
  } finally {
    loading.value = false
  }
}

function selectSpell(spell: HomebrewSpell) {
  selectedSpell.value = spell
}

function startEditing() {
  if (!selectedSpell.value) return
  editingSpell.value = selectedSpell.value
  form.value = {
    name: selectedSpell.value.name,
    data: formatData(selectedSpell.value.data),
  }
}

function closeForm() {
  editingSpell.value = null
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
    const response = await invoke<ApiResponse<Array<Record<string, unknown>>>>('search_spells', {
      filter: { name_contains: cloneSearch.value },
      limit: 20,
      offset: 0,
    })
    if (response.success && response.data) {
      cloneResults.value = response.data.map((s: Record<string, unknown>) => {
        const level = typeof s.level === 'number' ? s.level : undefined
        const school = typeof s.school === 'string' ? schoolLabel(s.school) : undefined

        return {
          name: (s.name as string) || '',
          source: (s.source as string) || '',
          level,
          school,
          data: s,
        }
      })
    }
  } catch (e) {
    console.error('Spell catalog search failed:', e)
    cloneResults.value = []
  } finally {
    cloneSearching.value = false
  }
}

function extractLevel(data: Record<string, unknown>): number | undefined {
  if (typeof data.level === 'number') return data.level
  return undefined
}

function extractSchool(data: Record<string, unknown>): string | undefined {
  if (typeof data.school === 'string') return data.school
  return undefined
}

async function selectCloneResult(result: { name: string; source: string; level?: number; school?: string; data: Record<string, unknown> }) {
  showCloneModal.value = false

  try {
    const spell = await HomebrewSpellService.create({
      campaign_id: props.campaign!.id,
      name: result.name,
      level: extractLevel(result.data),
      school: extractSchool(result.data),
      data: JSON.stringify(result.data),
      cloned_from_name: result.name,
      cloned_from_source: result.source,
    })
    await loadSpells()
    selectedSpell.value = spell
  } catch (e: any) {
    console.error('Failed to clone spell:', e)
    if (e.message?.includes('UNIQUE')) {
      try {
        const spell = await HomebrewSpellService.create({
          campaign_id: props.campaign!.id,
          name: `${result.name} (Custom)`,
          level: extractLevel(result.data),
          school: extractSchool(result.data),
          data: JSON.stringify(result.data),
          cloned_from_name: result.name,
          cloned_from_source: result.source,
        })
        await loadSpells()
        selectedSpell.value = spell
      } catch (e2) {
        console.error('Failed to clone spell with suffix:', e2)
      }
    }
  }
}

// Delete
function cancelDelete() {
  showDeleteConfirm.value = false
}

async function confirmDelete() {
  if (!selectedSpell.value) return
  showDeleteConfirm.value = true
}

async function deleteSpell() {
  if (!selectedSpell.value) return
  saving.value = true
  try {
    await HomebrewSpellService.delete(selectedSpell.value.id)
    selectedSpell.value = null
    showDeleteConfirm.value = false
    await loadSpells()
  } catch (e) {
    console.error('Failed to delete homebrew spell:', e)
  } finally {
    saving.value = false
  }
}

async function saveSpell() {
  formError.value = ''
  if (jsonError.value) return
  saving.value = true

  try {
    const parsed = JSON.parse(form.value.data)
    await HomebrewSpellService.update(editingSpell.value!.id, {
      name: form.value.name,
      level: extractLevel(parsed) ?? null,
      school: extractSchool(parsed) ?? null,
      data: JSON.stringify(parsed),
    })
    closeForm()
    await loadSpells()
    const updated = spells.value.find(s => s.id === editingSpell.value?.id)
    if (updated) selectedSpell.value = updated
  } catch (e: any) {
    console.error('Failed to save homebrew spell:', e)
    formError.value = e.message || 'Failed to save'
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadSpells()

  const unsub1 = dataEvents.on('homebrew-spell:created', () => loadSpells())
  const unsub2 = dataEvents.on('homebrew-spell:updated', () => loadSpells())
  const unsub3 = dataEvents.on('homebrew-spell:deleted', () => loadSpells())

  onUnmounted(() => {
    unsub1()
    unsub2()
    unsub3()
  })
})

watch(() => props.campaign?.id, () => {
  selectedSpell.value = null
  loadSpells()
})
</script>

<style scoped>
/* Container */
.homebrew-spells {
  display: flex;
  flex-direction: column;
  height: 100%;
}

/* Spell-specific: form actions */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-md);
}
</style>
