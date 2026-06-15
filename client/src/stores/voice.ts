import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Room, RoomEvent } from 'livekit-client'

export interface VoiceState {
  user_id: string
  guild_id: string
  channel_id: string | null
  is_muted: boolean
  is_deafened: boolean
}

export const useVoiceStore = defineStore('voice', () => {
  const room = ref<Room | null>(null)
  const activeChannelId = ref<string | null>(null)
  const isMuted = ref(false)
  const isDeafened = ref(false)
  const voiceStates = ref<Map<string, VoiceState>>(new Map())
  const activeSpeakers = ref<Set<string>>(new Set())

  async function connectToLiveKit(url: string, token: string) {
    // Отключаемся от предыдущего если есть
    if (room.value) {
      await room.value.disconnect()
    }

    const newRoom = new Room({
      adaptiveStream: true,
      dynacast: true,
    })

    newRoom.on(RoomEvent.ActiveSpeakersChanged, (speakers) => {
      activeSpeakers.value = new Set(speakers.map((s) => s.identity))
    })

    newRoom.on(RoomEvent.Disconnected, () => {
      activeChannelId.value = null
    })

    await newRoom.connect(url, token)
    await newRoom.localParticipant.setMicrophoneEnabled(!isMuted.value)

    room.value = newRoom
  }

  async function disconnect() {
    await room.value?.disconnect()
    room.value = null
    activeChannelId.value = null
  }

  async function toggleMute() {
    isMuted.value = !isMuted.value
    await room.value?.localParticipant.setMicrophoneEnabled(!isMuted.value)
  }

  async function toggleDeafen() {
    isDeafened.value = !isDeafened.value
    // Глушим все remote треки
    room.value?.remoteParticipants.forEach((p) => {
      p.audioTrackPublications.forEach((pub) => {
        if (pub.track) pub.track.setMuted(isDeafened.value)
      })
    })
  }

  function updateVoiceState(state: VoiceState) {
    voiceStates.value.set(state.user_id, state)
    if (state.channel_id) {
      activeChannelId.value = state.channel_id
    }
  }

  function participantsInChannel(channelId: string): VoiceState[] {
    return [...voiceStates.value.values()].filter((s) => s.channel_id === channelId)
  }

  return {
    room, activeChannelId, isMuted, isDeafened, voiceStates, activeSpeakers,
    connectToLiveKit, disconnect, toggleMute, toggleDeafen,
    updateVoiceState, participantsInChannel,
  }
})
