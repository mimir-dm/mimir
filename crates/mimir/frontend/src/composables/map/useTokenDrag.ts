import { ref, type Ref, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import type { Token } from '@/types/api'
import type TokenRenderer from '@/components/tokens/TokenRenderer.vue'

/**
 * Backend token response type (matches TokenResponse from backend)
 */
interface BackendToken {
  id: string
  map_id: string
  name: string
  token_type: string
  size: string
  x: number
  y: number
  visible_to_players: boolean
  color: string | null
  image_path: string | null
  monster_id: string | null
  character_id: string | null
  notes: string | null
  vision_type: string
  vision_range_ft: number | null
  vision_bright_ft: number | null
  vision_dim_ft: number | null
  vision_dark_ft: number
  light_radius_ft: number
  created_at: string
  updated_at: string
}

/**
 * Transform backend token to frontend Token format
 */
function transformToken(backendToken: BackendToken): Token {
  return {
    id: backendToken.id,
    map_id: backendToken.map_id,
    name: backendToken.name,
    token_type: backendToken.token_type as Token['token_type'],
    size: backendToken.size as Token['size'],
    x: backendToken.x,
    y: backendToken.y,
    visible_to_players: backendToken.visible_to_players,
    color: backendToken.color,
    image_path: backendToken.image_path,
    monster_id: backendToken.monster_id,
    character_id: backendToken.character_id,
    notes: backendToken.notes,
    vision_type: backendToken.vision_type as Token['vision_type'],
    vision_range_ft: backendToken.vision_range_ft,
    vision_bright_ft: backendToken.vision_bright_ft,
    vision_dim_ft: backendToken.vision_dim_ft,
    vision_dark_ft: backendToken.vision_dark_ft,
    light_radius_ft: backendToken.light_radius_ft,
    created_at: backendToken.created_at,
    updated_at: backendToken.updated_at
  }
}

interface UseTokenDragOptions {
  tokens: Ref<Token[]>
  selectedTokenId: Ref<string | null>
  zoom: Ref<number>
  gridSizePx: ComputedRef<number>
  gridOffsetX: ComputedRef<number>
  gridOffsetY: ComputedRef<number>
  mapId: ComputedRef<string | null>
  isDisplayOpen: Ref<boolean>
  revealMap: Ref<boolean>
  tokenRendererRef: Ref<InstanceType<typeof TokenRenderer> | null>
  onTokenMoved?: () => void
  onFogUpdate?: () => void
}

/**
 * Composable for handling token drag-and-drop operations
 */
export function useTokenDrag(options: UseTokenDragOptions) {
  const {
    tokens,
    selectedTokenId,
    zoom,
    gridSizePx,
    gridOffsetX,
    gridOffsetY,
    mapId,
    isDisplayOpen,
    revealMap,
    tokenRendererRef,
    onTokenMoved,
    onFogUpdate
  } = options

  // Drag state
  const draggingTokenId = ref<string | null>(null)
  const dragOffset = ref<{ x: number; y: number } | null>(null)
  const dragStartPos = ref<{ x: number; y: number; tokenX: number; tokenY: number } | null>(null)

  /**
   * Snap position to grid center
   */
  function snapToGrid(x: number, y: number): { x: number; y: number } {
    const gridSize = gridSizePx.value
    const offsetX = gridOffsetX.value
    const offsetY = gridOffsetY.value

    // Snap to nearest grid cell center
    const gridX = Math.round((x - offsetX) / gridSize) * gridSize + offsetX + gridSize / 2
    const gridY = Math.round((y - offsetY) / gridSize) * gridSize + offsetY + gridSize / 2

    return { x: gridX, y: gridY }
  }

  /**
   * Send tokens with live drag offset for smooth player display updates
   */
  async function sendTokensToDisplayWithDragOffset() {
    if (!isDisplayOpen.value || !mapId.value) return

    const visibleTokens = tokens.value
      .filter(t => t.visible_to_players)
      .map(t => {
        if (t.id === draggingTokenId.value && dragStartPos.value && dragOffset.value) {
          // Apply drag offset to the dragging token
          return {
            ...t,
            x: dragStartPos.value.tokenX + dragOffset.value.x,
            y: dragStartPos.value.tokenY + dragOffset.value.y
          }
        }
        return t
      })

    try {
      await emit('player-display:tokens-update', {
        mapId: mapId.value,
        tokens: visibleTokens
      })
    } catch (e) {
      console.error('Failed to send tokens to display:', e)
    }
  }

  /**
   * Handle token drag start
   */
  function handleTokenDragStart(event: MouseEvent, token: Token) {
    draggingTokenId.value = token.id
    selectedTokenId.value = token.id
    dragOffset.value = { x: 0, y: 0 }
    dragStartPos.value = {
      x: event.clientX,
      y: event.clientY,
      tokenX: token.x,
      tokenY: token.y
    }

    // Add document-level listeners for drag
    document.addEventListener('mousemove', handleTokenDrag)
    document.addEventListener('mouseup', handleTokenDragEnd)
  }

  /**
   * Handle token drag movement
   */
  function handleTokenDrag(event: MouseEvent) {
    if (!draggingTokenId.value || !dragStartPos.value) return

    const deltaX = (event.clientX - dragStartPos.value.x) / zoom.value
    const deltaY = (event.clientY - dragStartPos.value.y) / zoom.value

    // Mark as moved if we've dragged more than 5px
    if (Math.abs(deltaX) > 5 || Math.abs(deltaY) > 5) {
      tokenRendererRef.value?.setHasMoved(true)
    }

    dragOffset.value = { x: deltaX, y: deltaY }

    // Send live position update to player display for visible tokens
    const token = tokens.value.find(t => t.id === draggingTokenId.value)
    if (token?.visible_to_players) {
      sendTokensToDisplayWithDragOffset()
    }
  }

  /**
   * Handle token drag end
   */
  async function handleTokenDragEnd() {
    document.removeEventListener('mousemove', handleTokenDrag)
    document.removeEventListener('mouseup', handleTokenDragEnd)

    if (!draggingTokenId.value || !dragStartPos.value || !dragOffset.value) {
      draggingTokenId.value = null
      dragOffset.value = null
      dragStartPos.value = null
      return
    }

    const token = tokens.value.find(t => t.id === draggingTokenId.value)
    if (!token) {
      draggingTokenId.value = null
      dragOffset.value = null
      dragStartPos.value = null
      return
    }

    // Only process if we actually moved (more than 5px in any direction)
    const didMove = Math.abs(dragOffset.value.x) > 5 || Math.abs(dragOffset.value.y) > 5
    if (!didMove) {
      draggingTokenId.value = null
      dragOffset.value = null
      dragStartPos.value = null
      return
    }

    // Calculate new position with grid snapping
    const rawX = dragStartPos.value.tokenX + dragOffset.value.x
    const rawY = dragStartPos.value.tokenY + dragOffset.value.y
    const { x: snappedX, y: snappedY } = snapToGrid(rawX, rawY)

    // Only update if position changed
    if (snappedX !== token.x || snappedY !== token.y) {
      // Convert pixel coordinates to grid coordinates
      const gridSize = gridSizePx.value
      const gridX = Math.floor(snappedX / gridSize)
      const gridY = Math.floor(snappedY / gridSize)

      try {
        const response = await invoke<{ success: boolean; data?: BackendToken; error?: string }>('update_token_position', {
          id: token.id,
          gridX,
          gridY
        })

        if (response.success && response.data) {
          // Update local token position from response (backend returns pixel coords)
          const updated = transformToken(response.data)
          token.x = updated.x
          token.y = updated.y
          // Notify parent to sync to player display
          onTokenMoved?.()
          // Update fog/visibility when hiding is active (revealMap OFF)
          if (!revealMap.value) {
            onFogUpdate?.()
          }
        } else {
          console.error('Failed to update token position:', response.error)
        }
      } catch (e) {
        console.error('Failed to update token position:', e)
      }
    }

    // Clear drag state
    draggingTokenId.value = null
    dragOffset.value = null
    dragStartPos.value = null
  }

  /**
   * Cleanup function for unmount
   */
  function cleanup() {
    document.removeEventListener('mousemove', handleTokenDrag)
    document.removeEventListener('mouseup', handleTokenDragEnd)
  }

  return {
    // State
    draggingTokenId,
    dragOffset,
    dragStartPos,

    // Methods
    handleTokenDragStart,
    snapToGrid,
    cleanup,

    // Utilities
    transformToken
  }
}

export { transformToken, type BackendToken }
