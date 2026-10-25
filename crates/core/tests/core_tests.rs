use gilvave_core::{
    dto::{
        command::{CommandArgs, CommandResponse, CommandResult},
        messages::MessageView,
        servers::Server,
        ws::{ServerRecieve, ServerSend},
    },
    error::{ErrorInfo, ErrorMessage},
    ids::{ChannelId, MessageId, ServerId, UserId},
    settings::DeviceInfo,
};
use time::OffsetDateTime;

#[test]
fn test_id_generation_and_versions() {
    let user_id = UserId::default();
    assert_eq!(user_id.0.get_version_num(), 7);

    let message_id = MessageId::default();
    assert_eq!(message_id.0.get_version_num(), 7);

    let server_id = ServerId::default();
    assert_eq!(server_id.0.get_version_num(), 4);

    let channel_id = ChannelId::default();
    assert_eq!(channel_id.0.get_version_num(), 4);
}

#[test]
fn test_id_serde_roundtrip() {
    let u_id = UserId::default();
    let json = serde_json::to_string(&u_id).unwrap();
    let parsed: UserId = serde_json::from_str(&json).unwrap();
    assert_eq!(u_id, parsed);

    let s_id = ServerId::default();
    let json = serde_json::to_string(&s_id).unwrap();
    let parsed: ServerId = serde_json::from_str(&json).unwrap();
    assert_eq!(s_id, parsed);

    // TryFrom string
    let parsed_from_str = UserId::try_from(u_id.0.to_string()).unwrap();
    assert_eq!(u_id, parsed_from_str);
}

#[test]
fn test_error_info_and_message() {
    let err = ErrorInfo::default("custom service error".to_string());
    assert_eq!(err.0, 503);
    assert_eq!(err.1, "custom service error");

    let err_msg = ErrorMessage {
        error: "forbidden action".to_string(),
    };
    let json = serde_json::to_string(&err_msg).unwrap();
    let parsed: ErrorMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.error, "forbidden action");
}

#[test]
fn test_device_info_json() {
    let info = DeviceInfo {
        platform: Some("Windows".into()),
        os_version: Some("11".into()),
        client: "desktop".into(),
        client_version: DeviceInfo::client_version(),
        device_name: Some("test-pc".into()),
        browser: None,
        browser_version: None,
    };
    let json = info.to_json();
    assert_eq!(json["client"], "desktop");
    assert_eq!(json["platform"], "Windows");
    assert_eq!(json["client_version"], DeviceInfo::client_version());
}

#[test]
fn test_command_args_to_json_and_deserialize() {
    let set_cmd = CommandArgs::SetAccessToken {
        token: "test_token_123".to_string(),
    };
    let json_val = set_cmd.to_json();
    assert!(json_val.get("command").is_some());

    let parsed: CommandArgs = serde_json::from_value(json_val["command"].clone()).unwrap();
    match parsed {
        CommandArgs::SetAccessToken { token } => {
            assert_eq!(token, "test_token_123");
        }
        _ => panic!("Expected SetAccessToken variant"),
    }
}

#[test]
fn test_all_command_args_variants_to_json() {
    let commands = vec![
        CommandArgs::GetAccessToken,
        CommandArgs::GetRefreshToken,
        CommandArgs::SetAccessToken {
            token: "acc".to_string(),
        },
        CommandArgs::SetRefreshToken {
            token: "ref".to_string(),
        },
        CommandArgs::GetDeviceInfo,
        CommandArgs::WindowMinimize,
        CommandArgs::WindowToggleMaximize,
        CommandArgs::WindowClose,
        CommandArgs::WindowStartDragging,
        CommandArgs::OpenDevTools,
    ];

    for cmd in commands {
        let json_val = cmd.to_json();
        let cmd_inner = json_val.get("command").unwrap();
        let _parsed: CommandArgs = serde_json::from_value(cmd_inner.clone()).unwrap();
    }
}

#[test]
fn test_command_result_is_ok() {
    let ok_res = CommandResult::Ok(CommandResponse::SetAccessToken);
    assert!(ok_res.is_ok());

    let err_res = CommandResult::Error(ErrorInfo(400, "bad".to_string()));
    assert!(!err_res.is_ok());
}

#[test]
fn test_ws_server_receive_variants() {
    let hello_json = r#"{"op":"HELLO"}"#;
    let msg: ServerRecieve = serde_json::from_str(hello_json).unwrap();
    assert!(matches!(msg, ServerRecieve::Hello));

    let join_json = r#"{"op":"JOIN_SUCCESS"}"#;
    let msg: ServerRecieve = serde_json::from_str(join_json).unwrap();
    assert!(matches!(msg, ServerRecieve::JoinSuccess));

    let m_id = MessageId::default();
    let u_id = UserId::default();
    let c_id = ChannelId::default();
    let now = OffsetDateTime::now_utc();

    let message_view = MessageView {
        id: m_id,
        channel_id: c_id,
        author_id: Some(u_id),
        author_name: "author".to_string(),
        content: "test message".to_string(),
        created_at: now,
    };

    let msg_new = ServerRecieve::MessageNew(message_view.clone());
    let json = serde_json::to_string(&msg_new).unwrap();
    assert!(json.contains("MESSAGE_NEW"));

    let parsed: ServerRecieve = serde_json::from_str(&json).unwrap();
    match parsed {
        ServerRecieve::MessageNew(m) => {
            assert_eq!(m.id, m_id);
            assert_eq!(m.content, "test message");
        }
        _ => panic!("Expected MessageNew"),
    }
}

#[test]
fn test_ws_server_send_variants() {
    let c_id = ChannelId::default();
    let now = OffsetDateTime::now_utc();

    let send_join = ServerSend::JoinChannel { channel_id: c_id };
    let json = serde_json::to_string(&send_join).unwrap();
    assert!(json.contains("JOIN_CHANNEL"));
    let parsed: ServerSend = serde_json::from_str(&json).unwrap();
    match parsed {
        ServerSend::JoinChannel { channel_id } => assert_eq!(channel_id, c_id),
        _ => panic!("Expected JoinChannel"),
    }

    let send_msg = ServerSend::MessageCreate {
        channel_id: c_id,
        content: "hey".to_string(),
    };
    let json = serde_json::to_string(&send_msg).unwrap();
    assert!(json.contains("MESSAGE_CREATE"));

    let send_hist = ServerSend::ChannelHistoryBefore {
        channel_id: c_id,
        timestamp: now,
    };
    let json = serde_json::to_string(&send_hist).unwrap();
    assert!(json.contains("CHANNEL_HISTORY_BEFORE"));
}

#[test]
fn test_server_full_struct() {
    let s_id = ServerId::default();
    let owner_id = UserId::default();
    let now = OffsetDateTime::now_utc();
    let server = Server {
        id: s_id,
        owner_id,
        name: "Community Server".to_string(),
        description: "Community description".to_string(),
        icon_url: "https://example.com/icon.png".to_string(),
        cover: "https://example.com/cover.png".to_string(),
        is_public: true,
        members_count: 42,
        created_at: now,
    };
    let json = serde_json::to_string(&server).unwrap();
    let parsed: Server = serde_json::from_str(&json).unwrap();
    assert_eq!(server.id, parsed.id);
    assert_eq!(server.name, parsed.name);
    assert_eq!(server.owner_id, parsed.owner_id);
    assert_eq!(server.icon_url, parsed.icon_url);
    assert_eq!(server.cover, parsed.cover);
    assert_eq!(server.is_public, parsed.is_public);
    assert_eq!(server.members_count, 42);
}
