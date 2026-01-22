<template>
  <AppModal
    :visible="visible"
    title="Inventory & Equipment"
    size="lg"
    @close="closeDialog"
  >
    <div class="dialog-body">
        <!-- Tabs -->
        <div class="tabs">
          <button
            class="tab"
            :class="{ active: activeTab === 'inventory' }"
            @click="activeTab = 'inventory'"
          >
            Inventory
          </button>
          <button
            class="tab"
            :class="{ active: activeTab === 'equipment' }"
            @click="activeTab = 'equipment'"
          >
            Equipment
          </button>
          <button
            class="tab"
            :class="{ active: activeTab === 'currency' }"
            @click="activeTab = 'currency'"
          >
            Currency
          </button>
        </div>

        <!-- Inventory Tab -->
        <div v-if="activeTab === 'inventory'" class="tab-content">
          <div class="section-header">
            <h3>Items</h3>
            <button @click="showAddItemModal = true" class="btn-add">+ Add Item</button>
          </div>

          <EmptyState
            v-if="inventory.length === 0"
            variant="generic"
            title="No items in inventory"
            description="Add items using the button above"
          />

          <div v-else class="inventory-list">
            <div
              v-for="item in inventory"
              :key="item.id"
              class="inventory-item"
            >
              <div class="item-info">
                <span class="item-name">{{ item.item_name }}</span>
                <span class="item-quantity">x{{ item.quantity }}</span>
              </div>
              <div class="item-details">
                <span v-if="item.equipped" class="item-equipped">Equipped</span>
                <span v-if="item.notes" class="item-notes">{{ item.notes }}</span>
              </div>
              <div class="item-actions">
                <button @click="removeItem(item.id)" class="btn-remove" title="Remove">-</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Equipment Tab -->
        <div v-if="activeTab === 'equipment'" class="tab-content">
          <div class="section-header">
            <h3>Equipped Items</h3>
          </div>

          <EmptyState
            v-if="inventory.length === 0"
            variant="generic"
            title="No items"
            description="Add items to your inventory first"
          />

          <div v-else class="equipment-list">
            <label
              v-for="item in inventory"
              :key="item.id"
              class="equipment-item"
            >
              <input
                type="checkbox"
                :checked="item.equipped !== 0"
                @change="toggleEquipped(item)"
              />
              <span class="item-name">{{ item.item_name }}</span>
              <span class="item-source">{{ item.item_source }}</span>
            </label>
          </div>

          <!-- Attunement -->
          <div class="attunement-section">
            <h4>Attuned Items ({{ attunedCount }}/3)</h4>
            <EmptyState
              v-if="magicItems.length === 0"
              variant="generic"
              title="No magic items"
              description="No items requiring attunement"
            />
            <div v-else class="attunement-list">
              <label
                v-for="item in magicItems"
                :key="item.id"
                class="attunement-item"
              >
                <input
                  type="checkbox"
                  :checked="isAttuned(item)"
                  :disabled="!isAttuned(item) && attunedCount >= 3"
                  @change="toggleAttunement(item)"
                />
                {{ item.item_name }}
              </label>
            </div>
          </div>
        </div>

        <!-- Currency Tab -->
        <div v-if="activeTab === 'currency'" class="tab-content">
          <div class="currency-grid">
            <div class="currency-item">
              <label>Platinum (pp)</label>
              <input
                type="number"
                v-model.number="currencyPlatinum"
                min="0"
                @change="updateCurrency"
              />
            </div>
            <div class="currency-item">
              <label>Gold (gp)</label>
              <input
                type="number"
                v-model.number="currencyGold"
                min="0"
                @change="updateCurrency"
              />
            </div>
            <div class="currency-item">
              <label>Silver (sp)</label>
              <input
                type="number"
                v-model.number="currencySilver"
                min="0"
                @change="updateCurrency"
              />
            </div>
            <div class="currency-item">
              <label>Copper (cp)</label>
              <input
                type="number"
                v-model.number="currencyCopper"
                min="0"
                @change="updateCurrency"
              />
            </div>
          </div>
          <div class="currency-total">
            Total value: {{ totalGoldValue.toFixed(2) }} gp
          </div>
        </div>
      </div>

    <template #footer>
      <button @click="closeDialog" class="btn btn-primary">Done</button>
    </template>
  </AppModal>

  <!-- Add Item Modal -->
  <AppModal
    :visible="showAddItemModal"
    title="Add Item"
    size="sm"
    :stack-index="1"
    @close="showAddItemModal = false"
  >
    <div class="add-item-body">
          <div class="search-box">
            <input
              type="text"
              v-model="itemSearch"
              placeholder="Search items..."
              @input="searchItems"
            />
          </div>

          <div class="item-results">
            <div
              v-for="item in searchResults"
              :key="`${item.name}-${item.source}`"
              class="search-result-item"
              @click="selectItem(item)"
            >
              <span class="result-name">{{ item.name }}</span>
              <span class="result-source">{{ item.source }}</span>
            </div>
            <div v-if="itemSearch && searchResults.length === 0" class="no-results">
              No items found
            </div>
          </div>

          <div v-if="selectedItem" class="selected-item-details">
            <h4>{{ selectedItem.name }}</h4>
            <div class="quantity-input">
              <label>Quantity:</label>
              <input type="number" v-model.number="addQuantity" min="1" />
            </div>
            <div class="notes-input">
              <label>Notes (optional):</label>
              <input type="text" v-model="addNotes" placeholder="e.g., +1, silvered" />
            </div>
          </div>
    </div>

    <template #footer>
      <button @click="showAddItemModal = false" class="btn btn-secondary">Cancel</button>
      <button
        @click="addItem"
        class="btn btn-primary"
        :disabled="!selectedItem"
      >
        Add to Inventory
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import { useCharacterStore } from '../../../stores/characters'
import type { Character, CharacterInventory } from '../../../types/character'

interface CatalogItem {
  id: string
  name: string
  source: string
  item_type: string
  rarity: string | null
}

const props = defineProps<{
  visible: boolean
  characterId: string
  characterData: Character
}>()

const emit = defineEmits<{
  close: []
  updated: []
}>()

const characterStore = useCharacterStore()

// Tab state
const activeTab = ref<'inventory' | 'equipment' | 'currency'>('inventory')

// Add item modal state
const showAddItemModal = ref(false)
const itemSearch = ref('')
const searchResults = ref<CatalogItem[]>([])
const selectedItem = ref<CatalogItem | null>(null)
const addQuantity = ref(1)
const addNotes = ref('')

// Currency state (uses flat fields from Character: cp, sp, ep, gp, pp)
const currencyPlatinum = ref(0)
const currencyGold = ref(0)
const currencyElectrum = ref(0)
const currencySilver = ref(0)
const currencyCopper = ref(0)

// Inventory from store (fetched separately from character)
const inventory = computed(() => characterStore.currentInventory)

// Computed properties for equipment filtering
const armorItems = computed(() => {
  return inventory.value.filter((item: CharacterInventory) =>
    item.item_name.toLowerCase().includes('armor') ||
    item.item_name.toLowerCase().includes('mail') ||
    item.item_name.toLowerCase().includes('leather') ||
    item.item_name.toLowerCase().includes('hide') ||
    item.item_name.toLowerCase().includes('plate')
  )
})

const shieldItems = computed(() => {
  return inventory.value.filter((item: CharacterInventory) =>
    item.item_name.toLowerCase().includes('shield')
  )
})

const weaponItems = computed(() => {
  return inventory.value.filter((item: CharacterInventory) =>
    item.item_name.toLowerCase().includes('sword') ||
    item.item_name.toLowerCase().includes('axe') ||
    item.item_name.toLowerCase().includes('mace') ||
    item.item_name.toLowerCase().includes('bow') ||
    item.item_name.toLowerCase().includes('crossbow') ||
    item.item_name.toLowerCase().includes('dagger') ||
    item.item_name.toLowerCase().includes('spear') ||
    item.item_name.toLowerCase().includes('staff') ||
    item.item_name.toLowerCase().includes('hammer') ||
    item.item_name.toLowerCase().includes('flail') ||
    item.item_name.toLowerCase().includes('halberd') ||
    item.item_name.toLowerCase().includes('lance') ||
    item.item_name.toLowerCase().includes('pike') ||
    item.item_name.toLowerCase().includes('rapier') ||
    item.item_name.toLowerCase().includes('scimitar') ||
    item.item_name.toLowerCase().includes('trident') ||
    item.item_name.toLowerCase().includes('warhammer') ||
    item.item_name.toLowerCase().includes('whip')
  )
})

const offHandItems = computed(() => {
  return [...shieldItems.value, ...weaponItems.value]
})

const magicItems = computed(() => {
  return inventory.value.filter((item: CharacterInventory) =>
    item.notes && (
      item.notes.includes('+') ||
      item.notes.toLowerCase().includes('of ') ||
      item.notes.toLowerCase().includes('attune')
    )
  )
})

// Equipped items (items with equipped=1)
const equippedItems = computed(() => {
  return inventory.value.filter((item: CharacterInventory) => item.equipped !== 0)
})

const attunedCount = computed(() => inventory.value.filter((item: CharacterInventory) => item.attuned !== 0).length)

const totalGoldValue = computed(() => {
  return (
    currencyPlatinum.value * 10 +
    currencyGold.value +
    currencySilver.value * 0.1 +
    currencyCopper.value * 0.01
  )
})

// Methods
const closeDialog = () => {
  emit('close')
}

const searchItems = async () => {
  if (!itemSearch.value || itemSearch.value.length < 2) {
    searchResults.value = []
    return
  }

  try {
    const results = await invoke<CatalogItem[]>('search_items', {
      name: itemSearch.value
    })

    // Sort results to prioritize basic items and better matches
    const searchLower = itemSearch.value.toLowerCase()
    searchResults.value = results.sort((a, b) => {
      const aName = a.name.toLowerCase()
      const bName = b.name.toLowerCase()

      // Exact match first
      const aExact = aName === searchLower
      const bExact = bName === searchLower
      if (aExact && !bExact) return -1
      if (bExact && !aExact) return 1

      // Starts with search term
      const aStarts = aName.startsWith(searchLower)
      const bStarts = bName.startsWith(searchLower)
      if (aStarts && !bStarts) return -1
      if (bStarts && !aStarts) return 1

      // Basic items (no rarity or "none") before magical variants
      const aBasic = !a.rarity || a.rarity.toLowerCase() === 'none'
      const bBasic = !b.rarity || b.rarity.toLowerCase() === 'none'
      if (aBasic && !bBasic) return -1
      if (bBasic && !aBasic) return 1

      // Alphabetical
      return aName.localeCompare(bName)
    })
  } catch (e) {
    console.error('Failed to search items:', e)
    searchResults.value = []
  }
}

const selectItem = (item: CatalogItem) => {
  selectedItem.value = item
  addQuantity.value = 1
  addNotes.value = ''
}

const addItem = async () => {
  if (!selectedItem.value) return

  try {
    await characterStore.addInventoryItem(props.characterId, {
      item_name: selectedItem.value.name,
      item_source: selectedItem.value.source,
      quantity: addQuantity.value,
      notes: addNotes.value || undefined
    })

    // Reset and close modal
    selectedItem.value = null
    itemSearch.value = ''
    searchResults.value = []
    showAddItemModal.value = false

    emit('updated')
  } catch (e) {
    console.error('Failed to add item:', e)
  }
}

const removeItem = async (inventoryId: string) => {
  try {
    await characterStore.removeInventoryItem(inventoryId)
    emit('updated')
  } catch (e) {
    console.error('Failed to remove item:', e)
  }
}

const toggleEquipped = async (item: CharacterInventory) => {
  try {
    await characterStore.updateInventoryItem(item.id, {
      equipped: item.equipped === 0
    })
    emit('updated')
  } catch (e) {
    console.error('Failed to update equipment:', e)
  }
}

const updateCurrency = async () => {
  try {
    // Update character with new currency values (flat fields: cp, sp, ep, gp, pp)
    await characterStore.updateCharacter(props.characterId, {
      currency: [
        currencyCopper.value,
        currencySilver.value,
        currencyElectrum.value,
        currencyGold.value,
        currencyPlatinum.value
      ]
    })
    emit('updated')
  } catch (e) {
    console.error('Failed to update currency:', e)
  }
}

const isAttuned = (item: CharacterInventory) => {
  return item.attuned !== 0
}

const toggleAttunement = async (item: CharacterInventory) => {
  try {
    await characterStore.updateInventoryItem(item.id, {
      attuned: item.attuned === 0
    })
    emit('updated')
  } catch (e) {
    console.error('Failed to toggle attunement:', e)
  }
}

// Initialize state when dialog opens
watch(() => props.visible, async (visible) => {
  if (visible) {
    // Fetch inventory from store
    await characterStore.fetchInventory(props.characterId)

    // Set currency from character data (flat fields: cp, sp, ep, gp, pp)
    currencyPlatinum.value = props.characterData.pp
    currencyGold.value = props.characterData.gp
    currencyElectrum.value = props.characterData.ep
    currencySilver.value = props.characterData.sp
    currencyCopper.value = props.characterData.cp

    // Reset tab
    activeTab.value = 'inventory'
  }
})
</script>

<style scoped>
/* Domain-specific styles for Inventory Manager */

/* Tabs */
.tabs {
  display: flex;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
  padding-bottom: var(--spacing-sm);
}

.tab {
  padding: var(--spacing-sm) var(--spacing-md);
  background: none;
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-weight: 500;
  border-radius: var(--radius-sm);
}

.tab:hover {
  background: var(--color-surface-variant);
}

.tab.active {
  color: var(--color-primary-500);
  background: var(--color-surface-variant);
}

.tab-content {
  min-height: 300px;
}

/* Section header */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.section-header h3 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.btn-add {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 0.875rem;
}

.btn-add:hover {
  background: var(--color-primary-600);
}

/* Inventory list */
.inventory-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.inventory-item {
  display: flex;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  gap: var(--spacing-md);
}

.item-info {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.item-name {
  font-weight: 500;
  color: var(--color-text);
}

.item-quantity {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.item-details {
  display: flex;
  gap: var(--spacing-sm);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.item-actions {
  display: flex;
  gap: var(--spacing-xs);
}

.btn-remove {
  width: 24px;
  height: 24px;
  padding: 0;
  background: var(--color-error);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: bold;
}

.btn-remove:hover {
  opacity: 0.8;
}

/* Equipment slots */
.equipment-slots {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.equipment-slot {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.equipment-slot label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.equipment-slot select {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
}

/* Attunement */
.attunement-section {
  margin-top: var(--spacing-lg);
  padding-top: var(--spacing-lg);
  border-top: 1px solid var(--color-border);
}

.attunement-section h4 {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-md);
}

.attunement-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.attunement-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  cursor: pointer;
}

.attunement-item input:disabled {
  cursor: not-allowed;
}

/* Currency */
.currency-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--spacing-md);
}

.currency-item {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.currency-item label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.currency-item input {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  text-align: center;
}

.currency-total {
  margin-top: var(--spacing-md);
  padding-top: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  text-align: right;
  font-weight: 500;
  color: var(--color-text-secondary);
}

/* Add Item Modal - domain-specific styles */
.add-item-body {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.search-box {
  margin-bottom: var(--spacing-md);
}

.search-box input {
  width: 100%;
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
}

.item-results {
  max-height: 200px;
  overflow-y: auto;
  margin-bottom: var(--spacing-md);
}

.search-result-item {
  display: flex;
  justify-content: space-between;
  padding: var(--spacing-sm);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.search-result-item:hover {
  background: var(--color-surface-variant);
}

.result-name {
  font-weight: 500;
  color: var(--color-text);
}

.result-source {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.no-results {
  text-align: center;
  padding: var(--spacing-md);
  color: var(--color-text-secondary);
  font-style: italic;
}

.selected-item-details {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.selected-item-details h4 {
  margin-bottom: var(--spacing-md);
  color: var(--color-text);
}

.quantity-input,
.notes-input {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

.quantity-input label,
.notes-input label {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.quantity-input input {
  width: 80px;
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
}

.notes-input input {
  flex: 1;
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
}
</style>
