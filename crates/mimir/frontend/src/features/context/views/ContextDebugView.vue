<template>
  <div class="context-debug h-screen overflow-hidden flex flex-col bg-gray-900 text-gray-100">
    <!-- Header -->
    <div class="flex-none border-b border-gray-700 bg-gray-800 px-4 py-3">
      <div class="flex justify-between items-center">
        <h1 class="text-xl font-semibold">Context Debug Window</h1>
        <div class="flex items-center gap-4">
          <div v-if="contextUsage" class="text-sm text-gray-400">
            Context: {{ formatTokenCount(contextUsage) }} / 128k tokens
          </div>
          <button
            @click="clearContext"
            class="px-3 py-1 text-sm bg-red-600 hover:bg-red-700 text-white rounded transition-colors"
          >
            Clear Context
          </button>
          <button
            @click="refreshContext"
            class="px-3 py-1 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded transition-colors"
          >
            Refresh
          </button>
        </div>
      </div>
    </div>

    <!-- Context Display -->
    <div class="flex-1 overflow-y-auto p-4 font-mono text-sm">
      <div class="space-y-6">
        <!-- Campaign Context -->
        <section v-if="campaign && Object.keys(campaign).length > 0">
          <h2 class="text-green-400 font-bold mb-2">Campaign Context</h2>
          <pre class="bg-gray-800 rounded p-3 overflow-x-auto">{{ JSON.stringify(campaign, null, 2) }}</pre>
        </section>

        <!-- Module Context -->
        <section v-if="module && Object.keys(module).length > 0">
          <h2 class="text-blue-400 font-bold mb-2">Module Context</h2>
          <pre class="bg-gray-800 rounded p-3 overflow-x-auto">{{ JSON.stringify(module, null, 2) }}</pre>
        </section>

        <!-- Session Context -->
        <section v-if="session && Object.keys(session).length > 0">
          <h2 class="text-purple-400 font-bold mb-2">Session Context</h2>
          <pre class="bg-gray-800 rounded p-3 overflow-x-auto">{{ JSON.stringify(session, null, 2) }}</pre>
        </section>

        <!-- Reference Context -->
        <section v-if="reference && Object.keys(reference).length > 0">
          <h2 class="text-yellow-400 font-bold mb-2">Reference Context</h2>
          <pre class="bg-gray-800 rounded p-3 overflow-x-auto">{{ JSON.stringify(reference, null, 2) }}</pre>
        </section>

        <!-- Windows -->
        <section v-if="windowsArray.length > 0">
          <h2 class="text-cyan-400 font-bold mb-2">Active Windows</h2>
          <div class="bg-gray-800 rounded p-3">
            <div v-for="window in windowsArray" :key="window.id" class="mb-2">
              <span class="text-cyan-300">{{ window.type }}</span>:
              <span class="text-gray-400">{{ window.title }}</span>
              <span v-if="window.focused" class="ml-2 text-green-400">(focused)</span>
            </div>
          </div>
        </section>

        <!-- Recent Actions -->
        <section v-if="recentActions.length > 0">
          <h2 class="text-orange-400 font-bold mb-2">Recent Actions</h2>
          <div class="bg-gray-800 rounded p-3 space-y-2">
            <div v-for="action in recentActions" :key="action.timestamp" class="border-b border-gray-700 pb-2 last:border-0">
              <div class="flex justify-between">
                <span class="text-orange-300">{{ action.type }}</span>
                <span class="text-gray-500 text-xs">{{ formatTimestamp(action.timestamp) }}</span>
              </div>
              <div class="text-gray-400 mt-1">{{ action.description }}</div>
              <pre v-if="action.data" class="text-xs mt-1 text-gray-500">{{ JSON.stringify(action.data, null, 2) }}</pre>
            </div>
          </div>
        </section>

        <!-- Full Context JSON -->
        <section>
          <h2 class="text-pink-400 font-bold mb-2">Full Context JSON</h2>
          <pre class="bg-gray-800 rounded p-3 overflow-x-auto">{{ JSON.stringify(fullContext, null, 2) }}</pre>
        </section>

        <!-- LLM Context -->
        <section>
          <h2 class="text-indigo-400 font-bold mb-2">LLM Context View</h2>
          <pre class="bg-gray-800 rounded p-3 whitespace-pre-wrap">{{ llmContext }}</pre>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useSharedContextStore } from '@/stores/sharedContext'
import { invoke } from '@tauri-apps/api/core'

const contextStore = useSharedContextStore()

// Computed properties from store
const campaign = computed(() => contextStore.campaign)
const module = computed(() => contextStore.module)
const session = computed(() => contextStore.session)
const reference = computed(() => contextStore.reference)
const recentActions = computed(() => contextStore.recentActions)
const contextUsage = computed(() => contextStore.contextUsage)
const fullContext = computed(() => contextStore.fullContext)

// Convert windows Map to array for display
const windowsArray = computed(() => {
  return Array.from(contextStore.windows.values())
})

// LLM context view
const llmContext = ref<string>('')

// Format helpers
const formatTokenCount = (count: number) => {
  if (count > 1000) {
    return `${(count / 1000).toFixed(1)}k`
  }
  return count.toString()
}

const formatTimestamp = (timestamp: number) => {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

// Actions
const clearContext = async () => {
  await contextStore.clearContext()
  await loadLLMContext()
}

const refreshContext = async () => {
  await contextStore.loadFullContext()
  await loadLLMContext()
}

const loadLLMContext = async () => {
  try {
    llmContext.value = await invoke<string>('get_context_for_llm')
  } catch (error) {
    console.error('Failed to load LLM context:', error)
    llmContext.value = 'Error loading LLM context'
  }
}

// Register this window
onMounted(async () => {
  // Set window ID for this window
  (window as any).__TAURI_WINDOW_ID__ = 'context-debug'
  
  // Register window with context service
  await contextStore.registerWindow({
    id: 'context-debug',
    type: 'debug',
    title: 'Context Debug',
    focused: true
  })
  
  // Load initial context
  await refreshContext()
  
  // Set up periodic refresh
  const refreshInterval = setInterval(async () => {
    await loadLLMContext()
  }, 2000) // Refresh every 2 seconds
  
  // Clean up on unmount
  onUnmounted(() => {
    clearInterval(refreshInterval)
    contextStore.unregisterWindow('context-debug')
  })
})

import { onUnmounted } from 'vue'
</script>

<style scoped>
.context-debug {
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Menlo, Consolas, 'Courier New', monospace;
}

pre {
  white-space: pre;
  word-wrap: break-word;
  overflow-wrap: break-word;
}
</style>