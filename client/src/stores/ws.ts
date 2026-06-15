import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from './auth'
import { useGuildStore } from './guild'
import { useVoiceStore } from './voice'

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

      case 'MESSAGE_CREATE':
        guild.addMessage(event.d.message)
        break

      case 'VOICE_STATE_UPDATE':
        voice.updateVoiceState(event.d)
        break

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
