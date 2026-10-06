use include_dir::{Dir, include_dir};
use rusqlite::Connection;
use rusqlite_migration::Migrations;
use std::{fs, sync::LazyLock};
use tauri::{AppHandle, Manager};

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

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_create_schema() {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();

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

        conn.execute(
            "INSERT INTO servers (id, name, owner_id, icon_url, is_public) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("srv-1", "Test Server", "usr-1", "https://icon.png", 1),
        ).unwrap();

        let srv_name: String = conn
            .query_row("SELECT name FROM servers WHERE id = 'srv-1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(srv_name, "Test Server");
    }
}


