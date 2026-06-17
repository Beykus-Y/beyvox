<template>
  <div class="activity-feed-section">
    <!-- Шапка секции -->
    <div class="feed-header">
      <span class="uppercase-label">Последние события</span>
    </div>

    <!-- Список событий -->
    <div class="feed-list">
      <div v-if="events.length === 0" class="empty-feed">
        <span class="empty-text">Пока нет событий</span>
      </div>
      <div v-else class="events-container">
        <ActivityItem
          v-for="ev in limitedEvents"
          :key="ev.id"
          :event="ev"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ActivityItem from './ActivityItem.vue'
import type { ActivityEvent } from '../../stores/activity'

const props = defineProps<{
  events: ActivityEvent[]
}>()

// Ограничиваем отображение последними 15 событиями в сайдбаре
const limitedEvents = computed(() => {
  return props.events.slice(0, 15)
})
</script>

<style scoped>
.activity-feed-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 16px;
}

.feed-header {
  padding: 0 4px;
}

.feed-list {
  display: flex;
  flex-direction: column;
}

.events-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.empty-feed {
  padding: 12px;
  text-align: center;
  border: 1px dashed var(--border);
  border-radius: var(--radius-item);
}

.empty-text {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
