use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{FileId, MessageId, UserId};

/// Категория файла
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Image,
    Video,
    File,
}

impl FileType {
    /// Проверка возможности предпросмотра
    pub fn is_previewable(&self) -> bool {
        matches!(self, FileType::Image | FileType::Video)
    }

    /// Имя S3-бакета
    pub fn s3_bucket(&self) -> &'static str {
        match self {
            FileType::Image => "images",
            FileType::Video => "videos",
            FileType::File => "files",
        }
    }
}

/// Метаданные файла
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct File {
    pub id: FileId,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub message_id: Option<MessageId>,
    pub size_kb: i32,
    pub mime_type: String,
    #[serde(default)]
    pub uploader_id: Option<UserId>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Ответ при загрузке файла
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileUploadResponse {
    pub id: FileId,
    pub name: String,
    pub url: String,
    pub size_kb: i32,
    pub mime_type: String,
    pub file_type: FileType,
    pub is_previewable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_properties() {
        assert!(FileType::Image.is_previewable());
        assert!(FileType::Video.is_previewable());
        assert!(!FileType::File.is_previewable());

        assert_eq!(FileType::Image.s3_bucket(), "images");
        assert_eq!(FileType::Video.s3_bucket(), "videos");
        assert_eq!(FileType::File.s3_bucket(), "files");
    }
}
