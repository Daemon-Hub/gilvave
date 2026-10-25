use gilvave_core::{
    dto::users::{UserView, profile::CreateThemeInfo, profile::Theme},
    error::ErrorInfo,
    settings::BASE_HTTP_URL,
};

use crate::http::api::Api;

impl Api {
    pub async fn get_profile() -> Result<UserView, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/users/profile/me"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<UserView>(res).await
    }

    pub async fn get_themes_catalog(page: i64) -> Result<(Vec<Theme>, bool), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/users/profile/themes/{page}"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<(Vec<Theme>, bool)>(res).await
    }

    pub async fn create_theme(info: &CreateThemeInfo) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/profile/themes"),
            info,
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }
}
