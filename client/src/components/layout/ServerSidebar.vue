<template>
  <div class="server-sidebar">
    <div
      v-for="server in servers"
      :key="server.url"
      class="server-icon"
      :class="{ active: server.url === activeUrl }"
      :title="server.name"
      @click="$emit('select-server', server.url)"
      @contextmenu.prevent="openMenu($event, server.url)"
    >
      <span>{{ server.name[0]?.toUpperCase() }}</span>
      <div class="server-tooltip">{{ server.name }}</div>
    </div>

    <div v-if="servers.length" class="divider" />

    <div class="server-icon add-btn" title="Добавить сервер" @click="$emit('add-server')">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
      </svg>
    </div>

    <div class="spacer" />

    <div class="server-icon settings-btn" title="Настройки" @click="$emit('open-settings')">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.02 7.02 0 0 0-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.47.47 0 0 0-.59.22L2.74 8.87a.49.49 0 0 0 .12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.01-1.58zM12 15.6a3.6 3.6 0 1 1 0-7.2 3.6 3.6 0 0 1 0 7.2z"/>
      </svg>
    </div>

    <!-- Кастомное контекстное меню -->
    <Teleport to="body">
      <div v-if="menu.visible" class="ctx-overlay" @mousedown.self="closeMenu" @contextmenu.prevent>
        <div class="ctx-menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }">
          <button class="ctx-item" @click="disconnect">Отключиться</button>
          <div class="ctx-divider" />
          <button class="ctx-item danger" @click="removeServer">Удалить сервер</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import type { ServerInstance } from '../../stores/servers'

defineProps<{ servers: ServerInstance[]; activeUrl: string | null }>()
const emit = defineEmits(['select-server', 'add-server', 'remove-server', 'disconnect-server', 'open-settings'])

const menu = reactive({ visible: false, x: 0, y: 0, url: '' })

function openMenu(e: MouseEvent, url: string) {
  menu.url = url
  menu.x = Math.min(e.clientX, window.innerWidth - 160)
  menu.y = Math.min(e.clientY, window.innerHeight - 80)
  menu.visible = true
}

function closeMenu() {
  menu.visible = false
}

function disconnect() {
  emit('disconnect-server', menu.url)
  closeMenu()
}

function removeServer() {
  emit('remove-server', menu.url)
  closeMenu()
}
</script>

<style scoped>
.server-sidebar {
  width: var(--guilds-w);
  background: var(--bg-darkest);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0;
  gap: 4px;
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
}
.server-icon {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--bg-light);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 18px;
  font-weight: 700;
  color: var(--text2);
  transition: all 0.15s;
  overflow: visible;
  flex-shrink: 0;
}
.server-icon:hover { border-radius: 14px; color: var(--accent); background: var(--bg-hover); }
.server-icon.active { border-radius: 14px; color: var(--accent); background: var(--bg); box-shadow: -4px 0 0 0 var(--accent); }
.server-icon.add-btn { color: var(--green); }
.server-icon.add-btn:hover { border-radius: 14px; }
.server-icon.settings-btn { color: var(--text3); }
.server-tooltip {
  position: absolute;
  left: calc(100% + 12px);
  top: 50%;
  transform: translateY(-50%);
  background: var(--bg-dark);
  border: 1px solid var(--border);
  color: var(--text);
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s;
  z-index: 100;
}
.server-icon:hover .server-tooltip { opacity: 1; }
.divider { width: 32px; height: 1px; background: var(--border); flex-shrink: 0; margin: 4px 0; }
.spacer { flex: 1; }

.ctx-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
}
.ctx-menu {
  position: fixed;
  background: var(--bg-dark);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 4px;
  min-width: 150px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.4);
}
.ctx-item {
  display: block;
  width: 100%;
  text-align: left;
  background: none;
  border: none;
  color: var(--text1);
  padding: 7px 12px;
  border-radius: 5px;
  cursor: pointer;
  font-size: 13px;
}
.ctx-item:hover { background: var(--bg-hover); }
.ctx-item.danger { color: var(--red, #f04747); }
.ctx-item.danger:hover { background: rgba(240,71,71,0.15); }
.ctx-divider { height: 1px; background: var(--border); margin: 3px 0; }
</style>
