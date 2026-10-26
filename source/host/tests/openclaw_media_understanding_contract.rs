use std::fs;
use std::path::{Path, PathBuf};

use mahayana_host_runtime::attachment_paths::get_agent_attachments_dir;
use mahayana_host_runtime::extensions::attachments::attachments_service::read_attachment_text_with_root;
use mahayana_host_runtime::extensions::attachments::media_understanding_service::*;

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("test_und_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn test_ocrm_media_und_o01_stt_transcript_artifact_rejection() {
    // Artifact prompt echoes are recognized and rejected
    assert!(is_transcript_artifact_text(""));
    assert!(is_transcript_artifact_text("   "));
    assert!(is_transcript_artifact_text("context:"));
    assert!(is_transcript_artifact_text("  CONTEXT:  "));
    assert!(is_transcript_artifact_text("###"));
    assert!(is_transcript_artifact_text("transcribe the audio"));
    assert!(is_transcript_artifact_text("Transcribe  The Audio."));

    // Genuine speech transcriptions are accepted
    assert!(!is_transcript_artifact_text("Hello, how are you?"));
    assert!(!is_transcript_artifact_text("This is an important meeting recording."));
    assert!(!is_transcript_artifact_text("Context matters in machine learning."));
}

#[test]
fn test_ocrm_media_und_o02_audio_transcription_upload_filename() {
    // 1. Raw .aac file mapped to .m4a
    assert_eq!(
        resolve_audio_transcription_upload_file_name(Some("voice_note.aac"), None),
        "voice_note.m4a"
    );
    assert_eq!(
        resolve_audio_transcription_upload_file_name(Some("/tmp/nested/recording.AAC"), None),
        "recording.m4a"
    );

    // 2. Extensionless file with audio/aac mime
    assert_eq!(
        resolve_audio_transcription_upload_file_name(Some("recording"), Some("audio/aac")),
        "recording.m4a"
    );

    // 3. Standard audio files retain their original name
    assert_eq!(
        resolve_audio_transcription_upload_file_name(Some("lecture.mp3"), Some("audio/mp3")),
        "lecture.mp3"
    );
    assert_eq!(
        resolve_audio_transcription_upload_file_name(Some("sample.wav"), Some("audio/wav")),
        "sample.wav"
    );

    // 4. Fallback defaults
    assert_eq!(
        resolve_audio_transcription_upload_file_name(None, None),
        "audio"
    );
}

#[test]
fn test_ocrm_media_und_o03_openai_auth_headers() {
    let headers = build_open_ai_compatible_auth_headers(Some("sk-test-secret-12345")).unwrap();
    assert_eq!(headers.get("authorization").map(String::as_str), Some("Bearer sk-test-secret-12345"));

    assert!(build_open_ai_compatible_auth_headers(None).is_none());
    assert!(build_open_ai_compatible_auth_headers(Some("   ")).is_none());
}

#[test]
fn test_ocrm_media_und_o04_capability_order_and_voice_note_placeholder() {
    assert_eq!(
        CAPABILITY_ORDER,
        &[
            MediaUnderstandingCapability::Image,
            MediaUnderstandingCapability::Audio,
            MediaUnderstandingCapability::Video,
        ]
    );

    assert_eq!(
        EMPTY_VOICE_NOTE_PLACEHOLDER,
        "[Voice note could not be transcribed because the audio attachment was too small]"
    );
}

#[test]
fn test_ocrm_media_und_o05_append_file_blocks() {
    assert_eq!(append_file_blocks("Base prompt", Some("File block 1")), "Base prompt\n\nFile block 1");
    assert_eq!(append_file_blocks("", Some("File block 1")), "File block 1");
    assert_eq!(append_file_blocks("Base prompt", None), "Base prompt");
    assert_eq!(append_file_blocks("Base prompt", Some("   ")), "Base prompt");
}

#[test]
fn test_ocrm_media_und_o06_audio_input_budget() {
    let budget = AudioInputBudget::default();
    assert_eq!(budget.max_bytes, 25 * 1024 * 1024);
    assert!(budget.validate_payload_size(10 * 1024 * 1024).is_ok());
    assert!(budget.validate_payload_size(26 * 1024 * 1024).is_err());
}

#[test]
fn test_ocrm_media_und_o07_media_store_url_classification() {
    assert!(is_media_store_url("media://attachments/audio-1.mp3"));
    assert!(is_media_store_url("MEDIA://STORE/file.png"));
    assert!(!is_media_store_url("https://example.com/audio.mp3"));
    assert!(!is_media_store_url("file:///tmp/audio.mp3"));
}

#[test]
fn test_ocrm_media_und_o08_attachment_service_artifact_text_suppression() {
    let sand_root = temp_root("artifact_suppress");
    let agent_dir = sand_root.join("agent_suppress");
    let attachments_dir = get_agent_attachments_dir(&agent_dir);
    fs::create_dir_all(&attachments_dir).unwrap();

    // 1. Text file with STT artifact echo "###" is suppressed (returns None)
    let artifact_file = attachments_dir.join("echo.txt");
    fs::write(&artifact_file, b"###\n").unwrap();
    let preview = read_attachment_text_with_root(&sand_root, &agent_dir, &artifact_file);
    assert!(preview.is_none(), "STT prompt echo must be suppressed from transcript preview");

    // 2. Normal text file is returned cleanly
    let real_file = attachments_dir.join("real.txt");
    fs::write(&real_file, b"Actual transcript content here").unwrap();
    let real_preview = read_attachment_text_with_root(&sand_root, &agent_dir, &real_file);
    assert!(real_preview.is_some());
}
