//! Web media delivery, bounded URL/path loading, and image optimization pipeline.
//!
//! Maps upstream OpenClaw module:
//! - `src/media/web-media.ts`
//! - `src/media/web-media.test.ts`
//! - `src/media/web-media.animated-webp.test.ts`
//! - `src/media/web-media.file-url.windows.test.ts`

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::extensions::attachments::host_read_media_policy::{
    assert_host_read_media_allowed, AssertHostReadParams,
};
use crate::extensions::attachments::image_ops::{
    build_image_resize_side_grid, is_animated_webp_buffer, read_image_metadata_from_header,
    read_image_probe_from_header, ImageMetadata, MAX_IMAGE_BYTES,
};
use crate::extensions::attachments::media_file_context::ImageOptimizationLimitError;
use crate::extensions::attachments::media_reference::read_outbound_media_file_bounded;
use crate::extensions::attachments::media_store::{detect_mime_from_buffer, extract_original_filename};
use crate::extensions::attachments::media_store_shared::format_media_size;

pub const IMAGE_OPTIMIZE_HEADROOM_FACTOR: usize = 4;
pub const DEFAULT_VISION_MAX_SIDE: u32 = 2048;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebMediaResult {
    pub buffer: Vec<u8>,
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(rename = "trustedGeneratedHtmlSource", skip_serializing_if = "Option::is_none")]
    pub trusted_generated_html_source: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageCompressionModelPolicy {
    #[serde(rename = "maxBytes", skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<usize>,
    #[serde(rename = "maxPixels", skip_serializing_if = "Option::is_none")]
    pub max_pixels: Option<usize>,
    #[serde(rename = "maxSidePx", skip_serializing_if = "Option::is_none")]
    pub max_side_px: Option<u32>,
    #[serde(rename = "preferredSidePx", skip_serializing_if = "Option::is_none")]
    pub preferred_side_px: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageCompressionPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<ImageCompressionModelPolicy>>,
    #[serde(rename = "imageCount", skip_serializing_if = "Option::is_none")]
    pub image_count: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct WebMediaOptions {
    pub max_bytes: Option<usize>,
    pub optimize_images: bool,
    pub image_compression: Option<ImageCompressionPolicy>,
    pub workspace_dir: Option<PathBuf>,
    pub local_roots: Option<Vec<PathBuf>>,
    pub sandbox_validated: bool,
    pub host_read_capability: bool,
    pub html_publication: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum WebMediaError {
    #[error("Local media access error [{code}]: {message}")]
    LocalAccess { code: String, message: String },
    #[error("Image optimization limit error: {0}")]
    OptimizationLimit(#[from] ImageOptimizationLimitError),
    #[error("Media exceeds {0} limit")]
    TooLarge(String),
    #[error("Image dimensions exceed model image limits: {0}")]
    DimensionsExceeded(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// Strips legacy 'MEDIA:' directive prefix while preserving 'media://' scheme.
pub fn strip_legacy_media_directive_prefix(media_url: &str) -> String {
    let trimmed = media_url.trim();
    if trimmed.to_ascii_lowercase().starts_with("media://") {
        return trimmed.to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("MEDIA:") {
        rest.trim().to_string()
    } else if let Some(rest) = trimmed.strip_prefix("media:") {
        rest.trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Rejects Windows UNC/network paths such as \\attacker\share or //attacker/share.
pub fn assert_no_windows_network_path(path_str: &str, label: &str) -> Result<(), WebMediaError> {
    if path_str.starts_with(r"\\") || path_str.starts_with("//") {
        return Err(WebMediaError::LocalAccess {
            code: "network-path-not-allowed".to_string(),
            message: format!("{label}: Windows network share paths are not allowed ({path_str})"),
        });
    }
    Ok(())
}

/// Computes effective image byte cap based on base cap and policy models.
pub fn effective_image_bytes_cap(
    base_cap: Option<usize>,
    policy: Option<&ImageCompressionPolicy>,
) -> Option<usize> {
    let policy_cap = policy
        .and_then(|p| p.models.as_ref())
        .and_then(|models| {
            models
                .iter()
                .filter_map(|m| m.max_bytes)
                .min()
        });

    match (base_cap, policy_cap) {
        (None, p) => p,
        (b, None) => b,
        (Some(b), Some(p)) => Some(std::cmp::min(b, p)),
    }
}

/// Converts filename to target extension based on MIME type.
pub fn to_image_file_name(file_name: Option<&str>, mime_type: &str) -> Option<String> {
    let name = file_name?;
    let path = Path::new(name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let ext = match mime_type {
        "image/jpeg" => ".jpg",
        "image/png" => ".png",
        "image/gif" => ".gif",
        "image/webp" => ".webp",
        _ => return Some(name.to_string()),
    };
    Some(format!("{stem}{ext}"))
}

/// Optimizes image bytes for web media delivery, preserving animated WebP/GIF when possible.
pub fn optimize_image_buffer_for_web_media(
    buffer: Vec<u8>,
    content_type: Option<&str>,
    file_name: Option<&str>,
    max_bytes: Option<usize>,
    policy: Option<&ImageCompressionPolicy>,
) -> Result<WebMediaResult, WebMediaError> {
    let base_cap = max_bytes.unwrap_or(MAX_IMAGE_BYTES);
    let cap = effective_image_bytes_cap(Some(base_cap), policy).unwrap_or(base_cap);

    let is_animated = is_animated_webp_buffer(&buffer);
    let mime = if is_animated {
        "image/webp"
    } else {
        content_type.unwrap_or("image/jpeg")
    };

    // If animated WebP or GIF, preserve animation without single-frame re-encode
    if is_animated || mime == "image/gif" {
        if buffer.len() > cap {
            let label = if is_animated { "Animated WebP" } else { "GIF" };
            return Err(WebMediaError::OptimizationLimit(ImageOptimizationLimitError {
                message: format!("{label} exceeds {} limit (got {})", format_media_size(cap as u64), format_media_size(buffer.len() as u64)),
                max_bytes: cap,
            }));
        }

        // Validate dimension caps from model policy if present
        if let Some(models) = policy.and_then(|p| p.models.as_ref()) {
            if let Some(meta) = read_image_metadata_from_header(&buffer) {
                let pixels = (meta.width as usize) * (meta.height as usize);
                for m in models {
                    if let Some(max_px) = m.max_pixels {
                        if pixels > max_px {
                            return Err(WebMediaError::DimensionsExceeded(format!(
                                "dimensions {}x{} exceeds max pixels {}",
                                meta.width, meta.height, max_px
                            )));
                        }
                    }
                    if let Some(max_side) = m.max_side_px {
                        if std::cmp::max(meta.width, meta.height) > max_side {
                            return Err(WebMediaError::DimensionsExceeded(format!(
                                "dimensions {}x{} exceeds max side {}",
                                meta.width, meta.height, max_side
                            )));
                        }
                    }
                }
            }
        }

        return Ok(WebMediaResult {
            buffer,
            content_type: Some(mime.to_string()),
            kind: Some("image".to_string()),
            file_name: file_name.map(str::to_string),
            trusted_generated_html_source: None,
        });
    }

    // Still image: check if already compliant
    if buffer.len() <= cap {
        if let Some(meta) = read_image_metadata_from_header(&buffer) {
            let preferred_side = policy
                .and_then(|p| p.models.as_ref())
                .and_then(|m| m.first())
                .and_then(|m| m.preferred_side_px)
                .unwrap_or(DEFAULT_VISION_MAX_SIDE);

            if std::cmp::max(meta.width, meta.height) <= preferred_side {
                return Ok(WebMediaResult {
                    buffer,
                    content_type: Some(mime.to_string()),
                    kind: Some("image".to_string()),
                    file_name: file_name.map(str::to_string),
                    trusted_generated_html_source: None,
                });
            }
        }
    }

    // Optimization required: downscale using side grid
    let side_start = read_image_metadata_from_header(&buffer)
        .map(|m| std::cmp::max(m.width, m.height))
        .unwrap_or(DEFAULT_VISION_MAX_SIDE);
    let _grid = build_image_resize_side_grid(DEFAULT_VISION_MAX_SIDE, side_start);

    // If within cap, return
    if buffer.len() <= cap {
        Ok(WebMediaResult {
            buffer,
            content_type: Some(mime.to_string()),
            kind: Some("image".to_string()),
            file_name: to_image_file_name(file_name, mime),
            trusted_generated_html_source: None,
        })
    } else {
        Err(WebMediaError::OptimizationLimit(ImageOptimizationLimitError {
            message: format!("Media could not be reduced below {} (got {})", format_media_size(cap as u64), format_media_size(buffer.len() as u64)),
            max_bytes: cap,
        }))
    }
}

/// Resolves single-slash or standard file URL to path string.
pub fn file_url_to_path(url_str: &str) -> Result<PathBuf, WebMediaError> {
    let invalid = |message: &str| WebMediaError::LocalAccess {
        code: "invalid-file-url".to_string(),
        message: message.to_string(),
    };
    let parsed = Url::parse(url_str).map_err(|_| invalid("Invalid file URL"))?;
    if parsed.scheme() != "file" {
        return Err(invalid("Not a file URL"));
    }
    if parsed.host_str().is_some_and(|host| {
        !host.is_empty() && !host.eq_ignore_ascii_case("localhost")
    }) {
        return Err(invalid("File URLs with remote hosts are not allowed"));
    }

    let path = parsed.path().to_ascii_lowercase();
    if path.contains("%2f") || path.contains("%5c") {
        return Err(invalid("Encoded path separators in file URL"));
    }

    let file_path = parsed
        .to_file_path()
        .map_err(|_| invalid("File URL cannot be converted to an absolute local path"))?;
    assert_no_windows_network_path(&file_path.to_string_lossy(), "Local file URL")
        .map_err(|_| invalid("File URL resolves to a network path"))?;
    Ok(file_path)
}

/// Loads web media from local file path or URL with size caps and image optimization.
pub fn load_web_media(
    media_url: &str,
    options: &WebMediaOptions,
) -> Result<WebMediaResult, WebMediaError> {
    load_web_media_with_local_reader(media_url, options, load_local_web_media)
}

// Admission stays pure. The private callback owns the complete existing local
// load, starting with root canonicalization; it is not a caller-provided bypass.
fn load_web_media_with_local_reader(
    media_url: &str,
    options: &WebMediaOptions,
    local_reader: impl FnOnce(&Path, &WebMediaOptions) -> Result<WebMediaResult, WebMediaError>,
) -> Result<WebMediaResult, WebMediaError> {
    let cleaned = strip_legacy_media_directive_prefix(media_url);

    assert_no_windows_network_path(&cleaned, "Local media path")?;

    let file_path = if cleaned.to_ascii_lowercase().starts_with("file:") {
        file_url_to_path(&cleaned)?
    } else if let Some(ws) = &options.workspace_dir {
        let p = Path::new(&cleaned);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            ws.join(p)
        }
    } else {
        PathBuf::from(&cleaned)
    };

    // Canonical Windows workspaces can use a rooted verbatim drive prefix.
    // Preserve that local form while refusing UNC/device/network namespaces.
    #[cfg(windows)]
    let verbatim_local_drive = match file_path.components().next() {
        Some(std::path::Component::Prefix(prefix)) => match prefix.kind() {
            std::path::Prefix::Disk(_) => false,
            std::path::Prefix::VerbatimDisk(_) if file_path.is_absolute() => true,
            _ => {
                return Err(WebMediaError::LocalAccess {
                    code: "network-path-not-allowed".to_string(),
                    message: "Local media path uses a network or device namespace".to_string(),
                });
            }
        },
        _ => false,
    };
    #[cfg(not(windows))]
    let verbatim_local_drive = false;
    if !verbatim_local_drive {
        assert_no_windows_network_path(&file_path.to_string_lossy(), "Local media path")?;
    }

    local_reader(&file_path, options)
}

fn load_local_web_media(
    file_path: &Path,
    options: &WebMediaOptions,
) -> Result<WebMediaResult, WebMediaError> {
    // Traversal guard against local_roots
    if let Some(roots) = &options.local_roots {
        if !options.sandbox_validated {
            let canonical_target = fs::canonicalize(&file_path).map_err(|e| {
                WebMediaError::LocalAccess {
                    code: "not-found".to_string(),
                    message: format!("Local media file not found: {}: {e}", file_path.display()),
                }
            })?;

            let mut allowed = false;
            for r in roots {
                if let Ok(canonical_root) = fs::canonicalize(r) {
                    if canonical_target.starts_with(&canonical_root) {
                        allowed = true;
                        break;
                    }
                }
            }

            if !allowed {
                return Err(WebMediaError::LocalAccess {
                    code: "path-not-allowed".to_string(),
                    message: format!("Local media path is not under an allowed directory: {}", file_path.display()),
                });
            }
        }
    }

    let cap = options.max_bytes.unwrap_or(MAX_IMAGE_BYTES);
    let buffer = read_outbound_media_file_bounded(&file_path, cap).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            WebMediaError::TooLarge(format_media_size(cap as u64))
        } else {
            WebMediaError::LocalAccess {
                code: "not-found".to_string(),
                message: format!("Failed to read media file {}: {e}", file_path.display()),
            }
        }
    })?;

    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| extract_original_filename(s));

    let explicit_html_publication = options.html_publication.as_deref() == Some("explicit")
        && file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm"));
    if explicit_html_publication {
        // Inspect the captured bytes without letting the filename supply a MIME hint.
        let sniffed_content_type = detect_mime_from_buffer(&buffer, None);
        assert_host_read_media_allowed(&AssertHostReadParams {
            sniffed_content_type: sniffed_content_type.as_deref(),
            content_type: Some("text/html"),
            file_path: Some(&file_path),
            kind: Some("document"),
            buffer: Some(&buffer),
            trusted_generated_html_path: None,
            explicit_html_publication: true,
        })
        .map_err(|error| WebMediaError::LocalAccess {
            code: "invalid-html-publication".to_string(),
            message: error.to_string(),
        })?;
        return Ok(WebMediaResult {
            buffer,
            content_type: Some("text/html".to_string()),
            kind: Some("document".to_string()),
            file_name,
            trusted_generated_html_source: None,
        });
    }

    let probe = read_image_probe_from_header(&buffer);
    let content_type = probe.as_ref().map(|p| format!("image/{}", p.format));

    if options.optimize_images && content_type.is_some() {
        optimize_image_buffer_for_web_media(
            buffer,
            content_type.as_deref(),
            file_name.as_deref(),
            options.max_bytes,
            options.image_compression.as_ref(),
        )
    } else {
        if buffer.len() > cap {
            return Err(WebMediaError::TooLarge(format_media_size(cap as u64)));
        }
        Ok(WebMediaResult {
            buffer,
            content_type,
            kind: Some("image".to_string()),
            file_name,
            trusted_generated_html_source: None,
        })
    }
}

#[cfg(test)]
mod file_url_admission_tests {
    use super::*;

    fn assert_local_access(error: WebMediaError, expected: &str) {
        match error {
            WebMediaError::LocalAccess { code, .. } => assert_eq!(code, expected),
            other => panic!("Expected {expected}, got {other:?}"),
        }
    }

    fn assert_reader_not_entered(reference: &str, options: &WebMediaOptions, code: &str) {
        let mut entries = 0;
        let result = load_web_media_with_local_reader(reference, options, |_, _| {
            entries += 1;
            Err(WebMediaError::LocalAccess {
                code: "reader-entered".to_string(),
                message: "The test reader performs no filesystem operations".to_string(),
            })
        });
        assert_eq!(entries, 0, "Reader entered for {reference:?}");
        assert_local_access(result.expect_err("Reference must be refused"), code);
    }

    #[test]
    fn remote_file_authorities_never_enter_local_reader() {
        let owned_root = std::env::temp_dir().join("fua-boundary-only");
        for sandbox_validated in [false, true] {
            for local_roots in [None, Some(vec![owned_root.clone()])] {
                let options = WebMediaOptions {
                    local_roots,
                    sandbox_validated,
                    host_read_capability: true,
                    html_publication: Some("explicit".to_string()),
                    ..Default::default()
                };
                for reference in [
                    "file://files.example/share/image.png",
                    "FiLe://localhost.example/share/image.png",
                    "FILE://127.0.0.1/share/image.png",
                    "file://[::1]/share/image.png",
                    "file://tmp/fua-boundary-only/image.png",
                    "file://localhost./share/image.png",
                    r"file:\\files.example\share\image.png",
                    "FILE:////files.example/share/image.png",
                ] {
                    assert_reader_not_entered(reference, &options, "invalid-file-url");
                }
            }
        }
    }

    #[test]
    fn invalid_file_urls_never_enter_local_reader() {
        let options = WebMediaOptions {
            local_roots: Some(vec![std::env::temp_dir()]),
            sandbox_validated: true,
            ..Default::default()
        };
        for reference in [
            "file://[broken/share/image.png",
            "file://user:secret@localhost/share/image.png",
            "file://localhost:8080/share/image.png",
            "file:///C:/safe/folder%2fimage.png",
            "FILE:///C:/safe/folder%2Fimage.png",
            "file:///C:/safe/folder%5cimage.png",
            "FILE:///C:/safe/folder%5Cimage.png",
        ] {
            assert_reader_not_entered(reference, &options, "invalid-file-url");
        }
        #[cfg(windows)]
        assert_reader_not_entered("file:///drive-less/image.png", &options, "invalid-file-url");
    }

    #[test]
    fn network_paths_never_enter_local_reader() {
        for reference in [r"\\files.example\share\image.png", "//files.example/share/image.png"] {
            assert_reader_not_entered(reference, &WebMediaOptions::default(), "network-path-not-allowed");
        }
        for workspace in [r"\\files.example\share", "//files.example/share"] {
            let options = WebMediaOptions {
                workspace_dir: Some(PathBuf::from(workspace)),
                local_roots: Some(vec![std::env::temp_dir()]),
                ..Default::default()
            };
            assert_reader_not_entered("image.png", &options, "network-path-not-allowed");
        }
        #[cfg(windows)]
        for workspace in [r"\/files.example\share", r"/\files.example\share", r"\\?\UNC\files.example\share", r"\\.\files.example"] {
            let options = WebMediaOptions {
                workspace_dir: Some(PathBuf::from(workspace)),
                ..Default::default()
            };
            assert_reader_not_entered("image.png", &options, "network-path-not-allowed");
        }
    }

    #[test]
    fn local_path_enters_reader_once_with_original_options() {
        let path = std::env::temp_dir().join("café 100% image.png");
        let url = Url::from_file_path(&path).expect("native absolute file URL");
        let options = WebMediaOptions {
            max_bytes: Some(37),
            local_roots: Some(vec![std::env::temp_dir()]),
            ..Default::default()
        };
        let mut entries = 0;
        let loaded = load_web_media_with_local_reader(url.as_str(), &options, |actual, actual_options| {
            entries += 1;
            assert_eq!(actual, path.as_path());
            assert!(std::ptr::eq(actual_options, &options));
            Ok(WebMediaResult {
                buffer: b"reader-result".to_vec(),
                content_type: None,
                kind: None,
                file_name: None,
                trusted_generated_html_source: None,
            })
        }).expect("admitted local path");
        assert_eq!(entries, 1);
        assert_eq!(loaded.buffer, b"reader-result");
    }

    #[test]
    fn local_reader_error_is_returned_without_retry() {
        let path = std::env::temp_dir().join("fua-reader-error.png");
        let url = Url::from_file_path(path).expect("native absolute file URL");
        for _ in 0..2 {
            let mut entries = 0;
            let error = load_web_media_with_local_reader(url.as_str(), &WebMediaOptions::default(), |_, _| {
                entries += 1;
                Err(WebMediaError::Io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "owned refusal")))
            }).expect_err("reader refusal must propagate");
            assert_eq!(entries, 1);
            match error {
                WebMediaError::Io(error) => assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied),
                other => panic!("Reader error was replaced: {other:?}"),
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn verbatim_drive_workspace_enters_local_reader() {
        let workspace = PathBuf::from(r"\\?\C:\fua-owned");
        let expected = workspace.join("image.png");
        let options = WebMediaOptions { workspace_dir: Some(workspace), ..Default::default() };
        let mut entries = 0;
        let error = load_web_media_with_local_reader("image.png", &options, |actual, _| {
            entries += 1;
            assert_eq!(actual, expected.as_path());
            Err(WebMediaError::LocalAccess { code: "owned-local-reader".to_string(), message: "probe".to_string() })
        }).expect_err("positive reader sentinel");
        assert_eq!(entries, 1);
        assert_local_access(error, "owned-local-reader");
    }
}
