<template>
  <MainLayout>
    <div class="character-list-view">
      <div class="header">
        <h1 class="page-title">Player Characters</h1>
        <button @click="createCharacter" class="btn btn-primary">
          Create Character
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
        title="No player characters yet"
        description="Create your first player character to get started on your adventure. NPCs can be managed from each campaign's dashboard."
      >
        <template #action>
          <button @click="createCharacter" class="btn btn-primary">
            Create Character
          </button>
        </template>
      </EmptyState>

      <div v-else class="character-sections">
        <!-- Unassigned Characters -->
        <div v-if="unassignedCharacters.length > 0" class="character-section">
          <h2 class="section-title">Unassigned Characters</h2>
          <div class="character-grid">
            <CharacterCard
              v-for="character in unassignedCharacters"
              :key="character.id"
              :character="character"
              @click="viewCharacter"
            >
              <template #actions>
                <div class="extended-actions">
                  <div class="action-buttons">
                    <button @click="editCharacter(character)" class="btn btn-outline btn-secondary btn-xs">Edit</button>
                    <button @click="printCharacter(character)" class="btn btn-outline btn-secondary btn-xs">Print</button>
                    <button @click="levelUpCharacter(character)" class="btn btn-outline btn-secondary btn-xs">Level Up</button>
                    <button @click="deleteCharacter(character)" class="btn btn-outline btn-danger btn-xs">Delete</button>
                  </div>
                  <select class="campaign-select" @change="assignToCampaign(character.id, $event)">
                    <option value="">Add to Campaign...</option>
                    <option v-for="campaign in campaignStore.campaigns" :key="campaign.id" :value="campaign.id">
                      {{ campaign.name }}
                    </option>
                  </select>
                </div>
              </template>
            </CharacterCard>
          </div>
        </div>

        <!-- Campaign Characters -->
        <div v-for="(chars, campaignId) in charactersByCampaign" :key="campaignId" class="character-section">
          <h2 class="section-title">{{ getCampaignName(String(campaignId)) }}</h2>
          <div class="character-grid">
            <CharacterCard
              v-for="character in chars"
              :key="character.id"
              :character="character"
              @click="viewCharacter"
            >
              <template #actions>
                <div class="action-buttons">
                  <button @click="editCharacter(character)" class="btn btn-outline btn-secondary btn-xs">Edit</button>
                  <button @click="printCharacter(character)" class="btn btn-outline btn-secondary btn-xs">Print</button>
                  <button @click="levelUpCharacter(character)" class="btn btn-outline btn-secondary btn-xs">Level Up</button>
                  <button @click="deleteCharacter(character)" class="btn btn-outline btn-danger btn-xs">Delete</button>
                </div>
              </template>
            </CharacterCard>
          </div>
        </div>
      </div>
    </div>

    <!-- Character Creation Wizard -->
    <CharacterCreationWizard
      :visible="showWizard"
      :pc-only="true"
      @close="handleWizardClose"
      @created="handleCharacterCreated"
    />

    <!-- Character Print Dialog -->
    <CharacterPrintDialog
      v-if="selectedCharacterForPrint"
      :visible="showPrintDialog"
      :character-id="selectedCharacterForPrint.id"
      :character-name="selectedCharacterForPrint.name"
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
        <strong>{{ characterToDelete?.name }}</strong>?
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
import MainLayout from '@/shared/components/layout/MainLayout.vue'
import CharacterCreationWizard from '../components/CharacterCreationWizard.vue'
import LevelUpDialog from '../components/levelup/LevelUpDialog.vue'
import { CharacterPrintDialog } from '@/components/print'
import { CharacterCard } from '@/components/characters'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import { useCharacterStore } from '@/stores/characters'
import { useCampaignStore } from '@/stores/campaigns'
import type { Character } from '@/types/character'

const router = useRouter()
const characterStore = useCharacterStore()
const campaignStore = useCampaignStore()

onMounted(async () => {
  // Load campaigns first, then fetch all PCs across all campaigns
  await campaignStore.fetchCampaigns()
  const campaignIds = campaignStore.campaigns.map(c => c.id)
  await characterStore.fetchAllPcs(campaignIds)
})

// Characters are already PCs only (fetched via fetchAllPcs)
// NPCs are managed from the campaign dashboard's NPC tab
const characters = computed(() => characterStore.characters)

// Sort characters: PCs first, then NPCs, then alphabetically
const sortCharacters = (chars: Character[]) => {
  return [...chars].sort((a, b) => {
    // PCs (is_npc = 0) come before NPCs (is_npc = 1)
    if (a.is_npc !== b.is_npc) {
      return a.is_npc - b.is_npc
    }
    // Then sort alphabetically by name
    return a.name.localeCompare(b.name)
  })
}

const unassignedCharacters = computed(() =>
  sortCharacters(characters.value.filter(c => c.campaign_id === null))
)

const charactersByCampaign = computed(() => {
  const grouped: Record<string, Character[]> = {}

  characters.value
    .filter(c => c.campaign_id !== null && c.campaign_id !== undefined)
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

const getCampaignName = (campaignId: string): string => {
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

const reloadAllPcs = async () => {
  const campaignIds = campaignStore.campaigns.map(c => c.id)
  await characterStore.fetchAllPcs(campaignIds)
}

const handleCharacterCreated = async () => {
  showWizard.value = false
  await reloadAllPcs()
}

const viewCharacter = (character: Character) => {
  router.push(`/characters/${character.id}`)
}

const assignToCampaign = async (characterId: string, event: Event) => {
  const select = event.target as HTMLSelectElement
  const campaignId = select.value

  if (!campaignId) return

  try {
    await invoke('assign_character_to_campaign', {
      characterId,
      campaignId
    })
    await reloadAllPcs()
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
const selectedCharacterData = ref<Character | null>(null)

const levelUpCharacter = async (character: Character) => {
  // Level-up uses the character data directly (dialog shows unavailable message)
  selectedCharacterForLevelUp.value = character
  selectedCharacterData.value = character
  showLevelUpDialog.value = true
}

const closeLevelUpDialog = () => {
  showLevelUpDialog.value = false
  selectedCharacterForLevelUp.value = null
  selectedCharacterData.value = null
}

const handleLevelUpCompleted = async () => {
  closeLevelUpDialog()
  await reloadAllPcs()
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
    await reloadAllPcs()
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

.error-message {
  padding: var(--spacing-md);
  background-color: var(--color-error) / 0.1;
  border: 1px solid var(--color-error) / 0.2;
  border-radius: var(--radius-md);
  color: var(--color-error);
}

/* Extended actions for CharacterCard slot */
.extended-actions {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.action-buttons {
  display: flex;
  gap: var(--spacing-xs);
}

.action-buttons .btn {
  flex: 1;
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
