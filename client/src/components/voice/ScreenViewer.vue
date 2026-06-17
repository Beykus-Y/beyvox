<template>
  <div class="screen-viewer-overlay" @click.self="screenStore.closeStream()">
    <div class="screen-viewer">
      <div class="viewer-header">
        <span class="viewer-title">Трансляция — {{ username }}</span>
        <button class="viewer-close" @click="screenStore.closeStream()">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>

      <div class="viewer-canvas-wrap">
        <canvas ref="canvasRef" class="viewer-canvas" />
        <div v-if="!hasFrame" class="viewer-placeholder">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
            <path d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h5v2h8v-2h5c1.1 0 1.99-.9 1.99-2L23 5c0-1.1-.9-2-2-2zm0 16H3V5h18v14z"/>
          </svg>
          <p>Ожидание видео...</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useScreenStore } from '../../stores/screen'

const props = defineProps<{
  userId: string
  username: string
}>()

const screenStore = useScreenStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)
const hasFrame = ref(false)
let unlisten: UnlistenFn | null = null

async function subscribeToFrames(userId: string) {
  unlisten?.()
  unlisten = await listen<number[]>(`screen://frame/${userId}`, (event) => {
    const jpegBytes = new Uint8Array(event.payload)
    renderFrame(jpegBytes)
  })
}

function renderFrame(jpegBytes: Uint8Array) {
  const canvas = canvasRef.value
  if (!canvas) return

  const blob = new Blob([jpegBytes], { type: 'image/jpeg' })
  const url = URL.createObjectURL(blob)
  const img = new Image()
  img.onload = () => {
    canvas.width = img.width
    canvas.height = img.height
    const ctx = canvas.getContext('2d')
    ctx?.drawImage(img, 0, 0)
    URL.revokeObjectURL(url)
    hasFrame.value = true
  }
  img.src = url
}

onMounted(() => {
  subscribeToFrames(props.userId)
})

watch(() => props.userId, (newId) => {
  hasFrame.value = false
  subscribeToFrames(newId)
})

onUnmounted(() => {
  unlisten?.()
})
</script>

<style scoped>
.screen-viewer-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
}

.screen-viewer {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  max-width: 92vw;
  max-height: 90vh;
  overflow: hidden;
}

.viewer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.viewer-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.viewer-close {
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
.viewer-close:hover { background: var(--bg-hover); color: var(--text-primary); }

.viewer-canvas-wrap {
  position: relative;
  min-width: 640px;
  min-height: 360px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
}

.viewer-canvas {
  max-width: 88vw;
  max-height: calc(90vh - 60px);
  display: block;
}

.viewer-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-muted);
}

.viewer-placeholder p {
  font-size: 13px;
  margin: 0;
}
</style>
