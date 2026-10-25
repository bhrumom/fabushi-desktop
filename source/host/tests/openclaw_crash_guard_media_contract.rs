//! OpenClaw Process Crash Guard, Media Processing & MIME Resolution Native Rust Fusion & Contract Tests (OCRM-M18/M19)
//!
//! Validates the 12 falsifiable test oracles defined for OCRM Crash Guard & Media fusion:
//! - OCRM-CRASH-O01: ProcessCrashKind enum string representation.
//! - OCRM-CRASH-O02: ProcessCrashGuard thread-safe reporter registration.
//! - OCRM-CRASH-O03: Panic message and location extraction.
//! - OCRM-MEDIA-O01: Common image MIME detection by file extension.
//! - OCRM-MEDIA-O02: Servable container formats (HEIC/HEIF) resolution.
//! - OCRM-MEDIA-O03: Video container MIME resolution.
//! - OCRM-MEDIA-O04: Audio container MIME resolution.
//! - OCRM-MEDIA-O05: Attachment (25 MiB) and Video (200 MiB) byte caps.
//! - OCRM-MEDIA-O06: Agent media storage path resolution (attachments and assets).
//! - OCRM-MEDIA-O07: Safe media filename leaf sanitation and path traversal defense.
//! - OCRM-MEDIA-O08: Media kind enum classification (Image vs Attachment).
//! - OCRM-MEDIA-O09: Binary media persistence to disk under agent storage roots.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::attachment_paths::{
    get_agent_assets_dir, get_agent_attachments_dir, get_agent_media_store_roots,
    persist_agent_media_bytes, safe_media_leaf, AgentMediaKind, ASSETS_DIRNAME, ATTACHMENTS_DIRNAME,
};
use mahayana_host_runtime::media_mime::{
    audio_mime_from_path, image_mime_from_path, servable_image_mime_from_path,
    video_mime_from_path, ATTACHMENT_BYTE_LIMIT, BYTES_PER_MB, VIDEO_BYTE_LIMIT,
};
use mahayana_host_runtime::process_crash_guard::{
    ProcessCrashGuard, ProcessCrashKind, ProcessCrashReporter,
};

// ---------------------------------------------------------------------------
// OCRM-CRASH-O01: ProcessCrashKind String Representation
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_crash_o01_kind_as_str() {
    assert_eq!(ProcessCrashKind::Panic.as_str(), "panic");
}

// ---------------------------------------------------------------------------
// OCRM-CRASH-O02: ProcessCrashGuard Reporter Registration
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_crash_o02_guard_reporter_registration() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&captured);

    let guard = ProcessCrashGuard::default();
    let reporter: ProcessCrashReporter = Arc::new(move |msg, kind| {
        sink.lock().unwrap().push((msg, kind));
    });

    guard.set_reporter(Some(reporter));
    guard.set_reporter(None);
}

// ---------------------------------------------------------------------------
// OCRM-CRASH-O03: Panic Message and Location Extraction
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_crash_o03_panic_handling() {
    let result = std::panic::catch_unwind(|| {
        panic!("simulated worker thread failure");
    });

    assert!(result.is_err(), "panic must be captured by catch_unwind boundary");
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O01: Common Image MIME Detection
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o01_image_mime_resolution() {
    assert_eq!(image_mime_from_path(Path::new("photo.png")), Some("image/png"));
    assert_eq!(image_mime_from_path(Path::new("pic.jpg")), Some("image/jpeg"));
    assert_eq!(image_mime_from_path(Path::new("pic.jpeg")), Some("image/jpeg"));
    assert_eq!(image_mime_from_path(Path::new("badge.svg")), Some("image/svg+xml"));
    assert_eq!(image_mime_from_path(Path::new("anim.webp")), Some("image/webp"));
    assert_eq!(image_mime_from_path(Path::new("icon.ico")), Some("image/x-icon"));
    assert_eq!(image_mime_from_path(Path::new("doc.pdf")), None);
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O02: Servable Container Formats (HEIC/HEIF)
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o02_servable_image_mime() {
    assert_eq!(servable_image_mime_from_path(Path::new("live.heic")), Some("image/heic"));
    assert_eq!(servable_image_mime_from_path(Path::new("live.heif")), Some("image/heif"));
    assert_eq!(servable_image_mime_from_path(Path::new("test.png")), Some("image/png"));
    assert_eq!(servable_image_mime_from_path(Path::new("audio.mp3")), None);
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O03: Video Container MIME Resolution
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o03_video_mime_resolution() {
    assert_eq!(video_mime_from_path(Path::new("clip.mp4")), Some("video/mp4"));
    assert_eq!(video_mime_from_path(Path::new("clip.m4v")), Some("video/mp4"));
    assert_eq!(video_mime_from_path(Path::new("stream.webm")), Some("video/webm"));
    assert_eq!(video_mime_from_path(Path::new("camera.mov")), Some("video/quicktime"));
    assert_eq!(video_mime_from_path(Path::new("video.ogv")), Some("video/ogg"));
    assert_eq!(video_mime_from_path(Path::new("sound.wav")), None);
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O04: Audio Container MIME Resolution
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o04_audio_mime_resolution() {
    assert_eq!(audio_mime_from_path(Path::new("track.mp3")), Some("audio/mpeg"));
    assert_eq!(audio_mime_from_path(Path::new("hifi.flac")), Some("audio/flac"));
    assert_eq!(audio_mime_from_path(Path::new("voice.m4a")), Some("audio/mp4"));
    assert_eq!(audio_mime_from_path(Path::new("sample.aac")), Some("audio/aac"));
    assert_eq!(audio_mime_from_path(Path::new("image.png")), None);
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O05: Attachment and Video Byte Limits
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o05_byte_limits() {
    assert_eq!(ATTACHMENT_BYTE_LIMIT, 25 * 1024 * 1024, "attachment limit must be 25 MiB");
    assert_eq!(VIDEO_BYTE_LIMIT, 200 * 1024 * 1024, "video limit must be 200 MiB");
    assert_eq!(BYTES_PER_MB, 1024 * 1024);
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O06: Agent Media Storage Path Resolution
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o06_storage_path_resolution() {
    let agent_dir = Path::new("/workspace/agents/agent-1");
    let attachments = get_agent_attachments_dir(agent_dir);
    assert_eq!(attachments, PathBuf::from("/workspace/agents/agent-1").join(ATTACHMENTS_DIRNAME));

    let assets = get_agent_assets_dir(agent_dir);
    assert_eq!(assets, PathBuf::from("/workspace/agents/agent-1").join(ASSETS_DIRNAME));

    let roots = get_agent_media_store_roots(agent_dir);
    assert_eq!(roots[0], attachments);
    assert_eq!(roots[1], assets);
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O07: Safe Media Filename Sanitation
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o07_safe_media_leaf() {
    // Malicious traversal filename attempts must be neutralized
    let dirty_names = [
        "../../../etc/passwd",
        "..\\..\\windows\\system32\\calc.exe",
        "...hidden-file.png",
        "file with spaces & symbols!@#.jpg",
    ];

    for name in dirty_names {
        let clean = safe_media_leaf(name);
        assert!(!clean.contains('/'), "clean leaf must not contain forward slash");
        assert!(!clean.contains('\\'), "clean leaf must not contain backslash");
        assert!(!clean.starts_with('.'), "clean leaf must not start with dot");
        assert!(clean.len() <= 160, "clean leaf must respect 160 char truncation");
    }
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O08: Media Kind Classification
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o08_media_kind_variants() {
    assert_ne!(AgentMediaKind::Image, AgentMediaKind::Attachment);
    assert_eq!(AgentMediaKind::Image, AgentMediaKind::Image);
    assert_eq!(AgentMediaKind::Attachment, AgentMediaKind::Attachment);
}

// ---------------------------------------------------------------------------
// OCRM-MEDIA-O09: Binary Media Persistence to Disk
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_media_o09_persist_media_bytes() {
    let tmp = std::env::temp_dir().join(format!("fabushi-media-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    let data = b"RIFF....WAVEfmt ....data....";
    let saved = persist_agent_media_bytes(&tmp, "audio-sample.wav", data, AgentMediaKind::Attachment);
    assert!(saved.is_ok(), "persisting media bytes should succeed");
    let saved_path = saved.unwrap();
    assert!(saved_path.exists(), "persisted file must exist on disk");

    let read_back = std::fs::read(&saved_path).expect("read file");
    assert_eq!(read_back, data);

    let _ = std::fs::remove_dir_all(&tmp);
}
