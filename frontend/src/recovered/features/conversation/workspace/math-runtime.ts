// Pure AssistantMath runtime/cache contract. Kept React-free so the same shipping
// semantics are executable in Node contract tests and renderer builds.

export const KATEX_ASSET = "/upstream/assets/katex-DHMw6HUq.js";

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

export function renderKatexMarkup(runtime: KatexRuntime, expression: string, displayMode: boolean): string {
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

export function createAssistantMathMarkupCache(): AssistantMathMarkupCache {
  let byLoader = new WeakMap<KatexRuntimeLoader, Map<string, Promise<string>>>();
  const keyFor = (expression: string, displayMode: boolean) => `${displayMode ? "display" : "inline"}:\u0000${expression}`;
  return {
    load(loader, expression, displayMode) {
      let entries = byLoader.get(loader);
      if (entries == null) {
        entries = new Map();
        byLoader.set(loader, entries);
      }
      const key = keyFor(expression, displayMode);
      const existing = entries.get(key);
      if (existing != null) return existing;
      const request = loader()
        .then((runtime) => renderKatexMarkup(runtime, expression, displayMode))
        .catch((error) => {
          entries?.delete(key);
          throw error;
        });
      entries.set(key, request);
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
