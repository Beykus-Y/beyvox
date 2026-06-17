<template>
  <div
    class="server-card"
    :class="{ active: isActive }"
    @click="$emit('select', server.url)"
    @contextmenu.prevent="openContextMenu"
  >
    <!-- Верхняя часть карточки (Баннер) -->
    <div class="server-banner" :style="bannerStyle">
      <div class="banner-overlay" />
      <div class="banner-content">
        <span class="server-name">{{ server.name }}</span>
      </div>
    </div>

    <!-- Нижняя часть карточки (Метрики и Подключение) -->
    <div class="server-card-body">
      <div class="status-metrics-block">
        <div class="server-status-row">
          <span class="status-dot" :class="{ online: server.online }" />
          <span class="status-text">{{ server.online ? 'Онлайн' : 'Не в сети' }}</span>
        </div>
        <div class="metrics-row">
          <template v-if="server.online">
            <span class="metric-text" v-if="server.guildsCount !== undefined || server.onlineCount !== undefined">
              {{ pluralGuilds(server.guildsCount || 0) }} · {{ server.onlineCount || 0 }} онлайн
            </span>
            <span class="metric-text" v-else>Гильдий: — · Онлайн: —</span>
          </template>
          <template v-else>
            <span class="metric-text">
              Последний вход: {{ formatRelativeTime(server.lastSeenAt) }}
            </span>
          </template>
        </div>
      </div>

      <!-- Кнопка подключения / статус подключения -->
      <div class="connection-status-row" @click.stop>
        <div v-if="isConnected" class="connected-badge">
          <div class="connected-left">
            <svg class="plug-icon" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
            </svg>
            <span class="connected-text">Подключено</span>
          </div>
          <div class="connected-right">
            <span class="ping-dot" />
            <span
              v-if="server.pingMs !== undefined && server.pingMs !== null"
              class="ping-value"
              :class="pingClass(server.pingMs)"
            >
              {{ server.pingMs }}ms
            </span>
            <span v-else class="ping-value">—ms</span>
          </div>
        </div>
        
        <button
          v-else
          class="connect-btn-wide"
          :disabled="loading"
          @click="$emit('connect', server.url)"
        >
          {{ loading ? 'Подключение...' : 'Подключиться' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ServerInstance } from '../../stores/servers'

const props = defineProps<{
  server: ServerInstance
  isActive: boolean
  isConnected: boolean
  loading: boolean
}>()

const emit = defineEmits(['select', 'connect', 'contextmenu'])

const bannerStyle = computed(() => {
  if (props.server.bannerUrl) {
    return {
      backgroundImage: `url(${props.server.bannerUrl})`
    }
  }
  return {
    background: getDeterministicGradient(props.server.url || props.server.name)
  }
})

function openContextMenu(e: MouseEvent) {
  emit('contextmenu', e, props.server.url)
}

function getDeterministicGradient(input: string): string {
  let hash1 = 0
  let hash2 = 0
  for (let i = 0; i < input.length; i++) {
    hash1 = input.charCodeAt(i) + ((hash1 << 5) - hash1)
    hash2 = input.charCodeAt(i) * 31 + ((hash2 << 7) - hash2)
  }
  const color1 = `hsl(${Math.abs(hash1) % 360}, 50%, 35%)`
  const color2 = `hsl(${Math.abs(hash2) % 360}, 45%, 18%)`
  return `linear-gradient(135deg, ${color1}, ${color2})`
}

function pluralGuilds(count: number): string {
  const mod10 = count % 10
  const mod100 = count % 100
  if (mod10 === 1 && mod100 !== 11) {
    return `${count} гильдия`
  } else if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
    return `${count} гильдии`
  } else {
    return `${count} гильдий`
  }
}

function formatRelativeTime(dateStr?: string): string {
  if (!dateStr) return 'давно'
  const past = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - past.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMins / 60)
  const diffDays = Math.floor(diffHours / 24)

  if (diffMins < 1) return 'только что'
  if (diffMins < 60) return `${diffMins} мин. назад`
  if (diffHours < 24) return `${diffHours} ч. назад`
  return `${diffDays} дн. назад`
}

function pingClass(ping: number) {
  if (ping <= 60) return 'ping-good'
  if (ping <= 150) return 'ping-mid'
  return 'ping-bad'
}
</script>

<style scoped>
.server-card {
  width: 100%;
  height: 140px;
  background: var(--bg-elevated);
  border: 1.5px solid transparent;
  border-radius: 14px;
  overflow: hidden;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1), box-shadow 0.2s, border-color 0.2s;
  user-select: none;
}

.server-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(124, 108, 255, 0.15), 0 8px 24px rgba(0, 0, 0, 0.35);
}

.server-card.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent), 0 4px 16px rgba(124, 108, 255, 0.2);
  background: var(--bg-active);
}

.server-banner {
  height: 66px;
  background-size: cover;
  background-position: center;
  position: relative;
  display: flex;
  align-items: flex-end;
  padding: 8px 12px;
  flex-shrink: 0;
}

.banner-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(to bottom, transparent 40%, rgba(0, 0, 0, 0.7) 100%);
  z-index: 1;
}

.banner-content {
  position: relative;
  z-index: 2;
  width: 100%;
  display: flex;
  flex-direction: column;
}

.server-name {
  font-weight: 700;
  font-size: 14px;
  color: #ffffff;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.8);
}

.server-card-body {
  padding: 4px 12px 6px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  flex: 1;
  background: var(--bg-elevated);
}
.server-card.active .server-card-body {
  background: var(--bg-active);
}

.status-metrics-block {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.server-status-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--offline);
  box-shadow: 0 0 4px var(--offline);
}
.status-dot.online {
  background: var(--online);
  box-shadow: 0 0 4px var(--online);
}

.status-text {
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 500;
  line-height: 1.2;
}

.metrics-row {
  display: flex;
  align-items: center;
}

.metric-text {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 400;
  line-height: 1.2;
}

.connection-status-row {
  display: flex;
  width: 100%;
}

.connected-badge {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  font-size: 11px;
  font-weight: 700;
  color: var(--online);
  background: rgba(35, 197, 94, 0.08);
  padding: 0 10px;
  height: 26px;
  border-radius: var(--radius-item);
  border: 1.5px solid var(--online);
}

.connected-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.plug-icon {
  color: var(--online);
}

.connected-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.ping-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--online);
}

.ping-value {
  font-family: monospace;
  font-size: 11px;
  font-weight: 600;
}
.ping-value.ping-good {
  color: var(--ping-good);
}
.ping-value.ping-mid {
  color: var(--ping-mid);
}
.ping-value.ping-bad {
  color: var(--ping-bad);
}

.connect-btn-wide {
  width: 100%;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  font-weight: 600;
  height: 26px;
  padding: 0;
  border-radius: var(--radius-item);
  text-align: center;
  font-size: 11px;
  transition: all 0.15s;
}
.connect-btn-wide:hover:not(:disabled) {
  background: var(--bg-active);
  border-color: var(--text-secondary);
  color: var(--text-primary);
}
.connect-btn-wide:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
