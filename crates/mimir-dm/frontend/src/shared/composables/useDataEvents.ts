/**
 * Composable for using the data event bus with automatic cleanup.
 *
 * @example
 * const { on, emit } = useDataEvents()
 *
 * // Subscribe - automatically cleaned up on unmount
 * on('module:monsters:changed', ({ moduleId }) => {
 *   if (moduleId === currentModuleId.value) {
 *     refetch()
 *   }
 * })
 *
 * // Emit after mutation
 * emit('module:monsters:changed', { moduleId: 123 })
 */

import { onUnmounted } from 'vue'
import { dataEvents, type DataEventName, type DataEventPayloads } from '../utils/dataEvents'

export function useDataEvents() {
  const cleanupFunctions: (() => void)[] = []

  /**
   * Subscribe to an event. Automatically cleaned up when component unmounts.
   */
  function on<T extends DataEventName>(
    event: T,
    listener: (payload: DataEventPayloads[T]) => void
  ): () => void {
    const unsubscribe = dataEvents.on(event, listener)
    cleanupFunctions.push(unsubscribe)
    return unsubscribe
  }

  /**
   * Subscribe to an event once. Automatically cleaned up when component unmounts.
   */
  function once<T extends DataEventName>(
    event: T,
    listener: (payload: DataEventPayloads[T]) => void
  ): () => void {
    const unsubscribe = dataEvents.once(event, listener)
    cleanupFunctions.push(unsubscribe)
    return unsubscribe
  }

  /**
   * Emit an event.
   */
  function emit<T extends DataEventName>(
    event: T,
    payload: DataEventPayloads[T]
  ): void {
    dataEvents.emit(event, payload)
  }

  // Cleanup all subscriptions when component unmounts
  onUnmounted(() => {
    cleanupFunctions.forEach(cleanup => cleanup())
  })

  return {
    on,
    once,
    emit,
    // Expose raw bus for advanced use cases
    bus: dataEvents
  }
}
