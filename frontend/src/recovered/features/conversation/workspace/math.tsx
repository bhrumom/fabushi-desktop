// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js
// @evidence src/app/dist/renderer/assets/katex-DHMw6HUq.js
import { useEffect, useState } from "react";
import {
  SHARED_ASSISTANT_MATH_MARKUP_CACHE,
  loadShippedKatexRuntime,
  type KatexRuntimeLoader,
} from "./math-runtime";

export {
  KATEX_ASSET,
  createAssistantMathMarkupCache,
  loadShippedKatexRuntime,
  renderKatexMarkup,
} from "./math-runtime";
export type {
  AssistantMathMarkupCache,
  KatexRuntime,
  KatexRuntimeLoader,
} from "./math-runtime";

function decodeHtml(value: string): string {
  return value.replace(/&quot;|&#39;|&amp;|&lt;|&gt;/gu, (entity) => ({ "&quot;": '"', "&#39;": "'", "&amp;": "&", "&lt;": "<", "&gt;": ">" })[entity] ?? entity);
}

export interface AssistantMathSegment {
  kind: "text" | "math";
  text: string;
  displayMode?: boolean;
}

/** The shipped remark-math configuration disables single-dollar text math. */
export function splitAssistantInlineMath(text: string): AssistantMathSegment[] {
  const segments: AssistantMathSegment[] = [];
  const pattern = /\\\(([^\\\n]*?)\\\)/gu;
  let cursor = 0;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(text)) != null) {
    if (match.index > cursor) segments.push({ kind: "text", text: text.slice(cursor, match.index) });
    segments.push({ kind: "math", text: match[1] ?? "", displayMode: false });
    cursor = match.index + match[0].length;
  }
  if (cursor < text.length || segments.length === 0) segments.push({ kind: "text", text: text.slice(cursor) });
  return segments;
}

function MathMarkup({ markup, displayMode }: { markup: string; displayMode: boolean }) {
  const errorMarker = ["<", "span class=\"katex-error"].join("");
  if (markup.startsWith(errorMarker)) {
    const openingEnd = markup.indexOf(">");
    const closing = ["<", "/span", ">"].join("");
    if (openingEnd >= 0 && markup.endsWith(closing)) {
      const titleMarker = "title=\"";
      const titleStart = markup.indexOf(titleMarker, 0);
      const titleEnd = titleStart < 0 ? -1 : markup.indexOf("\"", titleStart + titleMarker.length);
      const title = titleStart >= 0 && titleEnd >= 0 ? decodeHtml(markup.slice(titleStart + titleMarker.length, titleEnd)) : undefined;
      return <span className="katex-error" dangerouslySetInnerHTML={{ __html: markup.slice(openingEnd + 1, -closing.length) }} style={{ color: "#cc0000" }} title={title} />;
    }
  }
  const className = displayMode ? "katex-display" : "katex";
  const prefix = ["<", "span class=\"", className, "\">"].join("");
  const closing = ["<", "/span", ">"].join("");
  const innerMarkup = markup.startsWith(prefix) && markup.endsWith(closing) ? markup.slice(prefix.length, -closing.length) : markup;
  return <span className={className} dangerouslySetInnerHTML={{ __html: innerMarkup }} />;
}

export function AssistantMath({ expression, displayMode, loadRuntime = loadShippedKatexRuntime }: { expression: string; displayMode: boolean; loadRuntime?: KatexRuntimeLoader }) {
  const [markup, setMarkup] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    setMarkup(null);
    void SHARED_ASSISTANT_MATH_MARKUP_CACHE.load(loadRuntime, expression, displayMode).then((next) => {
      if (active) setMarkup(next);
    }).catch(() => {
      if (active) setMarkup(null);
    });
    return () => { active = false; };
  }, [displayMode, expression, loadRuntime]);
  if (markup == null) return <code className={displayMode ? "language-math math-display" : "language-math math-inline"}>{expression}</code>;
  return <MathMarkup displayMode={displayMode} markup={markup} />;
}
