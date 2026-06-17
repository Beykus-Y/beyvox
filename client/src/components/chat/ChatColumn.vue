<template>
  <div class="chat-column">
    <!-- Крошки хлебных навигаций (Breadcrumbs) -->
    <ChatBreadcrumb
      :serverName="serverName"
      :guildName="guildName"
      :channelName="channel ? channel.name : 'general'"
      :activeChannelId="channel ? channel.id : null"
      :channels="channels"
      @select-channel="$emit('select-channel', $event)"
      @focus-servers="$emit('focus-servers')"
      @focus-guilds="$emit('focus-guilds')"
    />

    <template v-if="channel">
      <!-- Хедер текущего канала -->
      <ChannelHeader
        :channelName="channel.name"
        :description="channel.description || 'Основной текстовый канал'"
        :showInfoColumn="showInfoColumn"
        @toggle-info="$emit('toggle-info')"
        @search="setSearchQuery"
        @pins="togglePinsPopover"
      />

      <!-- Поповер закреплённых сообщений -->
      <div v-if="pinsPopoverVisible" class="pins-popover" v-click-outside="closePinsPopover">
        <div class="pins-header">
          <span class="pins-title">Закреплённые сообщения</span>
          <button class="close-pins-btn" @click="pinsPopoverVisible = false">✕</button>
        </div>
        <div class="pins-list">
          <div v-if="pinnedMessages.length === 0" class="empty-pins">
            Нет закреплённых сообщений в этом канале.
          </div>
          <div v-else v-for="pin in pinnedMessages" :key="pin.id" class="pin-item">
            <div class="pin-meta">
              <span class="pin-author">{{ pin.author_username }}</span>
              <span class="pin-time">{{ formatTime(pin.created_at) }}</span>
            </div>
            <p class="pin-text">{{ pin.content }}</p>
            <div class="pin-actions">
              <button class="pin-action-btn jump" @click="jumpToMessage(pin.id)">Перейти</button>
              <button class="pin-action-btn unpin" @click="unpinMessage(pin.id)">Открепить</button>
            </div>
          </div>
        </div>
      </div>

      <!-- Список сообщений -->
      <MessageList
        :messages="filteredMessages"
        :userId="userId"
        :username="username"
        :members="members"
        :loading="loading"
        :replyTo="replyTo"
        :editingMessageId="editingMessageId"
        @load-more="$emit('load-more-messages')"
        @toggle-reaction="handleToggleReaction"
        @reply="setReplyMessage"
        @add-reaction-click="openEmojiPicker"
        @edit-message="handleEditMessage"
        @delete-message="handleDeleteMessage"
        @cancel-edit="cancelEdit"
        @contextmenu="openMessageContextMenu"
      />

      <!-- Панель ввода сообщения -->
      <MessageComposer
        :channelName="channel.name"
        :replyTo="replyTo"
        :members="members"
        :username="username"
        @send="handleSend"
        @cancel-reply="cancelReply"
      />
    </template>

    <!-- Заглушка, если текстовый канал не выбран -->
    <div v-else class="no-channel-selected">
      <div class="no-channel-hint">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.35">
          <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z"/>
        </svg>
        <h3>Добро пожаловать в мессенджер</h3>
        <p>Выберите канал или гильдию для начала общения</p>
      </div>
    </div>

    <!-- Всплывающий пикер эмодзи (привязанный к координатам) -->
    <Teleport to="body">
      <div v-if="picker.visible" class="floating-picker-overlay" @click.self="closeEmojiPicker" @contextmenu.prevent>
        <div
          class="floating-emoji-picker"
          :style="{ top: picker.y + 'px', left: picker.x + 'px' }"
          v-click-outside="closeEmojiPicker"
        >
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
      </div>
    </Teleport>

    <!-- Контекстное меню сообщения -->
    <Teleport to="body">
      <div v-if="msgMenu.visible" class="ctx-overlay" @mousedown.self="closeMsgMenu" @contextmenu.prevent>
        <div class="ctx-menu" :style="{ top: msgMenu.y + 'px', left: msgMenu.x + 'px' }">
          <button class="ctx-item" @click="triggerMsgReply">Ответить</button>
          <button class="ctx-item" @click="triggerMsgCopy">Копировать текст</button>
          <button class="ctx-item" @click="triggerMsgPin">
            {{ isMsgPinned(msgMenu.msg) ? 'Открепить' : 'Закрепить' }}
          </button>
          <template v-if="msgMenu.msg && msgMenu.msg.author_id === userId">
            <div class="ctx-divider" />
            <button class="ctx-item" @click="triggerMsgEdit">Редактировать</button>
            <button class="ctx-item danger" @click="triggerMsgDelete">Удалить</button>
          </template>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import ChatBreadcrumb from './ChatBreadcrumb.vue'
import ChannelHeader from './ChannelHeader.vue'
import MessageList from './MessageList.vue'
import MessageComposer from './MessageComposer.vue'
import type { Channel, Message, Member } from '../../stores/guild'

const props = defineProps<{
  serverName: string
  guildName: string
  channel: Channel | null
  channels: Channel[]
  messages: Message[]
  userId: string
  username: string
  members: Member[]
  loading: boolean
  showInfoColumn: boolean
}>()

const emit = defineEmits([
  'send-message',
  'toggle-info',
  'load-more-messages',
  'toggle-reaction',
  'select-channel',
  'focus-servers',
  'focus-guilds',
  'edit-message',
  'delete-message'
])

const replyTo = ref<Message | null>(null)
const searchQuery = ref('')

const picker = reactive({
  visible: false,
  x: 0,
  y: 0,
  messageId: ''
})

// Состояние и логика закреплённых сообщений (Pins)
const pinsPopoverVisible = ref(false)
const pinnedIds = ref<string[]>([])

watch(() => props.channel?.id, (newChanId) => {
  if (newChanId) {
    const saved = localStorage.getItem(`beyvox_pins_${newChanId}`)
    pinnedIds.value = saved ? JSON.parse(saved) : []
  } else {
    pinnedIds.value = []
  }
  pinsPopoverVisible.value = false
}, { immediate: true })

const pinnedMessages = computed(() => {
  return props.messages.filter(m => pinnedIds.value.includes(m.id))
})

function togglePinsPopover() {
  pinsPopoverVisible.value = !pinsPopoverVisible.value
}

function closePinsPopover() {
  pinsPopoverVisible.value = false
}

function pinMessage(messageId: string) {
  if (!props.channel?.id) return
  if (!pinnedIds.value.includes(messageId)) {
    pinnedIds.value.push(messageId)
    localStorage.setItem(`beyvox_pins_${props.channel.id}`, JSON.stringify(pinnedIds.value))
  }
}

function unpinMessage(messageId: string) {
  if (!props.channel?.id) return
  pinnedIds.value = pinnedIds.value.filter(id => id !== messageId)
  localStorage.setItem(`beyvox_pins_${props.channel.id}`, JSON.stringify(pinnedIds.value))
}

function isMsgPinned(msg: Message | null): boolean {
  if (!msg) return false
  return pinnedIds.value.includes(msg.id)
}

function jumpToMessage(messageId: string) {
  const el = document.getElementById(`msg-${messageId}`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    el.classList.add('highlight-flash')
    setTimeout(() => {
      el.classList.remove('highlight-flash')
    }, 2000)
  }
  pinsPopoverVisible.value = false
}

function formatTime(iso: string): string {
  const date = new Date(iso)
  return date.toLocaleTimeString('ru', { hour: '2-digit', minute: '2-digit' })
}

// Состояние и логика контекстного меню сообщения
const msgMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  msg: null as Message | null
})

function openMessageContextMenu(e: MouseEvent, msg: Message) {
  msgMenu.msg = msg
  msgMenu.x = Math.min(e.clientX, window.innerWidth - 170)
  msgMenu.y = Math.min(e.clientY, window.innerHeight - 200)
  msgMenu.visible = true
}

function closeMsgMenu() {
  msgMenu.visible = false
}

function triggerMsgReply() {
  if (msgMenu.msg) {
    setReplyMessage(msgMenu.msg)
  }
  closeMsgMenu()
}

async function triggerMsgCopy() {
  if (msgMenu.msg) {
    try {
      await navigator.clipboard.writeText(msgMenu.msg.content)
    } catch (err) {
      console.error('Ошибка копирования:', err)
    }
  }
  closeMsgMenu()
}

function triggerMsgPin() {
  if (msgMenu.msg) {
    if (isMsgPinned(msgMenu.msg)) {
      unpinMessage(msgMenu.msg.id)
    } else {
      pinMessage(msgMenu.msg.id)
    }
  }
  closeMsgMenu()
}

const editingMessageId = ref<string | null>(null)

function triggerMsgEdit() {
  if (msgMenu.msg) {
    editingMessageId.value = msgMenu.msg.id
  }
  closeMsgMenu()
}

function triggerMsgDelete() {
  if (msgMenu.msg) {
    emit('delete-message', msgMenu.msg.id)
  }
  closeMsgMenu()
}

function handleEditMessage(messageId: string, content: string) {
  emit('edit-message', messageId, content)
  editingMessageId.value = null
}

function handleDeleteMessage(messageId: string) {
  emit('delete-message', messageId)
}

function cancelEdit() {
  editingMessageId.value = null
}

const commonEmojis = ['👍', '👎', '❤️', '😂', '😮', '😢', '😡', '🔥', '✅', '❌', '👀', '🎉', '🤔', '💯', '🙏', '⭐']

const filteredMessages = computed(() => {
  if (!searchQuery.value) return props.messages
  const q = searchQuery.value.toLowerCase().trim()
  return props.messages.filter(m => m.content.toLowerCase().includes(q))
})

function setSearchQuery(query: string) {
  searchQuery.value = query
}

function setReplyMessage(msg: Message) {
  replyTo.value = msg
}

function cancelReply() {
  replyTo.value = null
}

function handleSend(content: string) {
  emit('send-message', {
    content,
    replyTo: replyTo.value ? replyTo.value.id : null
  })
  replyTo.value = null
}

function handleToggleReaction(messageId: string, emoji: string, active: boolean) {
  emit('toggle-reaction', messageId, emoji, active)
}

function openEmojiPicker(e: MouseEvent, messageId: string) {
  picker.messageId = messageId
  picker.x = Math.min(e.clientX - 90, window.innerWidth - 190)
  picker.y = Math.min(e.clientY - 100, window.innerHeight - 150)
  picker.visible = true
}

function closeEmojiPicker() {
  picker.visible = false
}

function pickEmoji(emoji: string) {
  if (picker.messageId) {
    // Проверяем, реагировал ли текущий пользователь на это сообщение данным эмодзи
    const msg = props.messages.find(m => m.id === picker.messageId)
    const react = msg?.reactions?.find(r => r.emoji === emoji)
    const alreadyReacted = react ? react.me : false
    
    emit('toggle-reaction', picker.messageId, emoji, alreadyReacted)
  }
  closeEmojiPicker()
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
.chat-column {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  height: 100vh;
  background: var(--bg-app);
  position: relative;
}

.no-channel-selected {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-secondary);
  user-select: none;
}

.no-channel-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 24px;
}
.no-channel-hint h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
}
.no-channel-hint p {
  font-size: 13px;
  color: var(--text-muted);
}

/* Плавающий пикер */
.floating-picker-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
}

.floating-emoji-picker {
  position: fixed;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  padding: 6px;
  width: 176px;
  z-index: 10000;
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

/* Поповер закрепов */
.pins-popover {
  position: absolute;
  top: 62px;
  right: 16px;
  width: 340px;
  max-height: 480px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  z-index: 100;
  overflow: hidden;
  animation: slide-down 0.15s ease-out;
}

@keyframes slide-down {
  from { transform: translateY(-8px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.pins-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.pins-title {
  font-weight: 700;
  font-size: 14px;
  color: var(--text-primary);
}

.close-pins-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 11px;
}
.close-pins-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.pins-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-pins {
  padding: 32px 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}

.pin-item {
  background: var(--bg-app);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: border-color 0.15s;
  text-align: left;
}
.pin-item:hover {
  border-color: var(--accent);
}

.pin-meta {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.pin-author {
  font-weight: 700;
  color: var(--accent);
}

.pin-time {
  color: var(--text-muted);
}

.pin-text {
  font-size: 13px;
  line-height: 1.4;
  color: var(--text-primary);
  word-break: break-word;
  white-space: pre-wrap;
}

.pin-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
  justify-content: flex-end;
}

.pin-action-btn {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
}
.pin-action-btn.jump {
  background: var(--bg-hover);
  border: 1px solid var(--border);
  color: var(--text-primary);
}
.pin-action-btn.jump:hover {
  background: var(--bg-active);
}
.pin-action-btn.unpin {
  background: rgba(239, 68, 68, 0.1);
  color: var(--danger);
  border: 1px solid rgba(239, 68, 68, 0.2);
}
.pin-action-btn.unpin:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: var(--danger);
}
</style>
