<template>
  <div class="image-preview">
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>Loading image...</span>
    </div>

    <div v-else-if="error" class="error-state">
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="error-icon">
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
      </svg>
      <span>{{ error }}</span>
    </div>

    <div v-else-if="imageUrl" class="image-container">
      <img
        :src="imageUrl"
        :alt="document.title"
        class="preview-image"
        @load="handleImageLoad"
        @error="handleImageError"
      />
      <div class="image-info">
        <span class="image-title">{{ document.title }}</span>
        <span v-if="dimensions" class="image-dimensions">{{ dimensions }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Document {
  id: number
  campaign_id: number
  title: string
  file_path: string
  file_type?: string
}

const props = defineProps<{
  document: Document
}>()

const imageUrl = ref<string | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const dimensions = ref<string | null>(null)

const loadImage = async () => {
  loading.value = true
  error.value = null
  dimensions.value = null

  try {
    const response = await invoke<{ success: boolean; data?: string; error?: string }>('read_image_document', {
      filePath: props.document.file_path
    })

    if (response.success && response.data) {
      imageUrl.value = response.data
    } else {
      error.value = response.error || 'Failed to load image'
    }
  } catch (e) {
    console.error('Failed to load image:', e)
    error.value = 'Failed to load image'
  } finally {
    loading.value = false
  }
}

const handleImageLoad = (event: Event) => {
  const img = event.target as HTMLImageElement
  dimensions.value = `${img.naturalWidth} x ${img.naturalHeight}`
}

const handleImageError = () => {
  error.value = 'Failed to display image'
  imageUrl.value = null
}

// Watch for document changes
watch(() => props.document.id, () => {
  loadImage()
})

onMounted(() => {
  loadImage()
})
</script>

<style scoped>
.image-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-lg);
  background: var(--color-background);
}

.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
  color: var(--color-text-secondary);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary-500);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error-icon {
  width: 48px;
  height: 48px;
  color: var(--color-error);
}

.image-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
  max-width: 100%;
  max-height: 100%;
}

.preview-image {
  max-width: 100%;
  max-height: calc(100vh - 200px);
  object-fit: contain;
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-md);
}

.image-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

.image-title {
  font-size: 1rem;
  font-weight: 500;
  color: var(--color-text);
}

.image-dimensions {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}
</style>
