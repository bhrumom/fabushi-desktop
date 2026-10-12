import type { TranscriptCardEntry } from "../cards/transcript-card/protocol";
import type { TimelineEventData } from "../cards/timeline-event-registry";
import type { ToolResultCardSnapshot } from "../tool-results/model";
import type { SendMessageTextAdjacency, SendMessageTextImage } from "../cards/transcript-card/send-message-text";
import type { TranscriptDeliveryState } from "./transcript-delivery-state";

export const COMPOSER_ATTACHMENT_LIMIT = 6;

export type AttachmentKind =
  | "image"
  | "video"
  | "audio"
  | "pdf"
  | "markdown"
  | "table"
  | "json"
  | "document"
  | "archive"
  | "text"
  | "file";

export interface DraftAttachment {
  path: string;
  name: string;
  size?: number;
  mimeType?: string;
}

export interface ComposerDraftCursor {
  position: number;
  anchor: number;
  scroll: number;
}

export interface ComposerDraftLinkPreview {
  /** Opaque canonical Resource/Web preview identity. Use a string so 64-bit service ids remain lossless in JavaScript. */
  id?: string;
  url: string;
  forceLargeMedia?: boolean;
  forceSmallMedia?: boolean;
  invert?: boolean;
  manual?: boolean;
  removed?: boolean;
}

export interface ComposerDraft {
  prompt: string;
  attachments: DraftAttachment[];
  richText?: string;
  replyToId?: string;
  isFork?: boolean;
  cursor?: ComposerDraftCursor;
  linkPreview?: ComposerDraftLinkPreview;
}

export const COMPOSER_DRAFT_METADATA = {
  slice: "composer-drafts",
  schemaVersion: 2,
  scope: "client-persisted",
  accountSensitive: true
} as const;

export function isComposerDraftEmpty(draft: ComposerDraft): boolean {
  return draft.prompt.trim().length === 0 && draft.attachments.length === 0;
}

function emptyToNull(value: string | undefined): string | null {
  return value == null || value.length === 0 ? null : value;
}

export function areComposerDraftContentsEqual(left: ComposerDraft, right: ComposerDraft): boolean {
  return left.prompt === right.prompt
    && emptyToNull(left.richText) === emptyToNull(right.richText)
    && left.attachments.length === right.attachments.length
    && left.attachments.every((attachment, index) => {
      const candidate = right.attachments[index];
      return attachment.path === candidate?.path
        && attachment.name === candidate.name
        && attachment.size === candidate.size
        && attachment.mimeType === candidate.mimeType;
    });
}

function areComposerDraftCursorsEqual(left: ComposerDraftCursor | undefined, right: ComposerDraftCursor | undefined): boolean {
  if (left == null || right == null) return left == null && right == null;
  return left.position === right.position
    && left.anchor === right.anchor
    && left.scroll === right.scroll;
}

function areComposerDraftLinkPreviewsEqual(left: ComposerDraftLinkPreview | undefined, right: ComposerDraftLinkPreview | undefined): boolean {
  if (left == null || right == null) return left == null && right == null;
  return (left.id ?? null) === (right.id ?? null)
    && left.url === right.url
    && (left.forceLargeMedia ?? false) === (right.forceLargeMedia ?? false)
    && (left.forceSmallMedia ?? false) === (right.forceSmallMedia ?? false)
    && (left.invert ?? false) === (right.invert ?? false)
    && (left.manual ?? false) === (right.manual ?? false)
    && (left.removed ?? false) === (right.removed ?? false);
}

export function areComposerDraftsEqual(left: ComposerDraft, right: ComposerDraft): boolean {
  return areComposerDraftContentsEqual(left, right)
    && (left.replyToId ?? null) === (right.replyToId ?? null)
    && (left.isFork ?? false) === (right.isFork ?? false)
    && areComposerDraftCursorsEqual(left.cursor, right.cursor)
    && areComposerDraftLinkPreviewsEqual(left.linkPreview, right.linkPreview);
}

export function parseComposerDraft(value: unknown): ComposerDraft | null {
  if (typeof value !== "object" || value == null) return null;
  const candidate = value as Record<string, unknown>;
  if (
    typeof candidate.prompt !== "string"
    || (candidate.richText !== undefined && typeof candidate.richText !== "string")
    || (candidate.replyToId !== undefined && typeof candidate.replyToId !== "string")
    || (candidate.isFork !== undefined && typeof candidate.isFork !== "boolean")
    || !Array.isArray(candidate.attachments)
  ) return null;

  const cursor = candidate.cursor === undefined ? undefined : (() => {
    if (typeof candidate.cursor !== "object" || candidate.cursor == null || Array.isArray(candidate.cursor)) return null;
    const valueCursor = candidate.cursor as Record<string, unknown>;
    if (
      !Number.isSafeInteger(valueCursor.position)
      || !Number.isSafeInteger(valueCursor.anchor)
      || !Number.isSafeInteger(valueCursor.scroll)
    ) return null;
    return {
      position: valueCursor.position as number,
      anchor: valueCursor.anchor as number,
      scroll: valueCursor.scroll as number,
    };
  })();
  if (candidate.cursor !== undefined && cursor == null) return null;

  const linkPreview = candidate.linkPreview === undefined ? undefined : (() => {
    if (typeof candidate.linkPreview !== "object" || candidate.linkPreview == null || Array.isArray(candidate.linkPreview)) return null;
    const valuePreview = candidate.linkPreview as Record<string, unknown>;
    if (
      typeof valuePreview.url !== "string"
      || (valuePreview.id !== undefined && (typeof valuePreview.id !== "string" || valuePreview.id.length === 0))
      || (valuePreview.forceLargeMedia !== undefined && typeof valuePreview.forceLargeMedia !== "boolean")
      || (valuePreview.forceSmallMedia !== undefined && typeof valuePreview.forceSmallMedia !== "boolean")
      || (valuePreview.invert !== undefined && typeof valuePreview.invert !== "boolean")
      || (valuePreview.manual !== undefined && typeof valuePreview.manual !== "boolean")
      || (valuePreview.removed !== undefined && typeof valuePreview.removed !== "boolean")
    ) return null;
    return {
      url: valuePreview.url,
      ...(valuePreview.id === undefined ? {} : { id: valuePreview.id as string }),
      ...(valuePreview.forceLargeMedia === undefined ? {} : { forceLargeMedia: valuePreview.forceLargeMedia as boolean }),
      ...(valuePreview.forceSmallMedia === undefined ? {} : { forceSmallMedia: valuePreview.forceSmallMedia as boolean }),
      ...(valuePreview.invert === undefined ? {} : { invert: valuePreview.invert as boolean }),
      ...(valuePreview.manual === undefined ? {} : { manual: valuePreview.manual as boolean }),
      ...(valuePreview.removed === undefined ? {} : { removed: valuePreview.removed as boolean }),
    };
  })();
  if (candidate.linkPreview !== undefined && linkPreview == null) return null;

  const attachments: DraftAttachment[] = [];
  for (const valueAttachment of candidate.attachments) {
    if (typeof valueAttachment !== "object" || valueAttachment == null) return null;
    const attachment = valueAttachment as Record<string, unknown>;
    if (
      typeof attachment.path !== "string"
      || typeof attachment.name !== "string"
      || (attachment.mimeType !== undefined && typeof attachment.mimeType !== "string")
    ) return null;
    const size = typeof attachment.size === "number" && Number.isFinite(attachment.size) ? attachment.size : undefined;
    attachments.push({
      path: attachment.path,
      name: attachment.name,
      ...(size === undefined ? {} : { size }),
      ...(attachment.mimeType === undefined ? {} : { mimeType: attachment.mimeType }),
    });
  }

  return {
    prompt: candidate.prompt,
    attachments,
    ...(candidate.richText === undefined ? {} : { richText: candidate.richText }),
    ...(candidate.replyToId === undefined ? {} : { replyToId: candidate.replyToId }),
    ...(candidate.isFork === undefined ? {} : { isFork: candidate.isFork }),
    ...(cursor === undefined ? {} : { cursor }),
    ...(linkPreview === undefined ? {} : { linkPreview })
  };
}

export function attachmentBasename(value: string): string {
  let path = value;
  try {
    const url = new URL(value);
    path = url.protocol === "file:" ? decodeURIComponent(url.pathname) : url.pathname;
  } catch {
    // The value is already a local path.
  }
  const segments = path.split(/[/\\]/).filter((segment) => segment.length > 0);
  return segments.at(-1) ?? value;
}

export function formatAttachmentBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const imageExtensions = new Set(["avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"]);
const videoExtensions = new Set(["m4v", "mov", "mp4", "ogv", "webm"]);
const audioExtensions = new Set(["aac", "flac", "m4a", "mp3", "oga", "ogg", "opus", "wav", "weba"]);
const markdownExtensions = new Set(["md", "markdown", "mdx"]);
const tableExtensions = new Set(["csv", "tsv", "xlsx", "xls"]);
const jsonExtensions = new Set(["json", "jsonc", "json5", "ndjson"]);
const archiveExtensions = new Set(["zip", "tar", "gz", "tgz", "bz2", "tbz2", "xz", "txz", "zst", "7z", "rar"]);
const textExtensions = new Set([
  "txt", "text", "log", "rst", "adoc", "tex", "xml", "yaml", "yml", "toml", "ini", "cfg", "conf", "env",
  "properties", "plist", "gradle", "html", "htm", "css", "scss", "sass", "less", "js", "jsx", "mjs", "cjs", "ts",
  "tsx", "mts", "cts", "py", "pyi", "rb", "go", "rs", "java", "kt", "kts", "c", "h", "cc", "cpp", "cxx", "hpp",
  "hh", "cs", "php", "swift", "scala", "dart", "lua", "pl", "pm", "r", "sql", "graphql", "gql", "proto", "vue", "svelte",
  "astro", "sh", "bash", "zsh", "fish", "bat", "ps1", "tf", "tfvars", "dockerfile", "diff", "patch"
]);

function extension(value: string): string | null {
  let path = value;
  try {
    path = new URL(value).pathname;
  } catch {
    // Keep the original path.
  }
  const name = attachmentBasename(path);
  const dot = name.lastIndexOf(".");
  return dot <= 0 || dot === name.length - 1 ? null : name.slice(dot + 1).toLowerCase();
}

function kindFromMimeType(mimeType: string): AttachmentKind | null {
  const mime = (mimeType.split(";")[0] ?? "").trim().toLowerCase();
  if (mime.startsWith("image/")) return "image";
  if (mime.startsWith("video/")) return "video";
  if (mime.startsWith("audio/")) return "audio";
  if (mime === "application/pdf") return "pdf";
  if (mime === "text/markdown") return "markdown";
  if (["text/csv", "text/tab-separated-values", "application/vnd.ms-excel", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"].includes(mime)) return "table";
  if (mime === "application/json" || mime.endsWith("+json")) return "json";
  if (["application/msword", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"].includes(mime)) return "document";
  if (["application/zip", "application/x-zip-compressed", "application/gzip", "application/x-tar", "application/x-bzip2", "application/x-xz", "application/zstd", "application/x-7z-compressed", "application/x-rar-compressed", "application/vnd.rar"].includes(mime)) return "archive";
  return mime.startsWith("text/") ? "text" : null;
}

function kindFromPath(path: string): AttachmentKind | null {
  const ext = extension(path);
  if (ext == null) return null;
  if (imageExtensions.has(ext)) return "image";
  if (videoExtensions.has(ext)) return "video";
  if (audioExtensions.has(ext)) return "audio";
  if (ext === "pdf") return "pdf";
  if (markdownExtensions.has(ext)) return "markdown";
  if (tableExtensions.has(ext)) return "table";
  if (jsonExtensions.has(ext)) return "json";
  if (ext === "docx") return "document";
  if (archiveExtensions.has(ext)) return "archive";
  return textExtensions.has(ext) ? "text" : null;
}

export function inferAttachmentKind(input: { mimeType?: string; fileName?: string; urlOrPath?: string }): AttachmentKind {
  if (input.mimeType) {
    const kind = kindFromMimeType(input.mimeType);
    if (kind != null) return kind;
  }
  if (input.fileName) {
    const kind = kindFromPath(input.fileName);
    if (kind != null) return kind;
  }
  if (input.urlOrPath) {
    const kind = kindFromPath(input.urlOrPath);
    if (kind != null) return kind;
  }
  return "file";
}

export interface ConversationAgentSummary {
  id: string;
  name: string;
  updatedAt: number;
  isPinned?: boolean;
  isRunning?: boolean;
  isComposingMessage?: boolean;
  title?: string;
  isSharedRoom?: boolean;
  avatarDataUrl?: string | null;
  avatarVersion?: string | null;
  avatarShape?: string | null;
  avatarColor?: string | null;
  currentActivity?: unknown | null;
  memberIds?: readonly string[];
  awaitingUserResponse?: unknown | null;
  waitingReason?: string;
  draftPrompt?: string;
  lastEntry?: ConversationAgentLastEntry | null;
  lastMessageId?: string | null;
  lastMessagePreview?: string | null;
  lastMessage?: string;
}

export type ConversationAgentLastEntry =
  | { readonly kind: "text"; readonly text: string }
  | { readonly kind: "attachment"; readonly count: number; readonly kinds: Readonly<Record<string, number>> }
  | { readonly kind: "link"; readonly url: string };

export type TranscriptDelivery = TranscriptDeliveryState;

// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=4719000
export type TranscriptReplyPreview =
  | { kind: "user-text" | "assistant-text"; text: string }
  | { kind: "image"; url: string }
  | { kind: "file"; url: string; name?: string }
  | { kind: "link"; url: string }
  | { kind: "missing" };

export interface TranscriptMessage {
  kind: "message";
  id: string;
  role: "user" | "assistant";
  author: string;
  text: string;
  /** Serialized Tiptap JSON returned on durable user-message transcript entries. */
  richText?: string;
  timestampMs: number;
  attachments?: DraftAttachment[];
  delivery?: TranscriptDelivery;
  clientNonce?: string;
  composedAtMs?: number;
  isStreaming?: boolean;
  replyToId?: string;
  // @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5083971
  reactions?: readonly import("../cards/transcript-card/reaction-actions").TranscriptReaction[];
  myReactions?: ReadonlySet<string>;
  /** Exact send-message:text fields consumed by the mounted ordinary message view. */
  images?: readonly SendMessageTextImage[];
  channel?: string | null;
  isSourceTrusted?: boolean;
  adjacency?: SendMessageTextAdjacency;
  /** Exact send-message:text projection retained for its mounted transcript seam. */
  sendMessageText?: import("../cards/transcript-card/send-message-text").SendMessageTextProjection;
}

export type TranscriptToolCallStatus = "pending" | "running" | "done" | "failed" | "error" | "aborted";

export interface TranscriptToolCall {
  kind: "tool-call";
  id: string;
  name: string;
  status: TranscriptToolCallStatus;
  summary?: string;
  timestampMs: number;
  /** Read-only typed projection supplied by the transcript tool-result boundary. */
  toolResult?: ToolResultCardSnapshot;
}

export interface TranscriptThinking {
  kind: "thinking";
  id: string;
  text: string;
  durationMs?: number;
  timestampMs: number;
}

export interface TranscriptNotice {
  kind: "notice";
  id: string;
  text: string;
  timestampMs: number;
}

export interface TranscriptTimelineEvent {
  kind: "timeline-event";
  id: string;
  event: TimelineEventData;
  timestampMs: number;
}

export interface TranscriptTimeSeparator {
  kind: "time-separator";
  id: string;
  label: string;
}

export interface TranscriptUnreadDivider {
  kind: "unread-divider";
  id: string;
  newMessageCount: number;
}

export interface TranscriptComputerHandoff {
  kind: "computer-handoff";
  id: string;
  requestId: string;
  instruction: string;
  resolution: string | null;
  timestampMs: number;
}

export type TranscriptLocalToolPermissionAskStatus = "pending" | "always" | "never" | "denied" | "expired" | "allow-once";

export interface TranscriptLocalToolPermission {
  kind: "local-tool-permission";
  id: string;
  entryId: string;
  agentId: string | null;
  ask: {
    requestId: string;
    status: TranscriptLocalToolPermissionAskStatus;
    action: unknown;
    target: unknown;
  };
  permissionScope?: string;
  permissionScopeRevision?: number;
  timestampMs: number;
}

export interface TranscriptPermissionRequest {
  kind: "permission-request";
  id: string;
  title: string;
  timestampMs: number;
  isGroupStart?: boolean;
}

export type ConversationTranscriptEntry = TranscriptMessage | TranscriptToolCall | TranscriptThinking | TranscriptNotice | TranscriptTimelineEvent | TranscriptTimeSeparator | TranscriptUnreadDivider | TranscriptComputerHandoff | TranscriptLocalToolPermission | TranscriptPermissionRequest | TranscriptCardEntry;
