<template>
  <div class="guilds-column" :class="{ collapsed: isCollapsed }">
    <!-- Шапка гильдий -->
    <div class="guilds-header" v-if="!isCollapsed">
      <span class="uppercase-label">Гильдии на сервере</span>
      <div class="header-actions">
        <!-- Поиск / Фильтр -->
        <button
          class="action-btn"
          :class="{ active: showSearch }"
          @click="showSearch = !showSearch"
          title="Найти гильдию"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
          </svg>
        </button>
        <!-- Добавить гильдию -->
        <button
          class="action-btn"
          @click="$emit('create-guild')"
          title="Создать гильдию"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Поле поиска -->
    <div class="search-container" v-if="showSearch && !isCollapsed">
      <input
        v-model="searchQuery"
        placeholder="Поиск гильдии..."
        class="search-input"
        type="text"
        ref="searchInput"
        @keydown.esc="clearSearch"
      />
      <button class="clear-search-btn" @click="clearSearch" v-if="searchQuery">✕</button>
    </div>

    <!-- Список гильдий -->
    <div class="guilds-list-container">
      <div v-if="filteredGuilds.length === 0" class="empty-guilds">
        <span class="empty-text" v-if="!isCollapsed">
          {{ guilds.length === 0 ? 'Нет гильдий на сервере.' : 'Ничего не найдено.' }}
        </span>
      </div>
      <div class="guilds-list" v-else>
        <GuildRow
          v-for="guild in filteredGuilds"
          :key="guild.id"
          :guild="guild"
          :isActive="guild.id === activeGuildId"
          :title="isCollapsed ? guild.name : undefined"
          @select="$emit('select-guild', guild.id)"
          @contextmenu="(e, g) => $emit('guild-contextmenu', e, g)"
        />
      </div>
    </div>

    <!-- Кнопка инвайтов / Приглашений -->
    <div class="invites-footer">
      <button
        class="invites-btn"
        @click="$emit('join-invite')"
        :title="isCollapsed ? 'Приглашения' : 'Войти по коду приглашения'"
      >
        <div class="invites-btn-content">
          <svg class="invite-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H8V4h12v12z"/>
          </svg>
          <span class="invites-text" v-if="!isCollapsed">Приглашения</span>
        </div>
        <div v-if="pendingInvitesCount > 0" class="invites-badge">
          {{ pendingInvitesCount > 99 ? '99+' : pendingInvitesCount }}
        </div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import GuildRow from './GuildRow.vue'
import type { Guild } from '../../stores/guild'

const props = withDefaults(defineProps<{
  guilds: Guild[]
  activeGuildId: string | null
  isCollapsed?: boolean
  pendingInvitesCount?: number
}>(), {
  isCollapsed: false,
  pendingInvitesCount: 0
})

const emit = defineEmits(['select-guild', 'create-guild', 'join-invite', 'guild-contextmenu'])

const searchQuery = ref('')
const showSearch = ref(false)

const filteredGuilds = computed(() => {
  if (!searchQuery.value) return props.guilds
  const q = searchQuery.value.toLowerCase().trim()
  return props.guilds.filter(g => g.name.toLowerCase().includes(q))
})

function clearSearch() {
  searchQuery.value = ''
  showSearch.value = false
}
</script>

<style scoped>
.guilds-column {
  width: 300px;
  background: var(--bg-panel);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  height: 100vh;
  flex-shrink: 0;
  transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.guilds-column.collapsed {
  width: 70px;
}

@media (max-width: 1360px) {
  .guilds-column:not(.collapsed) {
    width: 240px;
  }
}

.guilds-header {
  height: 54px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}
.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.action-btn.active {
  color: var(--accent);
  background: rgba(124, 108, 255, 0.1);
}

.search-container {
  padding: 8px 12px;
  position: relative;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.search-input {
  width: 100%;
  padding-right: 28px;
}

.clear-search-btn {
  position: absolute;
  right: 20px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  font-size: 11px;
}
.clear-search-btn:hover {
  color: var(--text-primary);
}

.guilds-list-container {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.guilds-list {
  display: flex;
  flex-direction: column;
}

.empty-guilds {
  padding: 24px 16px;
  text-align: center;
}

.empty-text {
  font-size: 12px;
  color: var(--text-muted);
}

/* Футер */
.invites-footer {
  padding: 8px;
  border-top: 1px solid var(--border);
  background: var(--bg-panel);
  flex-shrink: 0;
}

.invites-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 12px;
  border-radius: var(--radius-item);
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-primary);
  font-weight: 600;
  transition: background 0.15s, border-color 0.15s;
}
.invites-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
}

.invites-btn-content {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.invite-icon {
  color: var(--text-secondary);
  flex-shrink: 0;
}

.invites-text {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.invites-badge {
  font-size: 10px;
  font-weight: 800;
  color: #fff;
  background: var(--accent);
  padding: 1px 6px;
  border-radius: 10px;
  line-height: 1.4;
  flex-shrink: 0;
}

/* Свернутое состояние */
.guilds-column.collapsed .invites-btn {
  justify-content: center;
  padding: 10px 0;
}
.guilds-column.collapsed .invites-badge {
  position: absolute;
  top: 4px;
  right: 4px;
  border: 2px solid var(--bg-elevated);
}
</style>
