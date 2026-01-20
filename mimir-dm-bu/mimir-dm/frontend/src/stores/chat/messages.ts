import { ref, computed, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  ChatMessage,
  ChatResponseWithUsage,
  IntermediateMessage,
  ToolResultMessage
} from './types'

// Re-export types for backwards compatibility
export type {
  ChatMessage,
  ChatResponseWithUsage,
  ActionDescription,
  ChangeDetail,
  FileEditDetail,
  FileWriteDetail,
  FileReadDetail,
  GenericDetail,
  LineEdit,
  DiffPreview,
  IntermediateMessage,
  ToolResultMessage
} from './types'

interface MessagesState {
  messages: Ref<ChatMessage[]>
  isLoading: Ref<boolean>
  isCancelling: Ref<boolean>
  error: Ref<string | null>
  editingMessageId: Ref<string | null>
}

interface MessagesComputed {
  lastMessage: ComputedRef<ChatMessage | null>
}

interface MessagesActions {
  initializeMessageListeners: (
    currentSessionId: () => string | null,
    onTodosUpdate: (todos: any[]) => void
  ) => Promise<void>
  sendMessage: (
    content: string,
    currentSessionId: string | null,
    buildSystemMessage: () => ChatMessage,
    maxTokens: number,
    temperature: number,
    llmEndpoint: string,
    onSaveSession: () => Promise<void>,
    onTokensUpdate: (tokens: number) => void,
    onTodosLoad: (sessionId: string) => Promise<void>
  ) => Promise<void>
  cancelMessage: (currentSessionId: string | null) => Promise<void>
  deleteMessage: (messageId: string, onSaveSession: () => Promise<void>, onTokensUpdate: (tokensToSubtract: number) => void) => Promise<void>
  startEditing: (messageId: string) => void
  cancelEditing: () => void
  editMessage: (
    messageId: string,
    newContent: string,
    onSaveSession: () => Promise<void>,
    onTokensUpdate: (tokensToSubtract: number) => void
  ) => Promise<void>
  resendConversation: (
    currentSessionId: string | null,
    buildSystemMessage: () => ChatMessage,
    maxTokens: number,
    temperature: number,
    llmEndpoint: string,
    onSaveSession: () => Promise<void>,
    onTokensUpdate: (tokens: number) => void,
    onTodosLoad: (sessionId: string) => Promise<void>
  ) => Promise<void>
  clearMessages: () => void
  addMessage: (message: ChatMessage) => void
}

export function createMessagesStore(): MessagesState & MessagesComputed & MessagesActions {
  // State
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  const isCancelling = ref(false)
  const error = ref<string | null>(null)
  const editingMessageId = ref<string | null>(null)

  // Computed
  const lastMessage = computed(() => {
    return messages.value[messages.value.length - 1] || null
  })

  // Actions
  const initializeMessageListeners = async (
    currentSessionId: () => string | null,
    onTodosUpdate: (todos: any[]) => void
  ) => {
    // Set up event listener for intermediate LLM messages
    await listen<IntermediateMessage>('llm-intermediate-message', (event) => {
      console.log('Received intermediate LLM message:', event.payload)
      const intermediateMsg = event.payload

      // Only process if this is for the current session
      if (!intermediateMsg.session_id || currentSessionId() === intermediateMsg.session_id) {
        const message: ChatMessage = {
          id: `intermediate_${Date.now()}_${Math.random()}`,
          role: 'assistant',
          content: intermediateMsg.content,
          timestamp: Date.now(),
          isIntermediate: true,
          iteration: intermediateMsg.iteration,
          toolCalls: intermediateMsg.tool_calls
        }
        messages.value.push(message)
        console.log(`Added intermediate message (iteration ${intermediateMsg.iteration})`)
      }
    })

    // Set up event listener for tool result messages
    await listen<ToolResultMessage>('tool-result-message', (event) => {
      console.log('Received tool result message:', event.payload)
      const toolResult = event.payload

      // Only process if this is for the current session
      if (!toolResult.session_id || currentSessionId() === toolResult.session_id) {
        const message: ChatMessage = {
          id: `tool_${Date.now()}_${Math.random()}`,
          role: 'tool',
          content: toolResult.result,
          timestamp: Date.now(),
          toolName: toolResult.tool_name,
          success: toolResult.success,
          iteration: toolResult.iteration,
          tool_call_id: toolResult.tool_call_id
        }
        messages.value.push(message)
        console.log(`Added tool result message: ${toolResult.tool_name}`)
      }
    })

    // Set up event listener for todos updates
    await listen<{session_id: string, todos: any[]}>('todos-updated', (event) => {
      console.log('Received todos update event:', event.payload)
      const update = event.payload

      // Only process if this is for the current session
      if (currentSessionId() === update.session_id) {
        onTodosUpdate(update.todos)
        console.log(`Updated todos for session ${update.session_id}: ${update.todos.length} items`)
      }
    })
  }

  const sendMessage = async (
    content: string,
    currentSessionId: string | null,
    buildSystemMessage: () => ChatMessage,
    maxTokens: number,
    temperature: number,
    llmEndpoint: string,
    onSaveSession: () => Promise<void>,
    onTokensUpdate: (tokens: number) => void,
    onTodosLoad: (sessionId: string) => Promise<void>
  ): Promise<void> => {
    if (!content.trim() || isLoading.value) return

    // Ensure we have a valid session before sending
    if (!currentSessionId) {
      console.error('Cannot send message: no active session')
      error.value = 'No active chat session. Please refresh the page.'
      return
    }

    error.value = null
    isCancelling.value = false
    isLoading.value = true

    // Add user message
    const userMessage: ChatMessage = {
      id: `msg_${Date.now()}`,
      role: 'user',
      content: content.trim(),
      timestamp: Date.now()
    }
    messages.value.push(userMessage)

    // Immediately save user message to session file
    try {
      await onSaveSession()
      console.log('User message saved to session immediately')
    } catch (saveError) {
      console.warn('Failed to save user message immediately:', saveError)
      // Continue with LLM call even if save fails
    }

    try {
      // Build system message with current context
      const systemMessage = buildSystemMessage()

      // Prepare messages for API (keep full content including thinking blocks)
      const conversationMessages = messages.value.map(msg => ({
        role: msg.role,
        content: msg.content,
        // Include tool_call_id for tool result messages (required by API)
        ...(msg.tool_call_id ? { tool_call_id: msg.tool_call_id } : {})
      }))

      // Combine system message with conversation (system message always first)
      const apiMessages = [
        { role: systemMessage.role, content: systemMessage.content },
        ...conversationMessages
      ]

      // Extract campaign info from context if available
      const { useSharedContextStore } = await import('../sharedContext')
      const contextStore = useSharedContextStore()
      const campaignDirectoryPath = contextStore.campaign?.directory_path || null
      const campaignId = contextStore.campaign?.id ? parseInt(contextStore.campaign.id, 10) : null

      // Send to backend
      const response = await invoke<ChatResponseWithUsage>('send_chat_message', {
        messages: apiMessages,
        maxTokens: maxTokens,
        temperature: temperature,
        enableTools: true,  // Enable tools for testing
        sessionId: currentSessionId,
        ollamaUrl: llmEndpoint,
        campaignDirectoryPath: campaignDirectoryPath,
        campaignId: campaignId
      })

      // Add assistant response
      const assistantMessage: ChatMessage = {
        id: `msg_${Date.now()}_assistant`,
        role: 'assistant',
        content: response.content, // Keep raw content for display (thinking blocks will be rendered as collapsible)
        timestamp: Date.now(),
        tokenUsage: {
          prompt: response.prompt_tokens,
          completion: response.completion_tokens,
          total: response.total_tokens
        }
      }

      messages.value.push(assistantMessage)

      // Immediately save assistant message to session file
      try {
        await onSaveSession()
        console.log('Assistant message saved to session immediately')
      } catch (saveError) {
        console.warn('Failed to save assistant message immediately:', saveError)
      }

      // Update total tokens
      onTokensUpdate(response.total_tokens)

      // Refresh todos after any assistant message (simple and reliable)
      if (currentSessionId) {
        console.log('Refreshing todos after message for session:', currentSessionId)
        await onTodosLoad(currentSessionId)
      } else {
        console.warn('No current session ID available for todo refresh')
      }

    } catch (err) {
      console.error('Failed to send message:', err)
      error.value = String(err)

      // Don't remove user message - it's already been saved to the session file
      // User should see their message was sent even if LLM fails to respond
      console.log('User message preserved in UI despite LLM error')
    } finally {
      isLoading.value = false
    }
  }

  const cancelMessage = async (currentSessionId: string | null): Promise<void> => {
    console.log('cancelMessage called, isLoading:', isLoading.value, 'sessionId:', currentSessionId)
    if (!isLoading.value) {
      console.log('Not loading, cancellation skipped')
      return
    }

    isCancelling.value = true
    try {
      console.log('Sending cancel_chat_message to backend...')
      await invoke('cancel_chat_message', {
        sessionId: currentSessionId
      })
      console.log('Cancel request completed successfully')
    } catch (err) {
      console.error('Failed to cancel message:', err)
    } finally {
      isCancelling.value = false
      isLoading.value = false
    }
  }

  const deleteMessage = async (
    messageId: string,
    onSaveSession: () => Promise<void>,
    onTokensUpdate: (tokensToSubtract: number) => void
  ) => {
    const idx = messages.value.findIndex(m => m.id === messageId)
    if (idx !== -1) {
      const msg = messages.value[idx]
      if (msg.tokenUsage) {
        onTokensUpdate(-msg.tokenUsage.total)
      }
      messages.value.splice(idx, 1)
      await onSaveSession()
    }
  }

  const startEditing = (messageId: string) => {
    editingMessageId.value = messageId
  }

  const cancelEditing = () => {
    editingMessageId.value = null
  }

  const editMessage = async (
    messageId: string,
    newContent: string,
    onSaveSession: () => Promise<void>,
    onTokensUpdate: (tokensToSubtract: number) => void
  ) => {
    const idx = messages.value.findIndex(m => m.id === messageId)
    if (idx === -1) return

    // Update the message content
    messages.value[idx].content = newContent.trim()

    // Remove all messages after the edited message and calculate token reduction
    let tokensToRemove = 0
    const removedMessages = messages.value.splice(idx + 1)
    for (const msg of removedMessages) {
      if (msg.tokenUsage) {
        tokensToRemove += msg.tokenUsage.total
      }
    }

    if (tokensToRemove > 0) {
      onTokensUpdate(-tokensToRemove)
    }

    // Clear editing state
    editingMessageId.value = null

    // Save the updated session
    await onSaveSession()
  }

  const resendConversation = async (
    currentSessionId: string | null,
    buildSystemMessage: () => ChatMessage,
    maxTokens: number,
    temperature: number,
    llmEndpoint: string,
    onSaveSession: () => Promise<void>,
    onTokensUpdate: (tokens: number) => void,
    onTodosLoad: (sessionId: string) => Promise<void>
  ): Promise<void> => {
    if (isLoading.value || messages.value.length === 0) return

    if (!currentSessionId) {
      console.error('Cannot resend: no active session')
      error.value = 'No active chat session. Please refresh the page.'
      return
    }

    error.value = null
    isCancelling.value = false
    isLoading.value = true

    try {
      const systemMessage = buildSystemMessage()

      const conversationMessages = messages.value.map(msg => ({
        role: msg.role,
        content: msg.content,
        // Include tool_call_id for tool result messages (required by API)
        ...(msg.tool_call_id ? { tool_call_id: msg.tool_call_id } : {})
      }))

      const apiMessages = [
        { role: systemMessage.role, content: systemMessage.content },
        ...conversationMessages
      ]

      const { useSharedContextStore } = await import('../sharedContext')
      const contextStore = useSharedContextStore()
      const campaignDirectoryPath = contextStore.campaign?.directory_path || null
      const campaignId = contextStore.campaign?.id ? parseInt(contextStore.campaign.id, 10) : null

      const response = await invoke<ChatResponseWithUsage>('send_chat_message', {
        messages: apiMessages,
        maxTokens: maxTokens,
        temperature: temperature,
        enableTools: true,
        sessionId: currentSessionId,
        ollamaUrl: llmEndpoint,
        campaignDirectoryPath: campaignDirectoryPath,
        campaignId: campaignId
      })

      const assistantMessage: ChatMessage = {
        id: `msg_${Date.now()}_assistant`,
        role: 'assistant',
        content: response.content,
        timestamp: Date.now(),
        tokenUsage: {
          prompt: response.prompt_tokens,
          completion: response.completion_tokens,
          total: response.total_tokens
        }
      }

      messages.value.push(assistantMessage)

      try {
        await onSaveSession()
      } catch (saveError) {
        console.warn('Failed to save assistant message:', saveError)
      }

      onTokensUpdate(response.total_tokens)

      if (currentSessionId) {
        await onTodosLoad(currentSessionId)
      }

    } catch (err) {
      console.error('Failed to resend conversation:', err)
      error.value = String(err)
    } finally {
      isLoading.value = false
    }
  }

  const clearMessages = () => {
    messages.value = []
    editingMessageId.value = null
  }

  const addMessage = (message: ChatMessage) => {
    messages.value.push(message)
  }

  return {
    // State
    messages,
    isLoading,
    isCancelling,
    error,
    editingMessageId,

    // Computed
    lastMessage,

    // Actions
    initializeMessageListeners,
    sendMessage,
    cancelMessage,
    deleteMessage,
    startEditing,
    cancelEditing,
    editMessage,
    resendConversation,
    clearMessages,
    addMessage
  }
}
