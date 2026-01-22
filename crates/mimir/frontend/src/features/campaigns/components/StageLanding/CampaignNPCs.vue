<template>
  <div class="campaign-npcs">
    <div class="section-header">
      <h3 class="section-title">Campaign NPCs</h3>
      <button class="btn-secondary btn-sm" @click="createNpc">
        + Add NPC
      </button>
    </div>

    <div v-if="loading" class="loading-state">
      Loading NPCs...
    </div>

    <EmptyState
      v-else-if="npcs.length === 0"
      variant="characters"
      title="No NPCs in this campaign yet"
      description="Create NPCs with full character sheets using the button above."
    />

    <div v-else class="npc-grid">
      <div
        v-for="npc in npcs"
        :key="npc.id"
        class="npc-card"
        @click="viewNpc(npc)"
      >
        <div class="npc-header">
          <h4 class="npc-name">{{ npc.name }}</h4>
          <span class="npc-badge">NPC</span>
        </div>
        <div class="npc-details">
          <span class="npc-class">
            {{ formatDetails(npc) }}
          </span>
        </div>
      </div>
    </div>

    <!-- Character Creation Wizard Modal -->
    <CharacterCreationWizard
      :visible="showWizard"
      :campaign-id="campaignId"
      :start-as-npc="true"
      @close="showWizard = false"
      @created="handleNpcCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useCharacterStore } from '@/stores/characters'
import CharacterCreationWizard from '@/features/characters/components/CharacterCreationWizard.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import type { Character } from '@/types/character'

const props = defineProps<{
  campaignId: string
}>()

const router = useRouter()
const characterStore = useCharacterStore()

const showWizard = ref(false)
const loading = ref(false)

// Get NPCs from store
const npcs = computed(() => {
  return characterStore.characters.filter((c: Character) => c.campaign_id === props.campaignId && c.is_npc === 1)
})

// Format character details
function formatDetails(npc: Character): string {
  const parts: string[] = []
  if (npc.race_name) parts.push(npc.race_name)
  if (npc.background_name) parts.push(npc.background_name)
  return parts.join(' ') || 'No details'
}

const loadNpcs = async () => {
  loading.value = true
  try {
    await characterStore.fetchNpcs(props.campaignId)
  } catch (e) {
    console.error('Failed to load NPCs:', e)
  } finally {
    loading.value = false
  }
}

const createNpc = () => {
  showWizard.value = true
}

const viewNpc = (npc: Character) => {
  router.push(`/characters/${npc.id}`)
}

const handleNpcCreated = () => {
  showWizard.value = false
  loadNpcs()
}

// Watch for campaign changes
watch(() => props.campaignId, () => {
  loadNpcs()
})

onMounted(() => {
  loadNpcs()
})
</script>

<style scoped>
.campaign-npcs {
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

.npc-badge {
  display: inline-block;
  padding: 2px 6px;
  font-size: 0.625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  background-color: var(--color-warning, #f59e0b);
  color: white;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.npc-details {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}
</style>
