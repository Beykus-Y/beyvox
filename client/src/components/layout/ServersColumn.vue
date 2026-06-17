<template>
  <div class="servers-column">
    <!-- Шапка бренда -->
    <AppBrand :username="username" @open-settings="$emit('open-settings')" @logout="$emit('logout')" />

    <!-- Заголовок секции -->
    <div class="section-header">
      <span class="uppercase-label">Мои сервера</span>
    </div>

    <!-- Список серверов -->
    <div class="servers-list-container">
      <div v-if="servers.length === 0" class="empty-servers">
        <span class="empty-text">Серверов пока нет.</span>
      </div>
      
      <div class="servers-list" v-else>
        <ServerCard
          v-for="srv in servers"
          :key="srv.url"
          :server="srv"
          :isActive="srv.url === activeUrl"
          :isConnected="srv.url === connectedUrl"
          :loading="srv.url === loadingUrl"
          @select="$emit('select-server', srv.url)"
          @connect="$emit('connect-server', srv.url)"
          @contextmenu="openMenu($event, srv.url)"
        />
      </div>

      <!-- Кнопка добавления сервера -->
      <button class="add-server-dashed" @click="$emit('add-server')" title="Добавить новый сервер">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
        <span>Добавить сервер</span>
      </button>
    </div>

    <!-- Блок пользователя -->
    <UserBar @open-settings="$emit('open-settings')" />

    <!-- Версия приложения -->
    <div class="app-version-container">
      <span class="app-version-text">BeyVox v{{ appVersion }}</span>
    </div>

    <!-- Контекстное меню -->
    <Teleport to="body">
      <div v-if="menu.visible" class="ctx-overlay" @mousedown.self="closeMenu" @contextmenu.prevent>
        <div class="ctx-menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }">
          <button class="ctx-item" @click="triggerDisconnect">Отключиться</button>
          <div class="ctx-divider" />
          <button class="ctx-item danger" @click="triggerRemove">Удалить сервер</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import AppBrand from './AppBrand.vue'
import ServerCard from './ServerCard.vue'
import UserBar from './UserBar.vue'
import type { ServerInstance } from '../../stores/servers'

defineProps<{
  servers: ServerInstance[]
  activeUrl: string | null
  connectedUrl: string | null
  loadingUrl: string | null
  username: string
}>()

const emit = defineEmits([
  'select-server',
  'connect-server',
  'add-server',
  'disconnect-server',
  'remove-server',
  'open-settings',
  'logout'
])

const appVersion = ref('0.2.0')

const menu = reactive({ visible: false, x: 0, y: 0, url: '' })

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '0.2.0'
  }
})

function openMenu(e: MouseEvent, url: string) {
  menu.url = url
  menu.x = Math.min(e.clientX, window.innerWidth - 160)
  menu.y = Math.min(e.clientY, window.innerHeight - 80)
  menu.visible = true
}

function closeMenu() {
  menu.visible = false
}

function triggerDisconnect() {
  emit('disconnect-server', menu.url)
  closeMenu()
}

function triggerRemove() {
  emit('remove-server', menu.url)
  closeMenu()
}
</script>

<style scoped>
.servers-column {
  width: 260px;
  background: var(--bg-panel);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  height: 100vh;
  flex-shrink: 0;
}

.section-header {
  padding: 16px 16px 8px;
  flex-shrink: 0;
}

.servers-list-container {
  flex: 1;
  overflow-y: auto;
  padding: 0 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.servers-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-top: 6px;
}

.empty-servers {
  padding: 24px 0;
  text-align: center;
  padding-top: 10px;
}

.empty-text {
  font-size: 12px;
  color: var(--text-muted);
}

.add-server-dashed {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px;
  border: 1px dashed var(--border);
  border-radius: var(--radius-card);
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 13px;
  background: transparent;
}
.add-server-dashed:hover {
  background: rgba(124, 108, 255, 0.05);
  border-color: var(--accent);
  color: var(--text-primary);
}

.app-version-container {
  padding: 4px 12px 10px;
  text-align: left;
  background: var(--bg-panel);
  flex-shrink: 0;
}

.app-version-text {
  font-size: 10px;
  color: var(--text-muted);
}
</style>
