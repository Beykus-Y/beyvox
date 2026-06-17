import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface ActivityEvent {
  id: string
  type: 'channel_created' | 'voice_joined' | 'message_sent' | 'guild_joined' | string
  actor: {
    id: string
    username: string
    avatarUrl?: string
  }
  targetLabel?: string
  createdAt: string // ISO string
}

export const useActivityStore = defineStore('activity', () => {
  const events = ref<ActivityEvent[]>([])

  function addEvent(event: Omit<ActivityEvent, 'id' | 'createdAt'>) {
    events.value.unshift({
      id: Math.random().toString(36).substring(2, 11),
      createdAt: new Date().toISOString(),
      ...event,
    })
    
    // Ограничиваем количество событий в памяти до 50
    if (events.value.length > 50) {
      events.value = events.value.slice(0, 50)
    }
  }

  function clear() {
    events.value = []
  }

  return { events, addEvent, clear }
})
