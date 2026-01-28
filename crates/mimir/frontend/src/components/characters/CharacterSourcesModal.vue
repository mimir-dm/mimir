<template>
  <AppModal
    :visible="visible"
    title="Character Sources"
    size="md"
    @close="$emit('close')"
  >
    <div class="sources-modal">
      <p class="description">
        Select which source books this character can use for spells, feats, races, classes,
        and other character options.
      </p>

      <div v-if="loading" class="loading-state">
        Loading sources...
      </div>

      <div v-else-if="availableSources.length === 0" class="empty-state">
        No sources imported yet. Import 5etools data to see available sources.
      </div>

      <div v-else class="sources-list">
        <label
          v-for="source in availableSources"
          :key="source.id"
          class="source-item"
          :class="{ selected: selectedSources.has(source.id) }"
        >
          <input
            type="checkbox"
            :checked="selectedSources.has(source.id)"
            @change="toggleSource(source.id)"
          />
          <span class="source-code">{{ source.id }}</span>
          <span class="source-name">{{ source.name }}</span>
        </label>
      </div>

      <div class="quick-actions">
        <button @click="selectAll" class="btn btn-sm btn-secondary">Select All</button>
        <button @click="selectNone" class="btn btn-sm btn-secondary">Select None</button>
        <button @click="selectCore" class="btn btn-sm btn-secondary">Core Only</button>
      </div>
    </div>

    <template #footer>
      <div class="footer-info">
        <span v-if="hasChanges" class="unsaved-indicator">Unsaved changes</span>
        <span class="selected-count">{{ selectedSources.size }} sources selected</span>
      </div>
      <div class="footer-actions">
        <button @click="$emit('close')" class="btn btn-secondary">Cancel</button>
        <button
          @click="save"
          class="btn btn-primary"
          :disabled="saving || !hasChanges"
        >
          {{ saving ? 'Saving...' : 'Save' }}
        </button>
      </div>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'

interface BookInfo {
  id: string
  name: string
}

const props = defineProps<{
  visible: boolean
  characterId: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved'): void
}>()

const loading = ref(false)
const saving = ref(false)
const availableSources = ref<BookInfo[]>([])
const selectedSources = ref<Set<string>>(new Set())
const originalSources = ref<Set<string>>(new Set())

// Core D&D 5e books for player options
const CORE_SOURCES = ['PHB', 'XPHB', 'DMG', 'XDMG', 'PHB2024', 'DMG2024', 'XGE', 'TCE']

const hasChanges = computed(() => {
  if (selectedSources.value.size !== originalSources.value.size) return true
  for (const s of selectedSources.value) {
    if (!originalSources.value.has(s)) return true
  }
  return false
})

watch(() => props.visible, async (newVal) => {
  if (newVal) {
    await loadData()
  }
})

async function loadData() {
  loading.value = true
  try {
    // Load available sources from catalog
    const sourcesResult = await invoke<{ success: boolean; data?: BookInfo[] }>('list_catalog_sources')
    if (sourcesResult.success && sourcesResult.data) {
      availableSources.value = sourcesResult.data.sort((a, b) => a.name.localeCompare(b.name))
    }

    // Load character's selected sources
    const characterResult = await invoke<{ success: boolean; data?: string[] }>('list_character_sources', {
      characterId: props.characterId
    })
    if (characterResult.success && characterResult.data) {
      selectedSources.value = new Set(characterResult.data)
      originalSources.value = new Set(characterResult.data)
    } else {
      // If no sources configured, default to all
      selectedSources.value = new Set(availableSources.value.map(s => s.id))
      originalSources.value = new Set()
    }
  } catch (err) {
    console.error('Failed to load sources:', err)
  } finally {
    loading.value = false
  }
}

function toggleSource(sourceId: string) {
  const newSet = new Set(selectedSources.value)
  if (newSet.has(sourceId)) {
    newSet.delete(sourceId)
  } else {
    newSet.add(sourceId)
  }
  selectedSources.value = newSet
}

function selectAll() {
  selectedSources.value = new Set(availableSources.value.map(s => s.id))
}

function selectNone() {
  selectedSources.value = new Set()
}

function selectCore() {
  const coreSet = new Set<string>()
  for (const source of availableSources.value) {
    if (CORE_SOURCES.includes(source.id)) {
      coreSet.add(source.id)
    }
  }
  selectedSources.value = coreSet
}

async function save() {
  saving.value = true
  try {
    const result = await invoke<{ success: boolean; error?: string }>('set_character_sources', {
      characterId: props.characterId,
      sourceCodes: Array.from(selectedSources.value)
    })

    if (result.success) {
      originalSources.value = new Set(selectedSources.value)
      emit('saved')
      emit('close')
    } else {
      alert(`Failed to save: ${result.error}`)
    }
  } catch (err) {
    console.error('Failed to save sources:', err)
    alert('Failed to save sources')
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.sources-modal {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.description {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  margin: 0;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

.sources-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm);
}

.source-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.source-item:hover {
  background: var(--color-surface-variant);
}

.source-item.selected {
  background: var(--color-surface-hover);
}

.source-item input[type="checkbox"] {
  cursor: pointer;
}

.source-code {
  font-family: monospace;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  min-width: 60px;
}

.source-name {
  flex: 1;
  font-size: 0.875rem;
}

.quick-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.btn-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.8rem;
}

/* Footer layout */
:deep(.modal-footer) {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  font-size: 0.875rem;
}

.unsaved-indicator {
  color: var(--color-warning);
  font-weight: 500;
}

.selected-count {
  color: var(--color-text-secondary);
}

.footer-actions {
  display: flex;
  gap: var(--spacing-sm);
}

/* Button styles */
.btn {
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  font-weight: 500;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: none;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-primary-600);
}

.btn-secondary {
  background: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--color-gray-200);
}

.theme-dark .btn-secondary:hover:not(:disabled) {
  background: var(--color-gray-700);
}
</style>
