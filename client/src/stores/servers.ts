import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'

export interface ServerInstance {
  url: string
  name: string
  requiresOwnerToken: boolean
  pingMs?: number | null
  online?: boolean
  bannerUrl?: string
  guildsCount?: number
  onlineCount?: number
  lastSeenAt?: string
}

const STORAGE_KEY = 'beyvox_servers'

function load(): ServerInstance[] {
  try {
    const list = JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]') as ServerInstance[]
    return list.map(s => ({
      ...s,
      pingMs: null,
      online: false
    }))
  } catch {
    return []
  }
}

function save(list: ServerInstance[]) {
  // Сохраняем только персистентные свойства
  const clean = list.map(({ url, name, requiresOwnerToken, bannerUrl }) => ({
    url,
    name,
    requiresOwnerToken,
    bannerUrl
  }))
  localStorage.setItem(STORAGE_KEY, JSON.stringify(clean))
}

export const useServersStore = defineStore('servers', () => {
  const servers = ref<ServerInstance[]>(load())
  const activeUrl = ref<string | null>(localStorage.getItem('active_server_url'))

  const activeServer = computed(() => servers.value.find(s => s.url === activeUrl.value) ?? null)

  async function checkPing(server: ServerInstance) {
    const start = Date.now()
    try {
      // Пингуем через запрос к эндпоинту /info
      const { data } = await axios.get(`${server.url}/info`, { timeout: 3000 })
      server.pingMs = Date.now() - start
      server.online = true
      if (data.name) server.name = data.name
      server.lastSeenAt = new Date().toISOString()
    } catch {
      server.pingMs = null
      server.online = false
    }
  }

  async function checkAllPings() {
    await Promise.all(servers.value.map(s => checkPing(s)))
  }

  async function addServer(url: string): Promise<ServerInstance> {
    const clean = url.replace(/\/$/, '')
    const { data } = await axios.get(`${clean}/info`)
    const server: ServerInstance = {
      url: clean,
      name: data.name ?? clean,
      requiresOwnerToken: data.requires_owner_token ?? true,
      online: true,
      pingMs: null
    }
    
    // Получаем пинг сразу
    await checkPing(server)

    const idx = servers.value.findIndex(s => s.url === clean)
    if (idx >= 0) {
      servers.value[idx] = { ...servers.value[idx], ...server }
    } else {
      servers.value.push(server)
    }
    save(servers.value)
    return servers.value.find(s => s.url === clean) ?? server
  }

  function setActive(url: string | null) {
    activeUrl.value = url
    if (url) {
      localStorage.setItem('active_server_url', url)
      const s = servers.value.find(sv => sv.url === url)
      if (s) {
        s.online = true
        checkPing(s)
      }
    } else {
      localStorage.removeItem('active_server_url')
    }
  }

  function removeServer(url: string) {
    servers.value = servers.value.filter(s => s.url !== url)
    save(servers.value)
    if (activeUrl.value === url) setActive(null)
  }

  return { servers, activeUrl, activeServer, addServer, setActive, removeServer, checkPing, checkAllPings }
})
