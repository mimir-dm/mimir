<template>
  <div :class="['log-viewer-window', currentTheme]">
    <LogViewerHeader
      :file-name="currentFileName"
      :total-lines="logContent.total_lines"
      :last-updated="lastUpdated"
      :auto-scroll="autoScroll"
      :live-mode="liveMode"
      @toggle-auto-scroll="toggleAutoScroll"
      @toggle-live-mode="toggleLiveMode"
      @refresh="refreshLogs"
    />

    <LogSearchControls
      v-model:search-query="searchQuery"
      :log-levels="logLevels"
      :active-levels="activeLevels"
      :is-chat-log="isChatLog"
      @clear-search="clearSearch"
      @toggle-level="toggleLogLevel"
    />

    <LogContentDisplay
      ref="logContentDisplayRef"
      :loading="loading"
      :error="error"
      :filtered-lines="filteredLines"
      :search-query="searchQuery"
      :file-name="currentFileName"
      @scroll="onScroll"
      @retry="refreshLogs"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useThemeStore } from '@/stores/theme'
import LogViewerHeader from './LogViewer/LogViewerHeader.vue'
import LogSearchControls from './LogViewer/LogSearchControls.vue'
import LogContentDisplay from './LogViewer/LogContentDisplay.vue'

interface LogContent {
  lines: string[]
  total_lines: number
  position: number
}

interface LogContentResponse {
  success: boolean
  data?: LogContent
  error?: string
}

interface LogTailResponse {
  success: boolean
  data?: {
    new_lines: string[]
    new_position: number
  }
  error?: string
}

interface LogLine {
  content: string
  lineNumber: number
}

// Component state
const currentFileName = ref<string>('')
const logContent = ref<LogContent>({ lines: [], total_lines: 0, position: 0 })
const loading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const autoScroll = ref(true)
const liveMode = ref(true)
const lastUpdated = ref<Date | null>(null)
const logContentDisplayRef = ref<InstanceType<typeof LogContentDisplay> | null>(null)

// Log levels and filtering
const logLevels = ['ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE']
const activeLevels = ref(new Set(['ERROR', 'WARN', 'INFO']))

// Polling for live updates
let pollInterval: number | null = null

// Theme support
const themeStore = useThemeStore()
const currentTheme = computed(() => `theme-${themeStore.currentTheme}`)

// Detect if this is a chat log (ends with .log and is not mimir.log)
const isChatLog = computed(() => {
  return currentFileName.value.endsWith('.log') && !currentFileName.value.startsWith('mimir.log')
})

// Get current window and parse filename from label
onMounted(async () => {
  // Initialize theme
  await themeStore.loadThemes()
  themeStore.applyTheme()
  await themeStore.initThemeSync()
  
  const webview = getCurrentWebviewWindow()
  const label = webview.label
  
  // Extract filename from URL query parameter
  const urlParams = new URLSearchParams(window.location.search)
  const fileName = urlParams.get('file')
  
  if (fileName) {
    currentFileName.value = decodeURIComponent(fileName)
    await loadLogContent()
    
    if (liveMode.value) {
      startPolling()
    }
  } else {
    error.value = 'No log file specified'
    loading.value = false
  }
})

onUnmounted(() => {
  stopPolling()
})

// Watch live mode to start/stop polling
watch(liveMode, (newValue) => {
  if (newValue) {
    startPolling()
  } else {
    stopPolling()
  }
})

// Load log content from backend
const loadLogContent = async () => {
  if (!currentFileName.value) return
  
  try {
    loading.value = true
    error.value = null
    
    const response = await invoke<LogContentResponse>('read_log_file', {
      fileName: currentFileName.value,
      offset: 0,
      limit: 10000 // Load up to 10k lines initially
    })
    
    if (response.success && response.data) {
      logContent.value = response.data
      lastUpdated.value = new Date()
      
      if (autoScroll.value) {
        await nextTick()
        scrollToBottom()
      }
    } else {
      error.value = response.error || 'Failed to load log content'
    }
  } catch (err) {
    console.error('Failed to load log content:', err)
    error.value = err instanceof Error ? err.message : 'Unknown error occurred'
  } finally {
    loading.value = false
  }
}

// Start polling for new log content
const startPolling = () => {
  if (pollInterval) return
  
  pollInterval = window.setInterval(async () => {
    await pollForUpdates()
  }, 1000) // Poll every second
}

// Stop polling
const stopPolling = () => {
  if (pollInterval) {
    clearInterval(pollInterval)
    pollInterval = null
  }
}

// Poll for new log content
const pollForUpdates = async () => {
  if (!currentFileName.value || loading.value) return
  
  try {
    const response = await invoke<LogTailResponse>('tail_log_file', {
      fileName: currentFileName.value,
      lastPosition: logContent.value.position
    })
    
    if (response.success && response.data && response.data.new_lines.length > 0) {
      // Append new lines to existing content
      logContent.value.lines.push(...response.data.new_lines)
      logContent.value.total_lines += response.data.new_lines.length
      logContent.value.position = response.data.new_position
      lastUpdated.value = new Date()
      
      if (autoScroll.value) {
        await nextTick()
        scrollToBottom()
      }
    }
  } catch (err) {
    console.error('Failed to poll for updates:', err)
  }
}

// Refresh logs manually
const refreshLogs = () => {
  loadLogContent()
}

// Toggle auto-scroll
const toggleAutoScroll = () => {
  autoScroll.value = !autoScroll.value
  if (autoScroll.value) {
    nextTick(() => scrollToBottom())
  }
}

// Toggle live mode
const toggleLiveMode = () => {
  liveMode.value = !liveMode.value
}

// Scroll to bottom of log content
const scrollToBottom = () => {
  const contentRef = logContentDisplayRef.value?.logContentRef
  if (contentRef) {
    contentRef.scrollTop = contentRef.scrollHeight
  }
}

// Handle scroll events
const onScroll = (event: Event) => {
  const target = event.target as HTMLElement
  if (!target) return

  const { scrollTop, scrollHeight, clientHeight } = target
  const isAtBottom = scrollTop + clientHeight >= scrollHeight - 10 // 10px tolerance

  // Disable auto-scroll if user scrolls up manually
  if (!isAtBottom && autoScroll.value) {
    autoScroll.value = false
  }
}

// Process lines with line numbers
const processedLines = computed<LogLine[]>(() => {
  return logContent.value.lines.map((line, index) => ({
    content: line,
    lineNumber: index + 1
  }))
})

// Filter lines based on search and log levels
const filteredLines = computed<LogLine[]>(() => {
  let lines = processedLines.value
  
  // Filter by log levels (only for non-chat logs)
  if (!isChatLog.value && activeLevels.value.size < logLevels.length) {
    lines = lines.filter(line => {
      return Array.from(activeLevels.value).some(level => 
        line.content.includes(`[${level}]`) || 
        line.content.includes(level.toUpperCase())
      )
    })
  }
  
  // Filter by search query
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    lines = lines.filter(line => 
      line.content.toLowerCase().includes(query)
    )
  }
  
  return lines
})

// Toggle log level filter
const toggleLogLevel = (level: string) => {
  if (activeLevels.value.has(level)) {
    activeLevels.value.delete(level)
  } else {
    activeLevels.value.add(level)
  }
  // Create new Set to trigger reactivity
  activeLevels.value = new Set(activeLevels.value)
}

// Clear search
const clearSearch = () => {
  searchQuery.value = ''
}
</script>

<style scoped>
.log-viewer-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--color-surface);
  color: var(--color-text);
  font-family: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
}
</style>