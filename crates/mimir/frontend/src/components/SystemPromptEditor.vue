<template>
  <div class="system-prompt-editor">
    <div class="editor-header">
      <h3 class="editor-title">System Prompt</h3>
      <div class="editor-stats">
        <span class="char-count">{{ characterCount }} characters</span>
        <span class="token-estimate">~{{ tokenEstimate }} tokens</span>
      </div>
    </div>
    
    <div class="editor-controls">
      <button 
        @click="togglePreview" 
        class="button button-secondary"
        :class="{ active: showPreview }"
      >
        {{ showPreview ? 'Edit' : 'Preview' }}
      </button>
      
      <button 
        @click="resetToDefault" 
        class="button button-outline"
        :disabled="!hasChanges"
      >
        Reset to Default
      </button>
    </div>
    
    <div class="editor-content">
      <div v-if="showPreview" class="preview-pane">
        <div class="preview-content" v-html="formattedPreview"></div>
      </div>
      
      <textarea 
        v-else
        v-model="localPrompt"
        class="prompt-textarea"
        placeholder="Enter your system prompt here..."
        @input="handleInput"
        rows="20"
      ></textarea>
    </div>
    
    <div class="editor-footer">
      <p class="editor-help">
        This prompt defines how Mimir behaves. You can customize it to match your DM style.
        Changes are saved automatically to your browser.
      </p>
      
      <div class="editor-actions">
        <button 
          @click="savePrompt" 
          class="button button-primary"
          :disabled="!hasUnsavedChanges"
        >
          {{ hasUnsavedChanges ? 'Save Changes' : 'Saved' }}
        </button>
      </div>
    </div>
    
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { marked } from 'marked'
import { DEFAULT_SYSTEM_PROMPT } from '@/constants/defaultSystemPrompt'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

// Local state
const localPrompt = ref(props.modelValue || DEFAULT_SYSTEM_PROMPT)
const showPreview = ref(false)
const hasUnsavedChanges = ref(false)

// Watch for external changes to modelValue
watch(() => props.modelValue, (newValue) => {
  if (newValue !== localPrompt.value) {
    localPrompt.value = newValue || DEFAULT_SYSTEM_PROMPT
    hasUnsavedChanges.value = false
  }
})

// Computed properties
const characterCount = computed(() => localPrompt.value.length)

const tokenEstimate = computed(() => {
  // Rough estimate: 4 characters per token on average
  return Math.ceil(characterCount.value / 4)
})

const hasChanges = computed(() => {
  return localPrompt.value !== DEFAULT_SYSTEM_PROMPT
})

const formattedPreview = computed(() => {
  try {
    return marked(localPrompt.value, {
      breaks: true,
      gfm: true
    })
  } catch (error) {
    return localPrompt.value.replace(/\n/g, '<br>')
  }
})

// Methods
const handleInput = () => {
  hasUnsavedChanges.value = true
}

const savePrompt = () => {
  emit('update:modelValue', localPrompt.value)
  hasUnsavedChanges.value = false
}

const resetToDefault = () => {
  if (confirm('Are you sure you want to reset to the default system prompt? This will lose any custom changes.')) {
    localPrompt.value = DEFAULT_SYSTEM_PROMPT
    hasUnsavedChanges.value = true
  }
}

const togglePreview = () => {
  showPreview.value = !showPreview.value
}

</script>

<style scoped>
.system-prompt-editor {
  @apply space-y-4;
}

.editor-header {
  @apply flex items-center justify-between;
}

.editor-title {
  @apply text-lg font-semibold;
  color: var(--color-text);
}

.editor-stats {
  @apply flex items-center gap-4 text-sm;
  color: var(--color-text-secondary);
}

.editor-controls {
  @apply flex items-center gap-2;
}

.editor-content {
  @apply relative;
  min-height: 400px;
}

.prompt-textarea {
  @apply w-full h-full min-h-[400px] p-4 rounded-lg border resize-y;
  background-color: var(--color-surface);
  border-color: var(--color-border);
  color: var(--color-text);
  font-family: 'JetBrains Mono', 'Courier New', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
}

.prompt-textarea:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 3px rgba(var(--color-primary-500-rgb), 0.1);
}

.preview-pane {
  @apply min-h-[400px] p-4 rounded-lg border;
  background-color: var(--color-surface);
  border-color: var(--color-border);
}

.preview-content {
  @apply max-w-none text-sm leading-relaxed;
  color: var(--color-text);
}

.preview-content :deep(h1) {
  @apply text-2xl font-bold mt-6 mb-4;
  color: var(--color-text);
}

.preview-content :deep(h2) {
  @apply text-xl font-semibold mt-5 mb-3;
  color: var(--color-text);
}

.preview-content :deep(h3) {
  @apply text-lg font-semibold mt-4 mb-2;
  color: var(--color-text);
}

.preview-content :deep(p) {
  @apply mb-4;
}

.preview-content :deep(ul) {
  @apply list-disc list-inside mb-4 space-y-1;
}

.preview-content :deep(ol) {
  @apply list-decimal list-inside mb-4 space-y-1;
}

.preview-content :deep(li) {
  @apply ml-4;
}

.preview-content :deep(code) {
  @apply px-1 py-0.5 rounded text-xs;
  background-color: var(--color-surface-variant);
  color: var(--color-text);
  font-family: 'JetBrains Mono', 'Courier New', monospace;
}

.preview-content :deep(pre) {
  @apply p-4 rounded-lg mb-4 overflow-x-auto;
  background-color: var(--color-surface-variant);
}

.preview-content :deep(pre code) {
  @apply p-0 bg-transparent;
}

.editor-footer {
  @apply space-y-3;
}

.editor-help {
  @apply text-sm;
  color: var(--color-text-secondary);
}

.editor-actions {
  @apply flex items-center justify-end gap-2;
}

.button {
  @apply px-3 py-2 rounded-md font-medium transition-all;
  font-size: 0.875rem;
}

.button-primary {
  background-color: var(--color-primary-500);
  color: white;
}

.button-primary:hover:not(:disabled) {
  background-color: var(--color-primary-600);
}

.button-primary:disabled {
  background-color: var(--color-primary-300);
  cursor: not-allowed;
}

.button-secondary {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.button-secondary:hover {
  background-color: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.button-secondary.active {
  background-color: var(--color-primary-100);
  color: var(--color-primary-700);
  border-color: var(--color-primary-300);
}

.button-outline {
  background: transparent;
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
}

.button-outline:hover:not(:disabled) {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
}

.button-outline:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Dark mode adjustments */
@media (prefers-color-scheme: dark) {
  .preview-content :deep(h1),
  .preview-content :deep(h2),
  .preview-content :deep(h3) {
    color: var(--color-text);
  }
  
  .button-secondary.active {
    background-color: var(--color-primary-900);
    color: var(--color-primary-300);
    border-color: var(--color-primary-700);
  }
}
</style>