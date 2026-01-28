<template>
  <MainLayout>
    <div class="campaign-create-view">
      <div class="form-container">
        <h1 class="form-title">Create New Campaign</h1>

        <form @submit.prevent="handleSubmit" class="campaign-form">
          <div class="form-group">
            <label for="campaign-name">Campaign Name *</label>
            <input
              id="campaign-name"
              v-model="form.name"
              type="text"
              class="form-input"
              placeholder="Enter campaign name"
              required
            />
          </div>

          <div class="form-group">
            <label for="campaign-description">Description</label>
            <textarea
              id="campaign-description"
              v-model="form.description"
              class="form-textarea"
              rows="3"
              placeholder="Optional campaign description"
            />
          </div>

          <div class="form-actions">
            <button type="button" class="btn-secondary" @click="handleCancel">
              Cancel
            </button>
            <button type="submit" class="btn-primary" :disabled="loading || !form.name.trim()">
              {{ loading ? 'Creating...' : 'Create Campaign' }}
            </button>
          </div>
        </form>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>
      </div>
    </div>
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import MainLayout from '@/shared/components/layout/MainLayout.vue'
import type { ApiResponse } from '@/types/api'

const router = useRouter()

const loading = ref(false)
const error = ref<string | null>(null)

const form = reactive({
  name: '',
  description: ''
})

const handleSubmit = async () => {
  error.value = null
  loading.value = true

  try {
    const response = await invoke<ApiResponse<any>>('create_campaign', {
      request: {
        name: form.name.trim(),
        description: form.description.trim() || null
      }
    })

    if (response.success && response.data) {
      // Navigate to the new campaign dashboard
      router.push(`/campaigns/${response.data.id}/dashboard`)
    } else {
      error.value = response.error || 'Failed to create campaign'
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'An unexpected error occurred'
  } finally {
    loading.value = false
  }
}

const handleCancel = () => {
  router.push('/')
}
</script>

<style scoped>
.campaign-create-view {
  max-width: 600px;
  margin: 0 auto;
}

.form-container {
  background-color: var(--color-surface);
  border-radius: var(--radius-lg);
  padding: var(--spacing-xl);
  box-shadow: var(--shadow-md);
}

.form-title {
  font-size: 1.875rem;
  font-weight: 700;
  color: var(--color-text);
  margin-bottom: var(--spacing-xl);
}

.campaign-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.form-group label {
  font-weight: 500;
  color: var(--color-text);
  font-size: 0.875rem;
}

.form-input,
.form-textarea {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-background);
  color: var(--color-text);
  font-size: 1rem;
  transition: border-color var(--transition-fast);
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 3px var(--color-primary-500) / 0.1;
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

.form-actions {
  display: flex;
  gap: var(--spacing-md);
  justify-content: flex-end;
  margin-top: var(--spacing-lg);
}

.btn-primary,
.btn-secondary {
  padding: var(--spacing-sm) var(--spacing-lg);
  border-radius: var(--radius-md);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: none;
  font-size: 1rem;
}

.btn-primary {
  background-color: var(--color-primary-500);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--color-primary-600);
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  background-color: var(--color-gray-200);
  border-color: var(--color-border-hover);
}

.error-message {
  margin-top: var(--spacing-md);
  padding: var(--spacing-md);
  background-color: var(--color-error) / 0.1;
  border: 1px solid var(--color-error) / 0.2;
  border-radius: var(--radius-md);
  color: var(--color-error);
  font-size: 0.875rem;
}

.theme-dark .btn-secondary:hover {
  background-color: var(--color-gray-700);
}
</style>