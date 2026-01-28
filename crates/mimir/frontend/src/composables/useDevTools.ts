import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type SeedAction = 'seed' | 'reseed' | 'clear'
export type MessageType = 'success' | 'error'

export interface DevToolsReturn {
  /** Whether the app is running in dev mode */
  isDevMode: Ref<boolean>
  /** Whether dev test data has been seeded */
  isDevSeeded: Ref<boolean>
  /** Whether a seed action is currently in progress */
  isPending: Ref<boolean>
  /** Feedback message from the last action */
  message: Ref<string>
  /** Type of the feedback message */
  messageType: Ref<MessageType>
  /** Initialize dev mode check (call on mount) */
  initialize: () => Promise<void>
  /** Seed test data */
  seed: () => Promise<void>
  /** Clear and reseed test data */
  reseed: () => Promise<void>
  /** Clear test data */
  clear: () => Promise<void>
}

/**
 * Composable for development tools functionality.
 *
 * Handles:
 * - Dev mode detection
 * - Test data seeding state
 * - Seed/reseed/clear actions with consolidated error handling
 *
 * @example
 * ```ts
 * const devTools = useDevTools()
 *
 * onMounted(() => devTools.initialize())
 *
 * // In template:
 * // <button v-if="!devTools.isDevSeeded.value" @click="devTools.seed">Seed</button>
 * ```
 */
export function useDevTools(): DevToolsReturn {
  const isDevMode = ref(false)
  const isDevSeeded = ref(false)
  const isPending = ref(false)
  const message = ref('')
  const messageType = ref<MessageType>('success')

  async function checkDevMode(): Promise<void> {
    try {
      const response = await invoke<{ success: boolean; data: boolean }>('is_dev_mode')
      isDevMode.value = response.success && response.data
    } catch (error) {
      console.error('Failed to check dev mode:', error)
      isDevMode.value = false
    }
  }

  async function checkDevSeeded(): Promise<void> {
    try {
      const response = await invoke<{ success: boolean; data: boolean }>('is_dev_seeded')
      isDevSeeded.value = response.success && response.data
    } catch (error) {
      console.error('Failed to check dev seeded status:', error)
      isDevSeeded.value = false
    }
  }

  async function initialize(): Promise<void> {
    await checkDevMode()
    if (isDevMode.value) {
      await checkDevSeeded()
    }
  }

  /**
   * Execute a seed action with consolidated error handling
   */
  async function executeSeedAction(
    action: SeedAction,
    command: string,
    successMessage: string
  ): Promise<void> {
    isPending.value = true
    message.value = ''

    try {
      const response = await invoke<{ success: boolean; data?: boolean; error?: string }>(command)

      if (response.success) {
        // For seed command, check if data already existed
        if (action === 'seed' && response.data === false) {
          message.value = 'Test data already exists.'
        } else {
          message.value = successMessage
        }
        messageType.value = 'success'
        await checkDevSeeded()
      } else {
        message.value = response.error || `Failed to ${action} data`
        messageType.value = 'error'
      }
    } catch (error) {
      message.value = `Error: ${error}`
      messageType.value = 'error'
    } finally {
      isPending.value = false
    }
  }

  async function seed(): Promise<void> {
    await executeSeedAction('seed', 'seed_dev_data', 'Test data seeded successfully!')
  }

  async function reseed(): Promise<void> {
    await executeSeedAction('reseed', 'reseed_dev_data', 'Test data reseeded successfully!')
  }

  async function clear(): Promise<void> {
    await executeSeedAction('clear', 'clear_dev_data', 'Test data cleared successfully!')
  }

  return {
    isDevMode,
    isDevSeeded,
    isPending,
    message,
    messageType,
    initialize,
    seed,
    reseed,
    clear
  }
}
