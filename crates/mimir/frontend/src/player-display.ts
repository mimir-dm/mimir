import { createApp } from 'vue'
import { createPinia } from 'pinia'
import PlayerDisplayWindow from './components/PlayerDisplayWindow.vue'

// Import styles - minimal for display window, optimized for visibility
import './assets/styles/main.css'
import './assets/styles/themes/dark.css'

const app = createApp(PlayerDisplayWindow)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
