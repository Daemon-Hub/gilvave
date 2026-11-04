use crate::{dto::files::FileType, ids::FileId};

/// Максимальный размер одного прикрепляемого файла: 100 МБ (104 857 600 байт)
pub const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_FILE_SIZE_MB: u64 = 100;

/// Проверка допустимости размера файла
#[inline]
pub fn is_file_size_valid(bytes_len: u64) -> bool {
    bytes_len > 0 && bytes_len <= MAX_FILE_SIZE_BYTES
}

/// Определение типа и MIME файла
pub fn detect_file_type(name: &str, bytes: &[u8]) -> (FileType, &'static str) {
    let lower_name = name.to_ascii_lowercase();
    let ext = lower_name.rsplit('.').next().unwrap_or("");

    if bytes.len() >= 4 {
        if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return (FileType::Image, "image/png");
        }
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return (FileType::Image, "image/jpeg");
        }
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            return (FileType::Image, "image/gif");
        }
        if bytes.starts_with(b"%PDF") {
            return (FileType::File, "application/pdf");
        }
        if bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            return (FileType::File, "application/zip");
        }
        if bytes.starts_with(b"Rar!") {
            return (FileType::File, "application/x-rar-compressed");
        }
        if bytes.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
            return (FileType::File, "application/x-7z-compressed");
        }
        if bytes.starts_with(&[0x1F, 0x8B]) {
            return (FileType::File, "application/gzip");
        }
        if bytes.starts_with(b"RIFF") && bytes.len() >= 12 {
            if &bytes[8..12] == b"WEBP" {
                return (FileType::Image, "image/webp");
            }
            if &bytes[8..12] == b"AVI " {
                return (FileType::Video, "video/x-msvideo");
            }
            if &bytes[8..12] == b"WAVE" {
                return (FileType::File, "audio/wav");
            }
        }
        if bytes.starts_with(b"OggS") {
            return (FileType::Video, "video/ogg");
        }
        if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
            return (FileType::Video, "video/mp4");
        }
        if bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            return (FileType::Video, "video/webm");
        }
    }

    match ext {
        "png" => (FileType::Image, "image/png"),
        "jpg" | "jpeg" | "jpe" => (FileType::Image, "image/jpeg"),
        "webp" => (FileType::Image, "image/webp"),
        "gif" => (FileType::Image, "image/gif"),
        "svg" => (FileType::Image, "image/svg+xml"),
        "bmp" => (FileType::Image, "image/bmp"),
        "ico" => (FileType::Image, "image/x-icon"),
        "avif" => (FileType::Image, "image/avif"),
        "tiff" | "tif" => (FileType::Image, "image/tiff"),
        "mp4" | "m4v" => (FileType::Video, "video/mp4"),
        "webm" => (FileType::Video, "video/webm"),
        "mov" | "qt" => (FileType::Video, "video/quicktime"),
        "mkv" => (FileType::Video, "video/x-matroska"),
        "avi" => (FileType::Video, "video/x-msvideo"),
        "ogv" => (FileType::Video, "video/ogg"),
        "3gp" => (FileType::Video, "video/3gpp"),
        "flv" => (FileType::Video, "video/x-flv"),
        "pdf" => (FileType::File, "application/pdf"),
        "zip" => (FileType::File, "application/zip"),
        "rar" => (FileType::File, "application/x-rar-compressed"),
        "7z" => (FileType::File, "application/x-7z-compressed"),
        "tar" => (FileType::File, "application/x-tar"),
        "gz" | "tgz" => (FileType::File, "application/gzip"),
        "txt" | "log" | "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "md" => {
            (FileType::File, "text/plain; charset=utf-8")
        }
        "json" => (FileType::File, "application/json"),
        "csv" => (FileType::File, "text/csv"),
        "doc" => (FileType::File, "application/msword"),
        "docx" => (
            FileType::File,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
        "xls" => (FileType::File, "application/vnd.ms-excel"),
        "xlsx" => (
            FileType::File,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        "ppt" => (FileType::File, "application/vnd.ms-powerpoint"),
        "pptx" => (
            FileType::File,
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ),
        "mp3" => (FileType::File, "audio/mpeg"),
        "wav" => (FileType::File, "audio/wav"),
        "ogg" | "oga" => (FileType::File, "audio/ogg"),
        "flac" => (FileType::File, "audio/flac"),
        "apk" => (FileType::File, "application/vnd.android.package-archive"),
        "exe" => (FileType::File, "application/vnd.microsoft.portable-executable"),
        _ => (FileType::File, "application/octet-stream"),
    }
}

/// Получение расширения файла
pub fn get_file_extension(name: &str, mime_type: &str) -> String {
    if let Some(ext) = name.rsplit('.').next()
        && !ext.is_empty()
        && ext.len() <= 8
        && ext != name
    {
        return ext.to_ascii_lowercase();
    }

    match mime_type {
        "image/png" => "png".to_string(),
        "image/jpeg" => "jpg".to_string(),
        "image/webp" => "webp".to_string(),
        "image/gif" => "gif".to_string(),
        "image/svg+xml" => "svg".to_string(),
        "video/mp4" => "mp4".to_string(),
        "video/webm" => "webm".to_string(),
        "application/pdf" => "pdf".to_string(),
        "application/zip" => "zip".to_string(),
        _ => "bin".to_string(),
    }
}

/// Парсинг бакета и ключа S3
pub fn parse_s3_url(url: &str) -> Option<(&str, &str)> {
    let path = if let Some(idx) = url.find("://") {
        let after_scheme = &url[idx + 3..];
        let slash_idx = after_scheme.find('/')?;
        &after_scheme[slash_idx + 1..]
    } else {
        url.trim_start_matches('/')
    };
    path.split_once('/')
}

/// Извлечение ID файлов из текста
pub fn extract_file_ids_from_text(text: &str) -> Vec<FileId> {
    let mut ids = Vec::new();

    let mut remaining = text;
    while let Some(pos) = remaining.find("file:") {
        let after = &remaining[pos + 5..];
        if after.len() >= 36 {
            let candidate = &after[..36];
            if let Ok(file_id) = candidate.parse::<FileId>()
                && !ids.contains(&file_id)
            {
                ids.push(file_id);
            }
        }
        remaining = after;
    }

    for word in text.split(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | '[' | ']' | '<' | '>')) {
        if let Some(filename) = word.rsplit('/').next() {
            let stem = filename.split('.').next().unwrap_or(filename);
            if stem.len() == 36
                && let Ok(file_id) = stem.parse::<FileId>()
                && !ids.contains(&file_id)
            {
                ids.push(file_id);
            }
        }
    }

    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type_images() {
        assert_eq!(detect_file_type("photo.png", &[]), (FileType::Image, "image/png"));
        assert_eq!(detect_file_type("avatar.JPG", &[]), (FileType::Image, "image/jpeg"));
        assert_eq!(detect_file_type("image.webp", &[]), (FileType::Image, "image/webp"));
        assert_eq!(detect_file_type("icon.svg", &[]), (FileType::Image, "image/svg+xml"));
    }

    #[test]
    fn test_detect_file_type_videos() {
        assert_eq!(detect_file_type("movie.mp4", &[]), (FileType::Video, "video/mp4"));
        assert_eq!(detect_file_type("clip.webm", &[]), (FileType::Video, "video/webm"));
        assert_eq!(detect_file_type("record.mkv", &[]), (FileType::Video, "video/x-matroska"));
    }

    #[test]
    fn test_detect_file_type_generic_files() {
        assert_eq!(detect_file_type("doc.pdf", &[]), (FileType::File, "application/pdf"));
        assert_eq!(detect_file_type("archive.zip", &[]), (FileType::File, "application/zip"));
        assert_eq!(detect_file_type("table.xlsx", &[]), (FileType::File, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"));
        assert_eq!(detect_file_type("notes.txt", &[]), (FileType::File, "text/plain; charset=utf-8"));
        assert_eq!(detect_file_type("unknown.xyz123", &[]), (FileType::File, "application/octet-stream"));
    }

    #[test]
    fn test_detect_file_type_magic_bytes() {
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_file_type("unnamed_file", &png_bytes), (FileType::Image, "image/png"));

        let jpeg_bytes = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_file_type("blob", &jpeg_bytes), (FileType::Image, "image/jpeg"));

        let pdf_bytes = b"%PDF-1.4";
        assert_eq!(detect_file_type("file_without_ext", pdf_bytes), (FileType::File, "application/pdf"));
    }

    #[test]
    fn test_parse_s3_url() {
        assert_eq!(
            parse_s3_url("http://127.0.0.1:8333/images/user1/test.png"),
            Some(("images", "user1/test.png"))
        );
        assert_eq!(
            parse_s3_url("https://s3.example.com/files/doc/123.pdf"),
            Some(("files", "doc/123.pdf"))
        );
        assert_eq!(
            parse_s3_url("/videos/clip.mp4"),
            Some(("videos", "clip.mp4"))
        );
        assert_eq!(parse_s3_url("invalid_without_slash"), None);
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("archive.tar.gz", "application/gzip"), "gz");
        assert_eq!(get_file_extension("photo.JPEG", "image/jpeg"), "jpeg");
        assert_eq!(get_file_extension("blob", "image/png"), "png");
        assert_eq!(get_file_extension("video", "video/mp4"), "mp4");
        assert_eq!(get_file_extension("doc", "application/pdf"), "pdf");
    }

    #[test]
    fn test_extract_file_ids_from_text() {
        let id1 = FileId::default();
        let id2 = FileId::default();
        let text = format!("Check this [doc](file:{id1}) and image ![img](http://localhost/images/user/{id2}.png)");
        let ids = extract_file_ids_from_text(&text);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }
}
