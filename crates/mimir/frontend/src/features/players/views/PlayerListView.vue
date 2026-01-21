<template>
  <MainLayout>
    <div class="player-list-view">
      <div class="header">
        <h1 class="page-title">Players</h1>
        <button @click="showAddPlayerDialog" class="btn-icon" title="Add Player">
          +
        </button>
      </div>

      <div v-if="playerStore.loading" class="loading">
        Loading players...
      </div>

      <div v-else-if="playerStore.error" class="error-message">
        {{ playerStore.error }}
      </div>

      <EmptyState
        v-else-if="playerStore.players.length === 0"
        variant="users"
        title="No players yet"
        description="Add players to track who's in your campaigns"
      >
        <template #action>
          <button @click="showAddPlayerDialog" class="btn-primary">
            Add your first player
          </button>
        </template>
      </EmptyState>

      <div v-else class="player-grid">
        <div
          v-for="player in playerStore.players"
          :key="player.id"
          class="player-card"
        >
          <div class="card-actions">
            <button @click="editPlayer(player)" class="btn-action" title="Edit">Edit</button>
            <button @click="confirmDeletePlayer(player)" class="btn-action btn-action-danger" title="Delete">Delete</button>
          </div>
          <h3 class="player-name">{{ player.name }}</h3>
          <div class="player-meta">
            <span v-if="player.email" class="player-email">
              {{ player.email }}
            </span>
            <span class="player-date">
              Added {{ formatDate(player.created_at) }}
            </span>
          </div>
          <p v-if="player.notes" class="player-notes">{{ player.notes }}</p>
        </div>
      </div>
    </div>

    <!-- Add/Edit Player Dialog -->
    <AppModal
      :visible="showPlayerDialog"
      :title="editingPlayer ? 'Edit Player' : 'Add Player'"
      size="sm"
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
        <button type="button" @click="closePlayerDialog" class="btn btn-secondary">
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
        <button type="button" @click="closeDeleteDialog" class="btn btn-secondary">
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
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MainLayout from '../../../shared/components/layout/MainLayout.vue'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '../../../shared/components/ui/EmptyState.vue'
import { usePlayerStore } from '../../../stores/players'
import type { Player } from '../../../types/character'

const playerStore = usePlayerStore()

// Dialog state
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

onMounted(async () => {
  await playerStore.fetchPlayers()
})

const showAddPlayerDialog = () => {
  editingPlayer.value = null
  playerForm.value = { name: '', email: '', notes: '' }
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
      await playerStore.updatePlayer(editingPlayer.value.id, {
        name: playerForm.value.name.trim(),
        email: playerForm.value.email.trim() || null,
        notes: playerForm.value.notes.trim() || null
      })
    } else {
      await playerStore.createPlayer(
        playerForm.value.name.trim(),
        playerForm.value.email.trim() || undefined,
        playerForm.value.notes.trim() || undefined
      )
    }
    closePlayerDialog()
  } catch (error) {
    formError.value = error instanceof Error ? error.message : 'Failed to save player'
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
  } finally {
    deleting.value = false
  }
}

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return date.toLocaleDateString()
}
</script>

<style scoped>
.player-list-view {
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

.btn-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  font-weight: 300;
  background-color: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-icon:hover {
  background-color: var(--color-primary-600);
  transform: scale(1.05);
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

.player-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: var(--spacing-lg);
}

.player-card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  transition: all var(--transition-base);
}

.player-card:hover {
  border-color: var(--color-primary-500);
  box-shadow: var(--shadow-md);
}

.card-actions {
  display: flex;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
}

.btn-action {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
  font-weight: 500;
  background-color: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-action:hover {
  background-color: var(--color-primary-500);
  color: white;
  border-color: var(--color-primary-500);
}

.btn-action-danger:hover {
  background-color: var(--color-error-500);
  border-color: var(--color-error-500);
}

.player-name {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
}

.player-meta {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
}

.player-email {
  font-size: 0.875rem;
  color: var(--color-primary-500);
}

.player-date {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.player-notes {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  font-style: italic;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
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
  background-color: var(--color-error-50);
  border: 1px solid var(--color-error-200);
  border-radius: var(--radius-md);
  color: var(--color-error-700);
}

.theme-dark .error-message,
.theme-hyper .error-message {
  background-color: var(--color-error-900);
  border-color: var(--color-error-700);
  color: var(--color-error-300);
}

/* Form styles */
.form-group {
  margin-bottom: var(--spacing-lg);
}

.form-label {
  display: block;
  margin-bottom: var(--spacing-xs);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.required {
  color: var(--color-error-500);
}

.form-input,
.form-textarea {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  font-size: 1rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  transition: border-color var(--transition-fast);
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

.form-error {
  margin-top: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-50);
  border: 1px solid var(--color-error-200);
  border-radius: var(--radius-md);
  color: var(--color-error-700);
  font-size: 0.875rem;
}

.theme-dark .form-error,
.theme-hyper .form-error {
  background: var(--color-error-900);
  border-color: var(--color-error-700);
  color: var(--color-error-300);
}

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
