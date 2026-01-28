import { ref, computed, type Ref, type ComputedRef } from 'vue'

export interface PlayerViewportOptions {
  /** Minimum zoom level (default: 0.25) */
  minZoom?: number
  /** Maximum zoom level (default: 5) */
  maxZoom?: number
  /** Zoom step multiplier for scroll (default: 0.1) */
  zoomStep?: number
}

export interface PlayerViewportReturn {
  /** Current pan X offset in pixels */
  panX: Ref<number>
  /** Current pan Y offset in pixels */
  panY: Ref<number>
  /** Current zoom level (1 = 100%) */
  zoom: Ref<number>
  /** Whether currently panning */
  isPanning: Ref<boolean>
  /** Base display scale (fit-to-screen) */
  displayScale: Ref<number>
  /** Natural image width */
  imageNaturalWidth: Ref<number>
  /** Natural image height */
  imageNaturalHeight: Ref<number>
  /** Combined CSS transform for the viewport */
  transform: ComputedRef<{ transform: string; transformOrigin: string }>
  /** Handle mouse down for pan start */
  handleMouseDown: (event: MouseEvent) => void
  /** Handle mouse move for panning */
  handleMouseMove: (event: MouseEvent) => void
  /** Handle mouse up to end panning */
  handleMouseUp: () => void
  /** Handle wheel for zooming */
  handleWheel: (event: WheelEvent) => void
  /** Update display scale based on image and viewport dimensions */
  updateDisplayScale: (imageEl: HTMLImageElement | null) => void
  /** Reset viewport to default (fit to screen) */
  reset: () => void
}

/**
 * Composable for player-controlled viewport pan/zoom.
 *
 * Provides independent pan and zoom controls for the player display,
 * with fit-to-screen base scaling and mouse-based interaction.
 *
 * @example
 * ```ts
 * const viewport = usePlayerViewport()
 *
 * // In template:
 * // <div @mousedown="viewport.handleMouseDown" @wheel.prevent="viewport.handleWheel" ...>
 * //   <div :style="viewport.transform.value">...</div>
 * // </div>
 *
 * // On image load:
 * // viewport.updateDisplayScale(imageRef.value)
 * ```
 */
export function usePlayerViewport(options: PlayerViewportOptions = {}): PlayerViewportReturn {
  const {
    minZoom = 0.25,
    maxZoom = 5,
    zoomStep = 0.1
  } = options

  // Pan state
  const panX = ref(0)
  const panY = ref(0)
  const isPanning = ref(false)
  const panStart = ref({ x: 0, y: 0 })

  // Zoom state
  const zoom = ref(1)

  // Display scale (fit-to-screen base scale)
  const displayScale = ref(1)
  const imageNaturalWidth = ref(0)
  const imageNaturalHeight = ref(0)

  // Combined transform
  const transform = computed(() => {
    const finalScale = displayScale.value * zoom.value
    return {
      transform: `translate(${panX.value}px, ${panY.value}px) scale(${finalScale})`,
      transformOrigin: 'center center'
    }
  })

  /**
   * Handle mouse down to start panning
   */
  function handleMouseDown(event: MouseEvent): void {
    if (event.button === 0) {
      event.preventDefault()
      isPanning.value = true
      panStart.value = {
        x: event.clientX - panX.value,
        y: event.clientY - panY.value
      }
    }
  }

  /**
   * Handle mouse move for panning
   */
  function handleMouseMove(event: MouseEvent): void {
    if (isPanning.value) {
      panX.value = event.clientX - panStart.value.x
      panY.value = event.clientY - panStart.value.y
    }
  }

  /**
   * Handle mouse up to end panning
   */
  function handleMouseUp(): void {
    isPanning.value = false
  }

  /**
   * Handle wheel for zooming toward mouse position
   */
  function handleWheel(event: WheelEvent): void {
    const delta = event.deltaY > 0 ? (1 - zoomStep) : (1 + zoomStep)
    const newZoom = Math.max(minZoom, Math.min(maxZoom, zoom.value * delta))

    // Zoom toward mouse position
    const target = event.currentTarget as HTMLElement
    if (target) {
      const rect = target.getBoundingClientRect()
      const mouseX = event.clientX - rect.left - rect.width / 2
      const mouseY = event.clientY - rect.top - rect.height / 2

      const zoomRatio = newZoom / zoom.value
      panX.value = mouseX - (mouseX - panX.value) * zoomRatio
      panY.value = mouseY - (mouseY - panY.value) * zoomRatio
    }

    zoom.value = newZoom
  }

  /**
   * Update display scale to fit image in viewport
   */
  function updateDisplayScale(imageEl: HTMLImageElement | null): void {
    if (!imageEl) return

    const naturalWidth = imageEl.naturalWidth
    const naturalHeight = imageEl.naturalHeight

    if (naturalWidth === 0 || naturalHeight === 0) return

    // Store dimensions
    imageNaturalWidth.value = naturalWidth
    imageNaturalHeight.value = naturalHeight

    // Get viewport dimensions
    const viewportWidth = window.innerWidth
    const viewportHeight = window.innerHeight

    // Calculate scale to fill (cover) - use larger scale for no black bars
    const scaleX = viewportWidth / naturalWidth
    const scaleY = viewportHeight / naturalHeight
    displayScale.value = Math.max(scaleX, scaleY)

    console.log('usePlayerViewport: Updated display scale to', displayScale.value,
      `(natural: ${naturalWidth}x${naturalHeight}, viewport: ${viewportWidth}x${viewportHeight})`)
  }

  /**
   * Reset viewport to default fit-to-screen
   */
  function reset(): void {
    panX.value = 0
    panY.value = 0
    zoom.value = 1
  }

  return {
    panX,
    panY,
    zoom,
    isPanning,
    displayScale,
    imageNaturalWidth,
    imageNaturalHeight,
    transform,
    handleMouseDown,
    handleMouseMove,
    handleMouseUp,
    handleWheel,
    updateDisplayScale,
    reset
  }
}
