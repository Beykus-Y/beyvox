import { createRouter, createWebHashHistory } from 'vue-router'
import Login from './views/Login.vue'
import AppView from './views/App.vue'

export default createRouter({
  history: createWebHashHistory(), // hash router для Tauri (нет HTTP сервера)
  routes: [
    { path: '/', redirect: '/login' },
    { path: '/login', component: Login },
    { path: '/app', component: AppView },
  ],
})
