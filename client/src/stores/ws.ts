import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from './auth'
import { useGuildStore } from './guild'
import { useVoiceStore } from './voice'
import { useActivityStore } from './activity'

type WsStatus = 'disconnected' | 'connecting' | 'connected'

export const useWsStore = defineStore('ws', () => {
  const status = ref<WsStatus>('disconnected')
  const socket = ref<WebSocket | null>(null)
  let heartbeatTimer: ReturnType<typeof setInterval> | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  let reconnectDelay = 1000

  function connect(serverUrl: string) {
    const auth = useAuthStore()
    if (!auth.accessToken) return

    const wsUrl = serverUrl.replace(/\/$/, '').replace(/^http/, 'ws') + '/ws'
    status.value = 'connecting'

    const ws = new WebSocket(wsUrl)
    socket.value = ws

    ws.onopen = () => {
      reconnectDelay = 1000
    }

    ws.onmessage = (e) => {
      try {
        handleEvent(JSON.parse(e.data))
      } catch {}
    }

    ws.onclose = () => {
      status.value = 'disconnected'
      clearHeartbeat()
      scheduleReconnect(serverUrl)
    }

    ws.onerror = () => {
      ws.close()
    }
  }

  function handleEvent(event: any) {
    const guild = useGuildStore()
    const voice = useVoiceStore()
    const auth = useAuthStore()
    const activity = useActivityStore()

    switch (event.op) {
      case 'HELLO':
        // Отправляем токен сразу после HELLO
        send({ op: 'IDENTIFY', d: { token: auth.accessToken } })
        startHeartbeat(event.d.heartbeat_interval)
        break

      case 'READY':
        status.value = 'connected'
        guild.setGuilds(event.d.guilds)
        break

      case 'MESSAGE_CREATE': {
        guild.addMessage(event.d.message)
        // Уведомление если нас упомянули в другом канале
        if (event.d.message.mention_user_ids?.includes(auth.userId)) {
          guild.markMention(event.d.message.channel_id)
        }
        // Добавление в ленту активностей
        const ch = guild.channels.find(c => c.id === event.d.message.channel_id)
        activity.addEvent({
          type: 'message_sent',
          actor: { id: event.d.message.author_id, username: event.d.message.author_username },
          targetLabel: ch ? '#' + ch.name : 'чат'
        })
        break
      }

      case 'MESSAGE_UPDATE':
        guild.updateMessage(event.d.message_id, event.d.content, event.d.edited_at)
        break

      case 'MESSAGE_DELETE':
        guild.deleteMessage(event.d.message_id)
        break

      case 'CHANNEL_CREATE': {
        guild.addChannel(event.d.channel)
        activity.addEvent({
          type: 'channel_created',
          actor: { id: 'system', username: 'Система' },
          targetLabel: '#' + event.d.channel.name
        })
        break
      }

      case 'CHANNEL_DELETE':
        guild.removeChannel(event.d.channel_id)
        break

      case 'REACTION_ADD':
        guild.handleReactionAdd(event.d)
        break

      case 'REACTION_REMOVE':
        guild.handleReactionRemove(event.d)
        break

      case 'VOICE_STATE_UPDATE': {
        const prevVoiceState = voice.voiceStates.get(event.d.user_id)
        const channelChanged = prevVoiceState?.channel_id !== event.d.channel_id
        
        voice.updateVoiceState(event.d)
        
        if (channelChanged && event.d.channel_id) {
          const member = guild.members.find(m => m.user_id === event.d.user_id)
          const name = member?.nickname || member?.username || event.d.user_id.slice(0, 8)
          const ch = guild.channels.find(c => c.id === event.d.channel_id)
          activity.addEvent({
            type: 'voice_joined',
            actor: { id: event.d.user_id, username: name },
            targetLabel: ch ? '🔊 ' + ch.name : 'Голос'
          })
        }
        break
      }

      case 'VOICE_SERVER_UPDATE':
        voice.connectToLiveKit(event.d.livekit_url, event.d.token)
        break

      case 'HEARTBEAT_ACK':
        break
    }
  }

  function send(payload: object) {
    if (socket.value?.readyState === WebSocket.OPEN) {
      socket.value.send(JSON.stringify(payload))
    }
  }

  function joinVoiceChannel(guildId: string, channelId: string | null) {
    send({
      op: 'VOICE_STATE_UPDATE',
      d: { guild_id: guildId, channel_id: channelId, is_muted: false, is_deafened: false },
    })
  }

  function startHeartbeat(interval: number) {
    let seq = 0
    heartbeatTimer = setInterval(() => {
      send({ op: 'HEARTBEAT', d: { seq: seq++ } })
    }, interval)
  }

  function clearHeartbeat() {
    if (heartbeatTimer) {
      clearInterval(heartbeatTimer)
      heartbeatTimer = null
    }
  }

  function scheduleReconnect(serverUrl: string) {
    reconnectTimer = setTimeout(() => {
      reconnectDelay = Math.min(reconnectDelay * 2, 30000)
      connect(serverUrl)
    }, reconnectDelay)
  }

  function disconnect() {
    clearHeartbeat()
    if (reconnectTimer) clearTimeout(reconnectTimer)
    socket.value?.close()
    socket.value = null
    status.value = 'disconnected'
  }

  return { status, connect, disconnect, send, joinVoiceChannel }
})
