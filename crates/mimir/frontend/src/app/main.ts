// Must be first: installs the dev-only browser IPC shim before anything
// touches @tauri-apps/api (no-op in production builds and in the real app).
import '../harness/bridge-shim'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import '../assets/styles/main.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')