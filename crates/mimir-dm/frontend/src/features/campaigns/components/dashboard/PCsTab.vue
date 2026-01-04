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
          <h3 class="character-name">{{ character.character_name }}</h3>
        </div>
        <div class="character-details">
          Level {{ character.current_level }} {{ character.race || '' }} {{ character.class || '' }}
        </div>
        <div class="character-player">
          {{ getPlayerName(character.player_id) }}
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
      :character-name="printingCharacter.character_name"
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
import { usePlayerStore } from '@/stores/players'
import CharacterCreationWizard from '@/features/characters/components/CharacterCreationWizard.vue'
import { CharacterPrintDialog } from '@/components/print'
import AddCharacterModal from './AddCharacterModal.vue'
import type { Campaign, BoardConfig } from '@/types'
import type { Character } from '@/types/character'

const props = defineProps<{
  campaign?: Campaign
  boardConfig?: BoardConfig
  documents?: any[]
}>()

const router = useRouter()
const characterStore = useCharacterStore()
const playerStore = usePlayerStore()

// Local state
const loading = ref(false)
const showCreateWizard = ref(false)
const showAddModal = ref(false)
const showPrintDialog = ref(false)
const printingCharacter = ref<Character | null>(null)

// PCs only
const pcs = computed(() => {
  if (!props.campaign?.id) return []
  return characterStore.characters.filter(c =>
    c.campaign_id === props.campaign!.id && !c.is_npc
  )
})

// Get player name
function getPlayerName(playerId: number | null): string {
  if (playerId === null) return 'No player assigned'
  const player = playerStore.players.find(p => p.id === playerId)
  return player?.name || 'Unknown Player'
}

// Load characters
async function loadCharacters() {
  loading.value = true
  try {
    await Promise.all([
      characterStore.fetchAllCharacters(),
      playerStore.fetchPlayers()
    ])
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
  padding: var(--spacing-lg, 16px);
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md, 12px);
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
  color: var(--color-text, #e0e0e0);
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm, 8px);
}

/* Loading/Empty states */
.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: var(--spacing-md, 12px);
  text-align: center;
  color: var(--color-text-muted, #888);
}

.empty-icon {
  font-size: 3rem;
  opacity: 0.5;
}

.empty-state h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.empty-state p {
  margin: 0;
  font-size: 0.875rem;
}

/* Character Grid */
.character-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-md, 12px);
}

.character-card {
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: var(--radius-md, 8px);
  padding: var(--spacing-md, 12px);
  cursor: pointer;
  transition: all 0.2s;
}

.character-card:hover {
  border-color: var(--color-primary, #4a9eff);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-xs, 4px);
}

.character-name {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text, #e0e0e0);
}

.character-details {
  font-size: 0.875rem;
  color: var(--color-primary, #4a9eff);
  margin-bottom: var(--spacing-xs, 4px);
}

.character-player {
  font-size: 0.75rem;
  color: var(--color-text-muted, #888);
  margin-bottom: var(--spacing-sm, 8px);
}

.card-actions {
  display: flex;
  gap: var(--spacing-xs, 4px);
  padding-top: var(--spacing-sm, 8px);
  border-top: 1px solid var(--color-border, #333);
}

.card-actions .btn {
  flex: 1;
}
</style>
