use std::collections::{HashMap, HashSet};
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::extensions::attachments::input_files::{
    resolve_input_file_limits, InputFileLimits, InputFileLimitsConfig,
};
use crate::extensions::attachments::media_file_context::{
    render_document_truncation_notice, render_file_context_block, DocumentExtractionMetadata,
    RenderFileContextParams,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaUnderstandingModelConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub entry_type: Option<String>, // "provider" or "cli"
    pub command: Option<String>,
    pub capabilities: Option<Vec<MediaUnderstandingCapability>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaUnderstandingProviderEntry {
    pub id: String,
    pub capabilities: Option<Vec<MediaUnderstandingCapability>>,
    pub shared_entry_capabilities: Option<Vec<MediaUnderstandingCapability>>,
    pub inherit_configured_image_capability: bool,
}

pub fn resolve_configured_media_entry_capabilities(
    entry: &MediaUnderstandingModelConfig,
) -> Option<Vec<MediaUnderstandingCapability>> {
    let caps = entry.capabilities.as_ref()?;
    if caps.is_empty() {
        None
    } else {
        Some(caps.clone())
    }
}

pub fn resolve_effective_media_entry_capabilities(
    entry: &MediaUnderstandingModelConfig,
    provider_registry: &HashMap<String, MediaUnderstandingProviderEntry>,
) -> Option<Vec<MediaUnderstandingCapability>> {
    if let Some(configured) = resolve_configured_media_entry_capabilities(entry) {
        return Some(configured);
    }
    let is_cli = entry.entry_type.as_deref() == Some("cli")
        || (entry.entry_type.is_none() && entry.command.is_some());
    if is_cli {
        return None;
    }
    let provider_id = entry.provider.as_deref()?.trim().to_ascii_lowercase();
    if provider_id.is_empty() {
        return None;
    }
    let provider = provider_registry.get(&provider_id)?;
    match &provider.shared_entry_capabilities {
        None => provider.capabilities.clone(),
        Some(shared) => {
            let effective = provider.capabilities.as_ref()?;
            let filtered: Vec<MediaUnderstandingCapability> = shared
                .iter()
                .filter(|c| effective.contains(c))
                .copied()
                .collect();
            Some(filtered)
        }
    }
}

pub fn matches_media_entry_capability(
    entry: &MediaUnderstandingModelConfig,
    capability: MediaUnderstandingCapability,
    provider_registry: &HashMap<String, MediaUnderstandingProviderEntry>,
) -> bool {
    resolve_effective_media_entry_capabilities(entry, provider_registry)
        .map(|caps| caps.contains(&capability))
        .unwrap_or(false)
}

// =========================================================================
// Unit 33: File Context & Extraction Limits (OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001 & FILE-CONTEXT-001)
// =========================================================================

pub const INBOUND_FILE_EXTRACTION_DEFAULT_MAX_MB: usize = 20;
pub const INBOUND_FILE_EXTRACTION_MAX_BYTES_CAP: usize = 25 * 1024 * 1024;
pub const INBOUND_FILE_EXTRACTION_DEFAULT_MAX_PAGES: usize = 20;
pub const INBOUND_FILE_EXTRACTION_MAX_PAGES_CAP: usize = 150;

#[derive(Debug, Clone, Default)]
pub struct FileExtractionLimitsConfig {
    pub media_max_mb: Option<usize>,
    pub pdf_max_pages: Option<usize>,
    pub files: Option<InputFileLimitsConfig>,
}

#[derive(Debug, Clone)]
pub struct FileExtractionLimits {
    pub limits: InputFileLimits,
    pub allowed_mimes_configured: bool,
}

pub fn resolve_file_extraction_limits(config: &FileExtractionLimitsConfig) -> FileExtractionLimits {
    let allowed_mimes_configured = config
        .files
        .as_ref()
        .and_then(|f| f.allowed_mimes.as_ref())
        .map_or(false, |m| !m.is_empty());

    let max_mb = config.media_max_mb.unwrap_or(INBOUND_FILE_EXTRACTION_DEFAULT_MAX_MB);
    let max_pages = config.pdf_max_pages.unwrap_or(INBOUND_FILE_EXTRACTION_DEFAULT_MAX_PAGES);

    let mut inbound_files = config.files.clone().unwrap_or_default();
    if inbound_files.max_bytes.is_none() {
        inbound_files.max_bytes = Some(std::cmp::min(
            max_mb * 1024 * 1024,
            INBOUND_FILE_EXTRACTION_MAX_BYTES_CAP,
        ));
    }
    let pdf_cfg = inbound_files.pdf.get_or_insert_with(Default::default);
    if pdf_cfg.max_pages.is_none() {
        pdf_cfg.max_pages = Some(std::cmp::min(
            max_pages,
            INBOUND_FILE_EXTRACTION_MAX_PAGES_CAP,
        ));
    }

    let resolved = resolve_input_file_limits(Some(&inbound_files));
    FileExtractionLimits {
        limits: resolved,
        allowed_mimes_configured,
    }
}

/// Extracted image payload from a document attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentExtractedImage {
    pub r#type: String,
    pub data: String,
    pub mime_type: String,
}

/// Transient image payload extracted from an inbound file attachment with index tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedFileImage {
    pub r#type: String,
    pub data: String,
    pub mime_type: String,
    pub attachment_index: usize,
}

pub fn strip_extracted_file_image_metadata(img: ExtractedFileImage) -> DocumentExtractedImage {
    DocumentExtractedImage {
        r#type: "image".to_string(),
        data: img.data,
        mime_type: img.mime_type,
    }
}

pub const MAX_SKIPPED_FILE_MARKERS: usize = 5;

pub fn render_skipped_file_overflow_summary(count: usize) -> String {
    format!("[{count} more attachment{} skipped]", if count == 1 { "" } else { "s" })
}

pub fn render_media_attachment_disposition(
    capability: MediaUnderstandingCapability,
    disposition: &MediaAttachmentDisposition,
) -> Option<String> {
    let cap_str = match capability {
        MediaUnderstandingCapability::Audio => "Audio",
        MediaUnderstandingCapability::Image => "Image",
        MediaUnderstandingCapability::Video => "Video",
    };
    let cap_lower = match capability {
        MediaUnderstandingCapability::Audio => "audio",
        MediaUnderstandingCapability::Image => "image",
        MediaUnderstandingCapability::Video => "video",
    };
    match disposition {
        MediaAttachmentDisposition::Handled | MediaAttachmentDisposition::HandedToNativeVision => None,
        MediaAttachmentDisposition::NotSelected => {
            Some(format!("[{cap_str} attachment not processed: attachment limit reached]"))
        }
        MediaAttachmentDisposition::CapabilityDisabled => {
            Some(format!("[{cap_str} attachment not analyzed: {cap_lower} understanding is disabled]"))
        }
        MediaAttachmentDisposition::NoModel => {
            Some(format!("[{cap_str} attachment not analyzed: no {cap_lower}-understanding model is configured]"))
        }
        MediaAttachmentDisposition::ScopeDenied => {
            Some(format!("[{cap_str} attachment not analyzed in this chat]"))
        }
        MediaAttachmentDisposition::Failed { .. } => {
            Some(format!("[{cap_str} attachment could not be analyzed]"))
        }
    }
}

pub struct AttachmentContextBlock {
    pub text: String,
    pub consumes_marker_budget: bool,
}

pub fn apply_attachment_marker_budget(blocks: &[AttachmentContextBlock]) -> Vec<String> {
    let mut rendered = Vec::new();
    let mut markers = 0;
    let mut overflow = 0;
    for block in blocks {
        if block.consumes_marker_budget && markers >= MAX_SKIPPED_FILE_MARKERS {
            overflow += 1;
            continue;
        }
        if block.consumes_marker_budget {
            markers += 1;
        }
        rendered.push(block.text.clone());
    }
    if overflow > 0 {
        rendered.push(render_skipped_file_overflow_summary(overflow));
    }
    rendered
}

static MIME_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    let mime_type = r"([a-zA-Z0-9!#$&^_.+-]+/[a-zA-Z0-9!#$&^_.+-]+)";
    let http_token = r"[a-zA-Z0-9!#$%&'*+.^_\`|~-]+";
    let http_quoted_string = r#""(?:[\t !#-\[\]-~]|\\[\t -~])*""#;
    let pattern = format!(
        r"(?i)^{mime_type}(?:[ \t]*;[ \t]*{http_token}=(?:{http_token}|{http_quoted_string}))*$"
    );
    regex::Regex::new(&pattern).expect("valid mime regex")
});

pub fn sanitize_mime_type(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let caps = MIME_REGEX.captures(trimmed)?;
    caps.get(1).map(|m| m.as_str().to_ascii_lowercase())
}

pub const MARKER_MIME_MAX_CHARS: usize = 100;

pub fn marker_safe_mime(value: Option<&str>) -> Option<String> {
    let mime = sanitize_mime_type(value)?;
    if mime.len() <= MARKER_MIME_MAX_CHARS {
        Some(mime)
    } else {
        None
    }
}

static PATH_SAFE_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"^[\p{L}\p{M}\p{N} /\\:._-]+$").expect("valid path safe regex")
});

pub const MARKER_LOCAL_PATH_MAX_CHARS: usize = 300;

pub fn marker_safe_local_path(
    value: Option<&str>,
    allow_workspace_relative: bool,
) -> Option<String> {
    let val = value?.trim();
    if val.is_empty() || val.chars().count() > MARKER_LOCAL_PATH_MAX_CHARS {
        return None;
    }
    let is_posix_abs = val.starts_with('/');
    let is_win_abs = val.len() >= 3
        && val.chars().next().map_or(false, |c| c.is_ascii_alphabetic())
        && val.as_bytes()[1] == b':'
        && val.as_bytes()[2] == b'\\';
    let is_absolute = is_posix_abs || is_win_abs;

    if !is_absolute {
        if !allow_workspace_relative || val.contains('\\') {
            return None;
        }
        for segment in val.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return None;
            }
        }
    }

    if PATH_SAFE_REGEX.is_match(val) {
        Some(val.to_string())
    } else {
        None
    }
}

static MARKER_SANITIZED_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"(?i)<<<\s*EXTERNAL[\s_]+UNTRUSTED[\s_]+CONTENT(?:\s+id=\\*"[^"]*")?\s*>>>"#)
        .expect("valid regex")
});

static END_MARKER_SANITIZED_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"(?i)<<<\s*END[\s_]+EXTERNAL[\s_]+UNTRUSTED[\s_]+CONTENT(?:\s+id=\\*"[^"]*")?\s*>>>"#)
        .expect("valid regex")
});

static SPECIAL_TOKENS_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)<\|im_start\|>|<\|im_end\|>|<\|endoftext\|>|<\|begin_of_text\|>|<\|end_of_text\|>|<\|start_header_id\|>|<\|end_header_id\|>|<\|eot_id\|>|<\|python_tag\|>|<\|eom_id\|>|\[INST\]|\[/INST\]|<<SYS>>|<</SYS>>|<s>|</s>|<\|channel\|>|<\|message\|>|<\|return\|>|<\|call\|>|<start_of_turn>|<end_of_turn>|<\|reserved_special_token_\d+\|>")
        .expect("valid regex")
});

pub fn sanitize_external_content_text(content: &str) -> String {
    let step1 = MARKER_SANITIZED_RE.replace_all(content, "[[MARKER_SANITIZED]]");
    let step2 = END_MARKER_SANITIZED_RE.replace_all(&step1, "[[END_MARKER_SANITIZED]]");
    let step3 = SPECIAL_TOKENS_RE.replace_all(&step2, "[REMOVED_SPECIAL_TOKEN]");
    step3.into_owned()
}

pub fn wrap_untrusted_attachment_content(content: &str) -> String {
    let sanitized = sanitize_external_content_text(content);
    let marker_id = &uuid::Uuid::new_v4().simple().to_string()[..16];
    format!("\n<<<EXTERNAL_UNTRUSTED_CONTENT id=\"{marker_id}\">>>\nSource: External\n---\n{sanitized}\n<<<END_EXTERNAL_UNTRUSTED_CONTENT id=\"{marker_id}\">>>")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAttachmentOutcome {
    Extracted {
        text: String,
        images: Vec<DocumentExtractedImage>,
        metadata: Option<DocumentExtractionMetadata>,
    },
    RenderedToImages {
        images: Vec<DocumentExtractedImage>,
        metadata: Option<DocumentExtractionMetadata>,
    },
    NoExtractableText {
        metadata: Option<DocumentExtractionMetadata>,
    },
    UnsupportedFormat {
        mime: Option<String>,
        local_path: Option<String>,
    },
    PolicyRejected {
        mime: Option<String>,
    },
    ReadFailure,
    UrlSourcesDisabled,
    ClaimedElsewhere,
}

pub fn resolve_file_extraction_outcome(
    text: Option<&str>,
    images: Vec<DocumentExtractedImage>,
    metadata: Option<DocumentExtractionMetadata>,
) -> FileAttachmentOutcome {
    let text = text.unwrap_or("");
    let is_text_truncated = metadata.as_ref().map(|m| m.text_truncated).unwrap_or(false);
    if !text.trim().is_empty() || is_text_truncated {
        FileAttachmentOutcome::Extracted {
            text: text.to_string(),
            images,
            metadata,
        }
    } else if !images.is_empty() {
        FileAttachmentOutcome::RenderedToImages { images, metadata }
    } else {
        FileAttachmentOutcome::NoExtractableText { metadata }
    }
}

pub fn is_skipped_file_outcome(outcome: &FileAttachmentOutcome) -> bool {
    matches!(
        outcome,
        FileAttachmentOutcome::UnsupportedFormat { .. }
            | FileAttachmentOutcome::PolicyRejected { .. }
            | FileAttachmentOutcome::ReadFailure
            | FileAttachmentOutcome::UrlSourcesDisabled
    )
}

#[derive(Debug, Clone, Default)]
pub struct RenderFileAttachmentOutcomeOptions<'a> {
    pub self_serve_local_path: Option<&'a str>,
    pub disable_self_serve: bool,
}

pub fn render_file_attachment_outcome(
    outcome: &FileAttachmentOutcome,
    options: Option<&RenderFileAttachmentOutcomeOptions>,
) -> Option<String> {
    match outcome {
        FileAttachmentOutcome::Extracted { text, metadata, .. } => {
            let notice = render_document_truncation_notice(metadata.as_ref(), None);
            let wrapped = wrap_untrusted_attachment_content(text);
            if let Some(n) = notice {
                Some(format!("{n}\n{wrapped}"))
            } else {
                Some(wrapped)
            }
        }
        FileAttachmentOutcome::RenderedToImages { metadata, .. } => {
            let notice = render_document_truncation_notice(metadata.as_ref(), None);
            let marker = "[PDF content rendered to images]";
            if let Some(n) = notice {
                Some(format!("{n}\n{marker}"))
            } else {
                Some(marker.to_string())
            }
        }
        FileAttachmentOutcome::NoExtractableText { metadata } => {
            let notice = render_document_truncation_notice(metadata.as_ref(), None);
            let marker = "[No extractable text]";
            if let Some(n) = notice {
                Some(format!("{n}\n{marker}"))
            } else {
                Some(marker.to_string())
            }
        }
        FileAttachmentOutcome::UnsupportedFormat { mime, local_path } => {
            let safe_mime = marker_safe_mime(mime.as_deref());
            let format_clause = match &safe_mime {
                Some(m) => format!("Unsupported document format: {m}."),
                None => "Unsupported document format.".to_string(),
            };
            let (target_path, allow_rel) = if let Some(opts) = options {
                if opts.disable_self_serve {
                    (None, false)
                } else if let Some(override_path) = opts.self_serve_local_path {
                    (Some(override_path), true)
                } else {
                    (local_path.as_deref(), false)
                }
            } else {
                (local_path.as_deref(), false)
            };
            let validated_path = marker_safe_local_path(target_path, allow_rel);
            let format_hint = if mime
                .as_deref()
                .map_or(false, |m| m.starts_with("application/vnd.openxmlformats-officedocument"))
            {
                " (this Office file is a zip archive containing XML)"
            } else {
                ""
            };
            match validated_path {
                Some(path) => {
                    let guidance = format!(
                        "[{format_clause} The approved local file path follows as external attachment metadata. Its text is not extracted automatically. Read the file yourself with your tools before answering{format_hint}; do not ask the user to paste the contents.]"
                    );
                    let wrapped = wrap_untrusted_attachment_content(&path);
                    Some(format!("{guidance}{wrapped}"))
                }
                None => Some(format!(
                    "[{format_clause} PDF and plain-text attachments can be read.]"
                )),
            }
        }
        FileAttachmentOutcome::PolicyRejected { mime } => {
            let safe_mime = marker_safe_mime(mime.as_deref());
            match safe_mime {
                Some(m) => Some(format!("[Attachment type not allowed: {m}]")),
                None => Some("[Attachment type not allowed]".to_string()),
            }
        }
        FileAttachmentOutcome::ReadFailure => Some("[Attachment could not be read]".to_string()),
        FileAttachmentOutcome::UrlSourcesDisabled => {
            Some("[Attachment skipped: URL file sources are disabled]".to_string())
        }
        FileAttachmentOutcome::ClaimedElsewhere => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundDocumentContext {
    pub text: String,
    pub images: Vec<ExtractedFileImage>,
}

#[derive(Debug, Clone)]
pub struct InboundDocumentItem {
    pub index: usize,
    pub file_name: Option<String>,
    pub mime: Option<String>,
    pub local_path: Option<String>,
    pub outcome: FileAttachmentOutcome,
}

pub fn render_inbound_document_context_from_items(
    items: &[InboundDocumentItem],
    self_serve_paths_enabled: bool,
) -> InboundDocumentContext {
    let mut blocks: Vec<AttachmentContextBlock> = Vec::new();
    let mut images: Vec<ExtractedFileImage> = Vec::new();

    for item in items {
        if let FileAttachmentOutcome::Extracted { images: imgs, .. }
        | FileAttachmentOutcome::RenderedToImages { images: imgs, .. } = &item.outcome
        {
            for img in imgs {
                images.push(ExtractedFileImage {
                    r#type: "image".to_string(),
                    data: img.data.clone(),
                    mime_type: img.mime_type.clone(),
                    attachment_index: item.index,
                });
            }
        }

        let options = RenderFileAttachmentOutcomeOptions {
            self_serve_local_path: None,
            disable_self_serve: !self_serve_paths_enabled,
        };
        let block_text = render_file_attachment_outcome(&item.outcome, Some(&options));
        if let Some(content) = block_text {
            let fallback = format!("file-{}", item.index + 1);
            let rendered_block = render_file_context_block(RenderFileContextParams {
                filename: item.file_name.as_deref(),
                fallback_name: Some(&fallback),
                mime_type: item.mime.as_deref(),
                content: &content,
                surround_content_with_newlines: true,
            });
            let consumes = is_skipped_file_outcome(&item.outcome);
            blocks.push(AttachmentContextBlock {
                text: rendered_block,
                consumes_marker_budget: consumes,
            });
        }
    }

    let budgeted = apply_attachment_marker_budget(&blocks);
    InboundDocumentContext {
        text: budgeted.join("\n\n"),
        images,
    }
}



