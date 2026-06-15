<template>
  <main>
    <section class="hero">
      <div class="container">
        <div class="badge">Open Source · Self-Hosted</div>
        <h1>Голосовой мессенджер<br><span class="accent">под твоим контролем</span></h1>
        <p class="subtitle">
          BeyVox — альтернатива Discord и TeamSpeak.<br>
          Поднимай свой сервер, общайся голосом, никаких посредников.
        </p>
        <div class="hero-actions">
          <a href="#" class="btn btn-primary">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 16l-6-6h4V4h4v6h4l-6 6zm-7 2h14v2H5v-2z"/></svg>
            Скачать клиент
          </a>
          <RouterLink to="/servers" class="btn btn-ghost">Найти сервер</RouterLink>
        </div>
      </div>
    </section>

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
          <p>Markdown, реакции, ответы, вложения файлов до 25 МБ.</p>
        </div>
      </div>
    </section>

    <section class="cta container">
      <div class="card cta-card">
        <h2>Своя инфраструктура за 5 минут</h2>
        <p>Один Docker контейнер — и сервер готов к работе.</p>
        <pre class="code-block">docker run -d --name beyvox-server \
  -p 8080:8080 \
  -e DOMAIN=my.server.com \
  beykus/beyvox-server</pre>
        <RouterLink to="/register" class="btn btn-primary">Создать аккаунт</RouterLink>
      </div>
    </section>
  </main>
</template>

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
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
}
.feature-card { display: flex; flex-direction: column; gap: 10px; }
.feature-icon { font-size: 28px; }
.feature-card h3 { font-size: 16px; font-weight: 600; }
.feature-card p { color: var(--text2); font-size: 14px; }

.cta { padding: 40px 0 80px; }
.cta-card { text-align: center; padding: 48px; }
.cta-card h2 { font-size: 28px; font-weight: 700; margin-bottom: 10px; }
.cta-card p { color: var(--text2); margin-bottom: 24px; }
.code-block {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px 20px;
  text-align: left;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  color: var(--green);
  margin-bottom: 24px;
  overflow-x: auto;
}
</style>
