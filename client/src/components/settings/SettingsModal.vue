<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal settings-modal">
      <!-- Сайдбар настроек -->
      <div class="settings-sidebar">
        <div
          v-for="tab in settingsTabs"
          :key="tab.id"
          class="settings-tab"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </div>
      </div>

      <!-- Контент настроек -->
      <div class="settings-content">
        <button class="settings-close" @click="$emit('close')" title="Закрыть настройки">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>

        <!-- ВКЛАДКА: АККАУНТ -->
        <template v-if="activeTab === 'account'">
          <h2>Настройки аккаунта</h2>
          <div class="settings-divider" />
          
          <div class="settings-row">
            <div class="settings-avatar">{{ auth.username[0]?.toUpperCase() }}</div>
            <div class="settings-userinfo">
              <div class="settings-username">{{ auth.username }}</div>
              <div class="settings-uuid">{{ auth.userId }}</div>
            </div>
          </div>
          
          <div class="settings-divider" />
          
          <div class="settings-field">
            <label>Токен доступа</label>
            <div class="token-row">
              <input class="token-input" type="password" :value="auth.accessToken" readonly />
              <button class="btn-secondary" @click="copyToken">
                {{ tokenCopied ? 'Скопировано!' : 'Копировать' }}
              </button>
            </div>
          </div>
          
          <div class="settings-divider" />
          
          <button class="btn-danger" @click="logout">Выйти из аккаунта</button>
        </template>

        <!-- ВКЛАДКА: АУДИО -->
        <template v-if="activeTab === 'audio'">
          <h2>Настройки звука</h2>
          <div class="settings-divider" />

          <div class="settings-field">
            <label>Устройство ввода (Микрофон)</label>
            <select :value="voice.selectedInputCpalName" @change="changeInputDevice">
              <option value="">По умолчанию (Windows)</option>
              <option v-for="d in inputDevices" :key="d.id" :value="d.id">{{ d.name }}</option>
            </select>
          </div>

          <div class="settings-field">
            <label>Устройство вывода (Динамики)</label>
            <select :value="voice.selectedOutputCpalName" @change="changeOutputDevice">
              <option value="">По умолчанию (Windows)</option>
              <option v-for="d in outputDevices" :key="d.id" :value="d.id">{{ d.name }}</option>
            </select>
          </div>

          <div class="settings-field">
            <label>Громкость микрофона — {{ voice.micVolume }}%</label>
            <input
              type="range" min="0" max="200" step="5"
              :value="voice.micVolume"
              @input="changeMicVolume"
              class="volume-slider"
            />
          </div>

          <div class="settings-field">
            <label>Громкость воспроизведения — {{ voice.playbackVolume }}%</label>
            <input
              type="range" min="0" max="200" step="5"
              :value="voice.playbackVolume"
              @input="changePlaybackVolume"
              class="volume-slider"
            />
          </div>

          <div class="settings-field">
            <label>Проверка микрофона</label>
            <button
              class="mic-test-btn"
              :class="{ active: voice.isMicTesting }"
              @click="toggleMicTest"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 14a3 3 0 0 0 3-3V5a3 3 0 0 0-6 0v6a3 3 0 0 0 3 3zm5-3a5 5 0 0 1-10 0H5a7 7 0 0 0 6 6.93V21h2v-3.07A7 7 0 0 0 19 11h-2z"/>
              </svg>
              {{ voice.isMicTesting ? 'Остановить проверку (слышишь себя)' : 'Проверить микрофон' }}
            </button>
          </div>

          <div class="settings-field">
            <label>Режим активации</label>
            <div class="voice-mode-group">
              <label class="voice-mode-opt" :class="{ active: voice.voiceMode === 'open' }">
                <input type="radio" v-model="voice.voiceMode" value="open" />
                Голосовая активация
              </label>
              <label class="voice-mode-opt" :class="{ active: voice.voiceMode === 'ptt' }">
                <input type="radio" v-model="voice.voiceMode" value="ptt" />
                Push-to-Talk (По кнопке)
              </label>
            </div>
          </div>

          <div v-if="voice.voiceMode === 'ptt'" class="settings-field">
            <label>Клавиша активации PTT</label>
            <button class="ptt-key-btn" :class="{ recording: recordingPttKey }" @click="startRecordingPttKey">
              {{ recordingPttKey ? 'Нажмите клавишу...' : formatKeyCode(voice.pttKey) }}
            </button>
            <span class="settings-hint-sm">Удерживайте кнопку для разговора</span>
          </div>
        </template>

        <!-- ВКЛАДКА: ЭФФЕКТЫ -->
        <template v-if="activeTab === 'effects'">
          <h2>Встроенные эффекты</h2>
          <p class="settings-hint">Эффекты применяются к микрофону до кодирования Opus. Цепочка: Gate → Compressor → EQ.</p>
          <div class="settings-divider" />

          <!-- Noise Gate -->
          <div class="effect-section">
            <div class="effect-header">
              <div class="effect-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3A4.5 4.5 0 0 0 14 7.97v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/></svg>
                Noise Gate (Шумоподавление)
              </div>
              <label class="fx-toggle">
                <input type="checkbox" v-model="fx.gate.enabled" @change="applyGate" />
                <span class="fx-toggle-track"></span>
              </label>
            </div>
            <div v-if="fx.gate.enabled" class="effect-params">
              <div class="settings-field">
                <label>Порог срабатывания — {{ fx.gate.threshold_db }} dB</label>
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
                Compressor / Limiter (Сжатие звука)
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
                  <label>Степень сжатия — {{ fx.comp.ratio }}:1</label>
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
                <label>Гейн компенсации — {{ fx.comp.makeup_db > 0 ? '+' : '' }}{{ fx.comp.makeup_db }} dB</label>
                <input type="range" min="-12" max="24" step="0.5" v-model.number="fx.comp.makeup_db" @input="applyCompressor" class="volume-slider" />
              </div>
            </div>
          </div>

          <!-- 3-band EQ -->
          <div class="effect-section">
            <div class="effect-header">
              <div class="effect-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M10 20h4V4h-4v16zm-6 0h4v-8H4v8zM16 9v11h4V9h-4z"/></svg>
                EQ (3-полосный Эквалайзер)
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
                <label>Частота фильтра средних — {{ fx.eq.mid_freq }} Hz</label>
                <input type="range" min="200" max="6000" step="50" v-model.number="fx.eq.mid_freq" @input="applyEq" class="volume-slider" />
              </div>
            </div>
          </div>
        </template>

        <!-- ВКЛАДКА: VST -->
        <template v-if="activeTab === 'vst'">
          <h2>VST Плагины</h2>
          <p class="settings-hint">VST3-эффекты для обработки голоса перед отправкой в LiveKit. Поддерживаются файлы .dll, .so, .vst3</p>
          <div class="settings-divider" />

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
              <button class="btn-icon-danger" @click="removeVst(i)" title="Удалить плагин">✕</button>
            </div>
          </div>
          
          <p v-if="vstError" class="vst-error-msg">{{ vstError }}</p>
          
          <button class="btn-add-vst" @click="addVst">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
            Добавить плагин
          </button>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openFile } from '@tauri-apps/plugin-dialog'
import { useAuthStore } from '../../stores/auth'
import { useVoiceStore } from '../../stores/voice'

const emit = defineEmits(['close'])

const auth = useAuthStore()
const voice = useVoiceStore()

const activeTab = ref('account')
const tokenCopied = ref(false)
const recordingPttKey = ref(false)

const inputDevices = ref<{ id: string; name: string }[]>([])
const outputDevices = ref<{ id: string; name: string }[]>([])

const settingsTabs = [
  { id: 'account', label: 'Аккаунт' },
  { id: 'audio', label: 'Аудио' },
  { id: 'effects', label: 'Эффекты' },
  { id: 'vst', label: 'VST плагины' }
]

// Built-in effects reactive state
const fx = reactive({
  gate: { enabled: false, threshold_db: -40, attack_ms: 5, release_ms: 200 },
  comp: { enabled: false, threshold_db: -18, ratio: 4, attack_ms: 5, release_ms: 100, makeup_db: 0 },
  eq: { enabled: false, low_db: 0, mid_db: 0, mid_freq: 1000, high_db: 0 }
})

interface VstInfo {
  path: string
  name: string
  vendor: string
  version: string
  num_inputs: number
  num_outputs: number
}
const vstPlugins = ref<VstInfo[]>([])
const vstError = ref('')

onMounted(async () => {
  try {
    inputDevices.value = await invoke<{ id: string; name: string }[]>('list_input_devices')
    outputDevices.value = await invoke<{ id: string; name: string }[]>('list_output_devices')
    
    // Загрузка плагинов
    const savedVst = localStorage.getItem('vst_plugins')
    if (savedVst) {
      vstPlugins.value = JSON.parse(savedVst)
    }
  } catch (e) {
    console.error('Ошибка инициализации настроек аудио:', e)
  }

  // Загружаем эффекты
  const savedGate = localStorage.getItem('fx_gate')
  const savedComp = localStorage.getItem('fx_comp')
  const savedEq = localStorage.getItem('fx_eq')
  if (savedGate) Object.assign(fx.gate, JSON.parse(savedGate))
  if (savedComp) Object.assign(fx.comp, JSON.parse(savedComp))
  if (savedEq) Object.assign(fx.eq, JSON.parse(savedEq))
})

async function copyToken() {
  await navigator.clipboard.writeText(auth.accessToken)
  tokenCopied.value = true
  setTimeout(() => { tokenCopied.value = false }, 2000)
}

function logout() {
  auth.logout()
  emit('close')
  window.location.reload()
}

function changeInputDevice(e: Event) {
  const val = (e.target as HTMLSelectElement).value
  voice.setInputCpalName(val)
}

function changeOutputDevice(e: Event) {
  const val = (e.target as HTMLSelectElement).value
  voice.setOutputCpalName(val)
}

function changeMicVolume(e: Event) {
  const val = parseInt((e.target as HTMLInputElement).value)
  voice.setMicVolume(val)
}

function changePlaybackVolume(e: Event) {
  const val = parseInt((e.target as HTMLInputElement).value)
  voice.setPlaybackVolume(val)
}

function toggleMicTest() {
  if (voice.isMicTesting) {
    voice.stopMicTest()
  } else {
    voice.startMicTest()
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

// Built-in effects apply functions
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

// VST Plugins handling
async function addVst() {
  vstError.value = ''
  const path = await openFile({
    title: 'Выбери VST плагин',
    filters: [{ name: 'VST Plugin', extensions: ['dll', 'so', 'vst3'] }],
    multiple: false,
  })
  if (!path || typeof path !== 'string') return
  try {
    const info = await invoke<VstInfo>('load_vst_info', { path })
    vstPlugins.value.push(info)
    localStorage.setItem('vst_plugins', JSON.stringify(vstPlugins.value))
  } catch (e: any) {
    vstError.value = 'Не удалось загрузить плагин: ' + String(e)
  }
}

function removeVst(i: number) {
  vstPlugins.value.splice(i, 1)
  localStorage.setItem('vst_plugins', JSON.stringify(vstPlugins.value))
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  display: flex;
  overflow: hidden;
}

.settings-modal {
  width: 760px;
  max-width: 95vw;
  height: 520px;
  max-height: 90vh;
  flex-direction: row;
}

.settings-sidebar {
  width: 180px;
  background: var(--bg-app);
  border-right: 1px solid var(--border);
  padding: 24px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.settings-tab {
  padding: 8px 12px;
  border-radius: var(--radius-item);
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
  transition: all 0.1s;
}
.settings-tab:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.settings-tab.active {
  background: var(--bg-active);
  color: var(--accent);
  font-weight: 600;
}

.settings-content {
  flex: 1;
  padding: 32px;
  overflow-y: auto;
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.settings-content h2 {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: -10px;
}

.settings-close {
  position: absolute;
  top: 16px;
  right: 16px;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.1s;
}
.settings-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.settings-row {
  display: flex;
  align-items: center;
  gap: 14px;
}

.settings-avatar {
  width: 54px;
  height: 54px;
  border-radius: 50%;
  background: var(--accent-grad);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
}

.settings-userinfo {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.settings-username {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
}

.settings-uuid {
  font-size: 11px;
  color: var(--text-muted);
  font-family: monospace;
}

.settings-divider {
  height: 1px;
  background: var(--border);
  flex-shrink: 0;
}

.settings-hint {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}

.settings-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.settings-field label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.settings-field select {
  cursor: pointer;
}
.settings-field select option {
  background: var(--bg-panel);
}

.token-row {
  display: flex;
  gap: 8px;
}

.token-input {
  flex: 1;
  font-family: monospace;
  font-size: 12px;
}

.btn-secondary {
  background: var(--bg-hover);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  padding: 0 16px;
  font-weight: 600;
}
.btn-secondary:hover {
  background: var(--bg-active);
  border-color: var(--accent);
}

.btn-danger {
  background: var(--danger);
  color: #fff;
  font-weight: 600;
  padding: 10px 16px;
  border-radius: var(--radius-item);
  align-self: flex-start;
}
.btn-danger:hover {
  opacity: 0.9;
}

.volume-slider {
  width: 100%;
  accent-color: var(--accent);
  cursor: pointer;
  height: 4px;
}

.mic-test-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: var(--radius-item);
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  font-weight: 600;
  align-self: flex-start;
}
.mic-test-btn:hover {
  border-color: var(--accent);
  background: var(--bg-hover);
}
.mic-test-btn.active {
  border-color: var(--online);
  background: rgba(35, 197, 94, 0.08);
  color: var(--online);
}

.voice-mode-group {
  display: flex;
  gap: 10px;
}

.voice-mode-opt {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  cursor: pointer;
  font-weight: 600;
  transition: all 0.15s;
}
.voice-mode-opt input {
  display: none;
}
.voice-mode-opt:hover {
  background: var(--bg-hover);
}
.voice-mode-opt.active {
  border-color: var(--accent);
  background: rgba(124, 108, 255, 0.08);
  color: var(--accent);
}

.ptt-key-btn {
  padding: 10px 16px;
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  border-radius: var(--radius-item);
  font-weight: 700;
  font-family: monospace;
  align-self: flex-start;
}
.ptt-key-btn.recording {
  border-color: var(--accent);
  background: rgba(124, 108, 255, 0.08);
  animation: pulse 1s infinite;
}

.settings-hint-sm {
  font-size: 10px;
  color: var(--text-muted);
}

/* Эффекты */
.effect-section {
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  overflow: hidden;
}

.effect-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-app);
}

.effect-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 700;
  font-size: 13px;
}

.effect-params {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  background: var(--bg-panel);
  border-top: 1px solid var(--border);
}

.fx-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.fx-toggle {
  position: relative;
  display: inline-flex;
  align-items: center;
  cursor: pointer;
}
.fx-toggle input {
  display: none;
}
.fx-toggle-track {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  position: relative;
  transition: all 0.2s;
}
.fx-toggle-track::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--text-secondary);
  transition: all 0.2s;
}
.fx-toggle input:checked + .fx-toggle-track {
  background: var(--accent);
  border-color: var(--accent);
}
.fx-toggle input:checked + .fx-toggle-track::after {
  transform: translateX(16px);
  background: #fff;
}

/* VST */
.vst-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.vst-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 32px;
  border: 1px dashed var(--border);
  border-radius: var(--radius-card);
  color: var(--text-secondary);
}

.vst-item {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--bg-app);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  padding: 12px;
}

.vst-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--bg-hover);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
}

.vst-info {
  flex: 1;
  min-width: 0;
}

.vst-name {
  font-weight: 700;
  font-size: 13px;
}

.vst-path {
  font-size: 10px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
}

.btn-icon-danger {
  color: var(--text-secondary);
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-icon-danger:hover {
  background: rgba(239, 68, 68, 0.1);
  color: var(--danger);
}

.vst-error-msg {
  color: var(--danger);
  font-size: 12px;
  background: rgba(239, 68, 68, 0.08);
  padding: 8px 12px;
  border-radius: 6px;
}

.btn-add-vst {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: var(--radius-item);
  background: var(--bg-hover);
  border: 1px solid var(--border);
  font-weight: 600;
  align-self: flex-start;
}
.btn-add-vst:hover {
  border-color: var(--accent);
  color: var(--accent);
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.6; }
  100% { opacity: 1; }
}
</style>
