/**
 * Composable for controlling the DM map window.
 *
 * The DM map window is a separate Tauri window that displays the battle map
 * while the main window stays on the module dashboard for reference.
 */

import { ref, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Window state
const isWindowOpen = ref(false)
const currentModuleId = ref<string | null>(null)
const currentCampaignId = ref<string | null>(null)

/**
 * Check if the DM map window is currently open
 */
async function checkWindowOpen(): Promise<boolean> {
  try {
    const open = await invoke<boolean>('is_dm_map_open')
    isWindowOpen.value = open
    return open
  } catch (err) {
    console.error('Failed to check DM map window status:', err)
    return false
  }
}

/**
 * Open the DM map window for a specific module
 */
async function openWindow(moduleId: string, campaignId: string): Promise<void> {
  try {
    await invoke('open_dm_map_window', { moduleId, campaignId })
    isWindowOpen.value = true
    currentModuleId.value = moduleId
    currentCampaignId.value = campaignId
  } catch (err) {
    console.error('Failed to open DM map window:', err)
    throw err
  }
}

/**
 * Close the DM map window
 */
async function closeWindow(): Promise<void> {
  try {
    await invoke('close_dm_map_window')
    isWindowOpen.value = false
    currentModuleId.value = null
    currentCampaignId.value = null
  } catch (err) {
    console.error('Failed to close DM map window:', err)
    throw err
  }
}

/**
 * Toggle fullscreen mode on the DM map window
 */
async function toggleFullscreen(): Promise<boolean> {
  try {
    return await invoke<boolean>('toggle_dm_map_fullscreen')
  } catch (err) {
    console.error('Failed to toggle DM map fullscreen:', err)
    throw err
  }
}

/**
 * Composable for DM map window control
 */
export function useDmMapWindow() {
  return {
    // State (readonly to prevent external mutation)
    isWindowOpen: readonly(isWindowOpen),
    currentModuleId: readonly(currentModuleId),
    currentCampaignId: readonly(currentCampaignId),

    // Methods
    checkWindowOpen,
    openWindow,
    closeWindow,
    toggleFullscreen
  }
}

export default useDmMapWindow
