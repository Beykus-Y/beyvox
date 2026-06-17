<template>
  <div class="breadcrumb-bar">
    <span class="segment clickable" @click="$emit('focus-servers')">{{ serverName }}</span>
    <span class="divider">/</span>
    <span class="segment clickable" @click="$emit('focus-guilds')">{{ guildName }}</span>
    <span class="divider">/</span>
    
    <!-- Переключатель каналов -->
    <div class="channel-selector-wrapper" v-click-outside="closeDropdown">
      <div class="segment clickable channel-segment" @click="toggleDropdown">
        <span class="prefix">#</span>
        <span class="channel-name">{{ channelName }}</span>
        <svg class="dropdown-chevron" :class="{ open: dropdownOpen }" width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
          <path d="M7 10l5 5 5-5z"/>
        </svg>
      </div>

      <!-- Выпадающий список каналов -->
      <div v-if="dropdownOpen" class="channels-dropdown">
        <div class="dropdown-title uppercase-label">Текстовые каналы</div>
        <div class="dropdown-list">
          <div
            v-for="ch in textChannels"
            :key="ch.id"
            class="dropdown-item"
            :class="{ active: ch.id === activeChannelId }"
            @click="selectChannel(ch)"
          >
            <span class="item-prefix">#</span>
            <span class="item-name">{{ ch.name }}</span>
            <span v-if="ch.id === activeChannelId" class="active-dot">●</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Channel } from '../../stores/guild'

const props = defineProps<{
  serverName: string
  guildName: string
  channelName: string
  activeChannelId: string | null
  channels: Channel[]
}>()

const emit = defineEmits(['select-channel', 'focus-servers', 'focus-guilds'])

const dropdownOpen = ref(false)

const textChannels = computed(() => {
  return props.channels.filter(c => c.type === 'text')
})

function toggleDropdown() {
  dropdownOpen.value = !dropdownOpen.value
}

function closeDropdown() {
  dropdownOpen.value = false
}

function selectChannel(ch: Channel) {
  emit('select-channel', ch)
  closeDropdown()
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
.breadcrumb-bar {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 16px;
  background: var(--bg-app);
  font-size: 11px;
  font-weight: 500;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  gap: 6px;
  flex-shrink: 0;
  user-select: none;
}

.segment {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 140px;
}

.segment.clickable {
  cursor: pointer;
  transition: color 0.15s ease;
}
.segment.clickable:hover {
  color: var(--text-primary);
}

.divider {
  color: var(--text-muted);
}

.channel-selector-wrapper {
  position: relative;
  display: inline-block;
}

.channel-segment {
  display: flex;
  align-items: center;
  gap: 2px;
  color: var(--text-primary);
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
}
.channel-segment:hover {
  background: var(--bg-hover);
}

.prefix {
  color: var(--text-muted);
}

.dropdown-chevron {
  color: var(--text-muted);
  transition: transform 0.2s ease;
}
.dropdown-chevron.open {
  transform: rotate(180deg);
}

/* Выпадающий список */
.channels-dropdown {
  position: absolute;
  top: 24px;
  left: 0;
  z-index: 100;
  width: 200px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  padding: 6px;
}

.dropdown-title {
  padding: 6px 8px;
}

.dropdown-list {
  display: flex;
  flex-direction: column;
  max-height: 180px;
  overflow-y: auto;
  gap: 2px;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  transition: background-color 0.15s, color 0.15s;
}
.dropdown-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.dropdown-item.active {
  background: var(--bg-active);
  color: var(--accent);
}

.item-prefix {
  color: var(--text-muted);
}

.item-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.active-dot {
  font-size: 9px;
  color: var(--accent);
}
</style>
