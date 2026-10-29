use gilvave_core::{
    dto::{
        messages::MessageView,
        servers::{ChannelType, ChannelView, ServerSmallPart},
    },
    ids::{ChannelId, MessageId, ServerId, UserId},
    uuid::Uuid,
};
use include_dir::{Dir, include_dir};
use rusqlite::{Connection, params};
use rusqlite_migration::Migrations;
use std::{collections::HashSet, fs, sync::LazyLock};
use tauri::{AppHandle, Manager};
use time::format_description::well_known::Rfc3339;

/// Удобный алиас: принимает rusqlite::Error, io::Error, tauri::Error,
/// rusqlite_migration::Error и любые другие std::error::Error.
pub type BoxResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Макрос include_dir! встраивает всё содержимое каталога в бинарь на этапе компиляции.
// Путь $CARGO_MANIFEST_DIR указывает на каталог с Cargo.toml текущего крейта.
static MIGRATIONS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/migrations");

static MIGRATIONS: LazyLock<Migrations<'static>> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).unwrap());

pub fn initialize_database(app_handle: &AppHandle) -> BoxResult<Connection> {
    tracing::debug!("initialize_database");
    let app_dir = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("app_local_data_dir: {e}"))?;
    fs::create_dir_all(&app_dir).map_err(|e| format!("create_dir_all: {e}"))?;

    let sqlite_path = app_dir.join("gilvave.sqlite");
    tracing::debug!(?sqlite_path, "открываю БД");

    let mut conn = Connection::open(&sqlite_path)
        .map_err(|e| format!("open {}: {e}", sqlite_path.display()))?;

    // PRAGMA вне транзакций (иначе no-op)
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("journal_mode=WAL: {e}"))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| format!("foreign_keys=ON: {e}"))?;

    MIGRATIONS
        .to_latest(&mut conn)
        .map_err(|e| format!("migrations: {e}"))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS auth_tokens (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("auth_tokens table: {e}"))?;

    Ok(conn)
}

pub fn get_token(conn: &Connection, key: &str) -> BoxResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM auth_tokens WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_token(conn: &Connection, key: &str, value: &str) -> BoxResult<()> {
    if value.is_empty() {
        conn.execute("DELETE FROM auth_tokens WHERE key = ?1", params![key])?;
    } else {
        conn.execute(
            "INSERT INTO auth_tokens (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
    }
    Ok(())
}

fn row_to_message_view(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageView> {
    let id_str: String = row.get(0)?;
    let ch_str: String = row.get(1)?;
    let author_str: Option<String> = row.get(2)?;
    let author_name: String = row.get(3)?;
    let content: String = row.get(4)?;
    let edited_num: i64 = row.get(5)?;
    let reply_str: Option<String> = row.get(6)?;
    let forwarded_str: Option<String> = row.get(7)?;
    let created_str: String = row.get(8)?;
    let pending_num: i64 = row.get(9)?;

    let id = MessageId(Uuid::parse_str(&id_str).unwrap_or_default());
    let channel_id = ChannelId(Uuid::parse_str(&ch_str).unwrap_or_default());
    let author_id = author_str
        .and_then(|s| Uuid::parse_str(&s).ok())
        .map(UserId);
    let reply_to_id = reply_str
        .and_then(|s| Uuid::parse_str(&s).ok())
        .map(MessageId);
    let forwarded_from = forwarded_str.and_then(|s| serde_json::from_str(&s).ok());
    let created_at = time::OffsetDateTime::parse(&created_str, &Rfc3339)
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());

    Ok(MessageView {
        id,
        channel_id,
        author_id,
        author_name,
        content,
        edited: edited_num != 0,
        reply_to_id,
        forwarded_from,
        created_at,
        is_pending: pending_num != 0,
    })
}

pub fn get_cached_messages(
    conn: &Connection,
    channel_id: ChannelId,
    limit: usize,
    before: Option<MessageId>,
) -> rusqlite::Result<Vec<MessageView>> {
    let mut messages = if let Some(before_id) = before {
        let mut stmt = conn.prepare(
            "SELECT id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at, is_pending
             FROM messages
             WHERE channel_id = ?1
               AND created_at < (SELECT created_at FROM messages WHERE id = ?2)
             ORDER BY created_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![channel_id.to_string(), before_id.to_string(), limit as i64],
            row_to_message_view,
        )?;
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at, is_pending
             FROM messages
             WHERE channel_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![channel_id.to_string(), limit as i64],
            row_to_message_view,
        )?;
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    };

    // Return in chronological order (oldest -> newest) for the UI
    messages.reverse();
    Ok(messages)
}

pub fn save_cached_message(conn: &Connection, message: &MessageView) -> rusqlite::Result<()> {
    let author_str = message.author_id.map(|a| a.to_string());
    let reply_str = message.reply_to_id.map(|r| r.to_string());
    let forwarded_str = message
        .forwarded_from
        .as_ref()
        .and_then(|v| serde_json::to_string(v).ok());
    let created_str = message
        .created_at
        .format(&Rfc3339)
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap());

    conn.execute(
        "INSERT INTO messages (id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at, is_pending)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
             content = excluded.content,
             edited = excluded.edited,
             reply_to_id = excluded.reply_to_id,
             forwarded_from = excluded.forwarded_from,
             is_pending = excluded.is_pending;",
        params![
            message.id.to_string(),
            message.channel_id.to_string(),
            author_str,
            message.author_name,
            message.content,
            if message.edited { 1 } else { 0 },
            reply_str,
            forwarded_str,
            created_str,
            if message.is_pending { 1 } else { 0 },
        ],
    )?;

    Ok(())
}

pub fn update_cached_message(
    conn: &Connection,
    id: MessageId,
    content: &str,
    edited: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE messages SET content = ?2, edited = ?3 WHERE id = ?1",
        params![id.to_string(), content, if edited { 1 } else { 0 }],
    )?;
    Ok(())
}

pub fn delete_cached_message(conn: &Connection, id: MessageId) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM messages WHERE id = ?1",
        params![id.to_string()],
    )?;
    Ok(())
}

pub fn sync_cached_messages(
    conn: &mut Connection,
    channel_id: ChannelId,
    messages: &[MessageView],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    if !messages.is_empty() {
        let min_time = messages
            .iter()
            .map(|m| m.created_at)
            .min()
            .expect("non-empty");
        let max_time = messages
            .iter()
            .map(|m| m.created_at)
            .max()
            .expect("non-empty");

        let min_str = min_time
            .format(&Rfc3339)
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap());
        let max_str = max_time
            .format(&Rfc3339)
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap());

        // Upsert all messages in the batch
        for msg in messages {
            let author_str = msg.author_id.map(|a| a.to_string());
            let reply_str = msg.reply_to_id.map(|r| r.to_string());
            let forwarded_str = msg
                .forwarded_from
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok());
            let created_str = msg
                .created_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap());

            tx.execute(
                "INSERT INTO messages (id, channel_id, author_id, author_name, content, edited, reply_to_id, forwarded_from, created_at, is_pending)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                     content = excluded.content,
                     edited = excluded.edited,
                     reply_to_id = excluded.reply_to_id,
                     forwarded_from = excluded.forwarded_from,
                     is_pending = excluded.is_pending;",
                params![
                    msg.id.to_string(),
                    msg.channel_id.to_string(),
                    author_str,
                    msg.author_name,
                    msg.content,
                    if msg.edited { 1 } else { 0 },
                    reply_str,
                    forwarded_str,
                    created_str,
                    if msg.is_pending { 1 } else { 0 },
                ],
            )?;
        }

        // Reconcile: delete messages in that time window that exist in cache but not on the server
        let server_id_set: HashSet<String> = messages.iter().map(|m| m.id.to_string()).collect();
        let mut stmt = tx.prepare(
            "SELECT id FROM messages WHERE channel_id = ?1 AND created_at >= ?2 AND created_at <= ?3 AND is_pending = 0",
        )?;
        let existing_ids: Vec<String> = stmt
            .query_map(params![channel_id.to_string(), min_str, max_str], |r| {
                r.get(0)
            })?
            .filter_map(Result::ok)
            .collect();
        drop(stmt);

        for old_id in existing_ids {
            if !server_id_set.contains(&old_id) {
                tx.execute("DELETE FROM messages WHERE id = ?1", params![old_id])?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn get_cached_servers(conn: &Connection) -> rusqlite::Result<Vec<ServerSmallPart>> {
    let mut stmt =
        conn.prepare("SELECT id, name, icon_url, is_owner FROM servers ORDER BY created_at ASC")?;
    let rows = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let icon_url: String = row.get(2)?;
        let is_owner_num: i64 = row.get(3)?;

        Ok(ServerSmallPart {
            id: ServerId(Uuid::parse_str(&id_str).unwrap_or_default()),
            name,
            icon_url,
            is_owner: is_owner_num != 0,
        })
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

pub fn sync_cached_servers(
    conn: &mut Connection,
    servers: &[ServerSmallPart],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    for s in servers {
        tx.execute(
            "INSERT INTO servers (id, name, icon_url, is_owner) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 icon_url = excluded.icon_url,
                 is_owner = excluded.is_owner;",
            params![
                s.id.to_string(),
                s.name,
                s.icon_url,
                if s.is_owner { 1 } else { 0 },
            ],
        )?;
    }

    let server_id_set: HashSet<String> = servers.iter().map(|s| s.id.to_string()).collect();
    let mut stmt = tx.prepare("SELECT id FROM servers")?;
    let all_ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .filter_map(Result::ok)
        .collect();
    drop(stmt);

    for old_id in all_ids {
        if !server_id_set.contains(&old_id) {
            tx.execute("DELETE FROM servers WHERE id = ?1", params![old_id])?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn get_cached_channels(
    conn: &Connection,
    server_id: ServerId,
) -> rusqlite::Result<Vec<ChannelView>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, type, position FROM channels WHERE server_id = ?1 ORDER BY position ASC",
    )?;
    let rows = stmt.query_map(params![server_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let type_str: String = row.get(2)?;
        let position: i32 = row.get(3)?;

        let r#type = match type_str.to_uppercase().as_str() {
            "VOICE" => ChannelType::VOICE,
            _ => ChannelType::TEXT,
        };

        Ok(ChannelView {
            id: ChannelId(Uuid::parse_str(&id_str).unwrap_or_default()),
            name,
            r#type,
            position,
        })
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

pub fn sync_cached_channels(
    conn: &mut Connection,
    server_id: ServerId,
    channels: &[ChannelView],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    for ch in channels {
        let type_str = match ch.r#type {
            ChannelType::TEXT => "TEXT",
            ChannelType::VOICE => "VOICE",
        };

        tx.execute(
            "INSERT INTO channels (id, server_id, name, type, position) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 type = excluded.type,
                 position = excluded.position;",
            params![
                ch.id.to_string(),
                server_id.to_string(),
                ch.name,
                type_str,
                ch.position,
            ],
        )?;
    }

    let ch_id_set: HashSet<String> = channels.iter().map(|c| c.id.to_string()).collect();
    let mut stmt = tx.prepare("SELECT id FROM channels WHERE server_id = ?1")?;
    let all_ids: Vec<String> = stmt
        .query_map(params![server_id.to_string()], |r| r.get(0))?
        .filter_map(Result::ok)
        .collect();
    drop(stmt);

    for old_id in all_ids {
        if !ch_id_set.contains(&old_id) {
            tx.execute("DELETE FROM channels WHERE id = ?1", params![old_id])?;
        }
    }

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    #[test]
    fn test_migrations_create_schema() {
        let conn = setup_test_db();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"servers".to_string()));
        assert!(tables.contains(&"channels".to_string()));
        assert!(tables.contains(&"messages".to_string()));
    }

    #[test]
    fn test_messages_crud_and_sync() {
        let mut conn = setup_test_db();
        let channel_id = ChannelId::default();
        let now = time::OffsetDateTime::now_utc();

        let msg1 = MessageView {
            id: MessageId::default(),
            channel_id,
            author_id: Some(UserId::default()),
            author_name: "Alice".into(),
            content: "Hello from Alice".into(),
            edited: false,
            reply_to_id: None,
            forwarded_from: None,
            created_at: now + time::Duration::seconds(2),
            is_pending: false,
        };

        let msg2 = MessageView {
            id: MessageId::default(),
            channel_id,
            author_id: Some(UserId::default()),
            author_name: "Bob".into(),
            content: "Hello from Bob".into(),
            edited: false,
            reply_to_id: None,
            forwarded_from: None,
            created_at: now + time::Duration::seconds(5),
            is_pending: false,
        };

        // Save individual
        save_cached_message(&conn, &msg1).unwrap();
        save_cached_message(&conn, &msg2).unwrap();

        // Query
        let cached = get_cached_messages(&conn, channel_id, 10, None).unwrap();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].id, msg1.id);
        assert_eq!(cached[1].id, msg2.id);

        // Update
        update_cached_message(&conn, msg1.id, "Alice updated", true).unwrap();
        let cached = get_cached_messages(&conn, channel_id, 10, None).unwrap();
        assert_eq!(cached[0].content, "Alice updated");
        assert!(cached[0].edited);

        // Sync batch with deletion: msg1 is absent on server in same time window [now, now + 5s]
        let msg3 = MessageView {
            id: MessageId::default(),
            channel_id,
            author_id: Some(UserId::default()),
            author_name: "Charlie".into(),
            content: "Hello from Charlie".into(),
            edited: false,
            reply_to_id: None,
            forwarded_from: None,
            created_at: now,
            is_pending: false,
        };

        // Server batch contains msg3 and msg2 (msg1 was deleted on server)
        sync_cached_messages(&mut conn, channel_id, &[msg3.clone(), msg2.clone()]).unwrap();

        let cached_after_sync = get_cached_messages(&conn, channel_id, 10, None).unwrap();
        assert_eq!(cached_after_sync.len(), 2);
        assert!(cached_after_sync.iter().any(|m| m.id == msg3.id));
        assert!(cached_after_sync.iter().any(|m| m.id == msg2.id));
        assert!(!cached_after_sync.iter().any(|m| m.id == msg1.id)); // Deleted!

        // Delete individual
        delete_cached_message(&conn, msg2.id).unwrap();
        let cached_after_delete = get_cached_messages(&conn, channel_id, 10, None).unwrap();
        assert_eq!(cached_after_delete.len(), 1);
        assert_eq!(cached_after_delete[0].id, msg3.id);
    }

    #[test]
    fn test_servers_and_channels_caching() {
        let mut conn = setup_test_db();
        let server_id = ServerId::default();

        let servers = vec![ServerSmallPart {
            id: server_id,
            name: "Main Server".into(),
            icon_url: "https://icon.png".into(),
            is_owner: true,
        }];

        sync_cached_servers(&mut conn, &servers).unwrap();
        let cached_servers = get_cached_servers(&conn).unwrap();
        assert_eq!(cached_servers.len(), 1);
        assert_eq!(cached_servers[0].name, "Main Server");
        assert!(cached_servers[0].is_owner);

        let channels = vec![
            ChannelView {
                id: ChannelId::default(),
                name: "general".into(),
                r#type: ChannelType::TEXT,
                position: 0,
            },
            ChannelView {
                id: ChannelId::default(),
                name: "voice-lounge".into(),
                r#type: ChannelType::VOICE,
                position: 1,
            },
        ];

        sync_cached_channels(&mut conn, server_id, &channels).unwrap();
        let cached_channels = get_cached_channels(&conn, server_id).unwrap();
        assert_eq!(cached_channels.len(), 2);
        assert_eq!(cached_channels[0].name, "general");
        assert_eq!(cached_channels[0].r#type, ChannelType::TEXT);
        assert_eq!(cached_channels[1].name, "voice-lounge");
        assert_eq!(cached_channels[1].r#type, ChannelType::VOICE);
    }

    #[test]
    fn test_auth_tokens_caching() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_tokens (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        assert_eq!(get_token(&conn, "access_token").unwrap(), None);

        set_token(&conn, "access_token", "sample_access_jwt").unwrap();
        assert_eq!(
            get_token(&conn, "access_token").unwrap(),
            Some("sample_access_jwt".to_string())
        );

        set_token(&conn, "refresh_token", "sample_refresh_jwt").unwrap();
        assert_eq!(
            get_token(&conn, "refresh_token").unwrap(),
            Some("sample_refresh_jwt".to_string())
        );

        // Update token
        set_token(&conn, "access_token", "updated_jwt").unwrap();
        assert_eq!(
            get_token(&conn, "access_token").unwrap(),
            Some("updated_jwt".to_string())
        );

        // Delete token with empty string
        set_token(&conn, "access_token", "").unwrap();
        assert_eq!(get_token(&conn, "access_token").unwrap(), None);
        assert_eq!(
            get_token(&conn, "refresh_token").unwrap(),
            Some("sample_refresh_jwt".to_string())
        );
    }
}
