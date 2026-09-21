import type { AttachmentContext } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';

export const AGENT_ATTACHMENT_LIMIT = 6;
export const AGENT_ATTACHMENT_BYTE_LIMIT = 25 * 1024 * 1024;
export const AGENT_VIDEO_ATTACHMENT_BYTE_LIMIT = 200 * 1024 * 1024;
export const AGENT_ATTACHMENT_TEXT_PREVIEW_BYTES = 64 * 1024;

export function agentAttachmentLooksLikeVideo(name: string): boolean {
  return /\.(?:mp4|mov|m4v|webm|mkv|avi|mpg|mpeg)$/i.test(name);
}

export function agentAttachmentLooksTextPreviewable(file: File): boolean {
  if (file.type.startsWith('text/')) return true;
  return /\.(?:txt|md|markdown|mdc|csv|tsv|json|jsonl|ya?ml|toml|xml|html?|css|jsx?|tsx?|py|rs|go|java|kt|swift|c|h|cpp|hpp|sh|bash|zsh|fish|log|sql|ini|conf)$/i.test(file.name);
}

export function agentAttachmentByteLimit(file: File): number {
  return agentAttachmentLooksLikeVideo(file.name)
    ? AGENT_VIDEO_ATTACHMENT_BYTE_LIMIT
    : AGENT_ATTACHMENT_BYTE_LIMIT;
}

export function validateAgentAttachment(file: File): string | null {
  if (!file.size) return `“${file.name}” is empty.`;
  const limit = agentAttachmentByteLimit(file);
  if (file.size > limit) {
    return `“${file.name}” exceeds the ${Math.round(limit / 1024 / 1024)} MB attachment limit.`;
  }
  return null;
}

export function agentFileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error ?? new Error(`Unable to read ${file.name}.`));
    reader.onload = () => {
      const result = typeof reader.result === 'string' ? reader.result : '';
      const comma = result.indexOf(',');
      if (comma < 0) reject(new Error(`Unable to encode ${file.name}.`));
      else resolve(result.slice(comma + 1));
    };
    reader.readAsDataURL(file);
  });
}

export async function enrichAgentAttachmentPreview(
  attachment: AttachmentContext,
  file: File,
): Promise<AttachmentContext> {
  if (!agentAttachmentLooksTextPreviewable(file)) return attachment;
  return {
    ...attachment,
    text: await file.slice(0, AGENT_ATTACHMENT_TEXT_PREVIEW_BYTES).text(),
  };
}

export function formatAgentAttachmentSize(sizeBytes?: number): string {
  if (sizeBytes == null || !Number.isFinite(sizeBytes)) return '';
  if (sizeBytes < 1024) return `${sizeBytes} B`;
  if (sizeBytes < 1024 * 1024) return `${Math.round(sizeBytes / 1024)} KB`;
  return `${(sizeBytes / 1024 / 1024).toFixed(1)} MB`;
}
