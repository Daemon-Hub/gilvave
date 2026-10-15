use gilvave_core::{
    dto::{
        command::{CommandArgs, CommandResponse, CommandResult},
        user::{
            AuthTokensResponse, LoginRequest, RegisterRequest, SessionView, UpdateTokensRequest,
            UserView,
        },
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

    pub async fn update_tokens() -> Result<(), ErrorInfo> {
        tracing::info!("update_tokens!");
        let refresh_token = match invoke_command(CommandArgs::GetRefreshToken.to_json()).await {
            CommandResult::Ok(CommandResponse::GetRefreshToken(t)) => t,
            _ => String::new(),
        };
        let json = UpdateTokensRequest { refresh_token };
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/sessions/refresh"),
            &json,
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

    // TODO [HTTP]: GET /users/sessions — получить список всех активных сессий пользователя
    pub async fn get_sessions() -> Result<Vec<SessionView>, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/users/sessions"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<Vec<SessionView>>(res).await
    }

    // TODO [HTTP]: DELETE /users/sessions/{session_id} — принудительно завершить сессию на конкретном устройстве
    pub async fn terminate_session(session_id: &str) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "DELETE",
            &format!("{BASE_HTTP_URL}/users/sessions/{session_id}"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }

    // TODO [HTTP]: DELETE /users/sessions/other — завершить все сессии кроме текущей
    pub async fn terminate_all_other_sessions() -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "DELETE",
            &format!("{BASE_HTTP_URL}/users/sessions/other"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }
}
