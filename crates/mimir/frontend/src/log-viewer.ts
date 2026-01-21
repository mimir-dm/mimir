import { createApp } from 'vue'
import { createPinia } from 'pinia'
import LogViewerWindow from './components/LogViewerWindow.vue'

// Import all the same styles as the main app
import './assets/styles/main.css'
import './assets/styles/themes/light.css'
import './assets/styles/themes/dark.css'
import './assets/styles/themes/hyper.css'
import './assets/styles/components.css'
import './assets/styles/utilities/animations.css'

const app = createApp(LogViewerWindow)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')