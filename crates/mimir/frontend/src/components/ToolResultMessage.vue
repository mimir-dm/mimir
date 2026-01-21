<template>
  <div class="tool-result-message">
    <div class="tool-header">
      <div class="tool-info">
        <span class="tool-icon">[Tool]</span>
        <span class="tool-name">{{ toolName }}</span>
        <span v-if="iteration" class="iteration-badge">{{ iteration }}</span>
      </div>
      <div class="status-indicator" :class="{ 'success': success, 'failure': !success }">
        {{ success ? '[✓]' : '[✗]' }}
      </div>
    </div>
    
    <div class="tool-result-content">
      <div v-if="formattedResult" class="structured-result">
        <div v-if="formattedResult.message" class="result-message">
          {{ formattedResult.message }}
        </div>
        <div v-if="formattedResult.details" class="result-details">
          <pre>{{ formattedResult.details }}</pre>
        </div>
      </div>
      <div v-else class="raw-result">
        <pre>{{ content }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  toolName: string
  content: string
  success: boolean
  iteration?: number
}>()

const formattedResult = computed(() => {
  try {
    const parsed = JSON.parse(props.content)
    
    // Handle structured responses
    if (parsed.status && parsed.message) {
      return {
        message: parsed.message,
        details: parsed.data ? JSON.stringify(parsed.data, null, 2) : null
      }
    }
    
    // Handle simple messages
    if (typeof parsed === 'string') {
      return { message: parsed }
    }
    
    return null
  } catch {
    // Not JSON, return raw content
    return null
  }
})
</script>

<style scoped>
.tool-result-message {
  @apply bg-gray-50 border-l-2 border-gray-300 rounded p-2 text-sm;
  @apply max-w-4xl opacity-80;
}

.tool-header {
  @apply flex items-center justify-between mb-2;
}

.tool-info {
  @apply flex items-center space-x-2;
}

.tool-icon {
  @apply font-mono text-xs text-gray-500 bg-gray-100 px-1 py-0.5 rounded text-xs;
}

.tool-name {
  @apply text-white text-xs;
}

.iteration-badge {
  @apply text-xs bg-gray-100 text-gray-600 px-1 py-0.5 rounded;
}

.status-indicator {
  @apply font-mono text-xs;
}

.status-indicator.success {
  @apply text-gray-600;
}

.status-indicator.failure {
  @apply text-gray-600;
}

.tool-result-content {
  @apply text-gray-600 text-xs;
}

.result-message {
  @apply mb-2 font-medium;
}

.result-details {
  @apply bg-gray-50 p-2 rounded text-xs;
}

.raw-result {
  @apply bg-gray-50 p-2 rounded text-xs;
}

.raw-result pre,
.result-details pre {
  @apply whitespace-pre-wrap break-words;
  @apply text-gray-700;
  @apply m-0;
}

/* Success and failure styling - subtle */
.tool-result-message:has(.status-indicator.success) {
  @apply border-gray-300 bg-gray-50;
}

.tool-result-message:has(.status-indicator.success) .tool-icon {
  @apply bg-gray-100 text-gray-500;
}

.tool-result-message:has(.status-indicator.failure) {
  @apply border-gray-400 bg-gray-50;
}

.tool-result-message:has(.status-indicator.failure) .tool-icon {
  @apply bg-gray-200 text-gray-600;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .tool-result-message {
    @apply bg-gray-800 border-gray-600;
  }

  .tool-icon {
    @apply bg-gray-700 text-gray-300;
  }

  .tool-name {
    @apply text-gray-300;
  }

  .iteration-badge {
    @apply bg-blue-800 text-blue-200;
  }

  .tool-result-content {
    @apply text-gray-200;
  }

  .result-details,
  .raw-result {
    @apply bg-gray-700;
  }

  .result-details pre,
  .raw-result pre {
    @apply text-gray-300;
  }

  /* Success dark mode */
  .tool-result-message:has(.status-indicator.success) {
    @apply border-green-500 bg-green-900;
  }

  .tool-result-message:has(.status-indicator.success) .tool-icon {
    @apply bg-green-800 text-green-200;
  }

  /* Failure dark mode */
  .tool-result-message:has(.status-indicator.failure) {
    @apply border-red-500 bg-red-900;
  }

  .tool-result-message:has(.status-indicator.failure) .tool-icon {
    @apply bg-red-800 text-red-200;
  }
}
</style>