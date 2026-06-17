<template>
  <div class="message-composer">
    <!-- Превью ответа -->
    <div v-if="replyTo" class="reply-preview-bar">
      <div class="reply-info">
        <span class="reply-label">Ответ пользователю</span>
        <span class="reply-author">@{{ replyTo.author_username }}</span>
      </div>
      <button class="cancel-reply-btn" @click="$emit('cancel-reply')" title="Отменить ответ">✕</button>
    </div>

    <div class="composer-input-row" v-click-outside="closeAllPopups">
      <!-- Добавить вложение -->
      <button class="attach-btn" title="Прикрепить файл" @click="$emit('attach')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </button>

      <!-- Контейнер ввода -->
      <div class="input-container">
        <!-- Меню упоминаний (@mention dropdown) -->
        <div v-if="mentionQuery !== null && filteredMembers.length" class="mention-dropdown">
          <div
            v-for="(m, i) in filteredMembers"
            :key="m.user_id"
            class="mention-item"
            :class="{ selected: i === mentionIndex }"
            @mousedown.prevent="insertMention(m.username)"
          >
            <div class="mention-avatar">
              <span>{{ (m.nickname || m.username)[0]?.toUpperCase() }}</span>
            </div>
            <div class="mention-info">
              <span class="mention-name">{{ m.nickname || m.username }}</span>
              <span class="mention-tag">@{{ m.username }}</span>
            </div>
          </div>
        </div>

        <textarea
          v-model="inputText"
          :placeholder="`Написать сообщение в #${channelName}`"
          ref="textareaEl"
          rows="1"
          @keydown="handleKeydown"
          @input="handleInput"
        />
      </div>

      <!-- Действия ввода -->
      <div class="composer-actions">
        <!-- Кнопка Эмодзи -->
        <button class="action-btn emoji-trigger" @click.stop="toggleEmojiPicker" title="Добавить эмодзи">
          😊
        </button>

        <!-- Пикер эмодзи -->
        <div v-if="emojiPickerOpen" class="emoji-picker-popover" @click.stop>
          <div class="emoji-grid">
            <button
              v-for="emoji in commonEmojis"
              :key="emoji"
              class="emoji-btn"
              @click="pickEmoji(emoji)"
            >
              {{ emoji }}
            </button>
          </div>
        </div>

        <!-- Кнопка Отправить -->
        <button
          class="send-btn"
          :disabled="!inputText.trim()"
          @click="triggerSend"
          title="Отправить сообщение"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import type { Message, Member } from '../../stores/guild'

const props = defineProps<{
  channelName: string
  replyTo: Message | null
  members: Member[]
  username: string
}>()

const emit = defineEmits(['send', 'cancel-reply', 'attach'])

const inputText = ref('')
const textareaEl = ref<HTMLTextAreaElement | null>(null)
const emojiPickerOpen = ref(false)

// @mention autocomplete state
const mentionQuery = ref<string | null>(null)
const mentionIndex = ref(0)
let mentionStart = -1

const commonEmojis = ['👍', '👎', '❤️', '😂', '😮', '😢', '😡', '🔥', '✅', '❌', '👀', '🎉', '🤔', '💯', '🙏', '⭐']

const filteredMembers = computed(() => {
  if (mentionQuery.value === null) return []
  const q = mentionQuery.value.toLowerCase()
  return props.members
    .filter(m => {
      const name = (m.nickname || m.username).toLowerCase()
      return name.startsWith(q) || m.username.toLowerCase().startsWith(q)
    })
    .slice(0, 8)
})

function handleInput() {
  autoResize()
  updateMentionQuery()
}

function handleKeydown(e: KeyboardEvent) {
  // Навигация по dropdown-упоминаниям
  if (mentionQuery.value !== null && filteredMembers.value.length) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      mentionIndex.value = Math.min(mentionIndex.value + 1, filteredMembers.value.length - 1)
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      mentionIndex.value = Math.max(mentionIndex.value - 1, 0)
      return
    }
    if (e.key === 'Enter' || e.key === 'Tab') {
      e.preventDefault()
      const m = filteredMembers.value[mentionIndex.value]
      if (m) insertMention(m.username)
      return
    }
    if (e.key === 'Escape') {
      mentionQuery.value = null
      return
    }
  }

  // Отправка по Enter (без Shift)
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    triggerSend()
  }
}

function updateMentionQuery() {
  const el = textareaEl.value
  if (!el) return

  const pos = el.selectionStart ?? 0
  const text = inputText.value

  // Ищем символ '@' непосредственно перед курсором
  let atIdx = -1
  for (let i = pos - 1; i >= 0; i--) {
    if (text[i] === '@') {
      atIdx = i
      break
    }
    if (text[i] === ' ' || text[i] === '\n') break
  }

  if (atIdx >= 0) {
    const q = text.slice(atIdx + 1, pos)
    if (/^[a-zA-Z0-9_]*$/.test(q)) {
      mentionQuery.value = q
      mentionStart = atIdx
      mentionIndex.value = 0
      return
    }
  }

  mentionQuery.value = null
}

function insertMention(username: string) {
  const el = textareaEl.value
  if (!el) return

  const pos = el.selectionStart ?? 0
  const text = inputText.value
  const before = text.slice(0, mentionStart)
  const after = text.slice(pos)
  
  inputText.value = before + '@' + username + ' ' + after
  mentionQuery.value = null

  nextTick(() => {
    const newPos = mentionStart + username.length + 2
    el.setSelectionRange(newPos, newPos)
    el.focus()
    autoResize()
  })
}

function autoResize() {
  const el = textareaEl.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = Math.min(el.scrollHeight, 200) + 'px'
}

function toggleEmojiPicker() {
  emojiPickerOpen.value = !emojiPickerOpen.value
}

function pickEmoji(emoji: string) {
  const el = textareaEl.value
  if (!el) return
  const pos = el.selectionStart ?? 0
  const before = inputText.value.slice(0, pos)
  const after = inputText.value.slice(pos)
  inputText.value = before + emoji + after
  
  nextTick(() => {
    const newPos = pos + emoji.length
    el.setSelectionRange(newPos, newPos)
    el.focus()
    autoResize()
  })
  
  emojiPickerOpen.value = false
}

function triggerSend() {
  const content = inputText.value.trim()
  if (!content) return
  emit('send', content)
  inputText.value = ''
  mentionQuery.value = null
  nextTick(() => autoResize())
}

function closeAllPopups() {
  emojiPickerOpen.value = false
  mentionQuery.value = null
}

// Простая директива клика вне элемента
const vClickOutside = {
  mounted(el: any, binding: any) {
    el.clickOutsideEvent = (event: Event) => {
      if (!(el === event.target || el.contains(event.target))) {
        binding.value(event)
      }
    }
    document.addEventListener('click', el.clickOutsideEvent)
  },
  unmounted(el: any) {
    document.removeEventListener('click', el.clickOutsideEvent)
  }
}
</script>

<style scoped>
.message-composer {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 12px;
  margin: 0 16px 16px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
  position: relative;
}

.reply-preview-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 8px 0;
  background: transparent;
  border-bottom: 1px solid var(--border);
  margin-bottom: 8px;
}

.reply-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
}

.reply-label {
  color: var(--text-muted);
}

.reply-author {
  color: var(--accent);
  font-weight: 600;
}

.cancel-reply-btn {
  color: var(--text-muted);
  font-size: 10px;
}
.cancel-reply-btn:hover {
  color: var(--text-primary);
}

.composer-input-row {
  display: flex;
  align-items: flex-end;
  gap: 10px;
  background: transparent;
  border: none;
  padding: 0;
  position: relative;
  width: 100%;
}

.attach-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.attach-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.input-container {
  flex: 1;
  min-width: 0;
  position: relative;
}

textarea {
  width: 100%;
  background: transparent;
  border: none !important;
  box-shadow: none !important;
  padding: 4px 0;
  resize: none;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-primary);
  max-height: 160px;
  min-height: 20px;
}
textarea::placeholder {
  color: var(--text-muted);
}

.composer-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  position: relative;
}

.action-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 15px;
}
.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.send-btn {
  background: var(--accent-grad);
  color: #fff;
  width: 34px;
  height: 34px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  box-shadow: none;
}
.send-btn:hover:not(:disabled) {
  box-shadow: 0 2px 8px rgba(124, 108, 255, 0.4);
  opacity: 0.95;
}
.send-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  background: var(--border);
  color: var(--text-muted);
  box-shadow: none;
}

/* Эмодзи пикер */
.emoji-picker-popover {
  position: absolute;
  bottom: 36px;
  right: 0;
  z-index: 100;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  padding: 8px;
  width: 176px;
}

.emoji-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
}

.emoji-btn {
  width: 36px;
  height: 36px;
  font-size: 18px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.emoji-btn:hover {
  background: var(--bg-hover);
}

/* Дропдаун автокомплита */
.mention-dropdown {
  position: absolute;
  bottom: calc(100% + 12px);
  left: 0;
  right: 0;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  box-shadow: 0 -8px 24px rgba(0, 0, 0, 0.4);
  z-index: 110;
  max-height: 200px;
  overflow-y: auto;
  padding: 4px;
}

.mention-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}
.mention-item:hover, .mention-item.selected {
  background: var(--bg-hover);
}

.mention-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--bg-active);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  color: var(--accent);
}

.mention-info {
  display: flex;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
}

.mention-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mention-tag {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
