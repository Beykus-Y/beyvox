import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'

export interface ServerInstance {
  url: string
  name: string
  requiresOwnerToken: boolean
}

const STORAGE_KEY = 'beyvox_servers'

function load(): ServerInstance[] {
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]') } catch { return [] }
}

function save(list: ServerInstance[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(list))
}

export const useServersStore = defineStore('servers', () => {
  const servers = ref<ServerInstance[]>(load())
  const activeUrl = ref<string | null>(localStorage.getItem('active_server_url'))

  const activeServer = computed(() => servers.value.find(s => s.url === activeUrl.value) ?? null)

  async function addServer(url: string): Promise<ServerInstance> {
    const clean = url.replace(/\/$/, '')
    const { data } = await axios.get(`${clean}/info`)
    const server: ServerInstance = {
      url: clean,
      name: data.name ?? clean,
      requiresOwnerToken: data.requires_owner_token ?? true,
    }
    // Обновляем если уже есть, добавляем если нет
    const idx = servers.value.findIndex(s => s.url === clean)
    if (idx >= 0) servers.value[idx] = server
    else servers.value.push(server)
    save(servers.value)
    return server
  }

  function setActive(url: string | null) {
    activeUrl.value = url
    if (url) localStorage.setItem('active_server_url', url)
    else localStorage.removeItem('active_server_url')
  }

  function removeServer(url: string) {
    servers.value = servers.value.filter(s => s.url !== url)
    save(servers.value)
    if (activeUrl.value === url) setActive(null)
  }

  return { servers, activeUrl, activeServer, addServer, setActive, removeServer }
})
