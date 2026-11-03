use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::connectors::channel_attachment::{
    CHANNEL_ATTACHMENT_MAX_UPLOAD_BYTES, GENERIC_BINARY_MIME, ResolvedChannelAttachment,
    resolve_channel_attachment, to_local_channel_attachment_path, url_looks_like_image,
};

fn scratch(label: &str) -> std::path::PathBuf {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("fabushi-channel-attachment-{label}-{}-{n}", std::process::id()))
}

#[test]
fn remote_urls_are_forwarded_without_host_fetching() {
    let image = resolve_channel_attachment(Some("https://example.com/a.PNG?x=1")).unwrap();
    assert_eq!(image, ResolvedChannelAttachment::Url {
        is_image: true,
        url: "https://example.com/a.PNG?x=1".into(),
    });
    let file = resolve_channel_attachment(Some("http://example.com/archive.bin")).unwrap();
    assert_eq!(file, ResolvedChannelAttachment::Url {
        is_image: false,
        url: "http://example.com/archive.bin".into(),
    });
    assert!(url_looks_like_image("https://example.com/photo.webp"));
    assert!(!url_looks_like_image("not-a-url"));
}

#[test]
fn file_urls_and_plain_paths_upload_bytes_with_frozen_mime_rules() {
    let root = scratch("upload");
    fs::create_dir_all(&root).unwrap();

    let image_path = root.join("shot.jpeg");
    fs::write(&image_path, b"jpeg").unwrap();
    let file_url = url::Url::from_file_path(&image_path).unwrap().to_string();
    assert_eq!(to_local_channel_attachment_path(&file_url), Some(image_path.clone()));
    match resolve_channel_attachment(Some(&file_url)).unwrap() {
        ResolvedChannelAttachment::Upload { is_image, bytes, filename, mime } => {
            assert!(is_image);
            assert_eq!(bytes, b"jpeg");
            assert_eq!(filename, "shot.jpeg");
            assert_eq!(mime, "image/jpeg");
        }
        other => panic!("unexpected attachment: {other:?}"),
    }

    let binary_path = root.join("blob.dat");
    fs::write(&binary_path, b"payload").unwrap();
    match resolve_channel_attachment(binary_path.to_str()).unwrap() {
        ResolvedChannelAttachment::Upload { is_image, bytes, mime, .. } => {
            assert!(!is_image);
            assert_eq!(bytes, b"payload");
            assert_eq!(mime, GENERIC_BINARY_MIME);
        }
        other => panic!("unexpected attachment: {other:?}"),
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_directory_zero_and_oversize_local_inputs_fail_closed() {
    let root = scratch("limits");
    fs::create_dir_all(&root).unwrap();
    assert!(resolve_channel_attachment(root.to_str()).is_none());

    let empty = root.join("empty.bin");
    fs::write(&empty, b"").unwrap();
    assert!(resolve_channel_attachment(empty.to_str()).is_none());

    let huge = root.join("huge.bin");
    let file = fs::File::create(&huge).unwrap();
    file.set_len(CHANNEL_ATTACHMENT_MAX_UPLOAD_BYTES + 1).unwrap();
    assert!(resolve_channel_attachment(huge.to_str()).is_none());

    assert!(resolve_channel_attachment(Some("ftp://example.com/a.png")).is_none());
    assert!(resolve_channel_attachment(Some("")).is_none());
    assert!(to_local_channel_attachment_path("https://example.com/x").is_none());

    fs::remove_dir_all(root).unwrap();
}


#[test]
fn frozen_shared_media_inventory_drives_channel_classification() {
    assert!(url_looks_like_image("https://example.com/favicon.ICO"));
    assert!(!url_looks_like_image("https://example.com/photo.heic"));

    let root = scratch("media-inventory");
    fs::create_dir_all(&root).unwrap();

    let heif = root.join("native.heif");
    fs::write(&heif, b"native").unwrap();
    match resolve_channel_attachment(heif.to_str()).unwrap() {
        ResolvedChannelAttachment::Upload { is_image, mime, .. } => {
            assert!(!is_image);
            assert_eq!(mime, GENERIC_BINARY_MIME);
        }
        other => panic!("unexpected attachment: {other:?}"),
    }

    let ogv = root.join("clip.ogv");
    fs::write(&ogv, b"video").unwrap();
    match resolve_channel_attachment(ogv.to_str()).unwrap() {
        ResolvedChannelAttachment::Upload { is_image, mime, .. } => {
            assert!(!is_image);
            assert_eq!(mime, "video/ogg");
        }
        other => panic!("unexpected attachment: {other:?}"),
    }

    let mkv = root.join("clip.mkv");
    fs::write(&mkv, b"video").unwrap();
    match resolve_channel_attachment(mkv.to_str()).unwrap() {
        ResolvedChannelAttachment::Upload { mime, .. } => {
            assert_eq!(mime, GENERIC_BINARY_MIME);
        }
        other => panic!("unexpected attachment: {other:?}"),
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lfa_channel_native_url_variants_preserve_bytes_and_metadata() {
    let root = scratch("lfa-native");
    fs::create_dir_all(&root).expect("native fixture directory");
    let name = "café Ω image.PNG";
    let path = root.join(name);
    let bytes = b"unchanged channel attachment bytes";
    fs::write(&path, bytes).expect("write source bytes");
    let url = url::Url::from_file_path(&path).expect("native absolute file URL").to_string();
    let expected = ResolvedChannelAttachment::Upload {
        is_image: true,
        bytes: bytes.to_vec(),
        filename: name.into(),
        mime: "image/png".into(),
    };
    for source in [
        url.clone(),
        url.replacen("file://", "FILE:", 1),
        url.replacen("file://", "file://localhost", 1),
        format!("\u{001f}f\ti\rL\ne:{}\n", &url[5..]),
        format!("{url}?signature=%2F#preview%5C"),
    ] {
        assert_eq!(to_local_channel_attachment_path(&source), Some(path.clone()));
        assert_eq!(resolve_channel_attachment(Some(&source)), Some(expected.clone()));
    }
    assert_eq!(fs::read(&path).expect("source retained"), bytes);
    fs::remove_dir_all(root).expect("remove native fixture");
}

#[test]
fn lfa_channel_invalid_file_urls_never_become_local_paths() {
    // Pure conversion only for remote authorities: no metadata/open of a share.
    for source in [
        "file://server/share/image.png",
        "file://127.0.0.1/share/image.png",
        "file://server/C:/safe/image.png",
        "\u{001f}f\tiLe://localhost:99/image.png",
        "file://[invalid/image.png",
    ] {
        assert!(to_local_channel_attachment_path(source).is_none(), "{source:?}");
    }
}

#[test]
fn lfa_channel_invalid_encoding_preserves_source_and_valid_retry() {
    let root = scratch("lfa-encoding");
    fs::create_dir_all(&root).expect("native fixture directory");
    let name = "literal%GG.png";
    let path = root.join(name);
    let bytes = b"literal percent source must remain intact";
    fs::write(&path, bytes).expect("write literal percent source");
    let url = url::Url::from_file_path(&path).expect("native absolute file URL").to_string();
    assert!(url.contains("%25GG"));
    let invalid = url.replacen("%25GG", "%GG", 1);
    assert!(to_local_channel_attachment_path(&invalid).is_none());
    for _ in 0..2 {
        assert!(resolve_channel_attachment(Some(&invalid)).is_none());
        assert_eq!(resolve_channel_attachment(Some(&url)), Some(ResolvedChannelAttachment::Upload {
            is_image: true,
            bytes: bytes.to_vec(),
            filename: name.into(),
            mime: "image/png".into(),
        }));
    }
    assert_eq!(fs::read(&path).expect("source retained"), bytes);
    fs::remove_dir_all(root).expect("remove percent fixture");
}

#[cfg(windows)]
#[test]
fn lfa_windows_channel_paths_reject_network_and_alias_spellings() {
    for source in [
        r"\\server\share\image.png",
        r"/\server/share/image.png",
        r"\\?\UNC\server\share\image.png",
        r"\\.\C:\safe\image.png",
        r"C:\safe\file.txt:secret.png",
        "file:///C:/safe/file.txt%3Asecret.png",
        "file:///var/tmp/image.png",
    ] {
        assert!(to_local_channel_attachment_path(source).is_none(), "{source}");
    }
}
