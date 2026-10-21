use gilvave_core::{
    dto::{
        command::CommandArgs,
        users::{AuthTokensResponse, LoginRequest, RegisterRequest},
    },
    error::ErrorInfo,
    settings::BASE_HTTP_URL,
};

use crate::{http::api::Api, utils::invoke_command};

impl Api {
    pub async fn register(register_request: RegisterRequest) -> Result<(), ErrorInfo> {
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/auth/register"),
            &register_request,
            None,
        )
        .await?;
        Api::response_to_empty(res).await
    }

    pub async fn login(request: LoginRequest) -> Result<(), ErrorInfo> {
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/auth/login"),
            &request,
            None,
        )
        .await?;
        let status = res.status();
        if !(200..=299).contains(&status) {
            return Err(Api::response_to_error(res).await);
        }
        let text = Api::response_to_text(res).await?;
        if !text.trim().is_empty()
            && let Ok(r) = serde_json::from_str::<AuthTokensResponse>(&text)
        {
            invoke_command(
                CommandArgs::SetAccessToken {
                    token: r.access_token,
                }
                .to_json(),
            )
            .await;
            invoke_command(
                CommandArgs::SetRefreshToken {
                    token: r.refresh_token,
                }
                .to_json(),
            )
            .await;
        }
        Ok(())
    }

    pub async fn logout() -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/auth/logout"),
            (),
            Some(&token),
        )
        .await?;
        let _ = invoke_command(
            CommandArgs::SetAccessToken {
                token: String::new(),
            }
            .to_json(),
        )
        .await;
        let _ = invoke_command(
            CommandArgs::SetRefreshToken {
                token: String::new(),
            }
            .to_json(),
        )
        .await;
        Api::response_to_empty(res).await
    }
}
