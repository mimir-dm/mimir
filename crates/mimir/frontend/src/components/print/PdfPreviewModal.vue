<template>
  <AppModal
    :visible="visible"
    :title="title"
    size="full"
    @close="closeModal"
  >
    <template #header>
      <h2>{{ title }}</h2>
      <div class="header-actions">
        <span v-if="pdfSize" class="pdf-size">{{ formatSize(pdfSize) }}</span>
      </div>
    </template>

    <div class="pdf-body">
      <div v-if="isLoading" class="loading-state">
        <div class="spinner"></div>
        <p>Generating PDF...</p>
      </div>

      <div v-else-if="error" class="error-state">
        <p class="error-title">Failed to generate PDF</p>
        <p class="error-message">{{ error }}</p>
        <button @click="retry" class="btn btn-primary">Try Again</button>
      </div>

      <div v-else-if="pdfUrl" class="pdf-container">
        <iframe
          :src="pdfUrl"
          class="pdf-frame"
          title="PDF Preview"
        ></iframe>
      </div>

      <EmptyState
        v-else
        variant="generic"
        title="No PDF to display"
        description="Generate a PDF to preview it here"
      />
    </div>

    <template #footer>
      <button
        @click="handlePrint"
        class="btn btn-secondary"
        :disabled="!pdfUrl || isLoading"
      >
        Print
      </button>
      <button
        @click="handleSave"
        class="btn btn-primary"
        :disabled="!pdfUrl || isLoading"
      >
        Save PDF
      </button>
      <button @click="closeModal" class="btn btn-secondary">
        Close
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { PrintService, type PrintResult } from '../../services/PrintService'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'

interface Props {
  visible: boolean
  title?: string
  defaultFileName?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: 'PDF Preview',
  defaultFileName: 'document.pdf'
})

const emit = defineEmits<{
  close: []
  retry: []
}>()

const isLoading = ref(false)
const error = ref<string | null>(null)
const pdfUrl = ref<string | null>(null)
const pdfResult = ref<PrintResult | null>(null)
const pdfSize = ref<number | null>(null)

// Clean up blob URL when component unmounts or PDF changes
function cleanupUrl() {
  if (pdfUrl.value) {
    URL.revokeObjectURL(pdfUrl.value)
    pdfUrl.value = null
  }
}

onUnmounted(() => {
  cleanupUrl()
})

// Reset state when modal closes
watch(() => props.visible, (newVisible) => {
  if (!newVisible) {
    // Don't clean up URL immediately - let it persist for a moment
    // in case the modal is being reopened
    setTimeout(() => {
      if (!props.visible) {
        cleanupUrl()
        error.value = null
        pdfResult.value = null
        pdfSize.value = null
      }
    }, 100)
  }
})

// Expose methods for parent to call
function setLoading(loading: boolean) {
  isLoading.value = loading
  if (loading) {
    error.value = null
  }
}

function setError(errorMessage: string) {
  error.value = errorMessage
  isLoading.value = false
}

function setPdfResult(result: PrintResult) {
  cleanupUrl()
  pdfResult.value = result
  pdfSize.value = result.size_bytes
  pdfUrl.value = PrintService.createPdfUrl(result)
  isLoading.value = false
  error.value = null
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

async function handleSave() {
  if (!pdfResult.value) return

  try {
    const savedPath = await PrintService.savePdf(pdfResult.value, props.defaultFileName)
    if (savedPath) {
      // Could show a toast notification here
      console.log('PDF saved to:', savedPath)
    }
  } catch (err) {
    console.error('Failed to save PDF:', err)
    error.value = 'Failed to save PDF. Please try again.'
  }
}

function handlePrint() {
  if (!pdfResult.value) return
  PrintService.printPdf(pdfResult.value)
}

function retry() {
  emit('retry')
}

function closeModal() {
  emit('close')
}

// Expose methods to parent
defineExpose({
  setLoading,
  setError,
  setPdfResult
})
</script>

<style scoped>
/* Domain-specific styles */
.pdf-body {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background);
  height: calc(100vh - 200px); /* Account for header + footer */
  min-height: 400px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.pdf-size {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-sm);
}

.loading-state,
.error-state,
.empty-state {
  text-align: center;
  padding: var(--spacing-xl);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary-500);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto var(--spacing-md);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-state p {
  color: var(--color-text-secondary);
  margin: 0;
}

.error-state {
  max-width: 400px;
}

.error-title {
  font-weight: 600;
  color: var(--color-error-600);
  margin: 0 0 var(--spacing-sm);
}

.theme-dark .error-title {
  color: var(--color-error-400);
}

.error-message {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  margin: 0 0 var(--spacing-lg);
}

.pdf-container {
  width: 100%;
  height: 100%;
  display: flex;
}

.pdf-frame {
  width: 100%;
  height: 100%;
  border: none;
  flex: 1;
}
</style>
