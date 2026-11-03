use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::extensions::attachments::media_store_shared::{
    MEDIA_FILE_MODE, SaveMediaSourceError, SaveMediaSourceErrorCode, format_media_size,
};
use crate::host_paths::get_sand_root_dir;

pub const MEDIA_MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const PLAYBACK_TRANSCODE_SUBDIR: &str = "playback-transcode";
pub const MANAGED_OUTGOING_SUBDIR: &str = "outgoing";
pub const OUTBOUND_STAGING_SUBDIR: &str = "outbound";
pub const OUTBOUND_STAGING_TTL_MS: u64 = 24 * 60 * 60 * 1_000;
pub const PLAYBACK_TRANSCODE_MAX_CACHE_BYTES: u64 = 512 * 1024 * 1024;
pub const PLAYBACK_TRANSCODE_TTL_MS: u64 = 7 * 24 * 60 * 60 * 1_000;
pub const DEFAULT_TTL_MS: u64 = 2 * 60 * 1_000;
pub const MEDIA_STORE_LIFECYCLE_INTERVAL: Duration = Duration::from_secs(60 * 60);

static PLAYBACK_CACHE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub type MediaStoreLifecycleReporter = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// The sole periodic producer for media-store retention work.
/// The worker owns no media state; each pass delegates to the existing owner.
pub struct MediaStoreLifecycleWorker {
    stop: Arc<std::sync::atomic::AtomicBool>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl MediaStoreLifecycleWorker {
    pub fn start(reporter: Option<MediaStoreLifecycleReporter>) -> Result<Self, String> {
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_worker = Arc::clone(&stop);
        let join = thread::Builder::new()
            .name("fabushi-media-store-lifecycle".into())
            .spawn(move || {
                loop {
                    for (operation, result) in run_media_store_lifecycle_pass() {
                        if let Err(error) = result {
                            if let Some(report) = reporter.as_ref() {
                                report(operation, &error);
                            }
                        }
                    }
                    if stop_worker.load(std::sync::atomic::Ordering::Acquire) {
                        break;
                    }
                    thread::park_timeout(MEDIA_STORE_LIFECYCLE_INTERVAL);
                    if stop_worker.load(std::sync::atomic::Ordering::Acquire) {
                        break;
                    }
                }
            })
            .map_err(|error| format!("could not start media-store lifecycle worker: {error}"))?;
        Ok(Self {
            stop,
            join: Mutex::new(Some(join)),
        })
    }

    pub fn stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        let mut join = self.join.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(handle) = join.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

impl Drop for MediaStoreLifecycleWorker {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(handle) = self
            .join
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

pub fn run_media_store_lifecycle_pass() -> Vec<(&'static str, Result<(), String>)> {
    vec![
        (
            "clean_old_media",
            clean_old_media(DEFAULT_TTL_MS, &CleanOldMediaOptions::default(), None),
        ),
        (
            "prune_playback_transcode_cache",
            prune_playback_transcode_cache(None),
        ),
        ("prune_outbound_media", prune_outbound_media(None)),
    ]
}

fn get_playback_cache_lock() -> &'static Mutex<()> {
    PLAYBACK_CACHE_LOCK.get_or_init(|| Mutex::new(()))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedMedia {
    pub id: String,
    pub path: String,
    pub size: u64,
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CleanOldMediaOptions {
    pub recursive: Option<bool>,
    pub prune_empty_dirs: Option<bool>,
}

pub fn get_media_dir() -> PathBuf {
    get_sand_root_dir().join("media")
}

pub fn ensure_media_dir() -> Result<PathBuf, io::Error> {
    let dir = get_media_dir();
    fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }
    Ok(dir)
}

pub fn resolve_media_subdir(subdir: &str, caller: &str) -> Result<String, String> {
    if subdir.is_empty() || subdir == "." {
        return Ok(String::new());
    }
    if subdir.contains('\0') {
        return Err(format!("{caller}: unsafe media subdir: {subdir:?}"));
    }
    let p = Path::new(subdir);
    if p.is_absolute() {
        return Err(format!("{caller}: unsafe media subdir: {subdir:?}"));
    }
    // Check for drive letters or UNC on Windows or backslash in posix
    if subdir.starts_with('/') || subdir.starts_with('\\') || (subdir.len() > 1 && subdir.as_bytes()[1] == b':') {
        return Err(format!("{caller}: unsafe media subdir: {subdir:?}"));
    }

    let mut segments = Vec::new();
    for seg in subdir.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err(format!("{caller}: unsafe media subdir: {subdir:?}"));
        }
        segments.push(seg);
    }
    if segments.is_empty() {
        Ok(String::new())
    } else {
        Ok(segments.join("/"))
    }
}

pub fn resolve_media_scoped_dir(base_dir: &Path, subdir: &str, caller: &str) -> Result<PathBuf, String> {
    let safe_subdir = resolve_media_subdir(subdir, caller)?;
    let target = if safe_subdir.is_empty() {
        base_dir.to_path_buf()
    } else {
        base_dir.join(&safe_subdir)
    };
    // Canonicalize the nearest existing ancestor so a symlinked intermediate
    // directory cannot bypass containment merely because the final directory
    // has not been created yet.
    let normalized_base = canonicalize_with_missing_tail(base_dir).map_err(|error| {
        format!("{caller}: could not resolve media directory identity: {error}")
    })?;
    let normalized_target = canonicalize_with_missing_tail(&target).map_err(|error| {
        format!("{caller}: could not resolve media target identity: {error}")
    })?;
    if !normalized_target.starts_with(&normalized_base) {
        return Err(format!("{caller}: media subdir escapes media directory: {subdir:?}"));
    }
    Ok(target)
}

fn canonicalize_with_missing_tail(path: &Path) -> io::Result<PathBuf> {
    let mut missing_tail = Vec::new();
    let mut cursor = path.to_path_buf();
    loop {
        match cursor.canonicalize() {
            Ok(mut canonical) => {
                for component in missing_tail.iter().rev() {
                    canonical.push(component);
                }
                return Ok(canonical);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let Some(name) = cursor.file_name() else {
                    return Err(error);
                };
                missing_tail.push(name.to_os_string());
                let Some(parent) = cursor.parent() else {
                    return Err(error);
                };
                cursor = parent.to_path_buf();
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn resolve_media_relative_path(id: &str, subdir: &str, caller: &str) -> Result<String, String> {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains('\0') || id == ".." {
        return Err(format!("{caller}: unsafe media ID: {id:?}"));
    }
    let safe_subdir = resolve_media_subdir(subdir, caller)?;
    if safe_subdir.is_empty() {
        Ok(id.to_string())
    } else {
        Ok(format!("{safe_subdir}/{id}"))
    }
}

pub fn sanitize_filename(name: &str) -> String {
    let nfc: String = name.nfc().collect();
    let mut sanitized = String::with_capacity(nfc.len());
    for ch in nfc.chars() {
        if ch.is_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    // Collapse consecutive underscores
    let mut collapsed = String::new();
    let mut last_was_underscore = false;
    for ch in sanitized.chars() {
        if ch == '_' {
            if !last_was_underscore {
                collapsed.push(ch);
                last_was_underscore = true;
            }
        } else {
            collapsed.push(ch);
            last_was_underscore = false;
        }
    }
    let trimmed = collapsed.trim_matches('_');
    // Truncate UTF-16 code units to 60
    let mut utf16_count = 0;
    let mut byte_limit = 0;
    for (idx, ch) in trimmed.char_indices() {
        let u16_len = ch.len_utf16();
        if utf16_count + u16_len > 60 {
            break;
        }
        utf16_count += u16_len;
        byte_limit = idx + ch.len_utf8();
    }
    trimmed[..byte_limit].to_string()
}

pub fn extract_original_filename(file_path: &str) -> String {
    let basename = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.bin");
    
    let path_obj = Path::new(basename);
    let ext = path_obj.extension().and_then(|e| e.to_str()).map(|e| format!(".{e}")).unwrap_or_default();
    let stem = path_obj.file_stem().and_then(|s| s.to_str()).unwrap_or(basename);

    if let Some(idx) = stem.rfind("---") {
        let suffix = &stem[idx + 3..];
        if suffix.len() == 36 && Uuid::parse_str(suffix).is_ok() {
            let original_stem = &stem[..idx];
            if !original_stem.is_empty() {
                return format!("{original_stem}{ext}");
            }
        }
    }
    basename.to_string()
}

pub fn build_saved_media_id(base_id: &str, ext: &str, original_filename: Option<&str>) -> String {
    let ext = if ext.is_empty() || ext.starts_with('.') {
        ext.to_string()
    } else {
        format!(".{ext}")
    };

    if let Some(orig) = original_filename {
        let orig_name = Path::new(orig)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let orig_stem = Path::new(orig_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(orig_name);
        let sanitized = sanitize_filename(orig_stem);
        if !sanitized.is_empty() {
            return format!("{sanitized}---{base_id}{ext}");
        }
    }
    format!("{base_id}{ext}")
}

fn safe_original_filename_extension(original_filename: Option<&str>) -> Option<String> {
    let orig = original_filename?;
    let ext = Path::new(orig).extension().and_then(|e| e.to_str())?;
    if !ext.is_empty() && ext.len() <= 16 && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(format!(".{ext}"))
    } else {
        None
    }
}

pub fn resolve_saved_media_extension(
    detected_mime: Option<&str>,
    content_type: Option<&str>,
    original_filename: Option<&str>,
    hint_path: Option<&str>,
) -> String {
    if let Some(det) = detected_mime {
        let is_container = det == "application/zip" || det == "application/octet-stream";
        let is_image_hdr = content_type.map(|c| c.starts_with("image/")).unwrap_or(false);
        if !(is_container && is_image_hdr) {
            if let Some(ext) = extension_for_mime(det) {
                return ext;
            }
        }
    }
    if let Some(ct) = content_type {
        if ct != "application/octet-stream" && ct != "binary/octet-stream" && ct != "application/zip" {
            if let Some(ext) = extension_for_mime(ct) {
                return ext;
            }
        }
    }
    if let Some(ext) = safe_original_filename_extension(original_filename) {
        return ext;
    }
    if let Some(hint) = hint_path {
        if let Some(ext) = Path::new(hint).extension().and_then(|e| e.to_str()) {
            return format!(".{ext}");
        }
    }
    String::new()
}

fn extension_for_mime(mime: &str) -> Option<String> {
    let clean = mime.split(';').next()?.trim().to_ascii_lowercase();
    match clean.as_str() {
        "image/png" => Some(".png".to_string()),
        "image/jpeg" | "image/jpg" => Some(".jpg".to_string()),
        "image/gif" => Some(".gif".to_string()),
        "image/webp" => Some(".webp".to_string()),
        "image/svg+xml" => Some(".svg".to_string()),
        "image/bmp" => Some(".bmp".to_string()),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some(".ico".to_string()),
        "audio/mpeg" | "audio/mp3" => Some(".mp3".to_string()),
        "audio/wav" | "audio/x-wav" => Some(".wav".to_string()),
        "audio/ogg" => Some(".ogg".to_string()),
        "audio/aac" => Some(".aac".to_string()),
        "audio/flac" => Some(".flac".to_string()),
        "video/mp4" => Some(".mp4".to_string()),
        "video/webm" => Some(".webm".to_string()),
        "application/pdf" => Some(".pdf".to_string()),
        "application/json" => Some(".json".to_string()),
        "text/plain" => Some(".txt".to_string()),
        "text/html" => Some(".html".to_string()),
        "text/markdown" => Some(".md".to_string()),
        _ => None,
    }
}

pub fn detect_mime_from_buffer(buf: &[u8], hint: Option<&str>) -> Option<String> {
    if buf.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png".to_string());
    }
    if buf.starts_with(b"\xff\xd8\xff") {
        return Some("image/jpeg".to_string());
    }
    if buf.starts_with(b"GIF87a") || buf.starts_with(b"GIF89a") {
        return Some("image/gif".to_string());
    }
    if buf.starts_with(b"RIFF") && buf.len() >= 12 && &buf[8..12] == b"WEBP" {
        return Some("image/webp".to_string());
    }
    if buf.starts_with(b"BM") {
        return Some("image/bmp".to_string());
    }
    if buf.starts_with(b"%PDF-") {
        return Some("application/pdf".to_string());
    }
    if buf.starts_with(b"PK\x03\x04") {
        return Some("application/zip".to_string());
    }
    if let Some(h) = hint {
        if let Some(ext) = Path::new(h).extension().and_then(|e| e.to_str()) {
            match ext.to_ascii_lowercase().as_str() {
                "png" => return Some("image/png".to_string()),
                "jpg" | "jpeg" => return Some("image/jpeg".to_string()),
                "gif" => return Some("image/gif".to_string()),
                "webp" => return Some("image/webp".to_string()),
                "pdf" => return Some("application/pdf".to_string()),
                "txt" => return Some("text/plain".to_string()),
                "html" => return Some("text/html".to_string()),
                "json" => return Some("application/json".to_string()),
                _ => {}
            }
        }
    }
    None
}

pub fn save_media_buffer(
    buffer: &[u8],
    content_type: Option<&str>,
    subdir: &str,
    max_bytes: u64,
    original_filename: Option<&str>,
    detection_hint: Option<&str>,
    base_dir: Option<&Path>,
) -> Result<SavedMedia, SaveMediaSourceError> {
    if buffer.len() as u64 > max_bytes {
        return Err(SaveMediaSourceError::too_large(max_bytes));
    }
    let media_root = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(get_media_dir);
    let target_dir = resolve_media_scoped_dir(&media_root, subdir, "save_media_buffer")
        .map_err(|e| SaveMediaSourceError::new(SaveMediaSourceErrorCode::InvalidPath, e))?;
    fs::create_dir_all(&target_dir).map_err(|e| {
        SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::InvalidPath,
            format!("Failed to create media directory: {e}"),
        )
    })?;

    let uuid = Uuid::new_v4().to_string();
    let detected = detect_mime_from_buffer(buffer, original_filename.or(detection_hint));
    let ext = resolve_saved_media_extension(
        detected.as_deref(),
        content_type,
        original_filename,
        detection_hint,
    );
    let id = build_saved_media_id(&uuid, &ext, original_filename);

    let temp_name = format!(".{id}.tmp-{}", Uuid::new_v4());
    let temp_path = target_dir.join(&temp_name);
    let final_path = target_dir.join(&id);

    {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|e| {
                SaveMediaSourceError::new(
                    SaveMediaSourceErrorCode::InvalidPath,
                    format!("Failed to open temp file: {e}"),
                )
            })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = f.set_permissions(fs::Permissions::from_mode(MEDIA_FILE_MODE));
        }
        f.write_all(buffer).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                format!("Failed to write media: {e}"),
            )
        })?;
        f.sync_all().map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                format!("Failed to sync media: {e}"),
            )
        })?;
    }

    fs::rename(&temp_path, &final_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::PathMismatch,
            format!("Atomic rename failed: {e}"),
        )
    })?;

    let canonical_final = final_path.canonicalize().map_err(|e| {
        SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::PathMismatch,
            format!("Saved media path identity could not be verified: {e}"),
        )
    })?;

    Ok(SavedMedia {
        id,
        path: canonical_final.to_string_lossy().to_string(),
        size: buffer.len() as u64,
        content_type: detected.or_else(|| content_type.map(|s| s.to_string())),
    })
}

pub fn save_media_source(
    source: &str,
    headers: Option<&BTreeMap<String, String>>,
    subdir: &str,
    max_bytes: u64,
    base_dir: Option<&Path>,
    workspace_root: Option<&Path>,
) -> Result<SavedMedia, SaveMediaSourceError> {
    if source.starts_with("http://") || source.starts_with("https://") {
        return save_remote_media_for_store(source, headers, subdir, max_bytes, base_dir);
    }

    let source_path = Path::new(source);
    let meta = match fs::symlink_metadata(source_path) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::NotFound,
                "Media path does not exist",
            ));
        }
        Err(e) => {
            return Err(SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                format!("Media path access error: {e}"),
            ));
        }
    };

    if meta.file_type().is_symlink() {
        return Err(SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::InvalidPath,
            "Media path must not be a symlink",
        ));
    }

    if meta.is_dir() {
        return Err(SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::NotFile,
            "Media path is not a file",
        ));
    }

    if meta.len() > max_bytes {
        return Err(SaveMediaSourceError::too_large(max_bytes));
    }

    if let Some(ws) = workspace_root {
        let canonical_src = source_path.canonicalize().map_err(|e| {
            SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                format!("Failed to canonicalize path: {e}"),
            )
        })?;
        let canonical_ws = ws.canonicalize().unwrap_or_else(|_| ws.to_path_buf());
        if !canonical_src.starts_with(&canonical_ws) {
            return Err(SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                "Media path is outside workspace root",
            ));
        }
    }

    let bytes = read_media_source_bytes(source_path, max_bytes).map_err(|e| {
        SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::InvalidPath,
            format!("Failed to read source file: {e}"),
        )
    })?;

    let original_filename = source_path.file_name().and_then(|n| n.to_str());
    save_media_buffer(
        &bytes,
        None,
        subdir,
        max_bytes,
        original_filename,
        Some(source),
        base_dir,
    )
}

fn read_media_source_bytes(path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "media source is not a regular file",
        ));
    }
    let read_limit = max_bytes.saturating_add(1);
    let mut bytes = Vec::new();
    file.take(read_limit).read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn save_remote_media_for_store(
    url: &str,
    _headers: Option<&BTreeMap<String, String>>,
    subdir: &str,
    max_bytes: u64,
    base_dir: Option<&Path>,
) -> Result<SavedMedia, SaveMediaSourceError> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::InvalidPath,
            "URL must use http or https protocol",
        ));
    }

    // In native runtime without active outbound networking, we guard or download boundedly
    // For local test URLs or mock servers:
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| {
            SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                format!("HTTP client error: {e}"),
            )
        })?
        .get(url)
        .send()
        .map_err(|e| {
            SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                format!("Remote fetch error: {e}"),
            )
        })?;

    if !response.status().is_success() {
        return Err(SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::InvalidPath,
            format!("Remote server responded with {}", response.status()),
        ));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut bytes = Vec::new();
    response
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| {
            SaveMediaSourceError::new(
                SaveMediaSourceErrorCode::InvalidPath,
                format!("Failed to read remote payload: {e}"),
            )
        })?;

    if bytes.len() as u64 > max_bytes {
        return Err(SaveMediaSourceError::too_large(max_bytes));
    }

    let synthetic_ext = Path::new(url)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!("_{e}"));

    save_media_buffer(
        &bytes,
        content_type.as_deref(),
        subdir,
        max_bytes,
        synthetic_ext.as_deref(),
        Some(url),
        base_dir,
    )
}

pub fn resolve_media_buffer_path(
    id: &str,
    subdir: &str,
    base_dir: Option<&Path>,
) -> Result<PathBuf, String> {
    let media_root = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(get_media_dir);
    let target_dir = resolve_media_scoped_dir(&media_root, subdir, "resolve_media_buffer_path")?;
    let safe_id = resolve_media_relative_path(id, "", "resolve_media_buffer_path")?;
    let target_file = target_dir.join(safe_id);

    let meta = fs::symlink_metadata(&target_file).map_err(|e| {
        format!("resolveMediaBufferPath: media ID does not resolve to a file: {id:?} ({e})")
    })?;
    if meta.file_type().is_symlink() {
        return Err(format!(
            "resolveMediaBufferPath: media ID must not resolve through a symlink: {id:?}"
        ));
    }
    if !meta.is_file() {
        return Err(format!("resolveMediaBufferPath: media ID does not resolve to a file: {id:?}"));
    }
    let canonical_dir = target_dir.canonicalize().map_err(|e| {
        format!("resolveMediaBufferPath: media directory cannot be resolved: {e}")
    })?;
    let canonical_file = target_file.canonicalize().map_err(|e| {
        format!("resolveMediaBufferPath: media file cannot be resolved: {e}")
    })?;
    if !canonical_file.starts_with(&canonical_dir) {
        return Err(format!(
            "resolveMediaBufferPath: media ID resolves outside its scoped directory: {id:?}"
        ));
    }
    Ok(canonical_file)
}

pub fn read_media_buffer(
    id: &str,
    subdir: &str,
    max_bytes: u64,
    base_dir: Option<&Path>,
) -> Result<(Vec<u8>, PathBuf), SaveMediaSourceError> {
    let path = resolve_media_buffer_path(id, subdir, base_dir)
        .map_err(|e| SaveMediaSourceError::new(SaveMediaSourceErrorCode::NotFound, e))?;
    let meta = fs::metadata(&path).map_err(|e| {
        SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::NotFound,
            format!("Failed to stat media buffer: {e}"),
        )
    })?;
    if meta.len() > max_bytes {
        return Err(SaveMediaSourceError::too_large(max_bytes));
    }
    let data = read_media_source_bytes(&path, max_bytes).map_err(|e| {
        SaveMediaSourceError::new(
            SaveMediaSourceErrorCode::InvalidPath,
            format!("Failed to read media buffer: {e}"),
        )
    })?;
    if data.len() as u64 > max_bytes {
        return Err(SaveMediaSourceError::too_large(max_bytes));
    }
    Ok((data, path))
}

pub fn delete_media_buffer(
    id: &str,
    subdir: &str,
    base_dir: Option<&Path>,
) -> Result<(), String> {
    let media_root = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(get_media_dir);
    let target_dir = resolve_media_scoped_dir(&media_root, subdir, "delete_media_buffer")?;
    let safe_id = resolve_media_relative_path(id, "", "delete_media_buffer")?;
    let target_file = target_dir.join(safe_id);
    if target_file.exists() {
        fs::remove_file(&target_file).map_err(|e| format!("Failed to delete media buffer: {e}"))?;
    }
    Ok(())
}

pub fn write_playback_transcode_cache(
    file_name: &str,
    buffer: &[u8],
    max_bytes: u64,
    base_dir: Option<&Path>,
) -> Result<PathBuf, String> {
    let _guard = get_playback_cache_lock().lock().unwrap();
    if buffer.len() as u64 > max_bytes {
        return Err(format!("Playback buffer exceeds max_bytes {max_bytes}"));
    }
    let media_root = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(get_media_dir);
    let cache_dir = resolve_media_scoped_dir(&media_root, PLAYBACK_TRANSCODE_SUBDIR, "write_playback_transcode_cache")?;
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache dir: {e}"))?;

    let safe_file_name = resolve_media_relative_path(
        file_name,
        "",
        "write_playback_transcode_cache",
    )?;
    let target_path = cache_dir.join(&safe_file_name);
    let temp_name = format!(".{safe_file_name}.tmp-{}", Uuid::new_v4());
    let temp_path = cache_dir.join(&temp_name);

    {
        let mut f = File::create(&temp_path).map_err(|e| format!("Failed to create temp cache file: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = f.set_permissions(fs::Permissions::from_mode(MEDIA_FILE_MODE));
        }
        f.write_all(buffer).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            format!("Failed to write cache file: {e}")
        })?;
    }
    fs::rename(&temp_path, &target_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        format!("Failed to rename cache file: {e}")
    })?;

    prune_playback_transcode_cache_to_size(&cache_dir, PLAYBACK_TRANSCODE_MAX_CACHE_BYTES)?;
    Ok(target_path)
}

pub fn prune_playback_transcode_cache_to_size(cache_dir: &Path, max_bytes: u64) -> Result<(), String> {
    if !cache_dir.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(cache_dir).map_err(|e| format!("Failed to read cache dir: {e}"))?;
    let mut files = Vec::new();
    let mut total_bytes: u64 = 0;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                let mtime = meta.modified().unwrap_or(UNIX_EPOCH);
                let size = meta.len();
                total_bytes += size;
                files.push((mtime, name, size, entry.path()));
            }
        }
    }

    // Sort by mtime ascending (oldest first)
    files.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    for (_, _, size, path) in files {
        if total_bytes <= max_bytes {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total_bytes = total_bytes.saturating_sub(size);
        }
    }
    Ok(())
}

pub fn prune_playback_transcode_cache(base_dir: Option<&Path>) -> Result<(), String> {
    let _guard = get_playback_cache_lock().lock().unwrap();
    let media_root = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(get_media_dir);
    let cache_dir = resolve_media_scoped_dir(&media_root, PLAYBACK_TRANSCODE_SUBDIR, "prune_playback_transcode_cache")?;
    if !cache_dir.exists() {
        return Ok(());
    }

    let now = SystemTime::now();
    let ttl = Duration::from_millis(PLAYBACK_TRANSCODE_TTL_MS);

    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    if let Ok(mtime) = meta.modified() {
                        if let Ok(age) = now.duration_since(mtime) {
                            if age > ttl {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
    prune_playback_transcode_cache_to_size(&cache_dir, PLAYBACK_TRANSCODE_MAX_CACHE_BYTES)
}

pub fn prune_outbound_media(base_dir: Option<&Path>) -> Result<(), String> {
    let media_root = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(get_media_dir);
    let outbound_dir = resolve_media_scoped_dir(&media_root, OUTBOUND_STAGING_SUBDIR, "prune_outbound_media")?;
    if !outbound_dir.exists() {
        return Ok(());
    }

    let now = SystemTime::now();
    let ttl = Duration::from_millis(OUTBOUND_STAGING_TTL_MS);

    if let Ok(entries) = fs::read_dir(&outbound_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    if let Ok(mtime) = meta.modified() {
                        if let Ok(age) = now.duration_since(mtime) {
                            if age > ttl {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn clean_old_media(
    ttl_ms: u64,
    options: &CleanOldMediaOptions,
    base_dir: Option<&Path>,
) -> Result<(), String> {
    let media_root = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(get_media_dir);
    cleanup::run(
        &media_root,
        Duration::from_millis(ttl_ms),
        options,
        SystemTime::now(),
        &mut |_| Ok(()),
    )
}

/// Private mechanics of the existing TTL owner, not a second filesystem authority.
/// All traversal and mutation below the admitted root use retained handles. The
/// Unix final stat/unlinkat pair is not an atomic conditional unlink: a replacement
/// in that last kernel-call interval can still change which same-parent name is
/// unlinked. Likewise, ancestry checks cannot freeze concurrent directory moves.
/// These controls do not establish the separate media read/publication contract.
mod cleanup {
    use super::{CleanOldMediaOptions, MANAGED_OUTGOING_SUBDIR, PLAYBACK_TRANSCODE_SUBDIR};
    use std::collections::HashSet;
    use std::ffi::{OsStr, OsString};
    use std::fs::File;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

    struct Directory {
        path: PathBuf,
        name: OsString,
        handle: File,
        identity: platform::Identity,
        entries: platform::Entries,
        seen: HashSet<OsString>,
        retain: bool,
    }

    fn expired(modified: Option<SystemTime>, now: SystemTime, ttl: Duration) -> bool {
        modified
            .and_then(|mtime| now.duration_since(mtime).ok())
            .is_some_and(|age| age > ttl)
    }

    fn protected(name: &OsStr) -> bool {
        #[cfg(any(windows, target_os = "macos"))]
        {
            // macOS can preserve a caller's spelling for a case-insensitive
            // protected store. Keep its aliases out of destructive traversal,
            // conservatively including case-sensitive macOS volumes.
            name.to_str().is_some_and(|name| {
                name.eq_ignore_ascii_case(PLAYBACK_TRANSCODE_SUBDIR)
                    || name.eq_ignore_ascii_case(MANAGED_OUTGOING_SUBDIR)
            })
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            name == PLAYBACK_TRANSCODE_SUBDIR || name == MANAGED_OUTGOING_SUBDIR
        }
    }

    // A missing/replaced root is an error after admission. A changed child
    // invalidates only its retained branch; its successor is never adopted.
    fn current(stack: &[Directory]) -> Result<Option<usize>, String> {
        let root = &stack[0];
        let current = platform::open_root(&root.path)
            .and_then(|file| platform::identity(&file))
            .map_err(|error| format!("Media root identity could not be rechecked: {error}"))?;
        if current != root.identity {
            return Err("Media root identity changed during cleanup".into());
        }
        for index in 1..stack.len() {
            let reopened = platform::open_directory(
                &stack[index - 1].handle,
                &stack[index].name,
            )
            .and_then(|file| platform::identity(&file));
            if !reopened.is_ok_and(|identity| identity == stack[index].identity) {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    fn retain_changed_branch(stack: &mut Vec<Directory>) -> Result<bool, String> {
        if let Some(index) = current(stack)? {
            stack.truncate(index);
            stack[index - 1].retain = true;
            return Ok(true);
        }
        Ok(false)
    }

    // The callback is a private deterministic seam at the actual removal
    // boundary. Production supplies a no-op; a refusal takes the same recovery
    // path as a native unlink error. Reobservation always follows the callback.
    pub(super) fn run(
        root: &Path,
        ttl: Duration,
        options: &CleanOldMediaOptions,
        now: SystemTime,
        before_remove: &mut impl FnMut(&Path) -> io::Result<()>,
    ) -> Result<(), String> {
        let root: PathBuf = std::path::absolute(root)
            .map_err(|error| format!("Failed to resolve media root: {error}"))?
            .components()
            .collect();
        let handle = match platform::open_root(&root) {
            Ok(handle) => handle,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(format!("Failed to admit media root: {error}")),
        };
        let identity = platform::identity(&handle)
            .map_err(|error| format!("Failed to admit media root identity: {error}"))?;
        let entries = platform::Entries::new(&handle)
            .map_err(|error| format!("Failed to read media root: {error}"))?;
        let mut stack = vec![Directory {
            path: root,
            name: OsString::new(),
            handle,
            identity,
            entries,
            seen: HashSet::new(),
            retain: false,
        }];

        loop {
            if retain_changed_branch(&mut stack)? {
                continue;
            }
            let index = stack.len() - 1;
            let next = stack[index].entries.next();
            let name = match next {
                Some(Ok(name)) => name,
                Some(Err(error)) => {
                    if index == 0 {
                        return Err(format!("Failed to read media root: {error}"));
                    }
                    stack.pop();
                    stack[index - 1].retain = true;
                    continue;
                }
                None => {
                    if index == 0 {
                        current(&stack)?;
                        return Ok(());
                    }
                    let mut removed = false;
                    if options.prune_empty_dirs == Some(true) && !stack[index].retain {
                        let admission = before_remove(&stack[index].path);
                        if retain_changed_branch(&mut stack)? {
                            continue;
                        }
                        // Re-enumeration must succeed and be genuinely empty.
                        let empty = platform::Entries::new(&stack[index].handle)
                            .map(|mut entries| matches!(entries.next(), None))
                            .unwrap_or(false);
                        if empty {
                            if retain_changed_branch(&mut stack)? {
                                continue;
                            }
                            let parent = &stack[index - 1].handle;
                            let directory = &stack[index];
                            let result = (|| -> io::Result<bool> {
                                let fresh = platform::observe(parent, &directory.name, true)?;
                                if !fresh.is_dir() || fresh.identity() != directory.identity {
                                    return Ok(false);
                                }
                                admission?;
                                platform::remove(parent, &directory.name, &fresh, true)?;
                                Ok(true)
                            })();
                            removed = match result {
                                Ok(removed) => removed,
                                Err(error) if index == 1 && !matches!(
                                    error.kind(),
                                    io::ErrorKind::NotFound
                                        | io::ErrorKind::DirectoryNotEmpty
                                        | io::ErrorKind::AlreadyExists
                                ) => {
                                    return Err(format!(
                                        "Failed to remove top-level media directory {}: {error}",
                                        directory.path.display(),
                                    ));
                                }
                                Err(_) => false,
                            };
                        }
                    }
                    stack.pop();
                    stack[index - 1].retain |= !removed;
                    continue;
                }
            };

            // Directory enumeration under concurrent rename is not a snapshot.
            // Never readmit an already processed name as a successor this pass.
            if !stack[index].seen.insert(name.clone()) {
                stack[index].retain = true;
                continue;
            }
            let mut components = Path::new(&name).components();
            if !matches!(components.next(), Some(std::path::Component::Normal(_)))
                || components.next().is_some()
            {
                stack[index].retain = true;
                continue;
            }
            if index == 0 && protected(&name) {
                stack[index].retain = true;
                continue;
            }
            let parent = &stack[index].handle;
            let observed = match platform::observe(parent, &name, false) {
                Ok(observed) => observed,
                Err(error) => {
                    stack[index].retain |= error.kind() != io::ErrorKind::NotFound;
                    continue;
                }
            };
            let path = stack[index].path.join(&name);
            if observed.is_dir() {
                let descend = match options.recursive {
                    Some(true) => true,
                    Some(false) => false,
                    None => index == 0,
                };
                if descend {
                    let child = (|| -> io::Result<Directory> {
                        let handle = platform::open_directory(parent, &name)?;
                        let identity = platform::identity(&handle)?;
                        if observed.identity() != identity {
                            return Err(io::Error::other("media directory identity changed"));
                        }
                        let entries = platform::Entries::new(&handle)?;
                        Ok(Directory { path, name, handle, identity, entries, seen: HashSet::new(), retain: false })
                    })();
                    if let Ok(child) = child {
                        stack.push(child);
                        continue;
                    }
                }
                stack[index].retain = true;
            } else if observed.is_file() && expired(observed.modified(), now, ttl) {
                let admission = before_remove(&path);
                if retain_changed_branch(&mut stack)? {
                    continue;
                }
                let parent = &stack[index].handle;
                let removed = (|| -> io::Result<bool> {
                    let fresh = platform::observe(parent, &name, true)?;
                    if !fresh.is_file()
                        || observed.identity() != fresh.identity()
                        || !expired(fresh.modified(), now, ttl)
                    {
                        return Ok(false);
                    }
                    admission?;
                    platform::remove(parent, &name, &fresh, false)?;
                    Ok(true)
                })()
                .unwrap_or(false);
                stack[index].retain |= !removed;
            } else {
                // Live/unknown times, links and special entries all survive.
                stack[index].retain = true;
            }
        }
    }

    #[cfg(unix)]
    mod platform {
        use std::ffi::{CStr, CString, OsStr, OsString};
        use std::fs::File;
        use std::io;
        use std::os::fd::{AsRawFd, FromRawFd};
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        use std::os::unix::fs::MetadataExt;
        use std::path::Path;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        fn name(value: &OsStr) -> io::Result<CString> {
            CString::new(value.as_bytes())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in media path"))
        }

        fn open_at(parent: libc::c_int, path: &OsStr) -> io::Result<File> {
            let path = name(path)?;
            let fd = unsafe {
                libc::openat(
                    parent,
                    path.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(unsafe { File::from_raw_fd(fd) })
            }
        }

        pub(super) fn open_root(path: &Path) -> io::Result<File> {
            open_at(libc::AT_FDCWD, path.as_os_str())
        }

        pub(super) fn open_directory(parent: &File, name: &OsStr) -> io::Result<File> {
            open_at(parent.as_raw_fd(), name)
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(super) struct Identity { device: u64, inode: u64 }

        pub(super) fn identity(file: &File) -> io::Result<Identity> {
            let metadata = file.metadata()?;
            Ok(Identity { device: metadata.dev(), inode: metadata.ino() })
        }

        pub(super) struct Entry(libc::stat);

        impl Entry {
            pub(super) fn is_dir(&self) -> bool {
                self.0.st_mode & libc::S_IFMT == libc::S_IFDIR
            }
            pub(super) fn is_file(&self) -> bool {
                self.0.st_mode & libc::S_IFMT == libc::S_IFREG
            }
            pub(super) fn identity(&self) -> Identity {
                Identity { device: self.0.st_dev as u64, inode: self.0.st_ino as u64 }
            }
            pub(super) fn modified(&self) -> Option<SystemTime> {
                let nanos = self.0.st_mtime_nsec as i128;
                if !(0..1_000_000_000).contains(&nanos) {
                    return None;
                }
                let total = self.0.st_mtime as i128 * 1_000_000_000 + nanos;
                let absolute = total.unsigned_abs();
                let duration = Duration::new(
                    (absolute / 1_000_000_000).try_into().ok()?,
                    (absolute % 1_000_000_000) as u32,
                );
                if total < 0 {
                    UNIX_EPOCH.checked_sub(duration)
                } else {
                    UNIX_EPOCH.checked_add(duration)
                }
            }
        }

        pub(super) fn observe(parent: &File, leaf: &OsStr, _deleting: bool) -> io::Result<Entry> {
            let leaf = name(leaf)?;
            let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
            let result = unsafe {
                libc::fstatat(parent.as_raw_fd(), leaf.as_ptr(), metadata.as_mut_ptr(), libc::AT_SYMLINK_NOFOLLOW)
            };
            if result < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(Entry(unsafe { metadata.assume_init() }))
            }
        }

        pub(super) fn remove(parent: &File, leaf: &OsStr, _entry: &Entry, directory: bool) -> io::Result<()> {
            let leaf = name(leaf)?;
            let flags = if directory { libc::AT_REMOVEDIR } else { 0 };
            if unsafe { libc::unlinkat(parent.as_raw_fd(), leaf.as_ptr(), flags) } < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        fn errno() -> io::Result<*mut libc::c_int> {
            #[cfg(any(target_os = "linux", target_os = "hurd"))]
            return Ok(unsafe { libc::__errno_location() });
            #[cfg(any(target_vendor = "apple", target_os = "freebsd", target_os = "dragonfly"))]
            return Ok(unsafe { libc::__error() });
            #[cfg(any(target_os = "android", target_os = "netbsd", target_os = "openbsd"))]
            return Ok(unsafe { libc::__errno() });
            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            return Ok(unsafe { libc::___errno() });
            #[cfg(not(any(
                target_os = "linux", target_os = "hurd", target_vendor = "apple",
                target_os = "freebsd", target_os = "dragonfly", target_os = "android",
                target_os = "netbsd", target_os = "openbsd", target_os = "solaris",
                target_os = "illumos",
            )))]
            Err(io::Error::new(io::ErrorKind::Unsupported, "media directory enumeration is unavailable"))
        }

        pub(super) struct Entries(*mut libc::DIR);

        impl Entries {
            pub(super) fn new(directory: &File) -> io::Result<Self> {
                use std::os::fd::IntoRawFd;
                errno()?;
                // A new open description avoids sharing a readdir cursor with
                // the fresh emptiness check performed during postorder pruning.
                let fd = open_directory(directory, OsStr::new("."))?.into_raw_fd();
                let stream = unsafe { libc::fdopendir(fd) };
                if stream.is_null() {
                    let error = io::Error::last_os_error();
                    unsafe { libc::close(fd) };
                    Err(error)
                } else {
                    Ok(Self(stream))
                }
            }
        }

        impl Iterator for Entries {
            type Item = io::Result<OsString>;
            fn next(&mut self) -> Option<Self::Item> {
                let errno = match errno() {
                    Ok(errno) => errno,
                    Err(error) => return Some(Err(error)),
                };
                loop {
                    unsafe { *errno = 0 };
                    let entry = unsafe { libc::readdir(self.0) };
                    if entry.is_null() {
                        let code = unsafe { *errno };
                        return (code != 0).then(|| Err(io::Error::from_raw_os_error(code)));
                    }
                    let bytes = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
                    if bytes != b"." && bytes != b".." {
                        return Some(Ok(OsString::from_vec(bytes.to_vec())));
                    }
                }
            }
        }

        impl Drop for Entries {
            fn drop(&mut self) {
                unsafe { libc::closedir(self.0) };
            }
        }
    }

    #[cfg(windows)]
    mod platform {
        use std::ffi::{OsStr, OsString, c_void};
        use std::fs::{File, Metadata, OpenOptions};
        use std::io;
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        use std::os::windows::io::{AsRawHandle, FromRawHandle};
        use std::path::Path;
        use std::ptr::null_mut;
        use std::time::SystemTime;

        type NativeHandle = *mut c_void;
        const READ_ATTRIBUTES: u32 = 0x80;
        const LIST_DIRECTORY: u32 = 0x1;
        const DELETE: u32 = 0x0001_0000;
        const SYNCHRONIZE: u32 = 0x0010_0000;
        const SHARE_ALL: u32 = 0x7;
        const DIRECTORY_FILE: u32 = 0x1;
        const OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const REPARSE_ATTRIBUTE: u32 = 0x400;

        // FILE_ID_INFO retains all 128 bits, including on ReFS. The existing
        // same-file 1.0.6 Windows equality uses a truncated 64-bit index.
        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(super) struct Identity {
            pub(super) volume: u64,
            pub(super) id: [u8; 16],
        }

        impl Identity {
            pub(super) fn admit(self) -> io::Result<Self> {
                // A successful query may still report an unavailable or
                // nonunique ID. Never compare two unknown objects as equal.
                if self.id == [0; 16] || self.id == [0xff; 16] {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "media file identity is unavailable or nonunique",
                    ));
                }
                Ok(self)
            }
        }

        // Same relative-handle ABI used by native_file_sandbox's private
        // Windows mechanism. That owner's JSON request API is not a TTL API.
        #[repr(C)]
        struct UnicodeString { length: u16, maximum_length: u16, buffer: *mut u16 }
        #[repr(C)]
        struct ObjectAttributes {
            length: u32,
            root_directory: NativeHandle,
            object_name: *mut UnicodeString,
            attributes: u32,
            security_descriptor: *mut c_void,
            security_quality_of_service: *mut c_void,
        }
        #[repr(C)]
        union IoStatusValue { status: i32, pointer: *mut c_void }
        #[repr(C)]
        struct IoStatusBlock { value: IoStatusValue, information: usize }

        #[link(name = "ntdll")]
        unsafe extern "system" {
            fn NtCreateFile(
                handle: *mut NativeHandle, access: u32, attributes: *mut ObjectAttributes,
                status: *mut IoStatusBlock, allocation: *mut i64, file_attributes: u32,
                sharing: u32, disposition: u32, options: u32, ea: *mut c_void, ea_length: u32,
            ) -> i32;
            fn NtQueryDirectoryFile(
                handle: NativeHandle, event: NativeHandle, apc: *mut c_void,
                context: *mut c_void, status: *mut IoStatusBlock, information: *mut c_void,
                length: u32, information_class: u32, single_entry: u8,
                name: *mut UnicodeString, restart: u8,
            ) -> i32;
            fn RtlNtStatusToDosError(status: i32) -> u32;
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetFileInformationByHandleEx(
                handle: NativeHandle, information_class: u32,
                information: *mut c_void, size: u32,
            ) -> i32;
            fn SetFileInformationByHandle(
                handle: NativeHandle, information_class: u32,
                information: *const c_void, size: u32,
            ) -> i32;
        }

        fn status_error(status: i32) -> io::Error {
            io::Error::from_raw_os_error(unsafe { RtlNtStatusToDosError(status) } as i32)
        }

        pub(super) fn identity(file: &File) -> io::Result<Identity> {
            let mut identity = Identity { volume: 0, id: [0; 16] };
            let result = unsafe {
                GetFileInformationByHandleEx(
                    file.as_raw_handle(), 18, // FileIdInfo
                    &mut identity as *mut Identity as *mut c_void,
                    std::mem::size_of::<Identity>() as u32,
                )
            };
            if result == 0 { Err(io::Error::last_os_error()) } else { identity.admit() }
        }

        fn metadata(file: &File) -> io::Result<Metadata> {
            let metadata = file.metadata()?;
            if metadata.file_attributes() & REPARSE_ATTRIBUTE != 0 {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "media reparse point refused"));
            }
            Ok(metadata)
        }

        fn open_at(parent: &File, name: &OsStr, access: u32, directory: bool) -> io::Result<File> {
            let mut wide: Vec<u16> = name.encode_wide().collect();
            let length = wide.len().checked_mul(2).and_then(|n| u16::try_from(n).ok())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "media name is too long"))?;
            if wide.contains(&0) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "NUL in media name"));
            }
            let mut name = UnicodeString { length, maximum_length: length, buffer: wide.as_mut_ptr() };
            let mut attributes = ObjectAttributes {
                length: std::mem::size_of::<ObjectAttributes>() as u32,
                root_directory: parent.as_raw_handle(),
                object_name: &mut name,
                attributes: 0, // Preserve the exact enumerated spelling, including case-sensitive directories.
                security_descriptor: null_mut(),
                security_quality_of_service: null_mut(),
            };
            let mut status = IoStatusBlock { value: IoStatusValue { status: 0 }, information: 0 };
            let mut handle = null_mut();
            let result = unsafe {
                NtCreateFile(
                    &mut handle, access | SYNCHRONIZE, &mut attributes, &mut status,
                    null_mut(), 0, if access & DELETE != 0 { 0x3 } else { SHARE_ALL },
                    1, OPEN_REPARSE_POINT | 0x20 | if directory { DIRECTORY_FILE } else { 0 },
                    null_mut(), 0,
                )
            };
            if result < 0 {
                return Err(status_error(result));
            }
            let file = unsafe { File::from_raw_handle(handle) };
            metadata(&file)?;
            Ok(file)
        }

        pub(super) fn open_root(path: &Path) -> io::Result<File> {
            let file = OpenOptions::new().read(true).share_mode(SHARE_ALL)
                .custom_flags(0x0200_0000 | OPEN_REPARSE_POINT).open(path)?;
            if !metadata(&file)?.is_dir() {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "media root is not a directory"));
            }
            Ok(file)
        }

        pub(super) fn open_directory(parent: &File, name: &OsStr) -> io::Result<File> {
            open_at(parent, name, LIST_DIRECTORY | READ_ATTRIBUTES, true)
        }

        pub(super) struct Entry { handle: File, identity: Identity, metadata: Metadata }
        impl Entry {
            pub(super) fn is_dir(&self) -> bool { self.metadata.is_dir() }
            pub(super) fn is_file(&self) -> bool { self.metadata.is_file() }
            pub(super) fn modified(&self) -> Option<SystemTime> { self.metadata.modified().ok() }
            pub(super) fn identity(&self) -> Identity { self.identity }
        }

        pub(super) fn observe(parent: &File, name: &OsStr, deleting: bool) -> io::Result<Entry> {
            let file = open_at(parent, name, READ_ATTRIBUTES | if deleting { DELETE } else { 0 }, false)?;
            let metadata = metadata(&file)?;
            let identity = identity(&file)?;
            Ok(Entry { handle: file, identity, metadata })
        }

        pub(super) fn remove(_parent: &File, _name: &OsStr, entry: &Entry, _directory: bool) -> io::Result<()> {
            // FileDispositionInfoEx: DELETE | POSIX_SEMANTICS. The fresh handle
            // denies delete-sharing until it closes, and targets this identity,
            // never a pathname successor. Nonempty directories are refused.
            let flags: u32 = 0x1 | 0x2;
            let result = unsafe {
                SetFileInformationByHandle(
                    entry.handle.as_raw_handle(), 21,
                    &flags as *const u32 as *const c_void, std::mem::size_of::<u32>() as u32,
                )
            };
            if result == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
        }

        pub(super) struct Entries { file: File, restart: bool, finished: bool }
        impl Entries {
            pub(super) fn new(directory: &File) -> io::Result<Self> {
                // Only one scan of a directory is active at once. The postorder
                // scan explicitly restarts after the original scan has ended.
                Ok(Self { file: directory.try_clone()?, restart: true, finished: false })
            }
        }
        impl Iterator for Entries {
            type Item = io::Result<OsString>;
            fn next(&mut self) -> Option<Self::Item> {
                if self.finished { return None; }
                loop {
                    // FILE_NAMES_INFORMATION is aligned and length-delimited.
                    // Request one entry so no untrusted offset chain is needed.
                    let mut buffer = [0u32; 16_384];
                    let mut status = IoStatusBlock { value: IoStatusValue { status: 0 }, information: 0 };
                    let result = unsafe {
                        NtQueryDirectoryFile(
                            self.file.as_raw_handle(), null_mut(), null_mut(), null_mut(),
                            &mut status, buffer.as_mut_ptr() as *mut c_void,
                            std::mem::size_of_val(&buffer) as u32, 12, 1, null_mut(),
                            u8::from(self.restart),
                        )
                    };
                    self.restart = false;
                    if result == 0x80000006u32 as i32 { // STATUS_NO_MORE_FILES
                        self.finished = true;
                        return None;
                    }
                    if result < 0 {
                        self.finished = true;
                        return Some(Err(status_error(result)));
                    }
                    let length = buffer[2] as usize;
                    if status.information < 12 || status.information > std::mem::size_of_val(&buffer)
                        || buffer[0] != 0 || length == 0 || length % 2 != 0
                        || length > status.information - 12
                    {
                        self.finished = true;
                        return Some(Err(io::Error::new(io::ErrorKind::InvalidData, "incomplete media directory entry")));
                    }
                    let wide = unsafe {
                        std::slice::from_raw_parts(buffer.as_ptr().cast::<u16>().add(6), length / 2)
                    };
                    let name = OsString::from_wide(wide);
                    if name != "." && name != ".." { return Some(Ok(name)); }
                }
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use super::run;
        use super::super::{CleanOldMediaOptions, MEDIA_MAX_BYTES, clean_old_media, save_media_buffer};
        use std::fs::{self, FileTimes};
        use std::io;
        use std::path::{Path, PathBuf};
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        struct Fixture(PathBuf);
        impl Fixture {
            fn new() -> Self {
                let path = std::env::temp_dir().join(format!("media-cleanup-{}", uuid::Uuid::new_v4()));
                fs::create_dir(&path).unwrap();
                // A fixture normalization only, so hooks and SavedMedia use the
                // same spelling on macOS /tmp aliases and Windows verbatim paths.
                Self(path.canonicalize().unwrap())
            }
            fn root(&self) -> PathBuf { self.0.join("media") }
            fn save(&self, subdir: &str, name: &str, modified: SystemTime) -> PathBuf {
                let saved = save_media_buffer(
                    b"original media", Some("text/plain"), subdir, MEDIA_MAX_BYTES,
                    Some(name), None, Some(&self.root()),
                ).unwrap();
                let path = PathBuf::from(saved.path);
                set_modified(&path, modified);
                path
            }
        }
        impl Drop for Fixture {
            fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
        }
        fn set_modified(path: &Path, modified: SystemTime) {
            fs::OpenOptions::new().write(true).open(path).unwrap()
                .set_times(FileTimes::new().set_modified(modified)).unwrap();
        }
        fn options() -> CleanOldMediaOptions {
            CleanOldMediaOptions { recursive: Some(true), prune_empty_dirs: Some(true) }
        }

        #[cfg(windows)]
        #[test]
        fn cleanup_windows_identity_compares_full_file_id() {
            use super::platform::Identity;
            let first = Identity { volume: 42, id: [7; 16] };
            assert_eq!(first.admit().unwrap(), first);
            let mut different_high_half = first;
            different_high_half.id[15] ^= 1;
            assert_eq!(&first.id[..8], &different_high_half.id[..8]);
            assert_ne!(first, different_high_half, "matching low halves do not establish identity");
            assert_ne!(first, Identity { volume: 43, ..first }, "the volume is part of identity");
            assert_eq!(first, Identity { volume: 42, id: [7; 16] });

            for id in [[0; 16], [0xff; 16]] {
                assert_eq!(
                    Identity { volume: 42, id }.admit().unwrap_err().kind(),
                    io::ErrorKind::Unsupported,
                    "API success cannot admit an unavailable or nonunique file ID",
                );
            }
            let mut full_width = Identity { volume: 42, id: [0; 16] };
            full_width.id[8] = 1;
            assert_eq!(full_width.admit().unwrap(), full_width,
                "a zero low half does not make a valid full-width ID unavailable");
        }

        #[test]
        fn cleanup_uses_one_clock_and_strict_ttl_boundary() {
            let fixture = Fixture::new();
            let boundary = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
            let ttl = Duration::from_secs(60);
            let now = boundary + ttl;
            let equal = fixture.save("cache/session", "equal.txt", boundary);
            let expired = fixture.save("cache/session", "expired.txt", boundary - Duration::from_secs(1));
            let future = fixture.save("cache/session", "future.txt", now + Duration::from_secs(1));

            run(&fixture.root(), ttl, &options(), now, &mut |_| Ok(())).unwrap();

            assert!(equal.exists(), "an age equal to TTL is retained");
            assert!(!expired.exists());
            assert!(future.exists(), "the pass must not sample a later wall clock");
        }

        #[test]
        fn cleanup_rechecks_refreshed_mtime_before_unlink() {
            let fixture = Fixture::new();
            let now = SystemTime::now();
            let old = now - Duration::from_secs(3_600);
            let target = fixture.save("cache/session/images", "refresh.txt", old);
            let mut refreshed = false;
            run(&fixture.root(), Duration::from_secs(60), &options(), now, &mut |path| {
                if path == target.as_path() {
                    set_modified(path, now + Duration::from_secs(3_600));
                    refreshed = true;
                }
                Ok(())
            }).unwrap();

            assert!(refreshed, "the real owner reached its removal boundary");
            assert_eq!(fs::read(&target).unwrap(), b"original media");
            assert!(target.parent().unwrap().is_dir());
        }

        #[test]
        fn cleanup_refuses_replaced_leaf() {
            let fixture = Fixture::new();
            let now = SystemTime::now();
            let old = now - Duration::from_secs(3_600);
            let target = fixture.save("cache/session/images", "replace.txt", old);
            let retired = fixture.0.join("retired.txt");
            let mut replaced = false;
            run(&fixture.root(), Duration::from_secs(60), &options(), now, &mut |path| {
                if path == target.as_path() && !replaced {
                    fs::rename(path, &retired).unwrap();
                    fs::write(path, b"successor media").unwrap();
                    set_modified(path, old);
                    replaced = true;
                }
                Ok(())
            }).unwrap();

            assert!(replaced);
            assert_eq!(fs::read(&target).unwrap(), b"successor media");
            assert_eq!(fs::read(&retired).unwrap(), b"original media");
        }

        #[test]
        fn cleanup_refuses_replaced_ancestor() {
            for symlink in [false, true] {
                let fixture = Fixture::new();
                let now = SystemTime::now();
                let old = now - Duration::from_secs(3_600);
                let root = fixture.root();
                let target = fixture.save("cache/session/images", "replace.txt", old);
                let other = fixture.save("other", "expired.txt", old);
                let relative = target.strip_prefix(root.join("cache")).unwrap().to_path_buf();
                let retired = fixture.0.join("retired-cache");
                let replacement = if symlink { fixture.0.join("outside") } else { root.join("cache") };
                let successor = replacement.join(&relative);
                let mut replaced = false;

                run(&root, Duration::from_secs(60), &options(), now, &mut |path| {
                    if path == target.as_path() && !replaced {
                        fs::rename(root.join("cache"), &retired).unwrap();
                        fs::create_dir_all(successor.parent().unwrap()).unwrap();
                        fs::write(&successor, b"successor media").unwrap();
                        set_modified(&successor, old);
                        if symlink {
                            #[cfg(unix)]
                            std::os::unix::fs::symlink(&replacement, root.join("cache")).unwrap();
                            #[cfg(windows)]
                            std::os::windows::fs::symlink_dir(&replacement, root.join("cache")).unwrap();
                        }
                        replaced = true;
                    }
                    Ok(())
                }).unwrap();

                assert!(replaced);
                assert_eq!(fs::read(retired.join(&relative)).unwrap(), b"original media");
                assert_eq!(fs::read(&successor).unwrap(), b"successor media");
                assert!(!other.exists(), "unaffected branches still make progress");
                assert!(root.is_dir());
            }
        }

        #[test]
        fn cleanup_refuses_replaced_root() {
            let fixture = Fixture::new();
            let now = SystemTime::now();
            let old = now - Duration::from_secs(3_600);
            let root = fixture.root();
            let target = fixture.save("cache/session/images", "replace.txt", old);
            let relative = target.strip_prefix(&root).unwrap().to_path_buf();
            let retired = fixture.0.join("retired-media");
            let mut replaced = false;

            let result = run(&root, Duration::from_secs(60), &options(), now, &mut |path| {
                if path == target.as_path() && !replaced {
                    fs::rename(&root, &retired).unwrap();
                    fs::create_dir_all(target.parent().unwrap()).unwrap();
                    fs::write(&target, b"successor media").unwrap();
                    set_modified(&target, old);
                    replaced = true;
                }
                Ok(())
            });

            assert!(replaced);
            assert!(result.unwrap_err().contains("Media root identity"));
            assert_eq!(fs::read(retired.join(relative)).unwrap(), b"original media");
            assert_eq!(fs::read(&target).unwrap(), b"successor media");
        }

        #[test]
        fn cleanup_retries_failed_unlink_without_pruning_branch() {
            let fixture = Fixture::new();
            let now = SystemTime::now();
            let old = now - Duration::from_secs(3_600);
            let retained = fixture.save("cache/session/images", "refused.txt", old);
            let other = fixture.save("independent/deep", "expired.txt", old);
            let mut refused = false;

            run(&fixture.root(), Duration::from_secs(60), &options(), now, &mut |path| {
                if path == retained.as_path() {
                    refused = true;
                    return Err(io::Error::new(io::ErrorKind::PermissionDenied, "controlled unlink refusal"));
                }
                Ok(())
            }).unwrap();

            assert!(refused);
            assert!(retained.exists());
            assert!(retained.parent().unwrap().is_dir());
            assert!(!other.exists());
            assert!(!fixture.root().join("independent").exists());
            clean_old_media(60_000, &options(), Some(&fixture.root())).unwrap();
            assert!(!retained.exists(), "a normal later call retries the unchanged expired file");
            assert!(!fixture.root().join("cache").exists());
            assert!(fixture.root().is_dir());
        }

        #[test]
        fn cleanup_top_level_prune_errors_propagate_with_partial_progress() {
            let fixture = Fixture::new();
            let now = SystemTime::now();
            let old = now - Duration::from_secs(3_600);
            let root = fixture.root();
            let file = fixture.save("cache/session/images", "expired.txt", old);
            let nested = file.parent().unwrap().to_path_buf();
            let top = root.join("cache");
            let mut refused_nested = false;
            run(&root, Duration::from_secs(60), &options(), now, &mut |path| {
                if path == nested.as_path() {
                    refused_nested = true;
                    return Err(io::Error::new(io::ErrorKind::PermissionDenied, "controlled nested prune refusal"));
                }
                Ok(())
            }).unwrap();
            assert!(refused_nested, "nested directory refusal must remain best effort");
            assert!(!file.exists());
            assert!(nested.is_dir());

            let partial = fixture.save("cache", "partial.txt", old);
            let mut refused_top = false;
            let result = run(&root, Duration::from_secs(60), &options(), now, &mut |path| {
                if path == top.as_path() {
                    refused_top = true;
                    return Err(io::Error::new(io::ErrorKind::PermissionDenied, "controlled top-level prune refusal"));
                }
                Ok(())
            });
            assert!(refused_top);
            assert!(result.unwrap_err().contains("controlled top-level prune refusal"));
            assert!(!partial.exists(), "earlier file removal is honest partial progress");
            assert!(!nested.exists(), "earlier nested pruning also remains committed");
            assert!(top.is_dir());
            assert!(fs::read_dir(&top).unwrap().next().is_none());

            clean_old_media(60_000, &options(), Some(&root)).unwrap();
            assert!(!top.exists(), "a normal later call retries the refused top-level prune");
            assert!(root.is_dir());
        }
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};

static ACTIVE_REMOTE_FIXTURE_SCOPES: AtomicUsize = AtomicUsize::new(0);

/// Scoped remote fixture simulating network interceptors for media store tests.
#[derive(Debug, Clone)]
pub struct StoreRemoteFixtureScope {
    pub url: String,
    pub active: bool,
    pub matched_count: usize,
}

impl StoreRemoteFixtureScope {
    pub fn new(url: &str) -> Self {
        ACTIVE_REMOTE_FIXTURE_SCOPES.fetch_add(1, Ordering::SeqCst);
        Self {
            url: url.to_string(),
            active: true,
            matched_count: 0,
        }
    }

    pub fn matches(&mut self, request_url: &str) -> bool {
        if self.active && self.url == request_url {
            self.matched_count += 1;
            true
        } else {
            false
        }
    }

    pub fn deactivate(&mut self) {
        if self.active {
            self.active = false;
            ACTIVE_REMOTE_FIXTURE_SCOPES.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

impl Drop for StoreRemoteFixtureScope {
    fn drop(&mut self) {
        self.deactivate();
    }
}

/// Verifies that all active test fixtures have joined and completed.
pub fn check_active_remote_fixtures_empty() -> Result<(), String> {
    let count = ACTIVE_REMOTE_FIXTURE_SCOPES.load(Ordering::SeqCst);
    if count != 0 {
        Err(format!("Store remote fixtures are still active: {count}"))
    } else {
        Ok(())
    }
}

/// Helper asserting original filename patterns from `store-filename.test-support.ts`.
pub fn expect_saved_original_filename_case(
    saved: &SavedMedia,
    expected_original_filename: Option<&str>,
    expect_uuid_only: bool,
) -> Result<(), String> {
    if expect_uuid_only {
        if saved.id.contains("---") {
            return Err(format!("Expected UUID-only ID, got: {}", saved.id));
        }
    }
    if let Some(expected) = expected_original_filename {
        let extracted = extract_original_filename(&saved.path);
        if extracted != expected {
            return Err(format!(
                "Original filename mismatch: expected '{expected}', extracted '{extracted}'"
            ));
        }
    }
    Ok(())
}
