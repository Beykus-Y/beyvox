<template>
  <div class="voice-channel-wrapper">
    <!-- Кнопка голосового канала -->
    <div
      class="voice-channel-item"
      :class="{
        active: isActive,
        disabled: isFull && !isActive
      }"
      @click="handleChannelClick"
      :title="isFull && !isActive ? 'Канал заполнен' : 'Присоединиться к голосовому каналу'"
    >
      <div class="channel-main-info">
        <!-- Иконка спикера -->
        <svg class="speaker-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
        </svg>
        <span class="channel-name">{{ channel.name }}</span>
      </div>

      <div class="channel-meta" @click.stop>
        <!-- Шеврон раскрытия участников -->
        <button class="expand-btn" :class="{ expanded }" @click="expanded = !expanded" title="Список участников">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M7 10l5 5 5-5z"/>
          </svg>
        </button>

        <!-- Счетчик заполненности -->
        <span class="occupancy-text" :class="{ danger: isFull }">
          {{ occupiedCount }}<template v-if="limit > 0">/{{ limit }}</template>
        </span>
      </div>
    </div>

    <!-- Список участников канала -->
    <transition name="expand">
      <div class="participants-list" v-show="expanded && participants.length > 0">
        <VoiceParticipant
          v-for="p in participants"
          :key="p.user_id"
          :userId="p.user_id"
          :username="getMemberName(p.user_id)"
          :isMuted="p.is_muted"
          :isSpeaking="activeSpeakers.has(p.user_id)"
          :isStreaming="screenStore.isParticipantSharing(p.user_id)"
        />
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import VoiceParticipant from './VoiceParticipant.vue'
import { useScreenStore } from '../../stores/screen'
import type { Channel, Member } from '../../stores/guild'
import type { VoiceState } from '../../stores/voice'

const screenStore = useScreenStore()

const props = defineProps<{
  channel: Channel
  activeVoiceChannelId: string | null
  voiceStates: Map<string, VoiceState>
  activeSpeakers: Set<string>
  members: Member[]
}>()

const emit = defineEmits(['join-voice'])

const expanded = ref(true)

const isActive = computed(() => {
  return props.activeVoiceChannelId === props.channel.id
})

const participants = computed(() => {
  return [...props.voiceStates.values()].filter(s => s.channel_id === props.channel.id)
})

const occupiedCount = computed(() => {
  return participants.value.length
})

const limit = computed(() => {
  return props.channel.user_limit ?? 0
})

const isFull = computed(() => {
  if (limit.value === 0) return false
  return occupiedCount.value >= limit.value
})

function getMemberName(userId: string): string {
  const m = props.members.find(member => member.user_id === userId)
  return m?.nickname || m?.username || userId.slice(0, 8)
}

function handleChannelClick() {
  if (isFull.value && !isActive.value) {
    return // Запрет входа при заполненности
  }
  emit('join-voice', props.channel)
}

// Автораскрытие списка, если кто-то зашел в канал
watch(occupiedCount, (newVal, oldVal) => {
  if (newVal > oldVal) {
    expanded.value = true
  }
})
</script>

<style scoped>
.voice-channel-wrapper {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.voice-channel-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: var(--radius-item);
  cursor: pointer;
  background: transparent;
  border-left: 3px solid transparent;
  transition: all 0.15s ease;
  user-select: none;
}
.voice-channel-item:hover:not(.disabled) {
  background: var(--bg-hover);
}
.voice-channel-item.active {
  background: var(--bg-active);
  border-left: 3px solid var(--online);
  border-radius: 0 var(--radius-item) var(--radius-item) 0;
}
.voice-channel-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.channel-main-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.speaker-icon {
  color: var(--text-secondary);
  flex-shrink: 0;
}
.voice-channel-item.active .speaker-icon {
  color: var(--online);
}

.channel-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.voice-channel-item.active .channel-name {
  color: var(--online);
}

.channel-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.occupancy-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
}
.occupancy-text.danger {
  color: var(--danger);
}

.expand-btn {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  transition: transform 0.2s ease, background-color 0.15s;
}
.expand-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.expand-btn.expanded {
  transform: rotate(180deg);
}

.participants-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-left: 20px;
  overflow: hidden;
}

.expand-enter-active,
.expand-leave-active {
  transition: max-height 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.25s ease;
  max-height: 400px;
}
.expand-enter-from,
.expand-leave-to {
  max-height: 0 !important;
  opacity: 0;
}
.expand-enter-to,
.expand-leave-from {
  max-height: 400px;
  opacity: 1;
}
</style>
