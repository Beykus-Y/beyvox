<template>
  <main>
    <!-- Hero -->
    <section class="hero">
      <div class="container">
        <div class="badge">Open Source · Self-Hosted · v0.0.2</div>
        <h1>Голосовой мессенджер<br><span class="accent">под твоим контролем</span></h1>
        <p class="subtitle">
          BeyVox — альтернатива Discord и TeamSpeak.<br>
          Поднимай свой сервер, общайся голосом, никаких посредников.
        </p>
        <div class="hero-actions">
          <a href="https://github.com/Beykus-Y/beyvox/releases/tag/v0.0.2" target="_blank" class="btn btn-primary">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 16l-6-6h4V4h4v6h4l-6 6zm-7 2h14v2H5v-2z"/></svg>
            Скачать v0.0.2
          </a>
          <a href="https://github.com/Beykus-Y/beyvox" target="_blank" class="btn btn-ghost">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>
            GitHub
          </a>
          <RouterLink to="/servers" class="btn btn-ghost">Найти сервер</RouterLink>
        </div>
      </div>
    </section>

    <!-- Features -->
    <section class="features container">
      <div class="feature-grid">
        <div class="card feature-card">
          <div class="feature-icon">🎙️</div>
          <h3>Голос без задержек</h3>
          <p>WebRTC + OPUS кодек, задержка до 150 мс. PTT и VAD на выбор.</p>
        </div>
        <div class="card feature-card">
          <div class="feature-icon">🔌</div>
          <h3>VST плагины</h3>
          <p>Загружай VST2/VST3 плагины прямо в клиент. EQ, компрессор, гейт — всё что нужно.</p>
        </div>
        <div class="card feature-card">
          <div class="feature-icon">🖥️</div>
          <h3>Стриминг экрана</h3>
          <p>Трансляция до 1080p60 с аппаратным H264 кодированием.</p>
        </div>
        <div class="card feature-card">
          <div class="feature-icon">🔒</div>
          <h3>Self-hosted</h3>
          <p>Поднимай собственный сервер. Твои данные — только у тебя.</p>
        </div>
        <div class="card feature-card">
          <div class="feature-icon">🌍</div>
          <h3>Кросс-платформа</h3>
          <p>Windows, Linux, macOS. Нативный клиент без Electron-overhead.</p>
        </div>
        <div class="card feature-card">
          <div class="feature-icon">💬</div>
          <h3>Текстовый чат</h3>
          <p>Markdown, реакции, ответы на сообщения, @упоминания.</p>
        </div>
      </div>
    </section>

    <!-- Roadmap -->
    <section class="roadmap container">
      <h2 class="section-title">Roadmap</h2>
      <p class="section-sub">Проект в активной разработке. Этапы идут последовательно.</p>
      <div class="roadmap-list">
        <div v-for="stage in stages" :key="stage.n" class="roadmap-item" :class="stage.status">
          <div class="roadmap-dot"></div>
          <div class="roadmap-body">
            <div class="roadmap-header">
              <span class="roadmap-num">Этап {{ stage.n }}</span>
              <span class="roadmap-badge">{{ stage.label }}</span>
            </div>
            <div class="roadmap-title">{{ stage.title }}</div>
            <div class="roadmap-desc">{{ stage.desc }}</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Как запустить -->
    <section class="deploy container">
      <h2 class="section-title">Запустить свой сервер</h2>
      <p class="section-sub">Docker Compose — единственное что нужно.</p>
      <div class="deploy-steps">
        <div class="deploy-step">
          <div class="step-num">1</div>
          <div class="step-body">
            <div class="step-title">Скачай compose файл</div>
            <pre class="code-block">curl -O https://raw.githubusercontent.com/Beykus-Y/beyvox/main/server/docker-compose.yml</pre>
          </div>
        </div>
        <div class="deploy-step">
          <div class="step-num">2</div>
          <div class="step-body">
            <div class="step-title">Создай .env</div>
            <pre class="code-block">DB_PASSWORD=your_strong_password
LK_SECRET=your_livekit_secret
LIVEKIT_URL=wss://live.yourdomain.com
OWNER_USERNAME=your_beyvox_username</pre>
          </div>
        </div>
        <div class="deploy-step">
          <div class="step-num">3</div>
          <div class="step-body">
            <div class="step-title">Запусти</div>
            <pre class="code-block">docker compose up -d</pre>
          </div>
        </div>
        <div class="deploy-step">
          <div class="step-num">4</div>
          <div class="step-body">
            <div class="step-title">Подключись в клиенте</div>
            <p class="step-desc">Скачай клиент, войди в аккаунт и добавь свой сервер по IP или домену.</p>
          </div>
        </div>
      </div>
    </section>

    <!-- FAQ -->
    <section class="faq container">
      <h2 class="section-title">FAQ</h2>
      <div class="faq-list">
        <div v-for="(item, i) in faq" :key="i" class="faq-item card" @click="toggleFaq(i)">
          <div class="faq-q">
            {{ item.q }}
            <svg class="faq-arrow" :class="{ open: openFaq === i }" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M7 10l5 5 5-5H7z"/></svg>
          </div>
          <div class="faq-a" v-if="openFaq === i">{{ item.a }}</div>
        </div>
      </div>
    </section>

    <!-- CTA -->
    <section class="cta container">
      <div class="card cta-card">
        <h2>Готов попробовать?</h2>
        <p>Скачай клиент или подними свой сервер прямо сейчас.</p>
        <div class="cta-actions">
          <a href="https://github.com/Beykus-Y/beyvox/releases/tag/v0.0.2" target="_blank" class="btn btn-primary">Скачать клиент</a>
          <RouterLink to="/register" class="btn btn-ghost">Создать аккаунт</RouterLink>
        </div>
      </div>
    </section>
  </main>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const openFaq = ref<number | null>(null)
function toggleFaq(i: number) {
  openFaq.value = openFaq.value === i ? null : i
}

const stages = [
  { n: 1, status: 'done', label: 'Готово', title: 'Foundation', desc: 'Авторизация, JWT, JWKS, каталог серверов, базовая структура клиента.' },
  { n: 2, status: 'done', label: 'Готово', title: 'Voice MVP', desc: 'Голосовые каналы, LiveKit SFU, PTT, VAD, индикация говорящих.' },
  { n: 3, status: 'done', label: 'Готово', title: 'Text Chat', desc: 'Сообщения в реальном времени, Markdown, реакции, @упоминания, создание каналов.' },
  { n: 4, status: 'planned', label: 'Планируется', title: 'Roles & Moderation', desc: 'Полная матрица прав, кик, мут, бан, channel overrides.' },
  { n: 5, status: 'planned', label: 'Планируется', title: 'Files & Media', desc: 'Вложения до 25 МБ, превью изображений и видео, Open Graph.' },
  { n: 6, status: 'planned', label: 'Планируется', title: 'Screen Share', desc: 'Захват экрана/окна, 720p/1080p, simulcast H264+VP8.' },
  { n: 7, status: 'planned', label: 'Планируется', title: 'VST Plugins', desc: 'Хостинг VST2/VST3, цепочка эффектов, нативный GUI плагина.' },
  { n: 8, status: 'planned', label: 'Планируется', title: 'Polish', desc: 'Десктопные уведомления, статусы, офлайн-кэш, rate limiting, паблик-релиз.' },
]

const faq = [
  { q: 'Это бесплатно?', a: 'Да, BeyVox полностью бесплатен и с открытым исходным кодом. Клиент и серверная часть распространяются под лицензией MIT.' },
  { q: 'Нужен ли аккаунт на beyvox.beykus.fun для использования?', a: 'Да — аккаунт нужен для авторизации на всех серверах. JWT токен верифицируется локально через JWKS, так что серверу не нужен постоянный доступ к центральному сервису.' },
  { q: 'Насколько это безопасно?', a: 'Данные хранятся только на твоём сервере. Голос передаётся через WebRTC (DTLS-SRTP). Центральный сервер не имеет доступа к сообщениям и звонкам.' },
  { q: 'В чём отличие от TeamSpeak / Mumble?', a: 'Современный стек (WebRTC, Tauri, Vue 3), текстовый чат с реакциями и @упоминаниями, VST плагины, стриминг экрана. При этом такой же self-hosted подход.' },
  { q: 'Когда будет мобильный клиент?', a: 'Мобильный клиент запланирован, но не входит в v1.0. Сначала нужно довести до стабильного состояния десктопную версию.' },
]
</script>

<style scoped>
.hero {
  padding: 100px 0 80px;
  text-align: center;
}
.badge {
  display: inline-block;
  background: rgba(91, 124, 246, 0.12);
  border: 1px solid rgba(91, 124, 246, 0.3);
  color: var(--accent);
  padding: 4px 14px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  margin-bottom: 24px;
  letter-spacing: 0.5px;
}
h1 {
  font-size: clamp(32px, 5vw, 56px);
  font-weight: 800;
  line-height: 1.15;
  margin-bottom: 20px;
  letter-spacing: -1px;
}
.accent { color: var(--accent); }
.subtitle {
  color: var(--text2);
  font-size: 18px;
  margin-bottom: 36px;
  line-height: 1.7;
}
.hero-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}

.features { padding: 60px 0; }
.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}
.feature-card { display: flex; flex-direction: column; gap: 10px; }
.feature-icon { font-size: 28px; }
.feature-card h3 { font-size: 16px; font-weight: 600; }
.feature-card p { color: var(--text2); font-size: 14px; }

.section-title { font-size: 28px; font-weight: 700; margin-bottom: 8px; }
.section-sub { color: var(--text2); margin-bottom: 36px; font-size: 15px; }

/* Roadmap */
.roadmap { padding: 60px 0; }
.roadmap-list {
  display: flex;
  flex-direction: column;
  gap: 0;
  position: relative;
  padding-left: 20px;
  border-left: 2px solid var(--border);
}
.roadmap-item {
  display: flex;
  gap: 20px;
  padding: 0 0 28px 24px;
  position: relative;
}
.roadmap-dot {
  position: absolute;
  left: -8px;
  top: 4px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--border);
  border: 2px solid var(--bg-dark);
  flex-shrink: 0;
}
.roadmap-item.done .roadmap-dot { background: var(--accent); }
.roadmap-item.active .roadmap-dot { background: var(--green); box-shadow: 0 0 8px var(--green); }
.roadmap-body { display: flex; flex-direction: column; gap: 4px; }
.roadmap-header { display: flex; align-items: center; gap: 10px; margin-bottom: 2px; }
.roadmap-num { font-size: 12px; color: var(--text3); }
.roadmap-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--bg-hover);
  color: var(--text2);
}
.roadmap-item.done .roadmap-badge { background: rgba(91,124,246,0.15); color: var(--accent); }
.roadmap-item.active .roadmap-badge { background: rgba(67,181,129,0.15); color: var(--green); }
.roadmap-title { font-weight: 600; font-size: 15px; }
.roadmap-desc { color: var(--text2); font-size: 13px; line-height: 1.5; }

/* Deploy */
.deploy { padding: 60px 0; }
.deploy-steps { display: flex; flex-direction: column; gap: 24px; }
.deploy-step { display: flex; gap: 20px; align-items: flex-start; }
.step-num {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--accent);
  color: white;
  font-weight: 700;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
}
.step-body { display: flex; flex-direction: column; gap: 8px; flex: 1; }
.step-title { font-weight: 600; font-size: 15px; }
.step-desc { color: var(--text2); font-size: 14px; margin: 0; }
.code-block {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px 18px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  color: var(--green);
  overflow-x: auto;
  margin: 0;
  white-space: pre;
}

/* FAQ */
.faq { padding: 60px 0; }
.faq-list { display: flex; flex-direction: column; gap: 8px; }
.faq-item { padding: 16px 20px; cursor: pointer; user-select: none; }
.faq-q {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-weight: 500;
  font-size: 15px;
  gap: 12px;
}
.faq-arrow { flex-shrink: 0; color: var(--text3); transition: transform 0.2s; }
.faq-arrow.open { transform: rotate(180deg); }
.faq-a { color: var(--text2); font-size: 14px; line-height: 1.6; margin-top: 12px; }

/* CTA */
.cta { padding: 40px 0 80px; }
.cta-card { text-align: center; padding: 48px; }
.cta-card h2 { font-size: 28px; font-weight: 700; margin-bottom: 10px; }
.cta-card p { color: var(--text2); margin-bottom: 28px; }
.cta-actions { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
</style>
