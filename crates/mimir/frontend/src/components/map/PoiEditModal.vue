<template>
  <AppModal
    :visible="visible"
    :title="isNew ? 'Add Point of Interest' : 'Edit Point of Interest'"
    size="sm"
    @close="handleClose"
  >
    <form @submit.prevent="handleSave" class="poi-form">
      <!-- Name -->
      <div class="form-group">
        <label for="poi-name">Name</label>
        <input
          id="poi-name"
          v-model="form.name"
          type="text"
          class="form-input"
          placeholder="Enter POI name"
          required
        />
      </div>

      <!-- Description -->
      <div class="form-group">
        <label for="poi-description">Description</label>
        <textarea
          id="poi-description"
          v-model="form.description"
          class="form-textarea"
          placeholder="Optional description for DM notes"
          rows="3"
        ></textarea>
      </div>

      <!-- Icon Picker -->
      <div class="form-group">
        <label>Icon</label>
        <div class="icon-picker">
          <button
            v-for="icon in availableIcons"
            :key="icon.value"
            type="button"
            class="icon-option"
            :class="{ selected: form.icon === icon.value }"
            :title="icon.label"
            @click="form.icon = icon.value"
          >
            <span class="icon-emoji">{{ icon.emoji }}</span>
          </button>
        </div>
      </div>

      <!-- Color Picker -->
      <div class="form-group">
        <label for="poi-color">Color</label>
        <div class="color-picker">
          <input
            id="poi-color"
            v-model="form.color"
            type="color"
            class="color-input"
          />
          <div class="color-presets">
            <button
              v-for="color in presetColors"
              :key="color"
              type="button"
              class="color-preset"
              :class="{ selected: form.color === color }"
              :style="{ backgroundColor: color }"
              @click="form.color = color"
            ></button>
          </div>
        </div>
      </div>

      <!-- Preview -->
      <div class="form-group">
        <label>Preview</label>
        <div class="poi-preview">
          <span class="preview-icon" :style="{ backgroundColor: form.color }">
            {{ getIconEmoji(form.icon) }}
          </span>
          <span class="preview-name">{{ form.name || 'POI Name' }}</span>
        </div>
      </div>
    </form>

    <template #footer>
      <button type="button" class="btn btn-secondary" @click="handleClose">Cancel</button>
      <button
        type="button"
        class="btn btn-primary"
        :disabled="!form.name || saving"
        @click="handleSave"
      >
        {{ saving ? 'Saving...' : (isNew ? 'Create' : 'Save') }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'

interface MapPoi {
  id: string
  map_id: string
  grid_x: number
  grid_y: number
  name: string
  description: string | null
  icon: string
  color: string | null
  visible: number
  created_at: string
  updated_at: string
}

interface Props {
  visible: boolean
  /** POI to edit, or null for new POI */
  poi: MapPoi | null
  /** Map ID (required for new POIs) */
  mapId?: string
  /** Grid position for new POIs */
  gridX?: number
  gridY?: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  saved: [poi: MapPoi]
  created: [poi: MapPoi]
}>()

// Available icons (matches backend schema)
const availableIcons = [
  { value: 'pin', emoji: '📍', label: 'Pin' },
  { value: 'star', emoji: '⭐', label: 'Star' },
  { value: 'skull', emoji: '💀', label: 'Skull' },
  { value: 'chest', emoji: '📦', label: 'Chest' },
  { value: 'door', emoji: '🚪', label: 'Door' },
  { value: 'secret', emoji: '🔮', label: 'Secret' },
  { value: 'question', emoji: '❓', label: 'Question' },
  { value: 'exclamation', emoji: '❗', label: 'Exclamation' }
]

// Preset colors
const presetColors = [
  '#3b82f6', // Blue
  '#22c55e', // Green
  '#ef4444', // Red
  '#f59e0b', // Amber
  '#8b5cf6', // Purple
  '#ec4899', // Pink
  '#06b6d4', // Cyan
  '#6b7280'  // Gray
]

// Form state
const form = reactive({
  name: '',
  description: '',
  icon: 'pin',
  color: '#3b82f6'
})

const saving = ref(false)
const isNew = ref(true)

// Get emoji for icon value
function getIconEmoji(iconValue: string): string {
  const icon = availableIcons.find(i => i.value === iconValue)
  return icon?.emoji || '📍'
}

// Reset form when POI changes
watch(() => props.poi, (poi) => {
  if (poi) {
    isNew.value = false
    form.name = poi.name
    form.description = poi.description || ''
    form.icon = poi.icon || 'pin'
    form.color = poi.color || '#3b82f6'
  } else {
    isNew.value = true
    form.name = ''
    form.description = ''
    form.icon = 'pin'
    form.color = '#3b82f6'
  }
}, { immediate: true })

// Reset when modal opens
watch(() => props.visible, (visible) => {
  if (visible && !props.poi) {
    form.name = ''
    form.description = ''
    form.icon = 'pin'
    form.color = '#3b82f6'
    isNew.value = true
  }
})

async function handleSave() {
  if (!form.name) return

  saving.value = true

  try {
    if (isNew.value) {
      // Create new POI
      if (!props.mapId || props.gridX === undefined || props.gridY === undefined) {
        console.error('Map ID and grid position required for new POI')
        return
      }

      const response = await invoke<{ success: boolean; data?: MapPoi; error?: string }>('create_map_poi', {
        request: {
          mapId: props.mapId,
          name: form.name,
          gridX: props.gridX,
          gridY: props.gridY,
          description: form.description || null,
          icon: form.icon,
          color: form.color,
          visible: false
        }
      })

      if (response.success && response.data) {
        emit('created', response.data)
        emit('close')
      } else {
        console.error('Failed to create POI:', response.error)
      }
    } else {
      // Update existing POI
      if (!props.poi) return

      const response = await invoke<{ success: boolean; data?: MapPoi; error?: string }>('update_map_poi', {
        id: props.poi.id,
        request: {
          name: form.name,
          description: form.description || null,
          icon: form.icon,
          color: form.color
        }
      })

      if (response.success && response.data) {
        emit('saved', response.data)
        emit('close')
      } else {
        console.error('Failed to update POI:', response.error)
      }
    }
  } catch (e) {
    console.error('Failed to save POI:', e)
  } finally {
    saving.value = false
  }
}

function handleClose() {
  emit('close')
}
</script>

<style scoped>
.poi-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.form-group label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.form-input,
.form-textarea {
  padding: 0.5rem 0.75rem;
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 0.875rem;
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}

.form-textarea {
  resize: vertical;
  min-height: 60px;
}

/* Icon Picker */
.icon-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.icon-option {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface-variant);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s ease;
}

.icon-option:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-primary-300);
}

.icon-option.selected {
  background: var(--color-primary-100);
  border-color: var(--color-primary-500);
}

.icon-emoji {
  font-size: 1.25rem;
}

/* Color Picker */
.color-picker {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.color-input {
  width: 40px;
  height: 40px;
  padding: 0;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  background: transparent;
}

.color-input::-webkit-color-swatch-wrapper {
  padding: 2px;
}

.color-input::-webkit-color-swatch {
  border: none;
  border-radius: var(--radius-sm);
}

.color-presets {
  display: flex;
  gap: 0.375rem;
}

.color-preset {
  width: 24px;
  height: 24px;
  border: 2px solid transparent;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.15s ease;
}

.color-preset:hover {
  transform: scale(1.1);
}

.color-preset.selected {
  border-color: var(--color-text);
  box-shadow: 0 0 0 2px var(--color-background);
}

/* Preview */
.poi-preview {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.preview-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  color: white;
}

.preview-name {
  font-weight: 500;
  color: var(--color-text);
}

/* Buttons */
.btn {
  padding: 0.5rem 1rem;
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-secondary {
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.btn-secondary:hover {
  background: var(--color-surface-hover);
}

.btn-primary {
  background: var(--color-primary-500);
  border: 1px solid var(--color-primary-500);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-primary-600);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
