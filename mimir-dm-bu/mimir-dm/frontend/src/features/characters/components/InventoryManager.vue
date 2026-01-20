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
            v-if="characterData.inventory.length === 0"
            variant="generic"
            title="No items in inventory"
            description="Add items using the button above"
          />

          <div v-else class="inventory-list">
            <div
              v-for="item in characterData.inventory"
              :key="item.name"
              class="inventory-item"
            >
              <div class="item-info">
                <span class="item-name">{{ item.name }}</span>
                <span class="item-quantity">x{{ item.quantity }}</span>
              </div>
              <div class="item-details">
                <span v-if="item.weight" class="item-weight">{{ item.weight }} lb</span>
                <span v-if="item.notes" class="item-notes">{{ item.notes }}</span>
              </div>
              <div class="item-actions">
                <button @click="removeItem(item.name)" class="btn-remove" title="Remove one">-</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Equipment Tab -->
        <div v-if="activeTab === 'equipment'" class="tab-content">
          <div class="equipment-slots">
            <div class="equipment-slot">
              <label>Armor</label>
              <select v-model="equippedArmor" @change="updateEquipment">
                <option :value="null">None</option>
                <option
                  v-for="item in armorItems"
                  :key="item.name"
                  :value="item.name"
                >
                  {{ item.name }}
                </option>
              </select>
            </div>

            <div class="equipment-slot">
              <label>Shield</label>
              <select v-model="equippedShield" @change="updateEquipment">
                <option :value="null">None</option>
                <option
                  v-for="item in shieldItems"
                  :key="item.name"
                  :value="item.name"
                >
                  {{ item.name }}
                </option>
              </select>
            </div>

            <div class="equipment-slot">
              <label>Main Hand</label>
              <select v-model="equippedMainHand" @change="updateEquipment">
                <option :value="null">None</option>
                <option
                  v-for="item in weaponItems"
                  :key="item.name"
                  :value="item.name"
                >
                  {{ item.name }}
                </option>
              </select>
            </div>

            <div class="equipment-slot">
              <label>Off Hand</label>
              <select v-model="equippedOffHand" @change="updateEquipment">
                <option :value="null">None</option>
                <option
                  v-for="item in offHandItems"
                  :key="item.name"
                  :value="item.name"
                >
                  {{ item.name }}
                </option>
              </select>
            </div>
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
                :key="item.name"
                class="attunement-item"
              >
                <input
                  type="checkbox"
                  :checked="isAttuned(item.name)"
                  :disabled="!isAttuned(item.name) && attunedCount >= 3"
                  @change="toggleAttunement(item.name)"
                />
                {{ item.name }}
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
import type { CharacterData, InventoryItem } from '../../../types/character'

interface CatalogItem {
  id: number
  name: string
  source: string
  item_type: string
  rarity: string | null
}

const props = defineProps<{
  visible: boolean
  characterId: number
  characterData: CharacterData
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

// Equipment state
const equippedArmor = ref<string | null>(null)
const equippedShield = ref<string | null>(null)
const equippedMainHand = ref<string | null>(null)
const equippedOffHand = ref<string | null>(null)

// Currency state
const currencyPlatinum = ref(0)
const currencyGold = ref(0)
const currencyElectrum = ref(0)
const currencySilver = ref(0)
const currencyCopper = ref(0)

// Attunement state
const attunedItems = ref<string[]>([])

// Computed properties
const armorItems = computed(() => {
  // Filter inventory for armor-type items
  return props.characterData.inventory.filter(item =>
    item.name.toLowerCase().includes('armor') ||
    item.name.toLowerCase().includes('mail') ||
    item.name.toLowerCase().includes('leather') ||
    item.name.toLowerCase().includes('hide') ||
    item.name.toLowerCase().includes('plate')
  )
})

const shieldItems = computed(() => {
  return props.characterData.inventory.filter(item =>
    item.name.toLowerCase().includes('shield')
  )
})

const weaponItems = computed(() => {
  return props.characterData.inventory.filter(item =>
    item.name.toLowerCase().includes('sword') ||
    item.name.toLowerCase().includes('axe') ||
    item.name.toLowerCase().includes('mace') ||
    item.name.toLowerCase().includes('bow') ||
    item.name.toLowerCase().includes('crossbow') ||
    item.name.toLowerCase().includes('dagger') ||
    item.name.toLowerCase().includes('spear') ||
    item.name.toLowerCase().includes('staff') ||
    item.name.toLowerCase().includes('hammer') ||
    item.name.toLowerCase().includes('flail') ||
    item.name.toLowerCase().includes('halberd') ||
    item.name.toLowerCase().includes('lance') ||
    item.name.toLowerCase().includes('pike') ||
    item.name.toLowerCase().includes('rapier') ||
    item.name.toLowerCase().includes('scimitar') ||
    item.name.toLowerCase().includes('trident') ||
    item.name.toLowerCase().includes('warhammer') ||
    item.name.toLowerCase().includes('whip')
  )
})

const offHandItems = computed(() => {
  // Can hold shields or light weapons in off hand
  return [...shieldItems.value, ...weaponItems.value]
})

const magicItems = computed(() => {
  // Items with notes suggesting they're magic (e.g., "+1", "of", etc.)
  return props.characterData.inventory.filter(item =>
    item.notes && (
      item.notes.includes('+') ||
      item.notes.toLowerCase().includes('of ') ||
      item.notes.toLowerCase().includes('attune')
    )
  )
})

const attunedCount = computed(() => attunedItems.value.length)

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
    await characterStore.addItem(
      props.characterId,
      selectedItem.value.name,
      selectedItem.value.source,
      addQuantity.value,
      addNotes.value || undefined
    )

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

const removeItem = async (itemName: string) => {
  try {
    await characterStore.removeItem(props.characterId, itemName, 1)
    emit('updated')
  } catch (e) {
    console.error('Failed to remove item:', e)
  }
}

const updateEquipment = async () => {
  try {
    await characterStore.updateEquipped(
      props.characterId,
      equippedArmor.value,
      equippedShield.value,
      equippedMainHand.value,
      equippedOffHand.value
    )
    emit('updated')
  } catch (e) {
    console.error('Failed to update equipment:', e)
  }
}

const updateCurrency = async () => {
  try {
    // Calculate deltas from current values
    const current = props.characterData.currency
    await characterStore.updateCurrency(props.characterId, {
      platinum: currencyPlatinum.value - current.platinum,
      gold: currencyGold.value - current.gold,
      electrum: currencyElectrum.value - current.electrum,
      silver: currencySilver.value - current.silver,
      copper: currencyCopper.value - current.copper
    })
    emit('updated')
  } catch (e) {
    console.error('Failed to update currency:', e)
  }
}

const isAttuned = (itemName: string) => {
  return attunedItems.value.includes(itemName)
}

const toggleAttunement = (itemName: string) => {
  if (isAttuned(itemName)) {
    attunedItems.value = attunedItems.value.filter(n => n !== itemName)
  } else if (attunedCount.value < 3) {
    attunedItems.value.push(itemName)
  }
  // Note: Attunement is currently local state only
  // Could be persisted to character notes or a dedicated field
}

// Initialize state when dialog opens
watch(() => props.visible, (visible) => {
  if (visible) {
    // Set equipment from character data
    equippedArmor.value = props.characterData.equipped.armor
    equippedShield.value = props.characterData.equipped.shield
    equippedMainHand.value = props.characterData.equipped.main_hand
    equippedOffHand.value = props.characterData.equipped.off_hand

    // Set currency from character data
    currencyPlatinum.value = props.characterData.currency.platinum
    currencyGold.value = props.characterData.currency.gold
    currencyElectrum.value = props.characterData.currency.electrum
    currencySilver.value = props.characterData.currency.silver
    currencyCopper.value = props.characterData.currency.copper

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
