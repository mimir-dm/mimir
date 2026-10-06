import { defineStore } from 'pinia'
import { ref, onUnmounted } from 'vue'
import { emit, listen } from '@tauri-apps/api/event'
import { THEMES } from '../constants/themes'
import type { UnlistenFn } from '@tauri-apps/api/event'

export const useThemeStore = defineStore('theme', () => {
  // Built-in list; there is no backend command for themes.
  const themes = THEMES
  const currentTheme = ref<string>('light')
  let unlistenThemeChange: UnlistenFn | null = null
  
  // Get saved theme preference from localStorage
  const getSavedTheme = (): string => {
    return localStorage.getItem('theme') || 'light'
  }
  
  // Save theme preference to localStorage
  const saveTheme = (theme: string) => {
    localStorage.setItem('theme', theme)
  }
  
  // Apply theme to the application
  const applyTheme = () => {
    const savedTheme = getSavedTheme()
    currentTheme.value = savedTheme
    applyThemeToBody(savedTheme)
  }

  // Apply theme class to body element for teleported components (modals)
  const applyThemeToBody = (theme: string) => {
    // Remove existing theme classes
    document.body.classList.remove(...THEMES.map((t) => `theme-${t.id}`))
    // Add new theme class
    document.body.classList.add(`theme-${theme}`)
  }
  
  // Change theme and broadcast to other windows
  const setTheme = async (theme: string, broadcast = true) => {
    currentTheme.value = theme
    saveTheme(theme)
    applyThemeToBody(theme)
    
    // Broadcast theme change to all windows
    if (broadcast) {
      try {
        await emit('theme-changed', { theme })
      } catch (error) {
      }
    }
  }
  
  // Initialize cross-window theme synchronization
  const initThemeSync = async () => {
    // Listen for theme changes from other windows
    unlistenThemeChange = await listen<{ theme: string }>('theme-changed', (event) => {
      // Update theme without broadcasting to avoid infinite loop
      setTheme(event.payload.theme, false)
    })
  }
  
  // Clean up event listener
  const cleanup = () => {
    if (unlistenThemeChange) {
      unlistenThemeChange()
      unlistenThemeChange = null
    }
  }
  
  // Auto-cleanup when store is disposed
  onUnmounted(() => {
    cleanup()
  })
  
  return {
    themes,
    currentTheme,
    applyTheme,
    setTheme,
    initThemeSync,
    cleanup
  }
})