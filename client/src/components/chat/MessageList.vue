<template>
  <div class="messages-list-wrapper">
    <!-- Список сообщений -->
    <div class="scroll-container" ref="scrollEl" @scroll="handleScroll">
      <!-- Загрузка старых сообщений -->
      <div v-if="loading" class="list-state-msg">
        <span class="spinner-sm" />
        Загрузка истории...
      </div>
      
      <!-- Пустое состояние -->
      <div v-if="messages.length === 0 && !loading" class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" class="empty-icon">
          <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H5.2L4 17.2V4h16v12z"/>
        </svg>
        <span class="empty-title">Здесь пока пусто.</span>
        <span class="empty-subtitle">Напишите первое сообщение в этот канал!</span>
      </div>

      <!-- Список сообщений -->
      <div class="messages-list" v-else>
        <MessageItem
          v-for="msg in messages"
          :key="msg.id"
          :msg="msg"
          :userId="userId"
          :username="username"
          :members="members"
          :replySource="getReplySource(msg)"
          :isEditing="msg.id === editingMessageId"
          @toggle-reaction="(mId, emoji, active) => $emit('toggle-reaction', mId, emoji, active)"
          @reply="$emit('reply', $event)"
          @add-reaction-click="(e, mId) => $emit('add-reaction-click', e, mId)"
          @edit-message="(mId, content) => $emit('edit-message', mId, content)"
          @delete-message="(mId) => $emit('delete-message', mId)"
          @cancel-edit="$emit('cancel-edit')"
          @contextmenu="(e, m) => $emit('contextmenu', e, m)"
        />
      </div>
    </div>

    <!-- Плашка «↓ Новые сообщения» -->
    <transition name="fade">
      <button v-if="showNewMessageBadge" class="new-messages-badge" @click="scrollToBottom">
        ↓ Новые сообщения
      </button>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import MessageItem from './MessageItem.vue'
import type { Message, Member } from '../../stores/guild'

const props = withDefaults(defineProps<{
  messages: Message[]
  userId: string
  username: string
  members: Member[]
  loading: boolean
  editingMessageId?: string | null
}>(), {
  editingMessageId: null
})

const emit = defineEmits([
  'load-more',
  'toggle-reaction',
  'reply',
  'add-reaction-click',
  'edit-message',
  'delete-message',
  'cancel-edit',
  'contextmenu'
])

const scrollEl = ref<HTMLElement | null>(null)
const showNewMessageBadge = ref(false)

let previousScrollHeight = 0
let autoScrollLock = false

function getReplySource(msg: Message): Message | null {
  if (!msg.reply_to) return null
  return props.messages.find(m => m.id === msg.reply_to) ?? null
}

function handleScroll() {
  const el = scrollEl.value
  if (!el) return

  // 1. Проверяем скролл на самый верх для ленивой подгрузки
  if (el.scrollTop === 0 && !props.loading && props.messages.length > 0) {
    previousScrollHeight = el.scrollHeight
    autoScrollLock = true
    emit('load-more')
  }

  // 2. Если доскроллили до низа, скрываем плашку
  const isNearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40
  if (isNearBottom) {
    showNewMessageBadge.value = false
  }
}

function scrollToBottom() {
  nextTick(() => {
    const el = scrollEl.value
    if (el) {
      el.scrollTop = el.scrollHeight
      showNewMessageBadge.value = false
    }
  })
}

// Следим за изменениями длины массива сообщений
watch(() => props.messages.length, (_, oldLen) => {
  const el = scrollEl.value
  if (!el) return

  if (autoScrollLock) {
    // Сохраняем положение скролла после подгрузки истории
    nextTick(() => {
      el.scrollTop = el.scrollHeight - previousScrollHeight
      autoScrollLock = false
    })
  } else {
    // При новом сообщении:
    const isNearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 120
    
    if (isNearBottom || oldLen === 0) {
      scrollToBottom()
    } else {
      // Пользователь читал историю выше — показываем плашку
      showNewMessageBadge.value = true
    }
  }
})

onMounted(() => {
  scrollToBottom()
})
</script>

<style scoped>
.messages-list-wrapper {
  flex: 1;
  min-height: 0;
  position: relative;
  display: flex;
  flex-direction: column;
  background: var(--bg-app);
}

.scroll-container {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.messages-list {
  padding: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.list-state-msg {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  font-size: 12px;
  color: var(--text-secondary);
  background: rgba(0, 0, 0, 0.1);
  user-select: none;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 40px;
  text-align: center;
  user-select: none;
}

.empty-icon {
  color: var(--text-muted);
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.empty-subtitle {
  font-size: 12px;
  color: var(--text-secondary);
}

/* Плашка */
.new-messages-badge {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--accent-grad);
  color: #fff;
  font-weight: 700;
  font-size: 12px;
  padding: 8px 16px;
  border-radius: 20px;
  box-shadow: 0 4px 16px rgba(124, 108, 255, 0.4);
  z-index: 50;
  transition: opacity 0.2s, transform 0.2s;
}
.new-messages-badge:hover {
  transform: translateX(-50%) translateY(-1px);
  box-shadow: 0 6px 20px rgba(124, 108, 255, 0.5);
}

/* Спиннер загрузки */
.spinner-sm {
  width: 12px;
  height: 12px;
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Анимации */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}
</style>
