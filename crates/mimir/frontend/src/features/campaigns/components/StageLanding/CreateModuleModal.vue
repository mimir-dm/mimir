<template>
  <AppModal
    :visible="show"
    title="Create New Module"
    size="sm"
    @close="$emit('close')"
  >
    <div class="form-group">
      <label for="module-name">Module Name</label>
      <input
        id="module-name"
        v-model="moduleName"
        type="text"
        class="form-input"
        placeholder="e.g., The Goblin Hideout"
        @keyup.enter="handleCreate"
      />
    </div>

    <div class="form-group">
      <label for="module-type">Module Type</label>
      <select id="module-type" v-model="moduleType" class="form-select">
        <option value="general">Standard Adventure</option>
        <option value="mystery">Mystery</option>
        <option value="dungeon">Dungeon Crawl</option>
        <option value="heist">Heist</option>
        <option value="horror">Horror</option>
        <option value="political">Political Intrigue</option>
      </select>
      <p class="hint">Determines the starting document template</p>
    </div>

    <div class="form-group">
      <label for="module-description">Description <span class="optional">(optional)</span></label>
      <textarea
        id="module-description"
        v-model="moduleDescription"
        class="form-textarea"
        placeholder="Brief summary of the module..."
        rows="3"
      ></textarea>
    </div>

    <template #footer>
      <button @click="$emit('close')" class="btn btn-secondary">
        Cancel
      </button>
      <button @click="handleCreate" class="btn btn-primary" :disabled="!canCreate">
        Create Module
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import AppModal from '@/components/shared/AppModal.vue'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
  create: [data: { name: string; type: string; description?: string }]
}>()

const moduleName = ref('')
const moduleType = ref('general')
const moduleDescription = ref('')

const canCreate = computed(() => moduleName.value.trim().length > 0)

// Reset form when modal closes
watch(() => props.show, (newShow) => {
  if (!newShow) {
    moduleName.value = ''
    moduleType.value = 'general'
    moduleDescription.value = ''
  }
})

const handleCreate = () => {
  if (!canCreate.value) return

  emit('create', {
    name: moduleName.value.trim(),
    type: moduleType.value,
    description: moduleDescription.value.trim() || undefined
  })
}
</script>

<style scoped>
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

.optional {
  font-weight: 400;
  color: var(--color-text-muted);
}

.form-input,
.form-select,
.form-textarea {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  background: var(--color-background);
  color: var(--color-text);
  transition: border-color var(--transition-fast);
}

.form-input:focus,
.form-select:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px var(--color-primary-100);
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

.hint {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin: var(--spacing-xs) 0 0 0;
}
</style>
