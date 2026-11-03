use std::fs;
use std::path::{Path, PathBuf};
use mahayana_host_runtime::extensions::attachments::host_read_media_policy::{
    assert_host_read_media_allowed, filter_stale_provenance_rows, has_html_document_shape,
    is_allowed_host_read_html, validate_provenance_row, AssertHostReadParams,
    GeneratedHtmlProvenanceRow, HostReadHtmlTrust, HostReadMediaTypeError,
    HOST_READ_DECLARED_TEXT_ERROR,
};
use sha2::{Digest, Sha256};

#[test]
fn test_html_provenance_marker_validation() {
    let valid_row = GeneratedHtmlProvenanceRow {
        realpath: "/path/to/report.html".to_string(),
        kind: "trusted-generated-html".to_string(),
        version: 1,
        sha256: "abc123def456".to_string(),
        size_bytes: 4096,
        created_at_ms: 1700000000000,
    };
    let marker = validate_provenance_row(&valid_row);
    assert!(marker.is_some());
    let marker = marker.unwrap();
    assert_eq!(marker.sha256, "abc123def456");
    assert_eq!(marker.size, 4096);

    let wrong_version = GeneratedHtmlProvenanceRow {
        version: 2,
        ..valid_row.clone()
    };
    assert!(validate_provenance_row(&wrong_version).is_none());

    let wrong_kind = GeneratedHtmlProvenanceRow {
        kind: "untrusted-html".to_string(),
        ..valid_row
    };
    assert!(validate_provenance_row(&wrong_kind).is_none());
}

#[test]
fn test_html_provenance_filter_stale_rows() {
    let temp_dir = std::env::temp_dir().join(format!("ocrm-html-provenance-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("create_dir_all");
    let regular_file = temp_dir.join("regular.html");
    fs::write(&regular_file, b"<!doctype html><html><body>Report</body></html>").unwrap();

    let missing_file = temp_dir.join("missing.html");

    #[cfg(unix)]
    let symlink_file = temp_dir.join("symlink.html");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&regular_file, &symlink_file).unwrap();

    let rows = vec![
        GeneratedHtmlProvenanceRow {
            realpath: regular_file.to_string_lossy().to_string(),
            kind: "trusted-generated-html".to_string(),
            version: 1,
            sha256: "dummy".to_string(),
            size_bytes: 40,
            created_at_ms: 1000,
        },
        GeneratedHtmlProvenanceRow {
            realpath: missing_file.to_string_lossy().to_string(),
            kind: "trusted-generated-html".to_string(),
            version: 1,
            sha256: "dummy".to_string(),
            size_bytes: 40,
            created_at_ms: 1000,
        },
        #[cfg(unix)]
        GeneratedHtmlProvenanceRow {
            realpath: symlink_file.to_string_lossy().to_string(),
            kind: "trusted-generated-html".to_string(),
            version: 1,
            sha256: "dummy".to_string(),
            size_bytes: 40,
            created_at_ms: 1000,
        },
    ];

    let stale = filter_stale_provenance_rows(&rows);
    #[cfg(unix)]
    {
        assert_eq!(stale.len(), 2);
        let stale_paths: Vec<_> = stale.iter().map(|r| r.realpath.as_str()).collect();
        assert!(stale_paths.contains(&missing_file.to_string_lossy().as_ref()));
        assert!(stale_paths.contains(&symlink_file.to_string_lossy().as_ref()));
    }
    #[cfg(not(unix))]
    {
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].realpath, missing_file.to_string_lossy().as_ref());
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_html_provenance_host_read_admission() {
    let payload = b"<!doctype html><html><head><title>Test</title></head><body>Valid</body></html>";
    let sha256 = format!("{:x}", Sha256::digest(payload));
    let size = payload.len();

    let trust = HostReadHtmlTrust::Outbound {
        expected_sha256: sha256.clone(),
        expected_size: size,
    };

    // Valid HTML with matching outbound provenance trust
    assert!(is_allowed_host_read_html(
        Some("text/html"),
        Some(payload),
        Some(&trust),
        false
    ));

    // Mismatched size rejected
    let bad_size_trust = HostReadHtmlTrust::Outbound {
        expected_sha256: sha256.clone(),
        expected_size: size + 1,
    };
    assert!(!is_allowed_host_read_html(
        Some("text/html"),
        Some(payload),
        Some(&bad_size_trust),
        false
    ));

    // Mismatched SHA256 rejected
    let bad_sha_trust = HostReadHtmlTrust::Outbound {
        expected_sha256: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        expected_size: size,
    };
    assert!(!is_allowed_host_read_html(
        Some("text/html"),
        Some(payload),
        Some(&bad_sha_trust),
        false
    ));

    // No provenance trust rejected unless explicit publication
    assert!(!is_allowed_host_read_html(
        Some("text/html"),
        Some(payload),
        None,
        false
    ));
    assert!(is_allowed_host_read_html(
        Some("text/html"),
        Some(payload),
        None,
        true
    ));
}

fn admit_html(
    payload: &[u8],
    sniffed_mime: Option<&str>,
    trust: Option<&HostReadHtmlTrust>,
    explicit_publication: bool,
) -> Result<(), HostReadMediaTypeError> {
    assert_host_read_media_allowed(&AssertHostReadParams {
        sniffed_content_type: sniffed_mime,
        content_type: Some("text/html"),
        file_path: Some(Path::new("report.html")),
        kind: Some("document"),
        buffer: Some(payload),
        trusted_generated_html_path: trust,
        explicit_html_publication: explicit_publication,
    })
}

fn outbound_trust(payload: &[u8]) -> HostReadHtmlTrust {
    HostReadHtmlTrust::Outbound {
        expected_sha256: format!("{:x}", Sha256::digest(payload)),
        expected_size: payload.len(),
    }
}

#[test]
fn test_html_shape_ecmascript_whitespace() {
    // These are ECMAScript WhiteSpace/LineTerminator code points, not Rust's set.
    let whitespace = [
        '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}',
        '\u{00a0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}',
        '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}',
        '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}', '\u{3000}',
        '\u{feff}',
    ];
    for c in whitespace {
        assert!(has_html_document_shape(&format!("{c}<HtMl>")), "leading {c:?}");
        assert!(has_html_document_shape(&format!("<!DoCtYpE{c}hTmL>")), "doctype {c:?}");
    }
    for c in ['\u{0085}', '\u{180e}', '\u{200b}'] {
        assert!(!has_html_document_shape(&format!("{c}<html>")), "leading {c:?}");
        assert!(!has_html_document_shape(&format!("<!doctype{c}html>")), "doctype {c:?}");
    }
    assert!(has_html_document_shape(&format!("{}<html>", "\u{feff} ".repeat(8192))));
    assert!(!has_html_document_shape(" \t\r\n\u{feff}"));
}

#[test]
fn test_html_shape_header_word_boundaries() {
    for html in [
        "<html>", "<HTML", "<html lang=en>", "<!DOCTYPE html>",
        "<!doctype\t\r\nHTML", "<htmlé>", "<html\u{0301}>", "<!doctype htmlé>",
        "prefix</HtMl>", "prefix</BoDy>", "<htmlish>body</html>",
    ] {
        assert!(has_html_document_shape(html), "expected HTML shape: {html:?}");
    }
    for html in [
        "<htmlish>", "<html_>", "<html0>", "<htmlſ>", "<htmlK>",
        "<!doctype htmlish>", "<!doctype html_>", "<!doctype html9>",
        "<!doctype htmlſ>", "<!doctype htmlK>", "<!doctypehtml>",
        "<!doctype >", "< html>", "<htm>", "</html >", "</body >", "", "plain text",
    ] {
        assert!(!has_html_document_shape(html), "unexpected HTML shape: {html:?}");
    }
}

#[test]
fn test_html_shape_utf16_window_boundaries() {
    for scalar in ["x", "é", "中"] {
        let inside = format!("{}</HtMl>", scalar.repeat(8185));
        let outside = format!("{}</HtMl>", scalar.repeat(8186));
        assert!(has_html_document_shape(&inside), "last complete tag for {scalar:?}");
        assert!(!has_html_document_shape(&outside), "partial tag for {scalar:?}");
    }
    assert!(has_html_document_shape(&format!("{}x</body>", "🦀".repeat(4092))));
    assert!(!has_html_document_shape(&format!("{}</body>", "🦀".repeat(4093))));
    assert!(!has_html_document_shape(&format!("{}🦀</html>", "x".repeat(8191))));
    assert!(!has_html_document_shape(&format!("{}</body>", "x".repeat(100_000))));

    // Each scalar straddles byte 8192. The recognized header must still return
    // true without a UTF-8 slicing panic, regardless of byte width.
    for scalar in ["é", "中", "🦀"] {
        let html = format!("<html>{}{scalar}", "x".repeat(8185));
        assert!(has_html_document_shape(&html), "byte boundary for {scalar:?}");
    }
}

#[test]
fn test_html_policy_public_assertion_trust_mime_and_byte_identity() {
    let payload = b"<!doctype html><html><body>safe</body></html>";
    let trust = outbound_trust(payload);
    assert!(admit_html(payload, Some("text/html"), Some(&trust), false).is_ok());
    assert!(admit_html(payload, None, Some(&trust), false).is_ok());
    assert!(admit_html(payload, Some("text/html"), Some(&HostReadHtmlTrust::TempRoot), false).is_ok());
    assert!(admit_html(payload, Some("text/html"), None, true).is_ok());
    assert_eq!(
        admit_html(payload, Some("text/html"), None, false),
        Err(HostReadMediaTypeError(HOST_READ_DECLARED_TEXT_ERROR.to_string()))
    );

    for mime in ["image/png", "application/pdf", "text/plain"] {
        assert!(admit_html(payload, Some(mime), Some(&trust), false).is_err(), "{mime}");
        assert!(admit_html(payload, Some(mime), None, true).is_err(), "explicit {mime}");
    }
    let bad_size = HostReadHtmlTrust::Outbound {
        expected_sha256: format!("{:x}", Sha256::digest(payload)),
        expected_size: payload.len() + 1,
    };
    let bad_hash = HostReadHtmlTrust::Outbound {
        expected_sha256: "0".repeat(64),
        expected_size: payload.len(),
    };
    assert!(admit_html(payload, Some("text/html"), Some(&bad_size), false).is_err());
    assert!(admit_html(payload, Some("text/html"), Some(&bad_hash), false).is_err());
    let changed = b"<!doctype html><html><body>evil</body></html>";
    assert_eq!(changed.len(), payload.len());
    assert!(admit_html(changed, Some("text/html"), Some(&trust), false).is_err());
    assert!(admit_html(&[0, 1, 2, 3], Some("text/html"), None, true).is_err());
    assert!(admit_html(b"", Some("text/html"), None, true).is_err());
    assert!(assert_host_read_media_allowed(&AssertHostReadParams {
        file_path: Some(Path::new("report.html")),
        sniffed_content_type: Some("text/html"),
        trusted_generated_html_path: Some(&trust),
        ..AssertHostReadParams::default()
    }).is_err());
}

#[test]
fn test_html_policy_public_assertion_unicode_and_malformed_shape() {
    let mut allowed = vec![
        format!("{}</body>", "é".repeat(8185)),
        format!("{}</html>", "中".repeat(8185)),
        format!("{}x</body>", "🦀".repeat(4092)),
        "\u{feff}<HTML>report".to_string(),
    ];
    for scalar in ["é", "中", "🦀"] {
        allowed.push(format!("<html>{}{scalar}", "x".repeat(8185)));
    }
    for html in allowed {
        let payload = html.as_bytes();
        let trust = outbound_trust(payload);
        assert!(admit_html(payload, Some("text/html"), Some(&trust), false).is_ok());
        assert!(admit_html(payload, Some("text/html"), None, true).is_ok());
        assert!(admit_html(payload, Some("text/html"), None, false).is_err());
    }

    let mut refused = vec![
        format!("{}</html>", "é".repeat(8186)),
        format!("{}</body>", "🦀".repeat(4093)),
        format!("{}🦀</html>", "x".repeat(8191)),
        format!("{}</html>", "x".repeat(8192)),
    ];
    for prefix in [
        "<htmlish>", "<html_>", "<html0>", "<htmlſ>", "<htmlK>",
        "<!doctype htmlish>", "<!doctype html_>", "<!doctype html9>",
        "<!doctype htmlſ>", "<!doctype htmlK>", "<!doctypehtml>",
    ] {
        refused.push(format!("{prefix}report without a closing document tag"));
    }
    for html in refused {
        let payload = html.as_bytes();
        let trust = outbound_trust(payload);
        assert!(admit_html(payload, Some("text/html"), Some(&trust), false).is_err());
        assert!(admit_html(payload, Some("text/html"), None, true).is_err());
    }
}

struct ProvenanceFixture {
    root: PathBuf,
}

impl ProvenanceFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("ocrm-html-policy-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("create owned provenance fixture");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for ProvenanceFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn provenance_row(path: &Path, serial: i64) -> GeneratedHtmlProvenanceRow {
    GeneratedHtmlProvenanceRow {
        realpath: path.to_str().expect("UTF-8 fixture path").to_string(),
        kind: "trusted-generated-html".to_string(),
        version: 1,
        sha256: format!("snapshot-{serial}"),
        size_bytes: 100 + serial,
        created_at_ms: 1_700_000_000_000 + serial,
    }
}

#[test]
fn test_html_provenance_stale_snapshot_order_and_missing_kinds() {
    let fixture = ProvenanceFixture::new();
    let regular = fixture.path("regular.html");
    let missing = fixture.path("missing.html");
    let file_parent = fixture.path("file-parent");
    let not_directory = file_parent.join("report.html");
    let directory = fixture.path("directory.html");
    fs::write(&regular, b"<html>regular</html>").unwrap();
    fs::write(&file_parent, b"not a directory").unwrap();
    fs::create_dir(&directory).unwrap();
    assert_eq!(fs::symlink_metadata(&missing).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    let parent_error = fs::symlink_metadata(&not_directory).unwrap_err();
    assert!(matches!(
        parent_error.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
    ));
    #[cfg(unix)]
    assert_eq!(parent_error.kind(), std::io::ErrorKind::NotADirectory);

    let rows = vec![
        provenance_row(&regular, 1),
        provenance_row(&directory, 2),
        provenance_row(&not_directory, 3),
        provenance_row(&missing, 4),
        provenance_row(&missing, 5),
    ];
    assert_eq!(filter_stale_provenance_rows(&rows), rows[1..].to_vec());
    assert_eq!(fs::read(&regular).unwrap(), b"<html>regular</html>");
    assert!(fs::symlink_metadata(&directory).unwrap().is_dir());
    assert_eq!(fs::read(&file_parent).unwrap(), b"not a directory");

    fs::remove_file(&file_parent).unwrap();
    fs::create_dir(&file_parent).unwrap();
    fs::write(&not_directory, b"<html>repaired</html>").unwrap();
    fs::write(&missing, b"<html>created</html>").unwrap();
    fs::remove_dir(&directory).unwrap();
    fs::write(&directory, b"<html>regular now</html>").unwrap();
    assert!(filter_stale_provenance_rows(&rows).is_empty());
}

#[cfg(unix)]
#[test]
fn test_html_provenance_stale_link_kinds_and_recovery() {
    use std::os::unix::fs::{symlink, MetadataExt};

    let fixture = ProvenanceFixture::new();
    let regular = fixture.path("regular.html");
    let hard_source = fixture.path("hard-source.html");
    let hard_link = fixture.path("hard-link.html");
    let symbolic = fixture.path("symbolic.html");
    let dangling = fixture.path("dangling.html");
    fs::write(&regular, b"<html>regular</html>").unwrap();
    fs::write(&hard_source, b"<html>linked</html>").unwrap();
    fs::hard_link(&hard_source, &hard_link).unwrap();
    symlink(&regular, &symbolic).unwrap();
    symlink(fixture.path("absent.html"), &dangling).unwrap();
    assert_eq!(fs::symlink_metadata(&hard_source).unwrap().nlink(), 2);
    assert_eq!(fs::symlink_metadata(&hard_link).unwrap().nlink(), 2);
    assert!(fs::symlink_metadata(&symbolic).unwrap().file_type().is_symlink());
    assert!(fs::symlink_metadata(&dangling).unwrap().file_type().is_symlink());
    let rows = vec![
        provenance_row(&regular, 10),
        provenance_row(&hard_source, 11),
        provenance_row(&symbolic, 12),
        provenance_row(&hard_link, 13),
        provenance_row(&dangling, 14),
    ];
    assert_eq!(filter_stale_provenance_rows(&rows), rows[1..].to_vec());

    fs::remove_file(&hard_link).unwrap();
    fs::write(&hard_link, b"<html>independent</html>").unwrap();
    fs::remove_file(&symbolic).unwrap();
    fs::write(&symbolic, b"<html>regular now</html>").unwrap();
    fs::remove_file(&dangling).unwrap();
    fs::write(&dangling, b"<html>regular now</html>").unwrap();
    assert_eq!(fs::symlink_metadata(&hard_source).unwrap().nlink(), 1);
    assert_eq!(fs::symlink_metadata(&hard_link).unwrap().nlink(), 1);
    assert!(filter_stale_provenance_rows(&rows).is_empty());
}

#[cfg(unix)]
#[test]
fn test_html_provenance_permission_denied_retained_and_recovered() {
    use std::os::unix::fs::PermissionsExt;

    struct RestorePermissions {
        path: PathBuf,
        permissions: fs::Permissions,
    }
    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            let _ = fs::set_permissions(&self.path, self.permissions.clone());
        }
    }

    let fixture = ProvenanceFixture::new();
    let locked = fixture.path("locked");
    fs::create_dir(&locked).unwrap();
    let report = locked.join("report.html");
    fs::write(&report, b"<html>retained</html>").unwrap();
    let row = provenance_row(&report, 20);
    let original = fs::metadata(&locked).unwrap().permissions();
    let _restore = RestorePermissions { path: locked.clone(), permissions: original.clone() };
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    let error = fs::symlink_metadata(&report)
        .expect_err("permission fixture requires an unprivileged Unix runner");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(filter_stale_provenance_rows(std::slice::from_ref(&row)).is_empty());

    fs::set_permissions(&locked, original).expect("restore directory inspection");
    assert!(fs::symlink_metadata(&report).unwrap().is_file());
    assert!(filter_stale_provenance_rows(std::slice::from_ref(&row)).is_empty());
    fs::remove_file(&report).unwrap();
    assert_eq!(filter_stale_provenance_rows(std::slice::from_ref(&row)), vec![row]);
}

#[cfg(unix)]
#[test]
fn test_html_provenance_symlink_loop_retained_and_recovered() {
    use std::os::unix::fs::symlink;

    let fixture = ProvenanceFixture::new();
    let loop_parent = fixture.path("loop");
    symlink("loop", &loop_parent).unwrap();
    let report = loop_parent.join("report.html");
    let row = provenance_row(&report, 30);
    let error = fs::symlink_metadata(&report).expect_err("parent symlink loop must refuse inspection");
    assert_eq!(error.raw_os_error(), Some(libc::ELOOP));
    assert!(filter_stale_provenance_rows(std::slice::from_ref(&row)).is_empty());

    fs::remove_file(&loop_parent).unwrap();
    fs::create_dir(&loop_parent).unwrap();
    fs::write(&report, b"<html>recovered</html>").unwrap();
    assert!(fs::symlink_metadata(&report).unwrap().is_file());
    assert!(filter_stale_provenance_rows(std::slice::from_ref(&row)).is_empty());
    fs::remove_file(&report).unwrap();
    assert_eq!(filter_stale_provenance_rows(std::slice::from_ref(&row)), vec![row]);
}
