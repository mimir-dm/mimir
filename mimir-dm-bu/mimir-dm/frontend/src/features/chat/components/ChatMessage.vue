<template>
  <div
    class="chat-message"
    :class="{
      'user-message': message.role === 'user',
      'assistant-message': message.role === 'assistant',
      'system-message': message.role === 'system',
      'tool-message': message.role === 'tool',
      'message-animate': isRecentMessage
    }"
    :style="animationStyle"
  >
    <!-- Tool confirmation UI for system messages -->
    <ToolConfirmation
      v-if="toolConfirmation"
      :confirmation="toolConfirmation"
      @confirm="handleConfirm"
      @reject="handleReject"
    />
    
    <!-- Tool result display -->
    <ToolResultMessage
      v-else-if="message.role === 'tool'"
      :tool-name="message.toolName || 'Unknown Tool'"
      :content="message.content"
      :success="message.success !== false"
      :iteration="message.iteration"
    />
    
    <!-- Regular message bubble -->
    <div v-else class="message-bubble">
      <!-- Action buttons for user messages -->
      <div v-if="message.role === 'user'" class="message-actions">
        <button @click="handleEdit" class="action-btn" title="Edit message">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
          </svg>
        </button>
        <button @click="handleDelete" class="action-btn action-btn-danger" title="Delete message">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
        </button>
      </div>

      <!-- Thinking blocks (if any) -->
      <div v-if="thinkingBlocks.length > 0" class="thinking-blocks">
        <div
          v-for="(block, index) in thinkingBlocks"
          :key="index"
          class="thinking-block"
          :class="{ expanded: expandedThinking[index] }"
        >
          <button
            @click="toggleThinking(index)"
            class="thinking-toggle"
          >
            <span class="toggle-icon">{{ expandedThinking[index] ? '▼' : '▶' }}</span>
            <span class="thinking-label">thinking{{ block.preview }}</span>
          </button>
          <div v-if="expandedThinking[index]" class="thinking-content">
            {{ block.content }}
          </div>
        </div>
      </div>
      
      <!-- Main message content -->
      <div class="message-content" v-html="formattedMainContent"></div>
      
      <div class="message-meta">
        <span class="timestamp">{{ formatTime(message.timestamp) }}</span>
        <span v-if="message.tokenUsage" class="token-count">
          {{ message.tokenUsage.total }} tokens
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ChatMessage } from '@/stores/chat'
import { useChatStore } from '@/stores/chat'
import { marked } from 'marked'
import ToolConfirmation from '@/components/ToolConfirmation.vue'
import ToolResultMessage from '@/components/ToolResultMessage.vue'

const props = defineProps<{
  message: ChatMessage
  animationIndex?: number
}>()

const chatStore = useChatStore()

// Check if message is recent (for animation) - only animate last 5 messages
const isRecentMessage = computed(() => {
  const totalMessages = chatStore.messages.length
  const index = props.animationIndex ?? 0
  return totalMessages - index <= 5
})

// Animation delay based on position (staggered effect)
const animationStyle = computed(() => {
  if (!isRecentMessage.value) return {}
  const totalMessages = chatStore.messages.length
  const index = props.animationIndex ?? 0
  const relativeIndex = index - (totalMessages - 5)
  const delay = Math.max(0, relativeIndex) * 0.08
  return {
    '--message-delay': `${delay}s`
  }
})

// Track which thinking blocks are expanded
const expandedThinking = ref<Record<number, boolean>>({})

// Check if this is a tool confirmation message
const toolConfirmation = computed(() => {
  if (props.message.role === 'system' && props.message.content.startsWith('TOOL_CONFIRMATION:')) {
    return chatStore.getConfirmationForMessage(props.message.content)
  }
  return null
})

// Parse thinking blocks from content
const parsedContent = computed(() => {
  if (props.message.role !== 'assistant') {
    return {
      thinkingBlocks: [],
      mainContent: props.message.content
    }
  }
  
  const content = props.message.content
  const thinkingBlocks: Array<{ content: string; preview: string }> = []
  
  // Match <thinking>, <think>, or variations with closing tags
  const thinkingRegex = /<think(?:ing)?>([\s\S]*?)<\/think(?:ing)?>/gi
  let lastIndex = 0
  let mainContent = ''
  
  let match
  while ((match = thinkingRegex.exec(content)) !== null) {
    // Add content before the thinking block
    mainContent += content.slice(lastIndex, match.index)
    
    // Extract thinking content
    const thinkingContent = match[1].trim()
    const preview = thinkingContent.length > 50 
      ? ': ' + thinkingContent.slice(0, 50) + '...'
      : ': ' + thinkingContent
    
    thinkingBlocks.push({
      content: thinkingContent,
      preview
    })
    
    lastIndex = thinkingRegex.lastIndex
  }
  
  // Add remaining content after last thinking block
  mainContent += content.slice(lastIndex)
  
  return {
    thinkingBlocks,
    mainContent: mainContent.trim()
  }
})

const thinkingBlocks = computed(() => parsedContent.value.thinkingBlocks)

// Format the main content (without thinking blocks)
const formattedMainContent = computed(() => {
  const mainContent = parsedContent.value.mainContent
  
  if (props.message.role === 'assistant' && mainContent) {
    // Parse markdown for assistant messages
    return marked(mainContent, {
      breaks: true,
      gfm: true
    })
  }
  // Plain text for user messages (with line breaks preserved)
  return mainContent.replace(/\n/g, '<br>')
})

const toggleThinking = (index: number) => {
  expandedThinking.value[index] = !expandedThinking.value[index]
}

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp)
  return date.toLocaleTimeString([], { 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}

// Handle tool confirmation actions
const handleConfirm = async (confirmationId: string) => {
  await chatStore.confirmToolAction(confirmationId)
}

const handleReject = async (confirmationId: string) => {
  await chatStore.rejectToolAction(confirmationId)
}

// Handle message edit/delete
const handleEdit = () => {
  chatStore.startEditing(props.message.id)
}

const handleDelete = async () => {
  await chatStore.deleteMessage(props.message.id)
}
</script>

<style scoped>
.chat-message {
  @apply flex mb-4;
}

/* Staggered fade-in animation for recent messages */
.message-animate {
  animation: message-fade-in 0.35s ease-out backwards;
  animation-delay: var(--message-delay, 0s);
}

@keyframes message-fade-in {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.user-message {
  @apply justify-end;
}

.assistant-message {
  @apply justify-start;
}

.system-message {
  @apply justify-center;
  width: 100%;
}

.tool-message {
  @apply justify-start;
  width: 100%;
}

.message-bubble {
  @apply max-w-[70%] rounded-lg px-4 py-2;
}

.user-message .message-bubble {
  @apply text-white relative;
  background-color: var(--color-primary-500);
}

.message-actions {
  @apply absolute flex gap-1 opacity-0 transition-opacity;
  top: 4px;
  right: 4px;
}

.user-message:hover .message-actions {
  @apply opacity-100;
}

.action-btn {
  @apply p-1 rounded transition-colors;
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.action-btn-danger:hover {
  background: rgba(239, 68, 68, 0.8);
}

.assistant-message .message-bubble {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
}

.system-message .message-bubble {
  @apply italic text-sm;
  background-color: var(--color-surface);
  color: var(--color-text-secondary);
}

.message-content {
  @apply text-sm leading-relaxed;
}

/* Markdown content styling */
.message-content :deep(pre) {
  @apply rounded p-2 my-2 overflow-x-auto;
  background-color: var(--color-background);
}

.message-content :deep(code) {
  @apply px-1 py-0.5 rounded text-xs;
  background-color: var(--color-background);
}

.message-content :deep(pre code) {
  @apply bg-transparent px-0 py-0;
}

.message-content :deep(ul),
.message-content :deep(ol) {
  @apply ml-4 my-2;
}

.message-content :deep(li) {
  @apply my-1;
}

.message-content :deep(h1),
.message-content :deep(h2),
.message-content :deep(h3) {
  @apply font-semibold mt-3 mb-2;
}

.message-meta {
  @apply flex justify-between items-center mt-1 text-xs opacity-60;
}

.timestamp {
  @apply mr-2;
}

.token-count {
  @apply text-xs;
}

/* Thinking blocks styling */
.thinking-blocks {
  @apply mb-2;
}

.thinking-block {
  @apply mb-1;
}

.thinking-toggle {
  @apply w-full text-left px-3 py-2 rounded text-xs transition-colors;
  @apply flex items-center gap-2;
  background: var(--gradient-chromatic-subtle);
  border: 1px solid var(--color-chromatic-border);
  color: var(--color-text);
  position: relative;
  overflow: hidden;
}

.thinking-toggle::before {
  content: '';
  position: absolute;
  top: -100%;
  left: -100%;
  width: 200%;
  height: 200%;
  background: linear-gradient(135deg, 
    transparent 30%, 
    rgba(255, 255, 255, 0.1) 50%, 
    transparent 70%);
  animation: shimmer 4s infinite linear;
  pointer-events: none;
}

.thinking-toggle:hover {
  border-color: var(--color-primary-400);
  background: var(--gradient-chromatic);
  opacity: 0.9;
}

.toggle-icon {
  @apply text-xs;
  width: 12px;
  display: inline-block;
  position: relative;
  z-index: 1;
}

.thinking-label {
  @apply italic;
  position: relative;
  z-index: 1;
}

.thinking-content {
  @apply mt-2 p-3 rounded text-xs whitespace-pre-wrap;
  background: var(--gradient-chromatic-subtle);
  border: 1px solid var(--color-chromatic-border);
  color: var(--color-text);
  max-height: 200px;
  overflow-y: auto;
  position: relative;
}

.thinking-content::-webkit-scrollbar {
  width: 6px;
}

.thinking-content::-webkit-scrollbar-thumb {
  background-color: var(--color-surface-variant);
  border-radius: 3px;
}

.thinking-content::-webkit-scrollbar-thumb:hover {
  background-color: var(--color-border-hover);
}

@keyframes shimmer {
  0% {
    transform: translate(-100%, -100%);
  }
  100% {
    transform: translate(100%, 100%);
  }
}
</style>