<template>
  <div class="logs-section">
    <!-- Loading state -->
    <div v-if="loading" class="loading-container">
      <div class="spinner"></div>
      <p class="loading-text">Loading log files...</p>
    </div>
    
    <!-- Error state -->
    <div v-else-if="error" class="error-container">
      <p class="error-message">{{ error }}</p>
      <button @click="refreshLogFiles" class="retry-button">
        Try Again
      </button>
    </div>
    
    <!-- Log files list -->
    <div v-else class="logs-container">
      <div class="logs-header">
        <h3 class="logs-title">Log Files</h3>
        <button @click="refreshLogFiles" class="refresh-button" title="Refresh">
          <svg 
            class="refresh-icon"
            width="16" 
            height="16" 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            stroke-width="2" 
            stroke-linecap="round" 
            stroke-linejoin="round"
          >
            <polyline points="23 4 23 10 17 10"></polyline>
            <polyline points="1 20 1 14 7 14"></polyline>
            <path d="m3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
          </svg>
        </button>
      </div>
      
      <!-- Application Logs Section -->
      <div class="log-section">
        <h4 class="section-title">Application Logs</h4>
        <div v-if="applicationLogs.length === 0" class="no-logs">
          <p>No application log files found</p>
        </div>
        <div v-else class="log-files-list">
          <div 
            v-for="file in applicationLogs" 
            :key="file.name"
            class="log-file-item"
          >
            <div class="file-info">
              <div class="file-name">
                {{ file.name }}
                <span v-if="file.is_current" class="current-badge">Current</span>
              </div>
              <div class="file-path">
                <span class="path-text">{{ file.full_path }}</span>
                <button @click="copyPath(file.full_path)" class="copy-path-button" title="Copy path">
                  📋
                </button>
              </div>
              <div class="file-details">
                <span class="file-size">{{ formatFileSize(file.size) }}</span>
                <span class="file-modified">{{ file.modified }}</span>
              </div>
            </div>
            <div class="file-actions">
              <button @click="openLogViewer(file.name)" class="view-button">
                View
              </button>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Chat Logs Section -->
      <div class="log-section">
        <h4 class="section-title">Chat Logs</h4>
        <div v-if="chatLogs.length === 0" class="no-logs">
          <p>No chat log files found</p>
        </div>
        <div v-else class="log-files-list">
          <div 
            v-for="file in chatLogs" 
            :key="file.name"
            class="log-file-item"
          >
            <div class="file-info">
              <div class="file-name">
                {{ file.name }}
              </div>
              <div class="file-path">
                <span class="path-text">{{ file.full_path }}</span>
                <button @click="copyPath(file.full_path)" class="copy-path-button" title="Copy path">
                  📋
                </button>
              </div>
              <div class="file-details">
                <span class="file-size">{{ formatFileSize(file.size) }}</span>
                <span class="file-modified">{{ file.modified }}</span>
              </div>
            </div>
            <div class="file-actions">
              <button @click="openLogViewer(file.name)" class="view-button">
                View
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface LogFileInfo {
  name: string
  full_path: string
  size: number
  modified: string
  is_current: boolean
}

interface LogFilesData {
  application_logs: LogFileInfo[]
  chat_logs: LogFileInfo[]
}

interface LogFilesResponse {
  success: boolean
  data?: LogFilesData
  error?: string
}

// Component state
const applicationLogs = ref<LogFileInfo[]>([])
const chatLogs = ref<LogFileInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

// Load log files on component mount
onMounted(() => {
  loadLogFiles()
})

// Load log files from backend
const loadLogFiles = async () => {
  try {
    loading.value = true
    error.value = null
    
    const response = await invoke<LogFilesResponse>('list_log_files')
    
    if (response.success && response.data) {
      applicationLogs.value = response.data.application_logs
      chatLogs.value = response.data.chat_logs
    } else {
      error.value = response.error || 'Failed to load log files'
    }
  } catch (err) {
    console.error('Failed to load log files:', err)
    error.value = err instanceof Error ? err.message : 'Unknown error occurred'
  } finally {
    loading.value = false
  }
}

// Refresh log files
const refreshLogFiles = () => {
  loadLogFiles()
}

// Open log viewer window
const openLogViewer = async (fileName: string) => {
  try {
    await invoke('open_log_viewer_window', { fileName })
  } catch (err) {
    console.error('Failed to open log viewer:', err)
  }
}

// Copy path to clipboard
const copyPath = async (path: string) => {
  try {
    await navigator.clipboard.writeText(path)
  } catch (err) {
    console.error('Failed to copy path:', err)
  }
}

// Format file size for display
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

</script>

<style scoped>
.logs-section {
  width: 100%;
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl);
  gap: var(--spacing-md);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top: 3px solid var(--color-primary-500);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-text {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.error-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-xl);
  text-align: center;
}

.error-message {
  color: var(--color-error, #dc2626);
  font-size: 0.875rem;
}

.retry-button {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.retry-button:hover {
  background: var(--color-primary-600);
}

.logs-container {
  width: 100%;
}

.log-section {
  margin-bottom: var(--spacing-xl);
}

.log-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--spacing-md) 0;
  padding-bottom: var(--spacing-xs);
  border-bottom: 1px solid var(--color-border);
}

.logs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--spacing-lg);
}

.logs-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.refresh-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.refresh-button:hover {
  background: var(--color-surface-variant);
  border-color: var(--color-border-hover);
  color: var(--color-text);
}

.refresh-icon {
  width: 16px;
  height: 16px;
}

.no-logs {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-secondary);
}

.log-files-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.log-file-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-lg);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  min-height: 80px;
}

.log-file-item:hover {
  background: var(--color-gray-100);
  border-color: var(--color-border-hover);
}

.theme-dark .log-file-item:hover {
  background: var(--color-gray-700);
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
  font-size: 0.95rem;
}

.file-path {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
}

.path-text {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  font-family: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  word-break: break-all;
}

.copy-path-button {
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.copy-path-button:hover {
  background: var(--color-surface-variant);
  color: var(--color-text);
}

.current-badge {
  display: inline-block;
  padding: 0.125rem 0.375rem;
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  font-size: 0.75rem;
  font-weight: 500;
  border-radius: var(--radius-sm);
}

.theme-dark .current-badge {
  background: var(--color-primary-900);
  color: var(--color-primary-300);
}

.file-details {
  display: flex;
  gap: var(--spacing-md);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.file-actions {
  flex-shrink: 0;
}

.view-button {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.view-button:hover {
  background: var(--color-primary-600);
}
</style>