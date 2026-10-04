use std::path::Path;

pub const ATTACHMENT_BYTE_LIMIT: u64 = 25 * 1024 * 1024;
pub const VIDEO_BYTE_LIMIT: u64 = 200 * 1024 * 1024;
pub const BYTES_PER_MB: u64 = 1024 * 1024;

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{}", value.to_ascii_lowercase()))
}

pub fn image_mime_from_path(path: &Path) -> Option<&'static str> {
    match extension(path)?.as_str() {
        ".avif" => Some("image/avif"),
        ".bmp" => Some("image/bmp"),
        ".gif" => Some("image/gif"),
        ".ico" => Some("image/x-icon"),
        ".jpeg" | ".jpg" => Some("image/jpeg"),
        ".png" => Some("image/png"),
        ".svg" => Some("image/svg+xml"),
        ".webp" => Some("image/webp"),
        _ => None,
    }
}

pub fn servable_image_mime_from_path(path: &Path) -> Option<&'static str> {
    image_mime_from_path(path).or_else(|| match extension(path)?.as_str() {
        ".heic" => Some("image/heic"),
        ".heif" => Some("image/heif"),
        _ => None,
    })
}

pub fn video_mime_from_path(path: &Path) -> Option<&'static str> {
    match extension(path)?.as_str() {
        ".m4v" | ".mp4" => Some("video/mp4"),
        ".mov" => Some("video/quicktime"),
        ".ogv" => Some("video/ogg"),
        ".webm" => Some("video/webm"),
        _ => None,
    }
}

pub fn audio_mime_from_path(path: &Path) -> Option<&'static str> {
    match extension(path)?.as_str() {
        ".aac" => Some("audio/aac"),
        ".flac" => Some("audio/flac"),
        ".m4a" => Some("audio/mp4"),
        ".mp3" => Some("audio/mpeg"),
        ".oga" | ".ogg" | ".opus" => Some("audio/ogg"),
        ".wav" => Some("audio/wav"),
        ".weba" => Some("audio/webm"),
        _ => None,
    }
}

pub fn extension_from_image_mime(mime: &str) -> Option<&'static str> {
    match mime.to_ascii_lowercase().as_str() {
        "image/avif" => Some(".avif"),
        "image/bmp" => Some(".bmp"),
        "image/gif" => Some(".gif"),
        "image/jpeg" => Some(".jpg"),
        "image/png" => Some(".png"),
        "image/svg+xml" => Some(".svg"),
        "image/webp" => Some(".webp"),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some(".ico"),
        _ => None,
    }
}

pub fn name_looks_like_video(path: &Path) -> bool {
    video_mime_from_path(path).is_some()
}

pub fn attachment_byte_limit_for_name(path: &Path) -> u64 {
    if name_looks_like_video(path) {
        VIDEO_BYTE_LIMIT
    } else {
        ATTACHMENT_BYTE_LIMIT
    }
}

pub fn format_megabytes(bytes: u64) -> String {
    format!("{} MB", (bytes + (BYTES_PER_MB / 2)) / BYTES_PER_MB)
}

pub fn format_attachment_too_large_notice(filename: &str) -> String {
    let video = name_looks_like_video(Path::new(filename));
    let limit = if video { VIDEO_BYTE_LIMIT } else { ATTACHMENT_BYTE_LIMIT };
    format!(
        "\"{filename}\" is too large to attach (max {}{}).",
        format_megabytes(limit),
        if video { " for video" } else { "" }
    )
}
