<template>
  <div class="sheet-content single-column">
    <!-- Currency -->
    <section class="sheet-section">
      <h2>Currency</h2>
      <div class="currency-display">
        <div class="currency-item large">
          <span class="currency-icon pp">PP</span>
          <span class="currency-value">{{ character.pp }}</span>
        </div>
        <div class="currency-item large">
          <span class="currency-icon gp">GP</span>
          <span class="currency-value">{{ character.gp }}</span>
        </div>
        <div class="currency-item">
          <span class="currency-icon ep">EP</span>
          <span class="currency-value">{{ character.ep }}</span>
        </div>
        <div class="currency-item">
          <span class="currency-icon sp">SP</span>
          <span class="currency-value">{{ character.sp }}</span>
        </div>
        <div class="currency-item">
          <span class="currency-icon cp">CP</span>
          <span class="currency-value">{{ character.cp }}</span>
        </div>
      </div>
    </section>

    <!-- Equipped Items -->
    <section class="sheet-section">
      <h2>Equipped Items</h2>
      <div v-if="equippedItems.length === 0" class="empty-state">
        No items equipped
      </div>
      <div v-else class="item-cards">
        <div
          v-for="item in equippedItems"
          :key="item.id"
          class="item-card"
          :class="{ expanded: isItemExpanded(item.item_name, item.item_source) }"
        >
          <div
            class="item-card-header"
            @click="toggleItemDetails(item.item_name, item.item_source)"
          >
            <span class="item-name">{{ item.item_name }}</span>
            <span class="item-meta">
              <span v-if="item.attuned" class="item-attuned">Attuned</span>
              <span class="item-source">{{ item.item_source }}</span>
              <span class="expand-icon">{{ isItemExpanded(item.item_name, item.item_source) ? '−' : '+' }}</span>
            </span>
          </div>
          <div
            v-if="isItemExpanded(item.item_name, item.item_source)"
            class="item-card-details"
          >
            <template v-if="getItemDetail(item.item_name, item.item_source)">
              <div class="item-detail-row" v-if="getItemDetail(item.item_name, item.item_source)?.rarity">
                <span class="detail-label">Rarity:</span>
                <span class="detail-value rarity" :class="getItemDetail(item.item_name, item.item_source)?.rarity?.toLowerCase()">
                  {{ getItemDetail(item.item_name, item.item_source)?.rarity }}
                </span>
              </div>
              <div class="item-detail-row" v-if="getItemProperties(getItemDetail(item.item_name, item.item_source)!).length > 0">
                <span class="detail-label">Properties:</span>
                <span class="detail-value">{{ getItemProperties(getItemDetail(item.item_name, item.item_source)!).join(', ') }}</span>
              </div>
              <div class="item-description" v-if="getItemDescription(getItemDetail(item.item_name, item.item_source)!)">
                {{ getItemDescription(getItemDetail(item.item_name, item.item_source)!) }}
              </div>
            </template>
            <div v-else class="loading-details">Loading details...</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Full Inventory -->
    <section class="sheet-section">
      <div class="section-header-row">
        <h2>Inventory</h2>
        <button @click="$emit('openInventory')" class="btn btn-secondary btn-sm">Manage</button>
      </div>
      <div v-if="loadingInventory" class="loading-inventory">Loading inventory...</div>
      <div v-else-if="inventory.length === 0" class="empty-state">
        No items in inventory
      </div>
      <div v-else class="item-cards">
        <div
          v-for="item in inventory"
          :key="item.id"
          class="item-card"
          :class="{ expanded: isItemExpanded(item.item_name, item.item_source) }"
        >
          <div
            class="item-card-header"
            @click="toggleItemDetails(item.item_name, item.item_source)"
          >
            <span class="item-name">
              {{ item.item_name }}
              <span v-if="item.quantity > 1" class="item-qty">x{{ item.quantity }}</span>
            </span>
            <span class="item-meta">
              <span v-if="item.equipped" class="item-equipped-badge">Equipped</span>
              <span v-if="item.attuned" class="item-attuned">Attuned</span>
              <span class="expand-icon">{{ isItemExpanded(item.item_name, item.item_source) ? '−' : '+' }}</span>
            </span>
          </div>
          <div
            v-if="isItemExpanded(item.item_name, item.item_source)"
            class="item-card-details"
          >
            <template v-if="getItemDetail(item.item_name, item.item_source)">
              <div class="item-detail-row" v-if="getItemDetail(item.item_name, item.item_source)?.rarity">
                <span class="detail-label">Rarity:</span>
                <span class="detail-value rarity" :class="getItemDetail(item.item_name, item.item_source)?.rarity?.toLowerCase()">
                  {{ getItemDetail(item.item_name, item.item_source)?.rarity }}
                </span>
              </div>
              <div class="item-detail-row" v-if="getItemProperties(getItemDetail(item.item_name, item.item_source)!).length > 0">
                <span class="detail-label">Properties:</span>
                <span class="detail-value">{{ getItemProperties(getItemDetail(item.item_name, item.item_source)!).join(', ') }}</span>
              </div>
              <div class="item-description" v-if="getItemDescription(getItemDetail(item.item_name, item.item_source)!)">
                {{ getItemDescription(getItemDetail(item.item_name, item.item_source)!) }}
              </div>
              <div v-if="item.notes" class="item-notes">
                <span class="detail-label">Notes:</span> {{ item.notes }}
              </div>
            </template>
            <div v-else class="loading-details">Loading details...</div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Character, CharacterInventory } from '@/types/character'

// Item detail from catalog
interface ItemDetail {
  name: string
  source: string
  item_type: string | null
  rarity: string | null
  data: Record<string, unknown>
  fluff: string | null
}

const props = defineProps<{
  character: Character
  inventory: CharacterInventory[]
  loadingInventory: boolean
}>()

defineEmits<{
  openInventory: []
}>()

// Computed
const equippedItems = computed(() => props.inventory.filter((i) => i.equipped !== 0))

// Item expansion state
const itemDetails = ref<Record<string, ItemDetail>>({})
const expandedItems = ref<Set<string>>(new Set())

// Item helper functions
const getItemKey = (name: string, source: string) => `${name}|${source}`

const isItemExpanded = (name: string, source: string) => {
  return expandedItems.value.has(getItemKey(name, source))
}

const toggleItemDetails = async (name: string, source: string) => {
  const key = getItemKey(name, source)

  if (expandedItems.value.has(key)) {
    expandedItems.value.delete(key)
    expandedItems.value = new Set(expandedItems.value)
    return
  }

  // Fetch item details if not already loaded
  if (!itemDetails.value[key]) {
    try {
      const result = await invoke<{ success: boolean; data?: Record<string, unknown> }>(
        'get_item_by_name',
        { name, source }
      )
      if (result.success && result.data) {
        const rawItem = result.data as unknown as {
          name: string
          source: string
          item_type: string | null
          rarity: string | null
          data: string | Record<string, unknown>
          fluff: string | null
        }
        const item: ItemDetail = {
          name: rawItem.name,
          source: rawItem.source,
          item_type: rawItem.item_type,
          rarity: rawItem.rarity,
          data: typeof rawItem.data === 'string' ? JSON.parse(rawItem.data) : rawItem.data,
          fluff: rawItem.fluff,
        }
        itemDetails.value[key] = item
      }
    } catch (e) {
      console.error('Failed to load item details:', e)
      return
    }
  }

  expandedItems.value.add(key)
  expandedItems.value = new Set(expandedItems.value)
}

const getItemDetail = (name: string, source: string): ItemDetail | null => {
  return itemDetails.value[getItemKey(name, source)] || null
}

const getItemDescription = (item: ItemDetail): string => {
  if (!item.data) return ''

  const entries = item.data.entries as unknown[]
  if (!entries || !Array.isArray(entries)) return ''

  return entries
    .map((entry) => {
      if (typeof entry === 'string') return entry
      if (typeof entry === 'object' && entry !== null) {
        const e = entry as Record<string, unknown>
        if (e.type === 'entries' && Array.isArray(e.entries)) {
          return (e.entries as unknown[])
            .filter((sub) => typeof sub === 'string')
            .join(' ')
        }
        if (e.type === 'list' && Array.isArray(e.items)) {
          return (e.items as unknown[])
            .filter((sub) => typeof sub === 'string')
            .join(', ')
        }
      }
      return ''
    })
    .filter(Boolean)
    .join(' ')
}

const getItemProperties = (item: ItemDetail): string[] => {
  if (!item.data) return []

  const props: string[] = []
  const data = item.data

  // Weapon properties
  if (data.property && Array.isArray(data.property)) {
    const propMap: Record<string, string> = {
      'F': 'Finesse',
      'H': 'Heavy',
      'L': 'Light',
      'R': 'Reach',
      'T': 'Thrown',
      '2H': 'Two-Handed',
      'V': 'Versatile',
      'A': 'Ammunition',
      'LD': 'Loading',
      'S': 'Special',
    }
    for (const p of data.property as string[]) {
      if (propMap[p]) props.push(propMap[p])
    }
  }

  // Armor properties
  if (data.stealth) props.push('Stealth Disadvantage')
  if (data.strength) props.push(`Str ${data.strength}+ required`)

  // Magic item properties
  if (data.reqAttune) {
    if (data.reqAttune === true) props.push('Requires Attunement')
    else if (typeof data.reqAttune === 'string') props.push(`Attunement: ${data.reqAttune}`)
  }

  return props
}
</script>

<style scoped>
.sheet-content.single-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  max-width: 700px;
}

.sheet-section {
  background: var(--color-surface);
  border: 1px solid #ccc;
  border-radius: var(--radius-lg);
  padding: var(--spacing-md);
}

.sheet-section h2 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--spacing-md) 0;
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.section-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
  padding-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--color-border);
}

.section-header-row h2 {
  margin: 0;
  padding: 0;
  border: none;
}

.empty-state {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-style: italic;
}

.loading-inventory {
  text-align: center;
  padding: var(--spacing-md);
  color: var(--color-text-secondary);
}

/* Currency */
.currency-display {
  display: flex;
  gap: var(--spacing-lg);
  justify-content: center;
}

.currency-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

.currency-item.large .currency-value {
  font-size: 1.5rem;
}

.currency-icon {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  background: var(--color-surface-variant);
}

.currency-icon.pp {
  background: #e0e7ff;
  color: #4338ca;
}

.currency-icon.gp {
  background: #fef3c7;
  color: #d97706;
}

.currency-icon.ep,
.currency-icon.sp {
  background: #f3f4f6;
  color: #6b7280;
}

.currency-icon.cp {
  background: #fef2f2;
  color: #dc2626;
}

.currency-value {
  font-size: 1.1rem;
  font-weight: bold;
}

/* Item Cards */
.item-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.item-card {
  background: var(--color-surface-variant);
  border: 1px solid #ccc;
  border-radius: var(--radius-md);
  overflow: hidden;
  transition: all 0.2s ease;
}

.item-card.expanded {
  border-color: var(--color-primary-300);
}

.item-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background 0.15s ease;
}

.item-card-header:hover {
  background: var(--color-surface-hover);
}

.item-card-header .item-name {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.item-card-header .item-qty {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-weight: normal;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 0.85rem;
}

.item-source {
  color: var(--color-text-secondary);
}

.item-equipped-badge {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
}

.item-attuned {
  background: var(--color-warning-100);
  color: var(--color-warning-700);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
}

.expand-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  font-weight: bold;
  color: var(--color-text-secondary);
}

.item-card-details {
  padding: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
  font-size: 0.9rem;
}

.item-detail-row {
  display: flex;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xs);
}

.detail-label {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.detail-value.rarity {
  text-transform: capitalize;
}

.detail-value.rarity.common {
  color: var(--color-text-secondary);
}

.detail-value.rarity.uncommon {
  color: #16a34a;
}

.detail-value.rarity.rare {
  color: #2563eb;
}

.detail-value.rarity.very.rare {
  color: #7c3aed;
}

.detail-value.rarity.legendary {
  color: #ea580c;
}

.detail-value.rarity.artifact {
  color: #dc2626;
}

.item-description {
  margin-top: var(--spacing-sm);
  color: var(--color-text);
  line-height: 1.5;
}

.item-notes {
  margin-top: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px dashed var(--color-border);
  font-style: italic;
  color: var(--color-text-secondary);
}

.loading-details {
  color: var(--color-text-secondary);
  font-style: italic;
}

/* Button */
.btn {
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-secondary {
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.btn-secondary:hover {
  background: var(--color-surface-hover);
}

.btn-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
}
</style>
