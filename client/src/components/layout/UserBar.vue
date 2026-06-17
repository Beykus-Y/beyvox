<template>
  <div class="user-bar">
    <div class="user-info-section">
      <div class="avatar-wrapper">
        <div class="user-avatar">
          <span>{{ auth.username[0]?.toUpperCase() }}</span>
        </div>
        <span class="status-indicator" :class="{ online: isOnline, offline: !isOnline }" />
      </div>
      <div class="user-details">
        <span class="user-name" :title="auth.username">{{ auth.username }}</span>
        <span class="user-status-text">{{ isOnline ? 'Онлайн' : 'Оффлайн' }}</span>
      </div>
    </div>

    <div class="user-controls">
      <!-- Микрофон -->
      <button
        class="control-btn"
        :class="{ active: !voice.isMuted, disabled: voice.isMuted }"
        @click="voice.toggleMute"
        :title="voice.isMuted ? 'Включить микрофон' : 'Выключить микрофон'"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path v-if="!voice.isMuted" d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm5.91-3c-.49 0-.9.36-.98.85C16.52 14.2 14.47 16 12 16s-4.52-1.8-4.93-4.15c-.08-.49-.49-.85-.98-.85-.61 0-1.09.54-1 1.14.49 3 2.89 5.35 5.91 5.78V20c0 .55.45 1 1 1s1-.45 1-1v-2.08c3.02-.43 5.42-2.78 5.91-5.78.1-.6-.39-1.14-1-1.14z"/>
          <path v-else d="m19 11c0 1.19-.34 2.3-.9 3.28l-1.23-1.23c.27-.62.43-1.31.43-2.05H19zm-7 7c-2.76 0-5-2.24-5-5v-1.17L5.27 10.1A7.012 7.012 0 0 0 5 11c0 3.53 2.61 6.43 6 6.92V20c0 .55.45 1 1 1s1-.45 1-1v-2.08c1.49-.21 2.87-.88 3.98-1.87l-1.46-1.46C14.46 17.61 13.28 18 12 18zm7.19 2.81L3.27 5 2 6.27l3.55 3.55C5.21 10.57 5 11.26 5 12c0 3.53 2.61 6.43 6 6.92V21c0 .55.45 1 1 1s1-.45 1-1v-2.08c1.77-.25 3.38-1.09 4.58-2.31l2.61 2.61L21.73 18l-2.54-2.54zM12 4c1.66 0 3 1.34 3 3v4.18l2 2V7c0-2.76-2.24-5-5-5-1.3 0-2.49.5-3.38 1.3L10.1 4.78C10.65 4.29 11.29 4 12 4z"/>
        </svg>
      </button>

      <!-- Наушники -->
      <button
        class="control-btn"
        :class="{ active: !voice.isDeafened, disabled: voice.isDeafened }"
        @click="voice.toggleDeafen"
        :title="voice.isDeafened ? 'Включить звук' : 'Выключить звук'"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path v-if="!voice.isDeafened" d="M12 2c-4.97 0-9 4.03-9 9v7c0 1.66 1.34 3 3 3h3v-8H5v-2c0-3.87 3.13-7 7-7s7 3.13 7 7v2h-4v8h3c1.66 0 3-1.34 3-3v-7c0-4.97-4.03-9-9-9z"/>
          <path v-else d="M12 4c3.87 0 7 3.13 7 7v1h-4v3.17l4.83 4.83c.11-.31.17-.65.17-1V11c0-4.97-4.03-9-9-9-1.32 0-2.56.29-3.69.8l1.45 1.45C9.64 4.09 10.78 4 12 4zm9.19 16.81l-1.41-1.41L4.34 3.93 2.93 5.34l3.11 3.11C5.4 9.24 5 10.07 5 11v7c0 1.66 1.34 3 3 3h3v-8H5v-2c0-.52.07-1.01.18-1.49L12 16.34V21c0 .55.45 1 1 1s1-.45 1-1v-4.66l5.78 5.78 1.41-1.41zM9 14v5H8c-.55 0-1-.45-1-1v-3.17L9 14z"/>
        </svg>
      </button>

      <!-- Настройки -->
      <button
        class="control-btn"
        @click="$emit('open-settings')"
        title="Настройки пользователя"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.02 7.02 0 0 0-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.47.47 0 0 0-.59.22L2.74 8.87a.49.49 0 0 0 .12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.01-1.58zM12 15.6a3.6 3.6 0 1 1 0-7.2 3.6 3.6 0 0 1 0 7.2z"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAuthStore } from '../../stores/auth'
import { useVoiceStore } from '../../stores/voice'
import { useWsStore } from '../../stores/ws'

defineEmits(['open-settings'])

const auth = useAuthStore()
const voice = useVoiceStore()
const ws = useWsStore()

const isOnline = computed(() => ws.status === 'connected')
</script>

<style scoped>
.user-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 52px;
  padding: 0 10px;
  background: var(--bg-panel);
  border-top: 1px solid var(--border);
  flex-shrink: 0;
  user-select: none;
}

.user-info-section {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.avatar-wrapper {
  position: relative;
  flex-shrink: 0;
}

.user-avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: var(--accent-grad);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 13px;
  color: #fff;
}

.status-indicator {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 9px;
  height: 9px;
  border-radius: 50%;
  border: 1.5px solid var(--bg-panel);
}
.status-indicator.online {
  background: var(--online);
}
.status-indicator.offline {
  background: var(--offline);
}

.user-details {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.user-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.2;
}

.user-status-text {
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 1.2;
}

.user-controls {
  display: flex;
  gap: 2px;
}

.control-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
}
.control-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.control-btn.disabled {
  color: var(--danger);
}
.control-btn.disabled:hover {
  background: rgba(239, 68, 68, 0.1);
}
</style>
