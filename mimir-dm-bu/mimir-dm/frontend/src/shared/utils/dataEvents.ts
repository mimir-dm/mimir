/**
 * Data Event Bus - Lightweight pub/sub for cache invalidation and data refresh.
 *
 * Use this to coordinate data updates across components and composables.
 * When a mutation occurs, emit an event. Composables listen and refetch as needed.
 *
 * @example
 * // In a component after mutation:
 * import { dataEvents } from '@/shared/utils/dataEvents'
 * dataEvents.emit('module:monsters:changed', { moduleId: 123 })
 *
 * // In a composable:
 * import { dataEvents } from '@/shared/utils/dataEvents'
 * onMounted(() => {
 *   const unsubscribe = dataEvents.on('module:monsters:changed', (payload) => {
 *     if (payload.moduleId === currentModuleId.value) {
 *       refetch()
 *     }
 *   })
 *   onUnmounted(unsubscribe)
 * })
 */

// Event payload types for type safety
export interface DataEventPayloads {
  // Module events
  'module:created': { campaignId: number; moduleId: number }
  'module:updated': { moduleId: number }
  'module:deleted': { campaignId: number; moduleId: number }

  // Module monster events
  'module:monsters:changed': { moduleId: number }
  'module:monster:added': { moduleId: number; monsterId: number }
  'module:monster:updated': { moduleId: number; monsterId: number }
  'module:monster:removed': { moduleId: number; monsterId: number }

  // Module NPC events
  'module:npcs:changed': { moduleId: number }
  'module:npc:added': { moduleId: number; npcId: number }
  'module:npc:removed': { moduleId: number; npcId: number }

  // Module item events
  'module:items:changed': { moduleId: number }

  // Module map events
  'module:maps:changed': { moduleId: number }
  'module:map:uploaded': { moduleId: number; mapId: number }

  // Document events
  'document:created': { moduleId: number; documentId: number }
  'document:updated': { documentId: number }
  'document:deleted': { moduleId: number; documentId: number }

  // Character events
  'character:created': { campaignId: number; characterId: number }
  'character:updated': { characterId: number }
  'character:deleted': { campaignId: number; characterId: number }

  // Campaign events
  'campaign:created': { campaignId: number }
  'campaign:updated': { campaignId: number }
  'campaign:deleted': { campaignId: number }

  // Generic refresh trigger (use sparingly)
  'data:refresh-all': { scope?: string }
}

export type DataEventName = keyof DataEventPayloads

type Listener<T extends DataEventName> = (payload: DataEventPayloads[T]) => void

class DataEventBus {
  private listeners = new Map<DataEventName, Set<Listener<any>>>()
  private debugMode = false

  /**
   * Enable debug logging for all events
   */
  setDebug(enabled: boolean) {
    this.debugMode = enabled
  }

  /**
   * Subscribe to an event. Returns an unsubscribe function.
   */
  on<T extends DataEventName>(event: T, listener: Listener<T>): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set())
    }
    this.listeners.get(event)!.add(listener)

    // Return unsubscribe function
    return () => {
      this.listeners.get(event)?.delete(listener)
    }
  }

  /**
   * Subscribe to an event for one emission only.
   */
  once<T extends DataEventName>(event: T, listener: Listener<T>): () => void {
    const wrappedListener: Listener<T> = (payload) => {
      this.off(event, wrappedListener)
      listener(payload)
    }
    return this.on(event, wrappedListener)
  }

  /**
   * Unsubscribe a specific listener from an event.
   */
  off<T extends DataEventName>(event: T, listener: Listener<T>): void {
    this.listeners.get(event)?.delete(listener)
  }

  /**
   * Emit an event to all subscribers.
   */
  emit<T extends DataEventName>(event: T, payload: DataEventPayloads[T]): void {
    if (this.debugMode) {
      console.log(`[DataEvents] ${event}`, payload)
    }

    const eventListeners = this.listeners.get(event)
    if (eventListeners) {
      eventListeners.forEach(listener => {
        try {
          listener(payload)
        } catch (error) {
          console.error(`[DataEvents] Error in listener for ${event}:`, error)
        }
      })
    }
  }

  /**
   * Remove all listeners for an event, or all listeners if no event specified.
   */
  clear(event?: DataEventName): void {
    if (event) {
      this.listeners.delete(event)
    } else {
      this.listeners.clear()
    }
  }

  /**
   * Get the count of listeners for debugging.
   */
  listenerCount(event?: DataEventName): number {
    if (event) {
      return this.listeners.get(event)?.size ?? 0
    }
    let total = 0
    this.listeners.forEach(set => total += set.size)
    return total
  }
}

// Singleton instance
export const dataEvents = new DataEventBus()

// Enable debug mode in development
if (import.meta.env.DEV) {
  dataEvents.setDebug(true)
}
