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

          <!-- Визуализация — sticky, всегда видна при скролле -->
          <div class="viz-panel">
            <div class="viz-header">
              <div class="viz-title-row">
                <span>Осциллограф</span>
                <span class="viz-sep-inline">·</span>
                <span>FFT</span>
              </div>
              <span :class="['viz-dot', { active: vizActive }]">{{ vizActive ? '● живой захват' : '○ нет захвата' }}</span>
            </div>

            <div class="viz-canvases">
              <canvas ref="waveCanvas" class="viz-wave"></canvas>
              <div class="viz-divider"></div>
              <canvas ref="specCanvas" class="viz-spec"></canvas>
            </div>
            <div class="viz-freq-labels">
              <span>20</span><span>100</span><span>500</span><span>1k</span><span>4k</span><span>10k</span><span>20k Hz</span>
            </div>

            <!-- Dual IN/OUT meter -->
            <div class="dual-meter">
              <div class="dual-meter-row">
                <span class="dm-label">IN</span>
                <div class="dm-track">
                  <div class="dm-fill dm-in" :style="{ width: inPct + '%' }"></div>
                  <div v-if="fx.gate.enabled" class="dm-pin dm-pin-gate" :style="{ left: gatePinPct + '%' }" title="Gate threshold"></div>
                  <div v-if="fx.comp.enabled" class="dm-pin dm-pin-comp" :style="{ left: compPinPct + '%' }" title="Compressor threshold"></div>
                </div>
                <span class="dm-val">{{ levelDbStr }}</span>
              </div>
              <div class="dual-meter-row">
                <span class="dm-label">OUT</span>
                <div class="dm-track">
                  <div class="dm-fill dm-out" :style="{ width: outPct + '%' }"></div>
                  <div v-if="fx.gate.enabled" class="dm-pin dm-pin-gate" :style="{ left: gatePinPct + '%' }"></div>
                  <div v-if="fx.comp.enabled" class="dm-pin dm-pin-comp" :style="{ left: compPinPct + '%' }"></div>
                </div>
                <span class="dm-val" :class="{ 'dm-gated': isGated }">{{ isGated ? 'GATED' : outDbStr }}</span>
              </div>
              <div class="dm-legend">
                <span class="dm-leg-gate" v-if="fx.gate.enabled">│ Gate</span>
                <span class="dm-leg-comp" v-if="fx.comp.enabled">│ Comp</span>
              </div>
            </div>
          </div>

          <!-- RNNoise -->
          <div class="effect-section">
            <div class="effect-header">
              <div class="effect-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14H9V8h2v8zm4 0h-2V8h2v8z"/></svg>
                RNNoise — AI шумоподавление
                <span class="fx-badge">AI</span>
              </div>
              <label class="fx-toggle">
                <input type="checkbox" v-model="fx.rnnoise" @change="applyRnnoise" />
                <span class="fx-toggle-track"></span>
              </label>
            </div>
            <div v-if="fx.rnnoise" class="effect-params">
              <p class="settings-hint">Нейросеть RNNoise удаляет фоновый шум (вентиляторы, клавиатура, улица). Работает на 48kHz, задержка ~10мс. Применяется до Gate и Compressor.</p>
            </div>
          </div>

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
              <!-- Gain Reduction meter -->
              <div class="meter-row">
                <span class="meter-side-label">GR</span>
                <div class="gr-track">
                  <div class="gr-fill" :style="{ width: grPct + '%' }"></div>
                </div>
                <span class="meter-db-val gr-val">{{ grDb > 0 ? '-' + grDb.toFixed(1) + ' dB' : '0 dB' }}</span>
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
import { ref, reactive, computed, watch, onMounted, onUnmounted } from 'vue'
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

// ── Visualization ─────────────────────────────────────────────────────────────
const waveCanvas = ref<HTMLCanvasElement | null>(null)
const specCanvas  = ref<HTMLCanvasElement | null>(null)
const vizActive   = ref(false)
const currentDb = ref(-80)   // входной уровень из Web Audio (для осциллографа/FFT)
const realInDb  = ref(-80)   // входной уровень из Rust (до эффектов)
const realOutDb = ref(-80)   // выходной уровень из Rust (после всех эффектов) — реальный!

const DB_MIN = -80
const DB_MAX = 0

const dbToPct = (db: number) =>
  Math.max(0, Math.min(100, ((db - DB_MIN) / (DB_MAX - DB_MIN)) * 100))

const levelDbStr = computed(() =>
  realInDb.value > DB_MIN + 1 ? realInDb.value.toFixed(1) + ' dB' : '-∞'
)
const inPct  = computed(() => dbToPct(realInDb.value))
const outPct = computed(() => dbToPct(realOutDb.value))
const outDbStr = computed(() =>
  realOutDb.value > DB_MIN + 1 ? realOutDb.value.toFixed(1) + ' dB' : '-∞'
)

// Gate: реально заглушён если OUT намного тише IN (≥30 dB разница при активном gate)
const isGated = computed(() =>
  fx.gate.enabled && (realInDb.value - realOutDb.value) > 30
)

const gatePinPct = computed(() => dbToPct(fx.gate.threshold_db))
const compPinPct = computed(() => dbToPct(fx.comp.threshold_db))

// GR из реальных уровней (а не математики)
const grDb  = computed(() => Math.max(0, realInDb.value - realOutDb.value))
const grPct = computed(() => Math.min(100, (grDb.value / 30) * 100))

let audioCtx: AudioContext | null = null
let analyserNode: AnalyserNode | null = null
let micStream: MediaStream | null = null
let rafId: number | null = null
let levelsInterval: ReturnType<typeof setInterval> | null = null

async function startViz() {
  if (vizActive.value) return
  try {
    micStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false })
    audioCtx = new AudioContext()
    analyserNode = audioCtx.createAnalyser()
    analyserNode.fftSize = 2048
    analyserNode.smoothingTimeConstant = 0.8
    audioCtx.createMediaStreamSource(micStream).connect(analyserNode)
    vizActive.value = true
    rafId = requestAnimationFrame(drawLoop)

    // Polling реальных уровней из Rust каждые 50мс
    levelsInterval = setInterval(async () => {
      try {
        const [inDb, outDb] = await invoke<[number, number]>('get_levels')
        realInDb.value  = inDb
        realOutDb.value = outDb
      } catch { /* Tauri недоступен в браузере */ }
    }, 50)
  } catch {
    // mic недоступен — визуализация молча не запустится
  }
}

function stopViz() {
  vizActive.value = false
  if (rafId !== null) { cancelAnimationFrame(rafId); rafId = null }
  if (levelsInterval !== null) { clearInterval(levelsInterval); levelsInterval = null }
  micStream?.getTracks().forEach(t => t.stop())
  micStream = null
  audioCtx?.close()
  audioCtx = null
  analyserNode = null
  currentDb.value = DB_MIN
  realInDb.value  = DB_MIN
  realOutDb.value = DB_MIN
}

function drawLoop() {
  if (!vizActive.value || !analyserNode) return
  rafId = requestAnimationFrame(drawLoop)

  const bufLen = analyserNode.frequencyBinCount
  const timeData = new Uint8Array(bufLen)
  const freqData = new Uint8Array(bufLen)
  analyserNode.getByteTimeDomainData(timeData)
  analyserNode.getByteFrequencyData(freqData)

  // currentDb только для осциллографа (порог gate на canvas)
  let sum = 0
  for (let i = 0; i < bufLen; i++) { const v = (timeData[i] - 128) / 128; sum += v * v }
  const rms = Math.sqrt(sum / bufLen)
  currentDb.value = rms > 0.00001 ? Math.max(DB_MIN, 20 * Math.log10(rms)) : DB_MIN

  drawWave(timeData)
  drawSpectrum(freqData)
}

function drawWave(timeData: Uint8Array) {
  const canvas = waveCanvas.value
  if (!canvas) return
  // Подгоняем внутренние пиксели под CSS-размер
  if (canvas.width !== canvas.offsetWidth) canvas.width = canvas.offsetWidth
  const W = canvas.width, H = canvas.height
  const ctx = canvas.getContext('2d')!

  ctx.clearRect(0, 0, W, H)

  // Центральная нулевая линия
  ctx.strokeStyle = 'rgba(255,255,255,0.06)'
  ctx.lineWidth = 1
  ctx.beginPath(); ctx.moveTo(0, H / 2); ctx.lineTo(W, H / 2); ctx.stroke()

  // Линия порога gate (если включён) — пунктир
  if (fx.gate.enabled) {
    const amp = Math.pow(10, fx.gate.threshold_db / 20)
    const yTop = H / 2 - amp * H / 2
    const yBot = H / 2 + amp * H / 2
    const passing = currentDb.value > fx.gate.threshold_db
    ctx.strokeStyle = passing ? 'rgba(35,197,94,0.5)' : 'rgba(239,68,68,0.5)'
    ctx.lineWidth = 1
    ctx.setLineDash([4, 4])
    ctx.beginPath(); ctx.moveTo(0, yTop); ctx.lineTo(W, yTop); ctx.stroke()
    ctx.beginPath(); ctx.moveTo(0, yBot); ctx.lineTo(W, yBot); ctx.stroke()
    ctx.setLineDash([])
  }

  // Осциллограмма
  ctx.strokeStyle = '#7c6cff'
  ctx.lineWidth = 1.5
  ctx.beginPath()
  const step = W / timeData.length
  for (let i = 0; i < timeData.length; i++) {
    const x = i * step
    const y = (timeData[i] / 128) * (H / 2)
    i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)
  }
  ctx.stroke()
}

function drawSpectrum(freqData: Uint8Array) {
  const canvas = specCanvas.value
  if (!canvas) return
  if (canvas.width !== canvas.offsetWidth) canvas.width = canvas.offsetWidth
  const W = canvas.width, H = canvas.height
  const ctx = canvas.getContext('2d')!

  ctx.clearRect(0, 0, W, H)

  // Полосы EQ — подсвечиваем диапазоны если EQ включён
  if (fx.eq.enabled) {
    const sampleRate = audioCtx?.sampleRate ?? 48000
    const binHz = sampleRate / (analyserNode!.fftSize)
    const lowBin  = Math.round(200 / binHz)
    const midBin  = Math.round(fx.eq.mid_freq / binHz)
    const highBin = Math.round(8000 / binHz)
    const half = freqData.length

    const xOf = (bin: number) => (bin / half) * W
    const xLow  = xOf(Math.max(0, lowBin - 30))
    const xMid  = xOf(Math.max(0, midBin - 60))
    const xHigh = xOf(Math.max(0, highBin - 80))
    ctx.fillStyle = 'rgba(124,108,255,0.06)'; ctx.fillRect(xLow, 0, 60 * W / half, H)
    ctx.fillStyle = 'rgba(96,165,250,0.06)';  ctx.fillRect(xMid, 0, 120 * W / half, H)
    ctx.fillStyle = 'rgba(52,211,153,0.06)';  ctx.fillRect(xHigh, 0, 160 * W / half, H)
  }

  // Спектральные бары — логарифмическая шкала по X
  const half = freqData.length
  const barW = Math.max(2, W / 80)
  for (let i = 0; i < 80; i++) {
    // Логарифмическое распределение: 20Hz–20kHz
    const freq = 20 * Math.pow(1000, i / 80)
    const sampleRate = audioCtx?.sampleRate ?? 48000
    const bin = Math.round(freq / (sampleRate / analyserNode!.fftSize / 2))
    const val = bin < half ? freqData[bin] : 0
    const barH = (val / 255) * H

    // Цвет: фиолетовый → синий → голубой по частоте
    const hue = 270 - (i / 80) * 100
    const sat = 60 + (val / 255) * 20
    const lit  = 45 + (val / 255) * 20
    ctx.fillStyle = `hsl(${hue},${sat}%,${lit}%)`
    ctx.fillRect(i * (W / 80), H - barH, barW - 1, barH)
  }
}

watch(activeTab, (tab) => {
  if (tab === 'effects') startViz()
  else stopViz()
})

onUnmounted(stopViz)

// ── Built-in effects reactive state ──────────────────────────────────────────
const fx = reactive({
  rnnoise: false,
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
  fx.rnnoise = localStorage.getItem('fx_rnnoise') === 'true'
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
async function applyRnnoise() {
  localStorage.setItem('fx_rnnoise', String(fx.rnnoise))
  await invoke('set_rnnoise', { enabled: fx.rnnoise }).catch(() => {})
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
  height: 680px;
  max-height: 92vh;
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
  min-height: 0;
  padding: 32px;
  overflow-y: auto;
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 20px;
  scrollbar-width: thin;
  scrollbar-color: var(--border) transparent;
}
/* Запрещаем flex-shrink на детях — иначе они сжимаются вместо того чтобы появился скролл */
.settings-content > * {
  flex-shrink: 0;
}
.settings-content::-webkit-scrollbar {
  width: 6px;
}
.settings-content::-webkit-scrollbar-track {
  background: transparent;
}
.settings-content::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}
.settings-content::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
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

/* ── Визуализация (sticky) ────────────────────────────────────────────────── */
.viz-panel {
  position: sticky;
  top: -32px; /* компенсируем padding родителя */
  z-index: 10;
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  overflow: hidden;
  background: #0a0a12;
  box-shadow: 0 4px 20px rgba(0,0,0,0.5);
}

.viz-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.09em;
  border-bottom: 1px solid var(--border);
}

.viz-title-row { display: flex; align-items: center; gap: 6px; }
.viz-sep-inline { color: var(--border); }
.viz-dot { font-size: 10px; color: var(--text-muted); }
.viz-dot.active { color: var(--online); }

.viz-canvases {
  display: flex;
  border-bottom: 1px solid var(--border);
}

.viz-wave {
  display: block;
  width: 40%;
  height: 52px;
  border-right: 1px solid var(--border);
  flex-shrink: 0;
}

.viz-divider { display: none; }

.viz-spec {
  display: block;
  flex: 1;
  height: 52px;
}

.viz-freq-labels {
  display: flex;
  justify-content: space-between;
  padding: 2px 8px 2px calc(40% + 8px);
  font-size: 9px;
  color: var(--text-muted);
  font-family: monospace;
  border-bottom: 1px solid var(--border);
}

/* ── Dual IN/OUT meter ────────────────────────────────────────────────────── */
.dual-meter {
  padding: 8px 10px 6px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.dual-meter-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dm-label {
  font-size: 9px;
  font-weight: 800;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  min-width: 24px;
}

.dm-track {
  flex: 1;
  height: 7px;
  background: rgba(255,255,255,0.06);
  border-radius: 4px;
  position: relative;
  overflow: visible;
}

.dm-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.04s linear;
}

.dm-in  { background: linear-gradient(90deg, #7c6cff, #a78bfa); }
.dm-out { background: linear-gradient(90deg, #059669, #34d399); }

.dm-pin {
  position: absolute;
  top: -3px;
  width: 2px;
  height: 13px;
  border-radius: 1px;
  transform: translateX(-50%);
  pointer-events: none;
}

.dm-pin-gate { background: #facc15; }
.dm-pin-comp { background: #fb923c; }

.dm-val {
  font-size: 10px;
  font-family: monospace;
  color: var(--text-secondary);
  min-width: 62px;
  text-align: right;
}

.dm-gated {
  color: var(--danger);
  font-weight: 700;
  letter-spacing: 0.04em;
}

.dm-legend {
  display: flex;
  gap: 10px;
  padding-left: 32px;
  font-size: 9px;
  font-family: monospace;
}

.dm-leg-gate { color: #facc15; }
.dm-leg-comp { color: #fb923c; }

/* ── GR meter (в Compressor) ──────────────────────────────────────────────── */
.meter-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.meter-side-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  min-width: 32px;
}

.meter-db-val {
  font-size: 11px;
  font-family: monospace;
  color: var(--text-secondary);
  min-width: 56px;
  text-align: right;
}

.gr-track {
  flex: 1;
  height: 8px;
  background: var(--bg-hover);
  border-radius: 4px;
  overflow: hidden;
}

.gr-fill {
  height: 100%;
  border-radius: 4px;
  background: linear-gradient(90deg, #7c6cff, #ef4444);
  transition: width 0.05s linear;
}

.gr-val { color: #ef9090; }

/* ── Эффекты ──────────────────────────────────────────────────────────────── */
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

.fx-badge {
  font-size: 9px;
  font-weight: 800;
  padding: 2px 5px;
  border-radius: 4px;
  background: linear-gradient(135deg, #7c6cff, #a78bfa);
  color: #fff;
  letter-spacing: 0.05em;
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
