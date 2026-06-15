import { createRouter, createWebHistory } from 'vue-router'
import Landing from './views/Landing.vue'
import Servers from './views/Servers.vue'
import Auth from './views/Auth.vue'

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: Landing },
    { path: '/servers', component: Servers },
    { path: '/login', component: Auth, props: { mode: 'login' } },
    { path: '/register', component: Auth, props: { mode: 'register' } },
  ],
})
