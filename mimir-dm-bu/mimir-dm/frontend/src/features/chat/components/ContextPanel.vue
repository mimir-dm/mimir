<template>
  <div class="context-panel" :class="{ collapsed: isCollapsed }">
    <!-- Collapsed state: single line -->
    <div v-if="isCollapsed" class="collapsed-content" @click="toggleCollapse">
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center gap-4 text-sm">
          <span v-if="campaign?.name" class="campaign-text">
            Campaign: {{ campaign.name }}
          </span>
          <span v-if="module?.name" class="module-text">
            Module: {{ module.name }}
          </span>
          <span v-if="contextUsage" class="context-text">
            Context: {{ formatTokenCount(contextUsage) }}/262k
          </span>
        </div>
        <button class="expand-button">
          ▼ Expand Context
        </button>
      </div>
    </div>
    
    <!-- Expanded state: full panel -->
    <div v-else class="expanded-content">
      <div class="panel-header">
        <h3 class="text-lg font-semibold">Context Information</h3>
        <button @click="toggleCollapse" class="text-gray-400 hover:text-gray-200">
          ▲ Collapse
        </button>
      </div>
      
      <div class="panel-body">
        <div class="grid grid-cols-3 gap-4">
          <!-- Campaign Info -->
          <div v-if="campaign && Object.keys(campaign).length > 0">
            <h4 class="text-green-400 font-semibold mb-1">Campaign</h4>
            <div class="text-sm text-gray-300">
              <div>{{ campaign.name }}</div>
              <div v-if="campaign.currentStage" class="text-gray-400">
                Stage: {{ campaign.currentStage }}
              </div>
            </div>
          </div>
          
          <!-- Module Info -->
          <div v-if="module && Object.keys(module).length > 0">
            <h4 class="text-blue-400 font-semibold mb-1">Module</h4>
            <div class="text-sm text-gray-300">
              <div>{{ module.name }}</div>
              <div v-if="module.currentStage" class="text-gray-400">
                Stage: {{ module.currentStage }}
              </div>
            </div>
          </div>
          
          <!-- Session Info -->
          <div v-if="session && Object.keys(session).length > 0">
            <h4 class="text-purple-400 font-semibold mb-1">Session</h4>
            <div class="text-sm text-gray-300">
              <div>{{ session.name }}</div>
              <div v-if="session.status" class="text-gray-400">
                Status: {{ session.status }}
              </div>
            </div>
          </div>
        </div>
        
        <!-- Recent Actions -->
        <div v-if="recentActions.length > 0" class="mt-4">
          <h4 class="text-orange-400 font-semibold mb-2">Recent Actions</h4>
          <div class="space-y-1 max-h-20 overflow-y-auto">
            <div
              v-for="action in recentActions.slice(0, 3)"
              :key="action.timestamp"
              class="text-sm text-gray-400"
            >
              {{ action.type }}: {{ action.description }}
            </div>
          </div>
        </div>
        
        <!-- Context Usage -->
        <div class="mt-4 flex items-center justify-between">
          <span class="text-sm text-gray-400">
            Context Usage: {{ formatTokenCount(contextUsage) }} / 262k tokens
          </span>
          <div class="flex gap-2">
            <button
              @click="refreshContext"
              class="px-2 py-1 text-xs bg-blue-600 hover:bg-blue-700 text-white rounded"
            >
              Refresh
            </button>
            <button
              @click="clearContext"
              class="px-2 py-1 text-xs bg-red-600 hover:bg-red-700 text-white rounded"
            >
              Clear
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSharedContextStore } from '@/stores/sharedContext'
import { invoke } from '@tauri-apps/api/core'

const contextStore = useSharedContextStore()

// Props
const props = defineProps<{
  startCollapsed?: boolean
}>()

// State
const isCollapsed = ref(props.startCollapsed ?? true)

// Computed from store
const campaign = computed(() => contextStore.campaign)
const module = computed(() => contextStore.module)
const session = computed(() => contextStore.session)
const recentActions = computed(() => contextStore.recentActions)
const contextUsage = computed(() => contextStore.contextUsage)

// Methods
const toggleCollapse = () => {
  isCollapsed.value = !isCollapsed.value
}

const formatTokenCount = (count: number) => {
  if (count > 1000) {
    return `${(count / 1000).toFixed(1)}k`
  }
  return count.toString()
}

const refreshContext = async () => {
  await contextStore.loadFullContext()
}

const clearContext = async () => {
  await contextStore.clearContext()
}
</script>

<style scoped>
.context-panel {
  @apply border-b transition-all duration-300;
  background-color: var(--color-surface);
  border-color: var(--color-border);
}

.collapsed-content {
  @apply px-4 py-2 cursor-pointer;
}

.collapsed-content:hover {
  background-color: var(--color-surface-variant);
}

.expanded-content {
  @apply p-4;
}

.panel-header {
  @apply flex justify-between items-center mb-3;
}

.panel-body {
  @apply text-sm;
}

.context-panel.collapsed {
  height: auto;
}

.context-panel:not(.collapsed) {
  max-height: 200px;
  overflow-y: auto;
}

/* Theme-aware text colors */
.campaign-text {
  color: var(--color-success);
}

.module-text {
  color: var(--color-info);
}

.context-text {
  color: var(--color-text-secondary);
}

.expand-button {
  color: var(--color-text-secondary);
  transition: color 0.2s ease;
}

.expand-button:hover {
  color: var(--color-text);
}
</style>