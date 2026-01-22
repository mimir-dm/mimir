<template>
  <AppModal
    :visible="visible"
    title="Level Up"
    size="sm"
    @close="closeDialog"
  >
    <div class="dialog-body">
      <p>Level-up functionality is not yet available in this version.</p>
      <p class="text-secondary">
        Character progression features are planned for a future release.
      </p>
    </div>

    <template #footer>
      <button type="button" @click="closeDialog" class="btn btn-primary">
        Close
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import AppModal from '@/components/shared/AppModal.vue'
import type { Character } from '../../../types/character'

defineProps<{
  visible: boolean
  characterId: string
  characterData: Character
}>()

const emit = defineEmits<{
  close: []
  completed: []
}>()

const closeDialog = () => {
  emit('close')
}
</script>

<style scoped>
.dialog-body {
  padding: var(--spacing-lg);
  text-align: center;
}

.dialog-body p {
  margin-bottom: var(--spacing-md);
}

.text-secondary {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}
</style>
