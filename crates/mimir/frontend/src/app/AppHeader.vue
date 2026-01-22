<template>
  <header class="app-header">
    <div class="header-content">
      <div class="header-left">
        <router-link to="/" class="skull-icon-link" title="Home">
          <img :src="skullIcon" alt="Mimir" class="skull-icon" />
        </router-link>
        <div class="header-divider"></div>
        <CampaignSelector />
      </div>

      <nav class="header-nav">
        <router-link to="/characters" class="nav-link" title="Manage Characters">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
          </svg>
          <span>Characters</span>
        </router-link>
        <button @click="handleOpenRules" class="nav-link" title="Open Reference Library">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
          </svg>
          <span>Reference</span>
        </button>
      </nav>

      <div class="header-right">
        <button
          v-if="isDevMode"
          @click="showReseedConfirm"
          class="dev-button"
          :disabled="isReseeding"
          title="Reset test data (dev only)"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M23 4v6h-6"/>
            <path d="M1 20v-6h6"/>
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          <span>{{ isReseeding ? 'Reseeding...' : 'Reseed' }}</span>
        </button>
        <router-link to="/settings" class="settings-icon" title="Settings">
          <img :src="gearIcon" alt="Settings" class="gear-icon" />
        </router-link>
      </div>
    </div>
  </header>

  <!-- Reseed Confirmation Modal -->
  <AppModal
    :visible="showConfirmModal"
    title="Reset Test Data"
    size="sm"
    @close="cancelReseed"
  >
    <p>This will delete and recreate all test data for "The Lost Mine of Phandelver" campaign.</p>
    <p class="warning-text">This action cannot be undone.</p>

    <div v-if="reseedError" class="error-message">
      {{ reseedError }}
    </div>

    <template #footer>
      <div class="modal-actions">
        <button @click="cancelReseed" class="btn btn-secondary">Cancel</button>
        <button @click="confirmReseed" class="btn btn-warning" :disabled="isReseeding">
          {{ isReseeding ? 'Reseeding...' : 'Reset Data' }}
        </button>
      </div>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useThemeStore } from '../stores/theme'
import { invoke } from '@tauri-apps/api/core'
import type { ApiResponse } from '../types/api'
import CampaignSelector from '../features/campaigns/components/CampaignSelector.vue'
import AppModal from '@/components/shared/AppModal.vue'
import { openSourcesReference } from '../shared/utils/windows'
// Gear icons
import lightGear from '../assets/images/themes/light/gear.png'
import darkGear from '../assets/images/themes/dark/gear.png'
import hyperGear from '../assets/images/themes/hyper/gear.png'
// Skull icons
import lightMimir from '../assets/images/themes/light/mimir.png'
import darkMimir from '../assets/images/themes/dark/mimir.png'
import hyperMimir from '../assets/images/themes/hyper/mimir.png'

const themeStore = useThemeStore()

// Dev mode state
const isDevMode = ref(false)
const isReseeding = ref(false)
const showConfirmModal = ref(false)
const reseedError = ref<string | null>(null)

onMounted(async () => {
  try {
    isDevMode.value = await invoke<boolean>('is_dev_mode')
  } catch (error) {
    // Dev mode command not available - that's fine, isDevMode stays false
  }
})

// Show reseed confirmation modal
const showReseedConfirm = () => {
  reseedError.value = null
  showConfirmModal.value = true
}

// Cancel reseed
const cancelReseed = () => {
  showConfirmModal.value = false
  reseedError.value = null
}

// Confirm and perform reseed
const confirmReseed = async () => {
  isReseeding.value = true
  reseedError.value = null

  try {
    const response = await invoke<ApiResponse<null>>('reseed_dev_data')
    if (response.success) {
      showConfirmModal.value = false
      window.location.reload()
    } else {
      reseedError.value = response.error || 'Failed to reset test data'
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : JSON.stringify(error)
    reseedError.value = `Failed to reset test data: ${errorMsg}`
  } finally {
    isReseeding.value = false
  }
}

// Handle opening the rules reference window
const handleOpenRules = async () => {
  try {
    await openSourcesReference()
  } catch (error) {
  }
}

// Dynamically select gear icon based on current theme
const gearIcon = computed(() => {
  switch (themeStore.currentTheme) {
    case 'dark':
      return darkGear
    case 'hyper':
      return hyperGear
    default:
      return lightGear
  }
})

// Dynamically select skull icon based on current theme
const skullIcon = computed(() => {
  switch (themeStore.currentTheme) {
    case 'dark':
      return darkMimir
    case 'hyper':
      return hyperMimir
    default:
      return lightMimir
  }
})
</script>

<style scoped>
.app-header {
  background-color: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  box-shadow: var(--shadow-sm);
}

.header-content {
  max-width: 1280px;
  margin: 0 auto;
  padding: 0 30px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: relative;
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.skull-icon-link {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
  cursor: pointer;
}

.skull-icon-link:hover {
  background-color: var(--color-surface-variant);
}

.skull-icon {
  width: 44px;
  height: 44px;
  object-fit: contain;
  /* Removed decorative glow effect for theme consistency */
  transition: transform var(--transition-fast);
  /* Scale up to eat negative space in the image */
  transform: scale(1.2);
}

.skull-icon-link:hover .skull-icon {
  transform: scale(1.35);
}

.header-center {
  flex: 1;
  /* Empty spacer to push left and right elements to sides */
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.settings-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
  cursor: pointer;
}

.settings-icon:hover {
  color: var(--color-text);
  background-color: var(--color-surface-variant);
}

.gear-icon {
  width: 44px;
  height: 44px;
  transition: transform var(--transition-fast);
  /* Scale up to eat negative space in the image */
  transform: scale(1.2);
}

.settings-icon:hover .gear-icon {
  transform: rotate(45deg) scale(1.25);
}

/* Header divider between logo and campaign selector */
.header-divider {
  width: 1px;
  height: 24px;
  background-color: var(--color-border);
  opacity: 0.6;
}

/* Center navigation */
.header-nav {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
}

.nav-link {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  background-color: transparent;
  color: var(--color-text-secondary);
  border: none;
  font-size: 0.875rem;
  font-weight: 500;
  text-decoration: none;
  transition: all var(--transition-fast);
  cursor: pointer;
}

.nav-link:hover {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
}

.nav-link:active {
  transform: scale(0.98);
}

.nav-link svg {
  opacity: 0.7;
  transition: opacity var(--transition-fast);
}

.nav-link:hover svg {
  opacity: 1;
}

/* Active nav link state */
.nav-link.router-link-active {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
}

.nav-link.router-link-active svg {
  opacity: 1;
}

/* Dev-only button */
.dev-button {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  background-color: var(--color-warning-500, #f59e0b);
  color: white;
  border: none;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all var(--transition-fast);
  cursor: pointer;
  box-shadow: var(--shadow-sm);
}

.dev-button:hover:not(:disabled) {
  background-color: var(--color-warning-600, #d97706);
  box-shadow: var(--shadow);
  transform: translateY(-1px);
}

.dev-button:active:not(:disabled) {
  transform: translateY(0) scale(0.98);
  box-shadow: var(--shadow-sm);
}

.dev-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.dev-button svg {
  opacity: 0.9;
}

/* Modal styles */
.warning-text {
  color: var(--color-warning-600, #d97706);
  font-size: 0.875rem;
  margin-top: var(--spacing-sm);
}

.error-message {
  background-color: var(--color-error-100);
  color: var(--color-error-700);
  border: 1px solid var(--color-error-300);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  margin-top: var(--spacing-md);
  font-size: 0.875rem;
}

.theme-dark .error-message {
  background-color: var(--color-error-900);
  color: var(--color-error-300);
  border-color: var(--color-error-700);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

.btn {
  padding: var(--spacing-sm) var(--spacing-lg);
  border-radius: var(--radius-md);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: none;
}

.btn-secondary {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  background-color: var(--color-gray-200);
}

.theme-dark .btn-secondary:hover {
  background-color: var(--color-gray-700);
}

.btn-warning {
  background-color: var(--color-warning-500, #f59e0b);
  color: white;
}

.btn-warning:hover:not(:disabled) {
  background-color: var(--color-warning-600, #d97706);
}

.btn-warning:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

</style>