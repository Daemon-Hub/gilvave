use gilvave_core::{
    dto::channel::{ChannelCreateInfo, ChannelView},
    error::ErrorInfo,
    ids::ServerId,
    settings::BASE_HTTP_URL,
};

use crate::http::api::Api;

impl Api {
    pub async fn get_server_channels(server_id: ServerId) -> Result<Vec<ChannelView>, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/servers/{server_id}/channels"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<Vec<ChannelView>>(res).await
    }

    pub async fn create_channel(
        server_id: ServerId,
        channel_info: ChannelCreateInfo,
    ) -> Result<ChannelView, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/servers/{server_id}/channels"),
            &channel_info,
            Some(&token),
        )
        .await?;
        Api::response_to::<ChannelView>(res).await
    }

    // ─── VOICE CALL ENDPOINTS ────────────────────────────────────────────────

    // TODO [HTTP]: POST /channels/{channel_id}/voice/join
    // Запрос токена подключения к WebRTC/SFU медиа-серверу (LiveKit room token, router RTP capabilities, ICE/STUN/TURN серверы)
    // pub async fn join_voice_channel(channel_id: ChannelId) -> Result<VoiceJoinResponse, ErrorInfo>

    // TODO [HTTP]: POST /channels/{channel_id}/voice/leave
    // Уведомление бэкенда о завершении голосовой сессии участником и закрытии SFU-пира
    // pub async fn leave_voice_channel(channel_id: ChannelId) -> Result<(), ErrorInfo>

    // TODO [HTTP]: GET /channels/{channel_id}/voice/participants
    // Получение текущего списка активных участников в голосовой комнате (никнеймы, аватары, Mute/Deafen)
    // pub async fn get_voice_participants(channel_id: ChannelId) -> Result<Vec<VoiceParticipantView>, ErrorInfo>
}
