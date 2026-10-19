/// <reference path="./ws-shim.d.ts" />

import { join } from "node:path";

import WebSocket from "ws";

import type { FabushiDeviceAgentSession } from "../../electron-main/account/auth-service-contract.js";

export const OFFICIAL_FABUSHI_DEVICE_GATEWAY_URL = "wss://fabushi-mcp.ombhrum.com/agent";
export const FABUSHI_DEVICE_GATEWAY_URL_ENV = "FABUSHI_REMOTE_DEVICE_GATEWAY_URL";
export const FABUSHI_APP_AGENT_STATUS_TOOL = "fabushi.app.status";
export const FABUSHI_APP_AGENT_TOOLS = [
  "fabushi.app.status", "fabushi.app.snapshot", "fabushi.app.find",
  "fabushi.app.action", "fabushi.app.wait", "fabushi.app.assert",
] as const;
const READ_ONLY_APP_AGENT_TOOLS = new Set<string>([
  "fabushi.app.status", "fabushi.app.snapshot", "fabushi.app.find",
  "fabushi.app.wait", "fabushi.app.assert",
]);

const DEFAULT_POLL_MS = 30_000;
const DEFAULT_HEARTBEAT_MS = 25_000;
const SESSION_EXPIRY_LEEWAY_MS = 30_000;
const MAX_RESULT_BYTES = 64 * 1024;

export interface OfficialDeviceRegistrySocket {
  readonly readyState: number;
  send(data: string, callback?: (error?: Error) => void): void;
  close(code?: number, reason?: string): void;
  terminate?(): void;
  on(event: string, listener: (...args: unknown[]) => void): unknown;
}

export interface FabushiOfficialDeviceRegistryOptions {
  readonly isPackaged: boolean;
  readonly env: NodeJS.ProcessEnv;
  readonly userDataPath: string;
  readonly deviceName?: string;
  readonly getDeviceAgentSession: () => Promise<FabushiDeviceAgentSession | null>;
  readonly connect?: (url: string, accessToken: string) => OfficialDeviceRegistrySocket;
  readonly surfaceStatus?: () => Promise<unknown>;
  readonly surfaceCall?: (toolName: string, input: Record<string, unknown>) => Promise<unknown>;
  readonly now?: () => number;
  readonly pollMs?: number;
  readonly heartbeatMs?: number;
  readonly reportFailure?: (operation: string, error: unknown) => void;
}

function sessionKey(session: FabushiDeviceAgentSession): string {
  return [session.deviceId, session.sessionId, session.accessToken].join("\0");
}

function isUsableSession(session: FabushiDeviceAgentSession | null, now: number): session is FabushiDeviceAgentSession {
  return session != null
    && session.accessToken.length >= 24
    && session.accessToken.length <= 16 * 1024
    && !/\s/u.test(session.accessToken)
    && /^[a-zA-Z0-9._:-]{1,128}$/u.test(session.deviceId)
    && session.sessionId.length > 0
    && session.sessionId.length <= 200
    && Number.isFinite(session.accessTokenExpiresAt)
    && session.accessTokenExpiresAt > now + SESSION_EXPIRY_LEEWAY_MS;
}

export function resolveFabushiDeviceGatewayUrl(isPackaged: boolean, env: NodeJS.ProcessEnv): string | null {
  if (isPackaged) return OFFICIAL_FABUSHI_DEVICE_GATEWAY_URL;
  const configured = String(env[FABUSHI_DEVICE_GATEWAY_URL_ENV] ?? "").trim();
  if (!configured) return null;
  const url = new URL(configured);
  if (url.protocol !== "wss:"
    || url.pathname !== "/agent"
    || url.username
    || url.password
    || url.search
    || url.hash) {
    throw new Error("FABUSHI_REMOTE_DEVICE_GATEWAY_URL must be a clean wss:// URL ending in /agent.");
  }
  return url.toString();
}

function defaultConnect(url: string, accessToken: string): OfficialDeviceRegistrySocket {
  return new WebSocket(url, { headers: { Authorization: "Bearer " + accessToken } }) as unknown as OfficialDeviceRegistrySocket;
}

async function defaultSurfaceStatus(discoveryPath: string): Promise<unknown> {
  const module = await import("../../../chatgpt-vps-control/lib/app-agent-surface-client.js");
  return await module.createAppAgentSurfaceClient({ discoveryPath }).status();
}

function appToolDescriptor(name: string): Record<string, unknown> {
  const operation = name.slice("fabushi.app.".length);
  return {
    name,
    title: "Fabushi desktop " + operation,
    description: "Operate the authenticated Fabushi desktop semantic App Agent Surface.",
    inputSchema: { type: "object", properties: {}, additionalProperties: true },
    outputSchema: { type: "object", additionalProperties: true },
    annotations: { readOnlyHint: READ_ONLY_APP_AGENT_TOOLS.has(name), destructiveHint: !READ_ONLY_APP_AGENT_TOOLS.has(name), idempotentHint: READ_ONLY_APP_AGENT_TOOLS.has(name), openWorldHint: false },
  };
}

function statusToolDescriptor(): Record<string, unknown> {
  return {
    ...appToolDescriptor(FABUSHI_APP_AGENT_STATUS_TOOL),
    title: "Fabushi desktop status",
    description: "Read the current Fabushi desktop App Agent Surface status. This operation does not modify device state.",
    inputSchema: { type: "object", properties: {}, additionalProperties: false },
    outputSchema: { type: "object", additionalProperties: true },
    annotations: {
      readOnlyHint: true,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: false,
    },
  };
}

function safeJsonResult(value: unknown, accessToken: string): { readonly structuredContent: unknown; readonly text: string } {
  let text = JSON.stringify(value ?? null);
  if (text.includes(accessToken)) text = text.split(accessToken).join("[redacted]");
  if (Buffer.byteLength(text, "utf8") > MAX_RESULT_BYTES) {
    throw new Error("Fabushi App Agent status response exceeded the bounded result size.");
  }
  return { structuredContent: JSON.parse(text), text };
}

function isReadOnlyPreflight(value: unknown): boolean {
  if (value == null || typeof value !== "object" || Array.isArray(value)) return false;
  const source = value as Record<string, unknown>;
  return source.impact === "none" && Array.isArray(source.targets) && source.targets.length === 0;
}

function decodeMessage(raw: unknown): Record<string, unknown> | null {
  try {
    const text = typeof raw === "string"
      ? raw
      : Buffer.isBuffer(raw)
        ? raw.toString("utf8")
        : String(raw);
    const parsed: unknown = JSON.parse(text);
    return parsed != null && typeof parsed === "object" && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : null;
  } catch {
    return null;
  }
}

export class FabushiOfficialDeviceRegistryService {
  private readonly now: () => number;
  private readonly connect: (url: string, accessToken: string) => OfficialDeviceRegistrySocket;
  private readonly surfaceStatus: () => Promise<unknown>;
  private readonly surfaceCall: (toolName: string, input: Record<string, unknown>) => Promise<unknown>;
  private readonly pollMs: number;
  private readonly heartbeatMs: number;
  private started = false;
  private syncing: Promise<void> | null = null;
  private pollTimer: ReturnType<typeof setInterval> | undefined;
  private heartbeatTimer: ReturnType<typeof setInterval> | undefined;
  private active: { readonly key: string; readonly session: FabushiDeviceAgentSession; readonly socket: OfficialDeviceRegistrySocket } | undefined;

  constructor(private readonly options: FabushiOfficialDeviceRegistryOptions) {
    this.now = options.now ?? Date.now;
    this.connect = options.connect ?? defaultConnect;
    const discoveryPath = join(options.userDataPath, "agent-surface", "bridge.json");
    this.surfaceStatus = options.surfaceStatus ?? (async () => await defaultSurfaceStatus(discoveryPath));
    this.surfaceCall = options.surfaceCall ?? (async (toolName, input) => { const module = await import("../../../chatgpt-vps-control/lib/app-agent-surface-client.js"); return await module.createAppAgentSurfaceClient({ discoveryPath }).call(toolName.slice("fabushi.app.".length), input); });
    this.pollMs = Math.max(1_000, options.pollMs ?? DEFAULT_POLL_MS);
    this.heartbeatMs = Math.max(1_000, options.heartbeatMs ?? DEFAULT_HEARTBEAT_MS);
  }

  start(): void {
    if (this.started) return;
    this.started = true;
    void this.syncNow();
    this.pollTimer = setInterval(() => { void this.syncNow(); }, this.pollMs);
    this.pollTimer.unref?.();
  }

  async stop(): Promise<void> {
    this.started = false;
    if (this.pollTimer != null) clearInterval(this.pollTimer);
    this.pollTimer = undefined;
    this.disconnect("device registry stopped");
    await this.syncing?.catch(() => {});
  }

  syncNow(): Promise<void> {
    if (this.syncing != null) return this.syncing;
    this.syncing = this.syncInternal()
      .catch((error) => this.options.reportFailure?.("sync", error))
      .finally(() => { this.syncing = null; });
    return this.syncing;
  }

  private async syncInternal(): Promise<void> {
    const gatewayUrl = resolveFabushiDeviceGatewayUrl(this.options.isPackaged, this.options.env);
    const session = await this.options.getDeviceAgentSession().catch(() => null);
    if (!gatewayUrl || !isUsableSession(session, this.now())) {
      this.disconnect("account unavailable");
      return;
    }
    const key = sessionKey(session);
    if (this.active?.key === key && this.active.socket.readyState <= 1) return;
    this.disconnect("account changed");
    const socket = this.connect(gatewayUrl, session.accessToken);
    this.active = { key, session, socket };
    socket.on("open", () => { void this.onOpen(socket, key); });
    socket.on("message", (raw) => { void this.onMessage(socket, key, raw); });
    socket.on("close", () => {
      if (this.active?.socket !== socket) return;
      this.clearHeartbeat();
      this.active = undefined;
    });
    socket.on("error", () => {
      if (this.active?.socket === socket) socket.terminate?.();
    });
  }

  private async onOpen(socket: OfficialDeviceRegistrySocket, key: string): Promise<void> {
    if (this.active?.socket !== socket || this.active.key !== key) return;
    const current = await this.options.getDeviceAgentSession().catch(() => null);
    if (!isUsableSession(current, this.now()) || sessionKey(current) !== key) {
      this.disconnect("stale account before registration");
      return;
    }
    this.send(socket, {
      type: "register",
      deviceId: current.deviceId,
      name: this.options.deviceName ?? "Fabushi Desktop",
      platform: process.platform,
      capabilities: [...FABUSHI_APP_AGENT_TOOLS],
      tools: FABUSHI_APP_AGENT_TOOLS.map((name) => name === FABUSHI_APP_AGENT_STATUS_TOOL ? statusToolDescriptor() : appToolDescriptor(name)),
      toolSchemaVersion: "fabushi-app-agent-surface-v1",
      generation: current.sessionId,
      leaseSeconds: 14_400,
      metadata: { kind: "fabushi-desktop", owner: "canonical-host" },
    });
    this.clearHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      if (this.active?.socket !== socket || socket.readyState !== 1) return;
      this.send(socket, { type: "heartbeat", at: this.now() });
    }, this.heartbeatMs);
    this.heartbeatTimer.unref?.();
  }

  private async onMessage(socket: OfficialDeviceRegistrySocket, key: string, raw: unknown): Promise<void> {
    if (this.active?.socket !== socket || this.active.key !== key) return;
    const message = decodeMessage(raw);
    if (message == null || message.type === "registered") return;
    if (message.type !== "call") return;
    const requestId = typeof message.requestId === "string" ? message.requestId.slice(0, 128) : "";
    if (!requestId) return;

    const currentBefore = await this.options.getDeviceAgentSession().catch(() => null);
    if (!isUsableSession(currentBefore, this.now()) || sessionKey(currentBefore) !== key) {
      this.disconnect("account revoked before device call");
      return;
    }

    try {
      const toolName = typeof message.toolName === "string" ? message.toolName : "";
      if (!FABUSHI_APP_AGENT_TOOLS.includes(toolName as typeof FABUSHI_APP_AGENT_TOOLS[number])) {
        throw new Error("The requested tool is not registered by the canonical Fabushi Host surface.");
      }
      if (!READ_ONLY_APP_AGENT_TOOLS.has(toolName) && !isReadOnlyPreflight(message.diskPreflight)) {
        throw new Error("diskPreflight is required for non-read-only Fabushi App Agent tools.");
      }
      let input: Record<string, unknown> = {};
      if (typeof message.argumentsJson === "string" && message.argumentsJson.length > 0) input = JSON.parse(message.argumentsJson);
      else if (message.arguments != null && typeof message.arguments === "object" && !Array.isArray(message.arguments)) input = message.arguments as Record<string, unknown>;
      const resultValue = toolName === FABUSHI_APP_AGENT_STATUS_TOOL ? await this.surfaceStatus() : await this.surfaceCall(toolName, input);
      const currentAfter = await this.options.getDeviceAgentSession().catch(() => null);
      if (!isUsableSession(currentAfter, this.now()) || sessionKey(currentAfter) !== key) {
        this.disconnect("account changed during device call");
        return;
      }
      const bounded = safeJsonResult(resultValue, currentAfter.accessToken);
      this.send(socket, {
        type: "result",
        requestId,
        ok: true,
        result: {
          isError: false,
          content: [{ type: "text", text: bounded.text }],
          structuredContent: bounded.structuredContent,
        },
      });
    } catch (error) {
      if (this.active?.socket !== socket || this.active.key !== key) return;
      this.send(socket, {
        type: "result",
        requestId,
        ok: false,
        error: error instanceof Error ? error.message.slice(0, 1_000) : "Fabushi device call failed.",
      });
    }
  }

  private send(socket: OfficialDeviceRegistrySocket, value: Record<string, unknown>): void {
    if (socket.readyState !== 1) return;
    socket.send(JSON.stringify(value), (error) => {
      if (error != null) this.options.reportFailure?.("send", error);
    });
  }

  private clearHeartbeat(): void {
    if (this.heartbeatTimer != null) clearInterval(this.heartbeatTimer);
    this.heartbeatTimer = undefined;
  }

  private disconnect(reason: string): void {
    this.clearHeartbeat();
    const active = this.active;
    this.active = undefined;
    if (active == null) return;
    try { active.socket.close(1000, reason.slice(0, 120)); }
    catch { active.socket.terminate?.(); }
  }
}

export function createFabushiOfficialDeviceRegistryService(
  options: FabushiOfficialDeviceRegistryOptions,
): FabushiOfficialDeviceRegistryService {
  return new FabushiOfficialDeviceRegistryService(options);
}
