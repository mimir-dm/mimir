<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="visible"
        class="modal-overlay"
        :class="{ 'modal-stacked': stackIndex > 0 }"
        :style="stackIndex > 0 ? { '--stack-index': stackIndex } : undefined"
        @click="handleOverlayClick"
        @keydown.esc="handleEscape"
      >
        <div
          ref="modalRef"
          class="modal-content"
          :class="sizeClass"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="titleId"
          @click.stop
        >
          <!-- Header -->
          <div v-if="$slots.header || title" class="modal-header">
            <slot name="header">
              <h2 :id="titleId" class="modal-title">{{ title }}</h2>
            </slot>
            <button
              v-if="closable"
              type="button"
              class="modal-close"
              aria-label="Close modal"
              @click="close"
            >
              &times;
            </button>
          </div>

          <!-- Body -->
          <div class="modal-body" :class="{ 'no-padding': noPadding }">
            <slot></slot>
          </div>

          <!-- Footer -->
          <div v-if="$slots.footer" class="modal-footer">
            <slot name="footer"></slot>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'

export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | 'full'

interface Props {
  visible: boolean
  title?: string
  size?: ModalSize
  closable?: boolean
  closeOnOverlay?: boolean
  closeOnEscape?: boolean
  noPadding?: boolean
  stackIndex?: number
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  size: 'md',
  closable: true,
  closeOnOverlay: true,
  closeOnEscape: true,
  noPadding: false,
  stackIndex: 0
})

const emit = defineEmits<{
  close: []
  'update:visible': [value: boolean]
}>()

const modalRef = ref<HTMLElement | null>(null)
const titleId = computed(() => `modal-title-${Math.random().toString(36).substr(2, 9)}`)

const sizeClass = computed(() => {
  const sizeMap: Record<ModalSize, string> = {
    sm: 'modal-sm',
    md: 'modal-md',
    lg: 'modal-lg',
    xl: 'modal-xl',
    full: 'modal-full'
  }
  return sizeMap[props.size]
})

function close() {
  emit('close')
  emit('update:visible', false)
}

function handleOverlayClick() {
  if (props.closeOnOverlay) {
    close()
  }
}

function handleEscape(event: KeyboardEvent) {
  if (props.closeOnEscape && event.key === 'Escape') {
    event.preventDefault()
    close()
  }
}

// Global escape key handler (for when modal doesn't have focus)
function handleGlobalKeydown(event: KeyboardEvent) {
  if (props.visible && props.closeOnEscape && event.key === 'Escape') {
    event.preventDefault()
    close()
  }
}

// Body scroll lock
let originalOverflow = ''

function lockBodyScroll() {
  originalOverflow = document.body.style.overflow
  document.body.style.overflow = 'hidden'
}

function unlockBodyScroll() {
  document.body.style.overflow = originalOverflow
}

// Focus management
function focusModal() {
  nextTick(() => {
    if (modalRef.value) {
      // Focus the first focusable element or the modal itself
      const focusable = modalRef.value.querySelector<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      )
      if (focusable) {
        focusable.focus()
      } else {
        modalRef.value.setAttribute('tabindex', '-1')
        modalRef.value.focus()
      }
    }
  })
}

watch(
  () => props.visible,
  (newVisible) => {
    if (newVisible) {
      lockBodyScroll()
      focusModal()
    } else {
      unlockBodyScroll()
    }
  },
  { immediate: true }
)

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown)
  // Ensure scroll is unlocked if component is destroyed while visible
  if (props.visible) {
    unlockBodyScroll()
  }
})
</script>

<style scoped>
/* Transition animations */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: translateY(20px) scale(0.95);
  opacity: 0;
}
</style>
