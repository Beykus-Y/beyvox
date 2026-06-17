import { createRouter, createWebHashHistory } from 'vue-router'
import Login from './views/Login.vue'
import AppView from './views/App.vue'

const router = createRouter({
  history: createWebHashHistory(), // hash router для Tauri (нет HTTP сервера)
  routes: [
    {
      path: '/',
      redirect: () => localStorage.getItem('access_token') ? '/app' : '/login',
    },
    { path: '/login', component: Login },
    { path: '/app', component: AppView },
  ],
})

router.beforeEach((to) => {
  const loggedIn = !!localStorage.getItem('access_token')
  if (to.path === '/login' && loggedIn) return '/app'
  if (to.path === '/app' && !loggedIn) return '/login'
})

export default router
