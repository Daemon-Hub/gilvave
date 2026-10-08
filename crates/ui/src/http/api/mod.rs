use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;

use gilvave_core::{
    dto::{
        command::{CommandArgs, CommandResponse, CommandResult},
        traits::RequestBody,
    },
    error::{ErrorInfo, ErrorMessage},
};

use crate::utils::invoke_command;

pub mod channel;
pub mod server;
pub mod user;

pub struct Api;

impl Api {
    pub(super) async fn fetch_access_token() -> String {
        match invoke_command(CommandArgs::GetAccessToken.to_json()).await {
            CommandResult::Ok(CommandResponse::GetAccessToken(t)) => t,
            _ => String::new(),
        }
    }

    async fn send_once(
        method: &str,
        url: &str,
        body: &Option<String>,
        auth_token: Option<&str>,
    ) -> Result<web_sys::Response, ErrorInfo> {
        let window =
            web_sys::window().ok_or_else(|| ErrorInfo(1, "No window available".to_string()))?;
        let opts = web_sys::RequestInit::new();
        opts.set_method(method);
        opts.set_credentials(web_sys::RequestCredentials::Include);

        let headers = web_sys::Headers::new().map_err(|e| ErrorInfo(1, format!("{e:?}")))?;
        headers
            .set("Accept", "application/json")
            .map_err(|e| ErrorInfo(1, format!("{e:?}")))?;
        if body.is_some() {
            headers
                .set("Content-Type", "application/json")
                .map_err(|e| ErrorInfo(1, format!("{e:?}")))?;
        }
        if let Some(token) = auth_token
            && !token.is_empty()
        {
            headers
                .set("Authorization", &format!("Bearer {token}"))
                .map_err(|e| ErrorInfo(1, format!("{e:?}")))?;
        }
        opts.set_headers(&headers);

        if let Some(b) = body {
            opts.set_body(&wasm_bindgen::JsValue::from_str(b));
        }

        let request = web_sys::Request::new_with_str_and_init(url, &opts)
            .map_err(|e| ErrorInfo(1, format!("{e:?}")))?;

        let promise = window.fetch_with_request(&request);
        let resp_value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| ErrorInfo(1, format!("Fetch error: {e:?}")))?;

        let response: web_sys::Response = resp_value
            .dyn_into()
            .map_err(|e| ErrorInfo(1, format!("Invalid response: {e:?}")))?;

        Ok(response)
    }

    pub async fn request_raw(
        method: &str,
        url: &str,
        body: impl RequestBody,
        auth_token: Option<&str>,
    ) -> Result<web_sys::Response, ErrorInfo> {
        let body_json = body.to_json()?;
        let response = Self::send_once(method, url, &body_json, auth_token).await?;
        if response.status() == 401
            && auth_token.is_some()
            && Box::pin(Self::update_tokens()).await.is_ok()
        {
            let new_token = Self::fetch_access_token().await;
            return Self::send_once(method, url, &body_json, Some(&new_token)).await;
        }
        Ok(response)
    }

    pub async fn response_to_bytes(response: web_sys::Response) -> Result<Vec<u8>, ErrorInfo> {
        let promise = response
            .array_buffer()
            .map_err(|e| ErrorInfo(1, format!("Response.array_buffer error: {e:?}")))?;
        let js_value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| ErrorInfo(1, format!("Response.array_buffer promise error: {e:?}")))?;
        let uint8_array = js_sys::Uint8Array::new(&js_value);
        Ok(uint8_array.to_vec())
    }

    pub async fn response_to_text(response: web_sys::Response) -> Result<String, ErrorInfo> {
        let promise = response
            .text()
            .map_err(|e| ErrorInfo(1, format!("Response.text error: {e:?}")))?;
        let js_value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| ErrorInfo(1, format!("Response.text promise error: {e:?}")))?;
        js_value
            .as_string()
            .ok_or_else(|| ErrorInfo(1, "Response text is not a valid string".to_string()))
    }

    pub async fn response_to<T: DeserializeOwned>(
        response: web_sys::Response,
    ) -> Result<T, ErrorInfo> {
        let status = response.status();
        if (200..=299).contains(&status) {
            let text = Self::response_to_text(response).await?;
            serde_json::from_str::<T>(&text)
                .map_err(|e| ErrorInfo(1, format!("JSON decode error: {e}")))
        } else {
            Err(Self::response_to_error(response).await)
        }
    }

    pub async fn response_to_empty(response: web_sys::Response) -> Result<(), ErrorInfo> {
        let status = response.status();
        if (200..=299).contains(&status) {
            Ok(())
        } else {
            Err(Self::response_to_error(response).await)
        }
    }

    pub(crate) async fn response_to_error(response: web_sys::Response) -> ErrorInfo {
        let status = response.status();
        let text = Self::response_to_text(response).await.unwrap_or_default();
        if let Ok(err_msg) = serde_json::from_str::<ErrorMessage>(&text) {
            ErrorInfo(status, err_msg.error)
        } else {
            ErrorInfo(status, text)
        }
    }
}
