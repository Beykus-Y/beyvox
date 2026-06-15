# BeyVox — TODO

Roadmap без дедлайнов. Делаем этап за этапом, не распыляясь.

---

## Этап 1 — Foundation

Цель: рабочая авторизация и каталог серверов.

- [ ] Инициализировать репозитории (`beyvox-server`, `beyvox-central`, `beyvox-client`, `beyvox-proto`)
- [ ] `beyvox-central`: регистрация пользователя (login + email + password)
- [ ] `beyvox-central`: логин, выдача JWT access + refresh токенов
- [ ] `beyvox-central`: refresh rotation endpoint
- [ ] `beyvox-central`: JWKS endpoint (`/.well-known/jwks.json`)
- [ ] `beyvox-central`: API каталога серверов (регистрация, листинг, пинг живости)
- [ ] `beyvox-server`: скелет Axum сервера, подключение PostgreSQL, миграции
- [ ] `beyvox-server`: верификация JWT через JWKS (с кэшированием ключа)
- [ ] `beyvox-server`: создание сервера (сообщества), инвайт-ссылки
- [ ] `beyvox-server`: создание каналов (текстовых и голосовых)
- [ ] `beyvox-server`: базовая система ролей (ADMINISTRATOR, MANAGE_CHANNELS, etc.)
- [ ] `beyvox-client`: скелет Tauri + Vue 3 + Vite проекта
- [ ] `beyvox-client`: экран логина / регистрации
- [ ] `beyvox-client`: главный layout (сайдбар серверов, список каналов, основная область)
- [ ] `beyvox-client`: просмотр каталога публичных серверов
- [ ] `beyvox-client`: подключение к серверу по прямому IP/домену

---

## Этап 2 — Voice MVP

Цель: можно зайти в голосовой канал и поговорить.

- [ ] Развернуть LiveKit сервер (Docker)
- [ ] `beyvox-server`: интеграция с LiveKit (создание room, выдача токенов)
- [ ] `beyvox-server`: WebSocket signaling (`VOICE_STATE_UPDATE`, `VOICE_SERVER_UPDATE`)
- [ ] `beyvox-client`: подключение к голосовому каналу через LiveKit Rust SDK
- [ ] `beyvox-client`: захват микрофона через `cpal`
- [ ] `beyvox-client`: выбор устройств (микрофон, динамики/наушники)
- [ ] `beyvox-client`: PTT (Push-to-Talk) — настройка клавиши
- [ ] `beyvox-client`: VAD (Voice Activity Detection)
- [ ] `beyvox-client`: индикация говорящих участников
- [ ] `beyvox-client`: mute себя / deafen
- [ ] `beyvox-client`: индивидуальная регулировка громкости участников
- [ ] `beyvox-client`: отображение участников в голосовом канале

---

## Этап 3 — Text Chat

Цель: полноценный текстовый чат внутри серверов.

- [ ] `beyvox-server`: WebSocket `MESSAGE_CREATE`, история с пагинацией
- [ ] `beyvox-server`: редактирование и удаление сообщений
- [ ] `beyvox-server`: упоминания @пользователь и @роль
- [ ] `beyvox-client`: отправка/получение сообщений в реальном времени
- [ ] `beyvox-client`: markdown рендеринг (bold, italic, code, codeblock)
- [ ] `beyvox-client`: lazy load истории при скролле вверх
- [ ] `beyvox-client`: reply на сообщение с превью
- [ ] `beyvox-client`: реакции emoji на сообщения
- [ ] `beyvox-client`: уведомления при упоминании

---

## Этап 4 — Roles & Moderation

Цель: полная система прав и модерация.

- [ ] `beyvox-server`: полная матрица прав (`ADMINISTRATOR`, `MANAGE_CHANNELS`, `MANAGE_ROLES`, `MANAGE_MEMBERS`, `SEND_MESSAGES`, `ATTACH_FILES`, `CONNECT_VOICE`, `STREAM_SCREEN`, `MUTE_MEMBERS`, `BAN_MEMBERS`)
- [ ] `beyvox-server`: channel overrides (allow/deny на уровне канала)
- [ ] `beyvox-server`: кик, мут, таймаут, бан участников
- [ ] `beyvox-client`: UI управления ролями
- [ ] `beyvox-client`: UI модерации участников
- [ ] `beyvox-client`: инвайт-ссылки с опциональным сроком и лимитом

---

## Этап 5 — Files & Media

Цель: вложения, превью, медиа в чате.

- [ ] `beyvox-server`: загрузка файлов (multipart, лимит 25 МБ)
- [ ] `beyvox-server`: локальное хранилище или S3-совместимое
- [ ] `beyvox-client`: прикрепление файлов к сообщениям, прогресс-бар
- [ ] `beyvox-client`: inline превью изображений (JPEG, PNG, GIF, WebP)
- [ ] `beyvox-client`: превью видео с миниатюрой
- [ ] `beyvox-client`: Open Graph превью для ссылок
- [ ] `beyvox-client`: скачивание файлов

---

## Этап 6 — Screen Share

Цель: трансляция экрана/окна в голосовом канале.

- [ ] `beyvox-client`: нативный захват экрана (Windows DXGI, Mac ScreenCaptureKit, Linux PipeWire)
- [ ] `beyvox-client`: выбор — весь экран или конкретное окно
- [ ] `beyvox-client`: настройки качества (360p/720p/1080p, 15/30/60 fps)
- [ ] LiveKit simulcast: H264 основной + VP8 фолбэк
- [ ] `beyvox-client`: UI просмотра стрима участников
- [ ] Опционально: захват аудио рабочего стола (WASAPI/CoreAudio)

---

## Этап 7 — VST Plugins

Цель: пользователь загружает VST плагин, он применяется к микрофону.

- [ ] `beyvox-client`: UI загрузки VST файла в настройках аудио
- [ ] `beyvox-client`: хостинг VST2 (`.dll`/`.so`/`.dylib`) через крейт `vst`
- [ ] `beyvox-client`: хостинг VST3 (`.vst3`) через `vst3-sys`
- [ ] `beyvox-client`: цепочка эффектов — порядок плагинов настраивается
- [ ] `beyvox-client`: открытие нативного GUI плагина (HWND/NSView/XWindow)
- [ ] `beyvox-client`: сохранение настроек плагина в SQLite
- [ ] `beyvox-client`: bypass / включение плагина без удаления

---

## Этап 8 — Polish

Цель: готово к публичному релизу.

- [ ] Десктопные уведомления (упоминания, входящие сообщения)
- [ ] Статус пользователя: онлайн / не беспокоить / отошёл / невидимый
- [ ] Смена аватара, username, пароля
- [ ] Автоматическое переподключение (exponential backoff)
- [ ] Offline режим: чтение истории из SQLite кэша
- [ ] Rate limiting на API (брутфорс, спам)
- [ ] Опциональное шумоподавление через RNNoise
- [ ] Упаковка: NSIS installer (Windows), AppImage (Linux), DMG (macOS)
- [ ] Страница каталога серверов на `beyvox.beykus.fun`

---

## Идеи на будущее (не в v1.0)

- E2EE голоса (DTLS-SRTP с per-user ключами)
- DM между пользователями
- Мобильный клиент
- Браузерный клиент
- Боты и публичный API
- Видеозвонки
