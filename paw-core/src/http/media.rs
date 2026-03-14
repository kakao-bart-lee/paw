#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaAttachment {
    pub filename: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl MediaAttachment {
    pub fn new(
        filename: impl Into<String>,
        content_type: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            filename: filename.into(),
            content_type: content_type.into(),
            bytes: bytes.into(),
        }
    }

    pub fn inferred_content_type(filename: &str) -> &'static str {
        match filename
            .rsplit('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "heic" => "image/heic",
            "mp4" => "video/mp4",
            "mov" => "video/quicktime",
            "pdf" => "application/pdf",
            _ => "application/octet-stream",
        }
    }
}
