//! Contract test suite for OpenClaw Media Fetch Pipeline (`OCRM-UNIT-MEDIA-012`).
//!
//! Validates:
//! - Content-Disposition RFC 5987 / 6266 extended filenames (UTF-8, ISO-8859-1).
//! - Unquoted apostrophes, Windows paths, and malformed extended fallbacks.
//! - URL path basename extraction with malformed percent escape tolerance and slash substitution.
//! - Content-Length header parsing and maxBytes overflow rejection.
//! - MIME container hint resolution (video/mp4 vs audio/mp4) and HTTP charset retention.
//! - Telegram bot token redaction in URLs and error messages.
//! - Empty body / 204 rejection without disk pollution.
//! - Gateway dispatch integration for `readRemoteMediaBuffer` and `saveRemoteMedia`.
//! - Bounded non-2xx diagnostics through real HTTP responses and Gateway calls.
//! - Restrictive hostname admission, private-origin trust, and per-hop redirect policy.
//! - Real loopback response bodies, byte bounds, and explicit/default redirect caps.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentsBox, AttachmentsService,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
use mahayana_host_runtime::extensions::attachments::media_fetch::{
    check_media_fetch_url, check_ssrf_allowed, is_generic_response_content_type,
    parse_content_disposition_file_name, parse_media_content_length,
    read_remote_media_buffer, redact_sensitive_text, resolve_remote_file_name,
    resolve_response_content_type, retain_charset_parameter,
    save_response_media, FetchMediaOptions, MediaFetchErrorCode, SaveResponseMediaOptions,
};

use serde_json::Value;

const MEDIA_FETCH_SOURCE: &str = include_str!("../src/extensions/attachments/media_fetch.rs");

#[test]
fn remote_fetch_entries_use_bounded_response_reads() {
    assert!(MEDIA_FETCH_SOURCE.contains("fn read_bounded_response("));
    assert!(MEDIA_FETCH_SOURCE.contains("take(max_bytes.saturating_add(1) as u64)"));
    assert!(!MEDIA_FETCH_SOURCE.contains("res.bytes()"));
}

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ocrm-fetch-test-{name}-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct MockAuth;
impl GenerateImageAuth for MockAuth {
    fn get_access_token(&self) -> Result<String, String> {
        Ok("token".into())
    }
    fn get_machine_id(&self) -> Result<String, String> {
        Ok("machine".into())
    }
}

struct MockBox;
impl AttachmentsBox for MockBox {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        Ok("running".into())
    }
    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

struct MediaHttpFixture {
    origin: String,
    release: Option<mpsc::Sender<()>>,
    prefix_sent: mpsc::Receiver<()>,
    accepted: Arc<AtomicBool>,
    suffix_released: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<io::Result<()>>>,
}

impl MediaHttpFixture {
    fn start(status: u16, body: Vec<u8>, declared_length: usize, suffix: Option<Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind media HTTP fixture");
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let (release, released) = mpsc::channel();
        let (prefix_ready, prefix_sent) = mpsc::channel();
        let accepted = Arc::new(AtomicBool::new(false));
        let worker_accepted = Arc::clone(&accepted);
        let suffix_released = Arc::new(AtomicBool::new(false));
        let worker_released = Arc::clone(&suffix_released);
        let worker = thread::spawn(move || {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        match released.try_recv() {
                            Ok(()) | Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
                            Err(mpsc::TryRecvError::Empty) => {}
                        }
                        thread::park_timeout(Duration::from_millis(5));
                    }
                    Err(error) => return Err(error),
                }
            };
            worker_accepted.store(true, Ordering::SeqCst);
            stream.set_nonblocking(false)?;
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
            stream.set_write_timeout(Some(Duration::from_secs(5)))?;
            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];
            while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                let count = stream.read(&mut chunk)?;
                if count == 0 || request.len() + count > 64 * 1024 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "missing request headers"));
                }
                request.extend_from_slice(&chunk[..count]);
            }
            if !request.starts_with(b"GET ") {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "expected media GET"));
            }
            write!(
                stream,
                "HTTP/1.1 {status} Fixture\r\nContent-Length: {declared_length}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n"
            )?;
            stream.write_all(&body)?;
            stream.flush()?;
            let _ = prefix_ready.send(());
            if let Some(suffix) = suffix {
                // No EOF or suffix reaches the client until the test releases
                // this exact server, including assertion-failure cleanup.
                let _ = released.recv();
                worker_released.store(true, Ordering::SeqCst);
                // A bounded reader has already dropped its response, so a
                // closed peer during cleanup is expected.
                let _ = stream.write_all(&suffix);
                let _ = stream.flush();
            }
            Ok(())
        });
        Self {
            origin,
            release: Some(release),
            prefix_sent,
            accepted,
            suffix_released,
            worker: Some(worker),
        }
    }

    fn args(&self, root: &Path) -> serde_json::Value {
        serde_json::json!({
            "url": format!("{}/media", self.origin),
            "allowed_origins": [self.origin.clone()],
            "timeout_ms": 20_000,
            "max_bytes": 50 * 1024 * 1024,
            "base_dir": root,
            "subdir": "inbound"
        })
    }

    fn release(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
        if let Some(worker) = &self.worker {
            worker.thread().unpark();
        }
    }

    fn finish(&mut self) -> io::Result<()> {
        self.release();
        match self.worker.take() {
            Some(worker) => worker.join().unwrap_or_else(|_| {
                Err(io::Error::new(io::ErrorKind::Other, "media HTTP fixture panicked"))
            }),
            None => Ok(()),
        }
    }
}

impl Drop for MediaHttpFixture {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

fn media_gateway_call(method: &str, args: &serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
    AttachmentsService::new(root, Arc::new(MockAuth), Arc::new(MockBox), None)
        .dispatch_gateway(method, args)
        .expect("registered media gateway method")
}

fn media_tree_snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, directory: &Path, snapshot: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if entry.file_type().unwrap().is_dir() {
                snapshot.insert(relative, None);
                visit(root, &path, snapshot);
            } else {
                snapshot.insert(relative, Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut snapshot = BTreeMap::new();
    visit(root, root, &mut snapshot);
    snapshot
}

fn assert_media_gateway_stops_before_eof(method: &'static str) {
    let temp = TempTestDir::new(method);
    fs::write(temp.path.join("keep.bin"), b"existing media stays intact").unwrap();
    let before = media_tree_snapshot(&temp.path);
    let mut prefix = b"  diagnostic\t".to_vec();
    prefix.resize(8191, b' ');
    // The 8192nd byte starts a scalar whose remaining bytes are held back.
    prefix.push(0xf0);
    let suffix = [vec![0x9f, 0x98, 0x80], b"SUFFIX_MUST_NOT_BE_READ".to_vec()].concat();
    let mut fixture = MediaHttpFixture::start(503, prefix, 8192 + suffix.len(), Some(suffix));
    let args = fixture.args(&temp.path);
    let expected = format!(
        "MediaFetchError: Failed to fetch media from {}/media: HTTP 503; body: diagnostic…",
        fixture.origin
    );
    let root = temp.path.clone();
    let (result_tx, result_rx) = mpsc::channel();
    let caller = thread::spawn(move || {
        let _ = result_tx.send(media_gateway_call(method, &args, &root));
    });

    let prefix_sent = fixture.prefix_sent.recv_timeout(Duration::from_secs(5));
    let result_before_release = result_rx.recv_timeout(Duration::from_secs(5));
    let suffix_already_released = fixture.suffix_released.load(Ordering::SeqCst);
    // Release and join both exact owned threads before any assertion can unwind.
    fixture.release();
    let caller_joined = caller.join();
    let server_joined = fixture.finish();

    assert!(prefix_sent.is_ok(), "server must deliver the complete diagnostic prefix");
    assert!(!suffix_already_released, "the suffix gate must still be closed at settlement");
    caller_joined.expect("media gateway caller did not panic");
    server_joined.expect("media HTTP fixture completed");
    assert_eq!(
        result_before_release.expect("gateway must settle while suffix and EOF remain held").unwrap_err(),
        expected
    );
    assert_eq!(media_tree_snapshot(&temp.path), before);
}

#[test]
fn test_ocrm_media_fetch_content_disposition_rfc5987_and_fallbacks() {
    // 1. Unquoted filename retains apostrophes
    assert_eq!(
        parse_content_disposition_file_name(Some("attachment; filename='report.csv")),
        Some("'report.csv".to_string())
    );
    assert_eq!(
        parse_content_disposition_file_name(Some("attachment; filename=report.csv'")),
        Some("report.csv'".to_string())
    );
    assert_eq!(
        parse_content_disposition_file_name(Some("attachment; filename='report.csv'")),
        Some("'report.csv'".to_string())
    );

    // 2. Unrelated quoted parameter containing filename is ignored
    assert_eq!(
        parse_content_disposition_file_name(Some(
            r#"attachment; note="x; filename=spoof.csv; y"; filename=safe.csv"#
        )),
        Some("safe.csv".to_string())
    );

    // 3. Mixed Windows path separators preserve final basename
    assert_eq!(
        parse_content_disposition_file_name(Some(r#"attachment; filename="C:/tmp/reports\Q1.csv""#)),
        Some("Q1.csv".to_string())
    );

    // 4. Legacy relative Windows path reduced to basename
    assert_eq!(
        parse_content_disposition_file_name(Some(r#"attachment; filename="reports\Q1.csv""#)),
        Some("Q1.csv".to_string())
    );

    // 5. ISO-8859-1 extended filename
    assert_eq!(
        parse_content_disposition_file_name(Some("attachment; filename*=ISO-8859-1''caf%E9.csv")),
        Some("café.csv".to_string())
    );

    // 6. Malformed extended filename falls back to plain filename
    assert_eq!(
        parse_content_disposition_file_name(Some(
            "attachment; filename=fallback.csv; filename*=UTF-8''%ZZbad.csv"
        )),
        Some("fallback.csv".to_string())
    );

    // 7. Unsupported extended charset (UTF-16) falls back to plain filename
    assert_eq!(
        parse_content_disposition_file_name(Some(
            "attachment; filename*=UTF-16''bad.csv; filename=fallback.csv"
        )),
        Some("fallback.csv".to_string())
    );

    // 8. Extended UTF-8 with decoded separators converted to underscores
    assert_eq!(
        parse_content_disposition_file_name(Some(
            "attachment; filename*=UTF-8''reports%2F%2FQ1.pdf"
        )),
        Some("reports__Q1.pdf".to_string())
    );
}

#[test]
fn test_ocrm_media_fetch_url_basename_and_malformed_percent() {
    // 1. Malformed percent escapes keep raw URL path basename
    let url1 = "https://example.com/files/bad%E0%A4%A.pdf";
    let name1 = resolve_remote_file_name(None, url1, None);
    assert_eq!(name1, Some("bad%E0%A4%A.pdf".to_string()));

    // 2. Decoded URL fallback separators (%5C) replaced with underscores inside basename
    let url2 = "https://example.com/files/reports%5CQ1.pdf";
    let name2 = resolve_remote_file_name(None, url2, None);
    assert_eq!(name2, Some("reports_Q1.pdf".to_string()));

    // 3. File path hint takes precedence over URL
    let url3 = "https://example.com/download?id=123";
    let name3 = resolve_remote_file_name(None, url3, Some("/path/to/custom_file.bin"));
    assert_eq!(name3, Some("custom_file.bin".to_string()));
}

#[test]
fn test_ocrm_media_fetch_content_length_and_max_bytes() {
    // 1. Valid content length
    assert_eq!(parse_media_content_length(Some("1024")), Ok(Some(1024)));
    assert_eq!(parse_media_content_length(None), Ok(None));
    assert_eq!(parse_media_content_length(Some("")), Ok(None));

    // 2. Malformed content length rejects before reads
    let err = parse_media_content_length(Some("1e9")).unwrap_err();
    assert!(err.contains("invalid content-length header: 1e9"));

    // 3. Save response media checks content-length against maxBytes
    let mut headers = BTreeMap::new();
    headers.insert("content-length".to_string(), "5".to_string());
    headers.insert("content-type".to_string(), "application/octet-stream".to_string());

    let opts = SaveResponseMediaOptions {
        source_url: Some("https://example.com/file.bin".to_string()),
        max_bytes: Some(4),
        ..Default::default()
    };
    let res = save_response_media(&[1, 2, 3, 4, 5], &headers, 200, &opts);
    assert!(res.is_err());
    let fetch_err = res.unwrap_err();
    assert_eq!(fetch_err.code, MediaFetchErrorCode::MaxBytes);
    assert!(fetch_err.message.contains("content length 5 exceeds maxBytes 4"));

    // 4. Streamed payload exceeds maxBytes
    let mut headers2 = BTreeMap::new();
    headers2.insert("content-type".to_string(), "application/octet-stream".to_string());
    let opts2 = SaveResponseMediaOptions {
        source_url: Some("https://example.com/large.bin".to_string()),
        max_bytes: Some(4),
        ..Default::default()
    };
    let res2 = save_response_media(&[1, 2, 3, 4, 5], &headers2, 200, &opts2);
    assert!(res2.is_err());
    let fetch_err2 = res2.unwrap_err();
    assert_eq!(fetch_err2.code, MediaFetchErrorCode::MaxBytes);
    assert!(fetch_err2.message.contains("payload exceeds maxBytes 4"));
}

#[test]
fn test_ocrm_media_fetch_content_type_container_hint_and_charset() {
    // 1. Generic content types
    assert!(is_generic_response_content_type(Some("application/octet-stream")));
    assert!(is_generic_response_content_type(Some("binary/octet-stream")));
    assert!(is_generic_response_content_type(Some("application/zip")));
    assert!(is_generic_response_content_type(None));
    assert!(!is_generic_response_content_type(Some("image/png")));

    // 2. Fallback used for generic type
    let resolved = resolve_response_content_type(
        Some("application/octet-stream"),
        Some("text/csv"),
    );
    assert_eq!(resolved, Some("text/csv".to_string()));

    // 3. Audio fallback preserved when header reports matching video container
    let container_resolved = resolve_response_content_type(
        Some("video/mp4"),
        Some("audio/mp4"),
    );
    assert_eq!(container_resolved, Some("audio/mp4".to_string()));

    // 4. HTTP charset retention
    let stored = "application/x-openclaw-charset-proof";
    let header = r#"application/x-openclaw-charset-proof; charset="cp1251""#;
    let retained = retain_charset_parameter(stored, header);
    assert_eq!(retained, "application/x-openclaw-charset-proof;charset=cp1251");

    let stored_utf8 = "text/plain";
    let header_utf8 = "text/plain; charset=gbk";
    assert_eq!(retain_charset_parameter(stored_utf8, header_utf8), "text/plain;charset=gbk");
}

#[test]
fn test_ocrm_media_fetch_ssrf_and_redaction() {
    // 1. Redact bot token from URL and errors
    let bot_token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcd";
    let raw_url = format!("https://files.example.test/file/bot{bot_token}/photos/1.jpg");
    let redacted = redact_sensitive_text(&raw_url);
    assert!(!redacted.contains(bot_token));
    assert_eq!(
        redacted,
        "https://files.example.test/file/bot123456…abcd/photos/1.jpg"
    );

    // 2. SSRF check blocks 127.0.0.1 by default
    let blocked = check_ssrf_allowed("http://127.0.0.1/sensitive.bin", None, None, false);
    assert!(blocked.is_err());
    let err = blocked.unwrap_err();
    assert_eq!(err.code, MediaFetchErrorCode::FetchFailed);
    assert!(err.message.contains("Blocked hostname or private/internal/special-use IP address"));

    // 3. SSRF check allows explicit origin
    let allowed = check_ssrf_allowed(
        "http://127.0.0.1:8080/media",
        Some(&["http://127.0.0.1:8080".to_string()]),
        None,
        false,
    );
    assert!(allowed.is_ok());
}

#[test]
fn test_ocrm_media_fetch_empty_response_and_original_filename() {
    let temp = TempTestDir::new("empty-and-orig");

    // 1. Empty body or 204 rejected with HTTP 204; empty response body
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/octet-stream".to_string());
    let opts_204 = SaveResponseMediaOptions {
        source_url: Some("https://example.com/empty".to_string()),
        base_dir: Some(temp.path.clone()),
        ..Default::default()
    };
    let res_204 = save_response_media(&[], &headers, 204, &opts_204);
    assert!(res_204.is_err());
    let err_204 = res_204.unwrap_err();
    assert_eq!(err_204.code, MediaFetchErrorCode::HttpError);
    assert_eq!(err_204.status, Some(204));
    assert!(err_204.message.contains("empty response body"));

    // 2. Preserves explicit original filename when saving stream
    let bytes = b"openclaw document test contents";
    let mut headers_docx = BTreeMap::new();
    headers_docx.insert(
        "content-type".to_string(),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
    );
    let opts_docx = SaveResponseMediaOptions {
        source_url: Some("https://example.com/views/original".to_string()),
        original_filename: Some("document.docx".to_string()),
        base_dir: Some(temp.path.clone()),
        ..Default::default()
    };
    let saved = save_response_media(bytes, &headers_docx, 200, &opts_docx).unwrap();
    assert_eq!(saved.file_name, Some("document.docx".to_string()));
    assert!(saved.path.contains("document---"));
    assert!(saved.path.ends_with(".docx"));
    assert_eq!(fs::read(&saved.path).unwrap(), bytes);
}

#[test]
fn test_ocrm_media_fetch_gateway_dispatch() {
    let temp = TempTestDir::new("gateway-dispatch");
    let service = AttachmentsService::new(
        &temp.path,
        Arc::new(MockAuth),
        Arc::new(MockBox),
        None,
    );

    // 1. Dispatch saveRemoteMedia with blocked localhost url
    let args = serde_json::json!({
        "url": "http://127.0.0.1/malicious.bin",
        "maxBytes": 1024
    });
    let result = service.dispatch_gateway("saveRemoteMedia", &args);
    assert!(result.is_some());
    let res = result.unwrap();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Blocked hostname or private/internal/special-use IP address"));

    // 2. Dispatch readRemoteMediaBuffer with blocked localhost url
    let result2 = service.dispatch_gateway("readRemoteMediaBuffer", &args);
    assert!(result2.is_some());
    let res2 = result2.unwrap();
    assert!(res2.is_err());
    assert!(res2.unwrap_err().contains("Blocked hostname or private/internal/special-use IP address"));
}

#[test]
fn media_http_error_buffer_gateway_stops_at_prefix_without_eof() {
    assert_media_gateway_stops_before_eof("readRemoteMediaBuffer");
}

#[test]
fn media_http_error_save_gateway_stops_at_prefix_without_eof() {
    assert_media_gateway_stops_before_eof("saveRemoteMedia");
}

#[test]
fn media_http_error_supplied_response_normalizes_unicode_and_whitespace() {
    let temp = TempTestDir::new("diagnostic-normalization");
    fs::write(temp.path.join("keep.bin"), b"existing media").unwrap();
    let before = media_tree_snapshot(&temp.path);
    let options = SaveResponseMediaOptions {
        source_url: Some("https://example.test/error".into()),
        max_bytes: Some(0),
        base_dir: Some(temp.path.clone()),
        ..Default::default()
    };
    let mut cases = vec![
        ("empty", Vec::new(), None),
        ("whitespace", "\t\n\u{000b}\u{000c}\r \u{00a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}".as_bytes().to_vec(), None),
        ("ECMAScript whitespace", "\u{feff} hello\u{00a0}\t\n world \u{feff}".as_bytes().to_vec(), Some("hello world".to_string())),
        ("0085 is not ECMAScript whitespace", "\u{0085}a\u{0085}\u{feff}b\u{0085}".as_bytes().to_vec(), Some("\u{0085}a\u{0085} b\u{0085}".to_string())),
        ("malformed complete UTF-8", b"a\xffb\xe2\x82".to_vec(), Some("a�b�".to_string())),
        ("UTF-8 BOM", b"\xef\xbb\xbfdiagnostic".to_vec(), Some("diagnostic".to_string())),
        ("UTF-16 BOM does not switch encoding", b"\xff\xfeA".to_vec(), Some("��A".to_string())),
        ("exact display budget", vec![b'a'; 200], Some("a".repeat(200))),
        ("display overflow", vec![b'a'; 201], Some(format!("{}…", "a".repeat(200)))),
        ("surrogate crosses display boundary", format!("{}😀tail", "a".repeat(199)).into_bytes(), Some(format!("{}…", "a".repeat(199)))),
        ("surrogate fits display boundary", format!("{}😀", "a".repeat(198)).into_bytes(), Some(format!("{}😀", "a".repeat(198)))),
    ];
    let mut short_prefix = b"diagnostic".to_vec();
    short_prefix.resize(8191, b' ');
    cases.push(("below byte budget", short_prefix.clone(), Some("diagnostic".into())));
    let mut exact_prefix = short_prefix.clone();
    exact_prefix.push(b' ');
    cases.push(("exact byte budget", exact_prefix, Some("diagnostic…".into())));
    let mut cut_scalar = short_prefix;
    cut_scalar.extend_from_slice("😀SUFFIX_MUST_NOT_APPEAR".as_bytes());
    cases.push(("cut UTF-8 scalar", cut_scalar, Some("diagnostic…".into())));
    let mut invalid_then_cut = b"a\xffb".to_vec();
    invalid_then_cut.resize(8191, b' ');
    invalid_then_cut.extend_from_slice("😀SUFFIX_MUST_NOT_APPEAR".as_bytes());
    cases.push(("replace interior and drop cut tail", invalid_then_cut, Some("a�b…".into())));
    let mut invalid_last_byte = b"diagnostic".to_vec();
    invalid_last_byte.resize(8191, b' ');
    invalid_last_byte.push(0xff);
    cases.push(("malformed full byte at budget", invalid_last_byte, Some("diagnostic �…".into())));
    let mut blank_prefix = vec![b' '; 8192];
    blank_prefix.extend_from_slice(b"SUFFIX_MUST_NOT_APPEAR");
    cases.push(("blank truncated prefix", blank_prefix, None));

    for (label, body, snippet) in cases {
        let error = save_response_media(&body, &BTreeMap::new(), 503, &options).unwrap_err();
        assert_eq!(error.code, MediaFetchErrorCode::HttpError, "{label}");
        assert_eq!(error.status, Some(503), "{label}");
        let detail = snippet.map(|text| format!("; body: {text}")).unwrap_or_default();
        assert_eq!(
            error.message,
            format!("Failed to fetch media from https://example.test/error: HTTP 503{detail}"),
            "{label}"
        );
    }
    assert_eq!(media_tree_snapshot(&temp.path), before);
}

#[test]
fn media_http_error_supplied_response_prioritizes_status_and_redacts() {
    let temp = TempTestDir::new("supplied-diagnostic-priority");
    fs::write(temp.path.join("keep.bin"), b"existing media").unwrap();
    let before = media_tree_snapshot(&temp.path);
    let token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcd";
    let source_url = format!("https://example.test/bot{token}/error");
    let body = format!(" \t echoed bot{token}\n ");
    let options = SaveResponseMediaOptions {
        source_url: Some(source_url),
        max_bytes: Some(0),
        base_dir: Some(temp.path.clone()),
        ..Default::default()
    };
    for status in [302, 401, 503] {
        for content_length in ["1e9", "999999999"] {
            let headers = BTreeMap::from([("content-length".into(), content_length.into())]);
            let error = save_response_media(body.as_bytes(), &headers, status, &options).unwrap_err();
            assert_eq!(error.code, MediaFetchErrorCode::HttpError);
            assert_eq!(error.status, Some(status));
            assert_eq!(error.message, format!(
                "Failed to fetch media from https://example.test/bot123456…abcd/error: HTTP {status}; body: echoed bot123456…abcd"
            ));
            assert!(!error.message.contains(token));
        }
    }
    assert_eq!(media_tree_snapshot(&temp.path), before);
}

#[test]
fn media_http_error_gateway_prioritizes_status_and_redacts() {
    let temp = TempTestDir::new("gateway-diagnostic-priority");
    fs::write(temp.path.join("keep.bin"), b"existing media").unwrap();
    let before = media_tree_snapshot(&temp.path);
    let token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcd";
    let body = format!(" \t echoed bot{token}\n ").into_bytes();
    for method in ["readRemoteMediaBuffer", "saveRemoteMedia"] {
        let mut fixture = MediaHttpFixture::start(401, body.clone(), body.len(), None);
        let mut args = fixture.args(&temp.path);
        args["url"] = serde_json::json!(format!("{}/bot{token}/media", fixture.origin));
        args["max_bytes"] = serde_json::json!(1);
        let result = media_gateway_call(method, &args, &temp.path);
        fixture.finish().expect("media HTTP fixture completed");
        assert!(fixture.accepted.load(Ordering::SeqCst));
        let error = result.unwrap_err();
        assert_eq!(error, format!(
            "MediaFetchError: Failed to fetch media from {}/bot123456…abcd/media: HTTP 401; body: echoed bot123456…abcd",
            fixture.origin
        ));
        assert!(!error.contains(token));
        assert_eq!(media_tree_snapshot(&temp.path), before);
    }
}

#[test]
fn media_http_error_gateway_read_failure_preserves_status() {
    let temp = TempTestDir::new("gateway-diagnostic-read-failure");
    fs::write(temp.path.join("keep.bin"), b"existing media").unwrap();
    let before = media_tree_snapshot(&temp.path);
    for method in ["readRemoteMediaBuffer", "saveRemoteMedia"] {
        // Closing before the declared length makes the real response reader
        // fail after headers and some bytes have already arrived.
        let body = b"PARTIAL_DIAGNOSTIC_MUST_BE_OMITTED".to_vec();
        let mut fixture = MediaHttpFixture::start(502, body.clone(), body.len() + 32, None);
        let result = media_gateway_call(method, &fixture.args(&temp.path), &temp.path);
        fixture.finish().expect("interrupted media fixture completed");
        assert!(fixture.accepted.load(Ordering::SeqCst));
        assert_eq!(result.unwrap_err(), format!(
            "MediaFetchError: Failed to fetch media from {}/media: HTTP 502",
            fixture.origin
        ));
        assert_eq!(media_tree_snapshot(&temp.path), before);
    }
}

#[test]
fn media_http_error_gateway_empty_and_whitespace_omit_body() {
    let temp = TempTestDir::new("gateway-empty-diagnostic");
    fs::write(temp.path.join("keep.bin"), b"existing media").unwrap();
    let before = media_tree_snapshot(&temp.path);
    for method in ["readRemoteMediaBuffer", "saveRemoteMedia"] {
        for body in [Vec::new(), " \t\r\n\u{feff}\u{00a0}".as_bytes().to_vec()] {
            let mut fixture = MediaHttpFixture::start(404, body.clone(), body.len(), None);
            let result = media_gateway_call(method, &fixture.args(&temp.path), &temp.path);
            fixture.finish().expect("empty media fixture completed");
            assert_eq!(result.unwrap_err(), format!(
                "MediaFetchError: Failed to fetch media from {}/media: HTTP 404",
                fixture.origin
            ));
            assert_eq!(media_tree_snapshot(&temp.path), before);
        }
    }
}

#[test]
fn media_http_error_gateway_success_and_default_guard_controls() {
    let temp = TempTestDir::new("gateway-success-and-guard");
    fs::write(temp.path.join("keep.bin"), b"existing media").unwrap();
    for method in ["readRemoteMediaBuffer", "saveRemoteMedia"] {
        let before = media_tree_snapshot(&temp.path);
        let mut refused = MediaHttpFixture::start(200, b"unreachable".to_vec(), 11, None);
        let mut args = refused.args(&temp.path);
        args.as_object_mut().unwrap().remove("allowed_origins");
        let refusal = media_gateway_call(method, &args, &temp.path);
        refused.finish().expect("refused fixture stopped without accepting");
        assert!(!refused.accepted.load(Ordering::SeqCst));
        assert!(refusal.unwrap_err().contains("Blocked hostname or private/internal/special-use IP address"));
        assert_eq!(media_tree_snapshot(&temp.path), before);

        let mut empty = MediaHttpFixture::start(204, Vec::new(), 0, None);
        let result = media_gateway_call(method, &empty.args(&temp.path), &temp.path);
        empty.finish().expect("204 fixture completed");
        assert!(result.unwrap_err().contains("HTTP 204; empty response body"));
        assert_eq!(media_tree_snapshot(&temp.path), before);

        let body = b"unchanged successful media bytes".to_vec();
        let mut success = MediaHttpFixture::start(200, body.clone(), body.len(), None);
        let mut args = success.args(&temp.path);
        args["max_bytes"] = serde_json::json!(body.len());
        let result = media_gateway_call(method, &args, &temp.path);
        success.finish().expect("successful media fixture completed");
        let result = result.unwrap();
        if method == "readRemoteMediaBuffer" {
            let buffer: Vec<u8> = serde_json::from_value(result["buffer"].clone()).unwrap();
            assert_eq!(buffer, body);
            assert_eq!(media_tree_snapshot(&temp.path), before);
        } else {
            let saved_path = Path::new(result["path"].as_str().unwrap());
            assert!(saved_path.starts_with(fs::canonicalize(&temp.path).unwrap()));
            assert_eq!(fs::read(saved_path).unwrap(), body);
        }
        assert_eq!(fs::read(temp.path.join("keep.bin")).unwrap(), b"existing media");
    }
}

#[test]
fn media_fetch_hostname_allowlist_matches_normalized_patterns() {
    // These controls exercise the production admission owner without DNS or HTTP.
    let cases: &[(&str, &str, Option<&[&str]>, bool)] = &[
        ("absent list", "https://other.example.com/file", None, true),
        ("empty list", "https://other.example.com/file", Some(&[]), true),
        ("effectively empty list", "https://other.example.com/file",
            Some(&["", " \t ", ".", "...", "*", " *. "]), true),
        ("exact normalized names", "https://API.Example.COM.../file",
            Some(&[" API.EXAMPLE.COM... ", "api.example.com", "", "*"]), true),
        ("policy entry trims again after trailing dots", "https://api.example.com/file",
            Some(&[" API.EXAMPLE.COM ... "]), true),
        ("final policy trim discards bare wildcard", "https://api.example.com/file",
            Some(&[" * ... "]), true),
        ("port does not change hostname", "https://api.example.com:8443/file",
            Some(&["api.example.com"]), true),
        ("exact excludes subdomain", "https://sub.api.example.com/file",
            Some(&["api.example.com"]), false),
        ("exact excludes suffix", "https://api.example.com.attacker.test/file",
            Some(&["api.example.com"]), false),
        ("one wildcard label", "https://files.example.com/file",
            Some(&[" *.EXAMPLE.COM... "]), true),
        ("deep wildcard labels", "https://a.b.example.com../file",
            Some(&["*.example.com"]), true),
        ("wildcard excludes apex", "https://example.com/file",
            Some(&["*.example.com"]), false),
        ("wildcard excludes deceptive suffix", "https://badexample.com/file",
            Some(&["*.example.com"]), false),
        ("wildcard excludes appended domain", "https://example.com.attacker.test/file",
            Some(&["*.example.com"]), false),
        ("wildcard is not an interior glob", "https://images.eu.example.com/file",
            Some(&["images.*.example.com"]), false),
        ("discarded entries do not broaden a restrictive list", "https://other.example.com/file",
            Some(&["*", "", "api.example.com", " API.EXAMPLE.COM. "]), false),
        ("IPv4 exact hostname", "https://8.8.8.8/file", Some(&[" 8.8.8.8... "]), true),
        ("bracketed IPv6 pattern", "https://[2606:4700:4700::1111]/file",
            Some(&[" [2606:4700:4700::1111]... "]), true),
        ("unbracketed IPv6 pattern", "https://[2606:4700:4700::1111]/file",
            Some(&["2606:4700:4700::1111"]), true),
        ("different IPv6 host", "https://[2606:4700:4700::1001]/file",
            Some(&["[2606:4700:4700::1111]"]), false),
    ];

    for &(label, url, patterns, allowed) in cases {
        let options = FetchMediaOptions {
            url: url.to_string(),
            hostname_allowlist: patterns.map(|entries| {
                entries.iter().map(|entry| (*entry).to_string()).collect()
            }),
            ..Default::default()
        };
        let result = check_media_fetch_url(url, &options);
        if allowed {
            assert!(result.is_ok(), "{label}: {result:?}");
        } else {
            let error = result.expect_err(label);
            assert_eq!(error.code, MediaFetchErrorCode::FetchFailed, "{label}");
            assert_eq!(error.status, None, "{label}");
            assert!(error.message.contains("Domain policy: Blocked hostname (not in allowlist):"),
                "{label}: {}", error.message);
        }
    }
}

#[test]
fn media_fetch_hostname_allowlist_does_not_override_private_trust() {
    for (url, pattern) in [
        ("http://127.0.0.1:8123/private", " 127.0.0.1... "),
        ("http://localhost:8123/private", " LOCALHOST... "),
    ] {
        let options = FetchMediaOptions {
            url: url.to_string(),
            hostname_allowlist: Some(vec![pattern.to_string()]),
            ..Default::default()
        };
        let error = check_media_fetch_url(url, &options)
            .expect_err("matching the restrictive list must not grant private access");
        assert_eq!(error.code, MediaFetchErrorCode::FetchFailed);
        assert!(error.message.contains("Blocked hostname or private/internal/special-use IP address"),
            "{url}: {}", error.message);
    }

    let url = "http://127.0.0.1:8123/private";
    let mut options = FetchMediaOptions {
        url: url.to_string(),
        hostname_allowlist: Some(vec!["127.0.0.1".into()]),
        allowed_origins: Some(vec!["http://127.0.0.1:8123".into()]),
        ..Default::default()
    };
    assert!(check_media_fetch_url(url, &options).is_ok(), "separate explicit origin trust");

    options.hostname_allowlist = Some(vec!["files.example.com".into()]);
    let error = check_media_fetch_url(url, &options)
        .expect_err("origin trust cannot bypass a restrictive list");
    assert!(error.message.contains("Domain policy: Blocked hostname (not in allowlist): 127.0.0.1"));

    options.allowed_origins = None;
    options.allowed_hostnames = Some(vec!["127.0.0.1".into()]);
    let error = check_media_fetch_url(url, &options)
        .expect_err("legacy hostname trust cannot bypass a restrictive list");
    assert!(error.message.contains("Domain policy: Blocked hostname (not in allowlist): 127.0.0.1"));

    options.hostname_allowlist = Some(vec!["127.0.0.1".into()]);
    assert!(check_media_fetch_url(url, &options).is_ok(), "legacy explicit trust remains separate");
    options.allowed_hostnames = None;
    options.allowed_origins = Some(vec!["http://127.0.0.1:8124".into()]);
    let error = check_media_fetch_url(url, &options)
        .expect_err("trust for a different port is not trust for this origin");
    assert!(error.message.contains("Blocked hostname or private/internal/special-use IP address"));

    options.allowed_origins = None;
    options.hostname_allowlist = Some(vec!["*".into(), " ".into()]);
    let error = check_media_fetch_url(url, &options)
        .expect_err("an effectively empty list still grants no private trust");
    assert!(error.message.contains("Blocked hostname or private/internal/special-use IP address"));
}

#[test]
fn media_fetch_url_admission_enforces_http_and_https_requirements() {
    for (url, require_https, expected_cause) in [
        ("http://example.com/media", None, None),
        ("http://example.com/media", Some(false), None),
        ("HTTPS://EXAMPLE.COM/media", Some(true), None),
        ("http://example.com/media", Some(true), Some("HTTPS required")),
        ("ftp://example.com/media", None, Some("HTTP or HTTPS required")),
        ("file:///tmp/media", None, Some("HTTP or HTTPS required")),
        ("data:text/plain,media", None, Some("HTTP or HTTPS required")),
        ("https://[", None, Some("Invalid URL:")),
    ] {
        let options = FetchMediaOptions {
            url: url.to_string(),
            require_https,
            // Existing trust is deliberately present and must not bypass the
            // URL scheme or HTTPS requirement at this shared admission seam.
            allowed_origins: Some(vec![url.to_string()]),
            ..Default::default()
        };
        let result = check_media_fetch_url(url, &options);
        match expected_cause {
            None => assert!(result.is_ok(), "{url}: {result:?}"),
            Some(cause) => {
                let error = result.expect_err("scheme policy must be enforced");
                assert_eq!(error.code, MediaFetchErrorCode::FetchFailed);
                assert_eq!(error.status, None);
                assert!(error.message.contains(cause), "{url}: {}", error.message);
            }
        }
    }
}

// This transport is owned by each test. Production admission, reqwest redirects,
// response reading and AttachmentsService dispatch are never mocked. A fixture
// deadline/read error is a test failure; stopping drains queued connections and
// joins the worker before assertions about the complete request log.
struct LoopbackMediaServer {
    address: SocketAddr,
    stop: mpsc::Sender<()>,
    worker: Option<JoinHandle<Result<Vec<String>, String>>>,
}

impl LoopbackMediaServer {
    fn new(make_routes: impl FnOnce(SocketAddr) -> BTreeMap<String, Vec<u8>>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind owned loopback HTTP fixture");
        let address = listener.local_addr().expect("loopback fixture address");
        listener.set_nonblocking(true).expect("bound fixture accept wait");
        let routes = make_routes(address);
        let (stop, stopping) = mpsc::channel();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(20);
            let mut requests = Vec::new();
            loop {
                if Instant::now() >= deadline {
                    return Err("owned HTTP fixture exceeded its deadline".into());
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        if requests.len() >= 64 {
                            return Err("owned HTTP fixture exceeded its request budget".into());
                        }
                        stream.set_nonblocking(false).map_err(|error| error.to_string())?;
                        stream.set_write_timeout(Some(Duration::from_secs(2)))
                            .map_err(|error| error.to_string())?;
                        let path = read_fixture_request(&mut stream)?;
                        requests.push(path.clone());
                        let response = routes.get(&path)
                            .ok_or_else(|| format!("unexpected loopback request: {path}"))?;
                        stream.write_all(response)
                            .map_err(|error| format!("respond to {path}: {error}"))?;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        match stopping.try_recv() {
                            Ok(()) | Err(mpsc::TryRecvError::Disconnected) => return Ok(requests),
                            Err(mpsc::TryRecvError::Empty) => thread::sleep(Duration::from_millis(2)),
                        }
                    }
                    Err(error) => return Err(format!("accept loopback request: {error}")),
                }
            }
        });
        Self { address, stop, worker: Some(worker) }
    }

    fn options(&self, path: &str) -> FetchMediaOptions {
        FetchMediaOptions {
            url: format!("http://{}{path}", self.address),
            max_bytes: Some(1024),
            timeout_ms: Some(5_000),
            hostname_allowlist: Some(vec!["127.0.0.1".into()]),
            allowed_origins: Some(vec![format!("http://{}", self.address)]),
            ..Default::default()
        }
    }

    fn stop_and_join(&mut self) -> Result<Vec<String>, String> {
        let _ = self.stop.send(());
        self.worker.take().expect("owned fixture worker exists")
            .join().map_err(|_| "owned HTTP fixture worker panicked".to_string())?
    }

    fn finish(mut self) -> Vec<String> {
        self.stop_and_join().expect("owned HTTP fixture completed without timeout or I/O failure")
    }
}

impl Drop for LoopbackMediaServer {
    fn drop(&mut self) {
        if self.worker.is_some() {
            let result = self.stop_and_join();
            if !thread::panicking() {
                result.expect("owned HTTP fixture teardown must succeed");
            }
        }
    }
}

fn read_fixture_request(stream: &mut TcpStream) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut request = Vec::new();
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        if request.len() >= 16 * 1024 {
            return Err("loopback request headers exceeded fixture bound".into());
        }
        let remaining = deadline.checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| "loopback request header deadline expired".to_string())?;
        stream.set_read_timeout(Some(remaining)).map_err(|error| error.to_string())?;
        let mut chunk = [0; 1024];
        let count = stream.read(&mut chunk)
            .map_err(|error| format!("read loopback request headers: {error}"))?;
        if count == 0 {
            return Err("loopback client closed before completing request headers".into());
        }
        request.extend_from_slice(&chunk[..count]);
    }
    let header = std::str::from_utf8(&request).map_err(|error| error.to_string())?;
    let mut fields = header.lines().next().unwrap_or_default().split_whitespace();
    if fields.next() != Some("GET") {
        return Err("loopback fixture expected a real GET request".into());
    }
    let path = fields.next().ok_or_else(|| "loopback request omitted its target".to_string())?;
    if fields.next() != Some("HTTP/1.1") || fields.next().is_some() {
        return Err("unexpected loopback HTTP request line".into());
    }
    Ok(path.to_string())
}

fn fixture_response(body: &[u8], content_type: &str, disposition: Option<&str>) -> Vec<u8> {
    let mut headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\nConnection: close\r\n",
        body.len(),
    );
    if let Some(disposition) = disposition {
        headers.push_str(&format!("Content-Disposition: {disposition}\r\n"));
    }
    headers.push_str("\r\n");
    let mut response = headers.into_bytes();
    response.extend_from_slice(body);
    response
}

fn fixture_chunked_response(body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n{:x}\r\n",
        body.len(),
    ).into_bytes();
    response.extend_from_slice(body);
    response.extend_from_slice(b"\r\n0\r\n\r\n");
    response
}

fn fixture_redirect(location: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    ).into_bytes()
}

fn gateway_media_read(service: &AttachmentsService, options: &FetchMediaOptions) -> Result<Value, String> {
    let args = serde_json::to_value(options).expect("serialize the real media fetch options");
    service.dispatch_gateway("readRemoteMediaBuffer", &args)
        .expect("actual media reader RPC dispatch")
}

#[test]
fn gateway_media_fetch_trusted_origin_preserves_body_and_byte_bounds() {
    const BODY: &[u8] = b"MURL-v1 loopback body";
    const CONTENT_TYPE: &str = "application/x-openclaw-loopback-proof; charset=utf-8";
    let temp = TempTestDir::new("trusted-loopback-body");
    let service = AttachmentsService::new(&temp.path, Arc::new(MockAuth), Arc::new(MockBox), None);
    let server = LoopbackMediaServer::new(|_| BTreeMap::from([
        ("/body".into(), fixture_response(BODY, CONTENT_TYPE,
            Some("attachment; filename*=UTF-8''loopback%20proof.dat"))),
        ("/stream".into(), fixture_chunked_response(BODY)),
    ]));
    let mut known_length = server.options("/body");
    known_length.max_bytes = Some(BODY.len());
    let result = gateway_media_read(&service, &known_length).expect("trusted loopback body at exact cap");
    assert_eq!(result["buffer"], serde_json::json!(BODY));
    assert_eq!(result["content_type"], CONTENT_TYPE);
    assert_eq!(result["file_name"], "loopback proof.dat");

    let mut untrusted = known_length.clone();
    untrusted.allowed_origins = None;
    let error = gateway_media_read(&service, &untrusted)
        .expect_err("a matching list alone cannot admit the initial private request");
    assert_eq!(error, format!(
        "MediaFetchError: Failed to fetch media from {}: Blocked hostname or private/internal/special-use IP address",
        untrusted.url,
    ));
    let mut off_list = known_length.clone();
    off_list.hostname_allowlist = Some(vec!["files.example.com".into()]);
    let error = gateway_media_read(&service, &off_list)
        .expect_err("explicit origin trust cannot bypass the initial restrictive list");
    assert_eq!(error, format!(
        "MediaFetchError: Failed to fetch media from {}: Domain policy: Blocked hostname (not in allowlist): 127.0.0.1",
        off_list.url,
    ));

    let mut streamed = server.options("/stream");
    streamed.max_bytes = Some(BODY.len());
    let result = gateway_media_read(&service, &streamed).expect("chunked loopback body at exact cap");
    assert_eq!(result["buffer"], serde_json::json!(BODY));

    for (mut options, cause) in [
        (known_length, format!("content length {} exceeds maxBytes {}", BODY.len(), BODY.len() - 1)),
        (streamed, format!("payload exceeds maxBytes {}", BODY.len() - 1)),
    ] {
        options.max_bytes = Some(BODY.len() - 1);
        let typed = read_remote_media_buffer(&options).expect_err("actual response exceeds the byte cap");
        assert_eq!(typed.code, MediaFetchErrorCode::MaxBytes);
        assert_eq!(typed.status, None);
        assert!(typed.message.contains(&cause), "{}", typed.message);
        let error = gateway_media_read(&service, &options).expect_err("gateway retains the byte refusal");
        assert_eq!(error, typed.to_string());
    }

    let requests = server.finish();
    assert_eq!(requests, ["/body", "/stream", "/body", "/body", "/stream", "/stream"]);
    assert_eq!(fs::read_dir(&temp.path).expect("inspect reader root").count(), 0,
        "buffer reads must not save or stage media");
}

#[test]
fn gateway_media_fetch_checks_relative_and_off_list_redirects() {
    const BODY: &[u8] = b"redirect reached the owned response";
    let temp = TempTestDir::new("loopback-redirect-policy");
    let service = AttachmentsService::new(&temp.path, Arc::new(MockAuth), Arc::new(MockBox), None);
    let server = LoopbackMediaServer::new(|address| BTreeMap::from([
        ("/dir/start".into(), fixture_redirect("next")),
        ("/dir/next".into(), fixture_response(BODY, "text/plain", None)),
        ("/off-list/control".into(), fixture_response(BODY, "text/plain", None)),
        ("/off-list/start".into(), fixture_redirect(&format!(
            "http://localhost:{}/off-list/blocked", address.port(),
        ))),
        ("/off-list/blocked".into(), fixture_response(BODY, "text/plain", None)),
        ("/private/start".into(), fixture_redirect(&format!(
            "http://localhost:{}/private/blocked", address.port(),
        ))),
        ("/private/blocked".into(), fixture_response(BODY, "text/plain", None)),
        ("/scheme/start".into(), fixture_redirect(&format!(
            "ftp://127.0.0.1:{}/scheme/blocked", address.port(),
        ))),
    ]));
    let localhost_origin = format!("http://localhost:{}", server.address.port());

    // Prove that the next-hop host has a live route before using its absence
    // as evidence. Only the generic reader receives explicit private trust.
    let mut control = server.options("/off-list/control");
    control.url = format!("{localhost_origin}/off-list/control");
    control.hostname_allowlist = Some(vec!["localhost".into()]);
    control.allowed_origins = Some(vec![localhost_origin.clone()]);
    let result = gateway_media_read(&service, &control).expect("owned localhost control must succeed");
    assert_eq!(result["buffer"], serde_json::json!(BODY));

    let mut relative = server.options("/dir/start");
    relative.max_redirects = Some(1);
    let result = gateway_media_read(&service, &relative).expect("one allowed relative redirect");
    assert_eq!(result["buffer"], serde_json::json!(BODY));

    let mut off_list = server.options("/off-list/start");
    off_list.allowed_origins.as_mut().unwrap().push(localhost_origin.clone());
    let denied_url = format!("{localhost_origin}/off-list/blocked");
    let expected_message = format!(
        "Failed to fetch media from {denied_url}: Domain policy: Blocked hostname (not in allowlist): localhost",
    );
    let typed = read_remote_media_buffer(&off_list).expect_err("restrictive list applies at the next hop");
    assert_eq!(typed.code, MediaFetchErrorCode::FetchFailed);
    assert_eq!(typed.status, None);
    assert_eq!(typed.message, expected_message, "reqwest must preserve the typed policy cause");
    let error = gateway_media_read(&service, &off_list).expect_err("gateway refuses an off-list next hop");
    assert_eq!(error, format!("MediaFetchError: {expected_message}"));

    let mut private_hop = server.options("/private/start");
    private_hop.hostname_allowlist = Some(vec!["127.0.0.1".into(), "localhost".into()]);
    let error = gateway_media_read(&service, &private_hop)
        .expect_err("an allowlisted next hop still needs its own private-origin trust");
    assert_eq!(error, format!(
        "MediaFetchError: Failed to fetch media from {localhost_origin}/private/blocked: Blocked hostname or private/internal/special-use IP address",
    ));

    let invalid_scheme = server.options("/scheme/start");
    let error = gateway_media_read(&service, &invalid_scheme)
        .expect_err("redirect admission must reapply the HTTP(S) requirement");
    assert_eq!(error, format!(
        "MediaFetchError: Failed to fetch media from ftp://127.0.0.1:{}/scheme/blocked: HTTP or HTTPS required",
        server.address.port(),
    ));

    relative.require_https = Some(true);
    let error = gateway_media_read(&service, &relative)
        .expect_err("explicit origin trust cannot bypass HTTPS admission");
    assert_eq!(error, format!("MediaFetchError: Failed to fetch media from {}: HTTPS required", relative.url));

    let requests = server.finish();
    assert_eq!(requests, [
        "/off-list/control", "/dir/start", "/dir/next", "/off-list/start", "/off-list/start",
        "/private/start", "/scheme/start",
    ], "neither the off-list nor the untrusted private next hop may be requested");
    assert_eq!(fs::read_dir(&temp.path).expect("inspect reader root").count(), 0);
}

#[test]
fn gateway_media_fetch_applies_explicit_and_default_redirect_caps() {
    const BODY: &[u8] = b"terminal redirect body";
    let temp = TempTestDir::new("loopback-redirect-caps");
    let service = AttachmentsService::new(&temp.path, Arc::new(MockAuth), Arc::new(MockBox), None);
    for (label, cap, hops, omit_cap, succeeds) in [
        ("zero permits the initial response", Some(0), 0, false, true),
        ("zero refuses the first follow-up", Some(0), 1, false, false),
        ("one permits one follow-up", Some(1), 1, false, true),
        ("one refuses two follow-ups", Some(1), 2, false, false),
        ("N permits exactly N follow-ups", Some(3), 3, false, true),
        ("N refuses N plus one", Some(3), 4, false, false),
        ("omitted default permits five", None, 5, true, true),
        ("omitted default refuses six", None, 6, true, false),
        ("null default permits five", None, 5, false, true),
        ("null default refuses six", None, 6, false, false),
    ] {
        let server = LoopbackMediaServer::new(|_| {
            (0..=hops).map(|index| {
                let response = if index == hops {
                    fixture_response(BODY, "text/plain", None)
                } else {
                    fixture_redirect(&format!("/hop/{}", index + 1))
                };
                (format!("/hop/{index}"), response)
            }).collect()
        });
        let mut options = server.options("/hop/0");
        options.max_redirects = cap;
        let mut args = serde_json::to_value(&options).expect("serialize real redirect options");
        if omit_cap {
            args.as_object_mut().unwrap().remove("max_redirects");
        }
        let result = service.dispatch_gateway("readRemoteMediaBuffer", &args)
            .expect("actual media reader RPC dispatch");
        let effective_cap = cap.unwrap_or(5);
        if succeeds {
            let value = result.unwrap_or_else(|error| panic!("{label}: {error}"));
            assert_eq!(value["buffer"], serde_json::json!(BODY), "{label}");
        } else {
            let error = result.expect_err(label);
            assert_eq!(error, format!(
                "MediaFetchError: Failed to fetch media from {}: Too many redirects (limit: {effective_cap})",
                options.url,
            ), "{label}: a stopped redirect or unrelated network error is not a policy refusal");
        }
        let requested_hops = if succeeds { hops } else { effective_cap };
        let expected: Vec<String> = (0..=requested_hops).map(|index| format!("/hop/{index}")).collect();
        assert_eq!(server.finish(), expected, "{label}: exact initial and follow-up request count");
        assert_eq!(fs::read_dir(&temp.path).expect("inspect reader root").count(), 0, "{label}");
    }
}
