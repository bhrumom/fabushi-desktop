import type { JsonSchema, McpTool, McpToolResult } from "./types";

export type WebMcpAnnotations = {
  readOnlyHint?: boolean;
  untrustedContentHint?: boolean;
};

export type WebMcpTool = {
  name: string;
  title?: string;
  description: string;
  inputSchema?: JsonSchema;
  annotations?: WebMcpAnnotations;
  execute: (
    input: Record<string, unknown>,
    options?: { signal?: AbortSignal },
  ) => Promise<unknown> | unknown;
};

type WebMcpRegisteredTool = {
  name: string;
  title?: string | null;
  description: string;
  inputSchema?: string;
  annotations?: WebMcpAnnotations | null;
};

type WebMcpModelContext = {
  registerTool(
    tool: WebMcpTool,
    options?: { signal?: AbortSignal; exposedTo?: string[] },
  ): Promise<void> | void;
  getTools?(options?: { fromOrigins?: string[] }): Promise<WebMcpRegisteredTool[]>;
  executeTool?(
    tool: WebMcpRegisteredTool,
    input?: Record<string, unknown>,
    options?: { signal?: AbortSignal },
  ): Promise<string>;
};

type FabushiWebMcpRegistry = {
  readonly version: 1;
  list(): Array<Omit<WebMcpTool, "execute">>;
  call(name: string, input?: Record<string, unknown>): Promise<unknown>;
};

declare global {
  interface Document {
    modelContext?: WebMcpModelContext;
  }
  interface Window {
    __fabushiWebMcp?: FabushiWebMcpRegistry;
  }
}

const localTools = new Map<string, WebMcpTool>();

function publicTool(tool: WebMcpTool): Omit<WebMcpTool, "execute"> {
  const { execute: _execute, ...metadata } = tool;
  return metadata;
}

function installFallbackRegistry(): void {
  if (typeof window === "undefined" || window.__fabushiWebMcp) return;
  Object.defineProperty(window, "__fabushiWebMcp", {
    configurable: true,
    enumerable: false,
    value: {
      version: 1 as const,
      list: () => [...localTools.values()].map(publicTool),
      call: async (name: string, input: Record<string, unknown> = {}) => {
        const tool = localTools.get(name);
        if (!tool) throw new Error(`Unknown WebMCP tool: ${name}`);
        return tool.execute(input);
      },
    },
  });
}

function parseNativeResult(value: string): unknown {
  try {
    return JSON.parse(value);
  } catch {
    return value;
  }
}

export function supportsNativeWebMcp(): boolean {
  return typeof document !== "undefined" && typeof document.modelContext?.registerTool === "function";
}

export function listRegisteredWebMcpTools(): Array<Omit<WebMcpTool, "execute">> {
  return [...localTools.values()].map(publicTool);
}

export async function callRegisteredWebMcpTool(
  name: string,
  input: Record<string, unknown> = {},
): Promise<unknown> {
  const context = typeof document !== "undefined" ? document.modelContext : undefined;
  if (context?.getTools && context.executeTool) {
    const registered = await context.getTools();
    const tool = registered.find((candidate) => candidate.name === name);
    if (tool) {
      return parseNativeResult(await context.executeTool(tool, input));
    }
  }
  const tool = localTools.get(name);
  if (!tool) throw new Error(`Unknown WebMCP tool: ${name}`);
  return tool.execute(input);
}

export function registerWebMcpTool(tool: WebMcpTool): () => void {
  const name = tool.name.trim();
  if (!name) throw new Error("WebMCP tool name is required");
  if (!tool.description.trim()) throw new Error(`WebMCP tool ${name} requires a description`);
  localTools.set(name, tool);
  installFallbackRegistry();

  const context = typeof document !== "undefined" ? document.modelContext : undefined;
  const controller = typeof AbortController !== "undefined" ? new AbortController() : undefined;
  if (context?.registerTool) {
    const registration = context.registerTool(
      tool,
      controller ? { signal: controller.signal } : undefined,
    );
    if (registration && typeof registration.then === "function") {
      void registration.catch((error: unknown) => {
        // The Fabushi registry remains available as a compatibility path when a
        // browser exposes the draft API but rejects registration by policy.
        console.warn(`WebMCP registration failed for ${name}`, error);
      });
    }
  }

  return () => {
    if (localTools.get(name) !== tool) return;
    localTools.delete(name);
    // The current WebMCP draft unregisters tools by aborting the signal that
    // was supplied to registerTool(). No separate unregisterTool API is used.
    controller?.abort();
  };
}

export function exposeMcpToolsAsWebMcp(
  tools: readonly McpTool[],
  callTool: (name: string, args: Record<string, unknown>) => Promise<McpToolResult>,
): () => void {
  const disposers = tools.map((tool) => registerWebMcpTool({
    name: tool.name,
    title: tool.title,
    description: tool.description?.trim() || tool.title?.trim() || tool.name,
    inputSchema: tool.inputSchema,
    annotations: {
      ...(tool.annotations?.readOnlyHint === undefined
        ? {}
        : { readOnlyHint: tool.annotations.readOnlyHint }),
    },
    execute: (input) => callTool(tool.name, input),
  }));
  return () => {
    for (const dispose of disposers.reverse()) dispose();
  };
}
