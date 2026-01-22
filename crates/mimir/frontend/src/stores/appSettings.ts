/**
 * App Settings Store
 *
 * Manages general application preferences like MCP server settings.
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AppSettings {
  mcp_server_enabled: boolean
}

export interface McpServerStatus {
  running: boolean
}

export const useAppSettingsStore = defineStore('appSettings', () => {
  // State
  const mcpServerEnabled = ref(false)
  const mcpServerRunning = ref(false)
  const isLoading = ref(false)
  const isLoaded = ref(false)
  const mcpActionPending = ref(false)

  // Load settings from backend
  async function loadSettings() {
    if (isLoaded.value) return // Already loaded

    isLoading.value = true
    try {
      const settings = await invoke<AppSettings>('get_app_settings')
      mcpServerEnabled.value = settings.mcp_server_enabled
      isLoaded.value = true

      // Also get current MCP server status
      await refreshMcpServerStatus()
    } catch (error) {
      console.error('Failed to load app settings:', error)
      // Use defaults on error
      mcpServerEnabled.value = false
    } finally {
      isLoading.value = false
    }
  }

  // Save settings to backend
  async function saveSettings() {
    try {
      await invoke('save_app_settings', {
        settings: {
          mcp_server_enabled: mcpServerEnabled.value
        }
      })
    } catch (error) {
      console.error('Failed to save app settings:', error)
      throw error
    }
  }

  // Toggle MCP server auto-start and save
  async function setMcpServerEnabled(enabled: boolean) {
    mcpServerEnabled.value = enabled
    await saveSettings()

    // If enabling, start the server; if disabling, stop it
    if (enabled) {
      await startMcpServer()
    } else {
      await stopMcpServer()
    }
  }

  // Refresh MCP server status
  async function refreshMcpServerStatus() {
    try {
      const status = await invoke<McpServerStatus>('get_mcp_server_status')
      mcpServerRunning.value = status.running
    } catch (error) {
      console.error('Failed to get MCP server status:', error)
      mcpServerRunning.value = false
    }
  }

  // Start MCP server
  async function startMcpServer() {
    mcpActionPending.value = true
    try {
      const status = await invoke<McpServerStatus>('start_mcp_server')
      mcpServerRunning.value = status.running
    } catch (error) {
      console.error('Failed to start MCP server:', error)
      throw error
    } finally {
      mcpActionPending.value = false
    }
  }

  // Stop MCP server
  async function stopMcpServer() {
    mcpActionPending.value = true
    try {
      const status = await invoke<McpServerStatus>('stop_mcp_server')
      mcpServerRunning.value = status.running
    } catch (error) {
      console.error('Failed to stop MCP server:', error)
      throw error
    } finally {
      mcpActionPending.value = false
    }
  }

  // Restart MCP server
  async function restartMcpServer() {
    mcpActionPending.value = true
    try {
      const status = await invoke<McpServerStatus>('restart_mcp_server')
      mcpServerRunning.value = status.running
    } catch (error) {
      console.error('Failed to restart MCP server:', error)
      throw error
    } finally {
      mcpActionPending.value = false
    }
  }

  return {
    // State
    mcpServerEnabled,
    mcpServerRunning,
    mcpActionPending,
    isLoading,
    isLoaded,
    // Actions
    loadSettings,
    saveSettings,
    setMcpServerEnabled,
    refreshMcpServerStatus,
    startMcpServer,
    stopMcpServer,
    restartMcpServer
  }
})
