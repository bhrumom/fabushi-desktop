import { open } from "node:fs/promises";
import { join } from "node:path";

import { getSandRootDir, reanchorSandPath } from "../../shared/node/sand-data-root.js";
import { isPathWithin } from "../../shared/node/paths.js";

export const STAGING_DIRNAME = "attachment-staging";
export const ATTACHMENT_PREVIEW_BYTE_CAP = 25 * 1024 * 1024;

export interface ImageSize { readonly width: number; readonly height: number }
export interface DesktopImageAttachment { readonly dataUrl: string; readonly width: number | null; readonly height: number | null }
export interface NativeImageBufferPort {
  createFromBuffer(buffer: Buffer): { isEmpty(): boolean; getSize(): ImageSize };
}
export interface AttachmentManagerDeps {
  readonly nativeImage: NativeImageBufferPort;
  readonly readPortableDimensions: (buffer: Buffer) => ImageSize | null;
  readonly servableImageMimeFromPath: (path: string) => string | null;
  readonly readFile?: (path: string) => Promise<Buffer>;
  readonly previewByteCap?: number;
  readonly reanchorPath?: (path: string) => string;
}

export function getDesktopAttachmentStagingDir(): string { return join(getSandRootDir(), STAGING_DIRNAME); }

export function isWithinDesktopAttachmentStaging(filePath: unknown): boolean {
  return typeof filePath === "string" && filePath.length > 0 && isPathWithin(getDesktopAttachmentStagingDir(), reanchorSandPath(filePath));
}

export function readDesktopImageSize(buffer: Buffer, deps: Pick<AttachmentManagerDeps, "nativeImage" | "readPortableDimensions">): ImageSize | null {
  const portable = deps.readPortableDimensions(buffer);
  if (portable != null) return portable;
  try {
    const image = deps.nativeImage.createFromBuffer(buffer);
    if (image.isEmpty()) return null;
    const { width, height } = image.getSize();
    return width > 0 && height > 0 ? { width, height } : null;
  } catch { return null; }
}

async function readDesktopPreviewFileBounded(filePath: string, maxBytes: number): Promise<Buffer | null> {
  if (!Number.isSafeInteger(maxBytes) || maxBytes <= 0) return null;
  const handle = await open(filePath, "r");
  try {
    const before = await handle.stat();
    if (!before.isFile() || !Number.isSafeInteger(before.size) || before.size <= 0 || before.size > maxBytes) return null;
    const data = Buffer.allocUnsafe(before.size);
    let offset = 0;
    while (offset < data.length) {
      const { bytesRead } = await handle.read(data, offset, data.length - offset, offset);
      if (bytesRead <= 0) return null;
      offset += bytesRead;
    }
    const after = await handle.stat();
    return after.size === before.size ? data : null;
  } finally {
    await handle.close();
  }
}

export async function readDesktopImageAttachment(filePath: string, deps: AttachmentManagerDeps): Promise<DesktopImageAttachment | null> {
  const resolved = (deps.reanchorPath ?? reanchorSandPath)(filePath);
  const mime = deps.servableImageMimeFromPath(resolved);
  const previewByteCap = deps.previewByteCap ?? ATTACHMENT_PREVIEW_BYTE_CAP;
  if (mime == null || !Number.isSafeInteger(previewByteCap) || previewByteCap <= 0) return null;
  try {
    const data = deps.readFile == null
      ? await readDesktopPreviewFileBounded(resolved, previewByteCap)
      : await deps.readFile(resolved);
    if (data == null || data.byteLength === 0 || data.byteLength > previewByteCap) return null;
    const size = readDesktopImageSize(data, deps);
    return { dataUrl: `data:${mime};base64,${data.toString("base64")}`, width: size?.width ?? null, height: size?.height ?? null };
  } catch { return null; }
}

export async function resolveImageAttachment(
  filePath: string,
  readRemote: (path: string) => Promise<DesktopImageAttachment | null>,
  deps: AttachmentManagerDeps,
): Promise<DesktopImageAttachment | null> {
  return await readDesktopImageAttachment(filePath, deps) ?? await readRemote(filePath);
}
