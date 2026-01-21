/**
 * Composable for controlling the player display window.
 *
 * Provides reactive state and methods for managing the player display,
 * including opening/closing the window, sending maps, and controlling viewport.
 */

import { ref, readonly, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Display state
const isDisplayOpen = ref(false)
const currentMapId = ref<number | null>(null)
const isBlackout = ref(false)
const viewportState = ref({
  x: 0,
  y: 0,
  zoom: 1
})

/**
 * Check if the player display window is currently open
 */
async function checkDisplayOpen(): Promise<boolean> {
  try {
    const open = await invoke<boolean>('is_player_display_open')
    isDisplayOpen.value = open
    return open
  } catch (err) {
    console.error('Failed to check display status:', err)
    return false
  }
}

/**
 * Open the player display window
 */
async function openDisplay(): Promise<void> {
  try {
    await invoke('open_player_display_window')
    isDisplayOpen.value = true
  } catch (err) {
    console.error('Failed to open player display:', err)
    throw err
  }
}

/**
 * Close the player display window
 */
async function closeDisplay(): Promise<void> {
  try {
    await invoke('close_player_display_window')
    isDisplayOpen.value = false
    currentMapId.value = null
  } catch (err) {
    console.error('Failed to close player display:', err)
    throw err
  }
}

/**
 * Toggle the player display window open/closed
 */
async function toggleDisplay(): Promise<boolean> {
  if (isDisplayOpen.value) {
    await closeDisplay()
    return false
  } else {
    await openDisplay()
    return true
  }
}

/**
 * Toggle fullscreen mode on the player display
 */
async function toggleFullscreen(): Promise<boolean> {
  try {
    return await invoke<boolean>('toggle_player_display_fullscreen')
  } catch (err) {
    console.error('Failed to toggle fullscreen:', err)
    throw err
  }
}

/**
 * Send a map to the player display
 */
async function sendMapToDisplay(
  mapId: number,
  gridType: string = 'none',
  gridSizePx: number | null = null,
  gridOffsetX: number = 0,
  gridOffsetY: number = 0,
  ambientLight: string | null = null,
  mapWidth: number | null = null,
  mapHeight: number | null = null
): Promise<void> {
  try {
    await invoke('send_map_to_display', {
      mapId,
      gridType,
      gridSizePx,
      gridOffsetX,
      gridOffsetY,
      ambientLight,
      mapWidth,
      mapHeight
    })
    currentMapId.value = mapId
  } catch (err) {
    console.error('Failed to send map to display:', err)
    throw err
  }
}

/**
 * Update the viewport on the player display (pan/zoom)
 */
async function updateViewport(x: number, y: number, zoom: number): Promise<void> {
  try {
    await invoke('update_display_viewport', { x, y, zoom })
    viewportState.value = { x, y, zoom }
  } catch (err) {
    console.error('Failed to update viewport:', err)
    throw err
  }
}

/**
 * Toggle blackout mode on the player display
 */
async function toggleBlackout(): Promise<void> {
  try {
    const newState = !isBlackout.value
    await invoke('toggle_display_blackout', { isBlackout: newState })
    isBlackout.value = newState
  } catch (err) {
    console.error('Failed to toggle blackout:', err)
    throw err
  }
}

/**
 * Set blackout mode explicitly
 */
async function setBlackout(blackout: boolean): Promise<void> {
  try {
    await invoke('toggle_display_blackout', { isBlackout: blackout })
    isBlackout.value = blackout
  } catch (err) {
    console.error('Failed to set blackout:', err)
    throw err
  }
}

/**
 * Composable for player display control
 */
export function usePlayerDisplay() {
  return {
    // State (readonly to prevent external mutation)
    isDisplayOpen: readonly(isDisplayOpen),
    currentMapId: readonly(currentMapId),
    isBlackout: readonly(isBlackout),
    viewportState: readonly(viewportState),

    // Methods
    checkDisplayOpen,
    openDisplay,
    closeDisplay,
    toggleDisplay,
    toggleFullscreen,
    sendMapToDisplay,
    updateViewport,
    toggleBlackout,
    setBlackout
  }
}

export default usePlayerDisplay
