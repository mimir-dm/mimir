<template>
  <AppModal
    :visible="current !== null"
    :title="current?.options.title"
    size="sm"
    :stack-index="50"
    @close="cancel"
  >
    <p v-if="current" class="dialog-message" data-testid="dialog-message">{{ current.options.message }}</p>
    <template #footer>
      <template v-if="current?.kind === 'confirm'">
        <button type="button" class="btn btn-secondary" data-testid="dialog-cancel" @click="cancel">
          {{ current.options.cancelLabel ?? 'Cancel' }}
        </button>
        <button
          type="button"
          class="btn"
          :class="current.options.danger ? 'btn-danger' : 'btn-primary'"
          data-testid="dialog-confirm"
          @click="accept"
        >
          {{ current.options.confirmLabel ?? 'OK' }}
        </button>
      </template>
      <button v-else-if="current" type="button" class="btn btn-primary" data-testid="dialog-ok" @click="accept">
        {{ current.options.okLabel ?? 'OK' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
/**
 * Shows the dialogs of useDialog() (MIMIR-T-0684). Mount one per window root.
 * The dialog stacks above any open modal; Esc closes the dialog only.
 */
import { computed, onMounted, onUnmounted } from 'vue'
import AppModal from '@/components/shared/AppModal.vue'
import { pendingDialogs, registerDialogHost, settleDialog } from '@/composables/useDialog'

const current = computed(() => pendingDialogs.value[0] ?? null)

function accept() {
  if (current.value) settleDialog(current.value.id, true)
}

function cancel() {
  if (current.value) settleDialog(current.value.id, false)
}

// Capture phase: the dialog takes Esc before the modal under it sees it.
function onKeydown(event: KeyboardEvent) {
  if (event.key !== 'Escape' || !current.value) return
  event.preventDefault()
  event.stopImmediatePropagation()
  cancel()
}

let unregister: (() => void) | null = null

onMounted(() => {
  unregister = registerDialogHost()
  window.addEventListener('keydown', onKeydown, true)
})

onUnmounted(() => {
  unregister?.()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<style scoped>
.dialog-message {
  margin: 0;
  white-space: pre-line;
  line-height: 1.5;
}
</style>
