<template>
  <main class="auth-page">
    <div class="auth-box card">
      <RouterLink to="/" class="auth-logo">BeyVox</RouterLink>
      <h2>{{ isLogin ? 'Войти в аккаунт' : 'Создать аккаунт' }}</h2>

      <form @submit.prevent="submit">
        <div class="field">
          <label>Логин или Email</label>
          <input v-model="form.login" type="text" placeholder="username" autocomplete="username" required />
        </div>

        <div v-if="!isLogin" class="field">
          <label>Email</label>
          <input v-model="form.email" type="email" placeholder="you@example.com" required />
        </div>

        <div class="field">
          <label>Пароль</label>
          <input v-model="form.password" type="password" placeholder="••••••••" autocomplete="current-password" required />
        </div>

        <p v-if="error" class="error-msg">{{ error }}</p>

        <button type="submit" class="btn btn-primary submit-btn" :disabled="loading">
          {{ loading ? 'Подождите...' : (isLogin ? 'Войти' : 'Создать аккаунт') }}
        </button>
      </form>

      <p class="switch-link">
        {{ isLogin ? 'Нет аккаунта?' : 'Уже есть аккаунт?' }}
        <RouterLink :to="isLogin ? '/register' : '/login'">
          {{ isLogin ? 'Зарегистрироваться' : 'Войти' }}
        </RouterLink>
      </p>
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const props = defineProps<{ mode: 'login' | 'register' }>()
const router = useRouter()
const auth = useAuthStore()

const isLogin = computed(() => props.mode === 'login')
const loading = ref(false)
const error = ref('')

const form = ref({ login: '', email: '', password: '' })

async function submit() {
  error.value = ''
  loading.value = true
  try {
    if (isLogin.value) {
      await auth.login(form.value.login, form.value.password)
    } else {
      await auth.register(form.value.login, form.value.email, form.value.password)
    }
    router.push('/servers')
  } catch (e: any) {
    error.value = e?.response?.data?.error || 'Что-то пошло не так'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.auth-page {
  min-height: calc(100vh - 60px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}
.auth-box {
  width: 100%;
  max-width: 400px;
  padding: 36px;
}
.auth-logo {
  display: block;
  font-size: 22px;
  font-weight: 700;
  color: var(--accent);
  margin-bottom: 20px;
}
h2 { font-size: 22px; font-weight: 700; margin-bottom: 28px; }

form { display: flex; flex-direction: column; gap: 16px; }
.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: 13px; color: var(--text2); }

.submit-btn { width: 100%; justify-content: center; padding: 12px; }
.submit-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.switch-link {
  text-align: center;
  color: var(--text2);
  font-size: 13px;
  margin-top: 20px;
}
</style>
