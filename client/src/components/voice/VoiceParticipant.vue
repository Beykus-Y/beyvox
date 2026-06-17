<template>
  <div class="voice-participant" :class="{ speaking: isSpeaking }" @contextmenu.prevent="openMenu">
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

    <!-- Кнопка просмотра стрима (если участник стримит) -->
    <button
      v-if="isStreaming && !isSelf"
      class="watch-btn"
      title="Смотреть трансляцию"
      @click.stop="watchStream"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
      </svg>
    </button>

    <!-- Иконка стрима на своём участнике -->
    <span v-if="isStreaming && isSelf" class="streaming-badge" title="Идёт трансляция">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
        <path d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h5v2h8v-2h5c1.1 0 1.99-.9 1.99-2L23 5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"/>
      </svg>
    </span>

    <!-- ПКМ меню для модерации -->
    <Teleport to="body">
      <div v-if="menuVisible && !isSelf && (canMute || canKick)" class="ctx-overlay" @mousedown.self="closeMenu" @contextmenu.prevent>
        <div class="ctx-menu" :style="{ top: menuY + 'px', left: menuX + 'px' }">
          <button v-if="canMute" class="ctx-item" @click="doMute">
            {{ isMuted ? 'Снять глушение' : 'Заглушить' }}
          </button>
          <div v-if="canMute && canKick" class="ctx-divider" />
          <button v-if="canKick" class="ctx-item ctx-item-danger" @click="doKick">Кикнуть с сервера</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVoiceStore } from '../../stores/voice'
import { useGuildStore, PERM } from '../../stores/guild'
import { useAuthStore } from '../../stores/auth'
import { useScreenStore } from '../../stores/screen'

const props = defineProps<{
  userId: string
  username: string
  isMuted: boolean
  isSpeaking: boolean
  isStreaming?: boolean
}>()

const voice = useVoiceStore()
const guildStore = useGuildStore()
const auth = useAuthStore()
const screenStore = useScreenStore()
const volume = ref(1.0)

const isSelf = computed(() => props.userId === auth.userId)
const canMute = computed(() => guildStore.hasPermission(PERM.MUTE_MEMBERS))
const canKick = computed(() => guildStore.hasPermission(PERM.MANAGE_MEMBERS))

const menuVisible = ref(false)
const menuX = ref(0)
const menuY = ref(0)

function openMenu(e: MouseEvent) {
  if (isSelf.value) return
  menuX.value = Math.min(e.clientX, window.innerWidth - 160)
  menuY.value = Math.min(e.clientY, window.innerHeight - 120)
  menuVisible.value = true
}

function closeMenu() {
  menuVisible.value = false
}

async function doMute() {
  closeMenu()
  const guildId = guildStore.activeGuildId
  if (!guildId) return
  const member = guildStore.members.find(m => m.user_id === props.userId)
  await guildStore.muteMember(guildId, props.userId, !member?.is_muted)
}

async function doKick() {
  closeMenu()
  const guildId = guildStore.activeGuildId
  if (!guildId) return
  if (confirm(`Кикнуть ${props.username}?`)) {
    await guildStore.kickMember(guildId, props.userId)
  }
}

function watchStream() {
  screenStore.watchStream(props.userId)
}

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

.ctx-overlay {
  position: fixed;
  inset: 0;
  z-index: 2000;
}

.ctx-menu {
  position: fixed;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 4px;
  min-width: 150px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

.ctx-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 7px 10px;
  border-radius: 5px;
  font-size: 13px;
  color: var(--text-primary);
  background: none;
  border: none;
  cursor: pointer;
}
.ctx-item:hover { background: var(--bg-hover); }
.ctx-item-danger { color: #e63946; }
.ctx-item-danger:hover { background: rgba(230, 57, 70, 0.12); }

.ctx-divider {
  height: 1px;
  background: var(--border);
  margin: 3px 0;
}

.watch-btn {
  width: 22px;
  height: 22px;
  border-radius: 5px;
  background: var(--accent);
  border: none;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}
.voice-participant:hover .watch-btn { opacity: 1; }

.streaming-badge {
  color: var(--accent);
  display: flex;
  align-items: center;
  flex-shrink: 0;
}
</style>
