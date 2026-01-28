import { ref, type Ref } from 'vue'

export interface ClipboardReturn {
  /** The text that was most recently copied (clears after timeout) */
  copiedText: Ref<string>
  /** Copy text to clipboard with visual feedback */
  copy: (text: string) => Promise<boolean>
  /** Check if a specific text was just copied */
  wasCopied: (text: string) => boolean
}

/**
 * Composable for clipboard operations with visual feedback.
 *
 * @param feedbackDuration - How long to show "Copied!" feedback (ms, default 2000)
 *
 * @example
 * ```ts
 * const { copy, wasCopied } = useClipboard()
 *
 * // In template:
 * // <button @click="copy(someText)">{{ wasCopied(someText) ? 'Copied!' : 'Copy' }}</button>
 * ```
 */
export function useClipboard(feedbackDuration = 2000): ClipboardReturn {
  const copiedText = ref('')
  let timeoutId: ReturnType<typeof setTimeout> | null = null

  async function copy(text: string): Promise<boolean> {
    try {
      await navigator.clipboard.writeText(text)

      // Clear any existing timeout
      if (timeoutId) {
        clearTimeout(timeoutId)
      }

      copiedText.value = text

      // Auto-clear after duration
      timeoutId = setTimeout(() => {
        copiedText.value = ''
        timeoutId = null
      }, feedbackDuration)

      return true
    } catch (error) {
      console.error('Failed to copy to clipboard:', error)
      return false
    }
  }

  function wasCopied(text: string): boolean {
    return copiedText.value === text
  }

  return {
    copiedText,
    copy,
    wasCopied
  }
}
