<template>
  <div class="activity-item">
    <!-- Маленький аватар актора -->
    <div class="actor-avatar">
      <span>{{ event.actor.username[0]?.toUpperCase() }}</span>
    </div>
    
    <!-- Тело события -->
    <div class="event-body">
      <div class="event-text-and-time">
        <div class="event-desc-wrapper">
          <span class="event-desc">
            <strong class="actor-name">{{ event.actor.username }}</strong>
            {{ eventActionText }}
            <strong class="target-label" v-if="event.targetLabel">{{ event.targetLabel }}</strong>
          </span>
        </div>
        
        <!-- Относительное время -->
        <span class="event-time" :title="formatFullDate(event.createdAt)">
          {{ relativeTime }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { ActivityEvent } from '../../stores/activity'

const props = defineProps<{
  event: ActivityEvent
}>()

const relativeTime = ref('')
let timer: any = null

const eventActionText = computed(() => {
  switch (props.event.type) {
    case 'message_sent':
      return 'отправил(а) сообщение в'
    case 'channel_created':
      return 'создал(а) новый канал'
    case 'voice_joined':
      return 'подключился(лась) к голосовому каналу'
    case 'guild_joined':
      return 'присоединился(лась) к гильдии'
    default:
      return 'совершил(а) действие'
  }
})

function formatRelativeTime(isoStr: string): string {
  const past = new Date(isoStr)
  const now = new Date()
  const diffMs = now.getTime() - past.getTime()
  const diffSecs = Math.floor(diffMs / 1000)
  const diffMins = Math.floor(diffSecs / 60)
  const diffHours = Math.floor(diffMins / 60)

  if (diffSecs < 10) return 'только что'
  if (diffSecs < 60) return `${diffSecs} сек. назад`
  if (diffMins < 60) return `${diffMins} мин. назад`
  if (diffHours < 24) return `${diffHours} ч. назад`
  return past.toLocaleDateString('ru', { day: 'numeric', month: 'short' })
}

function formatFullDate(isoStr: string): string {
  return new Date(isoStr).toLocaleString('ru')
}

function updateTime() {
  relativeTime.value = formatRelativeTime(props.event.createdAt)
}

onMounted(() => {
  updateTime()
  timer = setInterval(updateTime, 15000) // Обновляем каждые 15 сек.
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<style scoped>
.activity-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--bg-elevated);
  border: 1px solid transparent;
  transition: all 0.15s ease;
  user-select: none;
}
.activity-item:hover {
  background: var(--bg-hover);
  border-color: var(--border);
}

.actor-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--bg-active);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 11px;
  color: var(--accent);
  border: 1px solid var(--border);
  flex-shrink: 0;
  margin-top: 2px;
}

.event-body {
  flex: 1;
  min-width: 0;
}

.event-text-and-time {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 8px;
}

.event-desc-wrapper {
  flex: 1;
  min-width: 140px;
}

.event-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  word-wrap: break-word;
  word-break: break-word;
}

.actor-name {
  color: var(--text-primary);
  font-weight: 600;
}

.target-label {
  color: var(--accent);
  font-weight: 600;
}

.event-time {
  font-size: 13px;
  color: var(--text-muted);
  flex-shrink: 0;
  white-space: nowrap;
  margin-left: auto;
}
</style>
