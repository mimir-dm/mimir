import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  // Tauri expects the dist folder to be in this location
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL('./index.html', import.meta.url)),
        sources: fileURLToPath(new URL('./sources.html', import.meta.url)),
        contextDebug: fileURLToPath(new URL('./context-debug.html', import.meta.url)),
        chat: fileURLToPath(new URL('./chat.html', import.meta.url)),
        logViewer: fileURLToPath(new URL('./log-viewer.html', import.meta.url)),
        playerDisplay: fileURLToPath(new URL('./player-display.html', import.meta.url))
      }
    }
  },
  server: {
    port: 5173,
    strictPort: true
  }
})