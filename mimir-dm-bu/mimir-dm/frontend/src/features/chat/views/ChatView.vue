<template>
  <div class="chat-view" :class="currentTheme">
    <!-- Chat Sidebar -->
    <ChatSidebar />
    
    <!-- Main Content Area -->
    <div class="main-content">
      <!-- Context Panel (collapsible at top) -->
      <ContextPanel :start-collapsed="true" />
      
      <!-- Main Chat Area -->
      <div class="chat-container">
        <!-- Chat History -->
        <ChatHistory
          :messages="messages"
          :is-loading="isLoading"
          :current-session-id="currentSessionId"
        />
        
        <!-- Token Usage Bar -->
        <TokenUsage
          :last-message-tokens="lastMessageTokens"
          :conversation-tokens="conversationTokens"
          :max-context="maxContextTokens"
        />
        
        <!-- Chat Input -->
        <ChatInput
          :disabled="!isReady"
          :is-loading="isLoading"
          :is-cancelling="isCancelling"
          :error="error"
          :editing-message-id="editingMessageId"
          :editing-content="editingContent"
          @send="handleSendMessage"
          @update="handleUpdateMessage"
          @cancel-edit="handleCancelEdit"
        />
      </div>
      
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useChatStore } from '@/stores/chat'
import { useSharedContextStore } from '@/stores/sharedContext'
import { useThemeStore } from '@/stores/theme'
import ChatSidebar from '../components/ChatSidebar.vue'
import ContextPanel from '../components/ContextPanel.vue'
import ChatHistory from '../components/ChatHistory.vue'
import ChatInput from '../components/ChatInput.vue'
import TokenUsage from '../components/TokenUsage.vue'

// Stores
const chatStore = useChatStore()
const contextStore = useSharedContextStore()
const themeStore = useThemeStore()

// State
const isReady = ref(false)

// Computed from chat store
const messages = computed(() => chatStore.messages)
const isLoading = computed(() => chatStore.isLoading)
const isCancelling = computed(() => chatStore.isCancelling)
const error = computed(() => chatStore.error)
const conversationTokens = computed(() => chatStore.conversationTokens)
const maxContextTokens = computed(() => chatStore.modelInfo?.contextLength || 262144)
const currentSessionId = computed(() => chatStore.currentSessionId)
const editingMessageId = computed(() => chatStore.editingMessageId)

// Get the content of the message being edited
const editingContent = computed(() => {
  if (!editingMessageId.value) return undefined
  const msg = messages.value.find(m => m.id === editingMessageId.value)
  return msg?.content
})

// Calculate last message tokens
const lastMessageTokens = computed(() => {
  const lastMsg = chatStore.lastMessage
  return lastMsg?.tokenUsage?.total || 0
})

// Theme class
const currentTheme = computed(() => `theme-${themeStore.currentTheme}`)

// Methods
const handleSendMessage = async (content: string) => {
  await chatStore.sendMessage(content)
}

const handleUpdateMessage = async (messageId: string, newContent: string) => {
  // Edit the message and truncate subsequent messages
  await chatStore.editMessage(messageId, newContent)
  // Resend to LLM for new response
  await chatStore.resendConversation()
}

const handleCancelEdit = () => {
  chatStore.cancelEditing()
}

// Keyboard event handler for escape key cancellation
const handleKeyDown = async (event: KeyboardEvent) => {
  console.log('Key pressed:', event.key, 'isLoading:', chatStore.isLoading, 'isCancelling:', chatStore.isCancelling)
  if (event.key === 'Escape' && chatStore.isLoading && !chatStore.isCancelling) {
    console.log('Attempting to cancel message...')
    event.preventDefault()
    try {
      await chatStore.cancelMessage()
      console.log('Cancel request sent successfully')
    } catch (error) {
      console.error('Failed to cancel message:', error)
    }
  }
}


// Initialize on mount
onMounted(async () => {
  // Set window ID for this window
  (window as any).__TAURI_WINDOW_ID__ = 'chat'
  
  // Add global escape key handler
  document.addEventListener('keydown', handleKeyDown)
  
  // Initialize theme store first
  await themeStore.loadThemes()
  themeStore.applyTheme()
  await themeStore.initThemeSync()
  
  // Register window with context service
  await contextStore.registerWindow({
    id: 'chat',
    type: 'chat',
    title: 'Mimir Chat',
    focused: true
  })
  
  // Initialize stores
  await contextStore.loadFullContext()
  await chatStore.initialize()
  
  isReady.value = true
})

// Clean up on unmount
onUnmounted(() => {
  // Remove global escape key handler
  document.removeEventListener('keydown', handleKeyDown)
  
  themeStore.cleanup()
  contextStore.unregisterWindow('chat')
})
</script>

<style scoped>
.chat-view {
  @apply h-screen flex;
  background-color: var(--color-background);
  color: var(--color-text);
}

.main-content {
  @apply flex-1 flex flex-col overflow-hidden;
}


.chat-container {
  @apply flex-1 flex flex-col overflow-hidden;
}

</style>