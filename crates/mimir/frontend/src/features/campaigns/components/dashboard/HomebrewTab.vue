<template>
  <div class="homebrew-tab">
    <!-- Header -->
    <div class="tab-header">
      <h2>Homebrew Items</h2>
      <div class="header-actions">
        <button @click="showCreateForm = true" class="btn btn-primary btn-sm">
          Create Item
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="loading-state">Loading homebrew items...</div>

    <!-- Empty state -->
    <div v-else-if="items.length === 0" class="empty-state">
      <div class="empty-icon">&#9881;</div>
      <h3>No homebrew items yet</h3>
      <p>Create custom items for your campaign.</p>
      <button @click="showCreateForm = true" class="btn btn-primary">Create Item</button>
    </div>

    <!-- Item list + detail -->
    <div v-else class="homebrew-layout">
      <div class="homebrew-list">
        <div
          v-for="item in items"
          :key="item.id"
          class="homebrew-card"
          :class="{ selected: selectedItem?.id === item.id }"
          @click="selectItem(item)"
        >
          <div class="card-header">
            <span class="card-name">{{ item.name }}</span>
            <span v-if="item.rarity" class="card-rarity" :class="rarityClass(item.rarity)">{{ item.rarity }}</span>
          </div>
          <div class="card-meta">
            <span v-if="item.item_type" class="card-type">{{ item.item_type }}</span>
            <span v-if="item.cloned_from_name" class="card-cloned">
              Based on {{ item.cloned_from_name }}
            </span>
          </div>
        </div>
      </div>

      <!-- Detail pane -->
      <div v-if="selectedItem" class="homebrew-detail">
        <div class="detail-header">
          <h3>{{ selectedItem.name }}</h3>
          <div class="detail-actions">
            <button @click="startEditing" class="btn btn-secondary btn-sm">Edit</button>
            <button @click="confirmDelete" class="btn btn-danger btn-sm">Delete</button>
          </div>
        </div>
        <div v-if="selectedItem.cloned_from_name" class="detail-cloned">
          Based on {{ selectedItem.cloned_from_name }} ({{ selectedItem.cloned_from_source }})
        </div>
        <ItemDetailBlock :detail="toItemDetail(selectedItem)" />
        <details class="raw-json-toggle">
          <summary>Raw JSON</summary>
          <pre class="data-json">{{ formatData(selectedItem.data) }}</pre>
        </details>
      </div>
      <div v-else class="homebrew-detail empty-detail">
        <p>Select an item to view details</p>
      </div>
    </div>

    <!-- Create/Edit Modal -->
    <div v-if="showCreateForm || editingItem" class="modal-overlay" @click.self="closeForm">
      <div class="modal-content">
        <h3>{{ editingItem ? 'Edit Item' : 'Create Homebrew Item' }}</h3>
        <form @submit.prevent="saveItem">
          <div class="form-group">
            <label class="form-label required">Name</label>
            <input v-model="form.name" class="form-input" type="text" required placeholder="e.g. Sword of Storms" />
          </div>
          <div class="form-row">
            <div class="form-group">
              <label class="form-label">Type</label>
              <select v-model="form.item_type" class="form-select">
                <option value="">— None —</option>
                <option value="weapon">Weapon</option>
                <option value="armor">Armor</option>
                <option value="potion">Potion</option>
                <option value="ring">Ring</option>
                <option value="rod">Rod</option>
                <option value="scroll">Scroll</option>
                <option value="staff">Staff</option>
                <option value="wand">Wand</option>
                <option value="wondrous item">Wondrous Item</option>
                <option value="adventuring gear">Adventuring Gear</option>
              </select>
            </div>
            <div class="form-group">
              <label class="form-label">Rarity</label>
              <select v-model="form.rarity" class="form-select">
                <option value="">— None —</option>
                <option value="common">Common</option>
                <option value="uncommon">Uncommon</option>
                <option value="rare">Rare</option>
                <option value="very rare">Very Rare</option>
                <option value="legendary">Legendary</option>
                <option value="artifact">Artifact</option>
              </select>
            </div>
          </div>
          <div class="form-group">
            <label class="form-label required">Data (JSON)</label>
            <textarea
              v-model="form.data"
              class="form-textarea"
              rows="12"
              required
              placeholder='{"description": "A magical sword..."}'
            ></textarea>
            <div v-if="jsonError" class="form-help is-invalid">{{ jsonError }}</div>
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" @click="closeForm">Cancel</button>
            <button type="submit" class="btn btn-primary" :disabled="saving">
              {{ saving ? 'Saving...' : (editingItem ? 'Update' : 'Create') }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Delete confirmation -->
    <div v-if="showDeleteConfirm" class="modal-overlay" @click.self="showDeleteConfirm = false">
      <div class="modal-content modal-sm">
        <h3>Delete Item</h3>
        <p>Are you sure you want to delete <strong>{{ selectedItem?.name }}</strong>? This cannot be undone.</p>
        <div class="form-actions">
          <button class="btn btn-secondary" @click="showDeleteConfirm = false">Cancel</button>
          <button class="btn btn-danger" @click="deleteItem" :disabled="saving">
            {{ saving ? 'Deleting...' : 'Delete' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { HomebrewService, type HomebrewItem } from '@/services/HomebrewService'
import ItemDetailBlock from '@/features/characters/components/sheet/ItemDetailBlock.vue'
import { dataEvents } from '@/utils/dataEvents'
import type { Campaign } from '@/types'

const props = defineProps<{
  campaign?: Campaign
  documents?: any[]
}>()

const loading = ref(false)
const saving = ref(false)
const items = ref<HomebrewItem[]>([])
const selectedItem = ref<HomebrewItem | null>(null)
const showCreateForm = ref(false)
const editingItem = ref<HomebrewItem | null>(null)
const showDeleteConfirm = ref(false)
const jsonError = ref('')

const form = ref({
  name: '',
  item_type: '',
  rarity: '',
  data: '{}'
})

async function loadItems() {
  if (!props.campaign?.id) return
  loading.value = true
  try {
    items.value = await HomebrewService.list(props.campaign.id)
  } catch (e) {
    console.error('Failed to load homebrew items:', e)
  } finally {
    loading.value = false
  }
}

function selectItem(item: HomebrewItem) {
  selectedItem.value = item
}

function rarityClass(rarity: string): string {
  return 'rarity-' + rarity.replace(/\s+/g, '-').toLowerCase()
}

function formatData(data: string): string {
  try {
    return JSON.stringify(JSON.parse(data), null, 2)
  } catch {
    return data
  }
}

function toItemDetail(item: HomebrewItem) {
  let parsed: Record<string, unknown> = {}
  try {
    parsed = JSON.parse(item.data)
  } catch { /* ignore */ }
  return {
    name: item.name,
    source: 'HB',
    item_type: item.item_type || (parsed.type as string | null) || null,
    rarity: item.rarity || (parsed.rarity as string | null) || null,
    data: parsed,
    fluff: null
  }
}

function startEditing() {
  if (!selectedItem.value) return
  editingItem.value = selectedItem.value
  form.value = {
    name: selectedItem.value.name,
    item_type: selectedItem.value.item_type || '',
    rarity: selectedItem.value.rarity || '',
    data: formatData(selectedItem.value.data)
  }
}

function closeForm() {
  showCreateForm.value = false
  editingItem.value = null
  jsonError.value = ''
  form.value = { name: '', item_type: '', rarity: '', data: '{}' }
}

async function saveItem() {
  // Validate JSON
  try {
    JSON.parse(form.value.data)
  } catch {
    jsonError.value = 'Invalid JSON'
    return
  }
  jsonError.value = ''
  saving.value = true

  try {
    if (editingItem.value) {
      await HomebrewService.update(editingItem.value.id, {
        name: form.value.name,
        item_type: form.value.item_type || null,
        rarity: form.value.rarity || null,
        data: form.value.data
      })
    } else {
      await HomebrewService.create({
        campaign_id: props.campaign!.id,
        name: form.value.name,
        item_type: form.value.item_type || undefined,
        rarity: form.value.rarity || undefined,
        data: form.value.data
      })
    }
    closeForm()
    await loadItems()
  } catch (e: any) {
    console.error('Failed to save homebrew item:', e)
    jsonError.value = e.message || 'Failed to save'
  } finally {
    saving.value = false
  }
}

function confirmDelete() {
  showDeleteConfirm.value = true
}

async function deleteItem() {
  if (!selectedItem.value) return
  saving.value = true
  try {
    await HomebrewService.delete(selectedItem.value.id)
    selectedItem.value = null
    showDeleteConfirm.value = false
    await loadItems()
  } catch (e) {
    console.error('Failed to delete homebrew item:', e)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadItems()

  const unsub1 = dataEvents.on('homebrew:created', () => loadItems())
  const unsub2 = dataEvents.on('homebrew:updated', () => loadItems())
  const unsub3 = dataEvents.on('homebrew:deleted', () => loadItems())

  onUnmounted(() => {
    unsub1()
    unsub2()
    unsub3()
  })
})

watch(() => props.campaign?.id, () => {
  selectedItem.value = null
  loadItems()
})
</script>

<style scoped>
.homebrew-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: var(--spacing-lg);
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
  background: var(--color-primary-50, #eff6ff);
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

.card-rarity {
  font-size: 0.75rem;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  text-transform: capitalize;
}

.card-meta {
  display: flex;
  gap: var(--spacing-sm);
  margin-top: 2px;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.card-type {
  text-transform: capitalize;
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

/* Rarity colors */
.rarity-common { background: #f3f4f6; color: #6b7280; }
.rarity-uncommon { background: #dcfce7; color: #16a34a; }
.rarity-rare { background: #dbeafe; color: #2563eb; }
.rarity-very-rare { background: #ede9fe; color: #7c3aed; }
.rarity-legendary { background: #ffedd5; color: #ea580c; }
.rarity-artifact { background: #fee2e2; color: #dc2626; }

/* Form overrides */
.form-textarea.form-textarea {
  font-family: monospace;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-md);
}

</style>
