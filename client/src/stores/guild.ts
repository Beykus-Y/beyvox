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
  description?: string
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
  timeout_until?: string
  role_ids: string[]
}

export interface Role {
  id: string
  guild_id: string
  name: string
  color?: string
  permissions: number
  position: number
}

export interface Invite {
  code: string
  guild_id: string
  uses: number
  max_uses?: number
  expires_at?: string
}

// Битовые маски прав (зеркало сервера)
export const PERM = {
  ADMINISTRATOR:   1 << 0,
  MANAGE_CHANNELS: 1 << 1,
  MANAGE_ROLES:    1 << 2,
  MANAGE_MEMBERS:  1 << 3,
  SEND_MESSAGES:   1 << 4,
  ATTACH_FILES:    1 << 5,
  CONNECT_VOICE:   1 << 6,
  STREAM_SCREEN:   1 << 7,
  MUTE_MEMBERS:    1 << 8,
  BAN_MEMBERS:     1 << 9,
} as const

export const useGuildStore = defineStore('guild', () => {
  const guilds = ref<Guild[]>([])
  const activeGuildId = ref<string | null>(null)
  const activeChannelId = ref<string | null>(null)
  const channels = ref<Channel[]>([])
  const messages = ref<Message[]>([])
  const members = ref<Member[]>([])
  const roles = ref<Role[]>([])
  const invites = ref<Invite[]>([])
  // Мои права на текущей гильдии (i64 битовая маска, -1 = owner/все права)
  const myPermissions = ref<number>(0)

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

  async function createGuild(name: string) {
    const { data } = await apiClient().post('/guilds', { name })
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
    roles.value = []
    invites.value = []
    myPermissions.value = 0
    mentionedChannels.value = new Set()
  }

  function hasPermission(perm: number): boolean {
    // -1 означает все права (owner или результат i64::MAX сервера)
    if (myPermissions.value === -1) return true
    return (myPermissions.value & PERM.ADMINISTRATOR) !== 0 || (myPermissions.value & perm) !== 0
  }

  async function joinByInvite(code: string) {
    const { data } = await apiClient().post(`/invites/${code}/join`)
    guilds.value.push(data)
    return data
  }

  async function createInvite(guildId: string): Promise<string> {
    const { data } = await apiClient().post(`/guilds/${guildId}/invites`, {})
    return data.code as string
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

  async function editMessage(guildId: string, channelId: string, messageId: string, content: string) {
    await apiClient().patch(`/guilds/${guildId}/channels/${channelId}/messages/${messageId}`, {
      content,
    })
  }

  async function removeMessageFromServer(guildId: string, channelId: string, messageId: string) {
    await apiClient().delete(`/guilds/${guildId}/channels/${channelId}/messages/${messageId}`)
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
    members.value = data.map((m: any) => ({ ...m, role_ids: m.role_ids ?? [] }))
  }

  async function loadMyPermissions(guildId: string) {
    try {
      const auth = useAuthStore()
      const guild = guilds.value.find(g => g.id === guildId)
      if (guild?.owner_id === auth.userId) {
        myPermissions.value = -1
        return
      }
      const { data } = await apiClient().get(`/guilds/${guildId}/members/me`)
      // Вычисляем права локально: OR всех ролей юзера
      if (data.role_ids?.length && roles.value.length) {
        let perms = 0
        for (const rid of data.role_ids) {
          const role = roles.value.find(r => r.id === rid)
          if (role) perms |= role.permissions
        }
        const everyone = roles.value.find(r => r.name === '@everyone')
        if (everyone) perms |= everyone.permissions
        myPermissions.value = perms
      }
    } catch {
      myPermissions.value = 0
    }
  }

  async function loadRoles(guildId: string) {
    const { data } = await apiClient().get(`/guilds/${guildId}/roles`)
    roles.value = data
  }

  async function createRole(guildId: string, body: { name: string; color?: string; permissions?: number }) {
    const { data } = await apiClient().post(`/guilds/${guildId}/roles`, body)
    roles.value.push(data)
    return data as Role
  }

  async function updateRole(guildId: string, roleId: string, patch: { name?: string; color?: string; permissions?: number }) {
    const { data } = await apiClient().patch(`/guilds/${guildId}/roles/${roleId}`, patch)
    const idx = roles.value.findIndex(r => r.id === roleId)
    if (idx !== -1) roles.value[idx] = data
    return data as Role
  }

  async function deleteRole(guildId: string, roleId: string) {
    await apiClient().delete(`/guilds/${guildId}/roles/${roleId}`)
    roles.value = roles.value.filter(r => r.id !== roleId)
  }

  async function assignRole(guildId: string, userId: string, roleId: string) {
    await apiClient().put(`/guilds/${guildId}/members/${userId}/roles/${roleId}`)
  }

  async function removeRole(guildId: string, userId: string, roleId: string) {
    await apiClient().delete(`/guilds/${guildId}/members/${userId}/roles/${roleId}`)
  }

  async function kickMember(guildId: string, userId: string) {
    await apiClient().post(`/guilds/${guildId}/members/${userId}/kick`)
    members.value = members.value.filter(m => m.user_id !== userId)
  }

  async function banMember(guildId: string, userId: string) {
    await apiClient().post(`/guilds/${guildId}/members/${userId}/ban`)
    members.value = members.value.filter(m => m.user_id !== userId)
  }

  async function muteMember(guildId: string, userId: string, muted: boolean) {
    await apiClient().post(`/guilds/${guildId}/members/${userId}/mute`, { muted })
    const m = members.value.find(m => m.user_id === userId)
    if (m) m.is_muted = muted
  }

  async function timeoutMember(guildId: string, userId: string, seconds: number) {
    await apiClient().post(`/guilds/${guildId}/members/${userId}/timeout`, { seconds })
  }

  async function loadInvites(guildId: string) {
    const { data } = await apiClient().get(`/guilds/${guildId}/invites`)
    invites.value = data
  }

  async function createInviteWithOptions(guildId: string, opts: { maxUses?: number; expiresHours?: number } = {}) {
    const { data } = await apiClient().post(`/guilds/${guildId}/invites`, {
      max_uses: opts.maxUses,
      expires_hours: opts.expiresHours,
    })
    invites.value.unshift(data)
    return data as Invite
  }

  async function deleteInvite(guildId: string, code: string) {
    await apiClient().delete(`/guilds/${guildId}/invites/${code}`)
    invites.value = invites.value.filter(i => i.code !== code)
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

  function handleMemberUpdate(data: any) {
    const idx = members.value.findIndex(m => m.user_id === data.member.user_id)
    const updated = { ...data.member, role_ids: data.member.role_ids ?? [] }
    if (idx !== -1) {
      members.value[idx] = updated
    } else {
      members.value.push(updated)
    }
    // Обновляем свои права если это мы
    const auth = useAuthStore()
    if (data.member.user_id === auth.userId) {
      loadMyPermissions(data.guild_id)
    }
  }

  function handleMemberRemove(data: { guild_id: string; user_id: string }) {
    members.value = members.value.filter(m => m.user_id !== data.user_id)
  }

  function handleRoleCreate(data: { guild_id: string; role: Role }) {
    if (!roles.value.find(r => r.id === data.role.id)) {
      roles.value.push(data.role)
      roles.value.sort((a, b) => a.position - b.position)
    }
  }

  function handleRoleUpdate(data: { guild_id: string; role: Role }) {
    const idx = roles.value.findIndex(r => r.id === data.role.id)
    if (idx !== -1) roles.value[idx] = data.role
  }

  function handleRoleDelete(data: { guild_id: string; role_id: string }) {
    roles.value = roles.value.filter(r => r.id !== data.role_id)
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
    roles, invites, myPermissions,
    mentionedChannels,
    connectToServer, createGuild, createChannel, joinByInvite, createInvite, reset,
    loadChannels, loadMessages, loadMoreMessages, sendMessage, editMessage, removeMessageFromServer, loadMembers,
    loadMyPermissions, hasPermission,
    loadRoles, createRole, updateRole, deleteRole, assignRole, removeRole,
    kickMember, banMember, muteMember, timeoutMember,
    loadInvites, createInviteWithOptions, deleteInvite,
    addReaction, removeReaction,
    addMessage, updateMessage, deleteMessage, addChannel, removeChannel, setGuilds,
    handleReactionAdd, handleReactionRemove, markMention, clearMention,
    handleMemberUpdate, handleMemberRemove, handleRoleCreate, handleRoleUpdate, handleRoleDelete,
  }
})
