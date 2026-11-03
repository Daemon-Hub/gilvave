//! Server and channel caching, retrieval, and synchronization with SQLite.

use std::collections::HashSet;
use gilvave_core::{
    dto::servers::{ChannelType, ChannelView, ServerSmallPart},
    ids::{ChannelId, ServerId},
    uuid::Uuid,
};
use rusqlite::{Connection, params};

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
