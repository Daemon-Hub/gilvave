use gilvave_core::{
    dto::{
        command::{CommandArgs, CommandResponse, CommandResult},
        messages::MessageView,
        servers::{ChannelView, ServerSmallPart},
    },
    ids::{ChannelId, MessageId, ServerId},
};

use crate::utils::invoke_command;

pub struct CacheService;

impl CacheService {
    pub async fn get_cached_messages(
        channel_id: ChannelId,
        limit: usize,
        before: Option<MessageId>,
    ) -> Vec<MessageView> {
        let cmd = CommandArgs::GetCachedMessages {
            channel_id,
            limit,
            before,
        };
        match invoke_command(cmd.to_json()).await {
            CommandResult::Ok(CommandResponse::GetCachedMessages(messages)) => messages,
            _ => Vec::new(),
        }
    }

    pub async fn sync_cached_messages(channel_id: ChannelId, messages: Vec<MessageView>) {
        let cmd = CommandArgs::SyncCachedMessages {
            channel_id,
            messages,
        };
        let _ = invoke_command(cmd.to_json()).await;
    }

    pub async fn save_cached_message(message: MessageView) {
        let cmd = CommandArgs::SaveCachedMessage { message };
        let _ = invoke_command(cmd.to_json()).await;
    }

    pub async fn update_cached_message(id: MessageId, content: String, edited: bool) {
        let cmd = CommandArgs::UpdateCachedMessage {
            id,
            content,
            edited,
        };
        let _ = invoke_command(cmd.to_json()).await;
    }

    pub async fn delete_cached_message(id: MessageId) {
        let cmd = CommandArgs::DeleteCachedMessage { id };
        let _ = invoke_command(cmd.to_json()).await;
    }

    pub async fn get_cached_servers() -> Vec<ServerSmallPart> {
        let cmd = CommandArgs::GetCachedServers;
        match invoke_command(cmd.to_json()).await {
            CommandResult::Ok(CommandResponse::GetCachedServers(servers)) => servers,
            _ => Vec::new(),
        }
    }

    pub async fn sync_cached_servers(servers: Vec<ServerSmallPart>) {
        let cmd = CommandArgs::SyncCachedServers { servers };
        let _ = invoke_command(cmd.to_json()).await;
    }

    pub async fn get_cached_channels(server_id: ServerId) -> Vec<ChannelView> {
        let cmd = CommandArgs::GetCachedChannels { server_id };
        match invoke_command(cmd.to_json()).await {
            CommandResult::Ok(CommandResponse::GetCachedChannels(channels)) => channels,
            _ => Vec::new(),
        }
    }

    pub async fn sync_cached_channels(server_id: ServerId, channels: Vec<ChannelView>) {
        let cmd = CommandArgs::SyncCachedChannels {
            server_id,
            channels,
        };
        let _ = invoke_command(cmd.to_json()).await;
    }
}
