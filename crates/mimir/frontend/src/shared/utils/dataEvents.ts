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
// Note: IDs are strings (UUIDs) matching the mimir-core backend
export interface DataEventPayloads {
  // Module events
  'module:created': { campaignId: string; moduleId: string }
  'module:updated': { moduleId: string }
  'module:deleted': { campaignId: string; moduleId: string }

  // Module monster events
  'module:monsters:changed': { moduleId: string }
  'module:monster:added': { moduleId: string; monsterId: string }
  'module:monster:updated': { moduleId: string; monsterId: string }
  'module:monster:removed': { moduleId: string; monsterId: string }

  // Module NPC events
  'module:npcs:changed': { moduleId: string }
  'module:npc:added': { moduleId: string; npcId: string }
  'module:npc:removed': { moduleId: string; npcId: string }

  // Module item events
  'module:items:changed': { moduleId: string }

  // Module map events
  'module:maps:changed': { moduleId: string }
  'module:map:uploaded': { moduleId: string; mapId: string }

  // Document events
  'document:created': { moduleId: string; documentId: string }
  'document:updated': { documentId: string }
  'document:deleted': { moduleId: string; documentId: string }

  // Character events
  'character:created': { campaignId: string; characterId: string }
  'character:updated': { characterId: string }
  'character:deleted': { campaignId: string; characterId: string }

  // Campaign events
  'campaign:created': { campaignId: string }
  'campaign:updated': { campaignId: string }
  'campaign:deleted': { campaignId: string }

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
