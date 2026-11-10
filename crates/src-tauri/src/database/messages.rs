//! Message caching, retrieval, updates, and synchronization with SQLite.

use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId, UserId},
    uuid::Uuid,
};
use rusqlite::{Connection, params};
use time::format_description::well_known::Rfc3339;

pub(crate) fn row_to_message_view(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageView> {
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
        is_failed: false,
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
        #[cfg(not(target_arch = "wasm32"))]
        let (min_time, max_time) = {
            let min = messages.par_iter().map(|m| m.created_at).min().expect("non-empty");
            let max = messages.par_iter().map(|m| m.created_at).max().expect("non-empty");
            (min, max)
        };
        #[cfg(target_arch = "wasm32")]
        let (min_time, max_time) = {
            let min = messages.iter().map(|m| m.created_at).min().expect("non-empty");
            let max = messages.iter().map(|m| m.created_at).max().expect("non-empty");
            (min, max)
        };

        let min_str = min_time
            .format(&Rfc3339)
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap());
        let max_str = max_time
            .format(&Rfc3339)
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        let prepared_records: Vec<_> = messages
            .par_iter()
            .map(|msg| {
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
                (
                    msg.id.to_string(),
                    msg.channel_id.to_string(),
                    author_str,
                    &msg.author_name,
                    &msg.content,
                    if msg.edited { 1i64 } else { 0i64 },
                    reply_str,
                    forwarded_str,
                    created_str,
                    if msg.is_pending { 1i64 } else { 0i64 },
                )
            })
            .collect();

        #[cfg(target_arch = "wasm32")]
        let prepared_records: Vec<_> = messages
            .iter()
            .map(|msg| {
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
                (
                    msg.id.to_string(),
                    msg.channel_id.to_string(),
                    author_str,
                    &msg.author_name,
                    &msg.content,
                    if msg.edited { 1i64 } else { 0i64 },
                    reply_str,
                    forwarded_str,
                    created_str,
                    if msg.is_pending { 1i64 } else { 0i64 },
                )
            })
            .collect();

        // Upsert all messages in the batch using pre-processed parameters
        for rec in prepared_records {
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
                    rec.0,
                    rec.1,
                    rec.2,
                    rec.3,
                    rec.4,
                    rec.5,
                    rec.6,
                    rec.7,
                    rec.8,
                    rec.9,
                ],
            )?;
        }

        // Reconcile: delete messages in that time window that exist in cache but not on the server
        #[cfg(not(target_arch = "wasm32"))]
        let server_id_set: HashSet<MessageId> = messages.par_iter().map(|m| m.id).collect();
        #[cfg(target_arch = "wasm32")]
        let server_id_set: HashSet<MessageId> = messages.iter().map(|m| m.id).collect();

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

        for old_id_str in existing_ids {
            let old_id = MessageId(Uuid::parse_str(&old_id_str).unwrap_or_default());
            if !server_id_set.contains(&old_id) {
                tx.execute("DELETE FROM messages WHERE id = ?1", params![old_id_str])?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}
