<template>
  <div class="app-layout">
    <!-- Уровень 1: инстансы beyvox-server -->
    <ServerSidebar
      :servers="serversStore.servers"
      :active-url="serversStore.activeUrl"
      @select-server="switchServer"
      @add-server="openAddServer"
      @remove-server="serversStore.removeServer"
      @open-settings="openSettings"
    />

    <!-- Уровень 2: список серверов (гильдий) внутри выбранного инстанса -->
    <GuildSidebar
      v-if="serversStore.activeUrl"
      :server-name="serversStore.activeServer?.name || ''"
      :guilds="guild.guilds"
      :active-guild-id="guild.activeGuildId"
      :collapsed="!guildSidebarOpen"
      @select-guild="selectGuild"
      @create-guild="openCreateGuild"
      @toggle="guildSidebarOpen = !guildSidebarOpen"
    />

    <!-- Если инстанс не выбран — заглушка вместо guild sidebar -->
    <div v-else class="no-server">
      <div class="no-server-hint">
        <div class="no-server-icon">
          <svg width="52" height="52" viewBox="0 0 24 24" fill="currentColor" opacity="0.35">
            <path d="M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z"/>
          </svg>
        </div>
        <h3>Добро пожаловать в BeyVox</h3>
        <p>Добавь BeyVox-сервер слева</p>
        <button class="connect-btn" @click="openAddServer">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
          Добавить сервер
        </button>
      </div>
    </div>

    <!-- Уровень 3: каналы выбранной гильдии -->
    <ChannelSidebar
      v-if="guild.activeGuildId"
      :guild-name="activeGuild?.name || ''"
      :channels="guild.channels"
      :active-channel-id="guild.activeChannelId"
      :active-voice-channel-id="voice.activeChannelId"
      :username="auth.username"
      :ws-status="wsStatusLabel"
      :is-muted="voice.isMuted"
      :is-deafened="voice.isDeafened"
      :voice-states="voice.voiceStates"
      :active-speakers="voice.activeSpeakers"
      :members="guild.members"
      :participant-volumes="voice.participantVolumes"
      :mentioned-channels="guild.mentionedChannels"
      :collapsed="!channelSidebarOpen"
      @select-channel="selectTextChannel"
      @join-voice="joinVoice"
      @toggle-mute="voice.toggleMute()"
      @toggle-deafen="voice.toggleDeafen()"
      @set-volume="voice.setParticipantVolume"
      @create-channel="openCreateChannel"
      @toggle="channelSidebarOpen = !channelSidebarOpen"
    />

    <!-- Нет выбранной гильдии — подсказка в основной области -->
    <div v-else-if="serversStore.activeUrl" class="no-guild-main">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
        <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z"/>
      </svg>
      <p>Выбери или создай сервер слева</p>
    </div>

    <!-- Основная область: чат -->
    <ChatArea
      v-if="guild.activeChannelId"
      :channel-name="activeChannel?.name || ''"
      :messages="guild.messages"
      :user-id="auth.userId"
      :username="auth.username"
      :loading="messagesLoading"
      :members="guild.members"
      @send="onSend"
      @load-more="loadMore"
      @toggle-reaction="onToggleReaction"
    />

    <div v-else-if="guild.activeGuildId" class="no-channel">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
        <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2z"/>
      </svg>
      <p>Выбери канал слева</p>
    </div>

    <!-- Модалка: добавить инстанс сервера -->
    <div v-if="showAddServer" class="modal-overlay" @click.self="closeAddServer">
      <div class="modal">
        <h3>Подключиться к серверу</h3>
        <p class="modal-hint">Адрес BeyVox-сервера</p>
        <input v-model="serverUrlInput" placeholder="http://localhost:8080" @keydown.enter="connectServer" autofocus />
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="closeAddServer">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading || !serverUrlInput.trim()" @click="connectServer">
            {{ serverLoading ? '...' : 'Подключиться' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Модалка: создать сервер (гильдию) -->
    <div v-if="showCreateGuild" class="modal-overlay" @click.self="showCreateGuild = false">
      <div class="modal">
        <h3>Создать сервер</h3>
        <div class="modal-field">
          <label>Название</label>
          <input v-model="newGuildName" placeholder="Мой сервер" @keydown.enter="createGuild" autofocus />
        </div>
        <div class="modal-field" v-if="serversStore.activeServer?.requiresOwnerToken">
          <label>Токен владельца</label>
          <input v-model="ownerToken" type="password" placeholder="из логов сервера при старте" />
        </div>
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showCreateGuild = false">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading || !newGuildName.trim()" @click="createGuild">
            {{ serverLoading ? '...' : 'Создать' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Инвайт-код -->
    <div v-if="showInvite" class="modal-overlay" @click.self="showInvite = false">
      <div class="modal">
        <h3>Войти по инвайту</h3>
        <input v-model="inviteCodeInput" placeholder="abc12345" @keydown.enter="joinByInvite" autofocus />
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showInvite = false">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading" @click="joinByInvite">
            {{ serverLoading ? '...' : 'Войти' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Создать канал -->
    <div v-if="showCreateChannel" class="modal-overlay" @click.self="showCreateChannel = false">
      <div class="modal">
        <h3>Создать {{ newChannelType === 'text' ? 'текстовый' : 'голосовой' }} канал</h3>
        <div class="modal-field">
          <label>Название</label>
          <input v-model="newChannelName" :placeholder="newChannelType === 'text' ? 'general' : 'Голос'" @keydown.enter="createChannel" autofocus />
        </div>
        <div v-if="newChannelType === 'voice'" class="modal-field">
          <label>Лимит участников (0 = без лимита)</label>
          <input v-model.number="newChannelLimit" type="number" min="0" max="99" placeholder="0" />
        </div>
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showCreateChannel = false">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading || !newChannelName.trim()" @click="createChannel">
            {{ serverLoading ? '...' : 'Создать' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Настройки -->
    <div v-if="showSettings" class="modal-overlay" @click.self="showSettings = false">
      <div class="modal settings-modal">
        <div class="settings-sidebar">
          <div
            v-for="tab in settingsTabs" :key="tab.id"
            class="settings-tab" :class="{ active: activeSettingsTab === tab.id }"
            @click="activeSettingsTab = tab.id"
          >{{ tab.label }}</div>
        </div>
        <div class="settings-content">
          <button class="settings-close" @click="showSettings = false">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
          </button>

          <template v-if="activeSettingsTab === 'account'">
            <div class="settings-row">
              <div class="settings-avatar">{{ auth.username[0]?.toUpperCase() }}</div>
              <div class="settings-userinfo">
                <div class="settings-username">{{ auth.username }}</div>
                <div class="settings-uuid">{{ auth.userId }}</div>
              </div>
            </div>
            <div class="settings-divider" />
            <button class="btn-danger" @click="logout">Выйти из аккаунта</button>
          </template>

          <template v-if="activeSettingsTab === 'audio'">
            <div class="settings-field">
              <label>Микрофон</label>
              <select v-model="voice.selectedInputId">
                <option value="" disabled>Выбери устройство...</option>
                <option v-for="d in inputDevices" :key="d.id" :value="d.id">{{ d.name }}</option>
              </select>
            </div>
            <div class="settings-field">
              <label>Динамики / Наушники</label>
              <select v-model="voice.selectedOutputId">
                <option value="" disabled>Выбери устройство...</option>
                <option v-for="d in outputDevices" :key="d.id" :value="d.id">{{ d.name }}</option>
              </select>
            </div>
            <div class="settings-field">
              <label>Режим микрофона</label>
              <div class="voice-mode-group">
                <label class="voice-mode-opt" :class="{ active: voice.voiceMode === 'open' }">
                  <input type="radio" v-model="voice.voiceMode" value="open" />
                  Открытый
                </label>
                <label class="voice-mode-opt" :class="{ active: voice.voiceMode === 'ptt' }">
                  <input type="radio" v-model="voice.voiceMode" value="ptt" />
                  Push-to-Talk
                </label>
                <label class="voice-mode-opt" :class="{ active: voice.voiceMode === 'vad' }">
                  <input type="radio" v-model="voice.voiceMode" value="vad" />
                  По голосу (VAD)
                </label>
              </div>
            </div>
            <div v-if="voice.voiceMode === 'ptt'" class="settings-field">
              <label>Клавиша PTT</label>
              <button class="ptt-key-btn" :class="{ recording: recordingPttKey }" @click="startRecordingPttKey">
                {{ recordingPttKey ? 'Нажми клавишу...' : formatKeyCode(voice.pttKey) }}
              </button>
              <span class="settings-hint-sm">Удерживай клавишу чтобы говорить</span>
            </div>
            <div v-if="voice.voiceMode === 'vad'" class="settings-hint">
              Микрофон автоматически включается при обнаружении голоса.
            </div>
          </template>

          <template v-if="activeSettingsTab === 'vst'">
            <p class="settings-hint">VST плагины обрабатывают твой микрофон перед отправкой. Поддерживаются .dll, .so, .vst3</p>
            <div class="vst-list">
              <div v-if="vstPlugins.length === 0" class="vst-empty">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor" opacity="0.4">
                  <path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6zm-2 16a2 2 0 1 1 0-4 2 2 0 0 1 0 4z"/>
                </svg>
                <span>Плагины не добавлены</span>
              </div>
              <div v-for="(vst, i) in vstPlugins" :key="i" class="vst-item">
                <div class="vst-icon">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6z"/>
                  </svg>
                </div>
                <div class="vst-info">
                  <div class="vst-name">{{ vst.name }}</div>
                  <div class="vst-path">{{ vst.path }}</div>
                </div>
                <button class="btn-icon-danger" @click="removeVst(i)" title="Удалить">✕</button>
              </div>
            </div>
            <p v-if="vstError" class="modal-error" style="margin-top:4px">{{ vstError }}</p>
            <button class="btn-add-vst" @click="addVst">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
              Добавить плагин
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open as openFile } from '@tauri-apps/plugin-dialog'
import ServerSidebar from '../components/layout/ServerSidebar.vue'
import GuildSidebar from '../components/layout/GuildSidebar.vue'
import ChannelSidebar from '../components/layout/ChannelSidebar.vue'
import ChatArea from '../components/chat/ChatArea.vue'
import { useAuthStore } from '../stores/auth'
import { useGuildStore } from '../stores/guild'
import { useWsStore } from '../stores/ws'
import { useVoiceStore } from '../stores/voice'
import { useServersStore } from '../stores/servers'

const router = useRouter()
const auth = useAuthStore()
const guild = useGuildStore()
const ws = useWsStore()
const voice = useVoiceStore()
const serversStore = useServersStore()

// Модалки
const showAddServer = ref(false)
const showCreateGuild = ref(false)
const showInvite = ref(false)
const showSettings = ref(false)
const showCreateChannel = ref(false)
const newChannelType = ref<'text' | 'voice'>('text')
const newChannelName = ref('')
const newChannelLimit = ref(0)

// Форма добавления сервера
const serverUrlInput = ref('')
const serverError = ref('')
const serverLoading = ref(false)

// Форма создания сервера (гильдии)
const newGuildName = ref('')
const ownerToken = ref('')

// Инвайт
const inviteCodeInput = ref('')

// Загрузка сообщений
const messagesLoading = ref(false)

// Состояние сайдбаров
const guildSidebarOpen = ref(true)
const channelSidebarOpen = ref(true)

// Settings
const activeSettingsTab = ref('account')
const settingsTabs = [
  { id: 'account', label: 'Аккаунт' },
  { id: 'audio', label: 'Аудио' },
  { id: 'vst', label: 'VST плагины' },
]
interface AudioDevice { id: string; name: string }
interface VstInfo { path: string; name: string; vendor: string; version: string; num_inputs: number; num_outputs: number }
const vstPlugins = ref<VstInfo[]>([])
const inputDevices = ref<AudioDevice[]>([])
const outputDevices = ref<AudioDevice[]>([])
const recordingPttKey = ref(false)

const activeGuild = computed(() => guild.guilds.find(g => g.id === guild.activeGuildId))
const activeChannel = computed(() => guild.channels.find(c => c.id === guild.activeChannelId))
const wsStatusLabel = computed(() => {
  switch (ws.status) {
    case 'connected': return '● онлайн'
    case 'connecting': return '○ подключение...'
    default: return '○ офлайн'
  }
})

onMounted(() => {
  if (!auth.isLoggedIn) { router.push('/login'); return }
  if (serversStore.activeUrl) {
    guild.connectToServer(serversStore.activeUrl)
    ws.connect(serversStore.activeUrl)
  }
  document.addEventListener('keydown', handlePttKeyDown)
  document.addEventListener('keyup', handlePttKeyUp)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handlePttKeyDown)
  document.removeEventListener('keyup', handlePttKeyUp)
})

function handlePttKeyDown(e: KeyboardEvent) {
  if (voice.voiceMode === 'ptt' && e.code === voice.pttKey && voice.activeChannelId) {
    e.preventDefault()
    voice.pttPress()
  }
}

function handlePttKeyUp(e: KeyboardEvent) {
  if (voice.voiceMode === 'ptt' && e.code === voice.pttKey) {
    voice.pttRelease()
  }
}

function startRecordingPttKey() {
  recordingPttKey.value = true
  const handler = (e: KeyboardEvent) => {
    e.preventDefault()
    e.stopPropagation()
    voice.pttKey = e.code
    recordingPttKey.value = false
    document.removeEventListener('keydown', handler, true)
  }
  document.addEventListener('keydown', handler, true)
}

function formatKeyCode(code: string): string {
  const map: Record<string, string> = {
    Space: 'Пробел',
    ShiftLeft: 'Shift (L)', ShiftRight: 'Shift (R)',
    ControlLeft: 'Ctrl (L)', ControlRight: 'Ctrl (R)',
    AltLeft: 'Alt (L)', AltRight: 'Alt (R)',
    CapsLock: 'Caps Lock',
  }
  if (map[code]) return map[code]
  if (code.startsWith('Key')) return code.slice(3)
  if (code.startsWith('Digit')) return code.slice(5)
  return code
}

// === Переключение серверов ===
async function switchServer(url: string) {
  ws.disconnect()
  guild.reset()
  serversStore.setActive(url)
  guild.connectToServer(url)
  ws.connect(url)
}

// === Добавить инстанс сервера ===
function openAddServer() {
  serverUrlInput.value = ''
  serverError.value = ''
  showAddServer.value = true
}

function closeAddServer() {
  showAddServer.value = false
  serverError.value = ''
}

async function connectServer() {
  const url = serverUrlInput.value.trim()
  if (!url) return
  serverError.value = ''
  serverLoading.value = true
  try {
    const server = await serversStore.addServer(url)
    await switchServer(server.url)
    // Ждём READY
    await new Promise<void>((resolve, reject) => {
      const start = Date.now()
      const id = setInterval(() => {
        if (ws.status === 'connected') { clearInterval(id); resolve() }
        if (Date.now() - start > 6000) { clearInterval(id); reject(new Error('timeout')) }
      }, 100)
    })
    closeAddServer()
  } catch (e: any) {
    const msg = e?.message || e?.response?.data?.error || String(e)
    serverError.value = `Ошибка: ${msg}`
    serversStore.removeServer(serverUrlInput.value.trim().replace(/\/$/, ''))
  } finally {
    serverLoading.value = false
  }
}

// === Создать сервер (гильдию) ===
function openCreateGuild() {
  newGuildName.value = ''
  ownerToken.value = ''
  serverError.value = ''
  showCreateGuild.value = true
}

async function createGuild() {
  const name = newGuildName.value.trim()
  if (!name) return
  serverError.value = ''
  serverLoading.value = true
  try {
    await guild.createGuild(name, ownerToken.value)
    showCreateGuild.value = false
  } catch (e: any) {
    const msg = e?.response?.data?.error
    if (e?.response?.status === 403) serverError.value = 'Неверный токен владельца'
    else serverError.value = msg || 'Ошибка создания'
  } finally {
    serverLoading.value = false
  }
}

// === Инвайт ===
async function joinByInvite() {
  const code = inviteCodeInput.value.trim()
  if (!code) return
  serverError.value = ''
  serverLoading.value = true
  try {
    await guild.joinByInvite(code)
    showInvite.value = false
    inviteCodeInput.value = ''
  } catch {
    serverError.value = 'Код не найден или истёк.'
  } finally {
    serverLoading.value = false
  }
}

// === Каналы — создание ===
function openCreateChannel(type: 'text' | 'voice') {
  newChannelType.value = type
  newChannelName.value = ''
  newChannelLimit.value = 0
  serverError.value = ''
  showCreateChannel.value = true
}

async function createChannel() {
  const name = newChannelName.value.trim()
  if (!name || !guild.activeGuildId) return
  serverError.value = ''
  serverLoading.value = true
  try {
    await guild.createChannel(
      guild.activeGuildId,
      name,
      newChannelType.value,
      newChannelLimit.value > 0 ? newChannelLimit.value : undefined
    )
    showCreateChannel.value = false
  } catch (e: any) {
    serverError.value = e?.response?.data?.error || 'Ошибка создания канала'
  } finally {
    serverLoading.value = false
  }
}

// === Каналы и чат ===
async function selectGuild(guildId: string) {
  await guild.loadChannels(guildId)
  await guild.loadMembers(guildId)
}

async function selectTextChannel(channel: any) {
  if (channel.type !== 'text') return
  // Optimistic: set channel immediately so ChatArea renders
  guild.activeChannelId = channel.id
  guild.messages = []
  messagesLoading.value = true
  try {
    await guild.loadMessages(guild.activeGuildId!, channel.id)
  } catch (e) {
    console.error('Failed to load messages:', e)
  } finally {
    messagesLoading.value = false
  }
}

function joinVoice(channel: any) {
  if (!guild.activeGuildId) return
  if (voice.activeChannelId === channel.id) {
    ws.joinVoiceChannel(guild.activeGuildId, null)
    voice.disconnect()
  } else {
    ws.joinVoiceChannel(guild.activeGuildId, channel.id)
  }
}

async function onSend({ content, replyTo }: { content: string; replyTo: string | null }) {
  if (!guild.activeGuildId || !guild.activeChannelId) return
  await guild.sendMessage(guild.activeGuildId, guild.activeChannelId, content, replyTo ?? undefined)
}

async function onToggleReaction(messageId: string, emoji: string, alreadyReacted: boolean) {
  if (!guild.activeGuildId || !guild.activeChannelId) return
  try {
    if (alreadyReacted) {
      await guild.removeReaction(guild.activeGuildId, guild.activeChannelId, messageId, emoji)
    } else {
      await guild.addReaction(guild.activeGuildId, guild.activeChannelId, messageId, emoji)
    }
  } catch (e) {
    console.error('reaction error:', e)
  }
}

async function loadMore() {
  if (!guild.activeGuildId || !guild.activeChannelId || messagesLoading.value) return
  messagesLoading.value = true
  try {
    await guild.loadMoreMessages(guild.activeGuildId, guild.activeChannelId)
  } catch (e) {
    console.error('Failed to load more messages:', e)
  } finally {
    messagesLoading.value = false
  }
}

// === Настройки ===
async function openSettings() {
  showSettings.value = true
  activeSettingsTab.value = 'account'
  try {
    inputDevices.value = await invoke<AudioDevice[]>('list_input_devices')
    outputDevices.value = await invoke<AudioDevice[]>('list_output_devices')
    if (!voice.selectedInputId) voice.selectedInputId = await invoke<string>('default_input_device').catch(() => '')
    if (!voice.selectedOutputId) voice.selectedOutputId = await invoke<string>('default_output_device').catch(() => '')
  } catch {}
}

async function addVst() {
  const path = await openFile({
    title: 'Выбери VST плагин',
    filters: [{ name: 'VST Plugin', extensions: ['dll', 'so', 'vst3'] }],
    multiple: false,
  })
  if (!path || typeof path !== 'string') return
  try {
    const info = await invoke<VstInfo>('load_vst_info', { path })
    vstPlugins.value.push(info)
  } catch (e: any) {
    vstError.value = 'Не удалось загрузить плагин: ' + String(e)
  }
}
const vstError = ref('')

function removeVst(i: number) { vstPlugins.value.splice(i, 1) }

function logout() { auth.logout(); router.push('/login') }
</script>

<style scoped>
.app-layout { height: 100%; display: flex; overflow: hidden; }

.no-server, .no-guild-main, .no-channel {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--text3);
  font-size: 13px;
}
.no-server-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
}
.no-server-icon { color: var(--text3); }
.no-server-hint h3 { font-size: 18px; font-weight: 700; color: var(--text); }
.no-server-hint p { color: var(--text2); font-size: 13px; }
.connect-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  padding: 10px 20px;
  background: var(--accent);
  color: white;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
}
.connect-btn:hover { background: var(--accent-hover); }

/* Модалки */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.modal {
  background: var(--bg-dark);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 28px;
  width: 380px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-shadow: 0 24px 64px rgba(0,0,0,0.5);
}
.modal h3 { font-size: 16px; font-weight: 700; }
.modal-hint { font-size: 12px; color: var(--text2); }
.modal-field { display: flex; flex-direction: column; gap: 5px; }
.modal-field label { font-size: 11px; font-weight: 600; color: var(--text2); text-transform: uppercase; letter-spacing: 0.5px; }
.modal-error { font-size: 12px; color: var(--red); padding: 6px 10px; background: rgba(255,85,85,0.1); border-radius: 6px; }
.modal-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 4px; }
.btn-ghost { padding: 8px 16px; border-radius: 6px; background: transparent; color: var(--text2); border: 1px solid var(--border); }
.btn-primary { padding: 8px 16px; border-radius: 6px; background: var(--accent); color: white; font-weight: 600; }
.btn-primary:hover { background: var(--accent-hover); }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }

/* Настройки */
.settings-modal {
  width: 740px; max-width: 95vw;
  height: 520px; max-height: 90vh;
  flex-direction: row; padding: 0; gap: 0; overflow: hidden;
}
.settings-sidebar {
  width: 180px; background: var(--bg-darkest);
  padding: 20px 8px; display: flex; flex-direction: column; gap: 2px; flex-shrink: 0;
}
.settings-tab { padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 13px; color: var(--text2); transition: all 0.1s; }
.settings-tab:hover { background: var(--bg-hover); color: var(--text); }
.settings-tab.active { background: var(--bg-hover); color: var(--text); font-weight: 600; }
.settings-content { flex: 1; padding: 28px; overflow-y: auto; position: relative; display: flex; flex-direction: column; gap: 16px; }
.settings-content h2 { font-size: 18px; font-weight: 700; margin-bottom: 4px; }
.settings-close {
  position: absolute; top: 16px; right: 16px;
  background: transparent; color: var(--text2);
  width: 28px; height: 28px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
}
.settings-close:hover { background: var(--bg-hover); color: var(--text); }

.settings-row { display: flex; align-items: center; gap: 14px; }
.settings-avatar {
  width: 54px; height: 54px; border-radius: 50%;
  background: linear-gradient(135deg, var(--accent), #7c6cf7);
  display: flex; align-items: center; justify-content: center;
  font-size: 22px; font-weight: 700; color: #fff; flex-shrink: 0;
}
.settings-userinfo { display: flex; flex-direction: column; gap: 3px; }
.settings-username { font-size: 16px; font-weight: 700; color: var(--text); }
.settings-uuid { font-size: 11px; color: var(--text2); font-family: 'JetBrains Mono', monospace; }

.settings-divider { height: 1px; background: var(--border); }
.settings-hint { font-size: 13px; color: var(--text2); line-height: 1.5; }

.settings-field { display: flex; flex-direction: column; gap: 7px; }
.settings-field label {
  font-size: 11px; font-weight: 700; color: var(--text2);
  text-transform: uppercase; letter-spacing: 0.8px;
}
.settings-field select {
  background: var(--bg-light); border: 1px solid var(--border);
  border-radius: var(--radius); color: var(--text);
  padding: 10px 14px; font-size: 14px; font-family: inherit; outline: none;
  cursor: pointer;
}
.settings-field select:focus { border-color: var(--accent); }
.settings-field select option { background: var(--bg-dark); }

.vst-list { display: flex; flex-direction: column; gap: 6px; min-height: 60px; }
.vst-empty {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 8px; padding: 24px; color: var(--text2); font-size: 13px;
  border: 1px dashed var(--border); border-radius: var(--radius); text-align: center;
}
.vst-item {
  display: flex; align-items: center; gap: 10px;
  background: var(--bg-light); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 10px 12px;
}
.vst-icon {
  width: 32px; height: 32px; border-radius: 8px;
  background: var(--bg-hover); display: flex; align-items: center; justify-content: center;
  color: var(--accent); flex-shrink: 0;
}
.vst-info { flex: 1; min-width: 0; }
.vst-name { font-size: 13px; font-weight: 600; color: var(--text); }
.vst-path { font-size: 11px; color: var(--text2); font-family: 'JetBrains Mono', monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-top: 2px; }

.btn-icon-danger {
  background: transparent; color: var(--text2);
  width: 28px; height: 28px; border-radius: 6px;
  display: flex; align-items: center; justify-content: center; flex-shrink: 0;
}
.btn-icon-danger:hover { background: rgba(255,85,85,0.15); color: var(--red); }

.btn-danger {
  padding: 10px 18px; border-radius: 6px;
  background: var(--red); color: #fff;
  font-weight: 600; font-size: 13px; align-self: flex-start;
}
.btn-danger:hover { opacity: 0.85; }

.voice-mode-group {
  display: flex;
  gap: 8px;
}
.voice-mode-opt {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-light);
  cursor: pointer;
  font-size: 13px;
  color: var(--text2);
  transition: all 0.1s;
  user-select: none;
}
.voice-mode-opt input[type="radio"] { display: none; }
.voice-mode-opt:hover { border-color: var(--accent); color: var(--text); }
.voice-mode-opt.active { border-color: var(--accent); background: rgba(88, 101, 242, 0.15); color: var(--text); font-weight: 600; }

.ptt-key-btn {
  padding: 9px 18px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-light);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  font-family: 'JetBrains Mono', monospace;
  transition: all 0.1s;
  align-self: flex-start;
}
.ptt-key-btn:hover { border-color: var(--accent); }
.ptt-key-btn.recording { border-color: var(--accent); background: rgba(88, 101, 242, 0.15); color: var(--accent); animation: ptt-pulse 1s ease-in-out infinite; }
@keyframes ptt-pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.6; } }

.settings-hint-sm { font-size: 11px; color: var(--text2); }

.btn-add-vst {
  display: flex; align-items: center; gap: 6px;
  padding: 8px 16px; border-radius: 6px;
  background: var(--bg-light); border: 1px solid var(--border);
  color: var(--text); font-size: 13px; font-weight: 600;
  align-self: flex-start; cursor: pointer;
}
.btn-add-vst:hover { background: var(--bg-hover); border-color: var(--accent); color: var(--accent); }
</style>
