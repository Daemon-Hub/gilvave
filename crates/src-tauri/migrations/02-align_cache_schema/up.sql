-- Align servers table to match Server and ServerSmallPart DTOs
CREATE TABLE servers_new (
    id TEXT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    owner_id TEXT DEFAULT '',
    icon_url TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    cover TEXT NOT NULL DEFAULT '',
    is_public INTEGER NOT NULL DEFAULT 0,
    members_count INTEGER NOT NULL DEFAULT 0,
    is_owner INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO servers_new (id, name, owner_id, icon_url, is_public, created_at)
SELECT id, name, owner_id, icon_url, is_public, created_at FROM servers;

DROP TABLE servers;
ALTER TABLE servers_new RENAME TO servers;

-- Align channels table to match ChannelView DTO
CREATE TABLE channels_new (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name VARCHAR(100) NOT NULL,
    type TEXT NOT NULL DEFAULT 'TEXT',
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    edited INTEGER DEFAULT 0,
    event_count INTEGER DEFAULT 0
);

INSERT OR IGNORE INTO channels_new (id, server_id, name, type, position, created_at, edited, event_count)
SELECT id, server_id, name, UPPER(type), position, created_at, edited, event_count FROM channels;

DROP TABLE channels;
ALTER TABLE channels_new RENAME TO channels;

-- Align messages table to match MessageView DTO
CREATE TABLE messages_new (
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

INSERT OR IGNORE INTO messages_new (id, channel_id, author_id, author_name, content, created_at)
SELECT id, channel_id, author_id, author_name, content, created_at FROM messages;

DROP TABLE messages;
ALTER TABLE messages_new RENAME TO messages;

-- Indices for performance
CREATE INDEX IF NOT EXISTS idx_messages_channel_created ON messages(channel_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_messages_channel_created_desc ON messages(channel_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_author_id ON messages(author_id);
CREATE INDEX IF NOT EXISTS idx_channels_server_id ON channels(server_id);
