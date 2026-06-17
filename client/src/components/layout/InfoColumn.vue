<template>
  <div class="info-column">
    <template v-if="guild">
      <!-- Информация о гильдии -->
      <GuildInfoCard
        :guild="guild"
        :members="members"
        :userId="userId"
        @open-guild-settings="$emit('open-guild-settings')"
        @create-invite="$emit('create-invite')"
        @search="$emit('search')"
        @events="$emit('events')"
      />

      <!-- Разделитель -->
      <div class="column-divider" />

      <!-- Голосовые каналы -->
      <VoiceChannelsSection
        :channels="channels"
        :activeVoiceChannelId="activeVoiceChannelId"
        :voiceStates="voiceStates"
        :activeSpeakers="activeSpeakers"
        :members="members"
        @join-voice="$emit('join-voice', $event)"
        @create-voice-channel="$emit('create-voice-channel')"
      />

      <!-- Разделитель -->
      <div class="column-divider" />

      <!-- Лента событий -->
      <ActivityFeed :events="activityEvents" />
    </template>

    <div v-else class="no-guild-info">
      <div class="no-guild-info-hint">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
        </svg>
        <p>Выбери гильдию для просмотра деталей</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import GuildInfoCard from './GuildInfoCard.vue'
import VoiceChannelsSection from '../voice/VoiceChannelsSection.vue'
import ActivityFeed from './ActivityFeed.vue'
import type { Guild, Channel, Member } from '../../stores/guild'
import type { VoiceState } from '../../stores/voice'
import type { ActivityEvent } from '../../stores/activity'

defineProps<{
  guild: Guild | null
  channels: Channel[]
  activeVoiceChannelId: string | null
  voiceStates: Map<string, VoiceState>
  activeSpeakers: Set<string>
  members: Member[]
  activityEvents: ActivityEvent[]
  userId: string
}>()

defineEmits([
  'join-voice',
  'create-voice-channel',
  'open-guild-settings',
  'create-invite',
  'search',
  'events'
])
</script>

<style scoped>
.info-column {
  width: 320px;
  background: var(--bg-panel);
  border-left: 1px solid var(--border);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100vh;
  overflow-y: auto;
  flex-shrink: 0;
}

@media (max-width: 1360px) {
  .info-column {
    width: 280px;
  }
}

@media (max-width: 1024px) {
  .info-column {
    width: 260px;
  }
}

.column-divider {
  height: 1px;
  background: var(--border);
  flex-shrink: 0;
  margin: 4px 0;
}

.no-guild-info {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  text-align: center;
  color: var(--text-secondary);
}

.no-guild-info-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 24px;
}
.no-guild-info-hint p {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
