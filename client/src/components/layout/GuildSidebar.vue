<template>
  <div class="guild-sidebar">
    <div class="server-header">
      <span class="server-name">{{ serverName }}</span>
      <button class="create-btn" title="Создать сервер" @click="$emit('create-guild')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </button>
    </div>

    <div class="guilds-list">
      <div v-if="guilds.length === 0" class="empty-hint">
        <p>Серверов нет</p>
        <p>Нажми + чтобы создать</p>
      </div>
      <div
        v-for="guild in guilds"
        :key="guild.id"
        class="guild-item"
        :class="{ active: guild.id === activeGuildId }"
        @click="$emit('select-guild', guild.id)"
      >
        <div class="guild-avatar">
          <img v-if="guild.icon_url" :src="guild.icon_url" :alt="guild.name" />
          <span v-else>{{ guild.name[0]?.toUpperCase() }}</span>
        </div>
        <div class="guild-info">
          <div class="guild-name">{{ guild.name }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Guild } from '../../stores/guild'

defineProps<{
  serverName: string
  guilds: Guild[]
  activeGuildId: string | null
}>()
defineEmits(['select-guild', 'create-guild'])
</script>

<style scoped>
.guild-sidebar {
  width: 220px;
  background: var(--bg-dark);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
}

.server-header {
  height: 48px;
  padding: 0 12px 0 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.server-name {
  flex: 1;
  font-size: 14px;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.create-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: transparent;
  color: var(--text3);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.create-btn:hover { background: var(--bg-hover); color: var(--green); }

.guilds-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.empty-hint {
  padding: 20px 16px;
  color: var(--text3);
  font-size: 12px;
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.guild-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  margin: 1px 6px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.1s;
}
.guild-item:hover { background: var(--bg-hover); }
.guild-item.active { background: var(--bg-hover); }

.guild-avatar {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: var(--bg-light);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 700;
  color: var(--accent);
  flex-shrink: 0;
  overflow: hidden;
}
.guild-item.active .guild-avatar {
  background: var(--accent);
  color: white;
  border-color: transparent;
}
.guild-avatar img { width: 100%; height: 100%; object-fit: cover; }

.guild-info { flex: 1; min-width: 0; }
.guild-name {
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
