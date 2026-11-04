use gilvave_core::{
    dto::files::{File, FileUploadResponse},
    error::{ErrorCode, ErrorInfo},
    ids::FileId,
    settings::BASE_HTTP_URL,
};

use super::Api;

impl Api {
    /// Загрузка файла
    pub async fn upload_file(file: &web_sys::File) -> Result<FileUploadResponse, ErrorInfo> {
        let file_size = file.size() as u64;
        if file_size > gilvave_core::files::MAX_FILE_SIZE_BYTES {
            return Err(ErrorInfo::from_code(
                ErrorCode::BadRequest,
                format!(
                    "Размер файла ({:.1} МБ) превышает лимит {} МБ",
                    file_size as f64 / (1024.0 * 1024.0),
                    gilvave_core::files::MAX_FILE_SIZE_MB
                ),
            ));
        }

        let form_data = web_sys::FormData::new().map_err(|e| {
            ErrorInfo::from_code(
                ErrorCode::HttpRequestFailed,
                format!("Failed to create FormData: {e:?}"),
            )
        })?;

        form_data
            .append_with_blob_and_filename("file", file, &file.name())
            .map_err(|e| {
                ErrorInfo::from_code(
                    ErrorCode::HttpRequestFailed,
                    format!("Failed to append file to FormData: {e:?}"),
                )
            })?;

        let token = Self::fetch_access_token().await;
        let url = format!("{BASE_HTTP_URL}/files");
        let response = Self::request_multipart("POST", &url, &form_data, Some(&token)).await?;
        let mut upload_res = Self::response_to::<FileUploadResponse>(response).await?;
        if upload_res.url.starts_with('/') {
            upload_res.url = format!("{BASE_HTTP_URL}{}", upload_res.url);
        }
        Ok(upload_res)
    }

    /// Получение информации о файле
    pub async fn get_file_info(file_id: FileId) -> Result<File, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let url = format!("{BASE_HTTP_URL}/files/{file_id}/info");
        let response = Self::request_raw("GET", &url, (), Some(&token)).await?;
        Self::response_to::<File>(response).await
    }

    /// Удаление файла
    pub async fn delete_file(file_id: FileId) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let url = format!("{BASE_HTTP_URL}/files/{file_id}");
        let response = Self::request_raw("DELETE", &url, (), Some(&token)).await?;
        Self::response_to_empty(response).await
    }

    /// Ссылка на скачивание файла
    pub fn get_file_download_url(file_id: FileId) -> String {
        format!("{BASE_HTTP_URL}/files/{file_id}")
    }
}
