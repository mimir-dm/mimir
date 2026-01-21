/**
 * App Settings Store
 *
 * Manages general application preferences like AI assistant visibility and MCP server.
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AppSettings {
  ai_assistant_enabled: boolean
  mcp_server_enabled: boolean
}

export interface McpServerStatus {
  running: boolean
}

export const useAppSettingsStore = defineStore('appSettings', () => {
  // State
  const aiAssistantEnabled = ref(false)
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
      aiAssistantEnabled.value = settings.ai_assistant_enabled
      mcpServerEnabled.value = settings.mcp_server_enabled
      isLoaded.value = true

      // Also get current MCP server status
      await refreshMcpServerStatus()
    } catch (error) {
      console.error('Failed to load app settings:', error)
      // Use defaults on error
      aiAssistantEnabled.value = false
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
          ai_assistant_enabled: aiAssistantEnabled.value,
          mcp_server_enabled: mcpServerEnabled.value
        }
      })
    } catch (error) {
      console.error('Failed to save app settings:', error)
      throw error
    }
  }

  // Toggle AI assistant and save
  async function setAiAssistantEnabled(enabled: boolean) {
    aiAssistantEnabled.value = enabled
    await saveSettings()
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
    aiAssistantEnabled,
    mcpServerEnabled,
    mcpServerRunning,
    mcpActionPending,
    isLoading,
    isLoaded,
    // Actions
    loadSettings,
    saveSettings,
    setAiAssistantEnabled,
    setMcpServerEnabled,
    refreshMcpServerStatus,
    startMcpServer,
    stopMcpServer,
    restartMcpServer
  }
})
