import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'

export const useAuthStore = defineStore('auth', () => {
  const accessToken = ref(localStorage.getItem('access_token') || '')
  const refreshToken = ref(localStorage.getItem('refresh_token') || '')
  const username = ref(localStorage.getItem('username') || '')
  const userId = ref(localStorage.getItem('user_id') || '')
  const centralUrl = ref(localStorage.getItem('central_url') || 'https://beyvox.beykus.fun')

  const isLoggedIn = computed(() => !!accessToken.value)

  function setAuth(data: {
    access_token: string
    refresh_token: string
    username: string
    user_id: string
  }) {
    accessToken.value = data.access_token
    refreshToken.value = data.refresh_token
    username.value = data.username
    userId.value = data.user_id
    localStorage.setItem('access_token', data.access_token)
    localStorage.setItem('refresh_token', data.refresh_token)
    localStorage.setItem('username', data.username)
    localStorage.setItem('user_id', data.user_id)
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

  function logout() {
    accessToken.value = ''
    refreshToken.value = ''
    username.value = ''
    userId.value = ''
    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
    localStorage.removeItem('username')
    localStorage.removeItem('user_id')
  }

  function setCentralUrl(url: string) {
    centralUrl.value = url
    localStorage.setItem('central_url', url)
  }

  return { isLoggedIn, username, userId, accessToken, centralUrl, login, register, logout, setCentralUrl }
})
