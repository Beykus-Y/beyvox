import axios from 'axios'

export const api = axios.create({
  baseURL: '', // динамически задаётся при подключении к серверу
})

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token')
  if (token) config.headers.Authorization = `Bearer ${token}`
  return config
})

api.interceptors.response.use(
  (r) => r,
  async (error) => {
    if (error.response?.status === 401) {
      const refresh = localStorage.getItem('refresh_token')
      if (refresh) {
        try {
          const centralUrl = import.meta.env.VITE_CENTRAL_URL || 'https://beyvox.beykus.fun'
          const { data } = await axios.post(`${centralUrl}/auth/refresh`, {
            refresh_token: refresh,
          })
          localStorage.setItem('access_token', data.access_token)
          localStorage.setItem('refresh_token', data.refresh_token)
          error.config.headers.Authorization = `Bearer ${data.access_token}`
          return api(error.config)
        } catch {
          localStorage.clear()
          window.location.reload()
        }
      }
    }
    return Promise.reject(error)
  }
)

export const centralApi = (baseURL: string) => axios.create({ baseURL })

export const serverApi = (serverUrl: string) =>
  axios.create({
    baseURL: serverUrl,
    headers: {
      Authorization: `Bearer ${localStorage.getItem('access_token')}`,
    },
  })

const CENTRAL_URL = import.meta.env.VITE_CENTRAL_URL || 'https://beyvox.beykus.fun'

export interface PublicServer {
  id: string
  name: string
  description: string | null
  icon_url: string | null
  address: string
  tags: string[]
  online_count: number
  total_members: number
}

export async function fetchPublicServers(): Promise<PublicServer[]> {
  const res = await fetch(`${CENTRAL_URL}/api/servers`)
  if (!res.ok) throw new Error('Не удалось загрузить список серверов')
  return res.json()
}
