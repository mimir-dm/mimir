<template>
  <div class="homebrew-tab">
    <!-- Sub-tab bar -->
    <div class="sub-tabs">
      <button
        class="sub-tab"
        :class="{ active: activeSubTab === 'items' }"
        @click="activeSubTab = 'items'"
      >Items</button>
      <button
        class="sub-tab"
        :class="{ active: activeSubTab === 'monsters' }"
        @click="activeSubTab = 'monsters'"
      >Monsters</button>
      <button
        class="sub-tab"
        :class="{ active: activeSubTab === 'spells' }"
        @click="activeSubTab = 'spells'"
      >Spells</button>
    </div>

    <!-- Monsters sub-tab -->
    <HomebrewMonstersSubTab
      v-if="activeSubTab === 'monsters'"
      :campaign="campaign"
    />

    <!-- Spells sub-tab -->
    <HomebrewSpellsSubTab
      v-if="activeSubTab === 'spells'"
      :campaign="campaign"
    />

    <!-- Items sub-tab (original content) -->
    <template v-if="activeSubTab === 'items'">
    <!-- Header -->
    <div class="tab-header">
      <h2>Homebrew Items</h2>
      <div class="header-actions">
        <button @click="openCloneFromCatalog" class="btn btn-secondary btn-sm">
          Clone from Catalog
        </button>
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
      <div class="modal-content modal-lg">
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

          <!-- Weapon fields -->
          <div v-if="form.item_type === 'weapon'" class="form-section">
            <h4 class="form-section-title">Weapon Stats</h4>
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">Weapon Category</label>
                <select v-model="form.weaponCategory" class="form-select">
                  <option value="">— None —</option>
                  <option value="simple">Simple</option>
                  <option value="martial">Martial</option>
                </select>
              </div>
              <div class="form-group">
                <label class="form-label">Bonus</label>
                <select v-model="form.bonusWeapon" class="form-select">
                  <option value="">— None —</option>
                  <option value="+1">+1</option>
                  <option value="+2">+2</option>
                  <option value="+3">+3</option>
                </select>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">Damage</label>
                <input v-model="form.dmg1" class="form-input" type="text" placeholder="e.g. 1d8" />
              </div>
              <div class="form-group">
                <label class="form-label">Damage Type</label>
                <select v-model="form.dmgType" class="form-select">
                  <option value="">— None —</option>
                  <option value="S">Slashing</option>
                  <option value="P">Piercing</option>
                  <option value="B">Bludgeoning</option>
                  <option value="F">Fire</option>
                  <option value="C">Cold</option>
                  <option value="L">Lightning</option>
                  <option value="T">Thunder</option>
                  <option value="I">Poison</option>
                  <option value="A">Acid</option>
                  <option value="N">Necrotic</option>
                  <option value="R">Radiant</option>
                  <option value="Y">Psychic</option>
                  <option value="O">Force</option>
                </select>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">Versatile Damage</label>
                <input v-model="form.dmg2" class="form-input" type="text" placeholder="e.g. 1d10" />
              </div>
              <div class="form-group">
                <label class="form-label">Range</label>
                <input v-model="form.range" class="form-input" type="text" placeholder="e.g. 80/320" />
              </div>
            </div>
            <div class="form-group">
              <label class="form-label">Properties</label>
              <div class="checkbox-row">
                <label v-for="prop in weaponProperties" :key="prop.code" class="checkbox-label">
                  <input type="checkbox" :value="prop.code" v-model="form.properties" />
                  {{ prop.label }}
                </label>
              </div>
            </div>
          </div>

          <!-- Armor fields -->
          <div v-if="form.item_type === 'armor'" class="form-section">
            <h4 class="form-section-title">Armor Stats</h4>
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">AC</label>
                <input v-model.number="form.ac" class="form-input" type="number" placeholder="e.g. 16" />
              </div>
              <div class="form-group">
                <label class="form-label">Bonus AC</label>
                <select v-model="form.bonusAc" class="form-select">
                  <option value="">— None —</option>
                  <option value="+1">+1</option>
                  <option value="+2">+2</option>
                  <option value="+3">+3</option>
                </select>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label class="form-label">Strength Required</label>
                <input v-model.number="form.strength" class="form-input" type="number" placeholder="e.g. 13" />
              </div>
              <div class="form-group form-group-checkbox">
                <label class="checkbox-label">
                  <input type="checkbox" v-model="form.stealth" />
                  Stealth Disadvantage
                </label>
              </div>
            </div>
          </div>

          <!-- Common fields -->
          <div class="form-row">
            <div class="form-group">
              <label class="form-label">Weight (lb.)</label>
              <input v-model.number="form.weight" class="form-input" type="number" step="0.01" placeholder="e.g. 3" />
            </div>
            <div class="form-group">
              <label class="form-label">Value (gp)</label>
              <input v-model.number="form.value" class="form-input" type="number" step="0.01" placeholder="e.g. 50" />
            </div>
          </div>

          <div class="form-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="form.reqAttune" />
              Requires Attunement
            </label>
            <input
              v-if="form.reqAttune"
              v-model="form.reqAttuneText"
              class="form-input form-input-inline"
              type="text"
              placeholder="by a cleric (optional restriction)"
            />
          </div>

          <div class="form-group">
            <label class="form-label">Description</label>
            <textarea
              v-model="form.description"
              class="form-textarea"
              rows="5"
              placeholder="Describe what this item does..."
            ></textarea>
          </div>

          <div v-if="formError" class="form-help is-invalid">{{ formError }}</div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" @click="closeForm">Cancel</button>
            <button type="submit" class="btn btn-primary" :disabled="saving">
              {{ saving ? 'Saving...' : (editingItem ? 'Update' : 'Create') }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Clone from Catalog Modal -->
    <div v-if="showCloneModal" class="modal-overlay" @click.self="showCloneModal = false">
      <div class="modal-content">
        <h3>Clone from Catalog</h3>
        <p class="clone-hint">Search the item catalog, then edit the cloned item before saving.</p>
        <div class="form-group">
          <input
            v-model="cloneSearch"
            class="form-input"
            type="text"
            placeholder="Search items by name..."
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
              <span v-if="result.rarity" class="card-rarity" :class="rarityClass(result.rarity)">{{ result.rarity }}</span>
            </div>
            <div class="card-meta">
              <span v-if="result.type" class="card-type">{{ result.type }}</span>
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
        <h3>Delete Item</h3>
        <div v-if="deleteWarningCharacters.length > 0" class="delete-warning">
          <p>This item is currently in the inventory of:</p>
          <ul>
            <li v-for="name in deleteWarningCharacters" :key="name"><strong>{{ name }}</strong></li>
          </ul>
          <p>Deleting this homebrew item will <strong>not</strong> remove it from their inventories, but the item data will no longer be available.</p>
        </div>
        <p>Are you sure you want to delete <strong>{{ selectedItem?.name }}</strong>? This cannot be undone.</p>
        <div class="form-actions">
          <button class="btn btn-secondary" @click="cancelDelete">Cancel</button>
          <button class="btn btn-danger" @click="deleteItem" :disabled="saving">
            {{ saving ? 'Deleting...' : 'Delete' }}
          </button>
        </div>
      </div>
    </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { HomebrewService, type HomebrewItem } from '@/services/HomebrewService'
import ItemDetailBlock from '@/features/characters/components/sheet/ItemDetailBlock.vue'
import HomebrewMonstersSubTab from './HomebrewMonstersSubTab.vue'
import HomebrewSpellsSubTab from './HomebrewSpellsSubTab.vue'
import { dataEvents } from '@/utils/dataEvents'
import type { Campaign } from '@/types'
import type { ApiResponse } from '@/types/api'
import type { CharacterInventory } from '@/types/character'

const props = defineProps<{
  campaign?: Campaign
  documents?: any[]
}>()

const activeSubTab = ref<'items' | 'monsters' | 'spells'>('items')

const loading = ref(false)
const saving = ref(false)
const items = ref<HomebrewItem[]>([])
const selectedItem = ref<HomebrewItem | null>(null)
const showCreateForm = ref(false)
const editingItem = ref<HomebrewItem | null>(null)
const showDeleteConfirm = ref(false)
const formError = ref('')

const weaponProperties = [
  { code: 'F', label: 'Finesse' },
  { code: 'H', label: 'Heavy' },
  { code: 'L', label: 'Light' },
  { code: 'R', label: 'Reach' },
  { code: 'T', label: 'Thrown' },
  { code: '2H', label: 'Two-Handed' },
  { code: 'V', label: 'Versatile' },
  { code: 'A', label: 'Ammunition' },
  { code: 'LD', label: 'Loading' },
  { code: 'S', label: 'Special' },
]

interface FormState {
  name: string
  item_type: string
  rarity: string
  description: string
  weight: number | null
  value: number | null
  reqAttune: boolean
  reqAttuneText: string
  // Weapon
  weaponCategory: string
  dmg1: string
  dmgType: string
  dmg2: string
  range: string
  bonusWeapon: string
  properties: string[]
  // Armor
  ac: number | null
  bonusAc: string
  strength: number | null
  stealth: boolean
  // Clone tracking
  cloned_from_name: string | undefined
  cloned_from_source: string | undefined
}

function emptyForm(): FormState {
  return {
    name: '', item_type: '', rarity: '', description: '',
    weight: null, value: null, reqAttune: false, reqAttuneText: '',
    weaponCategory: '', dmg1: '', dmgType: '', dmg2: '', range: '', bonusWeapon: '', properties: [],
    ac: null, bonusAc: '', strength: null, stealth: false,
    cloned_from_name: undefined, cloned_from_source: undefined,
  }
}

const form = ref<FormState>(emptyForm())

// Clone from catalog state
const showCloneModal = ref(false)
const cloneSearch = ref('')
const cloneSearching = ref(false)
const cloneResults = ref<Array<{ name: string; source: string; type?: string; rarity?: string; data: Record<string, unknown> }>>([])
let cloneSearchTimer: ReturnType<typeof setTimeout> | null = null

// Delete warning state
const deleteWarningCharacters = ref<string[]>([])

/** Build the JSON data blob from structured form fields. */
function formToDataJson(): string {
  const d: Record<string, unknown> = {}

  if (form.value.description) {
    d.entries = [form.value.description]
  }
  if (form.value.weight) d.weight = form.value.weight
  if (form.value.value) d.value = Math.round(form.value.value * 100) // store as cp

  if (form.value.reqAttune) {
    d.reqAttune = form.value.reqAttuneText || true
  }

  if (form.value.item_type === 'weapon') {
    d.weapon = true
    if (form.value.weaponCategory) d.weaponCategory = form.value.weaponCategory
    if (form.value.dmg1) d.dmg1 = form.value.dmg1
    if (form.value.dmgType) d.dmgType = form.value.dmgType
    if (form.value.dmg2) d.dmg2 = form.value.dmg2
    if (form.value.range) d.range = form.value.range
    if (form.value.bonusWeapon) d.bonusWeapon = form.value.bonusWeapon
    if (form.value.properties.length > 0) d.property = form.value.properties
  }

  if (form.value.item_type === 'armor') {
    d.armor = true
    if (form.value.ac) d.ac = form.value.ac
    if (form.value.bonusAc) d.bonusAc = form.value.bonusAc
    if (form.value.strength) d.strength = form.value.strength
    if (form.value.stealth) d.stealth = true
  }

  return JSON.stringify(d)
}

/** Populate structured form fields from a parsed data JSON object. */
function dataJsonToForm(data: Record<string, unknown>) {
  // Description from entries
  if (Array.isArray(data.entries) && data.entries.length > 0) {
    form.value.description = data.entries
      .map((e: unknown) => {
        if (typeof e === 'string') return e
        if (typeof e === 'object' && e !== null) {
          const obj = e as Record<string, unknown>
          if (obj.type === 'entries' && Array.isArray(obj.entries)) {
            return (obj.entries as unknown[]).filter(s => typeof s === 'string').join('\n')
          }
          if (obj.type === 'list' && Array.isArray(obj.items)) {
            return (obj.items as unknown[]).filter(s => typeof s === 'string').join(', ')
          }
        }
        return ''
      })
      .filter(Boolean)
      .join('\n\n')
  }

  if (data.weight) form.value.weight = data.weight as number
  if (data.value) form.value.value = (data.value as number) / 100 // cp to gp

  if (data.reqAttune) {
    form.value.reqAttune = true
    if (typeof data.reqAttune === 'string') form.value.reqAttuneText = data.reqAttune
  }

  // Weapon
  if (data.weaponCategory) form.value.weaponCategory = data.weaponCategory as string
  if (data.dmg1) form.value.dmg1 = data.dmg1 as string
  if (data.dmgType) form.value.dmgType = data.dmgType as string
  if (data.dmg2) form.value.dmg2 = data.dmg2 as string
  if (data.range) form.value.range = data.range as string
  if (data.bonusWeapon) form.value.bonusWeapon = data.bonusWeapon as string
  if (Array.isArray(data.property)) {
    form.value.properties = (data.property as string[]).map(p => p.split('|')[0])
  }

  // Armor
  if (data.ac) form.value.ac = data.ac as number
  if (data.bonusAc) form.value.bonusAc = data.bonusAc as string
  if (data.strength) form.value.strength = data.strength as number
  if (data.stealth) form.value.stealth = true
}

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
  form.value = emptyForm()
  form.value.name = selectedItem.value.name
  form.value.item_type = selectedItem.value.item_type || ''
  form.value.rarity = selectedItem.value.rarity || ''
  try {
    dataJsonToForm(JSON.parse(selectedItem.value.data))
  } catch { /* ignore */ }
}

function closeForm() {
  showCreateForm.value = false
  editingItem.value = null
  formError.value = ''
  form.value = emptyForm()
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
    const response = await invoke<ApiResponse<Array<Record<string, unknown>>>>('search_items', {
      filter: { name_contains: cloneSearch.value },
      limit: 20,
      offset: 0,
    })
    if (response.success && response.data) {
      cloneResults.value = response.data.map((item: Record<string, unknown>) => ({
        name: (item.name as string) || '',
        source: (item.source as string) || '',
        type: (item.type as string) || undefined,
        rarity: (item.rarity as string) || undefined,
        data: item,
      }))
    }
  } catch (e) {
    console.error('Catalog search failed:', e)
    cloneResults.value = []
  } finally {
    cloneSearching.value = false
  }
}

function selectCloneResult(result: { name: string; source: string; type?: string; rarity?: string; data: Record<string, unknown> }) {
  showCloneModal.value = false
  form.value = emptyForm()
  form.value.name = result.name
  form.value.item_type = result.type || ''
  form.value.rarity = result.rarity || ''
  form.value.cloned_from_name = result.name
  form.value.cloned_from_source = result.source
  dataJsonToForm(result.data)
  showCreateForm.value = true
}

// Delete with inventory warning
function cancelDelete() {
  showDeleteConfirm.value = false
  deleteWarningCharacters.value = []
}

async function saveItem() {
  formError.value = ''
  saving.value = true

  const dataJson = formToDataJson()

  try {
    if (editingItem.value) {
      await HomebrewService.update(editingItem.value.id, {
        name: form.value.name,
        item_type: form.value.item_type || null,
        rarity: form.value.rarity || null,
        data: dataJson,
      })
    } else {
      await HomebrewService.create({
        campaign_id: props.campaign!.id,
        name: form.value.name,
        item_type: form.value.item_type || undefined,
        rarity: form.value.rarity || undefined,
        data: dataJson,
        cloned_from_name: form.value.cloned_from_name || undefined,
        cloned_from_source: form.value.cloned_from_source || undefined,
      })
    }
    closeForm()
    await loadItems()
  } catch (e: any) {
    console.error('Failed to save homebrew item:', e)
    formError.value = e.message || 'Failed to save'
  } finally {
    saving.value = false
  }
}

async function confirmDelete() {
  if (!selectedItem.value || !props.campaign?.id) return
  deleteWarningCharacters.value = []

  try {
    // Fetch all characters for this campaign
    const charResponse = await invoke<ApiResponse<Array<{ id: string; name: string }>>>('list_characters', {
      campaignId: props.campaign.id,
    })
    if (charResponse.success && charResponse.data) {
      const affected: string[] = []
      for (const char of charResponse.data) {
        const invResponse = await invoke<ApiResponse<CharacterInventory[]>>('get_character_inventory', {
          characterId: char.id,
        })
        if (invResponse.success && invResponse.data) {
          const hasItem = invResponse.data.some(
            (inv) => inv.item_name === selectedItem.value!.name && inv.item_source === 'HB'
          )
          if (hasItem) affected.push(char.name)
        }
      }
      deleteWarningCharacters.value = affected
    }
  } catch (e) {
    console.error('Failed to check inventory usage:', e)
  }

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

.sub-tabs {
  display: flex;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
  padding-bottom: var(--spacing-xs);
}

.sub-tab {
  padding: var(--spacing-xs) var(--spacing-md);
  border: none;
  background: none;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  border-bottom: 2px solid transparent;
  transition: all 0.15s ease;
}

.sub-tab:hover {
  color: var(--color-text-primary);
}

.sub-tab.active {
  color: var(--color-primary-600);
  border-bottom-color: var(--color-primary-600);
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

/* Form sections */
.form-section {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  margin: var(--spacing-md) 0;
}

.form-section-title {
  margin: 0 0 var(--spacing-sm);
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.checkbox-row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs) var(--spacing-md);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.85rem;
  cursor: pointer;
  user-select: none;
}

.form-group-checkbox {
  display: flex;
  align-items: center;
}

.form-input-inline {
  margin-top: var(--spacing-xs);
}

.modal-lg {
  max-width: 640px;
  max-height: 85vh;
  overflow-y: auto;
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

.card-source {
  font-size: 0.75rem;
  opacity: 0.7;
}

/* Delete warning */
.delete-warning {
  background: var(--color-warning-50, #fffbeb);
  border: 1px solid var(--color-warning-200, #fde68a);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  margin-bottom: var(--spacing-md);
  font-size: 0.9rem;
}

.delete-warning ul {
  margin: var(--spacing-xs) 0;
  padding-left: var(--spacing-lg);
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
