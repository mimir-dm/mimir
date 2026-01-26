/**
 * Composable for handling dashboard entity focus events from the DM Map window.
 *
 * When the DM double-clicks a token, trap, or POI in the DM Map window, this
 * composable receives the event and provides callbacks to focus the appropriate
 * entity in the dashboard.
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface EntityFocusEvent {
  type: 'monster' | 'npc' | 'pc' | 'trap' | 'poi'
  entityId: string
  /** Token ID (for monster/npc/pc) */
  tokenId?: string
  /** Token name (for monster/npc/pc) */
  tokenName?: string
  /** Entity name (for trap/poi) */
  entityName?: string
}

export interface UseDashboardLinkOptions {
  /** Called when a monster should be focused */
  onFocusMonster?: (monsterId: string, tokenName: string) => void
  /** Called when an NPC should be focused */
  onFocusNpc?: (characterId: string, tokenName: string) => void
  /** Called when a PC should be focused */
  onFocusPc?: (characterId: string, tokenName: string) => void
  /** Called when a trap should be focused */
  onFocusTrap?: (trapId: string, trapName: string) => void
  /** Called when a POI should be focused */
  onFocusPoi?: (poiId: string, poiName: string) => void
}

/**
 * Composable for handling dashboard entity focus events.
 *
 * Usage:
 * ```typescript
 * useDashboardLink({
 *   onFocusMonster: (monsterId, tokenName) => {
 *     const monster = findMonsterById(monsterId)
 *     if (monster) selectMonster(monster)
 *   },
 *   onFocusNpc: (characterId, tokenName) => {
 *     const npc = findNpcById(characterId)
 *     if (npc) viewNpcDetail(npc)
 *   },
 *   onFocusTrap: (trapId, trapName) => {
 *     const trap = findTrapByName(trapName)
 *     if (trap) selectTrap(trap)
 *   },
 *   onFocusPoi: (poiId, poiName) => {
 *     // Handle POI focus
 *   }
 * })
 * ```
 */
export function useDashboardLink(options: UseDashboardLinkOptions = {}) {
  const lastFocusEvent = ref<EntityFocusEvent | null>(null)
  let unlisten: UnlistenFn | null = null

  async function setupListener() {
    try {
      unlisten = await listen<EntityFocusEvent>('dashboard:focus-entity', (event) => {
        const payload = event.payload
        lastFocusEvent.value = payload

        // Route to appropriate handler
        switch (payload.type) {
          case 'monster':
            options.onFocusMonster?.(payload.entityId, payload.tokenName || '')
            break
          case 'npc':
            options.onFocusNpc?.(payload.entityId, payload.tokenName || '')
            break
          case 'pc':
            options.onFocusPc?.(payload.entityId, payload.tokenName || '')
            break
          case 'trap':
            options.onFocusTrap?.(payload.entityId, payload.entityName || '')
            break
          case 'poi':
            options.onFocusPoi?.(payload.entityId, payload.entityName || '')
            break
        }
      })
    } catch (e) {
      console.error('Failed to set up dashboard link listener:', e)
    }
  }

  onMounted(() => {
    setupListener()
  })

  onUnmounted(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })

  return {
    /** The most recent focus event received */
    lastFocusEvent
  }
}

export default useDashboardLink
