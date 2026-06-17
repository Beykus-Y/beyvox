import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ScreenInfo {
  id: number
  name: string
  width: number
  height: number
}

export const useScreenStore = defineStore('screen', () => {
  const isSharing = ref(false)
  const shareTarget = ref<{ screenId: number; quality: string; fps: number } | null>(null)

  // userId → channel_id (null если channel неизвестен)
  const sharingParticipants = ref<Map<string, string | null>>(new Map())

  // Чей стрим сейчас смотрим
  const viewingParticipant = ref<string | null>(null)

  async function listScreens(): Promise<ScreenInfo[]> {
    return invoke<ScreenInfo[]>('list_screens')
  }

  async function startShare(screenId: number, quality: string, fps: number): Promise<void> {
    await invoke('start_screen_share', { screenId, quality, fps })
    isSharing.value = true
    shareTarget.value = { screenId, quality, fps }
  }

  async function stopShare(): Promise<void> {
    await invoke('stop_screen_share')
    isSharing.value = false
    shareTarget.value = null
  }

  function updateSharingState(userId: string, channelId: string | null, sharing: boolean) {
    if (sharing) {
      sharingParticipants.value.set(userId, channelId)
    } else {
      sharingParticipants.value.delete(userId)
    }
  }

  function isParticipantSharing(userId: string): boolean {
    return sharingParticipants.value.has(userId)
  }

  function watchStream(userId: string) {
    viewingParticipant.value = userId
  }

  function closeStream() {
    viewingParticipant.value = null
  }

  return {
    isSharing,
    shareTarget,
    sharingParticipants,
    viewingParticipant,
    listScreens,
    startShare,
    stopShare,
    updateSharingState,
    isParticipantSharing,
    watchStream,
    closeStream,
  }
})
