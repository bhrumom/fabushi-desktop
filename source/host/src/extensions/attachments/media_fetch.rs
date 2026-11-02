//! Media fetch pipeline with SSRF guards, Content-Disposition parsing,
//! streaming byte bounds, Content-Type/charset retention, and media store persistence.
//!
//! Source-informed media owner; scoped contracts track remaining equivalence gaps:
//! - `src/media/fetch.ts`
//! - `src/media/fetch.read-scope.ts`

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use percent_encoding::percent_decode_str;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::extensions::attachments::media_store::{
    detect_mime_from_buffer, resolve_saved_media_extension,
};
use crate::extensions::attachments::media_store_shared::{
    SaveMediaSourceError, SaveMediaSourceErrorCode,
};

pub const DEFAULT_MEDIA_RESPONSE_HEADER_TIMEOUT_MS: u64 = 15 * 60_000;
pub const MAX_DOCUMENT_BYTES: usize = 50 * 1024 * 1024; // 50MB
const MAX_HTTP_ERROR_BODY_BYTES: usize = 8 * 1024;
const MAX_HTTP_ERROR_SNIPPET_UTF16_UNITS: usize = 200;

fn read_bounded_response(
    response: &mut reqwest::blocking::Response,
    max_bytes: usize,
) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    response
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn is_ecmascript_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000d}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    )
}

fn http_error_body_snippet(bytes: &[u8]) -> Option<String> {
    let byte_truncated = bytes.len() >= MAX_HTTP_ERROR_BODY_BYTES;
    let prefix = &bytes[..bytes.len().min(MAX_HTTP_ERROR_BODY_BYTES)];
    // Match TextDecoder's UTF-8-only BOM removal. With last=false, dropping
    // this one-shot decoder discards only an unfinished trailing scalar.
    let mut decoder = encoding_rs::UTF_8.new_decoder_with_bom_removal();
    let mut decoded = String::with_capacity(decoder.max_utf8_buffer_length(prefix.len())?);
    let (result, read, _) = decoder.decode_to_string(prefix, &mut decoded, !byte_truncated);
    if !matches!(result, encoding_rs::CoderResult::InputEmpty) || read != prefix.len() {
        return None;
    }
    let mut collapsed = String::new();
    let mut pending_space = false;
    for character in decoded.chars() {
        if is_ecmascript_whitespace(character) {
            pending_space = !collapsed.is_empty();
        } else {
            if pending_space {
                collapsed.push(' ');
                pending_space = false;
            }
            collapsed.push(character);
        }
    }
    if collapsed.is_empty() {
        return None;
    }

    let mut utf16_units = 0;
    let display_end = collapsed.char_indices().find_map(|(offset, character)| {
        utf16_units += character.len_utf16();
        (utf16_units > MAX_HTTP_ERROR_SNIPPET_UTF16_UNITS).then_some(offset)
    });
    if let Some(end) = display_end {
        collapsed.truncate(end);
    }
    if byte_truncated || display_end.is_some() {
        collapsed.push('…');
    }
    Some(collapsed)
}

fn http_error_from_body(source_url: &str, status: u16, bytes: &[u8]) -> MediaFetchError {
    let detail = http_error_body_snippet(bytes)
        .map(|snippet| format!("body: {snippet}"))
        .unwrap_or_default();
    MediaFetchError::http_error(source_url, status, &detail)
}

fn read_http_error_response(
    response: &mut reqwest::blocking::Response,
    source_url: &str,
    status: u16,
) -> MediaFetchError {
    let mut prefix = Vec::with_capacity(MAX_HTTP_ERROR_BODY_BYTES);
    // Unlike successful payload reads, diagnostics stop at the budget itself:
    // no extra byte or EOF is needed, and returning drops the owned response.
    match response
        .take(MAX_HTTP_ERROR_BODY_BYTES as u64)
        .read_to_end(&mut prefix)
    {
        Ok(_) => http_error_from_body(source_url, status, &prefix),
        Err(_) => MediaFetchError::http_error(source_url, status, ""),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaFetchErrorCode {
    MaxBytes,
    HttpError,
    FetchFailed,
}

impl fmt::Display for MediaFetchErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxBytes => write!(f, "max_bytes"),
            Self::HttpError => write!(f, "http_error"),
            Self::FetchFailed => write!(f, "fetch_failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFetchError {
    pub code: MediaFetchErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

impl MediaFetchError {
    pub fn new(code: MediaFetchErrorCode, message: impl Into<String>, status: Option<u16>) -> Self {
        Self {
            code,
            message: message.into(),
            status,
        }
    }

    pub fn max_bytes(source_url: &str, message: impl Into<String>) -> Self {
        let msg = message.into();
        let full = if msg.starts_with("Failed to fetch") {
            msg
        } else {
            format!("Failed to fetch media from {}: {}", redact_sensitive_text(source_url), msg)
        };
        Self::new(MediaFetchErrorCode::MaxBytes, full, None)
    }

    pub fn http_error(source_url: &str, status: u16, detail: &str) -> Self {
        let detail = redact_sensitive_text(detail);
        let full = format!(
            "Failed to fetch media from {}: HTTP {}{}",
            redact_sensitive_text(source_url),
            status,
            if detail.is_empty() { String::new() } else { format!("; {detail}") }
        );
        Self::new(MediaFetchErrorCode::HttpError, full, Some(status))
    }

    pub fn http_error_message(source_url: &str, message: impl Into<String>, status: Option<u16>) -> Self {
        let msg = message.into();
        let full = if msg.starts_with("Failed to fetch") {
            msg
        } else {
            format!("Failed to fetch media from {}: {}", redact_sensitive_text(source_url), msg)
        };
        Self::new(MediaFetchErrorCode::HttpError, full, status)
    }

    pub fn fetch_failed(source_url: &str, cause: impl Into<String>) -> Self {
        let cause_str = cause.into();
        let full = format!(
            "Failed to fetch media from {}: {}",
            redact_sensitive_text(source_url),
            redact_sensitive_text(&cause_str)
        );
        Self::new(MediaFetchErrorCode::FetchFailed, full, None)
    }
}

impl fmt::Display for MediaFetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MediaFetchError: {}", self.message)
    }
}

impl std::error::Error for MediaFetchError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchMediaResult {
    pub buffer: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchMediaResultWithHeaders {
    pub buffer: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRemoteMedia {
    pub id: String,
    pub path: String,
    pub size: u64,
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchMediaOptions {
    pub url: String,
    #[serde(default)]
    pub file_path_hint: Option<String>,
    #[serde(default)]
    pub max_bytes: Option<usize>,
    #[serde(default)]
    pub fallback_content_type: Option<String>,
    #[serde(default)]
    pub require_https: Option<bool>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub response_header_timeout_ms: Option<u64>,
    #[serde(default)]
    pub read_idle_timeout_ms: Option<u64>,
    #[serde(default)]
    pub allowed_origins: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_hostnames: Option<Vec<String>>,
    /// Restrictive destination patterns; unlike allowed_hostnames, this grants no private trust.
    #[serde(default)]
    pub hostname_allowlist: Option<Vec<String>>,
    #[serde(default)]
    pub max_redirects: Option<usize>,
    #[serde(default)]
    pub allow_rfc2544_benchmark_range: Option<bool>,
    #[serde(default)]
    pub trust_explicit_proxy_dns: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveRemoteMediaOptions {
    pub url: String,
    #[serde(default)]
    pub file_path_hint: Option<String>,
    #[serde(default)]
    pub max_bytes: Option<usize>,
    #[serde(default)]
    pub fallback_content_type: Option<String>,
    #[serde(default)]
    pub subdir: Option<String>,
    #[serde(default)]
    pub original_filename: Option<String>,
    #[serde(default)]
    pub require_https: Option<bool>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub read_idle_timeout_ms: Option<u64>,
    #[serde(default)]
    pub allowed_origins: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_hostnames: Option<Vec<String>>,
    #[serde(default)]
    pub base_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveResponseMediaOptions {
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub file_path_hint: Option<String>,
    #[serde(default)]
    pub max_bytes: Option<usize>,
    #[serde(default)]
    pub read_idle_timeout_ms: Option<u64>,
    #[serde(default)]
    pub fallback_content_type: Option<String>,
    #[serde(default)]
    pub subdir: Option<String>,
    #[serde(default)]
    pub original_filename: Option<String>,
    #[serde(default)]
    pub base_dir: Option<PathBuf>,
}

/// Redact the supported Telegram bot-token pattern from text and URLs.
pub fn redact_sensitive_text(text: &str) -> String {
    let re = Regex::new(r"bot(\d{6,}:[A-Za-z0-9_-]{10,})").unwrap();
    re.replace_all(text, |caps: &regex::Captures| {
        let token = &caps[1];
        if token.chars().count() >= 18 {
            let start: String = token.chars().take(6).collect();
            let end: String = token.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
            format!("bot{start}…{end}")
        } else {
            "bot***".to_string()
        }
    }).to_string()
}

/// Returns the final filename segment for either POSIX or Windows-style paths.
pub fn basename_from_any_path(value: &str) -> String {
    let trimmed = value.trim_end_matches(|c| c == '/' || c == '\\');
    let last = trimmed.rsplit(|c| c == '/' || c == '\\').next().unwrap_or("");
    if last == "." || last == ".." {
        String::new()
    } else {
        last.to_string()
    }
}

/// Returns the extension from the final filename segment of any path flavor.
pub fn extname_from_any_path(value: &str) -> String {
    let base = basename_from_any_path(value);
    if let Some(pos) = base.rfind('.') {
        if pos > 0 && pos < base.len() - 1 {
            return base[pos..].to_string();
        }
    }
    String::new()
}

pub fn parse_content_disposition_parameters(header: &str) -> Vec<(String, String)> {
    let mut params = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
    let chars: Vec<char> = header.chars().collect();
    let len = chars.len();

    for index in 0..=len {
        let character = if index < len { Some(chars[index]) } else { None };
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && character == Some('\\') {
            escaped = true;
            continue;
        }
        if character == Some('"') {
            quoted = !quoted;
            continue;
        }
        if index != len && (quoted || character != Some(';')) {
            continue;
        }
        let end = index;
        if end >= start {
            let parameter: String = chars[start..end].iter().collect();
            let parameter = parameter.trim();
            if let Some(separator) = parameter.find('=') {
                let name = parameter[..separator].trim().to_lowercase();
                let raw_val = parameter[separator + 1..].trim();
                let value = if raw_val.starts_with('"') && raw_val.ends_with('"') && raw_val.len() >= 2 {
                    &raw_val[1..raw_val.len() - 1]
                } else if raw_val.starts_with('"') {
                    &raw_val[1..]
                } else if raw_val.ends_with('"') {
                    &raw_val[..raw_val.len() - 1]
                } else {
                    raw_val
                };
                params.push((name, value.to_string()));
            }
        }
        start = index + 1;
    }
    params
}

fn percent_decode_utf8(encoded: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let s_bytes = encoded.as_bytes();
    let mut i = 0;
    while i < s_bytes.len() {
        if s_bytes[i] == b'%' {
            if i + 2 >= s_bytes.len() {
                return None;
            }
            let h1 = (s_bytes[i + 1] as char).to_digit(16)?;
            let h2 = (s_bytes[i + 2] as char).to_digit(16)?;
            let b = ((h1 << 4) | h2) as u8;
            bytes.push(b);
            i += 3;
        } else {
            bytes.push(s_bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(bytes).ok()
}

fn percent_decode_iso8859_1(encoded: &str) -> Option<String> {
    let mut result = String::new();
    let bytes = encoded.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return None;
            }
            let h1 = (bytes[i + 1] as char).to_digit(16)?;
            let h2 = (bytes[i + 2] as char).to_digit(16)?;
            let b = (h1 << 4 | h2) as u8;
            result.push(b as char);
            i += 3;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    Some(result)
}

pub fn decode_extended_remote_file_name(value: &str) -> Option<String> {
    let first_tick = value.find('\'')?;
    let rest = &value[first_tick + 1..];
    let second_tick = rest.find('\'')?;
    let charset = value[..first_tick].to_lowercase();
    let encoded = &rest[second_tick + 1..];

    if charset == "utf-8" {
        percent_decode_utf8(encoded).map(|s| s.replace(['\\', '/'], "_"))
    } else if charset == "iso-8859-1" {
        percent_decode_iso8859_1(encoded).map(|s| s.replace(['\\', '/'], "_"))
    } else {
        None
    }
}

pub fn decode_quoted_remote_file_name(value: &str) -> String {
    let windows_drive_path = {
        let b = value.as_bytes();
        b.len() >= 3
            && b[0].is_ascii_alphabetic()
            && b[1] == b':'
            && (b[2] == b'\\' || b[2] == b'/')
    };
    let windows_network_path = value.starts_with(r"\\");
    let mixed_windows_path = value.contains('/') && value.contains('\\');
    let relative_windows_path = value.contains('\\')
        && !value.starts_with('/')
        && !value.starts_with('\\')
        && !value.starts_with(':');

    if !windows_drive_path && !windows_network_path && !mixed_windows_path && !relative_windows_path {
        unescape_slashes(value)
    } else if let Some(last_forward) = value.rfind('/') {
        let prefix = &value[..=last_forward];
        let rest = &value[last_forward + 1..];
        let file_name = unescape_non_alphanumeric_after_slash(rest);
        format!("{prefix}{file_name}")
    } else {
        let first_bs = value.find('\\');
        let last_bs = value.rfind('\\');
        if !windows_drive_path && !windows_network_path && first_bs == last_bs {
            let after_bs = first_bs.and_then(|idx| value[idx + 1..].chars().next());
            if after_bs.map(|c| !c.is_alphanumeric()).unwrap_or(false) {
                unescape_slashes(value)
            } else {
                value.replace(r#"\""#, r#"""#)
            }
        } else {
            value.replace(r#"\""#, r#"""#)
        }
    }
}

fn unescape_non_alphanumeric_after_slash(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                if !next.is_alphanumeric() {
                    chars.next();
                    out.push(next);
                } else {
                    out.push('\\');
                }
            } else {
                out.push('\\');
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn unescape_slashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn parse_content_disposition_file_name(header: Option<&str>) -> Option<String> {
    let header = header?.trim();
    if header.is_empty() {
        return None;
    }
    let mut fallback_file_name = None;
    for (name, val) in parse_content_disposition_parameters(header) {
        if name == "filename" {
            if fallback_file_name.is_none() {
                let decoded = decode_quoted_remote_file_name(&val);
                let base = basename_from_any_path(&decoded);
                if !base.is_empty() {
                    fallback_file_name = Some(base);
                }
            }
            continue;
        }
        if name == "filename*" {
            if let Some(decoded) = decode_extended_remote_file_name(&val) {
                let base = basename_from_any_path(&decoded);
                if !base.is_empty() {
                    return Some(base);
                }
            }
        }
    }
    fallback_file_name
}

pub fn resolve_remote_file_name(
    header_cd: Option<&str>,
    final_url: &str,
    file_path_hint: Option<&str>,
) -> Option<String> {
    if let Some(name) = parse_content_disposition_file_name(header_cd) {
        if !name.is_empty() {
            return Some(name);
        }
    }
    if let Some(hint) = file_path_hint {
        let base = basename_from_any_path(hint);
        if !base.is_empty() {
            return Some(base);
        }
    }
    if let Ok(parsed) = url::Url::parse(final_url) {
        let path = parsed.path();
        let base = basename_from_any_path(path);
        if !base.is_empty() {
            match percent_decode_str(&base).decode_utf8() {
                Ok(decoded) => {
                    let replaced = decoded.replace(['\\', '/'], "_");
                    if !replaced.is_empty() {
                        return Some(replaced);
                    }
                }
                Err(_) => {
                    return Some(base);
                }
            }
        }
    }
    None
}

pub fn is_generic_response_content_type(content_type: Option<&str>) -> bool {
    let ct = match content_type {
        Some(s) => s.split(';').next().unwrap_or("").trim().to_lowercase(),
        None => return true,
    };
    ct.is_empty()
        || ct == "application/octet-stream"
        || ct == "binary/octet-stream"
        || ct == "application/zip"
}

pub fn resolve_response_content_type(
    header_content_type: Option<&str>,
    fallback_content_type: Option<&str>,
) -> Option<String> {
    let fallback = match fallback_content_type {
        Some(fb) => fb,
        None => return header_content_type.map(ToString::to_string),
    };
    if is_generic_response_content_type(header_content_type) {
        return Some(fallback.to_string());
    }
    let header_main = header_content_type
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let fallback_main = fallback.split(';').next().unwrap_or("").trim().to_lowercase();

    if header_main.starts_with("video/")
        && fallback_main.starts_with("audio/")
        && header_main["video/".len()..] == fallback_main["audio/".len()..]
    {
        return Some(fallback.to_string());
    }
    header_content_type
        .map(ToString::to_string)
        .or_else(|| Some(fallback.to_string()))
}

pub fn retain_charset_parameter(stored_type: &str, header_type: &str) -> String {
    if let Some(charset_pos) = header_type.to_lowercase().find("charset=") {
        let charset_part = &header_type[charset_pos + "charset=".len()..];
        let charset_val = charset_part
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        if !charset_val.is_empty() && !stored_type.contains("charset=") {
            return format!("{stored_type};charset={charset_val}");
        }
    }
    stored_type.to_string()
}

pub fn parse_media_content_length(header: Option<&str>) -> Result<Option<usize>, String> {
    let raw = match header {
        Some(h) => h.trim(),
        None => return Ok(None),
    };
    if raw.is_empty() {
        return Ok(None);
    }
    raw.parse::<usize>()
        .map(Some)
        .map_err(|_| format!("invalid content-length header: {raw}"))
}

pub fn check_ssrf_allowed(
    url_str: &str,
    allowed_origins: Option<&[String]>,
    allowed_hostnames: Option<&[String]>,
    _allow_rfc2544: bool,
) -> Result<(), MediaFetchError> {
    let parsed = url::Url::parse(url_str).map_err(|e| {
        MediaFetchError::fetch_failed(url_str, format!("Invalid URL: {e}"))
    })?;

    if let Some(origins) = allowed_origins {
        let origin = parsed.origin().ascii_serialization();
        if origins.iter().any(|o| o == &origin || o == url_str) {
            return Ok(());
        }
    }

    if let Some(host) = parsed.host_str() {
        if let Some(hostnames) = allowed_hostnames {
            if hostnames.iter().any(|h| h.eq_ignore_ascii_case(host)) {
                return Ok(());
            }
        }
        // Block localhost / 127.0.0.1 / private IPs by default
        if host == "127.0.0.1" || host == "localhost" || host == "::1" {
            return Err(MediaFetchError::fetch_failed(
                url_str,
                "Blocked hostname or private/internal/special-use IP address",
            ));
        }
    }
    Ok(())
}

fn normalize_fetch_hostname(value: &str) -> String {
    let lower = value.trim().to_lowercase();
    let normalized = lower.trim_end_matches('.');
    normalized
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(normalized)
        .to_string()
}

/// Applies the reader's destination policy before initial and redirected requests.
/// A matching restrictive list never substitutes for existing private-host checks.
pub fn check_media_fetch_url(
    url_str: &str,
    options: &FetchMediaOptions,
) -> Result<(), MediaFetchError> {
    let parsed = url::Url::parse(url_str)
        .map_err(|error| MediaFetchError::fetch_failed(url_str, format!("Invalid URL: {error}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(MediaFetchError::fetch_failed(url_str, "HTTP or HTTPS required"));
    }
    if options.require_https.unwrap_or(false) && parsed.scheme() != "https" {
        return Err(MediaFetchError::fetch_failed(url_str, "HTTPS required"));
    }
    let hostname = parsed.host_str().map(normalize_fetch_hostname)
        .filter(|hostname| !hostname.is_empty())
        .ok_or_else(|| MediaFetchError::fetch_failed(url_str, "Invalid hostname"))?;
    let patterns: HashSet<String> = options.hostname_allowlist.as_deref().unwrap_or_default()
        .iter().map(|entry| normalize_fetch_hostname(entry).trim().to_string())
        .filter(|pattern| !pattern.is_empty() && pattern != "*" && pattern != "*.")
        .collect();
    let allowed = patterns.is_empty() || patterns.iter().any(|pattern| {
        if let Some(suffix) = pattern.strip_prefix("*.") {
            !suffix.is_empty() && hostname != suffix && hostname.ends_with(&format!(".{suffix}"))
        } else {
            hostname == *pattern
        }
    });
    if !allowed {
        return Err(MediaFetchError::fetch_failed(
            url_str,
            format!("Domain policy: Blocked hostname (not in allowlist): {hostname}"),
        ));
    }
    check_ssrf_allowed(
        url_str,
        options.allowed_origins.as_deref(),
        options.allowed_hostnames.as_deref(),
        options.allow_rfc2544_benchmark_range.unwrap_or(false),
    )
}

fn media_request_error(source_url: &str, error: reqwest::Error) -> MediaFetchError {
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        if let Some(policy_error) = cause.downcast_ref::<MediaFetchError>() {
            return policy_error.clone();
        }
        source = cause.source();
    }
    MediaFetchError::fetch_failed(source_url, error.to_string())
}

/// Fetches media through SSRF guards and returns the bounded response body as a buffer.
pub fn read_remote_media_buffer_with_headers(
    options: &FetchMediaOptions,
) -> Result<FetchMediaResultWithHeaders, MediaFetchError> {
    let effective_max_bytes = options.max_bytes.unwrap_or(MAX_DOCUMENT_BYTES);
    let source_url = &options.url;

    check_media_fetch_url(source_url, options)?;

    let redirect_options = options.clone();
    let max_redirects = options.max_redirects.unwrap_or(5);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(options.timeout_ms.unwrap_or(30_000)))
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() > max_redirects {
                return attempt.error(MediaFetchError::fetch_failed(
                    &redirect_options.url,
                    format!("Too many redirects (limit: {max_redirects})"),
                ));
            }
            match check_media_fetch_url(attempt.url().as_str(), &redirect_options) {
                Ok(()) => attempt.follow(),
                Err(error) => attempt.error(error),
            }
        }))
        .build()
        .map_err(|e| MediaFetchError::fetch_failed(source_url, e.to_string()))?;

    let mut res = client
        .get(source_url)
        .send()
        .map_err(|error| media_request_error(source_url, error))?;

    let status = res.status().as_u16();
    let final_url = res.url().as_str().to_string();

    if !res.status().is_success() {
        return Err(read_http_error_response(&mut res, source_url, status));
    }

    let cl_header = res
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok());
    match parse_media_content_length(cl_header) {
        Ok(Some(cl)) => {
            if cl > effective_max_bytes {
                return Err(MediaFetchError::max_bytes(
                    source_url,
                    format!("content length {cl} exceeds maxBytes {effective_max_bytes}"),
                ));
            }
        }
        Ok(None) => {}
        Err(err_msg) => {
            return Err(MediaFetchError::http_error_message(
                source_url,
                err_msg,
                Some(status),
            ));
        }
    }

    if status == 204 {
        return Err(MediaFetchError::http_error_message(
            source_url,
            format!("HTTP 204; empty response body"),
            Some(204),
        ));
    }

    let header_content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string);
    let header_cd = res
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string);

    let bytes = read_bounded_response(&mut res, effective_max_bytes)
        .map_err(|e| MediaFetchError::fetch_failed(source_url, e.to_string()))?;

    if bytes.len() > effective_max_bytes {
        return Err(MediaFetchError::max_bytes(
            source_url,
            format!("payload exceeds maxBytes {effective_max_bytes}"),
        ));
    }

    if bytes.is_empty() {
        return Err(MediaFetchError::http_error_message(
            source_url,
            format!("HTTP {status}; empty response body"),
            Some(status),
        ));
    }

    let mut file_name = resolve_remote_file_name(
        header_cd.as_deref(),
        &final_url,
        options.file_path_hint.as_deref(),
    );

    let file_name_ext = file_name.as_deref().map(extname_from_any_path);
    let has_ext = file_name_ext.as_ref().map(|e| !e.is_empty()).unwrap_or(false);

    let content_type = if let Some(ref hct) = header_content_type {
        if hct == "__proto__" {
            Some("__proto__".to_string())
        } else {
            detect_mime_from_buffer(&bytes, file_name.as_deref()).or_else(|| Some(hct.clone()))
        }
    } else {
        detect_mime_from_buffer(&bytes, file_name.as_deref())
    };

    if let (Some(name), false, Some(ct)) = (&file_name, has_ext, &content_type) {
        if ct != "__proto__" {
            let ext = resolve_saved_media_extension(Some(ct.as_str()), None, None, None);
            if !ext.is_empty() && !ext.eq_ignore_ascii_case("bin") {
                file_name = Some(format!("{name}.{ext}"));
            }
        }
    }

    Ok(FetchMediaResultWithHeaders {
        buffer: bytes.to_vec(),
        content_type,
        header_content_type,
        file_name,
    })
}

/// Preserves the public buffer result.
pub fn read_remote_media_buffer(
    options: &FetchMediaOptions,
) -> Result<FetchMediaResult, MediaFetchError> {
    let res = read_remote_media_buffer_with_headers(options)?;
    Ok(FetchMediaResult {
        buffer: res.buffer,
        content_type: res.content_type,
        file_name: res.file_name,
    })
}

/// Validates and saves caller-provided response bytes into the media store.
pub fn save_response_media(
    bytes: &[u8],
    headers: &BTreeMap<String, String>,
    status: u16,
    options: &SaveResponseMediaOptions,
) -> Result<SavedRemoteMedia, MediaFetchError> {
    let effective_max_bytes = options.max_bytes.unwrap_or(MAX_DOCUMENT_BYTES);
    let source_url = options.source_url.as_deref().unwrap_or("response");

    if status < 200 || status >= 300 {
        return Err(http_error_from_body(source_url, status, bytes));
    }

    let cl_header = headers.get("content-length").map(String::as_str);
    match parse_media_content_length(cl_header) {
        Ok(Some(cl)) => {
            if cl > effective_max_bytes {
                return Err(MediaFetchError::max_bytes(
                    source_url,
                    format!("content length {cl} exceeds maxBytes {effective_max_bytes}"),
                ));
            }
        }
        Ok(None) => {}
        Err(err_msg) => {
            return Err(MediaFetchError::http_error_message(
                source_url,
                err_msg,
                Some(status),
            ));
        }
    }

    if status == 204 {
        return Err(MediaFetchError::http_error_message(
            source_url,
            format!("HTTP 204; empty response body"),
            Some(204),
        ));
    }

    if bytes.is_empty() {
        return Err(MediaFetchError::http_error_message(
            source_url,
            format!("HTTP {status}; empty response body"),
            Some(status),
        ));
    }

    if bytes.len() > effective_max_bytes {
        return Err(MediaFetchError::max_bytes(
            source_url,
            format!("payload exceeds maxBytes {effective_max_bytes}"),
        ));
    }

    let header_cd = headers.get("content-disposition").map(String::as_str);
    let header_ct = headers.get("content-type").map(String::as_str);

    let file_name = resolve_remote_file_name(
        header_cd,
        source_url,
        options.file_path_hint.as_deref().or(options.original_filename.as_deref()),
    );

    let content_type = resolve_response_content_type(header_ct, options.fallback_content_type.as_deref());

    let subdir = options.subdir.as_deref().unwrap_or("inbound");
    let original_fn = options
        .original_filename
        .as_deref()
        .or(file_name.as_deref());

    let saved = crate::extensions::attachments::media_store::save_media_buffer(
        bytes,
        content_type.as_deref(),
        subdir,
        effective_max_bytes as u64,
        original_fn,
        options.file_path_hint.as_deref().or(file_name.as_deref()),
        options.base_dir.as_deref(),
    )
    .map_err(|e| match e.code {
        SaveMediaSourceErrorCode::TooLarge => MediaFetchError::max_bytes(
            source_url,
            format!("payload exceeds maxBytes {effective_max_bytes}"),
        ),
        _ => MediaFetchError::fetch_failed(source_url, e.message),
    })?;

    let final_ct = if let (Some(stored), Some(hct)) = (&saved.content_type, header_ct) {
        Some(retain_charset_parameter(stored, hct))
    } else {
        saved.content_type
    };

    Ok(SavedRemoteMedia {
        id: saved.id,
        path: saved.path,
        size: saved.size,
        content_type: final_ct,
        file_name,
    })
}

/// Fetches media through SSRF guards and saves the body into the media store.
pub fn save_remote_media(
    options: &SaveRemoteMediaOptions,
) -> Result<SavedRemoteMedia, MediaFetchError> {
    let source_url = &options.url;

    check_ssrf_allowed(
        source_url,
        options.allowed_origins.as_deref(),
        options.allowed_hostnames.as_deref(),
        false,
    )?;

    if options.require_https.unwrap_or(false) && !source_url.starts_with("https://") {
        return Err(MediaFetchError::fetch_failed(source_url, "HTTPS required"));
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(options.timeout_ms.unwrap_or(30_000)))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| MediaFetchError::fetch_failed(source_url, e.to_string()))?;

    let mut res = client
        .get(source_url)
        .send()
        .map_err(|e| MediaFetchError::fetch_failed(source_url, e.to_string()))?;

    let status = res.status().as_u16();
    let final_url = res.url().as_str().to_string();

    if !res.status().is_success() {
        return Err(read_http_error_response(&mut res, &final_url, status));
    }

    let mut headers = BTreeMap::new();
    for (k, v) in res.headers() {
        if let Ok(str_val) = v.to_str() {
            headers.insert(k.as_str().to_lowercase(), str_val.to_string());
        }
    }

    let effective_max_bytes = options.max_bytes.unwrap_or(MAX_DOCUMENT_BYTES);
    let bytes = read_bounded_response(&mut res, effective_max_bytes)
        .map_err(|e| MediaFetchError::fetch_failed(source_url, e.to_string()))?;

    let save_opts = SaveResponseMediaOptions {
        source_url: Some(final_url),
        file_path_hint: options.file_path_hint.clone(),
        max_bytes: options.max_bytes,
        read_idle_timeout_ms: options.read_idle_timeout_ms,
        fallback_content_type: options.fallback_content_type.clone(),
        subdir: options.subdir.clone(),
        original_filename: options.original_filename.clone(),
        base_dir: options.base_dir.clone(),
    };

    save_response_media(&bytes, &headers, status, &save_opts)
}
