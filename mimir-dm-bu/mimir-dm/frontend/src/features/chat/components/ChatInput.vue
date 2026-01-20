<template>
  <div class="chat-input-container">
    <div class="chat-input-wrapper">
      <textarea
        ref="textareaRef"
        v-model="message"
        @keydown="handleKeyDown"
        @input="adjustHeight"
        :disabled="disabled"
        :placeholder="textareaPlaceholder"
        class="form-textarea"
        :style="{ height: textareaHeight }"
      />
      <div class="button-group">
        <button
          v-if="isEditing"
          @click="cancelEdit"
          class="btn btn-secondary chat-cancel-btn"
        >
          Cancel
        </button>
        <button
          @click="sendMessage"
          :disabled="!canSend"
          class="btn btn-primary chat-send-btn"
        >
          <span v-if="!isLoading && !isCancelling">{{ isEditing ? 'Update' : 'Send' }}</span>
          <span v-else-if="isCancelling">Cancelling...</span>
          <span v-else>...</span>
        </button>
      </div>
    </div>
    <div v-if="error" class="form-error">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'

const props = defineProps<{
  disabled?: boolean
  isLoading?: boolean
  isCancelling?: boolean
  error?: string | null
  editingMessageId?: string | null
  editingContent?: string
}>()

const emit = defineEmits<{
  send: [message: string]
  update: [messageId: string, newContent: string]
  cancelEdit: []
}>()

// Computed for edit mode
const isEditing = computed(() => !!props.editingMessageId)

// State
const message = ref('')
const textareaRef = ref<HTMLTextAreaElement>()
const textareaHeight = ref('60px')

// Computed
const canSend = computed(() => {
  return message.value.trim().length > 0 && !props.disabled && !props.isLoading && !props.isCancelling
})

const textareaPlaceholder = computed(() => {
  if (props.isCancelling) {
    return 'Cancelling request... Press Escape to cancel'
  } else if (props.isLoading) {
    return 'AI is thinking... Press Escape to cancel'
  } else {
    return 'Type your message... (Ctrl+Enter to send, Escape to cancel)'
  }
})

// Watch for edit mode changes to populate textarea
watch(() => props.editingContent, (newContent) => {
  if (newContent !== undefined) {
    message.value = newContent
    nextTick(() => {
      adjustHeight()
      textareaRef.value?.focus()
    })
  }
})

// Methods
const sendMessage = () => {
  if (canSend.value) {
    if (isEditing.value && props.editingMessageId) {
      emit('update', props.editingMessageId, message.value)
    } else {
      emit('send', message.value)
    }
    message.value = ''
    nextTick(() => {
      adjustHeight()
    })
  }
}

const cancelEdit = () => {
  message.value = ''
  nextTick(() => {
    adjustHeight()
  })
  emit('cancelEdit')
}

const handleKeyDown = (event: KeyboardEvent) => {
  // Ctrl+Enter or Cmd+Enter to send
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    sendMessage()
  }
  // Escape to cancel editing
  if (event.key === 'Escape' && isEditing.value) {
    event.preventDefault()
    cancelEdit()
  }
}

const adjustHeight = () => {
  if (textareaRef.value) {
    // Reset height to auto to get the correct scrollHeight
    textareaRef.value.style.height = 'auto'
    const scrollHeight = textareaRef.value.scrollHeight
    // Set minimum height of 60px and maximum of 200px
    const height = Math.min(Math.max(scrollHeight, 60), 200)
    textareaHeight.value = `${height}px`
  }
}

onMounted(() => {
  adjustHeight()
})
</script>

<!-- All styling now handled by consolidated CSS classes -->