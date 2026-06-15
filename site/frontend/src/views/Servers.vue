<template>
  <main class="container" style="padding-top: 40px; padding-bottom: 60px;">
    <div class="page-header">
      <div>
        <h1>Серверы</h1>
        <p class="subtitle">{{ servers.length }} публичных серверов онлайн</p>
      </div>
      <input v-model="search" placeholder="Поиск по названию или тегу..." class="search-input" />
    </div>

    <div v-if="loading" class="state-msg">Загрузка...</div>
    <div v-else-if="filtered.length === 0" class="state-msg">Серверов не найдено</div>
    <div v-else class="server-grid">
      <div v-for="s in filtered" :key="s.id" class="card server-card">
        <div class="server-icon">
          <img v-if="s.icon_url" :src="s.icon_url" :alt="s.name" />
          <div v-else class="icon-placeholder">{{ s.name[0].toUpperCase() }}</div>
        </div>
        <div class="server-info">
          <div class="server-name">
            <span>{{ s.name }}</span>
            <span class="dot-green"></span>
          </div>
          <p class="server-desc">{{ s.description || 'Нет описания' }}</p>
          <div class="server-meta">
            <span class="online-count">{{ s.online_count }} онлайн</span>
            <span v-for="tag in s.tags" :key="tag" class="tag">{{ tag }}</span>
          </div>
        </div>
        <button class="btn btn-primary connect-btn" @click="connect(s.address)">
          Подключиться
        </button>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { catalogApi } from '../api'

interface Server {
  id: string
  name: string
  description: string | null
  icon_url: string | null
  address: string
  tags: string[]
  online_count: number
}

const servers = ref<Server[]>([])
const search = ref('')
const loading = ref(true)

const filtered = computed(() => {
  const q = search.value.toLowerCase()
  if (!q) return servers.value
  return servers.value.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.tags.some((t) => t.toLowerCase().includes(q))
  )
})

function connect(address: string) {
  window.location.href = `beyvox://${address}`
}

onMounted(async () => {
  try {
    const { data } = await catalogApi.list()
    servers.value = data
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 32px;
  flex-wrap: wrap;
}
h1 { font-size: 28px; font-weight: 700; }
.subtitle { color: var(--text2); font-size: 14px; margin-top: 4px; }
.search-input { max-width: 320px; }

.server-grid { display: flex; flex-direction: column; gap: 12px; }
.server-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
}
.server-icon { flex-shrink: 0; }
.server-icon img, .icon-placeholder {
  width: 52px; height: 52px;
  border-radius: 12px;
  object-fit: cover;
}
.icon-placeholder {
  background: var(--bg3);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  font-weight: 700;
  color: var(--accent);
}
.server-info { flex: 1; min-width: 0; }
.server-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  margin-bottom: 4px;
}
.server-desc {
  color: var(--text2);
  font-size: 13px;
  margin-bottom: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.server-meta { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.online-count { color: var(--green); font-size: 12px; font-weight: 500; }
.connect-btn { flex-shrink: 0; }

.state-msg { color: var(--text2); text-align: center; padding: 60px 0; }
</style>
