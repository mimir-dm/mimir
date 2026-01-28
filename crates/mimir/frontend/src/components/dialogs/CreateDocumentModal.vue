<template>
  <AppModal
    :visible="visible"
    title="Create Document"
    size="md"
    :closable="!uploading"
    :close-on-overlay="!uploading"
    :close-on-escape="!uploading"
    @close="handleClose"
  >
    <!-- Mode Selector Tabs -->
    <div class="mode-tabs">
      <button
        class="mode-tab"
        :class="{ active: mode === 'create' }"
        @click="mode = 'create'"
        :disabled="uploading"
      >
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="tab-icon">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
        </svg>
        New Document
      </button>
      <button
        class="mode-tab"
        :class="{ active: mode === 'upload' }"
        @click="mode = 'upload'"
        :disabled="uploading"
      >
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="tab-icon">
          <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
        </svg>
        Upload File
      </button>
    </div>

    <!-- Create Mode: Title Input -->
    <div v-if="mode === 'create'" class="create-mode">
      <div class="form-group">
        <label for="doc-title">Document Title</label>
        <input
          id="doc-title"
          v-model="docTitle"
          type="text"
          class="form-input"
          placeholder="e.g., Session 5 Notes, NPC List"
          @keyup.enter="handleCreate"
          :disabled="uploading"
        />
      </div>
      <p class="hint">Creates a blank markdown document you can edit.</p>
    </div>

    <!-- Upload Mode: Drop Zone -->
    <div v-else class="upload-mode">
      <div
        class="drop-zone"
        :class="{ 'drag-over': isDragging, 'has-file': previewUrl || selectedFile }"
        @dragover.prevent="isDragging = true"
        @dragleave.prevent="isDragging = false"
        @drop.prevent="handleDrop"
        @click="triggerFileInput"
      >
        <input
          ref="fileInput"
          type="file"
          :accept="acceptedTypes"
          class="file-input"
          @change="handleFileSelect"
        />

        <div v-if="previewUrl" class="preview-container">
          <img :src="previewUrl" :alt="docTitle" class="preview-image" />
          <button class="clear-btn" @click.stop="clearFile">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div v-else-if="selectedFile && !isImageFile" class="file-preview">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="file-icon">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
          </svg>
          <span class="file-name">{{ selectedFile.name }}</span>
          <button class="clear-btn-inline" @click.stop="clearFile">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div v-else class="drop-prompt">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="upload-icon">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
          </svg>
          <p class="drop-text">Drop file here or click to browse</p>
          <p class="drop-hint">Supports .md, .png, .jpg, .webp, .gif, .svg</p>
        </div>
      </div>

      <!-- Document Title for uploaded files -->
      <div v-if="selectedFile" class="form-group">
        <label for="upload-title">Document Title</label>
        <input
          id="upload-title"
          v-model="docTitle"
          type="text"
          class="form-input"
          placeholder="Enter a title for this document"
          :disabled="uploading"
        />
      </div>
    </div>

    <!-- Error Message -->
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </div>

    <!-- Upload Progress Message -->
    <div v-if="uploading" class="upload-progress">
      <div class="progress-spinner"></div>
      <span>{{ mode === 'create' ? 'Creating document...' : 'Uploading file...' }}</span>
    </div>

    <template #footer>
      <button class="btn btn-secondary" @click="handleClose" :disabled="uploading">
        Cancel
      </button>
      <button
        class="btn btn-primary"
        @click="handleSubmit"
        :disabled="!canSubmit || uploading"
      >
        {{ uploading ? 'Processing...' : (mode === 'create' ? 'Create Document' : 'Upload File') }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import { DocumentService } from '@/services/DocumentService'

const props = defineProps<{
  visible: boolean
  campaignId: string
  moduleId?: string
}>()

const emit = defineEmits<{
  close: []
  created: []
}>()

const mode = ref<'create' | 'upload'>('create')
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const previewUrl = ref<string | null>(null)
const docTitle = ref('')
const isDragging = ref(false)
const uploading = ref(false)
const errorMessage = ref('')

const acceptedTypes = '.md,.png,.jpg,.jpeg,.webp,.gif,.svg'

const imageExtensions = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg']

const isImageFile = computed(() => {
  if (!selectedFile.value) return false
  const ext = selectedFile.value.name.split('.').pop()?.toLowerCase()
  return ext ? imageExtensions.includes(ext) : false
})

const canSubmit = computed(() => {
  if (mode.value === 'create') {
    return docTitle.value.trim().length > 0
  } else {
    return selectedFile.value !== null && docTitle.value.trim().length > 0
  }
})

function triggerFileInput() {
  fileInput.value?.click()
}

function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files?.[0]) {
    processFile(input.files[0])
  }
}

function handleDrop(event: DragEvent) {
  isDragging.value = false
  const file = event.dataTransfer?.files?.[0]
  if (file) {
    processFile(file)
  }
}

function processFile(file: File) {
  const ext = file.name.split('.').pop()?.toLowerCase()
  const validExtensions = ['md', ...imageExtensions]

  if (!ext || !validExtensions.includes(ext)) {
    errorMessage.value = 'Invalid file type. Please upload .md, .png, .jpg, .webp, .gif, or .svg.'
    return
  }

  // Validate file size (max 20MB for images, 5MB for markdown)
  const maxSize = ext === 'md' ? 5 * 1024 * 1024 : 20 * 1024 * 1024
  if (file.size > maxSize) {
    errorMessage.value = ext === 'md'
      ? 'File too large. Maximum size for markdown is 5MB.'
      : 'File too large. Maximum size for images is 20MB.'
    return
  }

  errorMessage.value = ''
  selectedFile.value = file

  // Auto-generate title from filename if empty
  if (!docTitle.value) {
    const nameWithoutExt = file.name.replace(/\.[^/.]+$/, '')
    docTitle.value = nameWithoutExt
      .replace(/[-_]/g, ' ')
      .replace(/\b\w/g, c => c.toUpperCase())
  }

  // Generate preview for images
  if (imageExtensions.includes(ext)) {
    const reader = new FileReader()
    reader.onload = (e) => {
      previewUrl.value = e.target?.result as string
    }
    reader.readAsDataURL(file)
  } else {
    previewUrl.value = null
  }
}

function clearFile() {
  selectedFile.value = null
  previewUrl.value = null
  if (fileInput.value) {
    fileInput.value.value = ''
  }
}

async function handleCreate() {
  if (!docTitle.value.trim()) return

  uploading.value = true
  errorMessage.value = ''

  try {
    await DocumentService.create({
      campaign_id: props.campaignId,
      module_id: props.moduleId,
      title: docTitle.value.trim(),
      doc_type: 'note',
      content: ''
    })

    emit('created')
    resetForm()
  } catch (e) {
    console.error('Create error:', e)
    errorMessage.value = e instanceof Error ? e.message : 'Failed to create document. Please try again.'
  } finally {
    uploading.value = false
  }
}

async function handleUpload() {
  if (!selectedFile.value || !docTitle.value.trim()) return

  uploading.value = true
  errorMessage.value = ''

  try {
    const ext = selectedFile.value.name.split('.').pop()?.toLowerCase()
    const isImage = ext ? imageExtensions.includes(ext) : false

    if (isImage) {
      // Upload images as assets
      const mimeTypes: Record<string, string> = {
        'png': 'image/png',
        'jpg': 'image/jpeg',
        'jpeg': 'image/jpeg',
        'webp': 'image/webp',
        'gif': 'image/gif',
        'svg': 'image/svg+xml'
      }
      const mimeType = mimeTypes[ext || ''] || 'application/octet-stream'
      const data = await fileToBase64(selectedFile.value)

      const response = await invoke<{ success: boolean; error?: string }>('upload_asset', {
        request: {
          campaign_id: props.campaignId,
          module_id: props.moduleId || null,
          filename: selectedFile.value.name,
          description: docTitle.value.trim(),
          mime_type: mimeType,
          data_base64: data
        }
      })

      if (!response.success) {
        throw new Error(response.error || 'Failed to upload asset')
      }
    } else {
      // Upload markdown files as documents
      const content = await fileToText(selectedFile.value)
      await DocumentService.create({
        campaign_id: props.campaignId,
        module_id: props.moduleId,
        title: docTitle.value.trim(),
        doc_type: 'note',
        content
      })
    }

    emit('created')
    resetForm()
  } catch (e) {
    console.error('Upload error:', e)
    errorMessage.value = e instanceof Error ? e.message : 'Failed to upload file. Please try again.'
  } finally {
    uploading.value = false
  }
}

function handleSubmit() {
  if (mode.value === 'create') {
    handleCreate()
  } else {
    handleUpload()
  }
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result as string
      const base64 = result.split(',')[1]
      resolve(base64)
    }
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

function fileToText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      resolve(reader.result as string)
    }
    reader.onerror = reject
    reader.readAsText(file)
  })
}

function resetForm() {
  mode.value = 'create'
  docTitle.value = ''
  clearFile()
  errorMessage.value = ''
}

function handleClose() {
  if (!uploading.value) {
    resetForm()
    emit('close')
  }
}

// Reset form when modal closes
watch(() => props.visible, (visible) => {
  if (!visible) {
    resetForm()
  }
})
</script>

<style scoped>
.mode-tabs {
  display: flex;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
  padding-bottom: var(--spacing-md);
}

.mode-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.mode-tab:hover:not(:disabled) {
  border-color: var(--color-primary-300);
  color: var(--color-text);
}

.mode-tab.active {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
  color: var(--color-primary-700);
}

.mode-tab:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tab-icon {
  width: 18px;
  height: 18px;
}

.create-mode,
.upload-mode {
  margin-bottom: var(--spacing-md);
}

.hint {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin: 0;
}

.drop-zone {
  border: 2px dashed var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-xl);
  text-align: center;
  cursor: pointer;
  transition: all var(--transition-fast);
  margin-bottom: var(--spacing-md);
  position: relative;
}

.drop-zone:hover,
.drop-zone.drag-over {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
}

.drop-zone.has-file {
  padding: var(--spacing-md);
  border-style: solid;
}

.file-input {
  display: none;
}

.drop-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-sm);
}

.upload-icon {
  width: 48px;
  height: 48px;
  color: var(--color-text-secondary);
}

.drop-text {
  font-size: 0.875rem;
  color: var(--color-text);
  margin: 0;
}

.drop-hint {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin: 0;
}

.preview-container {
  position: relative;
  width: 100%;
}

.preview-image {
  width: 100%;
  max-height: 250px;
  object-fit: contain;
  border-radius: var(--radius-md);
}

.file-preview {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-md);
  background: var(--color-surface);
  border-radius: var(--radius-md);
}

.file-icon {
  width: 32px;
  height: 32px;
  color: var(--color-primary-500);
  flex-shrink: 0;
}

.file-name {
  flex: 1;
  font-size: 0.875rem;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.clear-btn {
  position: absolute;
  top: var(--spacing-sm);
  right: var(--spacing-sm);
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  cursor: pointer;
  border-radius: 50%;
  transition: all var(--transition-fast);
}

.clear-btn:hover {
  background: rgba(0, 0, 0, 0.8);
}

.clear-btn svg {
  width: 16px;
  height: 16px;
}

.clear-btn-inline {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: var(--color-surface-variant);
  color: var(--color-text-secondary);
  cursor: pointer;
  border-radius: 50%;
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.clear-btn-inline:hover {
  background: var(--color-error-100);
  color: var(--color-error);
}

.clear-btn-inline svg {
  width: 14px;
  height: 14px;
}

.form-group {
  margin-bottom: var(--spacing-md);
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: var(--spacing-xs);
}

.form-input {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  background: var(--color-background);
  color: var(--color-text);
  transition: all var(--transition-fast);
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}

.form-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-100);
  border: 1px solid var(--color-error);
  border-radius: var(--radius-md);
  color: var(--color-error);
  font-size: 0.875rem;
  margin-bottom: var(--spacing-md);
}

.upload-progress {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-md);
  background: var(--color-primary-50);
  border: 1px solid var(--color-primary-200);
  border-radius: var(--radius-md);
  color: var(--color-primary-700);
  font-size: 0.875rem;
}

.progress-spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--color-primary-200);
  border-top-color: var(--color-primary-500);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
