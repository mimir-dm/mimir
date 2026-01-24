<template>
  <AppModal
    :visible="visible"
    title="Upload Map"
    size="md"
    :closable="!uploading"
    :close-on-overlay="!uploading"
    :close-on-escape="!uploading"
    @close="handleClose"
  >
    <!-- Drop Zone -->
    <div
      class="drop-zone"
      :class="{ 'drag-over': isDragging, 'has-file': previewUrl }"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="handleDrop"
      @click="triggerFileInput"
    >
      <input
        ref="fileInput"
        type="file"
        accept="image/png,image/jpeg,image/jpg,image/webp,.dd2vtt,.uvtt"
        class="file-input"
        @change="handleFileSelect"
      />

      <div v-if="previewUrl" class="preview-container">
        <img :src="previewUrl" :alt="mapName" class="preview-image" />
        <button class="clear-btn" @click.stop="clearFile">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
        <!-- UVTT metadata badge -->
        <div v-if="uvttInfo" class="uvtt-badge">
          <span class="uvtt-label">UVTT</span>
          <span class="uvtt-stats">{{ uvttInfo.walls }} walls · {{ uvttInfo.portals }} doors · {{ uvttInfo.lights }} lights</span>
        </div>
      </div>

      <div v-else class="drop-prompt">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="upload-icon">
          <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
        </svg>
        <p class="drop-text">Drop map here or click to browse</p>
        <p class="drop-hint">Supports UVTT (.dd2vtt), PNG, JPG, WebP</p>
      </div>
    </div>

    <!-- Map Name Input -->
    <div class="form-group">
      <label for="map-name">Map Name</label>
      <input
        id="map-name"
        v-model="mapName"
        type="text"
        class="form-input"
        placeholder="e.g., Goblin Cave - Entrance"
      />
    </div>

    <!-- Image Dimensions (read-only) -->
    <div v-if="imageWidth && imageHeight" class="dimensions-info">
      <span>Image Size: {{ imageWidth }} x {{ imageHeight }} px</span>
    </div>

    <!-- Error Message -->
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </div>

    <!-- Upload Progress Message -->
    <div v-if="uploading" class="upload-progress">
      <div class="progress-spinner"></div>
      <span>Processing image... This may take a moment for large files.</span>
    </div>

    <template #footer>
      <button class="btn btn-secondary" @click="handleClose" :disabled="uploading">
        Cancel
      </button>
      <button
        class="btn btn-primary"
        @click="handleUpload"
        :disabled="!canUpload || uploading"
      >
        {{ uploading ? 'Uploading...' : 'Upload Map' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import { dataEvents } from '@/shared/utils/dataEvents'

const props = defineProps<{
  visible: boolean
  campaignId: string
  moduleId?: string
}>()

const emit = defineEmits<{
  close: []
  uploaded: []
}>()

interface UvttInfo {
  walls: number
  portals: number
  lights: number
  gridCols: number
  gridRows: number
  pixelsPerGrid: number
}

const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const previewUrl = ref<string | null>(null)
const mapName = ref('')
const imageWidth = ref<number | null>(null)
const imageHeight = ref<number | null>(null)
const isDragging = ref(false)
const uploading = ref(false)
const errorMessage = ref('')
const uvttInfo = ref<UvttInfo | null>(null)

const canUpload = computed(() => {
  return selectedFile.value && mapName.value.trim() && imageWidth.value && imageHeight.value
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
    const isImage = file.type.startsWith('image/')
    const isUvtt = file.name.toLowerCase().endsWith('.dd2vtt') || file.name.toLowerCase().endsWith('.uvtt')
    if (isImage || isUvtt) {
      processFile(file)
    }
  }
}

function processFile(file: File) {
  const isUvtt = file.name.toLowerCase().endsWith('.dd2vtt') || file.name.toLowerCase().endsWith('.uvtt')

  // Validate file type
  const validImageTypes = ['image/png', 'image/jpeg', 'image/jpg', 'image/webp']
  if (!isUvtt && !validImageTypes.includes(file.type)) {
    errorMessage.value = 'Invalid file type. Please upload UVTT, PNG, JPG, or WebP.'
    return
  }

  // Validate file size (max 50MB)
  if (file.size > 50 * 1024 * 1024) {
    errorMessage.value = 'File too large. Maximum size is 50MB.'
    return
  }

  errorMessage.value = ''
  selectedFile.value = file
  uvttInfo.value = null

  // Auto-generate map name from filename
  if (!mapName.value) {
    const nameWithoutExt = file.name.replace(/\.[^/.]+$/, '')
    mapName.value = nameWithoutExt
      .replace(/[-_]/g, ' ')
      .replace(/\b\w/g, c => c.toUpperCase())
  }

  if (isUvtt) {
    // Parse UVTT file
    processUvttFile(file)
  } else {
    // Process image file
    processImageFile(file)
  }
}

function processUvttFile(file: File) {
  const reader = new FileReader()
  reader.onload = (e) => {
    try {
      const uvtt = JSON.parse(e.target?.result as string)

      // Extract UVTT metadata
      uvttInfo.value = {
        walls: uvtt.line_of_sight?.length || 0,
        portals: uvtt.portals?.length || 0,
        lights: uvtt.lights?.length || 0,
        gridCols: Math.round(uvtt.resolution?.map_size?.x || 0),
        gridRows: Math.round(uvtt.resolution?.map_size?.y || 0),
        pixelsPerGrid: uvtt.resolution?.pixels_per_grid || 70
      }

      // Extract image for preview
      let imageData = uvtt.image || ''
      if (!imageData.startsWith('data:')) {
        imageData = `data:image/png;base64,${imageData}`
      }
      previewUrl.value = imageData

      // Get dimensions from the image
      const img = new Image()
      img.onload = () => {
        imageWidth.value = img.naturalWidth
        imageHeight.value = img.naturalHeight
      }
      img.onerror = () => {
        // Fallback to calculated dimensions
        imageWidth.value = uvttInfo.value!.gridCols * uvttInfo.value!.pixelsPerGrid
        imageHeight.value = uvttInfo.value!.gridRows * uvttInfo.value!.pixelsPerGrid
      }
      img.src = imageData
    } catch (err) {
      console.error('Failed to parse UVTT file:', err)
      errorMessage.value = 'Failed to parse UVTT file. Please check the file format.'
      clearFile()
    }
  }
  reader.onerror = () => {
    errorMessage.value = 'Failed to read file.'
    clearFile()
  }
  reader.readAsText(file)
}

function processImageFile(file: File) {
  const reader = new FileReader()
  reader.onload = (e) => {
    previewUrl.value = e.target?.result as string

    // Get image dimensions
    const img = new Image()
    img.onload = () => {
      imageWidth.value = img.naturalWidth
      imageHeight.value = img.naturalHeight
    }
    img.src = previewUrl.value
  }
  reader.readAsDataURL(file)
}

function clearFile() {
  selectedFile.value = null
  previewUrl.value = null
  imageWidth.value = null
  imageHeight.value = null
  uvttInfo.value = null
  if (fileInput.value) {
    fileInput.value.value = ''
  }
}

async function handleUpload() {
  if (!selectedFile.value || !imageWidth.value || !imageHeight.value) return

  uploading.value = true
  errorMessage.value = ''

  try {
    // Convert file to base64
    const base64Data = await fileToBase64(selectedFile.value)

    // Upload map (handles both UVTT and images)
    const response = await invoke<{ success: boolean; error?: string; data?: unknown }>('create_map', {
      request: {
        campaign_id: props.campaignId,
        module_id: props.moduleId ?? null,
        name: mapName.value.trim(),
        filename: selectedFile.value.name,
        uvtt_data_base64: base64Data
      }
    })

    if (response.success) {
      if (props.moduleId) {
        dataEvents.emit('module:maps:changed', { moduleId: props.moduleId })
      }
      emit('uploaded')
      resetForm()
    } else {
      errorMessage.value = response.error || 'Failed to upload map'
    }
  } catch (e) {
    console.error('Upload error:', e)
    errorMessage.value = 'Failed to upload map. Please try again.'
  } finally {
    uploading.value = false
  }
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      // Remove the data URL prefix (e.g., "data:image/png;base64,")
      const result = reader.result as string
      const base64 = result.split(',')[1]
      resolve(base64)
    }
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

function resetForm() {
  clearFile()
  mapName.value = ''
  errorMessage.value = ''
  uvttInfo.value = null
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
/* Drop zone styles */
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
  padding: 0;
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
  color: var(--color-text-muted);
}

.drop-text {
  font-size: 0.875rem;
  color: var(--color-text);
  margin: 0;
}

.drop-hint {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin: 0;
}

.preview-container {
  position: relative;
  width: 100%;
}

.preview-image {
  width: 100%;
  max-height: 300px;
  object-fit: contain;
  border-radius: var(--radius-md);
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

.uvtt-badge {
  position: absolute;
  bottom: var(--spacing-sm);
  left: var(--spacing-sm);
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: rgba(0, 0, 0, 0.75);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
}

.uvtt-label {
  color: #10b981;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.uvtt-stats {
  color: rgba(255, 255, 255, 0.8);
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

.dimensions-info {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-bottom: var(--spacing-md);
}

.error-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-100);
  border: 1px solid var(--color-error);
  border-radius: var(--radius-md);
  color: var(--color-error);
  font-size: 0.875rem;
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
