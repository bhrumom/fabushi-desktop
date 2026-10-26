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

pub const DEFAULT_ECHO_TRANSCRIPT_FORMAT: &str = "📝 \"{transcript}\"";

/// Formats the transcript into the echo template, preserving literal dollar sequences.
pub fn format_echo_transcript(transcript: &str, format: Option<&str>) -> String {
    let template = format.unwrap_or(DEFAULT_ECHO_TRANSCRIPT_FORMAT);
    template.replace("{transcript}", transcript)
}

/// Checks if a message channel is a valid deliverable destination.
pub fn is_deliverable_message_channel(channel: &str) -> bool {
    let normalized = channel.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "discord" | "telegram" | "whatsapp" | "slack" | "signal" | "imessage" | "wechat"
        | "mattermost" | "matrix" | "teams" => true,
        _ => false,
    }
}

/// Evaluates if echo transcript delivery should proceed based on destination context.
pub fn can_deliver_transcript_echo(channel: Option<&str>, to: Option<&str>) -> bool {
    let chan = match channel {
        Some(c) if !c.trim().is_empty() => c.trim(),
        _ => return false,
    };
    let dest = match to {
        Some(t) if !t.trim().is_empty() => t.trim(),
        _ => return false,
    };
    !dest.is_empty() && is_deliverable_message_channel(chan)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedMediaAttachment {
    pub path: Option<String>,
    pub url: Option<String>,
    pub mime: Option<String>,
    pub kind: Option<String>,
    pub file_name: Option<String>,
    pub workspace_dir: Option<String>,
    pub index: usize,
    pub already_transcribed: bool,
}

/// Normalizes local attachment paths while rejecting Windows UNC network paths and remote file schemes.
pub fn normalize_attachment_path(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }
    // Reject Windows UNC network paths (e.g. \\server\share or //server/share)
    if raw.starts_with(r"\\") || raw.starts_with("//") {
        return None;
    }
    if raw.to_ascii_lowercase().starts_with("file://") {
        let stripped = &raw[7..];
        if stripped.starts_with(r"\\") || stripped.starts_with("//") {
            return None;
        }
        return Some(stripped.to_string());
    }
    Some(raw.to_string())
}

/// Classifies an attachment by authoritative kind, MIME, then filename extension.
pub fn resolve_attachment_media_kind(attachment: &NormalizedMediaAttachment) -> &'static str {
    if let Some(ref k) = attachment.kind {
        let lk = k.to_ascii_lowercase();
        match lk.as_str() {
            "image" | "sticker" => return "image",
            "audio" => return "audio",
            "video" => return "video",
            "document" => return "unknown",
            _ => {}
        }
    }
    if let Some(ref m) = attachment.mime {
        let lm = m.to_ascii_lowercase();
        if lm.starts_with("image/") {
            return "image";
        }
        if lm.starts_with("audio/") {
            return "audio";
        }
        if lm.starts_with("video/") {
            return "video";
        }
    }
    if let Some(ref p) = attachment.path.as_deref().or(attachment.file_name.as_deref()) {
        let path = Path::new(p);
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let lext = ext.to_ascii_lowercase();
            match lext.as_str() {
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" => return "image",
                "mp3" | "wav" | "m4a" | "aac" | "ogg" | "flac" => return "audio",
                "mp4" | "mov" | "webm" | "mkv" | "avi" => return "video",
                _ => {}
            }
        }
    }
    "unknown"
}

pub fn is_audio_attachment(attachment: &NormalizedMediaAttachment) -> bool {
    resolve_attachment_media_kind(attachment) == "audio"
}

pub fn is_image_attachment(attachment: &NormalizedMediaAttachment) -> bool {
    resolve_attachment_media_kind(attachment) == "image"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentOrderPreference {
    First,
    Last,
    Path,
    Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentSelectionMode {
    First,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentSelectionPolicy {
    pub prefer: Option<AttachmentOrderPreference>,
    pub mode: AttachmentSelectionMode,
    pub max_attachments: usize,
}

impl Default for AttachmentSelectionPolicy {
    fn default() -> Self {
        Self {
            prefer: None,
            mode: AttachmentSelectionMode::First,
            max_attachments: 1,
        }
    }
}

pub struct AttachmentSelectionResult {
    pub selected: Vec<NormalizedMediaAttachment>,
    pub dropped_attachment_indexes: Vec<usize>,
}

pub fn select_attachments(
    capability: MediaUnderstandingCapability,
    attachments: &[NormalizedMediaAttachment],
    policy: &AttachmentSelectionPolicy,
    decided_audio_indexes: Option<&[usize]>,
) -> AttachmentSelectionResult {
    let target_kind = capability.as_str();
    let decided_set: std::collections::HashSet<usize> = decided_audio_indexes
        .map(|s| s.iter().copied().collect())
        .unwrap_or_default();

    let mut matches: Vec<NormalizedMediaAttachment> = attachments
        .iter()
        .filter(|att| {
            if capability == MediaUnderstandingCapability::Audio
                && (att.already_transcribed || decided_set.contains(&att.index))
            {
                return false;
            }
            resolve_attachment_media_kind(att) == target_kind
        })
        .cloned()
        .collect();

    if matches.is_empty() {
        return AttachmentSelectionResult {
            selected: Vec::new(),
            dropped_attachment_indexes: Vec::new(),
        };
    }

    match policy.prefer {
        Some(AttachmentOrderPreference::Last) => {
            matches.reverse();
        }
        Some(AttachmentOrderPreference::Path) => {
            matches.sort_by_key(|att| if att.path.is_some() { 0 } else { 1 });
        }
        Some(AttachmentOrderPreference::Url) => {
            matches.sort_by_key(|att| if att.url.is_some() { 0 } else { 1 });
        }
        _ => {}
    }

    let limit = match policy.mode {
        AttachmentSelectionMode::All => policy.max_attachments.max(1),
        AttachmentSelectionMode::First => 1,
    };

    let selected: Vec<NormalizedMediaAttachment> = matches.iter().take(limit).cloned().collect();
    let dropped: Vec<usize> = matches.iter().skip(limit).map(|att| att.index).collect();

    AttachmentSelectionResult {
        selected,
        dropped_attachment_indexes: dropped,
    }
}


