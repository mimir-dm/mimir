<template>
  <div ref="historyContainer" class="chat-history">
    <!-- Session Info Bar -->
    <div v-if="currentSessionId && !isInfoBarCollapsed" class="session-info-bar">
      <div class="session-info-content">
        <span class="session-label">Session:</span>
        <code class="session-id">{{ currentSessionId }}</code>
        <button 
          @click="copySessionId" 
          class="btn-copy"
          :class="{ 'btn-copy--copied': copyFeedback }"
          title="Copy session ID"
        >
          <svg v-if="!copyFeedback" width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
          </svg>
          <svg v-else width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
          </svg>
        </button>
        <button @click="openChatLogs" class="btn-logs" title="Open chat logs">
          <svg width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
          </svg>
          Logs
        </button>
      </div>
      <button @click="toggleInfoBar" class="btn-collapse" title="Collapse session info">
        <svg width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
        </svg>
      </button>
    </div>
    
    <!-- Collapsed Info Bar -->
    <div v-else-if="currentSessionId && isInfoBarCollapsed" class="session-info-bar session-info-bar--collapsed">
      <button @click="toggleInfoBar" class="btn-expand" title="Show session info">
        <svg width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
        </svg>
      </button>
      <span class="session-id-short">{{ currentSessionId?.slice(0, 8) }}...</span>
    </div>
    
    <div class="messages-container">
      <EmptyState
        v-if="messages.length === 0"
        variant="generic"
        title="No messages yet"
        description="Start a conversation by typing a message below"
      />
      <div v-else class="messages-content">
        <ChatMessage
          v-for="(message, index) in messages"
          :key="message.id"
          :message="message"
          :animation-index="index"
        />
        <div v-if="isLoading" class="loading-indicator">
          <div class="typing-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ChatMessage as ChatMessageType } from '@/stores/chat'
import ChatMessage from './ChatMessage.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'

const props = defineProps<{
  messages: ChatMessageType[]
  isLoading?: boolean
  currentSessionId?: string | null
}>()

// Refs
const historyContainer = ref<HTMLDivElement>()
const isInfoBarCollapsed = ref(false)
const copyFeedback = ref(false)

// Auto-scroll to bottom when new messages arrive
watch(
  () => props.messages.length,
  () => {
    nextTick(() => {
      if (historyContainer.value) {
        historyContainer.value.scrollTop = historyContainer.value.scrollHeight
      }
    })
  }
)

// Also scroll when loading state changes
watch(
  () => props.isLoading,
  () => {
    if (props.isLoading) {
      nextTick(() => {
        if (historyContainer.value) {
          historyContainer.value.scrollTop = historyContainer.value.scrollHeight
        }
      })
    }
  }
)

// Load collapsed state from localStorage
const storageKey = 'chat-session-info-collapsed'
isInfoBarCollapsed.value = localStorage.getItem(storageKey) === 'true'

// Session ID functionality
const copySessionId = async () => {
  if (!props.currentSessionId) return
  
  try {
    await navigator.clipboard.writeText(props.currentSessionId)
    copyFeedback.value = true
    setTimeout(() => {
      copyFeedback.value = false
    }, 2000)
  } catch (error) {
    console.error('Failed to copy session ID:', error)
  }
}

const openChatLogs = async () => {
  if (!props.currentSessionId) return
  
  try {
    // Use the existing log viewer window command with the session's log filename
    const filename = `${props.currentSessionId}.log`
    await invoke('open_log_viewer_window', { fileName: filename })
  } catch (error) {
    console.error('Failed to open log viewer:', error)
  }
}

const toggleInfoBar = () => {
  isInfoBarCollapsed.value = !isInfoBarCollapsed.value
  localStorage.setItem(storageKey, isInfoBarCollapsed.value.toString())
}
</script>

<style scoped>
.chat-history {
  @apply flex-1 flex flex-col overflow-hidden;
  background-color: var(--color-background);
}

.messages-container {
  @apply flex-1 overflow-y-auto p-4;
  scrollbar-width: thin;
}

.messages-container::-webkit-scrollbar {
  width: 8px;
}

.messages-container::-webkit-scrollbar-track {
  background-color: var(--color-background);
}

.messages-container::-webkit-scrollbar-thumb {
  background-color: var(--color-surface-variant);
  border-radius: 4px;
}

.messages-container::-webkit-scrollbar-thumb:hover {
  background-color: var(--color-border-hover);
}

/* Session Info Bar Styles */
.session-info-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background-color: var(--color-surface-variant);
  border-bottom: 1px solid var(--color-border);
  font-size: 12px;
  color: var(--color-text-secondary);
}

.session-info-bar--collapsed {
  padding: 4px 8px;
}

.session-info-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.session-label {
  color: var(--color-text-secondary);
  font-weight: 500;
}

.session-id {
  font-family: var(--font-mono);
  font-size: 11px;
  background-color: var(--color-surface);
  color: var(--color-text);
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border);
}

.session-id-short {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-secondary);
}

.btn-copy, .btn-logs, .btn-collapse, .btn-expand {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  font-size: 11px;
  border-radius: var(--radius-sm);
  transition: all 0.2s ease;
  color: var(--color-text-secondary);
  background: transparent;
  border: 1px solid transparent;
  cursor: pointer;
}

.btn-copy:hover, .btn-logs:hover, .btn-collapse:hover, .btn-expand:hover {
  background-color: var(--color-surface);
  border-color: var(--color-border);
}

.btn-copy--copied {
  color: var(--color-success);
}

.btn-logs {
  color: var(--color-primary);
}

.btn-collapse, .btn-expand {
  padding: 4px;
}

.messages-content {
  @apply min-h-full flex flex-col justify-end;
}

.loading-indicator {
  @apply flex justify-start mb-4;
}

.typing-dots {
  @apply bg-gray-700 rounded-lg px-4 py-3 inline-flex items-center gap-1;
}

.typing-dots span {
  @apply w-2 h-2 bg-gray-400 rounded-full;
  animation: typing 1.4s infinite;
}

.typing-dots span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-dots span:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes typing {
  0%, 60%, 100% {
    transform: translateY(0);
    opacity: 0.5;
  }
  30% {
    transform: translateY(-10px);
    opacity: 1;
  }
}
</style>