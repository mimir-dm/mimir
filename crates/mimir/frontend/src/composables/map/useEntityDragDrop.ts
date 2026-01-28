import { ref, type Ref } from 'vue'

/**
 * Configuration options for entity drag-and-drop behavior.
 *
 * Supports two coordinate types:
 * - 'pixel': Entities stored with x/y pixel coordinates (tokens, lights)
 * - 'grid': Entities stored with grid_x/grid_y grid coordinates (traps, POIs)
 */
export interface EntityDragOptions<T> {
  /** Reference to the viewport element for mouse coordinate calculation */
  viewportRef: Ref<HTMLElement | null>
  /** Base scale of the map image */
  baseScale: Ref<number>
  /** Current zoom level */
  zoom: Ref<number>
  /** Current pan X offset */
  panX: Ref<number>
  /** Current pan Y offset */
  panY: Ref<number>
  /** Grid size in pixels (from UVTT data) */
  uvttGridSize: Ref<number>
  /** Reference to the entities array */
  entities: Ref<T[]>
  /** Coordinate type for this entity */
  coordType: 'pixel' | 'grid'
  /** Field names for x/y coordinates */
  coordFields: { x: keyof T; y: keyof T }
  /** Function to save entity position to backend */
  savePosition: (entity: T) => Promise<void>
  /** Function to reload entities on save error */
  reload: () => Promise<void>
  /** Whether to snap pixel-based entities to grid (default: false) */
  snapToGrid?: boolean
  /** Whether to stop event propagation on mousedown (default: true) */
  stopPropagation?: boolean
}

export interface EntityDragReturn<T> {
  /** Currently dragging entity (null if not dragging) */
  draggingEntity: Ref<T | null>
  /** Start dragging an entity (call from @mousedown) */
  onMouseDown: (event: MouseEvent, entity: T) => void
  /**
   * Handle mouse move during drag.
   * Returns true if this handler consumed the event.
   */
  onMouseMove: (event: MouseEvent) => boolean
  /**
   * Handle mouse up to save position.
   * Returns true if this handler had an active drag.
   */
  onMouseUp: () => Promise<boolean>
}

/**
 * Composable for entity drag-and-drop on map canvas.
 *
 * Provides generic drag handling for any map entity (tokens, lights, traps, POIs).
 * Handles coordinate conversion between mouse, viewport, and entity coordinate systems.
 *
 * @example
 * ```ts
 * // For pixel-based entities (tokens, lights)
 * const tokenDrag = useEntityDragDrop<Token>({
 *   viewportRef, baseScale, zoom, panX, panY, uvttGridSize,
 *   entities: tokens,
 *   coordType: 'pixel',
 *   coordFields: { x: 'x', y: 'y' },
 *   savePosition: async (token) => {
 *     await invoke('update_token_position', { id: token.id, gridX, gridY })
 *   },
 *   reload: loadTokens,
 *   snapToGrid: true
 * })
 *
 * // For grid-based entities (traps, POIs)
 * const trapDrag = useEntityDragDrop<MapTrap>({
 *   viewportRef, baseScale, zoom, panX, panY, uvttGridSize,
 *   entities: mapTraps,
 *   coordType: 'grid',
 *   coordFields: { x: 'grid_x', y: 'grid_y' },
 *   savePosition: async (trap) => {
 *     await invoke('move_map_trap', { id: trap.id, gridX: trap.grid_x, gridY: trap.grid_y })
 *   },
 *   reload: loadMapTraps
 * })
 *
 * // In the main mouse handlers:
 * function onMouseMove(event: MouseEvent) {
 *   if (tokenDrag.onMouseMove(event)) return
 *   if (trapDrag.onMouseMove(event)) return
 *   // Handle other interactions...
 * }
 * ```
 */
export function useEntityDragDrop<T extends { id: string | number }>(
  options: EntityDragOptions<T>
): EntityDragReturn<T> {
  const {
    viewportRef,
    baseScale,
    zoom,
    panX,
    panY,
    uvttGridSize,
    entities,
    coordType,
    coordFields,
    savePosition,
    reload,
    snapToGrid = false,
    stopPropagation = true
  } = options

  const draggingEntity = ref<T | null>(null) as Ref<T | null>
  const dragOffsetX = ref(0)
  const dragOffsetY = ref(0)

  /**
   * Get pixel coordinates for an entity.
   * Grid-based entities are converted to pixel coords (center of cell).
   */
  function getEntityPixelCoords(entity: T): { x: number; y: number } {
    const xValue = entity[coordFields.x] as number
    const yValue = entity[coordFields.y] as number

    if (coordType === 'grid') {
      // Grid coords to pixel coords (center of cell)
      return {
        x: (xValue + 0.5) * uvttGridSize.value,
        y: (yValue + 0.5) * uvttGridSize.value
      }
    }
    return { x: xValue, y: yValue }
  }

  /**
   * Start dragging an entity.
   */
  function onMouseDown(event: MouseEvent, entity: T): void {
    // Only left-click to drag
    if (event.button !== 0) return

    event.preventDefault()
    if (stopPropagation) {
      event.stopPropagation()
    }

    draggingEntity.value = entity

    // Calculate offset from mouse to entity center
    if (viewportRef.value) {
      const rect = viewportRef.value.getBoundingClientRect()
      const mouseX = event.clientX - rect.left
      const mouseY = event.clientY - rect.top

      // Entity position in pixel coordinates
      const entityPixel = getEntityPixelCoords(entity)

      // Entity position in viewport coordinates
      const entityViewportX = entityPixel.x * baseScale.value + panX.value
      const entityViewportY = entityPixel.y * baseScale.value + panY.value

      // Store offset so entity moves smoothly from where we clicked
      dragOffsetX.value = mouseX - entityViewportX * zoom.value
      dragOffsetY.value = mouseY - entityViewportY * zoom.value
    }
  }

  /**
   * Handle mouse move during drag.
   * Updates entity position locally.
   * Returns true if this handler consumed the event.
   */
  function onMouseMove(event: MouseEvent): boolean {
    if (!draggingEntity.value || !viewportRef.value) {
      return false
    }

    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Convert to image coordinates
    const effectiveScale = baseScale.value * zoom.value
    const imageX = (mouseX - dragOffsetX.value - panX.value * zoom.value) / effectiveScale
    const imageY = (mouseY - dragOffsetY.value - panY.value * zoom.value) / effectiveScale

    // Find entity in array and update
    const entityIndex = entities.value.findIndex(e => e.id === draggingEntity.value?.id)
    if (entityIndex !== -1) {
      if (coordType === 'grid') {
        // Convert to grid coordinates for grid-based entities
        const gridX = Math.floor(imageX / uvttGridSize.value)
        const gridY = Math.floor(imageY / uvttGridSize.value)
        entities.value[entityIndex] = {
          ...entities.value[entityIndex],
          [coordFields.x]: gridX,
          [coordFields.y]: gridY
        }
      } else {
        // Store pixel coordinates directly for pixel-based entities
        entities.value[entityIndex] = {
          ...entities.value[entityIndex],
          [coordFields.x]: imageX,
          [coordFields.y]: imageY
        }
      }
    }

    return true
  }

  /**
   * Handle mouse up to save position.
   * Returns true if this handler had an active drag.
   */
  async function onMouseUp(): Promise<boolean> {
    if (!draggingEntity.value) {
      return false
    }

    const entity = entities.value.find(e => e.id === draggingEntity.value?.id)
    if (entity) {
      // Apply grid snapping for pixel-based entities if configured
      if (coordType === 'pixel' && snapToGrid) {
        const gridSize = uvttGridSize.value
        const currentX = entity[coordFields.x] as number
        const currentY = entity[coordFields.y] as number

        // Snap to grid center
        const snappedX = Math.round(currentX / gridSize) * gridSize + gridSize / 2
        const snappedY = Math.round(currentY / gridSize) * gridSize + gridSize / 2

        // Update local state with snapped position
        const entityIndex = entities.value.findIndex(e => e.id === entity.id)
        if (entityIndex !== -1) {
          entities.value[entityIndex] = {
            ...entities.value[entityIndex],
            [coordFields.x]: snappedX,
            [coordFields.y]: snappedY
          }
        }
      }

      // Save to backend
      try {
        // Re-fetch entity after potential snap update
        const updatedEntity = entities.value.find(e => e.id === draggingEntity.value?.id)
        if (updatedEntity) {
          await savePosition(updatedEntity)
        }
      } catch (e) {
        console.error('Failed to save entity position:', e)
        // Reload entities to restore original position on error
        await reload()
      }
    }

    draggingEntity.value = null
    return true
  }

  return {
    draggingEntity,
    onMouseDown,
    onMouseMove,
    onMouseUp
  }
}
