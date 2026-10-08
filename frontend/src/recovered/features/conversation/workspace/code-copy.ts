export interface TranscriptCodeClipboard {
  writeText(text: string): Promise<void>;
}

/**
 * Capability-minimal code-copy boundary. Only the immutable code payload and
 * the clipboard writer cross this helper; session/window/controller context
 * is intentionally absent.
 */
export async function copyTranscriptCodeText(
  code: string,
  clipboard: TranscriptCodeClipboard | null | undefined,
): Promise<boolean> {
  if (clipboard == null) return false;
  try {
    await clipboard.writeText(code);
    return true;
  } catch {
    return false;
  }
}
