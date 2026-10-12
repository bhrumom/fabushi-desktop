type UnknownRecord = Record<string, unknown>;

function requestRecord(value: unknown): UnknownRecord {
  return typeof value === "object" && value != null && !Array.isArray(value) ? value as UnknownRecord : {};
}

export function normalizeStageAttachmentEdgeRequest(raw: unknown): { readonly filename: unknown; readonly bytes: unknown } {
  const request = requestRecord(raw);
  if (request.bytes instanceof Uint8Array) return { filename: request.filename, bytes: request.bytes };
  if (typeof request.bytesBase64 !== "string") return { filename: request.filename, bytes: undefined };
  const compact = request.bytesBase64.replace(/\s+/gu, "");
  if (!/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/u.test(compact)) {
    return { filename: request.filename, bytes: undefined };
  }
  const decoded = Buffer.from(compact, "base64");
  return { filename: request.filename, bytes: new Uint8Array(decoded.buffer, decoded.byteOffset, decoded.byteLength) };
}

export function normalizeCommitStagedAttachmentsEdgeRequest(raw: unknown): {
  readonly paths: unknown;
  readonly filenames: unknown;
  readonly scope?: unknown;
} {
  const request = requestRecord(raw);
  const withScope = (value: { readonly paths: unknown; readonly filenames: unknown }) =>
    Object.prototype.hasOwnProperty.call(request, "scope")
      ? { ...value, scope: request.scope }
      : value;
  if (Array.isArray(request.items)) {
    return withScope({
      paths: request.items.map((item) => requestRecord(item).path),
      filenames: request.items.map((item) => requestRecord(item).name),
    });
  }
  // Compatibility only: older direct edge callers used parallel arrays. The
  // shipping preload owns the canonical item-list wire contract.
  return withScope({ paths: request.paths, filenames: request.filenames });
}
