use std::path::{Path, PathBuf};

use mahayana_host_runtime::extensions::attachments::media_reference::{
    admit_local_media_path, resolve_local_file_url,
};

fn native_file_url(name: &str) -> (PathBuf, String) {
    let path = std::env::temp_dir().join(name);
    let url = url::Url::from_file_path(&path).expect("native absolute file URL");
    (path, url.to_string())
}

#[test]
fn lfa_shared_owner_distinguishes_non_file_sources() {
    for source in [
        "",
        "relative/image.png",
        "https://example.invalid/image.png",
        "http://example.invalid/image.png",
        "data:image/png;base64,AQID",
        "media://inbound/image.png",
        "file-name.png",
        "file",
        "ftp://example.invalid/image.png",
        "\u{00a0}file:///tmp/image.png",
    ] {
        assert_eq!(resolve_local_file_url(source), Ok(None), "{source:?}");
    }
}

#[test]
fn lfa_shared_owner_preserves_native_file_url_variants() {
    let (path, url) = native_file_url("lfa café Ω image.png");
    let variants = [
        url.clone(),
        url.replacen("file:", "FILE:", 1),
        url.replacen("file:", "fIlE:", 1),
        url.replacen("file://", "FILE:", 1),
        url.replacen("file://", "file://localhost", 1),
        url.replacen("file://", "file://LOCALHOST", 1),
        url.replacen("file://", "file://%6cocalhost", 1),
        format!("\u{001f} \tf\ti\rL\ne:{}\r\n\u{0001}", &url[5..]),
    ].into_iter();
    #[cfg(windows)]
    let variants = variants.chain({
        let pipe_path = url.strip_prefix("file://").expect("native scheme").replacen(':', "|", 1);
        [
            format!("file://{pipe_path}"),
            format!("file://localhost{pipe_path}"),
            format!("file:{pipe_path}"),
            format!("file://{}", pipe_path.trim_start_matches('/')),
        ]
    });
    for source in variants {
        assert_eq!(resolve_local_file_url(&source), Ok(Some(path.clone())), "{source:?}");
    }
}

#[test]
fn lfa_shared_owner_rejects_all_remote_authorities() {
    for source in [
        "file://server/share/image.png",
        "FILE://not-attacker/share/image.png",
        "file://attacker/share/image.png",
        "file://127.0.0.1/share/image.png",
        "file://[::1]/share/image.png",
        "file://%73erver/share/image.png",
        "file://localhost.example/share/image.png",
        "\u{001f}f\ti\nle://server/share/image.png",
        // url 2.5.8 otherwise removes this host while normalizing a drive path.
        "file://server/C:/safe/image.png",
        "file://127.0.0.1/C:/safe/image.png",
        "file://server/C|/safe/image.png",
        "file://localhost.example/C|/safe/image.png",
    ] {
        assert!(resolve_local_file_url(source).is_err(), "{source:?}");
    }
}

#[test]
fn lfa_shared_owner_rejects_malformed_file_urls_after_normalization() {
    for source in [
        "file://[invalid/image.png",
        "file://localhost:99/image.png",
        "file://user@localhost/image.png",
        "file://user:password@localhost/image.png",
        "\u{0001} \tf\ti\rLe://localhost:bad/image.png\n",
    ] {
        assert!(resolve_local_file_url(source).is_err(), "{source:?}");
    }
}

#[test]
fn lfa_shared_owner_rejects_bad_path_escapes_without_fallback() {
    let (_, url) = native_file_url("lfa-path");
    for suffix in [
        "%",
        "%0",
        "%GG",
        "%2fimage.png",
        "%2Fimage.png",
        "%5cimage.png",
        "%5Cimage.png",
        "%ff",
        "%C0%AF",
        "%ED%A0%80",
        "%E2%82",
        "%00image.png",
        "literal\0image.png",
    ] {
        let source = format!("{url}{suffix}");
        assert!(resolve_local_file_url(&source).is_err(), "{source:?}");
    }
}

#[test]
fn lfa_shared_owner_decodes_path_once_and_ignores_url_metadata() {
    let (path, url) = native_file_url("lfa literal%2F café #.png");
    assert!(url.contains("%252F"), "the fixture must encode a literal percent");
    let source = format!("{url}?signature=%2F%GG#preview%5C%FF");
    assert_eq!(resolve_local_file_url(&source), Ok(Some(path)));
}

#[test]
fn lfa_shared_owner_keeps_refusal_stateless() {
    let (path, url) = native_file_url("lfa-after-refusal.png");
    let invalid = format!("{url}%FF");
    for _ in 0..3 {
        assert!(resolve_local_file_url(&invalid).is_err());
        assert_eq!(resolve_local_file_url(&url), Ok(Some(path.clone())));
    }
}

#[test]
fn lfa_path_guard_rejects_nul_on_every_platform() {
    assert!(admit_local_media_path(Path::new("ordinary\0image.png")).is_err());
}

#[cfg(unix)]
#[test]
fn lfa_unix_path_guard_preserves_colon_and_double_slash_paths() {
    for source in ["/tmp/file:stream.png", "//local/path/image.png", "/tmp/café Ω.png"] {
        assert!(admit_local_media_path(Path::new(source)).is_ok(), "{source}");
    }
}

#[cfg(unix)]
#[test]
fn lfa_unix_file_url_preserves_posix_control() {
    assert_eq!(
        resolve_local_file_url("FILE:/var/tmp/caf%C3%A9%20image.png"),
        Ok(Some(PathBuf::from("/var/tmp/café image.png"))),
    );
    assert_eq!(
        resolve_local_file_url("file:///var/tmp/file%3Astream.png"),
        Ok(Some(PathBuf::from("/var/tmp/file:stream.png"))),
    );
    assert_eq!(
        resolve_local_file_url("file:///C|/safe/file%7Cname.png"),
        Ok(Some(PathBuf::from("/C|/safe/file|name.png"))),
    );
}

#[cfg(windows)]
#[test]
fn lfa_windows_shared_owner_rejects_drive_less_and_alias_urls() {
    for source in [
        "file:///var/tmp/image.png",
        "file:///C:",
        "file:///C|",
        "file:///C|relative/image.png",
        "file:///C%7C/safe/image.png",
        "file:///C%7c/safe/image.png",
        "file:///C|/safe/image.png:stream",
        "file:///C|/safe/folder%2Fimage.png",
        "file:///C|/safe/folder%5Cimage.png",
        "file:///C|/safe/image%FF.png",
        "file:///C|/safe/image%00.png",
        "file:///C|/safe/image%.png",
        "file:///C%3A",
        "file:///C%3Arelative/image.png",
        "file:///C:/safe/file.txt:secret.png",
        "file:///C:/safe/file.txt%3Asecret.png",
        "file:////server/share/image.png",
        "file://localhost//server/share/image.png",
    ] {
        assert!(resolve_local_file_url(source).is_err(), "{source}");
    }
}

#[cfg(windows)]
#[test]
fn lfa_windows_path_guard_rejects_network_and_namespace_aliases() {
    for source in [
        r"\\server\share\image.png",
        "//server/share/image.png",
        r"/\server/share/image.png",
        r"\/server/share/image.png",
        r"\\.\C:\safe\image.png",
        r"\\?\UNC\server\share\image.png",
        r"\\?\GLOBALROOT\Device\HarddiskVolume1\image.png",
        r"\\?\C:",
        r"C:relative.png",
        r"C:\safe\image.png:secret",
        r"C:\safe\file:secret.png",
        r"\\?\C:\safe\image.png:secret",
        "relative:stream",
    ] {
        assert!(admit_local_media_path(Path::new(source)).is_err(), "{source}");
    }
}

#[cfg(windows)]
#[test]
fn lfa_windows_path_guard_accepts_rooted_drives() {
    for source in [
        r"C:\",
        r"C:\safe\image.png",
        "c:/safe/café Ω image.png",
        r"\\?\C:\",
        r"\\?\C:\safe\image.png",
    ] {
        assert!(admit_local_media_path(Path::new(source)).is_ok(), "{source}");
    }
}
