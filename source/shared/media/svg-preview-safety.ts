const SVG_PREVIEW_MAX_BYTES = 5 * 1024 * 1024;
const SVG_PREVIEW_MAX_ELEMENTS = 4096;
const SVG_PREVIEW_MAX_DEPTH = 128;

const DISALLOWED_ELEMENTS = new Set([
  "script", "style", "foreignobject", "iframe", "object", "embed",
  "image", "audio", "video", "a", "set", "animate", "animatemotion",
  "animatetransform", "mpath",
]);

const SAFE_ENTITY = /&(amp|lt|gt|quot|apos);/gi;
const TAG = /<\/?\s*([A-Za-z][A-Za-z0-9:_-]*)([^<>]*?)\/?>/g;
const ATTRIBUTE = /(?:^|\s)([A-Za-z_:][A-Za-z0-9:_.-]*)\s*=\s*(["'])(.*?)\2/gs;

function hasUnsafeEntity(text: string): boolean {
  return text.replace(SAFE_ENTITY, "").includes("&");
}

function isLocalFragment(value: string): boolean {
  return /^#[A-Za-z_][A-Za-z0-9_.:-]*$/.test(value);
}

/**
 * Fail-closed SVG preview gate for the attachment/resource owner.
 *
 * Chromium/nativeImage is not the security boundary for untrusted SVG. Keep
 * the preview subset intentionally smaller than full SVG: no scriptable
 * content, external resources, CSS, processing instructions, DTD/entities or
 * reference graphs. Unsupported documents remain downloadable attachments
 * instead of becoming active preview content.
 */
export function sanitizeSvgPreviewBytes(bytes: Buffer): Buffer | null {
  if (bytes.byteLength === 0 || bytes.byteLength > SVG_PREVIEW_MAX_BYTES) return null;

  let text: string;
  try {
    text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    return null;
  }
  if (text.includes("\0") || hasUnsafeEntity(text)) return null;

  const trimmed = text.trim();
  if (!/^<svg(?:\s|>)/i.test(trimmed) || !/(?:<\/svg\s*>|<svg\b[^>]*\/>)$/i.test(trimmed)) return null;
  if (/<!|<\?|<!--/i.test(trimmed)) return null;
  if (/\b(?:javascript|vbscript|data)\s*:/i.test(trimmed)) return null;
  if (/\burl\s*\(/i.test(trimmed)) return null;

  let depth = 0;
  let elements = 0;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = TAG.exec(trimmed)) != null) {
    const skipped = trimmed.slice(lastIndex, match.index);
    if (skipped.includes("<") || skipped.includes(">")) return null;
    lastIndex = TAG.lastIndex;

    const whole = match[0];
    const closing = /^<\//.test(whole);
    const selfClosing = /\/\s*>$/.test(whole);
    const name = match[1]!.toLowerCase();
    const rawAttributes = match[2] ?? "";

    if (DISALLOWED_ELEMENTS.has(name)) return null;
    if (closing) {
      depth -= 1;
      if (depth < 0) return null;
      continue;
    }

    elements += 1;
    if (elements > SVG_PREVIEW_MAX_ELEMENTS) return null;
    if (!selfClosing) {
      depth += 1;
      if (depth > SVG_PREVIEW_MAX_DEPTH) return null;
    }

    if (/\sstyle\s*=/i.test(rawAttributes)) return null;
    let consumed = rawAttributes;
    ATTRIBUTE.lastIndex = 0;
    let attribute: RegExpExecArray | null;
    while ((attribute = ATTRIBUTE.exec(rawAttributes)) != null) {
      const attributeName = attribute[1]!.toLowerCase();
      const value = attribute[3]!.trim();
      if (attributeName.startsWith("on")) return null;
      if (attributeName === "style") return null;
      if (attributeName === "href" || attributeName === "xlink:href") {
        if (!isLocalFragment(value)) return null;
        // Reference graphs are deliberately outside the safe preview subset.
        return null;
      }
      if (/\burl\s*\(/i.test(value) || /\b(?:javascript|vbscript|data)\s*:/i.test(value)) return null;
      consumed = consumed.replace(attribute[0], "");
    }
    if (consumed.trim() !== "") {
      // Bare/malformed attributes or parser-confusing syntax fail closed.
      return null;
    }
  }
  if (lastIndex !== trimmed.length || depth !== 0 || elements === 0) return null;
  return bytes;
}
