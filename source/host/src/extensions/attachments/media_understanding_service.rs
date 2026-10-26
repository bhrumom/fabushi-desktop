use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

pub const EMPTY_VOICE_NOTE_PLACEHOLDER: &str =
    "[Voice note could not be transcribed because the audio attachment was too small]";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaUnderstandingCapability {
    Image,
    Audio,
    Video,
}

impl MediaUnderstandingCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
        }
    }
}

pub const CAPABILITY_ORDER: &[MediaUnderstandingCapability] = &[
    MediaUnderstandingCapability::Image,
    MediaUnderstandingCapability::Audio,
    MediaUnderstandingCapability::Video,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MediaUnderstandingDecisionOutcome {
    Success,
    Failed,
    Skipped,
    Disabled,
    NoAttachment,
    ScopeDeny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MediaAttachmentDisposition {
    Handled,
    HandedToNativeVision,
    NotSelected,
    CapabilityDisabled,
    NoModel,
    ScopeDenied,
    Failed { reason: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaAttachmentProcessing {
    Completed,
    Omitted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaUnderstandingModelDecision {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub requested_backend: Option<String>,
    pub observed_backend: Option<String>,
    pub decision_type: String, // "provider" or "cli"
    pub outcome: String,       // "success", "skipped", "failed"
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaUnderstandingDecision {
    pub capability: MediaUnderstandingCapability,
    pub outcome: MediaUnderstandingDecisionOutcome,
    pub attachment_dispositions: HashMap<usize, MediaAttachmentDisposition>,
    pub attachment_processing: HashMap<usize, MediaAttachmentProcessing>,
    pub native_vision_active: bool,
}

/// Empty STT prompt echoes are not speech; match whole utterances, never prose fragments.
pub fn is_transcript_artifact_text(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    let normalized = words.join(" ").to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "" | "context:" | "###" | "transcribe the audio" | "transcribe the audio."
    )
}

/// Resolves the multipart upload filename, mapping AAC inputs to provider-friendly `.m4a`.
pub fn resolve_audio_transcription_upload_file_name(
    file_name: Option<&str>,
    mime: Option<&str>,
) -> String {
    let trimmed = file_name.map(str::trim).filter(|s| !s.is_empty());
    let base_name = match trimmed {
        Some(t) => Path::new(t)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("audio"),
        None => "audio",
    };

    let lower_mime = mime.map(|m| m.trim().to_ascii_lowercase());

    if base_name.to_ascii_lowercase().ends_with(".aac") {
        let stem = &base_name[..base_name.len() - 4];
        let prefix = if stem.is_empty() { "audio" } else { stem };
        return format!("{prefix}.m4a");
    }

    if Path::new(base_name).extension().is_none() && lower_mime.as_deref() == Some("audio/aac") {
        let prefix = if base_name.is_empty() { "audio" } else { base_name };
        return format!("{prefix}.m4a");
    }

    base_name.to_string()
}

pub fn build_open_ai_compatible_auth_headers(
    api_key: Option<&str>,
) -> Option<HashMap<String, String>> {
    let key = api_key?.trim();
    if key.is_empty() {
        return None;
    }
    let mut map = HashMap::new();
    map.insert("authorization".to_string(), format!("Bearer {key}"));
    Some(map)
}

pub fn append_file_blocks(body: &str, suffix: Option<&str>) -> String {
    let suf = match suffix {
        Some(s) => s.trim(),
        None => return body.to_string(),
    };
    if suf.is_empty() {
        return body.to_string();
    }
    let base = body.trim();
    if base.is_empty() {
        suf.to_string()
    } else {
        format!("{base}\n\n{suf}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioInputBudget {
    pub max_bytes: usize,
    pub max_duration_seconds: u32,
}

impl Default for AudioInputBudget {
    fn default() -> Self {
        Self {
            max_bytes: 25 * 1024 * 1024, // 25 MiB standard limit
            max_duration_seconds: 600,    // 10 minutes
        }
    }
}

impl AudioInputBudget {
    pub fn validate_payload_size(&self, byte_count: usize) -> Result<(), String> {
        if byte_count > self.max_bytes {
            Err(format!(
                "Audio payload size {byte_count} exceeds maximum allowed budget {}",
                self.max_bytes
            ))
        } else {
            Ok(())
        }
    }
}

pub fn is_media_store_url(url: &str) -> bool {
    let trimmed = url.trim().to_ascii_lowercase();
    trimmed.starts_with("media://")
}
