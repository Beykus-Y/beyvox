<template>
  <div class="voice-participant" :class="{ speaking: isSpeaking }">
    <!-- Аватар с индикацией разговора (speaking ring) -->
    <div class="avatar-container">
      <div class="participant-avatar" :class="{ 'speaking-ring': isSpeaking }">
        <span>{{ username[0]?.toUpperCase() }}</span>
      </div>
      <!-- Точка индикации мьюта/статуса -->
      <span class="status-badge" :class="{ muted: isMuted }" />
    </div>

    <!-- Детали участника и ползунок громкости -->
    <div class="participant-details">
      <div class="name-row">
        <span class="participant-name" :class="{ muted: isMuted }" :title="username">
          {{ username }}
        </span>
        
        <!-- Иконка мьюта / наушников -->
        <span class="mic-status-icon" v-if="isMuted">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 11h-1.7c0 .74-.16 1.43-.43 2.05l1.23 1.23c.56-.98.9-2.09.9-3.28zm-7 7c-2.76 0-5-2.24-5-5v-1.17L5.27 10.1A7.012 7.012 0 0 0 5 11c0 3.53 2.61 6.43 6 6.92V20c0 .55.45 1 1 1s1-.45 1-1v-2.08c1.49-.21 2.87-.88 3.98-1.87l-1.46-1.46C14.46 17.61 13.28 18 12 18z"/>
          </svg>
        </span>
      </div>

      <!-- Регулятор громкости участника -->
      <div class="volume-slider-row" @click.stop>
        <svg class="volume-icon" width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3A4.5 4.5 0 0 0 14 7.97v8.05c1.48-.73 2.5-2.25 2.5-4.02z"/>
        </svg>
        <input
          type="range"
          min="0"
          max="2"
          step="0.05"
          v-model.number="volume"
          @input="updateVolume"
          class="volume-input"
          title="Громкость участника"
        />
        <span class="volume-percent">{{ Math.round(volume * 100) }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVoiceStore } from '../../stores/voice'

const props = defineProps<{
  userId: string
  username: string
  isMuted: boolean
  isSpeaking: boolean
}>()

const voice = useVoiceStore()
const volume = ref(1.0)

function updateVolume() {
  voice.participantVolumes.set(props.userId, volume.value)
  localStorage.setItem(`voice_vol_${props.userId}`, String(volume.value))
  
  // Вызов Tauri-команды настройки громкости конкретного спикера с обработкой ошибок
  invoke('set_participant_volume', { userId: props.userId, volume: volume.value })
    .catch(() => {
      // Молча проглатываем, если команда не реализована на бэкенде
    })
}

onMounted(() => {
  const saved = localStorage.getItem(`voice_vol_${props.userId}`)
  if (saved !== null) {
    volume.value = parseFloat(saved)
    voice.participantVolumes.set(props.userId, volume.value)
  } else {
    const storeVal = voice.participantVolumes.get(props.userId)
    if (storeVal !== undefined) {
      volume.value = storeVal
    }
  }
})
</script>

<style scoped>
.voice-participant {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  border-radius: var(--radius-item);
  background: var(--bg-elevated);
  border: 1px solid transparent;
  transition: background-color 0.15s, border-color 0.15s;
  user-select: none;
}
.voice-participant:hover {
  background: var(--bg-hover);
  border-color: var(--border);
}

.avatar-container {
  position: relative;
  flex-shrink: 0;
}

.participant-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--bg-active);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 12px;
  color: var(--text-primary);
  border: 1px solid var(--border);
  transition: box-shadow 0.2s ease;
}

.status-badge {
  position: absolute;
  bottom: -1px;
  right: -1px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--online);
  border: 1.5px solid var(--bg-panel);
}
.status-badge.muted {
  background: var(--danger);
}

.participant-details {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.name-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.participant-name {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.participant-name.muted {
  color: var(--text-muted);
}

.mic-status-icon {
  color: var(--danger);
  display: flex;
  align-items: center;
}

.volume-slider-row {
  display: flex;
  align-items: center;
  gap: 6px;
  opacity: 0;
  height: 0;
  overflow: hidden;
  transition: opacity 0.15s ease, height 0.15s ease;
}
.voice-participant:hover .volume-slider-row {
  opacity: 1;
  height: 12px;
  margin-top: 2px;
}

.volume-icon {
  color: var(--text-secondary);
}

.volume-input {
  flex: 1;
  height: 3px;
  accent-color: var(--accent);
  background: var(--border);
  cursor: pointer;
  padding: 0;
  border: none;
}

.volume-percent {
  font-size: 9px;
  font-family: monospace;
  color: var(--text-secondary);
  min-width: 24px;
  text-align: right;
}
</style>
