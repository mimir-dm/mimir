<template>
  <div class="chat-sidebar">
    <!-- Header -->
    <div class="chat-sidebar__header">
      <h3 class="chat-sidebar__title">Chat History</h3>
      <button
        @click="createNewChat"
        class="btn btn-primary btn-icon"
        title="New chat"
        :disabled="isCreating"
      >
        <svg width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
        </svg>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="sessionsLoading" class="loading-container">
      <div class="loading-spinner"></div>
      <span>Loading sessions...</span>
    </div>

    <!-- Sessions list -->
    <div v-else class="chat-sidebar__content">
      <ul class="chat-session-list">
        <li
          v-for="session in sessions"
          :key="session.id"
          @click="switchToSession(session.id)"
          class="chat-session-item"
          :class="{ 'chat-session-item--active': currentSessionId === session.id }"
        >
          <div class="session-content">
            <div class="chat-session-title">{{ session.title }}</div>
            <div class="chat-session-preview">{{ session.preview }}</div>
            <div class="chat-session-meta">
              <span class="session-date">{{ formatDate(session.updated_at) }}</span>
              <span class="session-count">{{ session.message_count }} messages</span>
            </div>
            <div class="session-id-row">
              <code class="session-id-badge" :title="session.id">{{ session.id.slice(0, 8) }}...</code>
              <div class="session-actions">
                <button 
                  @click.stop="copySessionId(session.id)"
                  class="session-action-btn copy-btn"
                  :class="{ 'copy-btn--copied': copiedSessionId === session.id }"
                  title="Copy session ID"
                >
                  <svg v-if="copiedSessionId !== session.id" width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                  </svg>
                  <svg v-else width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                  </svg>
                </button>
                <button 
                  @click.stop="openSessionLogs(session.id)"
                  class="session-action-btn logs-btn"
                  title="Open session logs"
                >
                  <svg width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <button
            @click.stop="deleteSessionHandler(session.id)"
            class="delete-button"
            title="Delete session"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </li>
      </ul>
    </div>

    <!-- Empty state -->
    <div v-if="!sessionsLoading && sessions.length === 0" class="empty-state">
      <p class="empty-message">No chat sessions yet</p>
      <button @click="createNewChat" class="btn btn-primary">
        Start your first chat
      </button>
    </div>
  </div>

  <!-- Delete Session Confirmation Modal -->
  <AppModal
    :visible="showDeleteModal"
    title="Delete Chat Session"
    size="sm"
    @close="cancelDelete"
  >
    <p>Are you sure you want to delete this chat session?</p>
    <p class="warning-text">This action cannot be undone. All messages in this session will be permanently deleted.</p>

    <div v-if="deleteError" class="error-message">
      {{ deleteError }}
    </div>

    <template #footer>
      <button @click="cancelDelete" class="btn btn-secondary">
        Cancel
      </button>
      <button @click="confirmDelete" class="btn btn-danger">
        Delete Session
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import { useChatStore } from '@/stores/chat'

const chatStore = useChatStore()

// Reactive state
const showDeleteModal = ref(false)
const sessionToDelete = ref<string | null>(null)
const deleteError = ref<string | null>(null)
const copiedSessionId = ref<string | null>(null)

// Computed
const sessions = computed(() => chatStore.sessions)
const sessionsLoading = computed(() => chatStore.sessionsLoading)
const currentSessionId = computed(() => chatStore.currentSessionId)
const isCreating = computed(() => chatStore.isLoading)

// Methods
const switchToSession = async (sessionId: string) => {
  await chatStore.switchToSession(sessionId)
}

const createNewChat = async () => {
  await chatStore.createNewSession()
}

const deleteSessionHandler = (sessionId: string) => {
  sessionToDelete.value = sessionId
  deleteError.value = null
  showDeleteModal.value = true
}

const confirmDelete = async () => {
  if (!sessionToDelete.value) return

  deleteError.value = null
  try {
    await chatStore.deleteSession(sessionToDelete.value)
    showDeleteModal.value = false
    sessionToDelete.value = null
  } catch (error) {
    deleteError.value = 'Failed to delete session. Please try again.'
  }
}

const cancelDelete = () => {
  showDeleteModal.value = false
  sessionToDelete.value = null
  deleteError.value = null
}

const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000) // Convert from seconds to milliseconds
  const now = new Date()
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24))
  
  if (diffDays === 0) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } else if (diffDays === 1) {
    return 'Yesterday'
  } else if (diffDays < 7) {
    return `${diffDays} days ago`
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' })
  }
}

// Session ID functionality
const copySessionId = async (sessionId: string) => {
  try {
    await navigator.clipboard.writeText(sessionId)
    copiedSessionId.value = sessionId
    setTimeout(() => {
      copiedSessionId.value = null
    }, 2000)
  } catch (error) {
    console.error('Failed to copy session ID:', error)
  }
}

const openSessionLogs = async (sessionId: string) => {
  try {
    // Use the existing log viewer window command with the session's log filename
    const filename = `${sessionId}.log`
    await invoke('open_log_viewer_window', { fileName: filename })
  } catch (error) {
    console.error('Failed to open log viewer:', error)
  }
}
</script>

<style scoped>
/* Session ID Row Styles */
.session-id-row {
  @apply flex items-center justify-between mt-2 pt-2 border-t border-gray-200 dark:border-gray-700;
  opacity: 0.8;
}

.session-id-badge {
  @apply font-mono text-xs bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 px-2 py-1 rounded;
  font-size: 10px;
}

.session-actions {
  @apply flex items-center gap-1;
}

.session-action-btn {
  @apply p-1 rounded transition-all duration-200 opacity-70 hover:opacity-100;
  background: transparent;
  border: none;
  cursor: pointer;
}

.session-action-btn:hover {
  @apply bg-gray-100 dark:bg-gray-700;
}

.copy-btn {
  @apply text-gray-500 dark:text-gray-400;
}

.copy-btn--copied {
  @apply text-green-600 dark:text-green-400;
}

.logs-btn {
  @apply text-blue-600 dark:text-blue-400;
}

/* Show session actions only on hover for non-active items */
.chat-session-item:not(.chat-session-item--active) .session-actions {
  @apply opacity-0 transition-opacity duration-200;
}

.chat-session-item:not(.chat-session-item--active):hover .session-actions {
  @apply opacity-100;
}

/* Always show for active session */
.chat-session-item--active .session-actions {
  @apply opacity-100;
}
</style>

<!-- Main styling handled by consolidated CSS classes -->