<template>
  <div class="homebrew-spells">
    <!-- Header -->
    <div class="tab-header">
      <h2>Homebrew Spells</h2>
      <div class="header-actions">
        <button @click="openCloneFromCatalog" class="btn btn-secondary btn-sm">
          Clone from Catalog
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="loading-state">Loading homebrew spells...</div>

    <!-- Empty state -->
    <div v-else-if="spells.length === 0" class="empty-state">
      <div class="empty-icon">&#10024;</div>
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
          <div class="card-header">
            <span class="card-name">{{ spell.name }}</span>
            <span v-if="spell.level !== null" class="card-level">{{ levelLabel(spell.level) }}</span>
          </div>
          <div class="card-meta">
            <span v-if="spell.school" class="card-school">{{ spell.school }}</span>
            <span v-if="spell.cloned_from_name" class="card-cloned">
              Based on {{ spell.cloned_from_name }}
            </span>
          </div>
        </div>
      </div>

      <!-- Detail pane -->
      <div v-if="selectedSpell" class="homebrew-detail">
        <div class="detail-header">
          <h3>{{ selectedSpell.name }}</h3>
          <div class="detail-actions">
            <button @click="startEditing" class="btn btn-secondary btn-sm">Edit</button>
            <button @click="confirmDelete" class="btn btn-danger btn-sm">Delete</button>
          </div>
        </div>
        <div v-if="selectedSpell.cloned_from_name" class="detail-cloned">
          Based on {{ selectedSpell.cloned_from_name }} ({{ selectedSpell.cloned_from_source }})
        </div>
        <!-- Spell stat block preview -->
        <SpellStatBlock v-if="parsedSpellData" :data="parsedSpellData" :name="selectedSpell.name" />
        <details class="raw-json-toggle">
          <summary>Raw JSON</summary>
          <pre class="data-json">{{ formatData(selectedSpell.data) }}</pre>
        </details>
      </div>
      <div v-else class="homebrew-detail empty-detail">
        <p>Select a spell to view details</p>
      </div>
    </div>

    <!-- Edit Modal -->
    <div v-if="editingSpell" class="modal-overlay" @click.self="closeForm">
      <div class="modal-content modal-lg">
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
              class="form-textarea json-editor"
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
        <p class="clone-hint">Search the spell catalog, then edit the cloned spell's JSON.</p>
        <div class="form-group">
          <input
            v-model="cloneSearch"
            class="form-input"
            type="text"
            placeholder="Search spells by name..."
            @input="debouncedCatalogSearch"
          />
        </div>
        <div v-if="cloneSearching" class="clone-status">Searching...</div>
        <div v-else-if="cloneResults.length > 0" class="clone-results">
          <div
            v-for="result in cloneResults"
            :key="`${result.name}-${result.source}`"
            class="clone-result-card"
            @click="selectCloneResult(result)"
          >
            <div class="card-header">
              <span class="card-name">{{ result.name }}</span>
              <span v-if="result.level !== undefined" class="card-level">{{ levelLabel(result.level) }}</span>
            </div>
            <div class="card-meta">
              <span v-if="result.school" class="card-school">{{ result.school }}</span>
              <span class="card-source">{{ result.source }}</span>
            </div>
          </div>
        </div>
        <div v-else-if="cloneSearch.length >= 2" class="clone-status">No results found</div>
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
.homebrew-spells {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.tab-header h2 {
  margin: 0;
  font-size: 1.25rem;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

.empty-state h3 {
  margin: var(--spacing-sm) 0;
}

.empty-icon {
  font-size: 2.5rem;
  opacity: 0.5;
}

/* Layout */
.homebrew-layout {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: var(--spacing-md);
  flex: 1;
  min-height: 0;
}

.homebrew-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  overflow-y: auto;
}

.homebrew-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: all 0.15s ease;
}

.homebrew-card:hover {
  background: var(--color-surface-hover);
}

.homebrew-card.selected {
  border-color: var(--color-primary-400);
  background: color-mix(in srgb, var(--color-primary-400) 12%, var(--color-surface));
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-name {
  font-weight: 600;
  font-size: 0.95rem;
}

.card-level {
  font-size: 0.75rem;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-variant);
  font-weight: 600;
}

.card-meta {
  display: flex;
  gap: var(--spacing-sm);
  margin-top: 2px;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.card-school {
  text-transform: capitalize;
}

.card-source {
  font-size: 0.75rem;
  opacity: 0.7;
}

/* Detail pane */
.homebrew-detail {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  overflow-y: auto;
  text-align: left;
}

.homebrew-detail.empty-detail {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.detail-header h3 {
  margin: 0;
  font-size: 1.2rem;
}

.detail-actions {
  display: flex;
  gap: var(--spacing-xs);
}

.detail-cloned {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-style: italic;
  margin-bottom: var(--spacing-md);
}

.raw-json-toggle {
  margin-top: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-sm);
}

.raw-json-toggle summary {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  cursor: pointer;
  user-select: none;
}

.data-json {
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  font-size: 0.8rem;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  margin: var(--spacing-sm) 0 0;
}

/* JSON editor */
.json-editor {
  font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 0.85rem;
  line-height: 1.5;
  resize: vertical;
  min-height: 300px;
}

/* Clone modal */
.clone-hint {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.clone-results {
  max-height: 300px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-md);
}

.clone-result-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: all 0.15s ease;
}

.clone-result-card:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-primary-400);
}

.clone-status {
  text-align: center;
  color: var(--color-text-secondary);
  padding: var(--spacing-md);
  font-size: 0.85rem;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-md);
}

.modal-lg {
  max-width: 700px;
  max-height: 85vh;
  overflow-y: auto;
}
</style>
