<template>
  <div class="login-page">
    <div class="login-glow" />
    <div class="login-box">
      <div class="logo">
        <div class="logo-icon">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 3a9 9 0 0 1 9 9 9 9 0 0 1-9 9 9 9 0 0 1-9-9 9 9 0 0 1 9-9m0 2a7 7 0 0 0-7 7 7 7 0 0 0 7 7 7 7 0 0 0 7-7 7 7 0 0 0-7-7m0 1.5a5.5 5.5 0 1 1 0 11 5.5 5.5 0 0 1 0-11M9.5 8v4.75l4 2.25.75-1.23-3.25-1.88V8H9.5z"/>
          </svg>
        </div>
        <span class="logo-text">BeyVox</span>
      </div>
      <p class="subtitle">Голосовой мессенджер под твоим контролем</p>

      <div class="tabs">
        <button :class="{ active: mode === 'login' }" @click="mode = 'login'">Войти</button>
        <button :class="{ active: mode === 'register' }" @click="mode = 'register'">Регистрация</button>
      </div>

      <form @submit.prevent="submit">
        <div class="field">
          <label>Сервер авторизации</label>
          <input v-model="centralUrl" placeholder="https://beyvox.beykus.fun" />
        </div>

        <div class="field">
          <label>{{ mode === 'login' ? 'Логин или Email' : 'Логин' }}</label>
          <input v-model="form.login" type="text" required autocomplete="username" />
        </div>

        <div v-if="mode === 'register'" class="field">
          <label>Email</label>
          <input v-model="form.email" type="email" required />
        </div>

        <div class="field">
          <label>Пароль</label>
          <input v-model="form.password" type="password" required autocomplete="current-password" />
        </div>

        <p v-if="error" class="error">{{ error }}</p>

        <button type="submit" class="submit-btn" :disabled="loading">
          <span v-if="loading" class="spinner" />
          {{ loading ? 'Подождите...' : (mode === 'login' ? 'Войти' : 'Создать аккаунт') }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const auth = useAuthStore()

const mode = ref<'login' | 'register'>('login')
const loading = ref(false)
const error = ref('')
const centralUrl = ref(auth.centralUrl)
const form = ref({ login: '', email: '', password: '' })

async function submit() {
  error.value = ''
  loading.value = true
  auth.setCentralUrl(centralUrl.value)
  try {
    if (mode.value === 'login') {
      await auth.login(form.value.login, form.value.password)
    } else {
      await auth.register(form.value.login, form.value.email, form.value.password)
    }
    router.push('/app')
  } catch (e: any) {
    error.value = e?.response?.data?.error || 'Ошибка подключения'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-page {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-darkest);
  position: relative;
  overflow: hidden;
}

.login-glow {
  position: absolute;
  width: 500px;
  height: 500px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(91, 124, 246, 0.12) 0%, transparent 70%);
  pointer-events: none;
}

.login-box {
  width: 420px;
  background: var(--bg-dark);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 40px;
  position: relative;
  z-index: 1;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.logo-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: linear-gradient(135deg, var(--accent), #a78bfa);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.logo-text {
  font-size: 26px;
  font-weight: 800;
  background: linear-gradient(135deg, var(--accent), #a78bfa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.subtitle { color: var(--text2); font-size: 13px; margin-bottom: 28px; }

.tabs {
  display: flex;
  background: var(--bg);
  border-radius: 8px;
  padding: 3px;
  margin-bottom: 24px;
}
.tabs button {
  flex: 1;
  padding: 8px;
  border-radius: 6px;
  background: transparent;
  color: var(--text2);
  transition: all 0.15s;
}
.tabs button.active {
  background: var(--bg-dark);
  color: var(--text);
  font-weight: 600;
  box-shadow: 0 1px 4px rgba(0,0,0,0.3);
}

form { display: flex; flex-direction: column; gap: 14px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: 11px; font-weight: 600; color: var(--text2); text-transform: uppercase; letter-spacing: 0.5px; }

.error {
  color: var(--red);
  font-size: 13px;
  background: rgba(255, 85, 85, 0.1);
  padding: 8px 12px;
  border-radius: 6px;
  border-left: 3px solid var(--red);
}

.submit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  background: linear-gradient(135deg, var(--accent), #7c6cf7);
  color: white;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  margin-top: 4px;
  transition: opacity 0.15s, transform 0.1s;
}
.submit-btn:hover { opacity: 0.9; transform: translateY(-1px); }
.submit-btn:active { transform: translateY(0); }
.submit-btn:disabled { opacity: 0.5; cursor: not-allowed; transform: none; }

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
