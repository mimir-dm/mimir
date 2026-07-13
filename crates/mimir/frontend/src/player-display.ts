// Dev-only browser IPC shim for the UI harness (inert in the real app).
import './harness/bridge-shim'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import PlayerDisplayWindow from './components/PlayerDisplayWindow.vue'

// Import styles - minimal for display window, optimized for visibility
import './assets/styles/main.css'
import './assets/styles/themes/dark.css'

// Apply dark theme by default for Player Display window
document.body.classList.add('theme-dark')

const app = createApp(PlayerDisplayWindow)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
