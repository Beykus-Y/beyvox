<template>
  <div v-if="auth.isLoggedIn && !auth.emailVerified && !isBannerDismissed" class="verify-banner">
    <div class="container verify-banner-inner">
      <span class="verify-banner-text">
        ⚠️ Пожалуйста, подтвердите вашу почту. Письмо с ссылкой подтверждения отправлено на ваш email.
      </span>
      <button class="verify-banner-close" @click="isBannerDismissed = true">✕</button>
    </div>
  </div>
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
import { ref } from 'vue'
import { useAuthStore } from './stores/auth'
const auth = useAuthStore()
const isBannerDismissed = ref(false)
</script>

<style scoped>
.verify-banner {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  color: white;
  font-size: 13px;
  font-weight: 600;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
.verify-banner-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
}
.verify-banner-text {
  display: flex;
  align-items: center;
  gap: 8px;
}
.verify-banner-close {
  background: transparent;
  color: rgba(255, 255, 255, 0.85);
  font-size: 14px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.1s;
}
.verify-banner-close:hover {
  background: rgba(255, 255, 255, 0.15);
  color: white;
}
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
