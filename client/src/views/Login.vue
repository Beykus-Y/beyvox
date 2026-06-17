<template>
  <div class="login-page">
    <div class="login-glow" />
    
    <!-- Экран подтверждения почты -->
    <div v-if="showVerify" class="login-box">
      <div class="logo">
        <div class="logo-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M22 6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6zm-2 0l-8 5-8-5h16zm0 12H4V8l8 5 8-5v10z"/>
          </svg>
        </div>
        <span class="logo-text">Проверьте почту</span>
      </div>
      <p class="subtitle">
        Мы отправили письмо на почту <strong class="highlight-email">{{ form.email }}</strong> с ссылкой для подтверждения аккаунта.
      </p>

      <p v-if="error" class="error-banner">{{ error }}</p>
      
      <div class="action-buttons-group">
        <button class="submit-btn" :disabled="loading" @click="checkEmailVerified">
          <span v-if="loading" class="spinner-sm" />
          {{ loading ? 'Проверка...' : 'Я подтвердил почту' }}
        </button>
        <button class="btn-ghost-action" @click="enterApp">
          Подтвердить позже
        </button>
      </div>
    </div>

    <!-- Основной экран входа / регистрации -->
    <div v-else class="login-box">
      <div class="logo">
        <div class="logo-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 3a9 9 0 0 1 9 9 9 9 0 0 1-9 9 9 9 0 0 1-9-9 9 9 0 0 1 9-9m0 2a7 7 0 0 0-7 7 7 7 0 0 0 7 7 7 7 0 0 0 7-7 7 7 0 0 0-7-7m0 1.5a5.5 5.5 0 1 1 0 11 5.5 5.5 0 0 1 0-11M9.5 8v4.75l4 2.25.75-1.23-3.25-1.88V8H9.5z"/>
          </svg>
        </div>
        <span class="logo-text">BeyVox</span>
      </div>
      <p class="subtitle">Голосовой мессенджер под твоим контролем</p>

      <!-- Переключатель вкладок -->
      <div class="tabs">
        <button :class="{ active: mode === 'login' }" @click="setMode('login')">Войти</button>
        <button :class="{ active: mode === 'register' }" @click="setMode('register')">Регистрация</button>
      </div>

      <form @submit.prevent="submit">
        <!-- Поле логина -->
        <div class="field">
          <label class="uppercase-label">{{ mode === 'login' ? 'Логин или Email' : 'Логин' }}</label>
          <input
            v-model="form.login"
            type="text"
            required
            autocomplete="username"
            placeholder="Введите ваше имя пользователя"
          />
        </div>

        <!-- Поле почты для регистрации -->
        <div v-if="mode === 'register'" class="field">
          <label class="uppercase-label">Email адрес</label>
          <input
            v-model="form.email"
            type="email"
            required
            placeholder="example@mail.com"
          />
        </div>

        <!-- Поле пароля -->
        <div class="field">
          <label class="uppercase-label">Пароль</label>
          <div class="password-wrapper">
            <input
              v-model="form.password"
              :type="showPassword ? 'text' : 'password'"
              required
              autocomplete="current-password"
              placeholder="••••••••"
            />
            <button
              type="button"
              class="password-toggle-btn"
              @click="showPassword = !showPassword"
              tabindex="-1"
              :title="showPassword ? 'Скрыть пароль' : 'Показать пароль'"
            >
              <span v-if="showPassword">👁️</span>
              <span v-else>🙈</span>
            </button>
          </div>
        </div>

        <!-- Баннер ошибки -->
        <p v-if="error" class="error-banner">{{ error }}</p>

        <!-- Кнопка действия -->
        <button type="submit" class="submit-btn" :disabled="loading || !isFormValid">
          <span v-if="loading" class="spinner-sm" />
          <span>{{ loading ? 'Пожалуйста, подождите...' : (mode === 'login' ? 'Войти' : 'Создать аккаунт') }}</span>
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const auth = useAuthStore()

const mode = ref<'login' | 'register'>('login')
const loading = ref(false)
const error = ref('')
const showVerify = ref(false)
const showPassword = ref(false)
const form = ref({ login: '', email: '', password: '' })

const isFormValid = computed(() => {
  if (!form.value.login.trim() || !form.value.password) return false
  if (mode.value === 'register' && !form.value.email.trim()) return false
  return true
})

function setMode(newMode: 'login' | 'register') {
  mode.value = newMode
  error.value = ''
}

async function submit() {
  if (!isFormValid.value) return
  error.value = ''
  loading.value = true
  try {
    if (mode.value === 'login') {
      await auth.login(form.value.login, form.value.password)
      router.push('/app')
    } else {
      await auth.register(form.value.login, form.value.email, form.value.password)
      showVerify.value = true
    }
  } catch (e: any) {
    const status = e?.response?.status
    const msg = e?.response?.data?.error
    
    if (mode.value === 'login' && status === 401) {
      error.value = 'Неверный логин или пароль'
    } else if (status === 409) {
      error.value = msg === 'username already taken' ? 'Этот ник уже занят' : 'Этот email уже зарегистрирован'
    } else if (status === 400) {
      error.value = msg || 'Некорректный логин, пароль или email'
    } else {
      error.value = 'Ошибка сетевого соединения с сервером авторизации'
    }
  } finally {
    loading.value = false
  }
}

function enterApp() {
  router.push('/app')
}

async function checkEmailVerified() {
  error.value = ''
  loading.value = true
  try {
    const verified = await auth.checkVerificationStatus()
    if (verified) {
      router.push('/app')
    } else {
      error.value = 'Почта ещё не подтверждена. Пожалуйста, перейдите по ссылке в письме.'
    }
  } catch (e: any) {
    error.value = e?.response?.data?.error || 'Не удалось связаться с сервером верификации'
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
  background: var(--bg-app);
  position: relative;
  overflow: hidden;
}

.login-glow {
  position: absolute;
  width: 600px;
  height: 600px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(124, 108, 255, 0.08) 0%, transparent 70%);
  pointer-events: none;
}

.login-box {
  width: 400px;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  padding: 36px;
  position: relative;
  z-index: 1;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.logo-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-squircle);
  background: var(--accent-grad);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
  box-shadow: 0 4px 12px rgba(124, 108, 255, 0.25);
}

.logo-text {
  font-size: 24px;
  font-weight: 800;
  background: var(--accent-grad);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.subtitle {
  color: var(--text-secondary);
  font-size: 13px;
  margin-bottom: 24px;
  line-height: 1.45;
}

.highlight-email {
  color: var(--text-primary);
}

/* Табы авторизации */
.tabs {
  display: flex;
  background: var(--bg-app);
  border-radius: var(--radius-item);
  padding: 3px;
  margin-bottom: 20px;
  border: 1px solid var(--border);
}
.tabs button {
  flex: 1;
  padding: 8px;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  font-weight: 500;
  transition: all 0.15s;
}
.tabs button:hover:not(.active) {
  color: var(--text-secondary);
}
.tabs button.active {
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}
.password-wrapper input {
  padding-right: 40px;
  width: 100%;
}

.password-toggle-btn {
  position: absolute;
  right: 12px;
  background: transparent;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Ошибки */
.error-banner {
  color: var(--danger);
  font-size: 12.5px;
  background: rgba(239, 68, 68, 0.08);
  padding: 10px 12px;
  border-radius: var(--radius-item);
  border-left: 3px solid var(--danger);
}

.submit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  background: var(--accent-grad);
  color: white;
  border-radius: var(--radius-item);
  font-size: 14px;
  font-weight: 600;
  margin-top: 6px;
  box-shadow: 0 4px 14px rgba(124, 108, 255, 0.25);
}
.submit-btn:hover:not(:disabled) {
  opacity: 0.95;
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(124, 108, 255, 0.35);
}
.submit-btn:active:not(:disabled) {
  transform: translateY(0);
}
.submit-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  background: var(--border);
  color: var(--text-muted);
  box-shadow: none;
}

.action-buttons-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.btn-ghost-action {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 11px;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-item);
  font-size: 13.5px;
  font-weight: 600;
}
.btn-ghost-action:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Спиннер */
.spinner-sm {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
