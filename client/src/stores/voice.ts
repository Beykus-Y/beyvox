import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Room, RoomEvent, RemoteAudioTrack } from 'livekit-client'

export type VoiceMode = 'open' | 'ptt' | 'vad'

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

  // Audio devices (по имени устройства из cpal)
  const selectedInputId = ref('')
  const selectedOutputId = ref('')

  // Voice mode
  const voiceMode = ref<VoiceMode>('open')
  const pttKey = ref('Space')
  const pttActive = ref(false)

  // Громкость per-participant, 0–1
  const participantVolumes = ref<Map<string, number>>(new Map())

  // VAD internals (non-reactive)
  let vadContext: AudioContext | null = null
  let vadStream: MediaStream | null = null
  let vadAnimFrame: number | null = null
  let vadSpeaking = false
  let vadSilenceStart = 0
  const VAD_THRESHOLD = 12   // avg byte energy 0–255
  const VAD_SILENCE_MS = 400

  async function connectToLiveKit(url: string, token: string) {
    try {
      if (room.value) await room.value.disconnect()

      const newRoom = new Room({ adaptiveStream: true, dynacast: true })

      newRoom.on(RoomEvent.ActiveSpeakersChanged, (speakers) => {
        activeSpeakers.value = new Set(speakers.map((s) => s.identity))
      })

      newRoom.on(RoomEvent.Disconnected, () => {
        activeChannelId.value = null
        stopVad()
        pttActive.value = false
      })

      console.log('[voice] connecting to LiveKit:', url)
      await newRoom.connect(url, token)
      console.log('[voice] connected to LiveKit room:', newRoom.name)

      const micOpts = selectedInputId.value ? { deviceId: selectedInputId.value } : undefined

      if (voiceMode.value === 'open') {
        isMuted.value = false
        await newRoom.localParticipant.setMicrophoneEnabled(true, micOpts)
      } else {
        isMuted.value = true
        await newRoom.localParticipant.setMicrophoneEnabled(false, micOpts)
        if (voiceMode.value === 'vad') startVad()
      }

      room.value = newRoom
    } catch (e) {
      console.error('[voice] LiveKit connection failed:', e)
      activeChannelId.value = null
    }
  }

  async function disconnect() {
    stopVad()
    pttActive.value = false
    await room.value?.disconnect()
    room.value = null
    activeChannelId.value = null
    isMuted.value = false
    isDeafened.value = false
  }

  async function toggleMute() {
    if (voiceMode.value !== 'open') return
    isMuted.value = !isMuted.value
    await room.value?.localParticipant.setMicrophoneEnabled(!isMuted.value)
  }

  async function toggleDeafen() {
    isDeafened.value = !isDeafened.value
    room.value?.remoteParticipants.forEach((p) => {
      p.audioTrackPublications.forEach((pub) => {
        if (pub.track) (pub.track as RemoteAudioTrack).setMuted(isDeafened.value)
      })
    })
  }

  async function pttPress() {
    if (voiceMode.value !== 'ptt' || !room.value || pttActive.value) return
    pttActive.value = true
    isMuted.value = false
    await room.value.localParticipant.setMicrophoneEnabled(true)
  }

  async function pttRelease() {
    if (voiceMode.value !== 'ptt' || !room.value) return
    pttActive.value = false
    isMuted.value = true
    await room.value.localParticipant.setMicrophoneEnabled(false)
  }

  async function startVad() {
    stopVad()
    try {
      vadStream = await navigator.mediaDevices.getUserMedia({
        audio: selectedInputId.value
          ? { deviceId: { exact: selectedInputId.value } }
          : true,
      })
      vadContext = new AudioContext()
      const source = vadContext.createMediaStreamSource(vadStream)
      const analyser = vadContext.createAnalyser()
      analyser.fftSize = 256
      source.connect(analyser)
      const data = new Uint8Array(analyser.frequencyBinCount)
      vadSpeaking = false
      vadSilenceStart = 0

      const tick = () => {
        analyser.getByteFrequencyData(data)
        const energy = data.reduce((a, b) => a + b, 0) / data.length

        if (energy > VAD_THRESHOLD) {
          if (!vadSpeaking) {
            vadSpeaking = true
            vadSilenceStart = 0
            room.value?.localParticipant.setMicrophoneEnabled(true)
          }
        } else if (vadSpeaking) {
          if (!vadSilenceStart) vadSilenceStart = Date.now()
          if (Date.now() - vadSilenceStart > VAD_SILENCE_MS) {
            vadSpeaking = false
            room.value?.localParticipant.setMicrophoneEnabled(false)
          }
        }

        vadAnimFrame = requestAnimationFrame(tick)
      }
      vadAnimFrame = requestAnimationFrame(tick)
    } catch (e) {
      console.error('VAD init failed:', e)
    }
  }

  function stopVad() {
    if (vadAnimFrame !== null) { cancelAnimationFrame(vadAnimFrame); vadAnimFrame = null }
    vadStream?.getTracks().forEach((t) => t.stop())
    vadStream = null
    vadContext?.close()
    vadContext = null
    vadSpeaking = false
    vadSilenceStart = 0
  }

  function setParticipantVolume(userId: string, volume: number) {
    participantVolumes.value.set(userId, volume)
    room.value?.remoteParticipants.forEach((p) => {
      if (p.identity === userId) {
        p.audioTrackPublications.forEach((pub) => (pub.track as RemoteAudioTrack | undefined)?.setVolume(volume))
      }
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
    selectedInputId, selectedOutputId, voiceMode, pttKey, pttActive,
    participantVolumes,
    connectToLiveKit, disconnect, toggleMute, toggleDeafen,
    pttPress, pttRelease, startVad, stopVad,
    setParticipantVolume, updateVoiceState, participantsInChannel,
  }
})
