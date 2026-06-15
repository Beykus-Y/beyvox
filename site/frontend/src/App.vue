<template>
  <nav class="nav">
    <div class="container nav-inner">
      <RouterLink to="/" class="logo">BeyVox</RouterLink>
      <div class="nav-links">
        <RouterLink to="/servers">Серверы</RouterLink>
        <template v-if="auth.isLoggedIn">
          <span class="nav-user">{{ auth.username }}</span>
          <button class="btn btn-ghost" @click="auth.logout()">Выйти</button>
        </template>
        <template v-else>
          <RouterLink to="/login" class="btn btn-ghost">Войти</RouterLink>
          <RouterLink to="/register" class="btn btn-primary">Регистрация</RouterLink>
        </template>
      </div>
    </div>
  </nav>
  <RouterView />
</template>

<script setup lang="ts">
import { useAuthStore } from './stores/auth'
const auth = useAuthStore()
</script>

<style scoped>
.nav {
  position: sticky;
  top: 0;
  z-index: 100;
  background: rgba(14, 15, 19, 0.85);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--border);
}
.nav-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 60px;
}
.logo {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent);
  letter-spacing: -0.5px;
}
.nav-links {
  display: flex;
  align-items: center;
  gap: 16px;
}
.nav-links a { color: var(--text2); font-size: 14px; }
.nav-links a:hover { color: var(--text); }
.nav-user { color: var(--text2); font-size: 14px; }
</style>
