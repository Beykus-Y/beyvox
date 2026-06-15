CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Сервер (сообщество/гильдия)
CREATE TABLE guilds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    icon_url TEXT,
    owner_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Каналы
CREATE TYPE channel_type AS ENUM ('text', 'voice');

CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    type channel_type NOT NULL DEFAULT 'text',
    position INTEGER NOT NULL DEFAULT 0,
    user_limit INTEGER, -- только для voice, NULL = без лимита
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_channels_guild_id ON channels(guild_id);

-- Роли
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7), -- hex: #rrggbb
    position INTEGER NOT NULL DEFAULT 0,
    -- Права (битовая маска)
    permissions BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_roles_guild_id ON roles(guild_id);

-- Константы прав (для документации, используются в коде)
-- ADMINISTRATOR     = 1 << 0  = 1
-- MANAGE_CHANNELS   = 1 << 1  = 2
-- MANAGE_ROLES      = 1 << 2  = 4
-- MANAGE_MEMBERS    = 1 << 3  = 8
-- SEND_MESSAGES     = 1 << 4  = 16
-- ATTACH_FILES      = 1 << 5  = 32
-- CONNECT_VOICE     = 1 << 6  = 64
-- STREAM_SCREEN     = 1 << 7  = 128
-- MUTE_MEMBERS      = 1 << 8  = 256
-- BAN_MEMBERS       = 1 << 9  = 512

-- Участники
CREATE TABLE members (
    user_id UUID NOT NULL,
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    nickname VARCHAR(32),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_muted BOOLEAN NOT NULL DEFAULT false,
    is_banned BOOLEAN NOT NULL DEFAULT false,
    timeout_until TIMESTAMPTZ,
    PRIMARY KEY (user_id, guild_id)
);

CREATE INDEX idx_members_guild_id ON members(guild_id);

-- Роли участников
CREATE TABLE member_roles (
    user_id UUID NOT NULL,
    guild_id UUID NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, guild_id, role_id),
    FOREIGN KEY (user_id, guild_id) REFERENCES members(user_id, guild_id) ON DELETE CASCADE
);

-- Переопределения прав на уровне канала
CREATE TABLE channel_permission_overrides (
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    allow BIGINT NOT NULL DEFAULT 0,
    deny BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (channel_id, role_id)
);

-- Сообщения
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    edited_at TIMESTAMPTZ,
    reply_to UUID REFERENCES messages(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_channel_id_created_at ON messages(channel_id, created_at DESC);

-- Реакции на сообщения
CREATE TABLE message_reactions (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    emoji VARCHAR(64) NOT NULL,
    PRIMARY KEY (message_id, user_id, emoji)
);

-- Голосовые состояния (кто в каком канале)
CREATE TABLE voice_states (
    user_id UUID NOT NULL,
    guild_id UUID NOT NULL,
    channel_id UUID REFERENCES channels(id) ON DELETE SET NULL,
    is_muted BOOLEAN NOT NULL DEFAULT false,
    is_deafened BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, guild_id)
);

CREATE INDEX idx_voice_states_channel_id ON voice_states(channel_id);

-- Инвайт-ссылки
CREATE TABLE invites (
    code VARCHAR(16) PRIMARY KEY,
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    created_by UUID NOT NULL,
    expires_at TIMESTAMPTZ,
    max_uses INTEGER,
    uses INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
