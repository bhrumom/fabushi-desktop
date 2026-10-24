import type { JsonRpcRequest, JsonRpcResponse, McpToolResult } from "./types";

type Listener = (params: unknown) => void;

type BridgeInitMessage = {
  type: "mahayana/bridge-init";
  bridgeVersion: "2.0";
  pluginInstanceId: string;
  nonce: string;
  grants: string[];
};

type PortEnvelope = {
  pluginInstanceId: string;
  nonce: string;
  payload: JsonRpcRequest | (JsonRpcResponse & { method?: string; params?: unknown });
};

/**
 * MCP Apps bridge for sandboxed plugin UIs.
 *
 * The iframe has an opaque origin when hosted with `sandbox="allow-scripts"`,
 * therefore the parent performs one bootstrap postMessage with a transferred
 * MessagePort. All RPC after that bootstrap is bound to that private port plus
 * the plugin instance id and a per-load nonce; no request is sent through
 * `window.postMessage("*")`.
 */
export class McpAppBridge {
  private nextId = 1;
  private pending = new Map<number | string, (response: JsonRpcResponse) => void>();
  private listeners = new Map<string, Set<Listener>>();
  private port: MessagePort | null = null;
  private pluginInstanceId = "";
  private nonce = "";
  private grants = new Set<string>();
  private disposed = false;
  private readyResolve!: () => void;
  private readyReject!: (error: Error) => void;
  private readonly readyPromise: Promise<void>;

  constructor(private readonly target: Window = window.parent) {
    this.readyPromise = new Promise<void>((resolve, reject) => {
      this.readyResolve = resolve;
      this.readyReject = reject;
    });
    window.addEventListener("message", this.handleBootstrap);
  }

  ready(): Promise<void> {
    return this.readyPromise;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    window.removeEventListener("message", this.handleBootstrap);
    this.port?.removeEventListener("message", this.handlePortMessage);
    this.port?.close();
    this.port = null;
    for (const done of this.pending.values()) {
      done({ jsonrpc: "2.0", id: 0, error: { code: -32001, message: "MCP App bridge disposed" } });
    }
    this.pending.clear();
    this.listeners.clear();
    this.readyReject(new Error("MCP App bridge disposed before initialization"));
  }

  async callTool(name: string, args: Record<string, unknown> = {}): Promise<McpToolResult> {
    return this.request("tools/call", { name, arguments: args }) as Promise<McpToolResult>;
  }

  async sendMessage(content: string): Promise<unknown> {
    return this.request("ui/message", { content });
  }

  async updateModelContext(context: Record<string, unknown>): Promise<unknown> {
    return this.request("ui/update-model-context", context);
  }

  on(method: "ui/notifications/tool-input" | "ui/notifications/tool-result", listener: Listener): () => void {
    const listeners = this.listeners.get(method) ?? new Set<Listener>();
    listeners.add(listener);
    this.listeners.set(method, listeners);
    return () => listeners.delete(listener);
  }

  private async request(method: string, params: Record<string, unknown>): Promise<unknown> {
    await this.readyPromise;
    if (!this.port || this.disposed) throw new Error("MCP App bridge is not available");
    if (!this.grants.has(method)) throw new Error(`MCP App bridge capability not granted: ${method}`);
    const id = this.nextId++;
    const request: JsonRpcRequest = { jsonrpc: "2.0", id, method, params };
    return new Promise((resolve, reject) => {
      this.pending.set(id, (response) => {
        if (response.error) reject(new Error(response.error.message));
        else resolve(response.result);
      });
      const envelope: PortEnvelope = {
        pluginInstanceId: this.pluginInstanceId,
        nonce: this.nonce,
        payload: request,
      };
      this.port!.postMessage(envelope);
    });
  }

  private handleBootstrap = (event: MessageEvent): void => {
    if (this.disposed || this.port || event.source !== this.target) return;
    const init = event.data as Partial<BridgeInitMessage> | undefined;
    if (
      !init ||
      init.type !== "mahayana/bridge-init" ||
      init.bridgeVersion !== "2.0" ||
      typeof init.pluginInstanceId !== "string" ||
      !init.pluginInstanceId ||
      typeof init.nonce !== "string" ||
      init.nonce.length < 16 ||
      !Array.isArray(init.grants) ||
      event.ports.length !== 1
    ) return;

    this.pluginInstanceId = init.pluginInstanceId;
    this.nonce = init.nonce;
    this.grants = new Set(init.grants.filter((grant): grant is string => typeof grant === "string"));
    this.port = event.ports[0];
    this.port.addEventListener("message", this.handlePortMessage);
    this.port.start();
    window.removeEventListener("message", this.handleBootstrap);
    this.readyResolve();
  };

  private handlePortMessage = (event: MessageEvent): void => {
    const envelope = event.data as Partial<PortEnvelope> | undefined;
    if (
      !envelope ||
      envelope.pluginInstanceId !== this.pluginInstanceId ||
      envelope.nonce !== this.nonce ||
      !envelope.payload
    ) return;
    const message = envelope.payload as JsonRpcResponse & { method?: string; params?: unknown };
    if (message.jsonrpc !== "2.0") return;
    if (message.id !== undefined && this.pending.has(message.id)) {
      const done = this.pending.get(message.id)!;
      this.pending.delete(message.id);
      done(message);
      return;
    }
    if (message.method) {
      for (const listener of this.listeners.get(message.method) ?? []) listener(message.params);
    }
  };
}
