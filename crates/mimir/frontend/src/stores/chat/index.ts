import { defineStore } from 'pinia'
import { createTokensStore } from './tokens'
import { createSessionStore } from './session'
import { createMessagesStore } from './messages'
import { createTodosStore } from './todos'
import { createToolConfirmationsStore } from './tool-confirmations'

// Export all types and interfaces
export * from './types'
export * from './messages'
export * from './session'
export * from './tokens'
export * from './todos'

export const useChatStore = defineStore('chat', () => {
  // Create all concern-specific stores
  const messagesStore = createMessagesStore()
  const sessionStore = createSessionStore()
  const tokensStore = createTokensStore(() => messagesStore.messages.value)
  const todosStore = createTodosStore()
  const toolConfirmationsStore = createToolConfirmationsStore()

  // Coordinated initialization
  const initialize = async () => {
    try {
      // Initialize tokens (get model info and load config)
      await tokensStore.initializeTokens()

      // Load available sessions
      await sessionStore.loadSessions()

      // Try to load most recent session or create new one
      if (sessionStore.sessions.value.length > 0) {
        // Load the most recent session
        await loadSession(sessionStore.sessions.value[0].id)
      } else {
        // Create a new session
        await createNewSession()
      }

      // Set up event listeners for messages
      await messagesStore.initializeMessageListeners(
        () => sessionStore.currentSessionId.value,
        (todos) => todosStore.updateTodos(todos)
      )

      // Set up tool confirmation listener
      await toolConfirmationsStore.initializeToolConfirmationListener((request) => {
        // Add a system message to show the confirmation UI
        messagesStore.addMessage({
          id: `confirm_${Date.now()}`,
          role: 'system',
          content: `TOOL_CONFIRMATION:${request.id}`,
          timestamp: Date.now()
        })
      })
    } catch (err) {
      console.error('Failed to initialize chat:', err)
      messagesStore.error.value = String(err)
    }
  }

  // Wrapped actions that coordinate between stores
  const sendMessage = async (content: string): Promise<void> => {
    await messagesStore.sendMessage(
      content,
      sessionStore.currentSessionId.value,
      tokensStore.buildSystemMessage,
      tokensStore.systemConfig.value.maxTokens || tokensStore.maxResponseTokens.value,
      tokensStore.systemConfig.value.temperature || 0.3,
      tokensStore.systemConfig.value.llmEndpoint || 'http://localhost:11434',
      () => saveCurrentSession(),
      (tokens) => { tokensStore.totalTokensUsed.value += tokens },
      (sessionId) => todosStore.loadTodosForSession(sessionId)
    )
  }

  const cancelMessage = async (): Promise<void> => {
    await messagesStore.cancelMessage(sessionStore.currentSessionId.value)
  }

  const deleteMessage = async (messageId: string) => {
    await messagesStore.deleteMessage(
      messageId,
      () => saveCurrentSession(),
      (tokens) => { tokensStore.totalTokensUsed.value += tokens }
    )
  }

  const startEditing = (messageId: string) => {
    messagesStore.startEditing(messageId)
  }

  const cancelEditing = () => {
    messagesStore.cancelEditing()
  }

  const editMessage = async (messageId: string, newContent: string) => {
    await messagesStore.editMessage(
      messageId,
      newContent,
      () => saveCurrentSession(),
      (tokens) => { tokensStore.totalTokensUsed.value += tokens }
    )
  }

  const resendConversation = async (): Promise<void> => {
    await messagesStore.resendConversation(
      sessionStore.currentSessionId.value,
      tokensStore.buildSystemMessage,
      tokensStore.systemConfig.value.maxTokens || tokensStore.maxResponseTokens.value,
      tokensStore.systemConfig.value.temperature || 0.3,
      tokensStore.systemConfig.value.llmEndpoint || 'http://localhost:11434',
      () => saveCurrentSession(),
      (tokens) => { tokensStore.totalTokensUsed.value += tokens },
      (sessionId) => todosStore.loadTodosForSession(sessionId)
    )
  }

  const clearHistory = async () => {
    if (sessionStore.currentSessionId.value) {
      // Create a new session to replace the current one
      await createNewSession()
    }
  }

  const loadSession = async (sessionId: string) => {
    await sessionStore.loadSession(
      sessionId,
      (messages) => { messagesStore.messages.value = messages },
      (totalTokens) => { tokensStore.totalTokensUsed.value = totalTokens },
      (sessionId) => {
        todosStore.clearTodos()
        return todosStore.loadTodosForSession(sessionId)
      }
    )
  }

  const saveCurrentSession = async () => {
    await sessionStore.saveCurrentSession(messagesStore.messages.value)
  }

  const createNewSession = async () => {
    await sessionStore.createNewSession(
      () => { messagesStore.clearMessages() },
      () => { tokensStore.totalTokensUsed.value = 0 },
      (sessionId) => {
        todosStore.clearTodos()
        return todosStore.loadTodosForSession(sessionId)
      }
    )
  }

  const deleteSession = async (sessionId: string) => {
    await sessionStore.deleteSession(
      sessionId,
      () => { messagesStore.clearMessages() },
      () => { tokensStore.totalTokensUsed.value = 0 },
      (sessionId) => {
        todosStore.clearTodos()
        return todosStore.loadTodosForSession(sessionId)
      }
    )
  }

  const switchToSession = async (sessionId: string) => {
    await sessionStore.switchToSession(
      sessionId,
      (messages) => { messagesStore.messages.value = messages },
      (totalTokens) => { tokensStore.totalTokensUsed.value = totalTokens },
      (sessionId) => {
        todosStore.clearTodos()
        return todosStore.loadTodosForSession(sessionId)
      }
    )
  }

  return {
    // State from all stores
    ...messagesStore,
    ...sessionStore,
    ...tokensStore,
    ...todosStore,
    ...toolConfirmationsStore,

    // Coordinated actions (override individual store actions where coordination is needed)
    initialize,
    sendMessage,
    cancelMessage,
    deleteMessage,
    startEditing,
    cancelEditing,
    editMessage,
    resendConversation,
    clearHistory,
    loadSession,
    saveCurrentSession,
    createNewSession,
    deleteSession,
    switchToSession
  }
})
