<template>
  <div class="guild-info-card" v-if="guild">
    <!-- Крупный аватар гильдии -->
    <div class="guild-avatar-large" :style="avatarStyle">
      <img v-if="guild.icon_url && !avatarLoadError" :src="guild.icon_url" :alt="guild.name" @error="onAvatarError" />
      <span v-else>{{ guild.name[0]?.toUpperCase() }}</span>
    </div>

    <!-- Текстовое описание -->
    <div class="guild-info-details">
      <span class="guild-name">{{ guild.name }}</span>
      <div class="guild-stats">
        <span class="stat-item">Участников: <strong class="stat-val">{{ members.length }}</strong></span>
        <span class="stat-divider">·</span>
        <span class="stat-item">Онлайн: <strong class="stat-val">{{ onlineCount }}</strong></span>
      </div>
    </div>

    <!-- Ряд кнопок действий -->
    <div class="guild-actions-row">
      <!-- Поиск в гильдии -->
      <button class="action-btn" title="Поиск" @click="$emit('search')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
        </svg>
      </button>

      <!-- Пригласить (создать инвайт) -->
      <button class="action-btn" title="Создать приглашение" @click="$emit('create-invite')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm-9-2V7H4v3H1v2h3v3h2v-3h3v-2H6zm9 4c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
        </svg>
      </button>

      <!-- Календарь / События -->
      <button class="action-btn" title="Календарь событий" @click="$emit('events')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 4h-1V2h-2v2H8V2H6v2H5c-1.11 0-1.99.9-1.99 2L3 20c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 16H5V10h14v10zm0-12H5V6h14v2z"/>
        </svg>
      </button>

      <!-- Настройки гильдии (показываются только владельцу) -->
      <button
        v-if="isOwner"
        class="action-btn"
        title="Настройки гильдии"
        @click="$emit('open-guild-settings')"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.02 7.02 0 0 0-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.47.47 0 0 0-.59.22L2.74 8.87a.49.49 0 0 0 .12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.01-1.58zM12 15.6a3.6 3.6 0 1 1 0-7.2 3.6 3.6 0 0 1 0 7.2z"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Guild, Member } from '../../stores/guild'

const props = defineProps<{
  guild: Guild
  members: Member[]
  userId: string
}>()

defineEmits(['open-guild-settings', 'create-invite', 'search', 'events'])

const avatarLoadError = ref(false)

const isOwner = computed(() => {
  return props.guild && props.guild.owner_id === props.userId
})

const onlineCount = computed(() => {
  // Поскольку у нас нет явного статуса сети для каждого участника,
  // мы считаем всех активных участников (или возвращаем приблизительное число/длину списка)
  // Для MVP: считаем, что все загруженные члены гильдии в сети, либо берем как members.length
  return props.members.length
})

const avatarStyle = computed(() => {
  if (props.guild.icon_url && !avatarLoadError.value) {
    return {}
  }
  return {
    background: getDeterministicBackground(props.guild.id || props.guild.name)
  }
})

function onAvatarError() {
  avatarLoadError.value = true
}

function getDeterministicBackground(id: string): string {
  let hash = 0
  for (let i = 0; i < id.length; i++) {
    hash = id.charCodeAt(i) + ((hash << 5) - hash)
  }
  return `linear-gradient(135deg, hsl(${Math.abs(hash) % 360}, 55%, 45%), hsl(${(Math.abs(hash) + 120) % 360}, 50%, 25%))`
}
</script>

<style scoped>
.guild-info-card {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  padding: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  user-select: none;
}

.guild-avatar-large {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 800;
  font-size: 24px;
  color: #fff;
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}
.guild-avatar-large img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.guild-info-details {
  text-align: center;
  width: 100%;
}

.guild-name {
  display: block;
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.guild-stats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-secondary);
}

.stat-divider {
  color: var(--text-muted);
}

.stat-val {
  color: var(--text-primary);
}

.guild-actions-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-btn {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}
.action-btn:hover {
  background: var(--bg-active);
  color: var(--text-primary);
  border-color: var(--accent);
}
.action-btn svg {
  width: 20px;
  height: 20px;
}
</style>
