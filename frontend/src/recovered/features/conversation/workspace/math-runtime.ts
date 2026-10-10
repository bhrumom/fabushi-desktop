// Pure AssistantMath runtime/cache contract. Kept React-free so the same shipping
// semantics are executable in Node contract tests and renderer builds.

export const KATEX_ASSET = "/upstream/assets/katex-DHMw6HUq.js";
export const MAX_ASSISTANT_MATH_EXPRESSION_LENGTH = 32 * 1024;
export const DEFAULT_ASSISTANT_MATH_CACHE_BUDGET_BYTES = 32 * 1024 * 1024;
const ASSISTANT_MATH_CACHE_ENTRY_OVERHEAD_BYTES = 256;
const UTF16_BYTES_PER_CODE_UNIT = 2;

export interface KatexRuntime {
  renderToString(expression: string, options: { displayMode: boolean; throwOnError: boolean; strict?: "ignore" }): string;
}

interface KatexRuntimeModule {
  default?: KatexRuntime;
  renderToString?: KatexRuntime["renderToString"];
}

export type KatexRuntimeLoader = () => Promise<KatexRuntime>;

export async function loadShippedKatexRuntime(): Promise<KatexRuntime> {
  const module = await import(/* @vite-ignore */ KATEX_ASSET) as KatexRuntimeModule;
  if (module.default != null) return module.default;
  if (module.renderToString != null) return { renderToString: module.renderToString };
  throw new Error("Shipped KaTeX runtime is unavailable.");
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/gu, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[character] ?? character);
}

function assertAssistantMathExpressionWithinLimit(expression: string, maxExpressionLength: number): void {
  if (expression.length > maxExpressionLength) {
    throw new Error(`Assistant math expression exceeds the ${maxExpressionLength}-code-unit parsing limit.`);
  }
}

export function renderKatexMarkup(
  runtime: KatexRuntime,
  expression: string,
  displayMode: boolean,
  maxExpressionLength = MAX_ASSISTANT_MATH_EXPRESSION_LENGTH,
): string {
  assertAssistantMathExpressionWithinLimit(expression, maxExpressionLength);
  try {
    return runtime.renderToString(expression, { displayMode, throwOnError: true });
  } catch (error) {
    try {
      return runtime.renderToString(expression, { displayMode, strict: "ignore", throwOnError: false });
    } catch {
      const opening = ["<", "span class=\"katex-error\" style=\"color:#cc0000\" title=\""].join("");
      const closing = ["\">", escapeHtml(expression), "<", "/span", ">"].join("");
      return [opening, escapeHtml(String(error)), closing].join("");
    }
  }
}

export interface AssistantMathMarkupCache {
  load(loader: KatexRuntimeLoader, expression: string, displayMode: boolean): Promise<string>;
  invalidate(loader?: KatexRuntimeLoader): void;
}

export interface AssistantMathMarkupCacheOptions {
  maxBytes?: number;
  maxExpressionLength?: number;
}

interface AssistantMathCacheEntry {
  promise: Promise<string>;
  estimatedBytes: number;
}

interface AssistantMathLoaderCache {
  entries: Map<string, AssistantMathCacheEntry>;
  totalBytes: number;
}

function estimateStringBytes(value: string): number {
  return value.length * UTF16_BYTES_PER_CODE_UNIT;
}

export function createAssistantMathMarkupCache(
  options: AssistantMathMarkupCacheOptions = {},
): AssistantMathMarkupCache {
  const maxBytes = Math.max(0, Math.floor(options.maxBytes ?? DEFAULT_ASSISTANT_MATH_CACHE_BUDGET_BYTES));
  const maxExpressionLength = Math.max(0, Math.floor(
    options.maxExpressionLength ?? MAX_ASSISTANT_MATH_EXPRESSION_LENGTH,
  ));
  let byLoader = new WeakMap<KatexRuntimeLoader, AssistantMathLoaderCache>();
  const keyFor = (expression: string, displayMode: boolean) => `${displayMode ? "display" : "inline"}:\u0000${expression}`;

  const removeEntry = (
    cache: AssistantMathLoaderCache,
    key: string,
    expected?: AssistantMathCacheEntry,
  ): void => {
    const entry = cache.entries.get(key);
    if (entry == null || (expected != null && entry !== expected)) return;
    cache.entries.delete(key);
    cache.totalBytes = Math.max(0, cache.totalBytes - entry.estimatedBytes);
  };

  const evictToBudget = (cache: AssistantMathLoaderCache): void => {
    while (cache.totalBytes > maxBytes && cache.entries.size > 0) {
      const oldestKey = cache.entries.keys().next().value as string | undefined;
      if (oldestKey == null) break;
      removeEntry(cache, oldestKey);
    }
  };

  return {
    load(loader, expression, displayMode) {
      if (expression.length > maxExpressionLength) {
        return Promise.reject(new Error(
          `Assistant math expression exceeds the ${maxExpressionLength}-code-unit parsing limit.`,
        ));
      }

      let cache = byLoader.get(loader);
      if (cache == null) {
        cache = { entries: new Map(), totalBytes: 0 };
        byLoader.set(loader, cache);
      }
      const key = keyFor(expression, displayMode);
      const existing = cache.entries.get(key);
      if (existing != null) {
        cache.entries.delete(key);
        cache.entries.set(key, existing);
        return existing.promise;
      }

      const baseBytes = ASSISTANT_MATH_CACHE_ENTRY_OVERHEAD_BYTES + estimateStringBytes(key);
      let entry: AssistantMathCacheEntry;
      const request = loader()
        .then((runtime) => renderKatexMarkup(runtime, expression, displayMode, maxExpressionLength))
        .then((markup) => {
          if (cache?.entries.get(key) === entry) {
            const resolvedBytes = baseBytes + estimateStringBytes(markup);
            cache.totalBytes += resolvedBytes - entry.estimatedBytes;
            entry.estimatedBytes = resolvedBytes;
            evictToBudget(cache);
          }
          return markup;
        })
        .catch((error) => {
          if (cache != null) removeEntry(cache, key, entry);
          throw error;
        });
      entry = { promise: request, estimatedBytes: baseBytes };
      cache.entries.set(key, entry);
      cache.totalBytes += baseBytes;
      evictToBudget(cache);
      return request;
    },
    invalidate(loader) {
      if (loader == null) {
        byLoader = new WeakMap();
        return;
      }
      byLoader.delete(loader);
    },
  };
}

export const SHARED_ASSISTANT_MATH_MARKUP_CACHE = createAssistantMathMarkupCache();
