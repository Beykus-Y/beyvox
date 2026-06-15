import { defineStore } from 'pinia'
import { ref } from 'vue'
import axios from 'axios'
import { useAuthStore } from './auth'

export interface Channel {
  id: string
  guild_id: string
  name: string
  type: 'text' | 'voice'
  position: number
  user_limit?: number
}

export interface Guild {
  id: string
  name: string
  description?: string
  icon_url?: string
  owner_id: string
  member_count: number
  channels?: Channel[]
}

export interface Message {
  id: string
  channel_id: string
  author_id: string
  author_username: string
  content: string
  created_at: string
  edited_at?: string
  reply_to?: string
}

export interface Member {
  user_id: string
  nickname?: string
  joined_at: string
  is_muted: boolean
}

export const useGuildStore = defineStore('guild', () => {
  const guilds = ref<Guild[]>([])
  const activeGuildId = ref<string | null>(null)
  const activeChannelId = ref<string | null>(null)
  const channels = ref<Channel[]>([])
  const messages = ref<Message[]>([])
  const members = ref<Member[]>([])

  // Адрес текущего подключённого сервера
  const serverUrl = ref(localStorage.getItem('server_url') || '')

  function apiClient() {
    const auth = useAuthStore()
    return axios.create({
      baseURL: serverUrl.value,
      headers: { Authorization: `Bearer ${auth.accessToken}` },
    })
  }

  async function connectToServer(url: string) {
    serverUrl.value = url.replace(/\/$/, '')
    localStorage.setItem('server_url', serverUrl.value)
  }

  async function createGuild(name: string, ownerToken?: string) {
    const { data } = await apiClient().post('/guilds', { name, owner_token: ownerToken || undefined })
    guilds.value.push(data)
    return data as Guild
  }

  function reset() {
    guilds.value = []
    activeGuildId.value = null
    activeChannelId.value = null
    channels.value = []
    messages.value = []
    members.value = []
  }

  async function joinByInvite(code: string) {
    const { data } = await apiClient().post(`/invites/${code}/join`)
    guilds.value.push(data.guild)
    return data
  }

  async function loadChannels(guildId: string) {
    const { data } = await apiClient().get(`/guilds/${guildId}/channels`)
    channels.value = data
    activeGuildId.value = guildId
  }

  async function loadMessages(guildId: string, channelId: string) {
    const { data } = await apiClient().get(
      `/guilds/${guildId}/channels/${channelId}/messages`
    )
    messages.value = data
    activeChannelId.value = channelId
  }

  async function sendMessage(guildId: string, channelId: string, content: string, replyTo?: string) {
    await apiClient().post(`/guilds/${guildId}/channels/${channelId}/messages`, {
      content,
      reply_to: replyTo,
    })
  }

  async function loadMembers(guildId: string) {
    const { data } = await apiClient().get(`/guilds/${guildId}/members`)
    members.value = data
  }

  function addMessage(msg: Message) {
    if (msg.channel_id === activeChannelId.value) {
      messages.value.push(msg)
    }
  }

  function setGuilds(list: Guild[]) {
    guilds.value = list
  }

  return {
    guilds, activeGuildId, activeChannelId, channels, messages, members, serverUrl,
    connectToServer, createGuild, joinByInvite, reset,
    loadChannels, loadMessages, sendMessage, loadMembers,
    addMessage, setGuilds,
  }
})
