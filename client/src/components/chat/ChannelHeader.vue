<template>
  <div class="channel-header">
    <!-- Левая часть: Имя и Описание -->
    <div class="header-left">
      <span class="prefix">#</span>
      <span class="channel-name" :title="channelName">{{ displayChannelName }}</span>
      <span class="channel-description" v-if="description" :title="description">
        {{ description }}
      </span>
    </div>

    <!-- Правая часть: Действия -->
    <div class="header-right">
      <!-- Закрепленные сообщения -->
      <button class="action-btn" title="Закреплённые сообщения" @click="$emit('pins')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M16 12V4h1v-2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"/>
        </svg>
      </button>

      <!-- Переключатель правой инфо-панели -->
      <button
        class="action-btn"
        :class="{ active: showInfoColumn }"
        title="Панель информации"
        @click="$emit('toggle-info')"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
        </svg>
      </button>

      <!-- Поиск в канале -->
      <div class="search-box">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Поиск по каналу"
          class="search-input"
          @input="$emit('search', searchQuery)"
        />
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
        </svg>
      </div>

      <!-- Дополнительно -->
      <button class="action-btn" title="Дополнительно" @click="$emit('more')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  channelName: string
  description?: string
  showInfoColumn: boolean
}>()

defineEmits(['toggle-info', 'search', 'pins', 'more'])

const searchQuery = ref('')

const displayChannelName = computed(() => {
  const name = props.channelName
  if (name.length > 30) {
    return name.slice(0, 29) + '...'
  }
  return name
})
</script>

<style scoped>
.channel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px; /* Увеличено для лучшего соответствия макету */
  padding: 0 16px;
  background: var(--bg-app);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  min-width: 0;
}

.prefix {
  color: var(--text-muted);
  font-size: 20px;
  font-weight: 700;
  margin-right: 4px;
}

.channel-name {
  font-size: 18px; /* Изменено с 16px на 18px */
  font-weight: 700;
  color: var(--text-primary);
  white-space: nowrap;
}

.channel-description {
  font-size: 14px; /* Изменено с 12px на 14px */
  color: var(--text-secondary);
  border-left: 1px solid var(--border);
  margin-left: 12px;
  padding-left: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 600px) {
  .channel-description {
    display: none;
  }
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px; /* Зазор 8px */
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-input {
  width: 160px;
  height: 32px;
  padding-left: 12px;
  padding-right: 32px;
  font-size: 13px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  transition: width 0.2s, border-color 0.2s;
}
.search-input::placeholder {
  color: var(--text-muted);
}
.search-input:focus {
  width: 200px;
  border-color: var(--accent);
}

.search-icon {
  position: absolute;
  right: 10px;
  color: var(--text-muted);
  pointer-events: none;
}

@media (max-width: 768px) {
  .search-box {
    display: none;
  }
}

.action-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  transition: color 0.15s, background-color 0.15s;
}
.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.action-btn.active {
  color: var(--accent);
  background: rgba(124, 108, 255, 0.1);
}
.action-btn svg {
  width: 20px;
  height: 20px;
}
</style>
