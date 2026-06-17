import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAuthStore } from './auth'
import { SoundEffects, PhoneCallSounds } from '../utils/sounds'

export type VoiceMode = 'open' | 'ptt'

export interface VoiceState {
  user_id: string
  guild_id: string
  channel_id: string | null
  is_muted: boolean
  is_deafened: boolean
}

export const useVoiceStore = defineStore('voice', () => {
  const activeChannelId = ref<string | null>(null)
  const isMuted = ref(false)
  const isDeafened = ref(false)
  const voiceStates = ref<Map<string, VoiceState>>(new Map())
  const activeSpeakers = ref<Set<string>>(new Set())

  const selectedInputCpalName = ref(localStorage.getItem('voice_input_device') || '')
  const selectedOutputCpalName = ref(localStorage.getItem('voice_output_device') || '')

  const micVolume = ref(Number(localStorage.getItem('voice_mic_volume') ?? 100))
  const playbackVolume = ref(Number(localStorage.getItem('voice_playback_volume') ?? 100))

  const voiceMode = ref<VoiceMode>((localStorage.getItem('voice_mode') as VoiceMode) || 'open')
  const pttKey = ref(localStorage.getItem('voice_ptt_key') || 'Space')
  const pttActive = ref(false)

  const micError = ref('')
  const isMicTesting = ref(false)

  // Громкость per-participant
  const participantVolumes = ref<Map<string, number>>(new Map())

  // Подписка на события от Rust
  let unlistenSpeakers: (() => void) | null = null
  let unlistenDisconnected: (() => void) | null = null

  async function setupListeners() {
    unlistenSpeakers?.()
    unlistenDisconnected?.()

    unlistenSpeakers = await listen<string[]>('voice://active-speakers', (e) => {
      activeSpeakers.value = new Set(e.payload)
    })

    unlistenDisconnected = await listen('voice://disconnected', () => {
      activeChannelId.value = null
      isMuted.value = false
      isDeafened.value = false
      pttActive.value = false
    })
  }

  async function connectToLiveKit(url: string, token: string) {
    micError.value = ''
    try {
      await setupListeners()
      await invoke('join_voice_channel', {
        url,
        token,
        inputDevice: selectedInputCpalName.value || null,
        outputDevice: selectedOutputCpalName.value || null,
      })

      PhoneCallSounds.stop()
      SoundEffects.join()

      if (voiceMode.value === 'open') {
        isMuted.value = false
      } else {
        isMuted.value = true
        await invoke('set_muted', { muted: true })
      }
    } catch (e: any) {
      PhoneCallSounds.stop()
      micError.value = 'Ошибка подключения к голосу: ' + String(e)
      console.error('[voice] join failed:', e)
      activeChannelId.value = null
    }
  }

  async function disconnect() {
    pttActive.value = false
    isMuted.value = false
    isDeafened.value = false
    activeChannelId.value = null
    PhoneCallSounds.stop()
    SoundEffects.leave()
    await invoke('leave_voice_channel').catch(() => {})
    unlistenSpeakers?.()
    unlistenDisconnected?.()
  }

  async function toggleMute() {
    if (voiceMode.value !== 'open') return
    isMuted.value = !isMuted.value
    isMuted.value ? SoundEffects.muteMic() : SoundEffects.unmuteMic()
    await invoke('set_muted', { muted: isMuted.value })
  }

  async function toggleDeafen() {
    isDeafened.value = !isDeafened.value
    isDeafened.value ? SoundEffects.deafen() : SoundEffects.undeafen()
    await invoke('set_deafened', { deafened: isDeafened.value })
  }

  async function pttPress() {
    if (voiceMode.value !== 'ptt' || pttActive.value) return
    pttActive.value = true
    isMuted.value = false
    await invoke('set_muted', { muted: false })
  }

  async function pttRelease() {
    if (voiceMode.value !== 'ptt') return
    pttActive.value = false
    isMuted.value = true
    await invoke('set_muted', { muted: true })
  }

  async function startMicTest() {
    stopMicTest()
    try {
      await invoke('start_mic_test', {
        inputDevice: selectedInputCpalName.value || null,
      })
      isMicTesting.value = true
    } catch (e) {
      console.error('[voice] mic test failed:', e)
    }
  }

  function stopMicTest() {
    isMicTesting.value = false
    invoke('stop_mic_test').catch(() => {})
  }

  function setInputCpalName(name: string) {
    selectedInputCpalName.value = name
    localStorage.setItem('voice_input_device', name)
  }

  function setOutputCpalName(name: string) {
    selectedOutputCpalName.value = name
    localStorage.setItem('voice_output_device', name)
  }

  async function setMicVolume(percent: number) {
    micVolume.value = percent
    localStorage.setItem('voice_mic_volume', String(percent))
    await invoke('set_mic_volume', { percent })
  }

  async function setPlaybackVolume(percent: number) {
    playbackVolume.value = percent
    localStorage.setItem('voice_playback_volume', String(percent))
    await invoke('set_playback_volume', { percent })
  }

  watch(voiceMode, v => localStorage.setItem('voice_mode', v))

  // Вызывается при старте приложения — восстанавливает gain в Rust из сохранённых настроек
  async function initVoiceSettings() {
    await invoke('set_mic_volume', { percent: micVolume.value })
    await invoke('set_playback_volume', { percent: playbackVolume.value })
  }

  function updateVoiceState(state: VoiceState) {
    voiceStates.value.set(state.user_id, state)
    const auth = useAuthStore()
    if (state.user_id === auth.userId) {
      activeChannelId.value = state.channel_id
    }
  }

  function participantsInChannel(channelId: string): VoiceState[] {
    return [...voiceStates.value.values()].filter((s) => s.channel_id === channelId)
  }

  // no-op для совместимости (больше не нужен — AudioContext не используется)
  async function prewarmAudio() {}

  return {
    activeChannelId, isMuted, isDeafened, voiceStates, activeSpeakers,
    selectedInputCpalName, selectedOutputCpalName,
    voiceMode, pttKey, pttActive,
    participantVolumes, micError,
    isMicTesting,
    connectToLiveKit, disconnect, toggleMute, toggleDeafen,
    pttPress, pttRelease,
    startMicTest, stopMicTest,
    setInputCpalName, setOutputCpalName,
    micVolume, playbackVolume, setMicVolume, setPlaybackVolume, initVoiceSettings,
    prewarmAudio, updateVoiceState, participantsInChannel,
  }
})
