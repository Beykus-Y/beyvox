<template>
  <div
    class="guild-row"
    :class="{ active: isActive }"
    @click="$emit('select', guild.id)"
    @contextmenu.prevent="$emit('contextmenu', $event, guild)"
  >
    <!-- Иконка гильдии (Squircle) -->
    <div class="guild-avatar" :style="avatarStyle">
      <img v-if="guild.icon_url" :src="guild.icon_url" :alt="guild.name" @error="onAvatarError" />
      <span v-else>{{ guild.name[0]?.toUpperCase() }}</span>
    </div>

    <!-- Текстовое описание -->
    <div class="guild-info">
      <span class="guild-name">{{ guild.name }}</span>
      <span class="guild-online-count" v-if="onlineCount !== undefined && onlineCount !== null">
        {{ onlineCount }} онлайн
      </span>
    </div>

    <!-- Стрелка справа -->
    <svg class="chevron-right" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
      <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Guild } from '../../stores/guild'

const props = defineProps<{
  guild: Guild
  isActive: boolean
}>()

const emit = defineEmits(['select', 'contextmenu'])

const avatarLoadError = ref(false)

const onlineCount = computed(() => {
  return props.guild.member_count !== undefined ? props.guild.member_count : undefined
})

const avatarStyle = computed(() => {
  if (props.guild.icon_url && !avatarLoadError.value) {
    return {}
  }
  return {
    background: getDeterministicGradient(props.guild.id || props.guild.name)
  }
})

function onAvatarError() {
  avatarLoadError.value = true
}

function getDeterministicGradient(id: string): string {
  let hash1 = 0
  let hash2 = 0
  for (let i = 0; i < id.length; i++) {
    hash1 = id.charCodeAt(i) + ((hash1 << 5) - hash1)
    hash2 = id.charCodeAt(i) * 31 + ((hash2 << 7) - hash2)
  }
  const color1 = `hsl(${Math.abs(hash1) % 360}, 55%, 45%)`
  const color2 = `hsl(${Math.abs(hash2) % 360}, 50%, 25%)`
  return `linear-gradient(135deg, ${color1}, ${color2})`
}
</script>

<style scoped>
.guild-row {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 56px;
  padding: 8px 12px;
  margin: 2px 8px 2px 0;
  border-radius: 8px;
  cursor: pointer;
  background: transparent;
  position: relative;
  transition: background 0.15s ease, border-radius 0.15s ease;
  user-select: none;
}
.guild-row:hover {
  background: var(--bg-hover);
}
.guild-row.active {
  background: var(--bg-active);
  border-radius: 0 8px 8px 0;
}
.guild-row::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 3px;
  background: var(--accent);
  border-radius: 0 4px 4px 0;
  opacity: 0;
  transition: opacity 0.15s ease;
}
.guild-row.active::before {
  opacity: 1;
}

.guild-avatar {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 18px;
  color: #fff;
  flex-shrink: 0;
  overflow: hidden;
}
.guild-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.guild-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.guild-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.guild-online-count {
  font-size: 12px;
  color: var(--text-secondary);
}

.chevron-right {
  color: var(--text-muted);
  opacity: 0.6;
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.guild-row:hover .chevron-right {
  opacity: 1;
}
.guild-row.active .chevron-right {
  opacity: 1;
  color: var(--text-primary);
}
</style>
