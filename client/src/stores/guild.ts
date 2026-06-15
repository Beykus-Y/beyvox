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

export interface Reaction {
  emoji: string
  count: number
  me: boolean
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
  mention_user_ids: string[]
  reactions: Reaction[]
}

export interface Member {
  user_id: string
  username: string
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

  // Каналы с непрочитанными упоминаниями
  const mentionedChannels = ref<Set<string>>(new Set())

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

  async function createChannel(guildId: string, name: string, type: 'text' | 'voice', userLimit?: number) {
    const { data } = await apiClient().post(`/guilds/${guildId}/channels`, {
      name,
      type,
      user_limit: userLimit || undefined,
    })
    return data as Channel
  }

  function reset() {
    guilds.value = []
    activeGuildId.value = null
    activeChannelId.value = null
    channels.value = []
    messages.value = []
    members.value = []
    mentionedChannels.value = new Set()
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
    messages.value = data.map(normalizeMessage)
    activeChannelId.value = channelId
    mentionedChannels.value.delete(channelId)
  }

  async function loadMoreMessages(guildId: string, channelId: string) {
    const oldest = messages.value[0]
    if (!oldest) return
    const { data } = await apiClient().get(
      `/guilds/${guildId}/channels/${channelId}/messages`,
      { params: { before: oldest.id, limit: 50 } }
    )
    if (data.length > 0) {
      messages.value = [...data.map(normalizeMessage), ...messages.value]
    }
  }

  async function sendMessage(guildId: string, channelId: string, content: string, replyTo?: string) {
    await apiClient().post(`/guilds/${guildId}/channels/${channelId}/messages`, {
      content,
      reply_to: replyTo,
    })
  }

  async function addReaction(guildId: string, channelId: string, messageId: string, emoji: string) {
    await apiClient().put(
      `/guilds/${guildId}/channels/${channelId}/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`
    )
  }

  async function removeReaction(guildId: string, channelId: string, messageId: string, emoji: string) {
    await apiClient().delete(
      `/guilds/${guildId}/channels/${channelId}/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`
    )
  }

  async function loadMembers(guildId: string) {
    const { data } = await apiClient().get(`/guilds/${guildId}/members`)
    members.value = data
  }

  function addMessage(msg: Message) {
    if (msg.channel_id === activeChannelId.value) {
      messages.value.push(normalizeMessage(msg))
    } else {
      // Если сообщение не в активном канале и есть упоминание — помечаем
      const auth = useAuthStore()
      if (msg.mention_user_ids?.includes(auth.userId)) {
        mentionedChannels.value.add(msg.channel_id)
      }
    }
    // Упоминание в активном канале — не ставим значок (пользователь видит)
  }

  function updateMessage(messageId: string, content: string, editedAt: string) {
    const msg = messages.value.find(m => m.id === messageId)
    if (msg) {
      msg.content = content
      msg.edited_at = editedAt
    }
  }

  function deleteMessage(messageId: string) {
    messages.value = messages.value.filter(m => m.id !== messageId)
  }

  function addChannel(channel: Channel) {
    if (!channels.value.find(c => c.id === channel.id)) {
      channels.value.push(channel)
      channels.value.sort((a, b) => a.position - b.position)
    }
  }

  function removeChannel(channelId: string) {
    channels.value = channels.value.filter(c => c.id !== channelId)
    if (activeChannelId.value === channelId) {
      activeChannelId.value = null
      messages.value = []
    }
  }

  function setGuilds(list: Guild[]) {
    guilds.value = list
  }

  function handleReactionAdd(data: { message_id: string; user_id: string; emoji: string }) {
    const msg = messages.value.find(m => m.id === data.message_id)
    if (!msg) return
    const auth = useAuthStore()
    const isMe = data.user_id === auth.userId
    const existing = msg.reactions.find(r => r.emoji === data.emoji)
    if (existing) {
      existing.count++
      if (isMe) existing.me = true
    } else {
      msg.reactions.push({ emoji: data.emoji, count: 1, me: isMe })
    }
  }

  function handleReactionRemove(data: { message_id: string; user_id: string; emoji: string }) {
    const msg = messages.value.find(m => m.id === data.message_id)
    if (!msg) return
    const auth = useAuthStore()
    const isMe = data.user_id === auth.userId
    const idx = msg.reactions.findIndex(r => r.emoji === data.emoji)
    if (idx === -1) return
    if (msg.reactions[idx].count <= 1) {
      msg.reactions.splice(idx, 1)
    } else {
      msg.reactions[idx].count--
      if (isMe) msg.reactions[idx].me = false
    }
  }

  function markMention(channelId: string) {
    if (channelId !== activeChannelId.value) {
      mentionedChannels.value.add(channelId)
    }
  }

  function clearMention(channelId: string) {
    mentionedChannels.value.delete(channelId)
  }

  function normalizeMessage(msg: any): Message {
    return {
      ...msg,
      mention_user_ids: msg.mention_user_ids ?? [],
      reactions: msg.reactions ?? [],
    }
  }

  return {
    guilds, activeGuildId, activeChannelId, channels, messages, members, serverUrl,
    mentionedChannels,
    connectToServer, createGuild, createChannel, joinByInvite, reset,
    loadChannels, loadMessages, loadMoreMessages, sendMessage, loadMembers,
    addReaction, removeReaction,
    addMessage, updateMessage, deleteMessage, addChannel, removeChannel, setGuilds,
    handleReactionAdd, handleReactionRemove, markMention, clearMention,
  }
})
