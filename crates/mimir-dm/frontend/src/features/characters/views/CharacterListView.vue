<template>
  <MainLayout>
    <div class="character-list-view">
      <div class="header">
        <h1 class="page-title">Characters</h1>
        <button @click="createCharacter" class="btn-primary">
          Create Character
        </button>
      </div>

      <!-- Filter Tabs -->
      <div class="filter-tabs">
        <button
          class="filter-tab"
          :class="{ active: characterFilter === 'all' }"
          @click="characterFilter = 'all'"
        >
          All ({{ allCharactersCount }})
        </button>
        <button
          class="filter-tab"
          :class="{ active: characterFilter === 'pc' }"
          @click="characterFilter = 'pc'"
        >
          Player Characters ({{ pcCount }})
        </button>
        <button
          class="filter-tab"
          :class="{ active: characterFilter === 'npc' }"
          @click="characterFilter = 'npc'"
        >
          NPCs ({{ npcCount }})
        </button>
      </div>

      <div v-if="characterStore.loading" class="loading">
        Loading characters...
      </div>

      <div v-else-if="characterStore.error" class="error-message">
        {{ characterStore.error }}
      </div>

      <EmptyState
        v-else-if="characters.length === 0"
        variant="characters"
        title="No characters yet"
        description="Create your first character to get started on your adventure"
      >
        <template #action>
          <button @click="createCharacter" class="btn-primary">
            Create Character
          </button>
        </template>
      </EmptyState>

      <div v-else class="character-sections">
        <!-- Unassigned Characters -->
        <div v-if="unassignedCharacters.length > 0" class="character-section">
          <h2 class="section-title">Unassigned Characters</h2>
          <div class="character-grid">
            <div
              v-for="character in unassignedCharacters"
              :key="character.id"
              class="character-card"
              :class="{ 'is-npc': character.is_npc }"
              @click="viewCharacter(character)"
            >
              <div class="character-header">
                <h3 class="character-name">{{ character.character_name }}</h3>
                <span v-if="character.is_npc" class="npc-badge">NPC</span>
              </div>
              <div class="character-class-race">
                Level {{ character.current_level }} {{ character.race || '' }} {{ character.class || '' }}
              </div>
              <div class="character-meta">
                <span class="character-player">{{ getPlayerName(character.player_id) }}</span>
              </div>
              <div class="character-actions" @click.stop>
                <div class="action-buttons">
                  <button @click="editCharacter(character)" class="btn-action" title="Edit">
                    Edit
                  </button>
                  <button @click="printCharacter(character)" class="btn-action" title="Print PDF">
                    Print
                  </button>
                  <button @click="levelUpCharacter(character)" class="btn-action" title="Level Up">
                    Level Up
                  </button>
                  <button @click="deleteCharacter(character)" class="btn-action btn-action-danger" title="Delete">
                    Delete
                  </button>
                </div>
                <select
                  class="campaign-select"
                  @change="assignToCampaign(character.id, $event)"
                >
                  <option value="">Add to Campaign...</option>
                  <option
                    v-for="campaign in campaignStore.campaigns"
                    :key="campaign.id"
                    :value="campaign.id"
                  >
                    {{ campaign.name }}
                  </option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- Campaign Characters -->
        <div v-for="(chars, campaignId) in charactersByCampaign" :key="campaignId" class="character-section">
          <h2 class="section-title">{{ getCampaignName(Number(campaignId)) }}</h2>
          <div class="character-grid">
            <div
              v-for="character in chars"
              :key="character.id"
              class="character-card"
              :class="{ 'is-npc': character.is_npc }"
              @click="viewCharacter(character)"
            >
              <div class="character-header">
                <h3 class="character-name">{{ character.character_name }}</h3>
                <span v-if="character.is_npc" class="npc-badge">NPC</span>
              </div>
              <div class="character-class-race">
                Level {{ character.current_level }} {{ character.race || '' }} {{ character.class || '' }}
              </div>
              <div class="character-meta">
                <span class="character-player">{{ getPlayerName(character.player_id) }}</span>
              </div>
              <div class="character-actions" @click.stop>
                <div class="action-buttons">
                  <button @click="editCharacter(character)" class="btn-action" title="Edit">
                    Edit
                  </button>
                  <button @click="printCharacter(character)" class="btn-action" title="Print PDF">
                    Print
                  </button>
                  <button @click="levelUpCharacter(character)" class="btn-action" title="Level Up">
                    Level Up
                  </button>
                  <button @click="deleteCharacter(character)" class="btn-action btn-action-danger" title="Delete">
                    Delete
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Character Creation Wizard -->
    <CharacterCreationWizard
      :visible="showWizard"
      @close="handleWizardClose"
      @created="handleCharacterCreated"
    />

    <!-- Character Print Dialog -->
    <CharacterPrintDialog
      v-if="selectedCharacterForPrint"
      :visible="showPrintDialog"
      :character-id="selectedCharacterForPrint.id"
      :character-name="selectedCharacterForPrint.character_name"
      @close="closePrintDialog"
    />

    <!-- Level Up Dialog -->
    <LevelUpDialog
      v-if="selectedCharacterForLevelUp && selectedCharacterData"
      :visible="showLevelUpDialog"
      :character-id="selectedCharacterForLevelUp.id"
      :character-data="selectedCharacterData"
      @close="closeLevelUpDialog"
      @completed="handleLevelUpCompleted"
    />

    <!-- Delete Confirmation Dialog -->
    <AppModal
      :visible="showDeleteDialog"
      title="Confirm Delete"
      size="sm"
      @close="closeDeleteDialog"
    >
      <p>
        Are you sure you want to delete
        <strong>{{ characterToDelete?.character_name }}</strong>?
      </p>
      <p class="warning-text">
        This action cannot be undone.
      </p>

      <template #footer>
        <button type="button" @click="closeDeleteDialog" class="btn btn-secondary">
          Cancel
        </button>
        <button
          @click="confirmDelete"
          class="btn btn-danger"
          :disabled="deleting"
        >
          {{ deleting ? 'Deleting...' : 'Delete Character' }}
        </button>
      </template>
    </AppModal>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import MainLayout from '../../../shared/components/layout/MainLayout.vue'
import CharacterCreationWizard from '../components/CharacterCreationWizard.vue'
import LevelUpDialog from '../components/LevelUpDialog.vue'
import { CharacterPrintDialog } from '../../../components/print'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '../../../shared/components/ui/EmptyState.vue'
import { useCharacterStore } from '../../../stores/characters'
import { usePlayerStore } from '../../../stores/players'
import { useCampaignStore } from '../../../stores/campaigns'
import type { Character, CharacterData } from '../../../types/character'

const router = useRouter()
const characterStore = useCharacterStore()
const playerStore = usePlayerStore()
const campaignStore = useCampaignStore()

onMounted(async () => {
  // Load all data
  await Promise.all([
    playerStore.fetchPlayers(),
    campaignStore.fetchCampaigns(),
    characterStore.fetchAllCharacters()
  ])
})

// Filter state
type CharacterFilter = 'all' | 'pc' | 'npc'
const characterFilter = ref<CharacterFilter>('all')

// Counts for filter tabs
const allCharactersCount = computed(() => characterStore.characters.length)
const pcCount = computed(() => characterStore.characters.filter(c => !c.is_npc).length)
const npcCount = computed(() => characterStore.characters.filter(c => c.is_npc).length)

// Filtered characters based on selected filter
const characters = computed(() => {
  const all = characterStore.characters
  switch (characterFilter.value) {
    case 'pc':
      return all.filter(c => !c.is_npc)
    case 'npc':
      return all.filter(c => c.is_npc)
    default:
      return all
  }
})

// Sort characters: PCs first, then NPCs, then alphabetically
const sortCharacters = (chars: Character[]) => {
  return [...chars].sort((a, b) => {
    // PCs (is_npc = 0/false) come before NPCs (is_npc = 1/true)
    if (a.is_npc !== b.is_npc) {
      return a.is_npc ? 1 : -1
    }
    // Then sort alphabetically by name
    return a.character_name.localeCompare(b.character_name)
  })
}

const unassignedCharacters = computed(() =>
  sortCharacters(characters.value.filter(c => c.campaign_id === null))
)

const charactersByCampaign = computed(() => {
  const grouped: Record<number, Character[]> = {}

  characters.value
    .filter(c => c.campaign_id !== null)
    .forEach(character => {
      const campaignId = character.campaign_id!
      if (!grouped[campaignId]) {
        grouped[campaignId] = []
      }
      grouped[campaignId].push(character)
    })

  // Sort each campaign's characters
  for (const campaignId in grouped) {
    grouped[campaignId] = sortCharacters(grouped[campaignId])
  }

  return grouped
})

const getPlayerName = (playerId: number | null): string => {
  if (playerId === null) {
    return 'NPC'
  }
  const player = playerStore.players.find(p => p.id === playerId)
  return player?.name || 'Unknown Player'
}

const getCampaignName = (campaignId: number): string => {
  const campaign = campaignStore.campaigns.find(c => c.id === campaignId)
  return campaign?.name || 'Unknown Campaign'
}

const showWizard = ref(false)

const createCharacter = () => {
  showWizard.value = true
}

const handleWizardClose = () => {
  showWizard.value = false
}

const handleCharacterCreated = async () => {
  showWizard.value = false
  // Reload characters list
  await characterStore.fetchAllCharacters()
}

const viewCharacter = (character: Character) => {
  router.push(`/characters/${character.id}`)
}

const assignToCampaign = async (characterId: number, event: Event) => {
  const select = event.target as HTMLSelectElement
  const campaignId = parseInt(select.value)

  if (!campaignId) return

  try {
    await invoke('assign_character_to_campaign', {
      characterId,
      campaignId
    })
    // Reload characters to show updated list
    await characterStore.fetchAllCharacters()
  } catch (error) {
    console.error('Failed to assign character to campaign:', error)
    characterStore.error = `Failed to assign character: ${error}`
  }

  // Reset select
  select.value = ''
}

// Character action handlers
const editCharacter = (character: Character) => {
  // Navigate to character sheet in edit mode
  router.push(`/characters/${character.id}?edit=true`)
}

// Print dialog state
const showPrintDialog = ref(false)
const selectedCharacterForPrint = ref<Character | null>(null)

const printCharacter = (character: Character) => {
  selectedCharacterForPrint.value = character
  showPrintDialog.value = true
}

const closePrintDialog = () => {
  showPrintDialog.value = false
  // Don't null selectedCharacterForPrint here - it would unmount the component
  // and destroy the PdfPreviewModal before it can show the result
}

// Level up dialog state
const showLevelUpDialog = ref(false)
const selectedCharacterForLevelUp = ref<Character | null>(null)
const selectedCharacterData = ref<CharacterData | null>(null)

const levelUpCharacter = async (character: Character) => {
  try {
    // Fetch full character data for the level up dialog
    const result = await invoke<{ character: Character; data: CharacterData }>('get_character', {
      characterId: character.id
    })
    selectedCharacterForLevelUp.value = character
    selectedCharacterData.value = result.data
    showLevelUpDialog.value = true
  } catch (error) {
    console.error('Failed to load character data:', error)
    characterStore.error = `Failed to load character: ${error}`
  }
}

const closeLevelUpDialog = () => {
  showLevelUpDialog.value = false
  selectedCharacterForLevelUp.value = null
  selectedCharacterData.value = null
}

const handleLevelUpCompleted = async () => {
  closeLevelUpDialog()
  // Reload characters to show updated level
  await characterStore.fetchAllCharacters()
}

// Delete dialog state
const showDeleteDialog = ref(false)
const characterToDelete = ref<Character | null>(null)
const deleting = ref(false)

const deleteCharacter = (character: Character) => {
  characterToDelete.value = character
  showDeleteDialog.value = true
}

const closeDeleteDialog = () => {
  showDeleteDialog.value = false
  characterToDelete.value = null
}

const confirmDelete = async () => {
  if (!characterToDelete.value) return

  deleting.value = true
  try {
    await characterStore.deleteCharacter(characterToDelete.value.id)
    await characterStore.fetchAllCharacters()
    closeDeleteDialog()
  } catch (error) {
    console.error('Failed to delete character:', error)
    characterStore.error = `Failed to delete character: ${error}`
  } finally {
    deleting.value = false
  }
}
</script>

<style scoped>
.character-list-view {
  @apply space-y-6;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-title {
  font-size: 2rem;
  font-weight: 700;
  color: var(--color-text);
}

.loading,
.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-secondary);
}

.empty-state {
  @apply space-y-4;
}

.empty-subtitle {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.character-sections {
  @apply space-y-8;
}

.character-section {
  @apply space-y-4;
}

.section-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text);
  padding-bottom: var(--spacing-sm);
  border-bottom: 2px solid var(--color-border);
}

.character-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: var(--spacing-lg);
}

.character-card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  cursor: pointer;
  transition: all var(--transition-base);
}

.character-card:hover {
  border-color: var(--color-primary-500);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.character-name {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-xs);
}

.character-class-race {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-primary-500);
  margin-bottom: var(--spacing-sm);
}

.character-meta {
  display: flex;
  gap: var(--spacing-md);
}

.character-player {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.character-actions {
  margin-top: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--color-border);
}

.campaign-select {
  width: 100%;
  padding: var(--spacing-xs) var(--spacing-sm);
  background-color: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
}

.campaign-select:hover {
  border-color: var(--color-primary-500);
}

.campaign-select:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px var(--color-primary-500) / 0.2;
}

.btn-primary {
  padding: var(--spacing-sm) var(--spacing-lg);
  background-color: var(--color-primary-500);
  color: var(--color-background);
  border-radius: var(--radius-md);
  border: none;
  font-weight: 500;
  transition: all var(--transition-fast);
  cursor: pointer;
}

.btn-primary:hover {
  background-color: var(--color-primary-600);
  transform: translateY(-1px);
}

.error-message {
  padding: var(--spacing-md);
  background-color: var(--color-error) / 0.1;
  border: 1px solid var(--color-error) / 0.2;
  border-radius: var(--radius-md);
  color: var(--color-error);
}

/* Filter Tabs */
.filter-tabs {
  display: flex;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  background-color: var(--color-surface);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border);
}

.filter-tab {
  padding: var(--spacing-sm) var(--spacing-md);
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.filter-tab:hover {
  background-color: var(--color-background);
  color: var(--color-text);
}

.filter-tab.active {
  background-color: var(--color-primary-500);
  color: var(--color-background);
}

/* Character Card Header */
.character-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--spacing-sm);
}

.character-header .character-name {
  margin-bottom: 0;
}

/* NPC Badge */
.npc-badge {
  display: inline-block;
  padding: 2px 8px;
  font-size: 0.625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  background-color: var(--color-warning, #f59e0b);
  color: white;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

/* NPC card styling */
.character-card.is-npc {
  border-left: 3px solid var(--color-warning, #f59e0b);
}

/* Action Buttons */
.action-buttons {
  display: flex;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
}

.btn-action {
  flex: 1;
  padding: var(--spacing-xs) var(--spacing-sm);
  background-color: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-action:hover {
  background-color: var(--color-surface);
  border-color: var(--color-primary-500);
  color: var(--color-primary-500);
}

.btn-action-danger:hover {
  background-color: var(--color-error);
  border-color: var(--color-error);
  color: white;
}

/* Delete dialog */
.warning-text {
  margin-top: var(--spacing-md);
  color: var(--color-warning-600);
  font-size: 0.875rem;
}

.theme-dark .warning-text,
.theme-hyper .warning-text {
  color: var(--color-warning-400);
}
</style>
