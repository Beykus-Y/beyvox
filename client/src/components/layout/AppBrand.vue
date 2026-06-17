<template>
  <div class="brand-container">
    <div class="brand-left" @click="toggleMenu">
      <div class="brand-logo">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 3a9 9 0 0 1 9 9 9 9 0 0 1-9 9 9 9 0 0 1-9-9 9 9 0 0 1 9-9m0 2a7 7 0 0 0-7 7 7 7 0 0 0 7 7 7 7 0 0 0 7-7 7 7 0 0 0-7-7m0 1.5a5.5 5.5 0 1 1 0 11 5.5 5.5 0 0 1 0-11M9.5 8v4.75l4 2.25.75-1.23-3.25-1.88V8H9.5z"/>
        </svg>
      </div>
      <span class="brand-name">BeyVox</span>
      <svg class="chevron-icon" :class="{ open: menuOpen }" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
        <path d="M7 10l5 5 5-5z"/>
      </svg>
    </div>
    
    <button class="settings-btn" @click="$emit('open-settings')" title="Настройки">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.02 7.02 0 0 0-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.47.47 0 0 0-.59.22L2.74 8.87a.49.49 0 0 0 .12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.01-1.58zM12 15.6a3.6 3.6 0 1 1 0-7.2 3.6 3.6 0 0 1 0 7.2z"/>
      </svg>
    </button>

    <!-- Дропдаун аккаунта -->
    <div v-if="menuOpen" class="brand-dropdown" v-click-outside="closeMenu">
      <div class="dropdown-header">
        <span class="user-display">{{ username }}</span>
      </div>
      <div class="dropdown-divider" />
      <button class="dropdown-item" @click="openSettings">
        Настройки аккаунта
      </button>
      <button class="dropdown-item danger" @click="logout">
        Выйти
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

defineProps<{ username: string }>()
const emit = defineEmits(['open-settings', 'logout'])

const menuOpen = ref(false)

function toggleMenu() {
  menuOpen.value = !menuOpen.value
}

function closeMenu() {
  menuOpen.value = false
}

function openSettings() {
  emit('open-settings')
  closeMenu()
}

function logout() {
  emit('logout')
  closeMenu()
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
.brand-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 54px;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  position: relative;
  background: var(--bg-panel);
}

.brand-left {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.brand-logo {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-item);
  background: var(--accent-grad);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  box-shadow: 0 2px 8px rgba(124, 108, 255, 0.25);
}

.brand-name {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.chevron-icon {
  color: var(--text-secondary);
  transition: transform 0.2s ease;
}
.chevron-icon.open {
  transform: rotate(180deg);
}

.settings-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
}
.settings-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Дропдаун */
.brand-dropdown {
  position: absolute;
  top: 50px;
  left: 16px;
  z-index: 100;
  width: 180px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  padding: 4px;
}

.dropdown-header {
  padding: 8px 12px;
}

.user-display {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-primary);
}

.dropdown-divider {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}

.dropdown-item {
  width: 100%;
  text-align: left;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  display: block;
}
.dropdown-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.dropdown-item.danger {
  color: var(--danger);
}
.dropdown-item.danger:hover {
  background: rgba(239, 68, 68, 0.1);
}
</style>
