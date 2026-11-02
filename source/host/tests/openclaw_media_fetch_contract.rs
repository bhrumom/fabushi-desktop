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

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentsBox, AttachmentsService,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
use mahayana_host_runtime::extensions::attachments::media_fetch::{
    check_ssrf_allowed, is_generic_response_content_type,
    parse_content_disposition_file_name, parse_media_content_length,
    redact_sensitive_text, resolve_remote_file_name,
    resolve_response_content_type, retain_charset_parameter,
    save_response_media, MediaFetchErrorCode, SaveResponseMediaOptions,
};

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
