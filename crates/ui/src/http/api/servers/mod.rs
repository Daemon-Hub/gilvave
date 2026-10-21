pub mod channels;

use gilvave_core::{
    dto::servers::{MemberView, Server, ServerCreateInfo, ServerSmallPart},
    error::ErrorInfo,
    ids::ServerId,
    settings::BASE_HTTP_URL,
};

use crate::http::api::Api;

impl Api {
    pub async fn get_user_servers() -> Result<Vec<ServerSmallPart>, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res =
            Api::request_raw("GET", &format!("{BASE_HTTP_URL}/servers"), (), Some(&token)).await?;
        Api::response_to::<Vec<ServerSmallPart>>(res).await
    }

    pub async fn get_public_servers(page: u32) -> Result<(Vec<Server>, bool), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/servers/public/{page}"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<(Vec<Server>, bool)>(res).await
    }

    pub async fn get_members(server_id: ServerId) -> Result<Vec<MemberView>, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/servers/{server_id}/members"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<Vec<MemberView>>(res).await
    }

    pub async fn create_server(server_info: ServerCreateInfo) -> Result<Server, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/servers"),
            &server_info,
            Some(&token),
        )
        .await?;
        Api::response_to::<Server>(res).await
    }

    pub async fn get_server_by_id(server_id: ServerId) -> Result<Server, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/servers/{server_id}"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<Server>(res).await
    }

    pub async fn join_public_server(server_id: ServerId) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/servers/{server_id}/join_public"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }
}
