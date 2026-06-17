<template>
  <div class="voice-channels-section">
    <!-- Шапка секции -->
    <div class="section-header">
      <span class="uppercase-label">Голосовые каналы</span>
      <button class="add-channel-btn" @click="$emit('create-voice-channel')" title="Создать голосовой канал">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </button>
    </div>

    <!-- Список голосовых каналов -->
    <div class="channels-list">
      <div v-if="voiceChannels.length === 0" class="empty-channels">
        <span class="empty-text">Нет голосовых каналов.</span>
      </div>
      <VoiceChannelItem
        v-for="ch in voiceChannels"
        :key="ch.id"
        :channel="ch"
        :activeVoiceChannelId="activeVoiceChannelId"
        :voiceStates="voiceStates"
        :activeSpeakers="activeSpeakers"
        :members="members"
        @join-voice="$emit('join-voice', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import VoiceChannelItem from './VoiceChannelItem.vue'
import type { Channel, Member } from '../../stores/guild'
import type { VoiceState } from '../../stores/voice'

const props = defineProps<{
  channels: Channel[]
  activeVoiceChannelId: string | null
  voiceStates: Map<string, VoiceState>
  activeSpeakers: Set<string>
  members: Member[]
}>()

defineEmits(['join-voice', 'create-voice-channel'])

const voiceChannels = computed(() => {
  return props.channels.filter(c => c.type === 'voice')
})
</script>

<style scoped>
.voice-channels-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px;
}

.add-channel-btn {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
}
.add-channel-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.channels-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.empty-channels {
  padding: 8px 12px;
  text-align: center;
}

.empty-text {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
