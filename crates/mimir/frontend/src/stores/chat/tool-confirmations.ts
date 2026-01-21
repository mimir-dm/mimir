import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ToolConfirmationRequest, PendingConfirmation, ChatMessage } from './types'

interface ToolConfirmationsState {
  pendingConfirmations: Ref<Map<string, PendingConfirmation>>
}

interface ToolConfirmationsActions {
  initializeToolConfirmationListener: (onConfirmationRequest: (request: ToolConfirmationRequest) => void) => Promise<void>
  confirmToolAction: (confirmationId: string) => Promise<void>
  rejectToolAction: (confirmationId: string) => Promise<void>
  getConfirmationForMessage: (messageContent: string) => PendingConfirmation | null
}

export function createToolConfirmationsStore(): ToolConfirmationsState & ToolConfirmationsActions {
  // State
  const pendingConfirmations = ref<Map<string, PendingConfirmation>>(new Map())

  // Actions
  const initializeToolConfirmationListener = async (
    onConfirmationRequest: (request: ToolConfirmationRequest) => void
  ) => {
    // Set up event listener for tool confirmation requests
    await listen<ToolConfirmationRequest>('tool-confirmation-request', (event) => {
      console.log('Received confirmation request:', event.payload)
      const request = event.payload

      // Add to pending confirmations
      pendingConfirmations.value.set(request.id, {
        request,
        status: 'pending',
        messageId: `confirm_${Date.now()}`
      })

      // Notify caller to add system message
      onConfirmationRequest(request)
    })
  }

  const confirmToolAction = async (confirmationId: string) => {
    try {
      await invoke('confirm_tool_action', {
        confirmationId,
        confirmed: true
      })

      // Update status
      const confirmation = pendingConfirmations.value.get(confirmationId)
      if (confirmation) {
        confirmation.status = 'confirmed'
      }
    } catch (error) {
      console.error('Failed to confirm action:', error)
      throw error
    }
  }

  const rejectToolAction = async (confirmationId: string) => {
    try {
      await invoke('confirm_tool_action', {
        confirmationId,
        confirmed: false
      })

      // Update status
      const confirmation = pendingConfirmations.value.get(confirmationId)
      if (confirmation) {
        confirmation.status = 'rejected'
      }
    } catch (error) {
      console.error('Failed to reject action:', error)
      throw error
    }
  }

  const getConfirmationForMessage = (messageContent: string) => {
    // Check if this is a confirmation message
    if (messageContent.startsWith('TOOL_CONFIRMATION:')) {
      const confirmationId = messageContent.split(':')[1]
      return pendingConfirmations.value.get(confirmationId) || null
    }
    return null
  }

  return {
    // State
    pendingConfirmations,

    // Actions
    initializeToolConfirmationListener,
    confirmToolAction,
    rejectToolAction,
    getConfirmationForMessage
  }
}
