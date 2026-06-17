CREATE TABLE guild_snapshots (
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    guild_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    member_count INTEGER NOT NULL DEFAULT 0,
    is_default BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server_id, guild_id)
);

CREATE INDEX idx_guild_snapshots_server_id ON guild_snapshots(server_id);
