import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '../api'

export const useAuthStore = defineStore('auth', () => {
  const accessToken = ref(localStorage.getItem('access_token') || '')
  const username = ref(localStorage.getItem('username') || '')
  const userId = ref(localStorage.getItem('user_id') || '')
  const emailVerified = ref(localStorage.getItem('email_verified') === 'true')

  const isLoggedIn = computed(() => !!accessToken.value)

  function setAuth(data: { access_token: string; refresh_token: string; username: string; user_id: string; email_verified: boolean }) {
    accessToken.value = data.access_token
    username.value = data.username
    userId.value = data.user_id
    emailVerified.value = data.email_verified
    localStorage.setItem('access_token', data.access_token)
    localStorage.setItem('refresh_token', data.refresh_token)
    localStorage.setItem('username', data.username)
    localStorage.setItem('user_id', data.user_id)
    localStorage.setItem('email_verified', String(data.email_verified))
  }

  async function login(login: string, password: string) {
    const { data } = await authApi.login(login, password)
    setAuth(data)
  }

  async function register(username: string, email: string, password: string) {
    const { data } = await authApi.register(username, email, password)
    setAuth(data)
  }

  function logout() {
    accessToken.value = ''
    username.value = ''
    userId.value = ''
    emailVerified.value = false
    localStorage.clear()
  }

  return { isLoggedIn, username, userId, emailVerified, login, register, logout }
})
