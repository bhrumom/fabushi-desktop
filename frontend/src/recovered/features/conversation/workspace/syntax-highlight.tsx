import { Fragment, type ReactNode } from "react";
// PrismJS remains a source-neutral renderer dependency. The repository's
// existing type dependency graph covers its module imports; this file keeps the
// tiny runtime surface narrowed locally so renderer code does not depend on
// Prism-specific token implementation details.
import Prism from "prismjs";
// Keep grammar imports static so Vite packages them into the offline renderer.
import "prismjs/components/prism-bash.js";
import "prismjs/components/prism-c.js";
import "prismjs/components/prism-cpp.js";
import "prismjs/components/prism-csharp.js";
import "prismjs/components/prism-diff.js";
import "prismjs/components/prism-docker.js";
import "prismjs/components/prism-go.js";
import "prismjs/components/prism-java.js";
import "prismjs/components/prism-json.js";
import "prismjs/components/prism-kotlin.js";
import "prismjs/components/prism-markdown.js";
import "prismjs/components/prism-powershell.js";
import "prismjs/components/prism-python.js";
import "prismjs/components/prism-rust.js";
import "prismjs/components/prism-sql.js";
import "prismjs/components/prism-swift.js";
import "prismjs/components/prism-typescript.js";
import "prismjs/components/prism-jsx.js";
import "prismjs/components/prism-tsx.js";
import "prismjs/components/prism-yaml.js";

const MAX_HIGHLIGHT_UTF8_BYTES = 512 * 1024;
const MAX_HIGHLIGHT_LINES = 10_000;

const LANGUAGE_ALIASES: Readonly<Record<string, string>> = {
  cxx: "cpp",
  "c++": "cpp",
  cs: "csharp",
  dockerfile: "docker",
  html: "markup",
  js: "javascript",
  jsx: "jsx",
  md: "markdown",
  py: "python",
  rs: "rust",
  sh: "bash",
  shell: "bash",
  ts: "typescript",
  tsx: "tsx",
  xml: "markup",
  yml: "yaml",
};

interface PrismToken {
  readonly type: string;
  readonly alias?: string | readonly string[];
  readonly content: PrismTokenStream;
}
type PrismTokenStream = string | PrismToken | readonly PrismTokenStream[];

interface PrismRuntime {
  readonly languages: Readonly<Record<string, unknown>>;
  tokenize(code: string, grammar: unknown): readonly PrismTokenStream[];
}

const prism = Prism as unknown as PrismRuntime;

function normalizedLanguage(value: string): string {
  const normalized = value.trim().toLowerCase();
  if (normalized.length === 0) return "";
  return LANGUAGE_ALIASES[normalized] ?? normalized;
}

function lineCountWithinLimit(code: string): boolean {
  let lines = 1;
  for (let index = 0; index < code.length; index += 1) {
    if (code.charCodeAt(index) !== 10) continue;
    lines += 1;
    if (lines > MAX_HIGHLIGHT_LINES) return false;
  }
  return true;
}

function utf8BytesWithinLimit(code: string): boolean {
  // UTF-8 byte length can never be smaller than UTF-16 code-unit count for
  // ASCII-heavy source. Avoid allocating the encoder buffer once the cheap
  // lower bound already exceeds the source-neutral renderer budget.
  if (code.length > MAX_HIGHLIGHT_UTF8_BYTES) return false;
  return new TextEncoder().encode(code).byteLength <= MAX_HIGHLIGHT_UTF8_BYTES;
}

function safeTokenClass(value: string): string | null {
  const normalized = value.trim().replace(/[^a-zA-Z0-9_-]/gu, "");
  return normalized.length === 0 ? null : normalized;
}

function tokenClasses(token: PrismToken): string {
  const classes = ["token"];
  const type = safeTokenClass(token.type);
  if (type != null) classes.push(type);
  const aliases = typeof token.alias === "string" ? [token.alias] : token.alias ?? [];
  for (const alias of aliases) {
    const safe = safeTokenClass(alias);
    if (safe != null) classes.push(safe);
  }
  return classes.join(" ");
}

function renderTokenStream(stream: PrismTokenStream, key: string): ReactNode {
  if (typeof stream === "string") return stream;
  if (Array.isArray(stream)) {
    return stream.map((child, index) => (
      <Fragment key={`${key}:${index}`}>{renderTokenStream(child, `${key}:${index}`)}</Fragment>
    ));
  }
  const token = stream as PrismToken;
  return (
    <span className={tokenClasses(token)} key={key}>
      {renderTokenStream(token.content, `${key}:content`)}
    </span>
  );
}

export interface TranscriptSyntaxCodeProps {
  readonly code: string;
  readonly language: string;
  readonly enabled?: boolean;
}

/**
 * Canonical desktop transcript syntax projection.
 *
 * It never emits HTML from message content. Prism only supplies a token tree,
 * which is projected through React text nodes. Unknown grammars, malformed
 * tokenization, streaming content, and over-budget source fail closed to the
 * original plain-text code.
 */
export function TranscriptSyntaxCode({
  code,
  language,
  enabled = true,
}: TranscriptSyntaxCodeProps) {
  const canonicalLanguage = normalizedLanguage(language);
  if (!enabled
    || canonicalLanguage.length === 0
    || !lineCountWithinLimit(code)
    || !utf8BytesWithinLimit(code)) {
    return <code className={canonicalLanguage.length > 0 ? `language-${canonicalLanguage}` : "sand-code-fallback"}>{code}</code>;
  }

  const grammar = prism.languages[canonicalLanguage];
  if (grammar == null) {
    return <code className={`language-${canonicalLanguage} sand-code-fallback`}>{code}</code>;
  }

  try {
    const tokens = prism.tokenize(code, grammar);
    return (
      <code className={`language-${canonicalLanguage} sand-code-highlighted`} data-syntax-highlighted="true">
        {renderTokenStream(tokens, "syntax")}
      </code>
    );
  } catch {
    return <code className={`language-${canonicalLanguage} sand-code-fallback`}>{code}</code>;
  }
}
