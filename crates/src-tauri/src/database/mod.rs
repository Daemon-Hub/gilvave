//! Local SQLite offline cache database for Desktop and Android platforms.

mod messages;
mod servers;
#[cfg(test)]
mod tests;
mod tokens;

pub use messages::{
    delete_cached_message, get_cached_messages, save_cached_message, sync_cached_messages,
    update_cached_message,
};
pub use servers::{
    get_cached_channels, get_cached_servers, sync_cached_channels, sync_cached_servers,
};
pub use tokens::{get_token, set_token};

use include_dir::{include_dir, Dir};
use rusqlite::Connection;
use rusqlite_migration::Migrations;
use std::{fs, sync::LazyLock};
use tauri::{AppHandle, Manager};

/// Convenience alias for errors originating from rusqlite, IO, Tauri, or migrations.
pub type BoxResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

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

    // PRAGMA updates outside transaction
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
