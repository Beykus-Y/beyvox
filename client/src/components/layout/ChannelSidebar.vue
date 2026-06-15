<template>
  <div class="channel-sidebar">
    <div class="guild-header">
      <span class="guild-name">{{ guildName }}</span>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" style="color:var(--text3);flex-shrink:0">
        <path d="M7 10l5 5 5-5z"/>
      </svg>
    </div>

    <div class="channels-list">
      <!-- Текстовые каналы -->
      <div class="channel-group">
        <div class="group-label">Текстовые каналы</div>
        <div
          v-for="ch in textChannels"
          :key="ch.id"
          class="channel-item"
          :class="{ active: ch.id === activeChannelId }"
          @click="$emit('select-channel', ch)"
        >
          <span class="ch-icon">#</span>
          <span class="ch-name">{{ ch.name }}</span>
        </div>
      </div>

      <!-- Голосовые каналы -->
      <div class="channel-group">
        <div class="group-label">Голосовые каналы</div>
        <div
          v-for="ch in voiceChannels"
          :key="ch.id"
          class="channel-item"
          :class="{ active: ch.id === activeVoiceChannelId }"
          @click="$emit('join-voice', ch)"
        >
          <span class="ch-icon">🔊</span>
          <span class="ch-name">{{ ch.name }}</span>
          <span v-if="ch.user_limit" class="ch-limit">
            {{ participantCount(ch.id) }}/{{ ch.user_limit }}
          </span>
          <!-- Участники в канале -->
          <div v-if="participantsInChannel(ch.id).length > 0" class="voice-participants">
            <div
              v-for="p in participantsInChannel(ch.id)"
              :key="p.user_id"
              class="voice-participant"
              :class="{ speaking: activeSpeakers.has(p.user_id) }"
            >
              <div class="avatar avatar-sm">{{ p.user_id.slice(0, 1).toUpperCase() }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Панель пользователя -->
    <div class="user-panel">
      <div class="avatar">{{ username[0]?.toUpperCase() }}</div>
      <div class="user-info">
        <div class="user-name">{{ username }}</div>
        <div class="user-status">{{ wsStatus }}</div>
      </div>
      <div class="user-controls">
        <button class="icon-btn" :class="{ active: !isMuted, danger: isMuted }" @click="$emit('toggle-mute')" title="Микрофон">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path v-if="!isMuted" d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm5.91-3c-.49 0-.9.36-.98.85C16.52 14.2 14.47 16 12 16s-4.52-1.8-4.93-4.15c-.08-.49-.49-.85-.98-.85-.61 0-1.09.54-1 1.14.49 3 2.89 5.35 5.91 5.78V20c0 .55.45 1 1 1s1-.45 1-1v-2.08c3.02-.43 5.42-2.78 5.91-5.78.1-.6-.39-1.14-1-1.14z"/>
            <path v-else d="m19 11c0 1.19-.34 2.3-.9 3.28l-1.23-1.23c.27-.62.43-1.31.43-2.05H19zm-7 7c-2.76 0-5-2.24-5-5v-1.17L5.27 10.1A7.012 7.012 0 0 0 5 11c0 3.53 2.61 6.43 6 6.92V20c0 .55.45 1 1 1s1-.45 1-1v-2.08c1.49-.21 2.87-.88 3.98-1.87l-1.46-1.46C14.46 17.61 13.28 18 12 18zm7.19 2.81L3.27 5 2 6.27l3.55 3.55C5.21 10.57 5 11.26 5 12c0 3.53 2.61 6.43 6 6.92V21c0 .55.45 1 1 1s1-.45 1-1v-2.08c1.77-.25 3.38-1.09 4.58-2.31l2.61 2.61L21.73 18l-2.54-2.54zM12 4c1.66 0 3 1.34 3 3v4.18l2 2V7c0-2.76-2.24-5-5-5-1.3 0-2.49.5-3.38 1.3L10.1 4.78C10.65 4.29 11.29 4 12 4z"/>
          </svg>
        </button>
        <button class="icon-btn" :class="{ active: !isDeafened, danger: isDeafened }" @click="$emit('toggle-deafen')" title="Звук">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path v-if="!isDeafened" d="M3 9v6h4l5 5V4L7 9H3zm13.5 3A4.5 4.5 0 0 0 14 7.97v8.05c1.48-.73 2.5-2.25 2.5-4.02z"/>
            <path v-else d="M16.5 12A4.5 4.5 0 0 0 14 7.97v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06a8.994 8.994 0 0 0 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Channel } from '../../stores/guild'
import type { VoiceState } from '../../stores/voice'
import { computed } from 'vue'

const props = defineProps<{
  guildName: string
  channels: Channel[]
  activeChannelId: string | null
  activeVoiceChannelId: string | null
  username: string
  wsStatus: string
  isMuted: boolean
  isDeafened: boolean
  voiceStates: Map<string, VoiceState>
  activeSpeakers: Set<string>
}>()

defineEmits(['select-channel', 'join-voice', 'toggle-mute', 'toggle-deafen'])

const textChannels = computed(() => props.channels.filter((c) => c.type === 'text'))
const voiceChannels = computed(() => props.channels.filter((c) => c.type === 'voice'))

function participantsInChannel(channelId: string): VoiceState[] {
  return [...props.voiceStates.values()].filter((s) => s.channel_id === channelId)
}

function participantCount(channelId: string): number {
  return participantsInChannel(channelId).length
}
</script>

<style scoped>
.channel-sidebar {
  width: var(--sidebar-w);
  background: var(--bg);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
}
.guild-header {
  height: 48px;
  padding: 0 12px 0 16px;
  display: flex;
  align-items: center;
  gap: 4px;
  border-bottom: 1px solid var(--border);
  font-weight: 700;
  font-size: 14px;
  flex-shrink: 0;
}
.guild-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.channels-list { flex: 1; overflow-y: auto; padding: 8px 0; }
.channel-group { margin-bottom: 4px; }
.group-label {
  padding: 16px 8px 4px 8px;
  font-size: 11px;
  font-weight: 700;
  color: var(--text2);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  display: flex;
  align-items: center;
  margin: 0 8px;
}
.channel-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px 5px 12px;
  border-radius: 6px;
  margin: 1px 6px;
  cursor: pointer;
  color: var(--text2);
  transition: all 0.1s;
}
.channel-item:hover, .channel-item.active {
  background: var(--bg-hover);
  color: var(--text);
}
.ch-icon { font-size: 16px; color: var(--text2); flex-shrink: 0; }
.ch-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 14px; }
.ch-limit { font-size: 11px; color: var(--text3); }
.voice-participants {
  display: flex;
  gap: 2px;
  flex-wrap: wrap;
  padding: 4px 0 0 24px;
}
.voice-participant .avatar-sm {
  width: 20px; height: 20px;
  font-size: 10px;
  border-radius: 50%;
  background: var(--bg-light);
  display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border);
}
.voice-participant.speaking .avatar-sm { border-color: var(--green); }

.user-panel {
  height: 56px;
  padding: 0 10px;
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--bg-darkest);
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
.user-info { flex: 1; min-width: 0; }
.user-name { font-size: 13px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; }
.user-status { font-size: 11px; color: var(--text2); margin-top: 1px; }
.user-controls { display: flex; gap: 2px; }
/* Иконки управления в нейтральном цвете, не зелёном */
.icon-btn { color: var(--text2) !important; }
.icon-btn.active { color: var(--text) !important; }
.icon-btn.danger { color: var(--red) !important; }
</style>
