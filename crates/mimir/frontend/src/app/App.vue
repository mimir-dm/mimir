<template>
  <div id="app" :class="[currentTheme]">
    <router-view v-slot="{ Component }" :key="routeKey">
      <transition name="view" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useThemeStore } from '../stores/theme'
import { useChatStore } from '../stores/chat'
import { useRoute } from 'vue-router'
import { appDataDir } from '@tauri-apps/api/path'

const route = useRoute()
const routeKey = computed(() => {
  // For campaign routes, use the campaign ID to force refresh when switching campaigns
  if (route.params.id && route.path.includes('/campaigns/')) {
    return `campaign-${route.params.id}`
  }
  // For module routes, use the module ID
  if (route.params.id && route.path.includes('/modules/')) {
    return `module-${route.params.id}`
  }
  // For other routes, use the path
  return route.path
})

const themeStore = useThemeStore()
const chatStore = useChatStore()
const currentTheme = computed(() => `theme-${themeStore.currentTheme}`)

onMounted(async () => {
  // Load available themes from backend
  await themeStore.loadThemes()
  // Apply saved theme preference
  themeStore.applyTheme()
  // Initialize cross-window theme synchronization
  await themeStore.initThemeSync()
  
  // Configure todo storage during app initialization
  try {
    const dataDir = await appDataDir()
    const todoStoragePath = `${dataDir}todos`
    await chatStore.configureTodoStorage(todoStoragePath)
    console.log(`Configured todo storage to: ${todoStoragePath}`)
  } catch (err) {
    console.error('Failed to configure todo storage during app initialization:', err)
  }
})
</script>

<style>
#app {
  height: 100vh;
  overflow: hidden;
  background-color: var(--color-background);
  color: var(--color-text);
  transition: background-color 0.3s ease, color 0.3s ease;
}
</style>