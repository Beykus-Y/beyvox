<template>
  <div class="app-layout">
    <!-- Уровень 1: инстансы beyvox-server -->
    <ServerSidebar
      :servers="serversStore.servers"
      :active-url="serversStore.activeUrl"
      @select-server="switchServer"
      @add-server="openAddServer"
      @remove-server="serversStore.removeServer"
      @disconnect-server="disconnectServer"
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
      @invite-guild="openInviteCreate"
      @join-invite="showInvite = true"
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
      @set-volume="() => {}"
      @create-channel="openCreateChannel"
      @toggle="channelSidebarOpen = !channelSidebarOpen"
    />

    <!-- Ошибка микрофона -->
    <div v-if="voice.micError" class="mic-error-banner" @click="voice.micError = ''">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/></svg>
      {{ voice.micError }}
    </div>

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

    <!-- Создать инвайт -->
    <div v-if="showCreateInvite" class="modal-overlay" @click.self="showCreateInvite = false">
      <div class="modal">
        <h3>Пригласить на сервер</h3>
        <p style="color:var(--text2);font-size:13px;margin-bottom:12px">Поделись этим кодом — человек вводит его в «Войти по инвайту»</p>
        <div v-if="createdInviteCode" class="invite-code-box">
          <span class="invite-code">{{ createdInviteCode }}</span>
          <button class="btn-secondary" @click="copyInviteCode">{{ inviteCopied ? 'Скопировано!' : 'Копировать' }}</button>
        </div>
        <p v-if="serverError" class="modal-error">{{ serverError }}</p>
        <div class="modal-actions">
          <button class="btn-ghost" @click="showCreateInvite = false">Закрыть</button>
          <button class="btn-primary" :disabled="serverLoading" @click="() => createInvite()">
            {{ serverLoading ? '...' : 'Новый инвайт' }}
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
            <div class="settings-field">
              <label>Access Token</label>
              <div class="token-row">
                <input class="token-input" type="password" :value="auth.accessToken" readonly />
                <button class="btn-secondary" @click="copyToken">{{ tokenCopied ? 'Скопировано!' : 'Копировать' }}</button>
              </div>
            </div>
            <div class="settings-divider" />
            <button class="btn-danger" @click="logout">Выйти из аккаунта</button>
          </template>

          <template v-if="activeSettingsTab === 'audio'">
            <div class="settings-field">
              <label>Микрофон</label>
              <select :value="voice.selectedInputCpalName" @change="voice.setInputCpalName(($event.target as HTMLSelectElement).value)">
                <option value="">По умолчанию (Windows)</option>
                <option v-for="d in inputDevices" :key="d.id" :value="d.id">{{ d.name }}</option>
              </select>
            </div>
            <div class="settings-field">
              <label>Динамики / Наушники</label>
              <select :value="voice.selectedOutputCpalName" @change="voice.setOutputCpalName(($event.target as HTMLSelectElement).value)">
                <option value="">По умолчанию (Windows)</option>
                <option v-for="d in outputDevices" :key="d.id" :value="d.id">{{ d.name }}</option>
              </select>
            </div>
            <div class="settings-field">
              <label>Громкость микрофона — {{ voice.micVolume }}%</label>
              <input
                type="range" min="0" max="200" step="5"
                :value="voice.micVolume"
                @input="voice.setMicVolume(+($event.target as HTMLInputElement).value)"
                class="volume-slider"
              />
            </div>
            <div class="settings-field">
              <label>Громкость воспроизведения — {{ voice.playbackVolume }}%</label>
              <input
                type="range" min="0" max="200" step="5"
                :value="voice.playbackVolume"
                @input="voice.setPlaybackVolume(+($event.target as HTMLInputElement).value)"
                class="volume-slider"
              />
            </div>
            <div class="settings-field">
              <label>Тест микрофона</label>
              <button
                class="mic-test-btn"
                :class="{ active: voice.isMicTesting }"
                @click="voice.isMicTesting ? voice.stopMicTest() : voice.startMicTest()"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 14a3 3 0 0 0 3-3V5a3 3 0 0 0-6 0v6a3 3 0 0 0 3 3zm5-3a5 5 0 0 1-10 0H5a7 7 0 0 0 6 6.93V21h2v-3.07A7 7 0 0 0 19 11h-2z"/>
                </svg>
                {{ voice.isMicTesting ? 'Остановить тест (слышишь себя)' : 'Проверить микрофон' }}
              </button>
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
              </div>
            </div>
            <div v-if="voice.voiceMode === 'ptt'" class="settings-field">
              <label>Клавиша PTT</label>
              <button class="ptt-key-btn" :class="{ recording: recordingPttKey }" @click="startRecordingPttKey">
                {{ recordingPttKey ? 'Нажми клавишу...' : formatKeyCode(voice.pttKey) }}
              </button>
              <span class="settings-hint-sm">Удерживай клавишу чтобы говорить</span>
            </div>
          </template>

          <template v-if="activeSettingsTab === 'effects'">
            <p class="settings-hint">Встроенные эффекты применяются к микрофону до OPUS кодека. Порядок: Gate → Compressor → EQ.</p>

            <!-- Noise Gate -->
            <div class="effect-section">
              <div class="effect-header">
                <div class="effect-title">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3A4.5 4.5 0 0 0 14 7.97v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/></svg>
                  Noise Gate
                </div>
                <label class="fx-toggle">
                  <input type="checkbox" v-model="fx.gate.enabled" @change="applyGate" />
                  <span class="fx-toggle-track"></span>
                </label>
              </div>
              <div v-if="fx.gate.enabled" class="effect-params">
                <div class="settings-field">
                  <label>Порог — {{ fx.gate.threshold_db }} dB</label>
                  <input type="range" min="-80" max="-10" step="1" v-model.number="fx.gate.threshold_db" @input="applyGate" class="volume-slider" />
                </div>
                <div class="fx-row">
                  <div class="settings-field">
                    <label>Атака — {{ fx.gate.attack_ms }} мс</label>
                    <input type="range" min="1" max="50" step="1" v-model.number="fx.gate.attack_ms" @input="applyGate" class="volume-slider" />
                  </div>
                  <div class="settings-field">
                    <label>Спад — {{ fx.gate.release_ms }} мс</label>
                    <input type="range" min="10" max="1000" step="10" v-model.number="fx.gate.release_ms" @input="applyGate" class="volume-slider" />
                  </div>
                </div>
              </div>
            </div>

            <!-- Compressor -->
            <div class="effect-section">
              <div class="effect-header">
                <div class="effect-title">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 14l-5-5 1.41-1.41L12 14.17l7.59-7.59L21 8l-9 9z"/></svg>
                  Compressor / Limiter
                </div>
                <label class="fx-toggle">
                  <input type="checkbox" v-model="fx.comp.enabled" @change="applyCompressor" />
                  <span class="fx-toggle-track"></span>
                </label>
              </div>
              <div v-if="fx.comp.enabled" class="effect-params">
                <div class="fx-row">
                  <div class="settings-field">
                    <label>Порог — {{ fx.comp.threshold_db }} dB</label>
                    <input type="range" min="-60" max="0" step="1" v-model.number="fx.comp.threshold_db" @input="applyCompressor" class="volume-slider" />
                  </div>
                  <div class="settings-field">
                    <label>Степень — {{ fx.comp.ratio }}:1</label>
                    <input type="range" min="1" max="20" step="0.5" v-model.number="fx.comp.ratio" @input="applyCompressor" class="volume-slider" />
                  </div>
                </div>
                <div class="fx-row">
                  <div class="settings-field">
                    <label>Атака — {{ fx.comp.attack_ms }} мс</label>
                    <input type="range" min="1" max="100" step="1" v-model.number="fx.comp.attack_ms" @input="applyCompressor" class="volume-slider" />
                  </div>
                  <div class="settings-field">
                    <label>Спад — {{ fx.comp.release_ms }} мс</label>
                    <input type="range" min="10" max="500" step="10" v-model.number="fx.comp.release_ms" @input="applyCompressor" class="volume-slider" />
                  </div>
                </div>
                <div class="settings-field">
                  <label>Компенсация — {{ fx.comp.makeup_db > 0 ? '+' : '' }}{{ fx.comp.makeup_db }} dB</label>
                  <input type="range" min="-12" max="24" step="0.5" v-model.number="fx.comp.makeup_db" @input="applyCompressor" class="volume-slider" />
                </div>
              </div>
            </div>

            <!-- 3-band EQ -->
            <div class="effect-section">
              <div class="effect-header">
                <div class="effect-title">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M10 20h4V4h-4v16zm-6 0h4v-8H4v8zM16 9v11h4V9h-4z"/></svg>
                  EQ (3-полосный)
                </div>
                <label class="fx-toggle">
                  <input type="checkbox" v-model="fx.eq.enabled" @change="applyEq" />
                  <span class="fx-toggle-track"></span>
                </label>
              </div>
              <div v-if="fx.eq.enabled" class="effect-params">
                <div class="fx-row">
                  <div class="settings-field">
                    <label>Низы 200Hz — {{ fx.eq.low_db > 0 ? '+' : '' }}{{ fx.eq.low_db }} dB</label>
                    <input type="range" min="-18" max="18" step="0.5" v-model.number="fx.eq.low_db" @input="applyEq" class="volume-slider" />
                  </div>
                  <div class="settings-field">
                    <label>Мид {{ fx.eq.mid_freq }}Hz — {{ fx.eq.mid_db > 0 ? '+' : '' }}{{ fx.eq.mid_db }} dB</label>
                    <input type="range" min="-18" max="18" step="0.5" v-model.number="fx.eq.mid_db" @input="applyEq" class="volume-slider" />
                  </div>
                  <div class="settings-field">
                    <label>Верх 8kHz — {{ fx.eq.high_db > 0 ? '+' : '' }}{{ fx.eq.high_db }} dB</label>
                    <input type="range" min="-18" max="18" step="0.5" v-model.number="fx.eq.high_db" @input="applyEq" class="volume-slider" />
                  </div>
                </div>
                <div class="settings-field">
                  <label>Частота мида — {{ fx.eq.mid_freq }} Hz</label>
                  <input type="range" min="200" max="6000" step="50" v-model.number="fx.eq.mid_freq" @input="applyEq" class="volume-slider" />
                </div>
              </div>
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
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
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
const showCreateInvite = ref(false)
const createdInviteCode = ref('')
const inviteCopied = ref(false)
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

// Инвайт
const inviteCodeInput = ref('')

// Загрузка сообщений
const messagesLoading = ref(false)

// Состояние сайдбаров
const guildSidebarOpen = ref(true)
const channelSidebarOpen = ref(true)

// Settings
const activeSettingsTab = ref('account')
const tokenCopied = ref(false)
const settingsTabs = [
  { id: 'account', label: 'Аккаунт' },
  { id: 'audio', label: 'Аудио' },
  { id: 'effects', label: 'Эффекты' },
  { id: 'vst', label: 'VST плагины' },
]

// Built-in effects state
const fx = reactive({
  gate: { enabled: false, threshold_db: -40, attack_ms: 5, release_ms: 200 },
  comp: { enabled: false, threshold_db: -18, ratio: 4, attack_ms: 5, release_ms: 100, makeup_db: 0 },
  eq: { enabled: false, low_db: 0, mid_db: 0, mid_freq: 1000, high_db: 0 },
})
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
  voice.initVoiceSettings()
  initEffects()
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
    localStorage.setItem('voice_ptt_key', e.code)
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

// === Built-in effects ===
function initEffects() {
  const savedGate = localStorage.getItem('fx_gate')
  const savedComp = localStorage.getItem('fx_comp')
  const savedEq = localStorage.getItem('fx_eq')
  if (savedGate) Object.assign(fx.gate, JSON.parse(savedGate))
  if (savedComp) Object.assign(fx.comp, JSON.parse(savedComp))
  if (savedEq) Object.assign(fx.eq, JSON.parse(savedEq))
  applyGate()
  applyCompressor()
  applyEq()
}

async function applyGate() {
  localStorage.setItem('fx_gate', JSON.stringify(fx.gate))
  await invoke('set_noise_gate', {
    enabled: fx.gate.enabled,
    thresholdDb: fx.gate.threshold_db,
    attackMs: fx.gate.attack_ms,
    releaseMs: fx.gate.release_ms,
  }).catch(() => {})
}

async function applyCompressor() {
  localStorage.setItem('fx_comp', JSON.stringify(fx.comp))
  await invoke('set_compressor', {
    enabled: fx.comp.enabled,
    thresholdDb: fx.comp.threshold_db,
    ratio: fx.comp.ratio,
    attackMs: fx.comp.attack_ms,
    releaseMs: fx.comp.release_ms,
    makeupGainDb: fx.comp.makeup_db,
  }).catch(() => {})
}

async function applyEq() {
  localStorage.setItem('fx_eq', JSON.stringify(fx.eq))
  await invoke('set_eq', {
    enabled: fx.eq.enabled,
    lowGainDb: fx.eq.low_db,
    midGainDb: fx.eq.mid_db,
    midFreq: fx.eq.mid_freq,
    highGainDb: fx.eq.high_db,
  }).catch(() => {})
}

// === Переключение серверов ===
async function switchServer(url: string) {
  ws.disconnect()
  guild.reset()
  serversStore.setActive(url)
  guild.connectToServer(url)
  ws.connect(url)
}

function disconnectServer(url: string) {
  if (serversStore.activeUrl === url) {
    ws.disconnect()
    guild.reset()
    serversStore.setActive(null)
  }
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
    if (e?.response?.status === 403) serverError.value = 'Нет прав на создание серверов'
    else serverError.value = msg || 'Ошибка создания'
  } finally {
    serverLoading.value = false
  }
}

// === Инвайт ===
async function openInviteCreate(guildId: string) {
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
    serverError.value = 'Не удалось создать инвайт'
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

async function joinVoice(channel: any) {
  if (!guild.activeGuildId) return
  if (voice.activeChannelId === channel.id) {
    ws.joinVoiceChannel(guild.activeGuildId, null)
    voice.disconnect()
  } else {
    // Вызываем прямо из click-хэндлера — до async WS round-trip, чтобы AudioContext разблокировался
    await voice.prewarmAudio()
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
async function copyToken() {
  await navigator.clipboard.writeText(auth.accessToken)
  tokenCopied.value = true
  setTimeout(() => { tokenCopied.value = false }, 2000)
}

async function openSettings() {
  showSettings.value = true
  activeSettingsTab.value = 'account'
  try {
    inputDevices.value = await invoke<{ id: string; name: string }[]>('list_input_devices')
    outputDevices.value = await invoke<{ id: string; name: string }[]>('list_output_devices')
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

.volume-slider {
  width: 100%;
  accent-color: var(--accent);
  cursor: pointer;
  height: 4px;
}

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

.mic-error-banner {
  position: fixed;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(242, 63, 67, 0.15);
  border: 1px solid var(--red);
  border-radius: 8px;
  padding: 10px 16px;
  color: var(--red);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  z-index: 1000;
  cursor: pointer;
  max-width: 500px;
  text-align: center;
}

.btn-danger {
  padding: 10px 18px; border-radius: 6px;
  background: var(--red); color: #fff;
  font-weight: 600; font-size: 13px; align-self: flex-start;
}
.btn-danger:hover { opacity: 0.85; }

.invite-code-box { display: flex; gap: 8px; align-items: center; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 10px 14px; margin-bottom: 4px; }
.invite-code { flex: 1; font-family: 'JetBrains Mono', monospace; font-size: 18px; font-weight: 700; color: var(--accent); letter-spacing: 2px; }
.token-row { display: flex; gap: 8px; }
.token-input { flex: 1; background: var(--bg-dark); border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; color: var(--text1); font-family: 'JetBrains Mono', monospace; font-size: 11px; }
.btn-secondary { background: var(--bg-hover); border: 1px solid var(--border); border-radius: 6px; color: var(--text1); padding: 6px 12px; cursor: pointer; white-space: nowrap; }
.btn-secondary:hover { background: var(--bg-active); }

.mic-test-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-light);
  color: var(--text2);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  align-self: flex-start;
}
.mic-test-btn:hover { border-color: var(--accent); color: var(--text); }
.mic-test-btn.active { border-color: var(--green, #23a55a); background: rgba(35, 165, 90, 0.12); color: var(--green, #23a55a); }

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

/* Effects tab */
.effect-section {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.effect-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 14px;
  background: var(--bg-light);
}
.effect-title {
  display: flex; align-items: center; gap: 8px;
  font-size: 13px; font-weight: 600; color: var(--text);
}
.effect-params {
  padding: 12px 14px;
  display: flex; flex-direction: column; gap: 10px;
  background: var(--bg-dark);
  border-top: 1px solid var(--border);
}
.fx-row {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 12px;
}

/* Toggle switch */
.fx-toggle { position: relative; display: inline-flex; align-items: center; cursor: pointer; }
.fx-toggle input { display: none; }
.fx-toggle-track {
  width: 36px; height: 20px; border-radius: 10px;
  background: var(--bg-hover); border: 1px solid var(--border);
  position: relative; transition: background 0.2s, border-color 0.2s;
}
.fx-toggle-track::after {
  content: ''; position: absolute;
  top: 2px; left: 2px;
  width: 14px; height: 14px; border-radius: 50%;
  background: var(--text2); transition: transform 0.2s, background 0.2s;
}
.fx-toggle input:checked + .fx-toggle-track {
  background: var(--accent); border-color: var(--accent);
}
.fx-toggle input:checked + .fx-toggle-track::after {
  transform: translateX(16px); background: #fff;
}

.btn-add-vst {
  display: flex; align-items: center; gap: 6px;
  padding: 8px 16px; border-radius: 6px;
  background: var(--bg-light); border: 1px solid var(--border);
  color: var(--text); font-size: 13px; font-weight: 600;
  align-self: flex-start; cursor: pointer;
}
.btn-add-vst:hover { background: var(--bg-hover); border-color: var(--accent); color: var(--accent); }
</style>
