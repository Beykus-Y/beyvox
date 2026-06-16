-- Заменяем case-sensitive UNIQUE на регистронезависимый уникальный индекс.
-- Без этого Alice и alice могут одновременно существовать в системе,
-- но при логине по username LOWER() найдёт первого попавшегося.
ALTER TABLE users DROP CONSTRAINT users_username_key;
CREATE UNIQUE INDEX users_username_lower_key ON users (LOWER(username));
