<template>
  <AppModal
    :visible="visible"
    title="Add NPC to Module"
    size="md"
    @close="handleClose"
  >
    <div class="npc-selector">
      <!-- Search/Filter -->
      <div class="search-bar">
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="Search NPCs..."
        />
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="loading-state">
        <div class="progress-spinner"></div>
        <p>Loading NPCs...</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="availableNpcs.length === 0" class="empty-state">
        <p v-if="allCampaignNpcs.length === 0">
          No NPCs in this campaign yet.
        </p>
        <p v-else-if="searchQuery">
          No NPCs match "{{ searchQuery }}"
        </p>
        <p v-else>
          All campaign NPCs are already in this module.
        </p>
      </div>

      <!-- NPC List -->
      <div v-else class="npc-list">
        <div
          v-for="npc in filteredNpcs"
          :key="npc.id"
          class="npc-item"
          :class="{ selected: selectedNpcIds.includes(npc.id), 'already-added': isAlreadyInModule(npc.id) }"
          @click="toggleNpc(npc)"
        >
          <div class="npc-checkbox">
            <input
              type="checkbox"
              :checked="selectedNpcIds.includes(npc.id) || isAlreadyInModule(npc.id)"
              :disabled="isAlreadyInModule(npc.id)"
              @click.stop
              @change="toggleNpc(npc)"
            />
          </div>
          <div class="npc-info">
            <span class="npc-name">{{ npc.name }}</span>
            <span class="npc-details">
              {{ npc.race_name || 'Unknown Race' }}
            </span>
          </div>
          <span v-if="isAlreadyInModule(npc.id)" class="already-badge">Added</span>
        </div>
      </div>

      <!-- Selection Summary -->
      <div v-if="selectedNpcIds.length > 0" class="selection-summary">
        {{ selectedNpcIds.length }} NPC{{ selectedNpcIds.length > 1 ? 's' : '' }} selected
      </div>
    </div>

    <template #footer>
      <button class="btn btn-tertiary" @click="showWizard = true">
        Create New NPC
      </button>
      <div class="footer-spacer"></div>
      <button class="btn btn-secondary" @click="handleClose">
        Cancel
      </button>
      <button
        class="btn btn-primary"
        :disabled="selectedNpcIds.length === 0 || adding"
        @click="handleAdd"
      >
        {{ adding ? 'Adding...' : `Add NPC${selectedNpcIds.length > 1 ? 's' : ''}` }}
      </button>
    </template>
  </AppModal>

  <!-- Character Creation Wizard (nested modal) -->
  <CharacterCreationWizard
    :visible="showWizard"
    :campaign-id="campaignId"
    :start-as-npc="true"
    @close="showWizard = false"
    @created="handleNpcCreated"
  />
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import CharacterCreationWizard from '@/features/characters/components/CharacterCreationWizard.vue'
import { dataEvents } from '@/shared/utils/dataEvents'
import type { Character } from '@/types/character'

interface Props {
  visible: boolean
  moduleId: string
  campaignId: string
  existingNpcIds: string[] // Character IDs already in the module
}

interface Emits {
  (e: 'close'): void
  (e: 'added'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const loading = ref(false)
const adding = ref(false)
const searchQuery = ref('')
const allCampaignNpcs = ref<Character[]>([])
const selectedNpcIds = ref<string[]>([])
const showWizard = ref(false)

// Filter to NPCs not already in the module
const availableNpcs = computed(() => {
  return allCampaignNpcs.value.filter(npc => !props.existingNpcIds.includes(npc.id))
})

// Apply search filter
const filteredNpcs = computed(() => {
  const query = searchQuery.value.toLowerCase().trim()
  if (!query) return allCampaignNpcs.value

  return allCampaignNpcs.value.filter(npc => {
    const name = npc.name.toLowerCase()
    const race = (npc.race_name || '').toLowerCase()
    return name.includes(query) || race.includes(query)
  })
})

function isAlreadyInModule(characterId: string): boolean {
  return props.existingNpcIds.includes(characterId)
}

function toggleNpc(npc: Character) {
  if (isAlreadyInModule(npc.id)) return

  const idx = selectedNpcIds.value.indexOf(npc.id)
  if (idx >= 0) {
    selectedNpcIds.value.splice(idx, 1)
  } else {
    selectedNpcIds.value.push(npc.id)
  }
}

async function loadCampaignNpcs() {
  loading.value = true
  try {
    // This command returns Vec<Character> directly, not ApiResponse
    const characters = await invoke<Character[]>('list_characters_for_campaign', {
      campaignId: props.campaignId
    })
    // Filter to only NPCs (is_npc is a number: 1 = NPC, 0 = PC)
    allCampaignNpcs.value = characters.filter(c => c.is_npc === 1)
  } catch (e) {
    console.error('Failed to load campaign NPCs:', e)
    allCampaignNpcs.value = []
  } finally {
    loading.value = false
  }
}

async function handleAdd() {
  if (selectedNpcIds.value.length === 0) return

  adding.value = true
  try {
    // Add each selected NPC to the module
    for (const characterId of selectedNpcIds.value) {
      await invoke('add_module_npc', {
        request: {
          module_id: props.moduleId,
          character_id: characterId,
          role: null,
          encounter_tag: null,
          notes: null
        }
      })
    }
    dataEvents.emit('module:npcs:changed', { moduleId: props.moduleId })
    emit('added')
    handleClose()
  } catch (e) {
    console.error('Failed to add NPCs:', e)
  } finally {
    adding.value = false
  }
}

function handleClose() {
  selectedNpcIds.value = []
  searchQuery.value = ''
  emit('close')
}

// Handle NPC created from wizard - reload list to show the new NPC
async function handleNpcCreated() {
  showWizard.value = false
  await loadCampaignNpcs()
}

// Load NPCs when modal opens
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    loadCampaignNpcs()
  }
})
</script>

<style scoped>
.npc-selector {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  min-height: 300px;
}

.search-bar {
  position: sticky;
  top: 0;
  background: var(--color-surface);
  z-index: 1;
}

.search-input {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 0.875rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 3px var(--color-primary-100);
}

.theme-dark .search-input:focus {
  box-shadow: 0 0 0 3px var(--color-primary-900);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
  flex: 1;
}

.progress-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
  text-align: center;
  flex: 1;
}

.npc-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  max-height: 400px;
  overflow-y: auto;
}

.npc-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.npc-item:hover:not(.already-added) {
  border-color: var(--color-primary-300);
  background: var(--color-surface-variant);
}

.npc-item.selected {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
}

.theme-dark .npc-item.selected {
  background: var(--color-primary-900);
}

.npc-item.already-added {
  opacity: 0.6;
  cursor: default;
}

.npc-checkbox {
  display: flex;
  align-items: center;
}

.npc-checkbox input {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.npc-checkbox input:disabled {
  cursor: default;
}

.npc-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.npc-name {
  font-weight: 600;
  color: var(--color-text);
}

.npc-details {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.already-badge {
  font-size: 0.625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 2px 6px;
  background: var(--color-gray-100);
  color: var(--color-text-secondary);
  border-radius: var(--radius-sm);
}

.theme-dark .already-badge {
  background: var(--color-gray-700);
}

.selection-summary {
  padding: var(--spacing-sm);
  background: var(--color-primary-50);
  border-radius: var(--radius-md);
  text-align: center;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-primary-700);
}

.theme-dark .selection-summary {
  background: var(--color-primary-900);
  color: var(--color-primary-300);
}

.footer-spacer {
  flex: 1;
}

.btn-tertiary {
  background: none;
  border: none;
  color: var(--color-primary-600);
  font-weight: 500;
  cursor: pointer;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  transition: background-color var(--transition-fast);
}

.btn-tertiary:hover {
  background: var(--color-primary-50);
}

.theme-dark .btn-tertiary {
  color: var(--color-primary-400);
}

.theme-dark .btn-tertiary:hover {
  background: var(--color-primary-900);
}
</style>
