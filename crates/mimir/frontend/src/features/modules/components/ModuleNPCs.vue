<template>
  <div class="module-npcs">
    <div class="section-header">
      <h3 class="section-title">Module NPCs</h3>
      <button class="btn-secondary btn-sm" @click="openSelector">
        + Add NPC
      </button>
    </div>

    <div v-if="loading" class="loading-state">
      Loading NPCs...
    </div>

    <EmptyState
      v-else-if="moduleNpcs.length === 0"
      variant="characters"
      title="No NPCs in this module"
      description="Add existing campaign NPCs to this module."
    />

    <div v-else class="npc-grid">
      <div
        v-for="npc in moduleNpcs"
        :key="npc.id"
        class="npc-card"
        @click="viewNpc(npc)"
      >
        <div class="npc-header">
          <h4 class="npc-name">{{ npc.character_name }}</h4>
          <button
            class="remove-npc"
            title="Remove from module"
            @click.stop="removeNpc(npc)"
          >
            &times;
          </button>
        </div>
        <div class="npc-details">
          <span v-if="npc.role" class="npc-role">{{ npc.role }}</span>
        </div>
      </div>
    </div>

    <!-- NPC Selector Modal -->
    <NpcSelectorModal
      :visible="showSelector"
      :module-id="moduleId"
      :campaign-id="campaignId"
      :existing-npc-ids="existingCharacterIds"
      @close="showSelector = false"
      @added="handleNpcsAdded"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import NpcSelectorModal from './NpcSelectorModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import { dataEvents } from '@/shared/utils/dataEvents'
import type { ApiResponse } from '@/types/api'

interface ModuleNpcWithCharacter {
  id: string
  module_id: string
  character_id: string
  role: string | null
  encounter_tag: string | null
  notes: string | null
  character_name: string
}

const props = defineProps<{
  moduleId: string
  campaignId: string
}>()

const router = useRouter()

const showSelector = ref(false)
const loading = ref(false)
const moduleNpcs = ref<ModuleNpcWithCharacter[]>([])

// Get character IDs that are already in the module
const existingCharacterIds = computed(() => {
  return moduleNpcs.value.map(npc => npc.character_id)
})

const loadNpcs = async () => {
  loading.value = true
  try {
    const response = await invoke<ApiResponse<ModuleNpcWithCharacter[]>>('list_module_npcs_with_data', {
      moduleId: props.moduleId
    })
    if (response.success && response.data) {
      moduleNpcs.value = response.data
    }
  } catch (e) {
    console.error('Failed to load module NPCs:', e)
  } finally {
    loading.value = false
  }
}

const openSelector = () => {
  showSelector.value = true
}

const viewNpc = (npc: ModuleNpcWithCharacter) => {
  router.push(`/characters/${npc.character_id}`)
}

const removeNpc = async (npc: ModuleNpcWithCharacter) => {
  try {
    await invoke<ApiResponse<void>>('remove_module_npc', {
      npcId: npc.id
    })
    dataEvents.emit('module:npcs:changed', { moduleId: props.moduleId })
    loadNpcs()
  } catch (e) {
    console.error('Failed to remove NPC:', e)
  }
}

const handleNpcsAdded = () => {
  showSelector.value = false
  loadNpcs()
}

// Watch for module changes
watch(() => props.moduleId, () => {
  loadNpcs()
})

// Subscribe to NPC change events for automatic refresh
let unsubscribe: (() => void) | null = null

onMounted(() => {
  loadNpcs()
  unsubscribe = dataEvents.on('module:npcs:changed', (payload) => {
    if (payload.moduleId === props.moduleId) {
      loadNpcs()
    }
  })
})

onUnmounted(() => {
  if (unsubscribe) {
    unsubscribe()
    unsubscribe = null
  }
})
</script>

<style scoped>
.module-npcs {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.section-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.btn-secondary {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  font-weight: 500;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: var(--color-surface);
  border-color: var(--color-primary-500);
}

.btn-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
}

.loading-state {
  text-align: center;
  padding: var(--spacing-lg);
  color: var(--color-text-secondary);
}

.npc-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.npc-card {
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-left: 3px solid var(--color-warning, #f59e0b);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.npc-card:hover {
  border-color: var(--color-primary-500);
  box-shadow: var(--shadow-sm);
}

.npc-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-xs);
}

.npc-name {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.remove-npc {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0;
  line-height: 1;
  opacity: 0;
  transition: opacity var(--transition-fast), color var(--transition-fast);
}

.npc-card:hover .remove-npc {
  opacity: 1;
}

.remove-npc:hover {
  color: var(--color-error, #ef4444);
}

.npc-details {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.npc-role {
  display: inline-block;
  padding: 2px 6px;
  font-size: 0.625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  background-color: var(--color-primary-100);
  color: var(--color-primary-700);
  border-radius: var(--radius-sm);
}

.theme-dark .npc-role {
  background-color: var(--color-primary-900);
  color: var(--color-primary-300);
}
</style>
