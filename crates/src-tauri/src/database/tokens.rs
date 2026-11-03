//! Auth token persistence in the local SQLite database.

use rusqlite::{Connection, params};
use super::BoxResult;

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
