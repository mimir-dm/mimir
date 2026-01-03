<template>
  <AppModal
    :visible="visible"
    title="Manage Players"
    size="lg"
    @close="closeModal"
  >
    <!-- Loading State -->
    <div v-if="playerStore.loading" class="loading-message">
      Loading players...
    </div>

    <!-- Error State -->
    <div v-else-if="playerStore.error" class="error-message">
      {{ playerStore.error }}
    </div>

    <!-- Empty State -->
    <EmptyState
      v-else-if="players.length === 0"
      variant="users"
      title="No players yet"
      description="Add your first player to get started"
    >
      <template #action>
        <button @click="showAddPlayerDialog" class="btn btn-primary">
          Add Player
        </button>
      </template>
    </EmptyState>

    <!-- Player List -->
    <div v-else class="player-container">
      <div class="player-header">
        <h3>Players ({{ players.length }})</h3>
        <button @click="showAddPlayerDialog" class="btn btn-primary">
          Add Player
        </button>
      </div>

      <div class="player-list">
        <div
          v-for="player in players"
          :key="player.id"
          class="player-item"
        >
          <div class="player-info">
            <div class="player-name">{{ player.name }}</div>
            <div class="player-meta">
              <span v-if="player.email" class="player-email">
                {{ player.email }}
              </span>
              <span class="player-date">
                Added: {{ formatDate(player.created_at) }}
              </span>
            </div>
            <div v-if="player.notes" class="player-notes">
              {{ player.notes }}
            </div>
          </div>
          <div class="player-actions">
            <button
              @click="editPlayer(player)"
              class="edit-button"
              title="Edit player"
            >
              Edit
            </button>
            <button
              @click="confirmDeletePlayer(player)"
              class="delete-button"
              title="Delete player"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>
  </AppModal>

  <!-- Add/Edit Player Dialog -->
  <AppModal
    :visible="showPlayerDialog"
    :title="editingPlayer ? 'Edit Player' : 'Add Player'"
    size="sm"
    :stack-index="1"
    @close="closePlayerDialog"
  >
    <form @submit.prevent="savePlayer" class="player-form">
      <div class="form-group">
        <label for="player-name" class="form-label">
          Name <span class="required">*</span>
        </label>
        <input
          id="player-name"
          v-model="playerForm.name"
          type="text"
          class="form-input"
          placeholder="Enter player name"
          required
        />
      </div>

      <div class="form-group">
        <label for="player-email" class="form-label">Email</label>
        <input
          id="player-email"
          v-model="playerForm.email"
          type="email"
          class="form-input"
          placeholder="player@example.com"
        />
      </div>

      <div class="form-group">
        <label for="player-notes" class="form-label">Notes</label>
        <textarea
          id="player-notes"
          v-model="playerForm.notes"
          class="form-textarea"
          placeholder="Additional notes about the player"
          rows="3"
        ></textarea>
      </div>

      <div v-if="formError" class="form-error">
        {{ formError }}
      </div>
    </form>

    <template #footer>
      <button
        type="button"
        @click="closePlayerDialog"
        class="btn btn-secondary"
      >
        Cancel
      </button>
      <button
        @click="savePlayer"
        class="btn btn-primary"
        :disabled="!playerForm.name.trim() || saving"
      >
        {{ saving ? 'Saving...' : editingPlayer ? 'Save Changes' : 'Add Player' }}
      </button>
    </template>
  </AppModal>

  <!-- Delete Confirmation Dialog -->
  <AppModal
    :visible="showDeleteDialog"
    title="Confirm Delete"
    size="sm"
    :stack-index="1"
    @close="closeDeleteDialog"
  >
    <p>
      Are you sure you want to delete
      <strong>{{ playerToDelete?.name }}</strong>?
    </p>
    <p class="warning-text">
      This will also delete all characters associated with this player. This
      action cannot be undone.
    </p>

    <template #footer>
      <button
        type="button"
        @click="closeDeleteDialog"
        class="btn btn-secondary"
      >
        Cancel
      </button>
      <button
        @click="deletePlayer"
        class="btn btn-danger"
        :disabled="deleting"
      >
        {{ deleting ? 'Deleting...' : 'Delete Player' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { usePlayerStore } from '../stores/players'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import type { Player } from '../types/character'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

// Store
const playerStore = usePlayerStore()

// State
const showPlayerDialog = ref(false)
const showDeleteDialog = ref(false)
const editingPlayer = ref<Player | null>(null)
const playerToDelete = ref<Player | null>(null)
const saving = ref(false)
const deleting = ref(false)
const formError = ref<string | null>(null)

// Form data
const playerForm = ref({
  name: '',
  email: '',
  notes: ''
})

// Computed
const players = computed(() => playerStore.players)

// Methods
const closeModal = () => {
  emit('close')
}

const showAddPlayerDialog = () => {
  editingPlayer.value = null
  playerForm.value = {
    name: '',
    email: '',
    notes: ''
  }
  formError.value = null
  showPlayerDialog.value = true
}

const editPlayer = (player: Player) => {
  editingPlayer.value = player
  playerForm.value = {
    name: player.name,
    email: player.email || '',
    notes: player.notes || ''
  }
  formError.value = null
  showPlayerDialog.value = true
}

const closePlayerDialog = () => {
  showPlayerDialog.value = false
  editingPlayer.value = null
  formError.value = null
}

const savePlayer = async () => {
  if (!playerForm.value.name.trim()) {
    formError.value = 'Player name is required'
    return
  }

  saving.value = true
  formError.value = null

  try {
    if (editingPlayer.value) {
      // Update existing player
      await playerStore.updatePlayer(editingPlayer.value.id, {
        name: playerForm.value.name.trim(),
        email: playerForm.value.email.trim() || null,
        notes: playerForm.value.notes.trim() || null
      })
    } else {
      // Create new player
      await playerStore.createPlayer(
        playerForm.value.name.trim(),
        playerForm.value.email.trim() || undefined,
        playerForm.value.notes.trim() || undefined
      )
    }

    closePlayerDialog()
  } catch (error) {
    formError.value =
      error instanceof Error ? error.message : 'Failed to save player'
  } finally {
    saving.value = false
  }
}

const confirmDeletePlayer = (player: Player) => {
  playerToDelete.value = player
  showDeleteDialog.value = true
}

const closeDeleteDialog = () => {
  showDeleteDialog.value = false
  playerToDelete.value = null
}

const deletePlayer = async () => {
  if (!playerToDelete.value) return

  deleting.value = true

  try {
    await playerStore.deletePlayer(playerToDelete.value.id)
    closeDeleteDialog()
  } catch (error) {
    console.error('Failed to delete player:', error)
    // Keep dialog open on error so user can see what happened
  } finally {
    deleting.value = false
  }
}

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}

// Load players when component becomes visible
watch(
  () => props.visible,
  async (isVisible) => {
    if (isVisible) {
      try {
        await playerStore.fetchPlayers()
      } catch (error) {
        console.error('Failed to load players:', error)
      }
    }
  },
  { immediate: true }
)

// Initial load
onMounted(async () => {
  if (props.visible) {
    try {
      await playerStore.fetchPlayers()
    } catch (error) {
      console.error('Failed to load players:', error)
    }
  }
})
</script>

<style scoped>
/* Domain-specific styles */

/* States */
.loading-message,
.error-message {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
}

.error-message {
  color: var(--error);
}

/* Player Container */
.player-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.player-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.player-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text);
}

/* Player List */
.player-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.player-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  transition: border-color 0.2s;
}

.player-item:hover {
  border-color: var(--primary);
}

.player-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.player-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.player-meta {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  font-size: 13px;
  color: var(--text-secondary);
}

.player-email {
  color: var(--primary);
}

.player-notes {
  font-size: 14px;
  color: var(--text-secondary);
  margin-top: 4px;
  font-style: italic;
}

.player-actions {
  display: flex;
  gap: 8px;
  margin-left: 16px;
}

/* Form */
.form-group {
  margin-bottom: 20px;
}

.form-label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
}

.required {
  color: var(--error);
}

.form-input,
.form-textarea {
  width: 100%;
  padding: 10px 12px;
  font-size: 14px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface);
  color: var(--text);
  font-family: inherit;
  transition: border-color 0.2s;
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--primary);
}

.form-textarea {
  resize: vertical;
  min-height: 60px;
}

.form-error {
  margin-top: 12px;
  padding: 10px;
  background: rgba(220, 38, 38, 0.1);
  border: 1px solid var(--error);
  border-radius: 4px;
  color: var(--error);
  font-size: 14px;
}

.warning-text {
  margin-top: 12px;
  color: var(--warning);
  font-size: 14px;
}

/* Action buttons in player list */
.edit-button {
  background: var(--primary);
  color: white;
  font-size: 13px;
  padding: 6px 12px;
}

.edit-button:hover {
  background: var(--primary-dark);
}

.delete-button {
  background: transparent;
  color: var(--error);
  border: 1px solid var(--error);
  font-size: 13px;
  padding: 6px 12px;
}

.delete-button:hover {
  background: var(--error);
  color: white;
}
</style>
