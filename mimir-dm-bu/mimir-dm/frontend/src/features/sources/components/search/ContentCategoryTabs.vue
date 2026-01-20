<template>
  <div class="content-category-tabs">
    <div class="tabs-header">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab-button', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>
    
    <div class="tabs-content">
      <div v-if="activeTab === 'rules'" class="category-grid">
        <button
          v-for="category in rulesCategories"
          :key="category.value"
          :class="['category-button', { selected: modelValue === category.value }]"
          @click="selectCategory(category.value)"
        >
          {{ category.label }}
        </button>
      </div>
      
      <div v-else-if="activeTab === 'player'" class="category-grid">
        <button
          v-for="category in playerCategories"
          :key="category.value"
          :class="['category-button', { selected: modelValue === category.value }]"
          @click="selectCategory(category.value)"
        >
          {{ category.label }}
        </button>
      </div>
      
      <div v-else-if="activeTab === 'reference'" class="category-grid">
        <button
          v-for="category in referenceCategories"
          :key="category.value"
          :class="['category-button', { selected: modelValue === category.value }]"
          @click="selectCategory(category.value)"
        >
          {{ category.label }}
        </button>
      </div>
      
      <div v-else-if="activeTab === 'dm-tools'" class="category-grid">
        <button
          v-for="category in dmToolsCategories"
          :key="category.value"
          :class="['category-button', { selected: modelValue === category.value }]"
          @click="selectCategory(category.value)"
        >
          {{ category.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'

interface Props {
  modelValue: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

interface Tab {
  id: string
  label: string
}

interface Category {
  value: string
  label: string
}

const tabs: Tab[] = [
  { id: 'rules', label: 'Rules' },
  { id: 'player', label: 'Player' },
  { id: 'reference', label: 'Reference' },
  { id: 'dm-tools', label: 'DM Tools' }
]

const activeTab = ref('reference')

const rulesCategories: Category[] = [
  { value: 'Tables', label: 'Tables' },
  { value: 'Variant Rules', label: 'Variant Rules' }
]

const playerCategories: Category[] = [
  { value: 'Classes', label: 'Classes' },
  { value: 'Backgrounds', label: 'Backgrounds' },
  { value: 'Feats', label: 'Feats' },
  { value: 'Races', label: 'Races' },
  { value: 'Other Options & Features', label: 'Other Options' }
]

const referenceCategories: Category[] = [
  { value: 'Actions', label: 'Actions' },
  { value: 'Monsters', label: 'Bestiary' },
  { value: 'Conditions', label: 'Conditions & Diseases' },
  { value: 'Deities', label: 'Deities' },
  { value: 'Equipment', label: 'Equipment' },
  { value: 'Magic Items', label: 'Magic Items' },
  { value: 'Languages', label: 'Languages' },
  { value: 'Rewards', label: 'Rewards' },
  { value: 'Psionics', label: 'Psionics' },
  { value: 'Spells', label: 'Spells' },
  { value: 'Vehicles', label: 'Vehicles' }
]

const dmToolsCategories: Category[] = [
  { value: 'Cults & Boons', label: 'Cults & Boons' },
  { value: 'Objects', label: 'Objects' },
  { value: 'Traps & Hazards', label: 'Traps & Hazards' }
]

// Map categories to their tabs
const categoryToTab: Record<string, string> = {
  // Rules
  'Tables': 'rules',
  'Variant Rules': 'rules',
  // Player
  'Classes': 'player',
  'Backgrounds': 'player',
  'Feats': 'player',
  'Races': 'player',
  'Other Options & Features': 'player',
  'Options': 'player', // Backward compatibility
  // Reference
  'Actions': 'reference',
  'Monsters': 'reference',
  'Conditions': 'reference',
  'Deities': 'reference',
  'Equipment': 'reference',
  'Magic Items': 'reference',
  'Languages': 'reference',
  'Rewards': 'reference',
  'Psionics': 'reference',
  'Spells': 'reference',
  'Vehicles': 'reference',
  // DM Tools
  'Cults & Boons': 'dm-tools',
  'Objects': 'dm-tools',
  'Traps & Hazards': 'dm-tools'
}

function selectCategory(value: string) {
  emit('update:modelValue', value)
}

// Set active tab based on current selection
watch(() => props.modelValue, (newValue) => {
  const tab = categoryToTab[newValue]
  if (tab) {
    activeTab.value = tab
  }
}, { immediate: true })

// Remember last active tab
onMounted(() => {
  const savedTab = localStorage.getItem('contentCategoryActiveTab')
  if (savedTab && tabs.some(t => t.id === savedTab)) {
    activeTab.value = savedTab
  }
})

watch(activeTab, (newTab) => {
  localStorage.setItem('contentCategoryActiveTab', newTab)
})
</script>

<style scoped>
.content-category-tabs {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md, 12px);
}

.tabs-header {
  display: flex;
  gap: var(--spacing-xs, 4px);
  border-bottom: 2px solid var(--color-border, #333);
  padding-bottom: 0;
}

.tab-button {
  padding: var(--spacing-sm, 8px) var(--spacing-lg, 16px);
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  color: var(--color-text-secondary, #999);
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: -2px;
}

.tab-button:hover {
  color: var(--color-text, #e0e0e0);
  background: var(--color-surface-hover, #242424);
}

.tab-button.active {
  color: var(--color-primary, #4a9eff);
  border-bottom-color: var(--color-primary, #4a9eff);
  background: var(--color-surface, #1a1a1a);
}

.tabs-content {
  padding: var(--spacing-md, 12px) 0;
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--spacing-sm, 8px);
}

.category-button {
  padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text, #e0e0e0);
  font-size: 0.9rem;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: center;
}

.category-button:hover {
  background: var(--color-surface-hover, #242424);
  border-color: var(--color-primary, #4a9eff);
}

.category-button.selected {
  background: var(--color-primary, #4a9eff);
  color: white;
  border-color: var(--color-primary, #4a9eff);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .tabs-header {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }
  
  .tab-button {
    white-space: nowrap;
    flex-shrink: 0;
  }
  
  .category-grid {
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  }
  
  .category-button {
    padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
    font-size: 0.85rem;
  }
}
</style>