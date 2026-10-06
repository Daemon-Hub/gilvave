-- Таблица серверов
CREATE TABLE servers (
    id TEXT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    owner_id TEXT NOT NULL, -- Владелец сервера (UUID)
    icon_url TEXT NOT NULL, -- Иконка сервера
    is_public INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE channels (
    id TEXT PRIMARY KEY,
    server_id TEXT REFERENCES servers(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    type TEXT NOT NULL DEFAULT 'text' CHECK (type IN ('text', 'voice')),
    position INTEGER NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    edited INTEGER DEFAULT 0,
    event_count INTEGER DEFAULT 0
);

-- Таблица сообщений
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    author_id TEXT NOT NULL,
    author_name VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Индексы
CREATE INDEX idx_messages_channel_id ON messages(channel_id);
CREATE INDEX idx_messages_author_id ON messages(author_id);
CREATE INDEX idx_channels_server_id ON channels(server_id);
CREATE INDEX idx_messages_channel_covering 
ON messages(channel_id, created_at DESC);