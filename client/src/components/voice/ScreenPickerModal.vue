<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="screen-picker-modal">
      <div class="picker-header">
        <h3>Поделиться экраном</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>

      <div v-if="loading" class="picker-loading">Загрузка экранов...</div>

      <div v-else class="screens-grid">
        <button
          v-for="screen in screens"
          :key="screen.id"
          class="screen-tile"
          :class="{ selected: selectedId === screen.id }"
          @click="selectedId = screen.id"
        >
          <div class="screen-thumb">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor" opacity="0.5">
              <path d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h5v2h8v-2h5c1.1 0 1.99-.9 1.99-2L23 5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"/>
            </svg>
          </div>
          <div class="screen-info">
            <span class="screen-name">{{ screen.name || `Экран ${screen.id + 1}` }}</span>
            <span class="screen-res">{{ screen.width }}×{{ screen.height }}</span>
          </div>
        </button>
      </div>

      <div class="picker-settings">
        <div class="setting-group">
          <label>Качество</label>
          <div class="setting-pills">
            <button
              v-for="q in qualityOptions"
              :key="q.value"
              class="pill"
              :class="{ active: quality === q.value }"
              @click="quality = q.value"
            >{{ q.label }}</button>
          </div>
        </div>

        <div class="setting-group">
          <label>FPS</label>
          <div class="setting-pills">
            <button
              v-for="f in fpsOptions"
              :key="f"
              class="pill"
              :class="{ active: fps === f }"
              @click="fps = f"
            >{{ f }}</button>
          </div>
        </div>
      </div>

      <div class="picker-actions">
        <button class="btn-cancel" @click="$emit('close')">Отмена</button>
        <button class="btn-start" :disabled="selectedId === null || starting" @click="startShare">
          {{ starting ? 'Запуск...' : 'Начать трансляцию' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useScreenStore, type ScreenInfo } from '../../stores/screen'

const emit = defineEmits<{ close: []; started: [] }>()

const screenStore = useScreenStore()
const screens = ref<ScreenInfo[]>([])
const loading = ref(true)
const selectedId = ref<number | null>(null)
const quality = ref('720p')
const fps = ref(30)
const starting = ref(false)

const qualityOptions = [
  { label: '360p', value: '360p' },
  { label: '720p', value: '720p' },
  { label: '1080p', value: '1080p' },
]
const fpsOptions = [15, 30, 60]

onMounted(async () => {
  try {
    screens.value = await screenStore.listScreens()
    if (screens.value.length > 0) selectedId.value = 0
  } finally {
    loading.value = false
  }
})

async function startShare() {
  if (selectedId.value === null) return
  starting.value = true
  try {
    await screenStore.startShare(selectedId.value, quality.value, fps.value)
    emit('started')
  } catch (e) {
    console.error('[screen] start failed:', e)
  } finally {
    starting.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
}

.screen-picker-modal {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  width: 520px;
  max-width: 96vw;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.picker-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.picker-header h3 {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.close-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: none;
  border: none;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.close-btn:hover { background: var(--bg-hover); color: var(--text-primary); }

.picker-loading {
  text-align: center;
  color: var(--text-muted);
  padding: 24px 0;
  font-size: 13px;
}

.screens-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 10px;
  max-height: 240px;
  overflow-y: auto;
}

.screen-tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px 8px;
  border-radius: 8px;
  background: var(--bg-elevated);
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s ease;
}
.screen-tile:hover { background: var(--bg-hover); border-color: var(--border); }
.screen-tile.selected { border-color: var(--accent); background: var(--bg-active); }

.screen-thumb {
  width: 80px;
  height: 50px;
  background: var(--bg-active);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
}

.screen-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.screen-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  text-align: center;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.screen-res {
  font-size: 10px;
  color: var(--text-muted);
  font-family: monospace;
}

.picker-settings {
  display: flex;
  gap: 24px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.setting-group label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.setting-pills {
  display: flex;
  gap: 6px;
}

.pill {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}
.pill:hover { background: var(--bg-hover); color: var(--text-primary); }
.pill.active { background: var(--accent); border-color: var(--accent); color: #fff; }

.picker-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn-cancel {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  cursor: pointer;
}
.btn-cancel:hover { background: var(--bg-hover); color: var(--text-primary); }

.btn-start {
  padding: 8px 18px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  background: var(--accent);
  border: none;
  color: #fff;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-start:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-start:not(:disabled):hover { opacity: 0.85; }
</style>
