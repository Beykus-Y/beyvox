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
          :class="{ own: msg.author_id === userId }"
        >
          <div class="avatar">{{ msg.author_username[0]?.toUpperCase() }}</div>
          <div class="msg-body">
            <div class="msg-header">
              <span class="msg-author">{{ msg.author_username }}</span>
              <span class="msg-time">{{ formatTime(msg.created_at) }}</span>
              <span v-if="msg.edited_at" class="msg-edited">(ред.)</span>
            </div>
            <div v-if="msg.reply_to" class="msg-reply">↩ ответ на сообщение</div>
            <div class="msg-content" v-html="renderMarkdown(msg.content)" />
          </div>
        </div>
      </template>
    </div>

    <div class="chat-input-area">
      <div v-if="replyTo" class="reply-preview">
        <span>↩ Ответ на сообщение</span>
        <button class="icon-btn" @click="replyTo = null">✕</button>
      </div>
      <div class="input-wrapper">
        <textarea
          v-model="inputText"
          :placeholder="`Написать в #${channelName}`"
          rows="1"
          @keydown.enter.exact.prevent="send"
          @keydown.enter.shift.exact="inputText += '\n'"
          @input="autoResize"
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
import { ref, watch, nextTick } from 'vue'
import type { Message } from '../../stores/guild'

const props = defineProps<{
  channelName: string
  messages: Message[]
  userId: string
  loading: boolean
}>()

const emit = defineEmits(['send', 'load-more'])

const inputText = ref('')
const replyTo = ref<string | null>(null)
const messagesEl = ref<HTMLElement | null>(null)
const inputEl = ref<HTMLTextAreaElement | null>(null)

let prevScrollHeight = 0
let isLoadMore = false

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
  nextTick(() => autoResize())
}

function onScroll() {
  if (messagesEl.value?.scrollTop === 0) {
    prevScrollHeight = messagesEl.value.scrollHeight
    isLoadMore = true
    emit('load-more')
  }
}

function autoResize() {
  const el = inputEl.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = Math.min(el.scrollHeight, 200) + 'px'
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString('ru', { hour: '2-digit', minute: '2-digit' })
}

function renderMarkdown(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/```([\s\S]+?)```/g, '<pre><code>$1</code></pre>')
    .replace(/\n/g, '<br>')
}
</script>

<style scoped>
.chat-area { display: flex; flex-direction: column; flex: 1; min-width: 0; }

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
  padding: 4px 0;
  border-radius: 6px;
}
.message:hover { background: var(--bg-hover); margin: 0 -8px; padding: 4px 8px; }

.msg-body { flex: 1; min-width: 0; }
.msg-header { display: flex; align-items: baseline; gap: 8px; margin-bottom: 2px; }
.msg-author { font-weight: 600; font-size: 13px; }
.message.own .msg-author { color: var(--accent); }
.msg-time { font-size: 11px; color: var(--text3); }
.msg-edited { font-size: 11px; color: var(--text3); }
.msg-reply { font-size: 12px; color: var(--text3); margin-bottom: 2px; }
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
</style>
