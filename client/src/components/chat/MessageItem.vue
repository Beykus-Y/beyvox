<template>
  <div
    :id="'msg-' + msg.id"
    class="message-container"
    :class="{ own: msg.author_id === userId, mentioned: isMentioned }"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
    @contextmenu.prevent="$emit('contextmenu', $event, msg)"
  >
    <!-- Аватар -->
    <div class="user-avatar-col">
      <div class="message-avatar" :style="avatarStyle">
        <span>{{ msg.author_username[0]?.toUpperCase() }}</span>
      </div>
    </div>

    <!-- Тело сообщения -->
    <div class="message-content-col">
      <div class="message-header">
        <span class="author-name" :style="{ color: authorColor }">{{ msg.author_username }}</span>
        <span v-if="msg.author_username === 'python'" class="bot-tag">BOT</span>
        <span class="message-time">{{ formatTime(msg.created_at) }}</span>
        <span class="edited-label" v-if="msg.edited_at">(ред.)</span>
      </div>

      <!-- Превью ответа -->
      <div v-if="msg.reply_to && replySource" class="reply-preview-container">
        <span class="reply-symbol">↪</span>
        <span class="reply-author" :style="{ color: getAuthorColor(replySource.author_id) }">
          {{ replySource.author_username }}
        </span>
        <span class="reply-snippet" :title="replySource.content">
          {{ replySource.content }}
        </span>
      </div>

      <!-- Текст сообщения (Markdown или редактор) -->
      <div v-if="isEditing" class="message-edit-container">
        <textarea
          v-model="editContent"
          class="message-edit-input"
          @keydown.enter.prevent="saveEdit"
          @keydown.esc="cancelEdit"
          rows="1"
          ref="editInput"
        />
        <div class="message-edit-actions">
          <span class="edit-hint">ESC — отмена · Enter — сохранить</span>
          <div class="edit-buttons">
            <button class="edit-btn cancel" @click="cancelEdit">Отмена</button>
            <button class="edit-btn save" @click="saveEdit">Сохранить</button>
          </div>
        </div>
      </div>
      <div v-else class="message-text" v-html="parsedMarkdown" @click="handleTextClick" />

      <!-- Чипы реакций -->
      <div class="message-reactions-row" v-if="msg.reactions && msg.reactions.length">
        <button
          v-for="r in msg.reactions"
          :key="r.emoji"
          class="reaction-chip"
          :class="{ active: r.me }"
          @click.stop="$emit('toggle-reaction', msg.id, r.emoji, r.me)"
          :title="r.emoji"
        >
          <span class="emoji-symbol">{{ r.emoji }}</span>
          <span class="emoji-count">{{ r.count }}</span>
        </button>
        
        <!-- Быстрое добавление реакции -->
        <button class="add-reaction-chip" @click.stop="$emit('add-reaction-click', $event, msg.id)">
          +
        </button>
      </div>
    </div>

    <!-- Всплывающие действия при ховере -->
    <div class="message-hover-actions" v-if="hovered">
      <button class="hover-action-btn" @click.stop="$emit('add-reaction-click', $event, msg.id)" title="Добавить реакцию">
        😊
      </button>
      <button class="hover-action-btn" @click="$emit('reply', msg)" title="Ответить">
        ↩
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { openUrl as openExternalLink } from '@tauri-apps/plugin-opener'
import type { Message, Member } from '../../stores/guild'

const props = withDefaults(defineProps<{
  msg: Message
  userId: string
  username: string
  members: Member[]
  replySource?: Message | null
  isEditing?: boolean
}>(), {
  isEditing: false
})

const emit = defineEmits([
  'toggle-reaction',
  'reply',
  'add-reaction-click',
  'edit-message',
  'delete-message',
  'cancel-edit',
  'contextmenu'
])

const hovered = ref(false)

const editContent = ref(props.msg.content)
const editInput = ref<HTMLTextAreaElement | null>(null)

watch(() => props.isEditing, (newVal) => {
  if (newVal) {
    editContent.value = props.msg.content
    nextTick(() => {
      editInput.value?.focus()
    })
  }
})

function saveEdit() {
  const trimmed = editContent.value.trim()
  if (!trimmed) {
    emit('cancel-edit')
    return
  }
  if (trimmed === props.msg.content) {
    emit('cancel-edit')
    return
  }
  emit('edit-message', props.msg.id, trimmed)
}

function cancelEdit() {
  emit('cancel-edit')
}

const authorColor = computed(() => getAuthorColor(props.msg.author_id))

const isMentioned = computed(() => {
  return props.msg.mention_user_ids?.includes(props.userId) ?? false
})

const parsedMarkdown = computed(() => {
  return renderMarkdown(props.msg.content)
})

const avatarStyle = computed(() => {
  return {
    background: getDeterministicBackground(props.msg.author_id)
  }
})

function getDeterministicBackground(id: string): string {
  let hash = 0
  for (let i = 0; i < id.length; i++) {
    hash = id.charCodeAt(i) + ((hash << 5) - hash)
  }
  return `hsl(${Math.abs(hash) % 360}, 50%, 35%)`
}

function getAuthorColor(authorId: string): string {
  if (props.msg.author_username === 'python') return '#5865f2'
  if (authorId === props.userId) return 'var(--accent)'
  
  // Детерминированный цвет на основе ID
  let hash = 0
  for (let i = 0; i < authorId.length; i++) {
    hash = authorId.charCodeAt(i) + ((hash << 5) - hash)
  }
  return `hsl(${Math.abs(hash) % 360}, 65%, 65%)`
}

function formatTime(iso: string): string {
  const date = new Date(iso)
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  const timeStr = date.toLocaleTimeString('ru', { hour: '2-digit', minute: '2-digit' })
  
  if (date.toDateString() === today.toDateString()) {
    return `Сегодня, ${timeStr}`
  } else if (date.toDateString() === yesterday.toDateString()) {
    return `Вчера в ${timeStr}`
  } else {
    const day = date.getDate()
    const months = ['янв.', 'февр.', 'мар.', 'апр.', 'май', 'июн.', 'июл.', 'авг.', 'сент.', 'окт.', 'нояб.', 'дек.']
    const monthStr = months[date.getMonth()]
    return `${day} ${monthStr} в ${timeStr}`
  }
}

function renderMarkdown(text: string): string {
  // 1. Экранирование HTML во избежание XSS
  let escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

  // 2. Обработка многострочного кода ```
  escaped = escaped.replace(/```([\s\S]+?)```/g, '<pre><code>$1</code></pre>')

  // 3. Обработка инлайн-кода `
  escaped = escaped.replace(/`([^`\n]+)`/g, '<code>$1</code>')

  // 4. Полужирный **
  escaped = escaped.replace(/\*\*([^*]+?)\*\*/g, '<strong>$1</strong>')

  // 5. Курсив *
  escaped = escaped.replace(/\*([^*]+?)\*/g, '<em>$1</em>')

  // 6. Поддержка ссылок в формате [Текст](Ссылка)
  escaped = escaped.replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer" class="chat-link">$1</a>')

  // 7. Поддержка сырых URL (не находящихся внутри <a>)
  // Для простоты подменяем любые явные http/https ссылки, исключая те, что уже завернуты в href
  escaped = escaped.replace(/(?<!href=")(https?:\/\/[^\s<]+)/g, '<a href="$1" target="_blank" rel="noopener noreferrer" class="chat-link">$1</a>')

  // 8. Обработка упоминаний @username
  const memberUsernames = props.members.map(m => m.username)
  escaped = escaped.replace(/@([a-zA-Z0-9_]{1,50})/g, (match, name) => {
    const isMember = memberUsernames.includes(name)
    const isMe = name === props.username
    return isMember
      ? `<span class="mention${isMe ? ' mention-me' : ''}">@${name}</span>`
      : match
  })

  // 9. Переносы строк
  escaped = escaped.replace(/\n/g, '<br>')

  return escaped
}

async function handleTextClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.tagName === 'A' && target.classList.contains('chat-link')) {
    e.preventDefault()
    const url = target.getAttribute('href')
    if (url) {
      try {
        await openExternalLink(url)
      } catch (err) {
        console.error('Не удалось открыть ссылку через Tauri Opener:', err)
      }
    }
  }
}
</script>

<style scoped>
.message-container {
  display: flex;
  gap: 12px;
  padding: 6px 16px;
  position: relative;
  border-radius: 6px;
  transition: background-color 0.1s;
}
.message-container:hover {
  background: var(--bg-hover);
}
.message-container.mentioned {
  background: rgba(124, 108, 255, 0.08);
  border-left: 2px solid var(--accent);
  padding-left: 14px;
}
.message-container.mentioned:hover {
  background: rgba(124, 108, 255, 0.12);
}

.user-avatar-col {
  flex-shrink: 0;
}

.message-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  color: #ffffff;
}

.message-content-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
  user-select: none;
}

.author-name {
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}
.author-name:hover {
  text-decoration: underline;
}

.bot-tag {
  background: var(--accent);
  color: #ffffff;
  font-size: 10px;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 4px;
  text-transform: uppercase;
}

.message-time {
  font-size: 12px;
  color: var(--text-secondary);
}

.edited-label {
  font-size: 10px;
  color: var(--text-muted);
  font-style: italic;
}

/* Превью ответа */
.reply-preview-container {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
  background: var(--bg-elevated);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  width: fit-content;
  max-width: 100%;
  border-left: 3px solid var(--accent);
  color: var(--text-secondary);
}

.reply-symbol {
  color: var(--text-muted);
  font-weight: 700;
}

.reply-author {
  font-weight: 600;
  flex-shrink: 0;
}

.reply-snippet {
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Текст */
.message-text {
  font-size: 14px;
  line-height: 1.45;
  color: var(--text-primary);
  word-wrap: break-word;
  word-break: break-word;
  margin-top: 2px;
}
.message-text :deep(pre) {
  background: var(--bg-app);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 10px;
  margin: 6px 0;
  overflow-x: auto;
}
.message-text :deep(code) {
  font-family: 'JetBrains Mono', Consolas, Monaco, monospace;
  font-size: 13px;
  background: var(--bg-app);
  padding: 2px 4px;
  border-radius: 4px;
  color: #f43f5e;
}
.message-text :deep(pre code) {
  background: transparent;
  padding: 0;
  color: var(--text-primary);
}
.message-text :deep(.mention) {
  background: rgba(124, 108, 255, 0.15);
  color: var(--accent);
  font-weight: 600;
  border-radius: 4px;
  padding: 1px 4px;
}
.message-text :deep(.mention-me) {
  background: var(--accent);
  color: #fff;
}
.message-text :deep(.chat-link) {
  color: var(--accent);
}
.message-text :deep(.chat-link:hover) {
  text-decoration: underline;
}

/* Реакции */
.message-reactions-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 6px;
  align-items: center;
}

.reaction-chip {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-reaction);
  color: var(--text-secondary);
  transition: all 0.1s ease;
}
.reaction-chip:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}
.reaction-chip.active {
  background: rgba(124, 108, 255, 0.12);
  border-color: var(--accent);
  color: var(--accent);
}

.emoji-symbol {
  font-size: 13px;
}

.emoji-count {
  font-size: 11px;
  font-weight: 700;
}

.add-reaction-chip {
  padding: 2px 8px;
  border: 1px dashed var(--border);
  border-radius: var(--radius-reaction);
  color: var(--text-muted);
  font-weight: 600;
}
.add-reaction-chip:hover {
  border-color: var(--accent);
  color: var(--text-primary);
  background: rgba(124, 108, 255, 0.05);
}

/* Плавающая панель действий при ховере */
.message-hover-actions {
  position: absolute;
  top: -14px;
  right: 16px;
  display: flex;
  gap: 2px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 3px 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 10;
}

.hover-action-btn {
  width: 26px;
  height: 26px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 13px;
}
.hover-action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Редактор сообщения */
.message-edit-container {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
  width: 100%;
}

.message-edit-input {
  width: 100%;
  background: var(--bg-app);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text-primary);
  padding: 8px 12px;
  font-size: 14px;
  font-family: inherit;
  resize: vertical;
  min-height: 38px;
  outline: none;
}
.message-edit-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.message-edit-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
}

.edit-hint {
  color: var(--text-muted);
}

.edit-buttons {
  display: flex;
  gap: 8px;
}

.edit-btn {
  padding: 4px 10px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s;
}
.edit-btn.cancel {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-secondary);
}
.edit-btn.cancel:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.edit-btn.save {
  background: var(--accent-grad);
  border: none;
  color: #fff;
}
.edit-btn.save:hover {
  opacity: 0.9;
}

/* Подсветка сообщения при переходе по закрепу */
@keyframes highlight-flash-animation {
  0% { background-color: rgba(124, 108, 255, 0.25); }
  100% { background-color: transparent; }
}
.message-container.highlight-flash {
  animation: highlight-flash-animation 2s ease-out;
}
</style>
