import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'

const DEFAULT_CENTRAL_URL = import.meta.env.VITE_CENTRAL_URL || 'https://beyvox.beykus.fun'

export const useAuthStore = defineStore('auth', () => {
  const accessToken = ref(localStorage.getItem('access_token') || '')
  const refreshToken = ref(localStorage.getItem('refresh_token') || '')
  const username = ref(localStorage.getItem('username') || '')
  const userId = ref(localStorage.getItem('user_id') || '')
  const emailVerified = ref(localStorage.getItem('email_verified') === 'true')
  const centralUrl = ref(DEFAULT_CENTRAL_URL)

  const isLoggedIn = computed(() => !!accessToken.value)

  function setAuth(data: {
    access_token: string
    refresh_token: string
    username: string
    user_id: string
    email_verified: boolean
  }) {
    accessToken.value = data.access_token
    refreshToken.value = data.refresh_token
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
    const { data } = await axios.post(`${centralUrl.value}/auth/login`, { login, password })
    setAuth(data)
  }

  async function register(username: string, email: string, password: string) {
    const { data } = await axios.post(`${centralUrl.value}/auth/register`, {
      username,
      email,
      password,
    })
    setAuth(data)
  }

  async function checkVerificationStatus(): Promise<boolean> {
    if (!accessToken.value) return false
    const { data } = await axios.get(`${centralUrl.value}/auth/status`, {
      headers: { Authorization: `Bearer ${accessToken.value}` }
    })
    emailVerified.value = data.email_verified
    localStorage.setItem('email_verified', String(data.email_verified))
    return data.email_verified
  }

  function logout() {
    accessToken.value = ''
    refreshToken.value = ''
    username.value = ''
    userId.value = ''
    emailVerified.value = false
    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
    localStorage.removeItem('username')
    localStorage.removeItem('user_id')
    localStorage.removeItem('email_verified')
  }

  return { isLoggedIn, username, userId, accessToken, emailVerified, centralUrl, login, register, checkVerificationStatus, logout }
})
