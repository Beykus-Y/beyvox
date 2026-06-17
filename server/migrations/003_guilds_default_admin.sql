ALTER TABLE guilds ADD COLUMN is_default BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE guilds ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT true;

-- Если уже есть гильдии — первая по дате создания становится дефолтной
UPDATE guilds SET is_default = true
WHERE id = (SELECT id FROM guilds ORDER BY created_at LIMIT 1);

-- Локальные аккаунты для admin panel (не связаны с beyvox auth)
CREATE TABLE server_admins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(64) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Сессии admin panel
CREATE TABLE admin_sessions (
    token UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    admin_id UUID NOT NULL REFERENCES server_admins(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_admin_sessions_expires_at ON admin_sessions(expires_at);
