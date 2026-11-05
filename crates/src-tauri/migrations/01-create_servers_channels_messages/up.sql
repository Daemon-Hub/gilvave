-- Таблица серверов
CREATE TABLE servers (
    id TEXT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    owner_id TEXT DEFAULT '',
    icon_url TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    cover TEXT NOT NULL DEFAULT '',
    is_public INTEGER NOT NULL DEFAULT 0,
    members_count INTEGER NOT NULL DEFAULT 0,
    is_owner INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Таблица каналов
CREATE TABLE channels (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name VARCHAR(100) NOT NULL,
    type TEXT NOT NULL DEFAULT 'TEXT',
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    edited INTEGER DEFAULT 0,
    event_count INTEGER DEFAULT 0
);

-- Таблица сообщений
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL,
    author_id TEXT,
    author_name VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    edited INTEGER NOT NULL DEFAULT 0,
    reply_to_id TEXT,
    forwarded_from TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_pending INTEGER NOT NULL DEFAULT 0
);

-- Таблица токенов авторизации (для локального SQLite на Desktop / Android)
CREATE TABLE IF NOT EXISTS auth_tokens (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Индексы производительности
CREATE INDEX IF NOT EXISTS idx_servers_position ON servers(position ASC);
CREATE INDEX IF NOT EXISTS idx_channels_server_id ON channels(server_id);
CREATE INDEX IF NOT EXISTS idx_channels_position ON channels(position ASC);
CREATE INDEX IF NOT EXISTS idx_messages_channel_created ON messages(channel_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_messages_channel_created_desc ON messages(channel_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_author_id ON messages(author_id);
