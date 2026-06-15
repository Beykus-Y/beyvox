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

  <footer class="footer">
    <div class="container footer-inner">
      <div class="footer-left">
        <span class="footer-logo">BeyVox</span>
        <span class="footer-copy">MIT License · <a href="https://beykus.fun" target="_blank">beykus.fun</a></span>
      </div>
      <div class="footer-links">
        <a href="https://github.com/Beykus-Y/beyvox" target="_blank" class="footer-link">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>
          GitHub
        </a>
        <a href="https://github.com/Beykus-Y/beyvox/releases" target="_blank" class="footer-link">Релизы</a>
        <a href="https://github.com/Beykus-Y/beyvox/blob/main/LICENSE" target="_blank" class="footer-link">MIT</a>
        <span class="footer-status">
          <span class="status-dot" :class="serverOnline ? 'online' : 'offline'"></span>
          {{ serverOnline ? 'Сервер работает' : 'Сервер недоступен' }}
        </span>
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from './stores/auth'
const auth = useAuthStore()
const isBannerDismissed = ref(false)
const serverOnline = ref(false)

onMounted(async () => {
  try {
    const r = await fetch('https://server.beyvox.beykus.fun/')
    serverOnline.value = r.ok
  } catch {
    serverOnline.value = false
  }
})
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

.footer {
  border-top: 1px solid var(--border);
  padding: 20px 0;
  margin-top: 40px;
}
.footer-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}
.footer-left { display: flex; align-items: center; gap: 14px; }
.footer-logo { font-size: 16px; font-weight: 700; color: var(--accent); }
.footer-copy { font-size: 12px; color: var(--text3); }
.footer-copy a { color: var(--text2); }
.footer-copy a:hover { color: var(--text); }
.footer-links { display: flex; align-items: center; gap: 20px; flex-wrap: wrap; }
.footer-link { font-size: 13px; color: var(--text2); display: flex; align-items: center; gap: 5px; }
.footer-link:hover { color: var(--text); }
.footer-status { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--text2); }
.status-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
.status-dot.online { background: var(--green); box-shadow: 0 0 6px var(--green); }
.status-dot.offline { background: #f04747; }
</style>
