//! Unit tests for database migrations, message caching, server/channel caching, and auth tokens.

#[cfg(test)]
mod tests {
    use super::super::*;
    use gilvave_core::{
        dto::{
            messages::MessageView,
            servers::{ChannelType, ChannelView, ServerSmallPart},
        },
        ids::{ChannelId, MessageId, ServerId, UserId},
    };
    use rusqlite::Connection;

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
