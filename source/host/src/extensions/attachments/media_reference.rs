use std::cell::Cell;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use regex::Regex;
use std::sync::LazyLock;
use thiserror::Error;
use url::{SyntaxViolation, Url};

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum MediaReferenceError {
    #[error("invalid-path: {0}")]
    InvalidPath(String),
    #[error("path-not-allowed: {0}")]
    PathNotAllowed(String),
}

/// Reject unsafe local path spellings before a reader performs any filesystem work.
/// This is admission only, not a root-capability or filesystem-identity check.
pub fn admit_local_media_path(path: &Path) -> Result<(), MediaReferenceError> {
    let bytes = path.as_os_str().as_encoded_bytes();
    if bytes.contains(&0) {
        return Err(MediaReferenceError::InvalidPath(
            "Local media paths cannot contain NUL".into(),
        ));
    }

    #[cfg(windows)]
    {
        let is_separator = |index: usize| {
            matches!(bytes.get(index).copied(), Some(b'/') | Some(b'\\'))
        };
        let has_drive = |index: usize| {
            bytes.get(index).is_some_and(|byte| byte.is_ascii_alphabetic())
                && bytes.get(index + 1) == Some(&b':')
        };
        let namespace = if is_separator(0)
            && is_separator(1)
            && is_separator(3)
            && matches!(bytes.get(2).copied(), Some(b'?') | Some(b'.'))
        {
            bytes.get(2).copied()
        } else {
            None
        };
        let drive_colon = if has_drive(0) && is_separator(2) {
            Some(1)
        } else if namespace.is_some() && has_drive(4) && is_separator(6) {
            Some(5)
        } else {
            None
        };
        if is_separator(0)
            && is_separator(1)
            && !(namespace == Some(b'?') && drive_colon == Some(5))
        {
            return Err(MediaReferenceError::PathNotAllowed(
                "Local media paths cannot use Windows network or device paths".into(),
            ));
        }
        if let Some(colon) = bytes.iter().position(|byte| *byte == b':') {
            if Some(colon) != drive_colon || bytes[colon + 1..].contains(&b':') {
                return Err(MediaReferenceError::PathNotAllowed(
                    "Local media paths cannot use Windows filesystem namespace aliases".into(),
                ));
            }
        }
    }
    Ok(())
}

fn has_file_url_scheme(raw: &str) -> bool {
    // Match the URL parser's C0/space trim and embedded ASCII tab/newline removal.
    // This is only the classification of parse failures, never a path parser.
    let mut prefix = raw
        .trim_matches(|character: char| character <= '\u{0020}')
        .chars()
        .filter(|character| !matches!(*character, '\t' | '\r' | '\n'));
    "file:".chars().all(|expected| {
        prefix.next().is_some_and(|actual| actual.eq_ignore_ascii_case(&expected))
    })
}

/// Distinguish non-file sources, admitted local file URLs, and rejected file URLs.
/// Conversion failures never become manually reconstructed paths or passthrough URLs.
pub fn resolve_local_file_url(raw: &str) -> Result<Option<PathBuf>, MediaReferenceError> {
    let removed_remote_host = Cell::new(false);
    let record_violation = |violation| {
        if matches!(violation, SyntaxViolation::FileWithHostAndWindowsDrive) {
            removed_remote_host.set(true);
        }
    };
    let parsed = match Url::options()
        .syntax_violation_callback(Some(&record_violation))
        .parse(raw)
    {
        Ok(parsed) if parsed.scheme() == "file" => parsed,
        Ok(_) => return Ok(None),
        Err(error) if has_file_url_scheme(raw) => {
            return Err(MediaReferenceError::InvalidPath(format!(
                "Invalid local file URL: {error}",
            )));
        }
        Err(_) => return Ok(None),
    };
    // url 2.5.8 can erase a remote host when the path starts with a drive letter.
    // Its syntax callback retains that refusal even after host normalization.
    if removed_remote_host.get()
        || parsed.host_str().is_some_and(|host| !host.is_empty() && host != "localhost")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
    {
        return Err(MediaReferenceError::PathNotAllowed(
            "Local file URLs must not contain a remote authority, userinfo, or port".into(),
        ));
    }

    #[cfg(windows)]
    let parsed = {
        let pathname = parsed.path();
        let bytes = pathname.as_bytes();
        // url 2.5.8 can leave a literal pipe in the first drive segment after
        // parsing an empty/localhost authority. Normalize only that rooted
        // drive delimiter, after authority admission, through the URL owner.
        // Encoded pipes and bare drives still fail the existing checks below.
        if bytes.first() == Some(&b'/')
            && bytes.get(1).is_some_and(|byte| byte.is_ascii_alphabetic())
            && bytes.get(2) == Some(&b'|')
            && bytes.get(3) == Some(&b'/')
        {
            let mut pathname = pathname.to_owned();
            pathname.replace_range(2..3, ":");
            let mut parsed = parsed;
            parsed.set_path(&pathname);
            parsed
        } else {
            parsed
        }
    };

    let pathname = parsed.path();
    let bytes = pathname.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }
        let hex = |byte: u8| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        };
        let octet = bytes.get(index + 1).copied().and_then(hex).zip(
            bytes.get(index + 2).copied().and_then(hex),
        ).map(|(high, low)| high * 16 + low);
        let Some(octet) = octet else {
            return Err(MediaReferenceError::InvalidPath(
                "Local file URL pathname contains an invalid percent escape".into(),
            ));
        };
        if matches!(octet, b'/' | b'\\') {
            return Err(MediaReferenceError::InvalidPath(
                "Local file URL pathname cannot encode path separators".into(),
            ));
        }
        index += 3;
    }
    let decoded = percent_encoding::percent_decode_str(pathname).decode_utf8().map_err(|_| {
        MediaReferenceError::InvalidPath("Local file URL pathname must be valid UTF-8".into())
    })?;
    if decoded.contains('\0') {
        return Err(MediaReferenceError::InvalidPath(
            "Local file URL pathname cannot contain NUL".into(),
        ));
    }
    #[cfg(windows)]
    {
        let decoded = decoded.as_bytes();
        // to_file_path's Windows implementation assumes an absolute drive path.
        // Check first so a bare C: cannot reach its debug assertion or the filesystem.
        if decoded.first() != Some(&b'/')
            || !decoded.get(1).is_some_and(|byte| byte.is_ascii_alphabetic())
            || decoded.get(2) != Some(&b':')
            || decoded.get(3) != Some(&b'/')
        {
            return Err(MediaReferenceError::InvalidPath(
                "Local file URL must contain an absolute Windows drive path".into(),
            ));
        }
    }
    let path = parsed.to_file_path().map_err(|_| {
        MediaReferenceError::InvalidPath("Local file URL cannot be converted to a native path".into())
    })?;
    if !path.is_absolute() {
        return Err(MediaReferenceError::InvalidPath(
            "Local file URL must resolve to an absolute path".into(),
        ));
    }
    admit_local_media_path(&path)?;
    Ok(Some(path))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundMediaUri {
    pub id: String,
    pub normalized_source: String,
}

static MEDIA_PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*MEDIA\s*:\s*").expect("valid regex"));
static WIN_DRIVE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z]:[\\/]").expect("valid regex"));
static SCHEME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^[a-z][a-z0-9+.-]*:").expect("valid regex"));

/// Strips legacy MEDIA: prefixes while preserving canonical media:// references.
pub fn normalize_media_reference_source(source: &str) -> String {
    let trimmed = source.trim();
    if trimmed.get(..8).is_some_and(|prefix| prefix.eq_ignore_ascii_case("media://")) {
        return trimmed.to_string();
    }
    MEDIA_PREFIX_RE.replace(trimmed, "").trim().to_string()
}

/// Parses canonical inbound media-store URIs and rejects nested or cross-bucket references.
pub fn parse_inbound_media_uri(source: &str) -> Result<Option<InboundMediaUri>, MediaReferenceError> {
    let normalized = normalize_media_reference_source(source);
    if !normalized.get(..8).is_some_and(|prefix| prefix.eq_ignore_ascii_case("media://")) {
        return Ok(None);
    }

    let parsed = Url::parse(&normalized).map_err(|e| {
        MediaReferenceError::InvalidPath(format!("Invalid media URI: {} ({})", normalized, e))
    })?;

    let host = parsed.host_str().unwrap_or("");
    if host != "inbound" {
        return Err(MediaReferenceError::PathNotAllowed(format!(
            "Unsupported media URI location: {}",
            if host.is_empty() { "(missing)" } else { host }
        )));
    }

    // Check raw path after host to prevent URL parser normalization from masking traversal
    let after_scheme = &normalized[8..];
    if let Some((_h, rest)) = after_scheme.split_once('/') {
        if rest.starts_with("../")
            || rest.contains("/../")
            || rest.ends_with("/..")
            || rest == ".."
            || rest == "."
            || rest.contains('\\')
        {
            return Err(MediaReferenceError::InvalidPath(format!(
                "Invalid media URI: {}",
                normalized
            )));
        }
    }

    if !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(MediaReferenceError::InvalidPath(format!(
            "Invalid media URI: {}",
            normalized
        )));
    }

    let raw_path = parsed.path().trim_start_matches('/');
    let decoded_id = percent_encoding::percent_decode_str(raw_path)
        .decode_utf8()
        .map_err(|e| {
            MediaReferenceError::InvalidPath(format!("Invalid media URI: {} ({})", normalized, e))
        })?
        .to_string();

    let invalid_id = decoded_id.is_empty()
        || decoded_id == "."
        || decoded_id == ".."
        || decoded_id.contains('/')
        || decoded_id.contains('\\')
        || decoded_id.contains('\0');

    if invalid_id {
        return Err(MediaReferenceError::InvalidPath(format!(
            "Invalid media URI: {}",
            normalized
        )));
    }

    Ok(Some(InboundMediaUri {
        id: decoded_id,
        normalized_source: normalized,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaReferenceSourceInfo {
    pub has_unsupported_scheme: bool,
    pub is_data_url: bool,
    pub is_file_url: bool,
    pub is_http_url: bool,
    pub is_media_store_url: bool,
    pub looks_like_windows_drive_path: bool,
}

/// Classifies media reference schemes before local resolution or sandbox rewriting.
pub fn classify_media_reference_source(
    source: &str,
    allow_data_url: bool,
) -> MediaReferenceSourceInfo {
    let looks_like_windows_drive_path = WIN_DRIVE_RE.is_match(source);
    let has_scheme = SCHEME_RE.is_match(source);
    // A local path may place a UTF-8 code point across any ASCII prefix boundary.
    let is_file_url = source.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("file:"));
    let is_http_url = source.get(..7).is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
        || source.get(..8).is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"));
    let is_data_url = source.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"));
    let is_media_store_url = source.get(..8).is_some_and(|prefix| prefix.eq_ignore_ascii_case("media://"));

    let has_unsupported_scheme = has_scheme
        && !looks_like_windows_drive_path
        && !is_file_url
        && !is_http_url
        && !is_media_store_url
        && !(allow_data_url && is_data_url);

    MediaReferenceSourceInfo {
        has_unsupported_scheme,
        is_data_url,
        is_file_url,
        is_http_url,
        is_media_store_url,
        looks_like_windows_drive_path,
    }
}

/// Converts a managed inbound path to a URI without exposing paths outside its store.
pub fn build_inbound_media_uri_from_path(source: &Path, media_dir: &Path) -> Option<String> {
    let inbound_dir = media_dir.join("inbound");
    let norm_source = if source.is_absolute() {
        source.to_path_buf()
    } else {
        return None;
    };

    let rel = match norm_source.strip_prefix(&inbound_dir) {
        Ok(p) => p,
        Err(_) => return None,
    };

    let mut components = rel.components();
    let first = components.next()?;
    if components.next().is_some() {
        // Multi-segment subdirectories are rejected
        return None;
    }

    let segment_name = match first {
        Component::Normal(s) => s.to_str()?,
        _ => return None,
    };

    if segment_name.is_empty()
        || segment_name == "."
        || segment_name == ".."
        || segment_name.contains('/')
        || segment_name.contains('\\')
    {
        return None;
    }

    let candidate = format!("media://inbound/{}", segment_name);
    parse_inbound_media_uri(&candidate).ok()?.map(|u| u.normalized_source)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaSandboxPathRewrite {
    pub resolved: String,
    pub rewritten_from: Option<String>,
}

/// Rewrites inbound media-store URIs to sandbox-relative paths for staged agent inputs.
pub fn resolve_media_reference_sandbox_path(
    source: &str,
    inbound_dir: Option<&str>,
) -> MediaSandboxPathRewrite {
    let normalized = normalize_media_reference_source(source);
    match parse_inbound_media_uri(&normalized) {
        Ok(Some(uri)) => {
            let base = inbound_dir.unwrap_or("media/inbound").replace('\\', "/");
            let clean_base = base.trim_end_matches('/');
            MediaSandboxPathRewrite {
                resolved: format!("{}/{}", clean_base, uri.id),
                rewritten_from: Some(uri.normalized_source),
            }
        }
        _ => MediaSandboxPathRewrite {
            resolved: normalized,
            rewritten_from: None,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundMediaReference {
    pub id: String,
    pub physical_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaReferenceLocalPathInfo {
    Inbound { id: String, path: PathBuf },
    Local { path: PathBuf },
}

/// Resolves inbound media:// URIs or first-level inbound file paths to concrete store files.
pub fn resolve_inbound_media_reference(
    source: &str,
    media_dir: &Path,
) -> Result<Option<InboundMediaReference>, MediaReferenceError> {
    let normalized = normalize_media_reference_source(source);
    if normalized.is_empty() {
        return Ok(None);
    }

    if let Some(uri) = parse_inbound_media_uri(&normalized)? {
        let physical = media_dir.join("inbound").join(&uri.id);
        return Ok(Some(InboundMediaReference {
            id: uri.id,
            physical_path: physical,
        }));
    }

    let path = Path::new(&normalized);
    if path.is_absolute() {
        let inbound_dir = media_dir.join("inbound");
        if let Ok(rel) = path.strip_prefix(&inbound_dir) {
            let mut components = rel.components();
            if let Some(Component::Normal(s)) = components.next() {
                if components.next().is_none() {
                    let id_str = s.to_string_lossy().into_owned();
                    return Ok(Some(InboundMediaReference {
                        id: id_str,
                        physical_path: path.to_path_buf(),
                    }));
                }
            }
        }
    }

    Ok(None)
}

/// Resolves a media reference while preserving whether it belongs to the inbound store.
pub fn resolve_media_reference_local_path_info(
    source: &str,
    media_dir: &Path,
) -> Result<MediaReferenceLocalPathInfo, MediaReferenceError> {
    if let Some(inbound) = resolve_inbound_media_reference(source, media_dir)? {
        Ok(MediaReferenceLocalPathInfo::Inbound {
            id: inbound.id,
            path: inbound.physical_path,
        })
    } else {
        let normalized = normalize_media_reference_source(source);
        Ok(MediaReferenceLocalPathInfo::Local {
            path: PathBuf::from(normalized),
        })
    }
}

const MEDIA_READ_CHUNK_BYTES: usize = 64 * 1024;

/// Reads a fresh ordinary file within the caller's byte limit.
/// Path/root authority remains with the caller; pathname identity is not pinned.
pub fn read_outbound_media_file_bounded(
    file_path: &Path,
    max_bytes: usize,
) -> std::io::Result<Vec<u8>> {
    check_outbound_media_metadata(&fs::metadata(file_path)?, max_bytes)?;
    let mut file = open_outbound_media_file_bounded(file_path, max_bytes)?;
    read_media_bytes_bounded(&mut file, max_bytes, reserve_media_read_capacity)
}

fn check_outbound_media_metadata(meta: &fs::Metadata, max_bytes: usize) -> io::Result<()> {
    if !meta.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "target path is not a file",
        ));
    }
    if meta.len() > u64::try_from(max_bytes).unwrap_or(u64::MAX) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file size ({} bytes) exceeds maximum limit ({} bytes)",
                meta.len(),
                max_bytes
            ),
        ));
    }
    Ok(())
}

fn open_outbound_media_file_bounded(file_path: &Path, max_bytes: usize) -> io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Keep ordinary symlinks working, but do not wait for a replaced FIFO's writer.
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = options.open(file_path)?;
    check_outbound_media_metadata(&file.metadata()?, max_bytes)?;
    Ok(file)
}

fn reserve_media_read_capacity(bytes: &mut Vec<u8>, capacity: usize) -> io::Result<()> {
    bytes
        .try_reserve_exact(capacity - bytes.len())
        // Avoid allocating an error message on the allocation-failure path.
        .map_err(|_| io::Error::from(io::ErrorKind::OutOfMemory))
}

fn read_media_bytes_bounded(
    reader: &mut impl Read,
    max_bytes: usize,
    mut reserve: impl FnMut(&mut Vec<u8>, usize) -> io::Result<()>,
) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut scratch = [0_u8; MEDIA_READ_CHUNK_BYTES];
    loop {
        let remaining = max_bytes - bytes.len();
        // At the limit, inspect one byte without computing max_bytes + 1 or
        // extending the result. A short read alone is never treated as EOF.
        let read_limit = remaining.min(scratch.len()).max(1);
        let read = match reader.read(&mut scratch[..read_limit]) {
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if read == 0 {
            return Ok(bytes);
        }
        if read > remaining {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "file grew beyond maximum limit during read",
            ));
        }
        if read > bytes.capacity() - bytes.len() {
            // The first request is at most one scratch chunk. Later requests
            // grow geometrically, within the cap, only after actual input.
            let required = bytes.len() + read;
            let capacity = required.max(bytes.capacity().saturating_mul(2).min(max_bytes));
            reserve(&mut bytes, capacity)?;
        }
        bytes.extend_from_slice(&scratch[..read]);
    }
}

#[cfg(test)]
mod bounded_read_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::io::{Seek, Write};

    struct ReadDirectory(PathBuf);

    impl ReadDirectory {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "fabushi-bounded-media-{label}-{}",
                uuid::Uuid::new_v4()
            ));
            fs::create_dir(&path).expect("create owned media-read fixture");
            Self(path)
        }
    }

    impl Drop for ReadDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    enum ReadStep {
        Bytes(&'static [u8]),
        Failure(io::ErrorKind),
        Eof,
    }

    struct ScriptedReader {
        steps: VecDeque<ReadStep>,
        requests: Vec<usize>,
        consumed: usize,
    }

    impl ScriptedReader {
        fn new(steps: impl IntoIterator<Item = ReadStep>) -> Self {
            Self {
                steps: steps.into_iter().collect(),
                requests: Vec::new(),
                consumed: 0,
            }
        }
    }

    impl Read for ScriptedReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            assert!(!buffer.is_empty(), "the production loop must request input");
            self.requests.push(buffer.len());
            match self.steps.pop_front().expect("reader exceeded its finite script") {
                ReadStep::Bytes(data) => {
                    let read = data.len().min(buffer.len());
                    buffer[..read].copy_from_slice(&data[..read]);
                    self.consumed += read;
                    if read < data.len() {
                        self.steps.push_front(ReadStep::Bytes(&data[read..]));
                    }
                    Ok(read)
                }
                ReadStep::Failure(kind) => Err(io::Error::new(kind, "scripted media read failure")),
                ReadStep::Eof => Ok(0),
            }
        }
    }

    #[test]
    fn br_loop_short_reads_and_interrupted_preserve_bytes() {
        let mut reader = ScriptedReader::new([
            ReadStep::Failure(io::ErrorKind::Interrupted),
            ReadStep::Bytes(b"a"),
            ReadStep::Failure(io::ErrorKind::Interrupted),
            ReadStep::Bytes(b"bc"),
            ReadStep::Eof,
        ]);
        let bytes = read_media_bytes_bounded(&mut reader, 3, reserve_media_read_capacity)
            .expect("short reads and interruptions must retain all bytes");
        assert_eq!(bytes, b"abc");
        assert_eq!(reader.consumed, 3);
        assert!(reader.steps.is_empty(), "success requires the scripted EOF");
        assert_eq!(reader.requests.last(), Some(&1), "exact-cap reads must probe EOF");
    }

    #[test]
    fn br_loop_zero_and_one_over_probe_without_allocating_extra_bytes() {
        let mut empty = ScriptedReader::new([ReadStep::Eof]);
        let empty_bytes = read_media_bytes_bounded(&mut empty, 0, |_, _| {
            panic!("an empty zero-cap read must not reserve result storage");
        })
        .expect("empty input at zero cap");
        assert!(empty_bytes.is_empty());
        assert_eq!(empty.requests, vec![1]);

        let mut nonempty = ScriptedReader::new([ReadStep::Bytes(b"x")]);
        let error = read_media_bytes_bounded(&mut nonempty, 0, |_, _| {
            panic!("the overflow probe must not allocate result storage");
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(nonempty.consumed, 1);

        let mut over = ScriptedReader::new([ReadStep::Bytes(b"abcdef")]);
        let mut capacities = Vec::new();
        let error = read_media_bytes_bounded(&mut over, 4, |bytes, capacity| {
            capacities.push(capacity);
            reserve_media_read_capacity(bytes, capacity)
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(over.consumed, 5, "only one over-limit byte may be consumed");
        assert_eq!(capacities, vec![4], "the sentinel never enters the result vector");
    }

    #[test]
    fn br_loop_late_read_error_discards_prefix() {
        let mut reader = ScriptedReader::new([
            ReadStep::Bytes(b"prefix"),
            ReadStep::Failure(io::ErrorKind::PermissionDenied),
        ]);
        let error = read_media_bytes_bounded(&mut reader, 20, reserve_media_read_capacity)
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), "scripted media read failure");
        assert_eq!(reader.consumed, 6);
        assert!(reader.steps.is_empty());
    }

    #[test]
    fn br_loop_allocation_failure_discards_prefix() {
        let source = vec![0x5a; MEDIA_READ_CHUNK_BYTES + 1];
        let mut reader = io::Cursor::new(source.as_slice());
        let mut reservations = 0;
        let error = read_media_bytes_bounded(&mut reader, source.len(), |bytes, capacity| {
            reservations += 1;
            if reservations == 2 {
                assert_eq!(bytes.as_slice(), &source[..MEDIA_READ_CHUNK_BYTES]);
                return Err(io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "injected media reservation failure",
                ));
            }
            reserve_media_read_capacity(bytes, capacity)
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::OutOfMemory);
        assert_eq!(error.to_string(), "injected media reservation failure");
        assert_eq!(reservations, 2);
    }

    #[test]
    fn br_loop_growth_is_bounded_and_amortized() {
        let source = vec![0x37; MEDIA_READ_CHUNK_BYTES * 8 + 1];
        let mut reader = io::Cursor::new(source.as_slice());
        let mut capacities = Vec::new();
        let bytes = read_media_bytes_bounded(&mut reader, source.len(), |bytes, capacity| {
            assert!(capacity <= source.len(), "requested capacity exceeds the byte cap");
            capacities.push(capacity);
            reserve_media_read_capacity(bytes, capacity)
        })
        .expect("finite multi-chunk read");
        assert_eq!(bytes, source);
        assert!(capacities[0] <= 64 * 1024);
        assert!(
            capacities.len() <= 5,
            "growth must not reallocate for every one of the nine input chunks: {capacities:?}"
        );
    }

    #[test]
    fn br_initial_allocation_ignores_large_metadata_hint() {
        let directory = ReadDirectory::new("allocation-hint");
        let path = directory.0.join("source.bin");
        let source = fs::File::create(&path).expect("create source");
        source.set_len((MEDIA_READ_CHUNK_BYTES * 8) as u64).expect("set bounded fixture length");
        drop(source);
        let metadata = fs::metadata(&path).expect("source metadata");
        assert!(metadata.len() > (64 * 1024) as u64);
        check_outbound_media_metadata(&metadata, usize::MAX).expect("regular-file preflight");
        let mut opened = open_outbound_media_file_bounded(&path, usize::MAX)
            .expect("opened source admission");

        let mut requested_capacity = None;
        let error = read_media_bytes_bounded(&mut opened, usize::MAX, |bytes, capacity| {
            assert!(bytes.is_empty() && bytes.capacity() == 0);
            requested_capacity = Some(capacity);
            Err(io::Error::from(io::ErrorKind::OutOfMemory))
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::OutOfMemory);
        let requested_capacity = requested_capacity.expect("one observed reservation");
        assert!((1..=64 * 1024).contains(&requested_capacity));
        assert!(
            opened.stream_position().expect("source offset") <= 64 * 1024,
            "allocation refusal must stop after at most one bounded input chunk"
        );
    }

    #[test]
    fn br_opened_admission_rechecks_size_after_preflight() {
        let directory = ReadDirectory::new("opened-size");
        let path = directory.0.join("source.bin");
        fs::write(&path, b"x").expect("write initial file");
        check_outbound_media_metadata(&fs::metadata(&path).unwrap(), 1)
            .expect("initial preflight");
        fs::write(&path, b"xy").expect("grow file after preflight");
        let error = open_outbound_media_file_bounded(&path, 1).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn br_opened_file_growth_is_not_successful_prefix() {
        let directory = ReadDirectory::new("opened-growth");
        let path = directory.0.join("source.bin");
        fs::write(&path, b"abc").expect("write initial file");
        check_outbound_media_metadata(&fs::metadata(&path).unwrap(), 3)
            .expect("initial preflight");
        let mut opened = open_outbound_media_file_bounded(&path, 3).expect("opened admission");
        let mut appender = fs::OpenOptions::new().append(true).open(&path).expect("owned append");
        appender.write_all(b"d").expect("grow after opened metadata admission");
        drop(appender);

        let error = read_media_bytes_bounded(&mut opened, 3, reserve_media_read_capacity)
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(opened.stream_position().unwrap(), 4, "one sentinel byte is enough");
        drop(opened);
        assert_eq!(read_outbound_media_file_bounded(&path, 4).unwrap(), b"abcd");
    }

    #[cfg(unix)]
    #[test]
    fn br_unix_opened_admission_rejects_directory_replacement() {
        let directory = ReadDirectory::new("opened-kind");
        let path = directory.0.join("source");
        fs::write(&path, b"x").expect("write initial file");
        check_outbound_media_metadata(&fs::metadata(&path).unwrap(), 1)
            .expect("regular-file preflight");
        fs::remove_file(&path).expect("remove owned regular leaf");
        fs::create_dir(&path).expect("substitute owned directory");
        let error = open_outbound_media_file_bounded(&path, 1).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[cfg(unix)]
    #[test]
    fn br_unix_open_owns_nonblocking_descriptor() {
        use std::os::fd::AsRawFd;

        let directory = ReadDirectory::new("open-flags");
        let path = directory.0.join("source.bin");
        fs::write(&path, b"x").expect("write source");
        let opened = open_outbound_media_file_bounded(&path, 1).expect("open ordinary file");
        let flags = unsafe { libc::fcntl(opened.as_raw_fd(), libc::F_GETFL) };
        assert!(flags >= 0, "inspect owned descriptor: {}", io::Error::last_os_error());
        assert_ne!(flags & libc::O_NONBLOCK, 0);
    }

    #[cfg(unix)]
    struct OwnedFifoChild {
        child: std::process::Child,
        log_path: PathBuf,
        output_reported: bool,
    }

    #[cfg(unix)]
    impl OwnedFifoChild {
        fn wait_for(
            &mut self,
            timeout: std::time::Duration,
        ) -> io::Result<Option<std::process::ExitStatus>> {
            let deadline = std::time::Instant::now() + timeout;
            loop {
                if let Some(status) = self.child.try_wait()? {
                    return Ok(Some(status));
                }
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Ok(None);
                }
                // Poll only this owned child's completion. The exit status and
                // deadline decide the oracle, not an assumed scheduling sleep.
                std::thread::sleep(remaining.min(std::time::Duration::from_millis(10)));
            }
        }

        fn report_output(&mut self) -> io::Result<()> {
            use base64::Engine;

            let output = fs::read(&self.log_path)?;
            println!(
                "BR_FIFO_CHILD_OUTPUT_BASE64={}",
                base64::engine::general_purpose::STANDARD.encode(output)
            );
            self.output_reported = true;
            Ok(())
        }
    }

    #[cfg(unix)]
    impl Drop for OwnedFifoChild {
        fn drop(&mut self) {
            if !matches!(self.child.try_wait(), Ok(Some(_))) {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
            if std::thread::panicking() && !self.output_reported {
                if let Err(error) = self.report_output() {
                    eprintln!("Failed to retain owned FIFO child output: {error}");
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn br_unix_fifo_replacement_after_preflight_does_not_block() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::OpenOptionsExt;
        use std::process::{Command, Stdio};
        use std::time::Duration;

        const CHILD_PATH_ENV: &str = "FABUSHI_BR_FIFO_PROBE_PATH";
        const CHILD_ACK_ENV: &str = "FABUSHI_BR_FIFO_PROBE_ACK";
        const TEST_NAME: &str = "br_unix_fifo_replacement_after_preflight_does_not_block";
        const ACK: &[u8] = b"opened-nonregular-rejected";

        if let Some(path) = std::env::var_os(CHILD_PATH_ENV) {
            // Re-enter the one selected test, not an ignored/dummy helper.
            let ack = std::env::var_os(CHILD_ACK_ENV).expect("owned child acknowledgment path");
            let error = open_outbound_media_file_bounded(Path::new(&path), 8).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::NotFound);
            let mut receipt = fs::OpenOptions::new().write(true).create_new(true)
                .open(ack).expect("create exclusive child acknowledgment");
            receipt.write_all(ACK).expect("write actual rejection acknowledgment");
            return;
        }

        let directory = ReadDirectory::new("fifo-replacement");
        let path = directory.0.join("source.bin");
        fs::write(&path, b"ordinary").expect("write initial regular file");
        check_outbound_media_metadata(&fs::metadata(&path).unwrap(), 8)
            .expect("prove regular-file preflight before substitution");
        fs::rename(&path, directory.0.join("original.bin")).expect("retain original owned file");
        let native_path = CString::new(path.as_os_str().as_bytes()).expect("fixture path has no NUL");
        let created = unsafe { libc::mkfifo(native_path.as_ptr(), 0o600) };
        assert_eq!(created, 0, "create owned FIFO: {}", io::Error::last_os_error());

        let ack_path = directory.0.join("child-rejection");
        let log_path = directory.0.join("child-output.log");
        let log = fs::File::create(&log_path).expect("create owned child output log");
        let error_log = log.try_clone().expect("capture owned child stderr");
        let module = module_path!().split_once("::").expect("module has crate prefix").1;
        let child = Command::new(std::env::current_exe().expect("current unit-test binary"))
            .arg("--exact")
            .arg(format!("{module}::{TEST_NAME}"))
            .arg("--nocapture")
            .env(CHILD_PATH_ENV, &path)
            .env(CHILD_ACK_ENV, &ack_path)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(error_log))
            .spawn()
            .expect("start the exact owned FIFO probe");
        let mut probe = OwnedFifoChild { child, log_path, output_reported: false };

        let (status, blocked) = match probe.wait_for(Duration::from_secs(5))
            .expect("wait for owned FIFO probe")
        {
            Some(status) => (status, false),
            None => {
                // A blocking-open mutation must be released and reaped before
                // this timeout is reported as failure. A read anchor makes the
                // writer open nonblocking even if the child has not run yet.
                let release_read = fs::OpenOptions::new().read(true)
                    .custom_flags(libc::O_NONBLOCK).open(&path)
                    .expect("open owned FIFO read anchor");
                let release_write = fs::OpenOptions::new().write(true)
                    .custom_flags(libc::O_NONBLOCK).open(&path)
                    .expect("release only the owned FIFO reader");
                let status = match probe.wait_for(Duration::from_secs(5))
                    .expect("wait after releasing owned FIFO")
                {
                    Some(status) => status,
                    None => {
                        let kill = probe.child.kill();
                        let reaped = probe.child.wait();
                        assert!(kill.is_ok(), "terminate exact owned probe: {kill:?}");
                        reaped.expect("reap terminated owned probe")
                    }
                };
                drop(release_write);
                drop(release_read);
                (status, true)
            }
        };
        probe.report_output().expect("retain complete child stdout/stderr as one encoded record");
        assert!(!blocked, "FIFO open exceeded its deadline; owned probe was released/reaped: {status}");
        assert!(status.success(), "owned FIFO probe failed: {status}");
        assert_eq!(
            fs::read(&ack_path).expect("child must prove it executed the real open"),
            ACK,
            "a zero-test child exit is not proof of rejection"
        );
    }
}

