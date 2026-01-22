<template>
  <div class="pcs-tab">
    <!-- Header -->
    <div class="tab-header">
      <h2>Player Characters</h2>
      <div class="header-actions">
        <button @click="showAddModal = true" class="btn btn-secondary">
          Add Existing
        </button>
        <button @click="showCreateWizard = true" class="btn btn-primary">
          Create PC
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading-state">
      Loading characters...
    </div>

    <!-- Empty state -->
    <div v-else-if="pcs.length === 0" class="empty-state">
      <div class="empty-icon">@</div>
      <h3>No player characters yet</h3>
      <p>Create a character for one of your players.</p>
      <button @click="showCreateWizard = true" class="btn btn-primary">
        Create PC
      </button>
    </div>

    <!-- Character Grid -->
    <div v-else class="character-grid">
      <div
        v-for="character in pcs"
        :key="character.id"
        class="character-card"
        @click="viewCharacter(character)"
      >
        <div class="card-header">
          <h3 class="character-name">{{ character.name }}</h3>
        </div>
        <div class="character-details">
          {{ formatCharacterDetails(character) }}
        </div>
        <div class="character-player">
          {{ character.player_name || 'No player assigned' }}
        </div>
        <div class="card-actions" @click.stop>
          <button @click="viewCharacter(character)" class="btn btn-sm btn-ghost">
            View
          </button>
          <button @click="printCharacter(character)" class="btn btn-sm btn-ghost">
            Print
          </button>
        </div>
      </div>
    </div>

    <!-- Character Creation Wizard -->
    <CharacterCreationWizard
      :visible="showCreateWizard"
      :campaign-id="campaign?.id"
      :start-as-npc="false"
      @close="showCreateWizard = false"
      @created="handleCharacterCreated"
    />

    <!-- Character Print Dialog -->
    <CharacterPrintDialog
      v-if="printingCharacter"
      :visible="showPrintDialog"
      :character-id="printingCharacter.id"
      :character-name="printingCharacter.name"
      @close="closePrintDialog"
    />

    <!-- Add Existing Character Modal -->
    <AddCharacterModal
      v-if="campaign"
      :visible="showAddModal"
      :campaign-id="campaign.id"
      :is-npc="false"
      @close="showAddModal = false"
      @added="handleCharacterAdded"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useCharacterStore } from '@/stores/characters'
import CharacterCreationWizard from '@/features/characters/components/CharacterCreationWizard.vue'
import { CharacterPrintDialog } from '@/components/print'
import AddCharacterModal from './AddCharacterModal.vue'
import type { Campaign } from '@/types'
import type { Character } from '@/types/character'

const props = defineProps<{
  campaign?: Campaign
  documents?: any[]
}>()

const router = useRouter()
const characterStore = useCharacterStore()

// Local state
const loading = ref(false)
const showCreateWizard = ref(false)
const showAddModal = ref(false)
const showPrintDialog = ref(false)
const printingCharacter = ref<Character | null>(null)

// PCs only (is_npc === 0 means PC)
const pcs = computed(() => {
  if (!props.campaign?.id) return []
  return characterStore.characters.filter(c =>
    c.campaign_id === props.campaign!.id && c.is_npc === 0
  )
})

// Format character details
function formatCharacterDetails(character: Character): string {
  const parts: string[] = []
  if (character.race_name) {
    parts.push(character.race_name)
  }
  if (character.background_name) {
    parts.push(character.background_name)
  }
  return parts.join(' ') || 'No details'
}

// Load characters
async function loadCharacters() {
  if (!props.campaign?.id) return
  loading.value = true
  try {
    await characterStore.fetchPcs(props.campaign.id)
  } catch (e) {
    console.error('Failed to load characters:', e)
  } finally {
    loading.value = false
  }
}

// View character
function viewCharacter(character: Character) {
  router.push(`/characters/${character.id}`)
}

// Print character
function printCharacter(character: Character) {
  printingCharacter.value = character
  showPrintDialog.value = true
}

function closePrintDialog() {
  showPrintDialog.value = false
}

// Handle character created
async function handleCharacterCreated() {
  showCreateWizard.value = false
  await loadCharacters()
}

// Handle character added from existing
async function handleCharacterAdded() {
  showAddModal.value = false
  await loadCharacters()
}

// Watch for campaign changes
watch(() => props.campaign?.id, () => {
  loadCharacters()
}, { immediate: true })

onMounted(() => {
  loadCharacters()
})
</script>

<style scoped>
.pcs-tab {
  padding: var(--spacing-lg);
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tab-header h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
}

/* Loading/Empty states */
.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: var(--spacing-md);
  text-align: center;
  color: var(--color-text-secondary);
}

.empty-icon {
  font-size: 3rem;
  opacity: 0.5;
}

.empty-state h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
}

.empty-state p {
  margin: 0;
  font-size: 0.875rem;
}

/* Character Grid */
.character-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-md);
}

.character-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  cursor: pointer;
  transition: all var(--transition-base);
}

.character-card:hover {
  border-color: var(--color-primary-500);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-xs);
}

.character-name {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
}

.character-details {
  font-size: 0.875rem;
  color: var(--color-primary-500);
  margin-bottom: var(--spacing-xs);
}

.character-player {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.card-actions {
  display: flex;
  gap: var(--spacing-xs);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--color-border);
}

.card-actions .btn {
  flex: 1;
}
</style>
