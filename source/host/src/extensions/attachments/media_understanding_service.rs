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

// =========================================================================
// Unit 34: Local Audio & OpenAI Compatible Audio/Video (OCRM-RESP-MEDIA-UND-LOCAL-AUDIO-001 & OPENAI-COMPAT-001)
// =========================================================================

pub const OPENAI_AUDIO_TRANSCRIPTIONS_API: &str = "openai-audio-transcriptions";

pub fn resolve_openai_audio_auth_model_api(
    capability: MediaUnderstandingCapability,
    provider_id: &str,
) -> Option<&'static str> {
    if capability == MediaUnderstandingCapability::Audio
        && provider_id.trim().eq_ignore_ascii_case("openai")
    {
        Some(OPENAI_AUDIO_TRANSCRIPTIONS_API)
    } else {
        None
    }
}

pub fn resolve_media_understanding_string(value: Option<&str>, fallback: &str) -> String {
    let trimmed = value.map(str::trim).unwrap_or("");
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn coerce_openai_compatible_video_text(payload: &serde_json::Value) -> Option<String> {
    let message = payload.get("choices")?.get(0)?.get("message")?;
    if let Some(content) = message.get("content") {
        if let Some(s) = content.as_str() {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        } else if let Some(arr) = content.as_array() {
            let parts: Vec<&str> = arr
                .iter()
                .filter_map(|p| p.get("text")?.as_str().map(str::trim))
                .filter(|s| !s.is_empty())
                .collect();
            if !parts.is_empty() {
                return Some(parts.join("\n"));
            }
        }
    }
    if let Some(reasoning) = message.get("reasoning_content").and_then(|r| r.as_str()) {
        let trimmed = reasoning.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

pub fn build_openai_compatible_video_request_body(
    model: &str,
    prompt: &str,
    mime: &str,
    buffer_base64: &str,
) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    {
                        "type": "video_url",
                        "video_url": {
                            "url": format!("data:{mime};base64,{buffer_base64}")
                        }
                    }
                ]
            }
        ]
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionFormData {
    pub file_name: String,
    pub mime: String,
    pub model: String,
    pub language: Option<String>,
    pub prompt: Option<String>,
}

pub fn build_audio_transcription_form_data(
    file_name: &str,
    mime: &str,
    model: &str,
    language: Option<&str>,
    prompt: Option<&str>,
) -> AudioTranscriptionFormData {
    let remapped_name = resolve_audio_transcription_upload_file_name(Some(file_name), Some(mime));
    AudioTranscriptionFormData {
        file_name: remapped_name,
        mime: mime.to_string(),
        model: model.to_string(),
        language: language.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string),
        prompt: prompt.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string),
    }
}

pub fn command_id(command: &str) -> String {
    let trimmed = command.trim();
    let basename = trimmed
        .rsplit(|c| c == '/' || c == '\\')
        .next()
        .unwrap_or(trimmed);
    let stem = if let Some(idx) = basename.rfind('.') {
        let ext = &basename[idx..];
        if ext.eq_ignore_ascii_case(".exe")
            || ext.eq_ignore_ascii_case(".com")
            || ext.eq_ignore_ascii_case(".cmd")
            || ext.eq_ignore_ascii_case(".bat")
        {
            &basename[..idx]
        } else {
            basename
        }
    } else {
        basename
    };
    stem.to_ascii_lowercase()
}

pub fn resolve_requested_local_audio_backend(command: &str, args: &[String]) -> Option<String> {
    let cmd = command_id(command);
    if cmd == "sherpa-onnx-offline" {
        for (i, arg) in args.iter().enumerate() {
            if arg == "--provider" {
                if let Some(next) = args.get(i + 1) {
                    return Some(next.trim().to_ascii_lowercase());
                }
            } else if let Some(stripped) = arg.strip_prefix("--provider=") {
                return Some(stripped.trim().to_ascii_lowercase());
            }
        }
        return Some("cpu".to_string());
    }
    if cmd == "whisper-cli" {
        if args.iter().any(|arg| arg == "-ng" || arg == "--no-gpu") {
            return Some("cpu".to_string());
        }
        for (i, arg) in args.iter().enumerate() {
            if arg == "-dev" || arg == "--device" {
                if let Some(next) = args.get(i + 1) {
                    let dev = next.trim();
                    if !dev.is_empty() {
                        return Some(format!("device:{dev}"));
                    }
                }
            } else if let Some(stripped) = arg.strip_prefix("--device=") {
                let dev = stripped.trim();
                if !dev.is_empty() {
                    return Some(format!("device:{dev}"));
                }
            }
        }
        return None;
    }
    None
}

static ACCELERATOR_FAILED_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)failed to initialize\s+(?:MTL\d+|Metal|CUDA\d*)\s+backend")
        .expect("valid regex")
});

static USING_METAL_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)using\s+(?:MTL\d+|Metal)\s+backend").expect("valid regex")
});

static USING_CUDA_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)using\s+CUDA\d*\s+backend").expect("valid regex")
});

static USING_CPU_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)using\s+CPU\s+backend|no GPU found").expect("valid regex")
});

pub fn record_local_audio_backend_observation(command: &str, output: &str) -> Option<&'static str> {
    if command_id(command) != "whisper-cli" {
        return None;
    }
    if ACCELERATOR_FAILED_RE.is_match(output) {
        return Some("cpu");
    }
    if USING_METAL_RE.is_match(output) {
        return Some("metal");
    }
    if USING_CUDA_RE.is_match(output) {
        return Some("cuda");
    }
    if USING_CPU_RE.is_match(output) {
        return Some("cpu");
    }
    None
}

pub fn rank_local_audio_candidate(candidate_id: &str, observed_backend: Option<&str>) -> usize {
    if candidate_id == "whisper-cli" && (observed_backend == Some("metal") || observed_backend == Some("cuda")) {
        0
    } else if candidate_id == "sherpa-onnx-offline" {
        1
    } else if candidate_id == "whisper-cli" {
        2
    } else if candidate_id == "parakeet-mlx" {
        3
    } else {
        4
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalAudioCandidate {
    pub id: String,
    pub command: String,
    pub available: bool,
    pub ready: bool,
    pub capable_backend: Option<String>,
    pub requested_backend: Option<String>,
    pub observed_backend: Option<String>,
    pub evidence: String,
    pub selected: bool,
    pub reason: Option<String>,
    pub entry_command: Option<String>,
    pub entry_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalAudioSelection {
    pub candidates: Vec<LocalAudioCandidate>,
    pub selected: Option<LocalAudioCandidate>,
}

pub fn format_local_audio_selection(selection: &LocalAudioSelection) -> Option<String> {
    let selected = selection.selected.as_ref()?;
    let describe_backend = |c: &LocalAudioCandidate| {
        let mut parts = Vec::new();
        if let Some(ref cap) = c.capable_backend {
            parts.push(format!("capable={cap}"));
        }
        if let Some(ref req) = c.requested_backend {
            parts.push(format!("requested={req}"));
        }
        parts.push(format!(
            "observed={}",
            c.observed_backend.as_deref().unwrap_or("unknown")
        ));
        parts.join(", ")
    };

    let fallbacks: Vec<String> = selection
        .candidates
        .iter()
        .filter(|c| c.ready && c.id != selected.id)
        .map(|c| format!("{} ({})", c.command, describe_backend(c)))
        .collect();

    let fallbacks_str = if fallbacks.is_empty() {
        String::new()
    } else {
        format!("; fallbacks: {}", fallbacks.join(", "))
    };

    Some(format!(
        "{} ({}); {}{fallbacks_str}",
        selected.command,
        describe_backend(selected),
        selected.evidence
    ))
}

// =========================================================================
// Unit 35: Image Understanding & Runtime Model Resolution (OCRM-RESP-MEDIA-UND-IMAGE-INPUT-001 & IMAGE-RUNTIME-001)
// =========================================================================

pub const MEDIA_UNDERSTANDING_MAX_SOURCE_PIXELS: usize = 40_000_000;

pub fn is_heic_image_input(mime: Option<&str>, file_name: Option<&str>) -> bool {
    if let Some(m) = mime.map(str::trim).filter(|s| !s.is_empty()) {
        let lower = m.to_ascii_lowercase();
        if lower == "image/heic"
            || lower == "image/heif"
            || lower == "image/heic-sequence"
            || lower == "image/heif-sequence"
        {
            return true;
        }
    }
    if let Some(f) = file_name.map(str::trim).filter(|s| !s.is_empty()) {
        let lower = f.to_ascii_lowercase();
        if lower.ends_with(".heic") || lower.ends_with(".heif") {
            return true;
        }
    }
    false
}

pub fn effective_image_bytes_cap(max_bytes: usize, model_max_bytes: Option<usize>) -> usize {
    model_max_bytes
        .filter(|&m| m > 0)
        .map_or(max_bytes, |m| std::cmp::min(max_bytes, m))
}

pub fn require_image_capable_model(
    model_provider: &str,
    model_id: &str,
    input_capabilities: &[String],
) -> Result<(), String> {
    if input_capabilities.iter().any(|c| c == "image") {
        Ok(())
    } else {
        let caps = if input_capabilities.is_empty() {
            "none".to_string()
        } else {
            input_capabilities.join(", ")
        };
        Err(format!(
            "Model does not support images: {model_provider}/{model_id} (input: {caps})"
        ))
    }
}

pub fn resolve_image_tool_max_tokens(
    model_max_tokens: Option<usize>,
    requested_max_tokens: usize,
) -> usize {
    match model_max_tokens {
        Some(m) if m > 0 => std::cmp::min(m, requested_max_tokens),
        _ => requested_max_tokens,
    }
}

pub fn disable_reasoning_for_image_retry_payload(
    payload: &serde_json::Value,
    is_native_responses: bool,
) -> Option<serde_json::Value> {
    let mut map = payload.as_object()?.clone();
    map.remove("reasoning");
    map.remove("reasoning_effort");

    if let Some(include) = map.get("include").and_then(|v| v.as_array()) {
        let filtered: Vec<serde_json::Value> = include
            .iter()
            .filter(|v| v.as_str() != Some("reasoning.encrypted_content"))
            .cloned()
            .collect();
        if filtered.is_empty() {
            map.remove("include");
        } else {
            map.insert("include".to_string(), serde_json::Value::Array(filtered));
        }
    }

    if is_native_responses {
        map.insert(
            "reasoning".to_string(),
            serde_json::json!({ "effort": "none" }),
        );
    }

    Some(serde_json::Value::Object(map))
}

pub fn should_place_image_prompt_in_user_content(provider: &str) -> bool {
    provider.trim().eq_ignore_ascii_case("github-copilot")
}

pub fn is_image_model_no_text_error(message: &str) -> bool {
    message.trim_start().starts_with("Image model returned no text")
}

// =========================================================================
// Unit 36: Provider Capability Registry & Execution (OCRM-RESP-MEDIA-UND-PROVIDER-REGISTRY-001 & PROVIDER-EXEC-001)
// =========================================================================

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaUnderstandingProvider {
    pub id: String,
    pub capabilities: Vec<MediaUnderstandingCapability>,
    pub default_models: HashMap<MediaUnderstandingCapability, String>,
    pub auto_priority: HashMap<MediaUnderstandingCapability, i32>,
    pub shared_entry_capabilities: Option<Vec<MediaUnderstandingCapability>>,
    pub inherit_configured_image_capability: bool,
}

pub fn normalize_media_provider_id(id: &str) -> String {
    id.trim().to_ascii_lowercase()
}

pub fn resolve_default_media_model_from_registry(
    provider_id: &str,
    capability: MediaUnderstandingCapability,
    registry: &HashMap<String, MediaUnderstandingProvider>,
) -> Option<String> {
    let normalized = normalize_media_provider_id(provider_id);
    let provider = registry.get(&normalized)?;
    provider.default_models.get(&capability).cloned()
}

pub fn resolve_auto_media_key_providers_from_registry(
    capability: MediaUnderstandingCapability,
    registry: &HashMap<String, MediaUnderstandingProvider>,
) -> Vec<String> {
    let mut entries: Vec<(&MediaUnderstandingProvider, i32)> = registry
        .values()
        .filter(|p| p.capabilities.contains(&capability))
        .filter_map(|p| p.auto_priority.get(&capability).map(|&prio| (p, prio)))
        .collect();

    entries.sort_by(|(p1, prio1), (p2, prio2)| {
        prio1.cmp(prio2).then_with(|| p1.id.cmp(&p2.id))
    });

    entries
        .into_iter()
        .map(|(p, _)| normalize_media_provider_id(&p.id))
        .collect()
}

pub fn build_media_understanding_registry(
    base_providers: &[MediaUnderstandingProvider],
    image_capable_providers: &[String],
    overrides: &HashMap<String, MediaUnderstandingProvider>,
) -> HashMap<String, MediaUnderstandingProvider> {
    let mut registry = HashMap::new();
    for p in base_providers {
        let key = normalize_media_provider_id(&p.id);
        registry.insert(key, p.clone());
    }
    for configured_id in image_capable_providers {
        let key = normalize_media_provider_id(configured_id);
        registry.entry(key.clone()).or_insert_with(|| MediaUnderstandingProvider {
            id: key,
            capabilities: vec![MediaUnderstandingCapability::Image],
            default_models: HashMap::new(),
            auto_priority: HashMap::new(),
            shared_entry_capabilities: None,
            inherit_configured_image_capability: false,
        });
    }
    for (k, v) in overrides {
        let key = normalize_media_provider_id(k);
        registry.insert(key, v.clone());
    }
    registry
}

pub fn resolve_provider_request_context(
    provider_base_url: Option<&str>,
    tools_base_url: Option<&str>,
    entry_base_url: Option<&str>,
    provider_headers: Option<&HashMap<String, String>>,
    tools_headers: Option<&HashMap<String, String>>,
    entry_headers: Option<&HashMap<String, String>>,
) -> (Option<String>, HashMap<String, String>) {
    let base_url = entry_base_url
        .or(tools_base_url)
        .or(provider_base_url)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let mut merged_headers = HashMap::new();
    if let Some(h) = provider_headers {
        for (k, v) in h {
            merged_headers.insert(k.clone(), v.clone());
        }
    }
    if let Some(h) = tools_headers {
        for (k, v) in h {
            merged_headers.insert(k.clone(), v.clone());
        }
    }
    if let Some(h) = entry_headers {
        for (k, v) in h {
            merged_headers.insert(k.clone(), v.clone());
        }
    }
    (base_url, merged_headers)
}



