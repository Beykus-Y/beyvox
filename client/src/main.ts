import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './style.css'

window.addEventListener('contextmenu', (e) => e.preventDefault())

createApp(App).use(createPinia()).use(router).mount('#app')
