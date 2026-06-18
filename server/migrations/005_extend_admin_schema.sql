-- Глобальные баны на уровне инстанса
CREATE TABLE IF NOT EXISTS banned_users (
    user_id UUID PRIMARY KEY,
    username VARCHAR(100) NOT NULL,
    banned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Настройки инстанса, изменяемые через админку
CREATE TABLE IF NOT EXISTS server_settings (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL
);
