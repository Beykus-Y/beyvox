<template>
  <div class="chat-area">
    <div class="chat-header">
      <span class="ch-prefix">#</span>
      <span class="ch-name">{{ channelName }}</span>
    </div>

    <div class="messages" ref="messagesEl" @scroll="onScroll">
      <div v-if="loading" class="state-msg">Загрузка...</div>
      <div v-else-if="messages.length === 0" class="state-msg">Нет сообщений. Напиши первым!</div>
      <template v-else>
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="message"
          :class="{ own: msg.author_id === userId, mentioned: isMentioned(msg) }"
        >
          <div class="avatar">{{ msg.author_username[0]?.toUpperCase() }}</div>
          <div class="msg-body">
            <div class="msg-header">
              <span class="msg-author">{{ msg.author_username }}</span>
              <span class="msg-time">{{ formatTime(msg.created_at) }}</span>
              <span v-if="msg.edited_at" class="msg-edited">(ред.)</span>
            </div>
            <div v-if="msg.reply_to" class="msg-reply">
              <span class="reply-author">↩ {{ replySource(msg.reply_to)?.author_username || '...' }}</span>
              <span class="reply-text">{{ replySource(msg.reply_to)?.content?.slice(0, 80) || 'сообщение' }}</span>
            </div>
            <div class="msg-content" v-html="renderMarkdown(msg.content)" />

            <!-- Реакции -->
            <div v-if="msg.reactions.length || hoveredMsgId === msg.id" class="msg-reactions">
              <button
                v-for="r in msg.reactions"
                :key="r.emoji"
                class="reaction-btn"
                :class="{ active: r.me }"
                @click="$emit('toggle-reaction', msg.id, r.emoji, r.me)"
                :title="r.emoji"
              >
                {{ r.emoji }} <span class="reaction-count">{{ r.count }}</span>
              </button>
              <button
                class="reaction-add-btn"
                @click.stop="togglePicker(msg.id)"
                title="Добавить реакцию"
              >+</button>
            </div>
          </div>

          <!-- Кнопки при наведении -->
          <div class="msg-actions" @mouseenter="hoveredMsgId = msg.id" @mouseleave="hoveredMsgId = null">
            <button class="msg-action-btn" @click.stop="togglePicker(msg.id)" title="Реакция">😊</button>
            <button class="msg-action-btn" @click="replyTo = msg.id" title="Ответить">↩</button>
          </div>
        </div>
      </template>
    </div>

    <!-- Пикер эмодзи -->
    <div v-if="pickerMsgId" class="emoji-picker" @click.stop>
      <button
        v-for="emoji in commonEmojis"
        :key="emoji"
        class="emoji-btn"
        @click="pickEmoji(emoji)"
      >{{ emoji }}</button>
    </div>

    <div class="chat-input-area">
      <div v-if="replyTo" class="reply-preview">
        <span>↩ Ответ на сообщение</span>
        <button class="icon-btn" @click="replyTo = null">✕</button>
      </div>
      <div class="input-wrapper" :class="{ 'mention-active': mentionQuery !== null }">
        <!-- @mention autocomplete -->
        <div v-if="mentionQuery !== null && filteredMembers.length" class="mention-dropdown">
          <div
            v-for="(m, i) in filteredMembers"
            :key="m.user_id"
            class="mention-item"
            :class="{ selected: i === mentionIndex }"
            @mousedown.prevent="insertMention(m.username)"
          >
            <div class="mention-avatar">{{ (m.nickname || m.username)[0]?.toUpperCase() }}</div>
            <div class="mention-info">
              <span class="mention-name">{{ m.nickname || m.username }}</span>
              <span class="mention-tag">@{{ m.username }}</span>
            </div>
          </div>
        </div>

        <textarea
          v-model="inputText"
          :placeholder="`Написать в #${channelName}`"
          rows="1"
          @keydown="onKeydown"
          @input="onInput"
          ref="inputEl"
        />
        <button class="send-btn" @click="send" :disabled="!inputText.trim()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed, onMounted, onUnmounted } from 'vue'
import type { Message, Member } from '../../stores/guild'

const props = defineProps<{
  channelName: string
  messages: Message[]
  userId: string
  username: string
  loading: boolean
  members: Member[]
}>()

const emit = defineEmits(['send', 'load-more', 'toggle-reaction'])

const inputText = ref('')
const replyTo = ref<string | null>(null)
const messagesEl = ref<HTMLElement | null>(null)
const inputEl = ref<HTMLTextAreaElement | null>(null)
const hoveredMsgId = ref<string | null>(null)
const pickerMsgId = ref<string | null>(null)

// @mention state
const mentionQuery = ref<string | null>(null)
const mentionIndex = ref(0)
let mentionStart = -1

const commonEmojis = ['👍', '👎', '❤️', '😂', '😮', '😢', '😡', '🔥', '✅', '❌', '👀', '🎉', '🤔', '💯', '🙏', '⭐']

let prevScrollHeight = 0
let isLoadMore = false

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

watch(() => props.messages.length, (newLen, oldLen) => {
  nextTick(() => {
    if (!messagesEl.value) return
    if (isLoadMore && newLen > oldLen) {
      messagesEl.value.scrollTop = messagesEl.value.scrollHeight - prevScrollHeight
      isLoadMore = false
    } else {
      messagesEl.value.scrollTop = messagesEl.value.scrollHeight
    }
  })
})

function send() {
  const content = inputText.value.trim()
  if (!content) return
  emit('send', { content, replyTo: replyTo.value })
  inputText.value = ''
  replyTo.value = null
  mentionQuery.value = null
  nextTick(() => autoResize())
}

function onKeydown(e: KeyboardEvent) {
  // Если открыт mention dropdown
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

  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    send()
  } else if (e.key === 'Enter' && e.shiftKey) {
    // Разрешаем перенос строки
  }
}

function onInput() {
  autoResize()
  updateMentionQuery()
}

function updateMentionQuery() {
  const el = inputEl.value
  if (!el) return

  const pos = el.selectionStart ?? 0
  const text = inputText.value

  // Ищем @ перед курсором
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
  const el = inputEl.value
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
  const el = inputEl.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = Math.min(el.scrollHeight, 200) + 'px'
}

function onScroll() {
  if (messagesEl.value?.scrollTop === 0) {
    prevScrollHeight = messagesEl.value.scrollHeight
    isLoadMore = true
    emit('load-more')
  }
}

function togglePicker(msgId: string) {
  pickerMsgId.value = pickerMsgId.value === msgId ? null : msgId
}

function pickEmoji(emoji: string) {
  if (!pickerMsgId.value) return
  emit('toggle-reaction', pickerMsgId.value, emoji, false)
  pickerMsgId.value = null
}

function closePicker(_e: MouseEvent) {
  pickerMsgId.value = null
}

onMounted(() => document.addEventListener('click', closePicker))
onUnmounted(() => document.removeEventListener('click', closePicker))

function replySource(id: string) {
  return props.messages.find(m => m.id === id)
}

function isMentioned(msg: Message): boolean {
  return msg.mention_user_ids?.includes(props.userId) ?? false
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString('ru', { hour: '2-digit', minute: '2-digit' })
}

function renderMarkdown(text: string): string {
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

  const memberUsernames = props.members.map(m => m.username)

  return escaped
    .replace(/```([\s\S]+?)```/g, '<pre><code>$1</code></pre>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/@([a-zA-Z0-9_]{1,50})/g, (match, name) => {
      const isMember = memberUsernames.includes(name)
      const isMe = name === props.username
      return isMember
        ? `<span class="mention${isMe ? ' mention-me' : ''}" >@${name}</span>`
        : match
    })
    .replace(/\n/g, '<br>')
}
</script>

<style scoped>
.chat-area { display: flex; flex-direction: column; flex: 1; min-width: 0; position: relative; }

.chat-header {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  gap: 6px;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  flex-shrink: 0;
}
.ch-prefix { color: var(--text3); font-size: 18px; }

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.state-msg { color: var(--text2); text-align: center; padding: 40px; }

.message {
  display: flex;
  gap: 12px;
  padding: 4px 8px;
  border-radius: 6px;
  position: relative;
}
.message:hover { background: var(--bg-hover); }
.message.mentioned { background: rgba(88, 101, 242, 0.07); border-left: 2px solid var(--accent); padding-left: 6px; }
.message.mentioned:hover { background: rgba(88, 101, 242, 0.12); }

.msg-body { flex: 1; min-width: 0; }
.msg-header { display: flex; align-items: baseline; gap: 8px; margin-bottom: 2px; }
.msg-author { font-weight: 600; font-size: 13px; }
.message.own .msg-author { color: var(--accent); }
.msg-time { font-size: 11px; color: var(--text3); }
.msg-edited { font-size: 11px; color: var(--text3); }
.msg-reply { font-size: 12px; color: var(--text3); margin-bottom: 2px; display: flex; gap: 6px; align-items: baseline; overflow: hidden; }
.reply-author { font-weight: 600; flex-shrink: 0; }
.reply-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.msg-content {
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
}
.msg-content :deep(code) {
  background: var(--bg-darkest);
  padding: 1px 5px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
}
.msg-content :deep(pre) {
  background: var(--bg-darkest);
  padding: 10px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 6px 0;
}
.msg-content :deep(pre code) { background: none; padding: 0; }
.msg-content :deep(.mention) {
  color: var(--accent);
  background: rgba(88, 101, 242, 0.15);
  border-radius: 4px;
  padding: 0 3px;
  font-weight: 600;
  cursor: default;
}
.msg-content :deep(.mention-me) {
  background: rgba(88, 101, 242, 0.3);
}

/* Реакции */
.msg-reactions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 4px;
}
.reaction-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: 10px;
  background: var(--bg-light);
  border: 1px solid var(--border);
  font-size: 13px;
  cursor: pointer;
  color: var(--text2);
  transition: all 0.1s;
}
.reaction-btn:hover { border-color: var(--accent); color: var(--text); }
.reaction-btn.active { border-color: var(--accent); background: rgba(88, 101, 242, 0.15); color: var(--accent); }
.reaction-count { font-size: 11px; font-weight: 600; }
.reaction-add-btn {
  padding: 2px 7px;
  border-radius: 10px;
  background: transparent;
  border: 1px dashed var(--border);
  font-size: 13px;
  color: var(--text3);
  cursor: pointer;
}
.reaction-add-btn:hover { border-color: var(--accent); color: var(--accent); background: rgba(88, 101, 242, 0.1); }

/* Кнопки при наведении */
.msg-actions {
  position: absolute;
  top: 4px;
  right: 8px;
  display: none;
  background: var(--bg-dark);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 2px 4px;
  gap: 2px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
}
.message:hover .msg-actions { display: flex; }
.msg-action-btn {
  width: 26px; height: 26px;
  border-radius: 5px;
  background: transparent;
  color: var(--text2);
  font-size: 14px;
  display: flex; align-items: center; justify-content: center;
}
.msg-action-btn:hover { background: var(--bg-hover); color: var(--text); }

/* Emoji picker */
.emoji-picker {
  position: absolute;
  bottom: 90px;
  right: 16px;
  background: var(--bg-dark);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 10px;
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 4px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.4);
  z-index: 100;
}
.emoji-btn {
  width: 34px; height: 34px;
  border-radius: 6px;
  background: transparent;
  font-size: 18px;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
  transition: background 0.1s;
}
.emoji-btn:hover { background: var(--bg-hover); }

/* Ввод */
.chat-input-area {
  padding: 8px 16px 16px;
  flex-shrink: 0;
}
.reply-preview {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: var(--bg-light);
  border-radius: 6px 6px 0 0;
  border: 1px solid var(--border);
  border-bottom: none;
  font-size: 12px;
  color: var(--text2);
}
.input-wrapper {
  position: relative;
  display: flex;
  align-items: flex-end;
  gap: 8px;
  background: var(--bg-light);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 12px;
}
textarea {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  color: var(--text);
  font-size: 14px;
  line-height: 1.5;
  max-height: 200px;
  padding: 0;
}
.send-btn {
  background: var(--accent);
  color: white;
  border-radius: 6px;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.send-btn:hover { background: var(--accent-hover); }
.send-btn:disabled { opacity: 0.4; cursor: not-allowed; }

/* @mention dropdown */
.mention-dropdown {
  position: absolute;
  bottom: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--bg-dark);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 -8px 24px rgba(0,0,0,0.3);
  z-index: 50;
  max-height: 240px;
  overflow-y: auto;
}
.mention-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.1s;
}
.mention-item:hover, .mention-item.selected { background: var(--bg-hover); }
.mention-avatar {
  width: 28px; height: 28px;
  border-radius: 50%;
  background: var(--bg-light);
  border: 1px solid var(--border);
  display: flex; align-items: center; justify-content: center;
  font-size: 12px; font-weight: 700;
  flex-shrink: 0;
}
.mention-info { display: flex; flex-direction: column; gap: 1px; }
.mention-name { font-size: 13px; font-weight: 600; color: var(--text); }
.mention-tag { font-size: 11px; color: var(--text3); }
</style>
