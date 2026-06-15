-- username хранится для @упоминаний без запросов к central
ALTER TABLE members ADD COLUMN IF NOT EXISTS username VARCHAR(100) NOT NULL DEFAULT '';

-- список упомянутых user_id (заполняется при send_message)
ALTER TABLE messages ADD COLUMN IF NOT EXISTS mention_user_ids UUID[] NOT NULL DEFAULT '{}';
