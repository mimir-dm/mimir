<template>
  <div class="theme-selector">
    <label for="theme-select" class="text-sm font-medium text-secondary">
      Theme
    </label>
    <select
      id="theme-select"
      v-model="selectedTheme"
      @change="handleThemeChange"
      class="mt-1 block w-full rounded-md border-default bg-surface px-3 py-2 text-sm focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
    >
      <option
        v-for="theme in availableThemes"
        :key="theme.id"
        :value="theme.id"
      >
        {{ theme.name }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useThemeStore } from '../../../stores/theme'

const themeStore = useThemeStore()

const selectedTheme = computed({
  get: () => themeStore.currentTheme,
  set: async (value) => await themeStore.setTheme(value)
})

const availableThemes = computed(() => {
  // Default themes available without backend
  const defaultThemes = [
    { id: 'light', name: 'Light', description: 'Clean light theme with soft purples' },
    { id: 'dark', name: 'Dark', description: 'Deep blues and navy tones' },
    { id: 'hyper', name: 'Hyper', description: 'Vaporwave neon aesthetic' }
  ]
  
  // Merge with backend themes if available
  return themeStore.themes.length > 0 ? themeStore.themes : defaultThemes
})

const handleThemeChange = async () => {
  // Ensure the async setTheme completes
  await themeStore.setTheme(selectedTheme.value)
}
</script>

<style scoped>
.theme-selector {
  @apply space-y-1;
}

select {
  transition: border-color var(--transition-fast);
}

.text-secondary {
  color: var(--color-text-secondary);
}

.border-default {
  border: 1px solid var(--color-border);
}

.bg-surface {
  background-color: var(--color-surface);
}

.focus\:border-primary:focus {
  border-color: var(--color-primary-500);
}

.focus\:ring-primary:focus {
  --tw-ring-color: var(--color-primary-500);
}
</style>