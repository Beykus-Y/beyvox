# BeyVox — TODO

Roadmap без дедлайнов. Делаем этап за этапом, не распыляясь.

---

## Этап 1 — Foundation

Цель: рабочая авторизация и каталог серверов.

- [x] Инициализировать репозитории (`beyvox-server`, `beyvox-central`, `beyvox-client`, `beyvox-proto`)
- [x] `beyvox-central`: регистрация пользователя (login + email + password)
- [x] `beyvox-central`: логин, выдача JWT access + refresh токенов
- [x] `beyvox-central`: refresh rotation endpoint
- [x] `beyvox-central`: JWKS endpoint (`/.well-known/jwks.json`)
- [x] `beyvox-central`: API каталога серверов (регистрация, листинг, пинг живости)
- [x] `beyvox-server`: скелет Axum сервера, подключение PostgreSQL, миграции
- [x] `beyvox-server`: верификация JWT через JWKS (с кэшированием ключа)
- [x] `beyvox-server`: создание сервера (сообщества), инвайт-ссылки
- [x] `beyvox-server`: создание каналов (текстовых и голосовых)
- [x] `beyvox-server`: базовая система ролей (ADMINISTRATOR, MANAGE_CHANNELS, etc.)
- [x] `beyvox-client`: скелет Tauri + Vue 3 + Vite проекта
- [x] `beyvox-client`: экран логина / регистрации
- [x] `beyvox-client`: главный layout (сайдбар серверов, список каналов, основная область)
- [ ] `beyvox-client`: просмотр каталога публичных серверов (включая guilds из central)
- [x] `beyvox-server`: is_default / is_public флаги на гильдиях, auto-join в default при вступлении
- [x] `beyvox-server`: setup endpoint (первый запуск — создание admin + default guild)
- [x] `beyvox-server`: admin panel (`/admin`) — управление гильдиями, участниками
- [x] `beyvox-server`: `GET /api/discovery` — публичный endpoint для central pull
- [x] `beyvox-central`: pull loop (каждые 5 мин опрашивает серверы, сохраняет guild snapshots)
- [x] `beyvox-central`: `GET /servers` возвращает guilds в ответе
- [x] `beyvox-client`: подключение к серверу по прямому IP/домену

---

## Этап 2 — Voice MVP

Цель: можно зайти в голосовой канал и поговорить.

- [x] Развернуть LiveKit сервер (Docker)
- [x] `beyvox-server`: интеграция с LiveKit (создание room, выдача токенов)
- [x] `beyvox-server`: WebSocket signaling (`VOICE_STATE_UPDATE`, `VOICE_SERVER_UPDATE`)
- [x] `beyvox-client`: подключение к голосовому каналу через LiveKit Rust SDK (cpal + NativeAudioSource/Stream)
- [x] `beyvox-client`: выбор устройств (микрофон, динамики/наушники)
- [x] `beyvox-client`: PTT (Push-to-Talk) — настройка клавиши
- [ ] `beyvox-client`: VAD (Voice Activity Detection) — убрано, будет через VST или RNNoise
- [x] `beyvox-client`: индикация говорящих участников
- [x] `beyvox-client`: mute себя / deafen
- [x] `beyvox-client`: индивидуальная регулировка громкости участников
- [x] `beyvox-client`: отображение участников в голосовом канале

---

## Этап 3 — Text Chat

Цель: полноценный текстовый чат внутри серверов.

- [x] `beyvox-server`: WebSocket `MESSAGE_CREATE`, история с пагинацией
- [x] `beyvox-server`: редактирование и удаление сообщений
- [x] `beyvox-server`: упоминания @пользователь (парсинг + хранение + broadcast с mention_user_ids)
- [x] `beyvox-client`: отправка/получение сообщений в реальном времени
- [x] `beyvox-client`: markdown рендеринг (bold, italic, code, codeblock)
- [x] `beyvox-client`: lazy load истории при скролле вверх
- [x] `beyvox-client`: reply на сообщение с превью
- [x] `beyvox-client`: реакции emoji на сообщения (пикер + real-time через WS)
- [x] `beyvox-client`: уведомления при упоминании (значок @ в списке каналов)
- [x] `beyvox-client`: создание текстового и голосового канала через UI
- [x] `beyvox-server`: реакции (PUT/DELETE + REACTION_ADD/REMOVE через WS)

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

- [x] `beyvox-client`: нативный захват экрана (screenshots crate — Windows/Mac/Linux)
- [x] `beyvox-client`: выбор монитора (список через `list_screens`)
- [x] `beyvox-client`: настройки качества (360p/720p/1080p, 15/30/60 fps)
- [x] LiveKit: публикация LocalVideoTrack с TrackSource::Screenshare (H264)
- [x] `beyvox-client`: UI просмотра стрима (ScreenViewer с canvas/JPEG кадрами через Tauri events)
- [x] `beyvox-server`: WS опкод SCREEN_SHARE_STATE_UPDATE (сигналинг, STREAM_SCREEN право)
- [ ] LiveKit simulcast: VP8 фолбэк
- [ ] Захват конкретного окна (а не только монитора)
- [ ] Опционально: захват аудио рабочего стола (WASAPI/CoreAudio)

---

## Этап 7 — VST Plugins

Цель: пользователь загружает VST плагин, он применяется к микрофону.

- [x] `beyvox-client`: UI загрузки VST файла в настройках аудио
- [x] `beyvox-client`: захват микрофона через `cpal` (основа аудиопайплайна)
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
- [x] Автоматическое переподключение (exponential backoff)
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
