<template>
  <div class="app-shell-container">
    <!-- Сетка разметки приложения из 4 колонок -->
    <div
      class="app-layout-grid"
      :class="{
        'hide-info': !showInfoColumn,
        'collapsed-guilds': isGuildsCollapsed
      }"
    >
      <!-- Колонка 1: Серверы -->
      <ServersColumn
        :servers="serversStore.servers"
        :activeUrl="serversStore.activeUrl"
        :connectedUrl="ws.status === 'connected' ? serversStore.activeUrl : null"
        :loadingUrl="ws.status === 'connecting' ? serversStore.activeUrl : null"
        :username="auth.username"
        @select-server="switchServer"
        @connect-server="switchServer"
        @add-server="openAddServer"
        @disconnect-server="disconnectServer"
        @remove-server="serversStore.removeServer"
        @open-settings="openSettings"
        @logout="logout"
      />

      <!-- Колонка 2: Гильдии -->
      <GuildsColumn
        v-if="serversStore.activeUrl"
        :guilds="guild.guilds"
        :activeGuildId="guild.activeGuildId"
        :isCollapsed="isGuildsCollapsed"
        :pendingInvitesCount="0"
        @select-guild="selectGuild"
        @create-guild="openCreateGuild"
        @join-invite="showInvite = true"
        @guild-contextmenu="openGuildContextMenu"
      />
      <!-- Заглушка, если сервер не выбран -->
      <div v-else class="no-server-selected">
        <div class="no-server-hint">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
            <path d="M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z"/>
          </svg>
          <h3>Нет серверов</h3>
          <p>Добавьте сервер для начала работы</p>
        </div>
      </div>

      <!-- Колонка 3: Чат -->
      <ChatColumn
        :serverName="serversStore.activeServer?.name || ''"
        :guildName="activeGuild?.name || ''"
        :channel="activeChannel"
        :channels="guild.channels"
        :messages="guild.messages"
        :userId="auth.userId"
        :username="auth.username"
        :members="guild.members"
        :loading="messagesLoading"
        :showInfoColumn="showInfoColumn"
        @select-channel="selectTextChannel"
        @send-message="onSend"
        @toggle-reaction="onToggleReaction"
        @load-more-messages="loadMore"
        @toggle-info="toggleInfoColumn"
        @focus-servers="focusServers"
        @focus-guilds="focusGuilds"
        @edit-message="onEditMessage"
        @delete-message="onDeleteMessage"
      />

      <!-- Колонка 4: Инфо-панель -->
      <InfoColumn
        v-if="showInfoColumn"
        :guild="activeGuild"
        :channels="guild.channels"
        :activeVoiceChannelId="voice.activeChannelId"
        :voiceStates="voice.voiceStates"
        :activeSpeakers="voice.activeSpeakers"
        :members="guild.members"
        :activityEvents="activityStore.events"
        :userId="auth.userId"
        @join-voice="joinVoice"
        @create-voice-channel="openCreateChannel('voice')"
        @open-guild-settings="openGuildSettings"
        @create-invite="openInviteCreate"
        @search="guildSearch"
        @events="guildEvents"
      />
    </div>

    <!-- Модалка: добавить сервер -->
    <div v-if="showAddServer" class="modal-overlay" @click.self="closeAddServer">
      <!-- Выбор режима -->
      <div v-if="addServerMode === 'choose'" class="modal">
        <h3>Добавить сервер</h3>
        <div class="add-server-options">
          <button class="add-option-card" @click="openBrowse">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg>
            <span class="add-option-title">Публичные серверы</span>
            <span class="add-option-desc">Найти сервер в каталоге BeyVox</span>
          </button>
          <button class="add-option-card" @click="addServerMode = 'direct'">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor"><path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/></svg>
            <span class="add-option-title">Прямое подключение</span>
            <span class="add-option-desc">Ввести адрес сервера вручную</span>
          </button>
        </div>
        <div class="modal-actions">
          <button class="btn-ghost" @click="closeAddServer">Отмена</button>
        </div>
      </div>

      <!-- Обзор публичных серверов -->
      <div v-else-if="addServerMode === 'browse'" class="modal modal-wide">
        <div class="modal-top">
          <button class="btn-back" @click="addServerMode = 'choose'">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/></svg>
          </button>
          <h3>Публичные серверы</h3>
        </div>
        <div v-if="publicLoading" class="browse-state">Загрузка...</div>
        <div v-else-if="publicError" class="browse-state error">{{ publicError }}</div>
        <div v-else-if="publicServers.length === 0" class="browse-state">Серверов не найдено</div>
        <div v-else class="public-server-list">
          <div v-for="s in publicServers" :key="s.id" class="public-server-card">
            <div class="ps-icon">
              <img v-if="s.icon_url" :src="s.icon_url" :alt="s.name" />
              <div v-else class="ps-icon-placeholder">{{ s.name[0].toUpperCase() }}</div>
            </div>
            <div class="ps-info">
              <div class="ps-name">{{ s.name }}</div>
              <div class="ps-desc">{{ s.description || 'Нет описания' }}</div>
              <div class="ps-meta">
                <span class="ps-members">{{ s.total_members }} участников</span>
                <span class="ps-online">{{ s.online_count }} онлайн</span>
                <span v-for="t in s.tags" :key="t" class="tag">{{ t }}</span>
              </div>
            </div>
            <button
              class="btn-primary ps-connect-btn"
              :disabled="serverLoading"
              @click="connectToPublicServer(s.address)"
            >
              {{ serverLoading ? '...' : 'Подключиться' }}
            </button>
          </div>
        </div>
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
      </div>

      <!-- Прямое подключение -->
      <div v-else-if="addServerMode === 'direct'" class="modal">
        <div class="modal-top">
          <button class="btn-back" @click="addServerMode = 'choose'">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/></svg>
          </button>
          <h3>Прямое подключение</h3>
        </div>
        <p class="modal-hint">Введите адрес инстанса BeyVox-сервера</p>
        <input
          v-model="serverUrlInput"
          placeholder="https://server.example.com"
          @keydown.enter="connectServer"
          autofocus
        />
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="closeAddServer">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading || !serverUrlInput.trim()" @click="connectServer">
            {{ serverLoading ? 'Подключение...' : 'Подключиться' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Модалка: создать гильдию -->
    <div v-if="showCreateGuild" class="modal-overlay" @click.self="showCreateGuild = false">
      <div class="modal">
        <h3>Создать гильдию</h3>
        <div class="modal-field">
          <label class="uppercase-label">Название гильдии</label>
          <input
            v-model="newGuildName"
            placeholder="Моё сообщество"
            @keydown.enter="createGuild"
            autofocus
          />
        </div>
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showCreateGuild = false">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading || !newGuildName.trim()" @click="createGuild">
            {{ serverLoading ? 'Создание...' : 'Создать' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Модалка: войти по инвайту -->
    <div v-if="showInvite" class="modal-overlay" @click.self="showInvite = false">
      <div class="modal">
        <h3>Присоединиться по приглашению</h3>
        <p class="modal-hint">Введите полученный код инвайта</p>
        <input
          v-model="inviteCodeInput"
          placeholder="abc12345"
          @keydown.enter="joinByInvite"
          autofocus
        />
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showInvite = false">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading || !inviteCodeInput.trim()" @click="joinByInvite">
            {{ serverLoading ? 'Вход...' : 'Войти' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Модалка: создать инвайт -->
    <div v-if="showCreateInvite" class="modal-overlay" @click.self="showCreateInvite = false">
      <div class="modal">
        <h3>Пригласить участников</h3>
        <p class="modal-hint-text">Скопируйте сгенерированный код и отправьте его друзьям</p>
        <div v-if="createdInviteCode" class="invite-code-box">
          <span class="invite-code">{{ createdInviteCode }}</span>
          <button class="btn-secondary" @click="copyInviteCode">
            {{ inviteCopied ? 'Скопировано!' : 'Копировать' }}
          </button>
        </div>
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showCreateInvite = false">Закрыть</button>
          <button class="btn-primary" :disabled="serverLoading" @click="() => createInvite()">
            {{ serverLoading ? 'Генерация...' : 'Новый код' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Модалка: создать канал -->
    <div v-if="showCreateChannel" class="modal-overlay" @click.self="showCreateChannel = false">
      <div class="modal">
        <h3>Создать {{ newChannelType === 'text' ? 'текстовый' : 'голосовой' }} канал</h3>
        <div class="modal-field">
          <label class="uppercase-label">Название канала</label>
          <input
            v-model="newChannelName"
            :placeholder="newChannelType === 'text' ? 'general' : 'Голосовая комната'"
            @keydown.enter="createChannel"
            autofocus
          />
        </div>
        <div v-if="newChannelType === 'voice'" class="modal-field">
          <label class="uppercase-label">Лимит участников (0 = без лимита)</label>
          <input v-model.number="newChannelLimit" type="number" min="0" max="99" placeholder="0" />
        </div>
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showCreateChannel = false">Отмена</button>
          <button class="btn-primary" :disabled="serverLoading || !newChannelName.trim()" @click="createChannel">
            {{ serverLoading ? 'Создание...' : 'Создать' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Модальное окно настроек (Глобальное) -->
    <SettingsModal v-if="showSettings" @close="showSettings = false" />

    <!-- Ошибка CPAL микрофона/устройства ввода -->
    <div v-if="voice.micError" class="mic-error-banner" @click="voice.micError = ''">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      {{ voice.micError }}
    </div>

    <!-- Контекстное меню гильдии -->
    <Teleport to="body">
      <div v-if="guildMenu.visible" class="ctx-overlay" @mousedown.self="closeGuildMenu" @contextmenu.prevent>
        <div class="ctx-menu" :style="{ top: guildMenu.y + 'px', left: guildMenu.x + 'px' }">
          <button class="ctx-item" @click="triggerGuildSettings">Настройки гильдии</button>
          <button class="ctx-item" @click="triggerGuildInvite">Создать приглашение</button>
          <div class="ctx-divider" />
          <button class="ctx-item" @click="triggerCreateChannel('text')">Создать текстовый канал</button>
          <button class="ctx-item" @click="triggerCreateChannel('voice')">Создать голосовой канал</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

import ServersColumn from '../components/layout/ServersColumn.vue'
import GuildsColumn from '../components/layout/GuildsColumn.vue'
import ChatColumn from '../components/chat/ChatColumn.vue'
import InfoColumn from '../components/layout/InfoColumn.vue'
import SettingsModal from '../components/settings/SettingsModal.vue'

import { fetchPublicServers, type PublicServer } from '../api'
import { useAuthStore } from '../stores/auth'
import { useGuildStore, type Guild } from '../stores/guild'
import { useWsStore } from '../stores/ws'
import { useVoiceStore } from '../stores/voice'
import { useServersStore } from '../stores/servers'
import { useActivityStore } from '../stores/activity'

const router = useRouter()
const auth = useAuthStore()
const guild = useGuildStore()
const ws = useWsStore()
const voice = useVoiceStore()
const serversStore = useServersStore()
const activityStore = useActivityStore()

// Состояния модалок
const showAddServer = ref(false)
const addServerMode = ref<'choose' | 'browse' | 'direct'>('choose')
const publicServers = ref<PublicServer[]>([])
const publicLoading = ref(false)
const publicError = ref('')
const showCreateGuild = ref(false)
const showInvite = ref(false)
const showCreateInvite = ref(false)
const showCreateChannel = ref(false)
const showSettings = ref(false)

// Входные данные
const serverUrlInput = ref('')
const newGuildName = ref('')
const inviteCodeInput = ref('')
const createdInviteCode = ref('')
const inviteCopied = ref(false)
const newChannelName = ref('')
const newChannelType = ref<'text' | 'voice'>('text')
const newChannelLimit = ref(0)

const serverError = ref('')
const serverLoading = ref(false)
const messagesLoading = ref(false)

// Слушатель ширины окна для адаптивного интерфейса
const windowWidth = ref(window.innerWidth)
const showInfoColumnForce = ref(true)

const isGuildsCollapsed = computed(() => {
  if (showInfoColumn.value) {
    return windowWidth.value < 1180
  }
  return windowWidth.value < 1024
})

const showInfoColumn = computed(() => {
  return showInfoColumnForce.value && guild.activeGuildId !== null
})

function toggleInfoColumn() {
  showInfoColumnForce.value = !showInfoColumnForce.value
}

function handleResize() {
  windowWidth.value = window.innerWidth
}

// Активная гильдия и активный текстовый канал
const activeGuild = computed(() => guild.guilds.find(g => g.id === guild.activeGuildId) || null)
const activeChannel = computed(() => guild.channels.find(c => c.id === guild.activeChannelId) || null)

onMounted(() => {
  if (!auth.isLoggedIn) {
    router.push('/login')
    return
  }
  window.addEventListener('resize', handleResize)
  
  if (serversStore.activeUrl) {
    guild.connectToServer(serversStore.activeUrl)
    ws.connect(serversStore.activeUrl)
    serversStore.checkAllPings()
  }

  // Запуск фоновых пингов каждые 20 секунд
  setInterval(() => {
    if (serversStore.servers.length) {
      serversStore.checkAllPings()
    }
  }, 20000)

  voice.initVoiceSettings()
  initEffectsStartup()
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})

// Инициализация звуковых эффектов при запуске
async function initEffectsStartup() {
  try {
    const savedGate = localStorage.getItem('fx_gate')
    const savedComp = localStorage.getItem('fx_comp')
    const savedEq = localStorage.getItem('fx_eq')
    
    if (savedGate) {
      const g = JSON.parse(savedGate)
      await invoke('set_noise_gate', {
        enabled: g.enabled,
        thresholdDb: g.threshold_db,
        attackMs: g.attack_ms,
        releaseMs: g.release_ms,
      })
    }
    if (savedComp) {
      const c = JSON.parse(savedComp)
      await invoke('set_compressor', {
        enabled: c.enabled,
        thresholdDb: c.threshold_db,
        ratio: c.ratio,
        attackMs: c.attack_ms,
        releaseMs: c.release_ms,
        makeupGainDb: c.makeup_db,
      })
    }
    if (savedEq) {
      const q = JSON.parse(savedEq)
      await invoke('set_eq', {
        enabled: q.enabled,
        lowGainDb: q.low_db,
        midGainDb: q.mid_db,
        midFreq: q.mid_freq,
        highGainDb: q.high_db,
      })
    }
  } catch (e) {
    console.error('Ошибка инициализации эффектов при запуске:', e)
  }
}

// Переключение серверов
async function switchServer(url: string) {
  ws.disconnect()
  guild.reset()
  activityStore.clear()
  serversStore.setActive(url)
  guild.connectToServer(url)
  ws.connect(url)
}

function disconnectServer(url: string) {
  if (serversStore.activeUrl === url) {
    ws.disconnect()
    guild.reset()
    activityStore.clear()
    serversStore.setActive(null)
  }
}

// Добавление сервера
function openAddServer() {
  serverUrlInput.value = ''
  serverError.value = ''
  addServerMode.value = 'choose'
  showAddServer.value = true
}

function closeAddServer() {
  showAddServer.value = false
  serverError.value = ''
}

async function openBrowse() {
  addServerMode.value = 'browse'
  publicServers.value = []
  publicError.value = ''
  publicLoading.value = true
  try {
    publicServers.value = await fetchPublicServers()
  } catch (e: any) {
    publicError.value = e?.message || 'Ошибка загрузки'
  } finally {
    publicLoading.value = false
  }
}

async function connectToPublicServer(address: string) {
  const url = address.startsWith('http') ? address : `https://${address}`
  serverError.value = ''
  serverLoading.value = true
  try {
    const server = await serversStore.addServer(url)
    await switchServer(server.url)
    await new Promise<void>((resolve, reject) => {
      const start = Date.now()
      const id = setInterval(() => {
        if (ws.status === 'connected') { clearInterval(id); resolve() }
        if (Date.now() - start > 6000) { clearInterval(id); reject(new Error('Превышено время ожидания')) }
      }, 150)
    })
    closeAddServer()
  } catch (e: any) {
    serverError.value = e?.message || 'Ошибка подключения'
    serversStore.removeServer(url.replace(/\/$/, ''))
  } finally {
    serverLoading.value = false
  }
}

async function connectServer() {
  const url = serverUrlInput.value.trim()
  if (!url) return
  serverError.value = ''
  serverLoading.value = true
  try {
    const server = await serversStore.addServer(url)
    await switchServer(server.url)
    
    // Ждём CONNECTED состояния
    await new Promise<void>((resolve, reject) => {
      const start = Date.now()
      const id = setInterval(() => {
        if (ws.status === 'connected') {
          clearInterval(id)
          resolve()
        }
        if (Date.now() - start > 6000) {
          clearInterval(id)
          reject(new Error('Превышено время ожидания подключения к серверу.'))
        }
      }, 150)
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

// Создание гильдии
function openCreateGuild() {
  newGuildName.value = ''
  serverError.value = ''
  showCreateGuild.value = true
}

async function createGuild() {
  const name = newGuildName.value.trim()
  if (!name) return
  serverError.value = ''
  serverLoading.value = true
  try {
    await guild.createGuild(name)
    showCreateGuild.value = false
  } catch (e: any) {
    const msg = e?.response?.data?.error
    if (e?.response?.status === 403) {
      serverError.value = 'Недостаточно прав для создания гильдий'
    } else {
      serverError.value = msg || 'Ошибка создания гильдии'
    }
  } finally {
    serverLoading.value = false
  }
}

// Приглашения
async function openInviteCreate(guildId?: string) {
  createdInviteCode.value = ''
  serverError.value = ''
  showCreateInvite.value = true
  await createInvite(guildId)
}

async function createInvite(guildId?: string) {
  const id = guildId || guild.activeGuildId
  if (!id) return
  serverLoading.value = true
  serverError.value = ''
  try {
    const code = await guild.createInvite(id)
    createdInviteCode.value = code
  } catch {
    serverError.value = 'Не удалось сгенерировать инвайт-код'
  } finally {
    serverLoading.value = false
  }
}

async function copyInviteCode() {
  await navigator.clipboard.writeText(createdInviteCode.value)
  inviteCopied.value = true
  setTimeout(() => { inviteCopied.value = false }, 2000)
}

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
    serverError.value = 'Код приглашения не найден или истёк'
  } finally {
    serverLoading.value = false
  }
}

// Каналы
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

// Состояние и методы контекстного меню гильдии
const guildMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  guild: null as Guild | null
})

function openGuildContextMenu(e: MouseEvent, g: Guild) {
  guildMenu.guild = g
  guildMenu.x = Math.min(e.clientX, window.innerWidth - 160)
  guildMenu.y = Math.min(e.clientY, window.innerHeight - 180)
  guildMenu.visible = true
}

function closeGuildMenu() {
  guildMenu.visible = false
}

async function triggerGuildSettings() {
  if (guildMenu.guild) {
    await selectGuild(guildMenu.guild.id)
    openGuildSettings()
  }
  closeGuildMenu()
}

async function triggerGuildInvite() {
  if (guildMenu.guild) {
    await selectGuild(guildMenu.guild.id)
    openInviteCreate(guildMenu.guild.id)
  }
  closeGuildMenu()
}

async function triggerCreateChannel(type: 'text' | 'voice') {
  if (guildMenu.guild) {
    await selectGuild(guildMenu.guild.id)
    openCreateChannel(type)
  }
  closeGuildMenu()
}

// Выбор элементов
async function selectGuild(guildId: string) {
  await guild.loadChannels(guildId)
  await guild.loadMembers(guildId)
  
  // Автоматический выбор первого текстового канала
  const textCh = guild.channels.find(c => c.type === 'text')
  if (textCh) {
    selectTextChannel(textCh)
  } else {
    guild.activeChannelId = null
    guild.messages = []
  }
}

async function selectTextChannel(channel: any) {
  if (channel.type !== 'text') return
  guild.activeChannelId = channel.id
  guild.messages = []
  messagesLoading.value = true
  try {
    await guild.loadMessages(guild.activeGuildId!, channel.id)
  } catch (e) {
    console.error('Ошибка загрузки сообщений канала:', e)
  } finally {
    messagesLoading.value = false
  }
}

async function joinVoice(channel: any) {
  if (!guild.activeGuildId) return
  if (voice.activeChannelId === channel.id) {
    // Выход из канала
    ws.joinVoiceChannel(guild.activeGuildId, null)
    voice.disconnect()
  } else {
    // Вход в канал
    await voice.prewarmAudio()
    ws.joinVoiceChannel(guild.activeGuildId, channel.id)
  }
}

// Отправка сообщений и реакции
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
    console.error('Ошибка добавления/удаления реакции:', e)
  }
}

async function onEditMessage(messageId: string, content: string) {
  if (!guild.activeGuildId || !guild.activeChannelId) return
  try {
    await guild.editMessage(guild.activeGuildId, guild.activeChannelId, messageId, content)
  } catch (e) {
    console.error('Ошибка редактирования сообщения:', e)
  }
}

async function onDeleteMessage(messageId: string) {
  if (!guild.activeGuildId || !guild.activeChannelId) return
  try {
    await guild.removeMessageFromServer(guild.activeGuildId, guild.activeChannelId, messageId)
  } catch (e) {
    console.error('Ошибка удаления сообщения:', e)
  }
}

async function loadMore() {
  if (!guild.activeGuildId || !guild.activeChannelId || messagesLoading.value) return
  messagesLoading.value = true
  try {
    await guild.loadMoreMessages(guild.activeGuildId, guild.activeChannelId)
  } catch (e) {
    console.error('Не удалось дозагрузить историю сообщений:', e)
  } finally {
    messagesLoading.value = false
  }
}

// Другие хендлеры и no-op методы для структуры
function openSettings() {
  showSettings.value = true
}

function logout() {
  auth.logout()
  router.push('/login')
}

function openGuildSettings() {
  // Настройки гильдии (если понадобятся, в ТЗ заглушка)
}

function guildSearch() {
  // Поиск по гильдии
}

function guildEvents() {
  // Календарь событий гильдии
}

function focusServers() {
  // Фокусировать сайдбар серверов
}

function focusGuilds() {
  // Фокусировать сайдбар гильдий
}
</script>

<style scoped>
.app-shell-container {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--bg-app);
}

.app-layout-grid {
  display: grid;
  grid-template-columns: 260px 300px minmax(480px, 1fr) 320px;
  grid-template-rows: 100vh;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* Сетки для адаптивности */
.app-layout-grid.collapsed-guilds {
  grid-template-columns: 260px 70px minmax(480px, 1fr) 320px;
}

.app-layout-grid.hide-info {
  grid-template-columns: 260px 300px minmax(480px, 1fr);
}

.app-layout-grid.collapsed-guilds.hide-info {
  grid-template-columns: 260px 70px minmax(480px, 1fr);
}

@media (max-width: 1360px) {
  .app-layout-grid {
    grid-template-columns: 260px 240px minmax(400px, 1fr) 280px;
  }
  .app-layout-grid.collapsed-guilds {
    grid-template-columns: 260px 70px minmax(400px, 1fr) 280px;
  }
  .app-layout-grid.hide-info {
    grid-template-columns: 260px 240px minmax(400px, 1fr);
  }
  .app-layout-grid.collapsed-guilds.hide-info {
    grid-template-columns: 260px 70px minmax(400px, 1fr);
  }
}

@media (max-width: 1024px) {
  .app-layout-grid {
    grid-template-columns: 260px 70px minmax(300px, 1fr) 260px;
  }
  .app-layout-grid.collapsed-guilds {
    grid-template-columns: 260px 70px minmax(300px, 1fr) 260px;
  }
  .app-layout-grid.hide-info {
    grid-template-columns: 260px 70px minmax(300px, 1fr);
  }
  .app-layout-grid.collapsed-guilds.hide-info {
    grid-template-columns: 260px 70px minmax(300px, 1fr);
  }
}

.no-server-selected {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-panel);
  border-right: 1px solid var(--border);
  color: var(--text-secondary);
  text-align: center;
  user-select: none;
  height: 100vh;
}

.no-server-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 24px;
}
.no-server-hint h3 {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
}
.no-server-hint p {
  font-size: 12px;
  color: var(--text-muted);
}

/* Модалки */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  padding: 24px;
  width: 380px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  user-select: none;
}

.modal h3 {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
}

.modal-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: -6px;
}

.modal-hint-text {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}

.modal-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.modal-error {
  font-size: 12px;
  color: var(--danger);
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border-radius: 6px;
  border-left: 3px solid var(--danger);
}

.modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 4px;
}

.btn-ghost {
  padding: 8px 16px;
  border-radius: var(--radius-item);
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border);
  font-weight: 600;
}
.btn-ghost:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-primary {
  padding: 8px 16px;
  border-radius: var(--radius-item);
  background: var(--accent);
  color: white;
  font-weight: 600;
}
.btn-primary:hover {
  opacity: 0.95;
}
.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.invite-code-box {
  display: flex;
  gap: 8px;
  align-items: center;
  background: var(--bg-app);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  padding: 10px 12px;
}

.invite-code {
  flex: 1;
  font-family: monospace;
  font-size: 18px;
  font-weight: 700;
  color: var(--accent);
  letter-spacing: 2px;
  text-align: center;
}

.btn-secondary {
  background: var(--bg-hover);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  padding: 6px 12px;
  font-weight: 600;
}
.btn-secondary:hover {
  background: var(--bg-active);
}

/* Модалка добавления сервера */
.modal-wide {
  width: 540px;
}

.modal-top {
  display: flex;
  align-items: center;
  gap: 10px;
}

.btn-back {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  border-radius: 6px;
  flex-shrink: 0;
}
.btn-back:hover { background: var(--bg-hover); color: var(--text-primary); }

.add-server-options {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.add-option-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: var(--bg-app);
  text-align: left;
  cursor: pointer;
  color: var(--text-primary);
  transition: border-color 0.15s, background 0.15s;
}
.add-option-card:hover {
  border-color: var(--accent);
  background: rgba(124, 108, 255, 0.05);
}
.add-option-card svg { color: var(--accent); margin-bottom: 4px; }
.add-option-title { font-weight: 700; font-size: 14px; }
.add-option-desc { font-size: 12px; color: var(--text-secondary); }

/* Список публичных серверов */
.public-server-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
  padding-right: 4px;
}

.public-server-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  background: var(--bg-app);
}

.ps-icon { flex-shrink: 0; }
.ps-icon img, .ps-icon-placeholder {
  width: 44px; height: 44px;
  border-radius: 10px;
  object-fit: cover;
}
.ps-icon-placeholder {
  background: var(--bg-hover);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: 700;
  color: var(--accent);
}

.ps-info { flex: 1; min-width: 0; }
.ps-name { font-weight: 700; font-size: 14px; margin-bottom: 2px; }
.ps-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.ps-meta { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; margin-bottom: 4px; }
.ps-online { font-size: 11px; color: var(--green); font-weight: 500; }
.tag {
  font-size: 10px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1px 6px;
  color: var(--text-secondary);
}

.ps-members { color: var(--text-secondary); font-size: 12px; }
.ps-guild-chip {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 11px;
  background: rgba(124, 108, 255, 0.1);
  border: 1px solid rgba(124, 108, 255, 0.2);
  border-radius: 4px;
  padding: 1px 6px;
  color: var(--accent);
}
.ps-guild-default { color: var(--green); font-size: 8px; }
.ps-guild-count { color: var(--text-muted); font-size: 10px; }

.ps-connect-btn { flex-shrink: 0; padding: 6px 14px; font-size: 13px; }

.browse-state {
  text-align: center;
  padding: 40px 0;
  color: var(--text-secondary);
  font-size: 14px;
}
.browse-state.error { color: var(--danger); }

/* CPAL мик баннер ошибки */
.mic-error-banner {
  position: fixed;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--danger);
  border-radius: var(--radius-item);
  padding: 10px 16px;
  color: var(--danger);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  z-index: 1000;
  cursor: pointer;
  max-width: 500px;
  text-align: center;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}
</style>
