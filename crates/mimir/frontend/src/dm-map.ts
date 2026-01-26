import { createApp } from 'vue'
import { createPinia } from 'pinia'
import DmMapWindow from './components/DmMapWindow.vue'

// Import styles
import './assets/styles/main.css'
import './assets/styles/themes/dark.css'

// Apply dark theme by default for DM Map window
// The DM Map window is used during play sessions where dark mode is preferred
document.body.classList.add('theme-dark')

const app = createApp(DmMapWindow)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
