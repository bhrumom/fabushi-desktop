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

#[test]
fn test_ocrm_media_und_o09_echo_transcript_formatting() {
    // Default format
    assert_eq!(
        format_echo_transcript("Meeting notes", None),
        "📝 \"Meeting notes\""
    );

    // Custom format preserving dollar signs
    assert_eq!(
        format_echo_transcript("Cost is $100 and $200", Some("Transcription: [{transcript}]")),
        "Transcription: [Cost is $100 and $200]"
    );
}

#[test]
fn test_ocrm_media_und_o10_echo_transcript_channel_deliverability() {
    assert!(can_deliver_transcript_echo(Some("telegram"), Some("chat123")));
    assert!(can_deliver_transcript_echo(Some("discord"), Some("channel456")));
    assert!(can_deliver_transcript_echo(Some("slack"), Some("general")));

    // Non-deliverable or empty channels
    assert!(!can_deliver_transcript_echo(Some("cli"), Some("user")));
    assert!(!can_deliver_transcript_echo(Some("internal"), Some("dest")));
    assert!(!can_deliver_transcript_echo(Some(""), Some("dest")));
    assert!(!can_deliver_transcript_echo(None, Some("dest")));
    assert!(!can_deliver_transcript_echo(Some("telegram"), Some("")));
    assert!(!can_deliver_transcript_echo(Some("telegram"), None));
}

#[test]
fn test_ocrm_media_und_o11_attachment_path_normalization_and_guards() {
    // Normal paths
    assert_eq!(
        normalize_attachment_path(Some("/tmp/image.png")),
        Some("/tmp/image.png".to_string())
    );

    // file:// URLs
    assert_eq!(
        normalize_attachment_path(Some("file:///tmp/image.png")),
        Some("/tmp/image.png".to_string())
    );

    // Windows UNC network paths rejected
    assert_eq!(normalize_attachment_path(Some(r"\\server\share\file.png")), None);
    assert_eq!(normalize_attachment_path(Some("//server/share/file.png")), None);
    assert_eq!(normalize_attachment_path(Some(r"file://\\server\share\file.png")), None);

    // Whitespace / None
    assert_eq!(normalize_attachment_path(Some("   ")), None);
    assert_eq!(normalize_attachment_path(None), None);
}

#[test]
fn test_ocrm_media_und_o12_attachment_kind_resolution() {
    let mut att = NormalizedMediaAttachment {
        path: Some("/tmp/test.png".to_string()),
        url: None,
        mime: None,
        kind: None,
        file_name: None,
        workspace_dir: None,
        index: 0,
        already_transcribed: false,
    };
    assert_eq!(resolve_attachment_media_kind(&att), "image");
    assert!(is_image_attachment(&att));
    assert!(!is_audio_attachment(&att));

    // Sticker kind is treated as image
    att.kind = Some("sticker".to_string());
    assert_eq!(resolve_attachment_media_kind(&att), "image");

    // Audio by mime
    att.kind = None;
    att.path = Some("/tmp/unknown_ext".to_string());
    att.mime = Some("audio/ogg".to_string());
    assert_eq!(resolve_attachment_media_kind(&att), "audio");
    assert!(is_audio_attachment(&att));

    // Video by extension
    att.mime = None;
    att.path = Some("/tmp/recording.mp4".to_string());
    assert_eq!(resolve_attachment_media_kind(&att), "video");
}

#[test]
fn test_ocrm_media_und_o13_attachment_selection_policy_ordering_and_limits() {
    let att0 = NormalizedMediaAttachment {
        path: Some("/tmp/0.png".to_string()),
        url: None,
        mime: None,
        kind: Some("image".to_string()),
        file_name: None,
        workspace_dir: None,
        index: 0,
        already_transcribed: false,
    };
    let att1 = NormalizedMediaAttachment {
        path: None,
        url: Some("https://example.com/1.png".to_string()),
        mime: None,
        kind: Some("image".to_string()),
        file_name: None,
        workspace_dir: None,
        index: 1,
        already_transcribed: false,
    };
    let att2 = NormalizedMediaAttachment {
        path: Some("/tmp/2.png".to_string()),
        url: None,
        mime: None,
        kind: Some("image".to_string()),
        file_name: None,
        workspace_dir: None,
        index: 2,
        already_transcribed: false,
    };

    let items = vec![att0.clone(), att1.clone(), att2.clone()];

    // 1. Default policy: Mode=First -> selects index 0, drops [1, 2]
    let default_policy = AttachmentSelectionPolicy::default();
    let res = select_attachments(MediaUnderstandingCapability::Image, &items, &default_policy, None);
    assert_eq!(res.selected.len(), 1);
    assert_eq!(res.selected[0].index, 0);
    assert_eq!(res.dropped_attachment_indexes, vec![1, 2]);

    // 2. Prefer Last: selects index 2, drops [1, 0]
    let last_policy = AttachmentSelectionPolicy {
        prefer: Some(AttachmentOrderPreference::Last),
        mode: AttachmentSelectionMode::First,
        max_attachments: 1,
    };
    let res_last = select_attachments(MediaUnderstandingCapability::Image, &items, &last_policy, None);
    assert_eq!(res_last.selected.len(), 1);
    assert_eq!(res_last.selected[0].index, 2);
    assert_eq!(res_last.dropped_attachment_indexes, vec![1, 0]);

    // 3. Mode All with max_attachments = 2
    let all_policy = AttachmentSelectionPolicy {
        prefer: None,
        mode: AttachmentSelectionMode::All,
        max_attachments: 2,
    };
    let res_all = select_attachments(MediaUnderstandingCapability::Image, &items, &all_policy, None);
    assert_eq!(res_all.selected.len(), 2);
    assert_eq!(res_all.selected[0].index, 0);
    assert_eq!(res_all.selected[1].index, 1);
    assert_eq!(res_all.dropped_attachment_indexes, vec![2]);

    // 4. Audio preflight skip of already transcribed / decided indexes
    let audio_att = NormalizedMediaAttachment {
        path: Some("/tmp/audio.mp3".to_string()),
        url: None,
        mime: None,
        kind: Some("audio".to_string()),
        file_name: None,
        workspace_dir: None,
        index: 5,
        already_transcribed: true,
    };
    let res_audio = select_attachments(
        MediaUnderstandingCapability::Audio,
        &[audio_att],
        &default_policy,
        None,
    );
    assert!(res_audio.selected.is_empty());
}

#[test]
fn test_ocrm_media_und_o14_entry_capabilities_resolution_and_matching() {
    use std::collections::HashMap;

    let mut provider_registry = HashMap::new();
    provider_registry.insert(
        "acme".to_string(),
        MediaUnderstandingProviderEntry {
            id: "acme".to_string(),
            capabilities: Some(vec![
                MediaUnderstandingCapability::Audio,
                MediaUnderstandingCapability::Image,
            ]),
            shared_entry_capabilities: Some(vec![
                MediaUnderstandingCapability::Image,
                MediaUnderstandingCapability::Video,
            ]),
            inherit_configured_image_capability: false,
        },
    );

    // 1. Shared capabilities narrowed against effective capabilities -> ["image"]
    let entry = MediaUnderstandingModelConfig {
        provider: Some("acme".to_string()),
        model: Some("acme-model".to_string()),
        entry_type: None,
        command: None,
        capabilities: None,
    };
    let effective = resolve_effective_media_entry_capabilities(&entry, &provider_registry);
    assert_eq!(effective, Some(vec![MediaUnderstandingCapability::Image]));
    assert!(matches_media_entry_capability(
        &entry,
        MediaUnderstandingCapability::Image,
        &provider_registry
    ));
    assert!(!matches_media_entry_capability(
        &entry,
        MediaUnderstandingCapability::Audio,
        &provider_registry
    ));

    // 2. Explicitly configured capabilities take precedence
    let explicit_entry = MediaUnderstandingModelConfig {
        provider: Some("acme".to_string()),
        model: Some("acme-model".to_string()),
        entry_type: None,
        command: None,
        capabilities: Some(vec![MediaUnderstandingCapability::Video]),
    };
    let explicit_caps = resolve_effective_media_entry_capabilities(&explicit_entry, &provider_registry);
    assert_eq!(explicit_caps, Some(vec![MediaUnderstandingCapability::Video]));

    // 3. CLI entry suppresses capability inference
    let cli_entry = MediaUnderstandingModelConfig {
        provider: Some("acme".to_string()),
        model: None,
        entry_type: Some("cli".to_string()),
        command: Some("ffmpeg".to_string()),
        capabilities: None,
    };
    assert_eq!(
        resolve_effective_media_entry_capabilities(&cli_entry, &provider_registry),
        None
    );

    // 4. Unknown provider yields None
    let unknown_entry = MediaUnderstandingModelConfig {
        provider: Some("unknown".to_string()),
        model: None,
        entry_type: None,
        command: None,
        capabilities: None,
    };
    assert_eq!(
        resolve_effective_media_entry_capabilities(&unknown_entry, &provider_registry),
        None
    );
}



