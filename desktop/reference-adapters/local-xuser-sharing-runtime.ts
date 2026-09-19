import {
  SandXuserSharingService,
  STATE_RECONCILE_INTERVAL_MS,
  SELF_IDENTITY_RETRY_MS,
} from "../../reference/grok-bot-0.18/source/host/extensions/cross-user-sharing/xuser-sharing-service.js";
import { XUSER_RELAY_POLL_INTERVAL_MS } from "../../reference/grok-bot-0.18/source/host/extensions/cross-user-sharing/xuser-relay.js";
import {
  createDeadlinePolicy,
  createExpiryPolicy,
  createPollingPolicy,
  realClock,
} from "../../reference/grok-bot-0.18/source/internal/scheduling.js";
import { REMOTE_MEMBER_TURN_TIMEOUT_MS } from "../../reference/grok-bot-0.18/source/host/groups/xuser.js";

type Loose = Record<string, any>;
const DISABLED_STATE = Object.freeze({
  isEnabled: false,
  selfAuthId: null,
  pendingJoinRequests: [],
  rooms: [],
  typingUsers: [],
});

function isLoopback(hostname: string): boolean {
  return ["127.0.0.1", "localhost", "::1", "[::1]"].includes(hostname);
}

export function normalizeSharingBackendUrl(value: unknown): string | null {
  const raw = String(value ?? "").trim();
  if (!raw) return null;
  const url = new URL(raw);
  if (url.protocol !== "https:" && !(url.protocol === "http:" && isLoopback(url.hostname))) {
    throw new Error("FABUSHI_SHARING_BACKEND_URL must use HTTPS (HTTP is allowed only for loopback development).");
  }
  if (!url.pathname.endsWith("/")) url.pathname += "/";
  url.search = "";
  url.hash = "";
  return url.toString();
}

export interface LocalXuserSharingRuntimeOptions {
  readonly backendUrl?: string | null;
  readonly getAccessToken: (options: { backendUrl: string }) => Promise<string>;
  readonly getSelfAuthId: () => Promise<string | null>;
  readonly manager: Loose;
  readonly emitSharing: (state: unknown) => void;
  readonly resolveAttachment?: (url: string) => Promise<{ data: Uint8Array; mimeType: string } | null>;
  readonly fetchImpl?: typeof fetch;
}

export function createLocalXuserSharingRuntime(options: LocalXuserSharingRuntimeOptions) {
  const backendUrl = normalizeSharingBackendUrl(options.backendUrl);
  let service: SandXuserSharingService | null = null;
  let starting: Promise<void> | null = null;
  const configured = backendUrl != null;
  const disabled = () => ({...DISABLED_STATE,pendingJoinRequests:[],rooms:[],typingUsers:[]});

  async function authorizedAuthId(): Promise<string | null> {
    if (!configured) return null;
    try {
      const value = await options.getSelfAuthId();
      return typeof value === "string" && value.trim() ? value.trim() : null;
    } catch {
      return null;
    }
  }

  async function ensureService(): Promise<SandXuserSharingService | null> {
    const authId = await authorizedAuthId();
    if (!configured || authId == null) {
      service?.stop();
      service = null;
      starting = null;
      return null;
    }
    if (service == null) {
      service = new SandXuserSharingService({
        getAccessToken: options.getAccessToken,
        getBackendUrl: () => backendUrl!,
        getSelfAuthId: authorizedAuthId,
        isEnabled: () => true,
        manager: options.manager,
        emitSharing: options.emitSharing,
        resolveAttachment: options.resolveAttachment ?? (async () => null),
        fetchImpl: options.fetchImpl,
        isNotifyConnected: () => false,
        isNotifySafetyPollEnabled: () => true,
        timing: {
          clock: realClock,
          relayPoll: createPollingPolicy(realClock, {
            name: "fabushi-cross-user-sharing-relay-poll",
            intervalMs: XUSER_RELAY_POLL_INTERVAL_MS,
          }),
          reconcilePoll: createPollingPolicy(realClock, {
            name: "fabushi-cross-user-sharing-state-reconcile",
            intervalMs: STATE_RECONCILE_INTERVAL_MS,
          }),
          selfIdentityRetry: createPollingPolicy(realClock, {
            name: "fabushi-cross-user-sharing-self-identity",
            intervalMs: SELF_IDENTITY_RETRY_MS,
          }),
          remoteTurnDeadline: createDeadlinePolicy(realClock, {
            name: "fabushi-cross-user-sharing-remote-turn",
            timeoutMs: REMOTE_MEMBER_TURN_TIMEOUT_MS,
          }),
          typingExpiry: (ttlMs: number) => createExpiryPolicy(realClock, {
            name: "fabushi-cross-user-sharing-typing-expiry",
            ttlMs,
          }),
        },
      });
      starting = service.start().finally(() => { starting = null; });
    }
    if (starting != null) await starting;
    return service;
  }

  return {
    configured,
    backendUrl,
    async start() { return (await ensureService()) != null; },
    async getState() {
      const active = await ensureService();
      return active == null ? disabled() : active.getState();
    },
    async createRoomFromAgent(agentId: string) {
      const active = await ensureService();
      return active == null
        ? {status:"error",message:configured?"Sign in to use shared rooms.":"Sharing backend is not configured."}
        : active.createRoomFromAgent(agentId);
    },
    async createRoomInvite(roomId: string) {
      const active = await ensureService();
      return active == null
        ? {status:"error",message:configured?"Sign in to use shared rooms.":"Sharing backend is not configured."}
        : active.createRoomInvite(roomId);
    },
    async joinRoom(link: string) {
      const active = await ensureService();
      return active == null
        ? {status:"error",message:configured?"Sign in to use shared rooms.":"Sharing backend is not configured."}
        : active.joinRoom(link);
    },
    async respondToJoinRequest(args: Loose) {
      const active = await ensureService();
      return active == null ? disabled() : active.respondToJoinRequest(args as any);
    },
    async createSharedRoom(args: Loose) {
      const active = await ensureService();
      return active == null
        ? {status:"error",message:configured?"Sign in to use shared rooms.":"Sharing backend is not configured."}
        : active.createSharedRoom(args as any);
    },
    async addOwnAgent(args: Loose) {
      const active = await ensureService();
      return active == null ? disabled() : active.addOwnAgent(args as any);
    },
    async removeOwnAgent(roomId: string, agentId: string) {
      const active = await ensureService();
      return active == null ? disabled() : active.removeOwnAgent(roomId, agentId);
    },
    async setRoomTyping(roomId: string, isTyping: boolean) {
      const active = await ensureService();
      if (active != null) await active.setRoomTyping(roomId, isTyping);
    },
    async leaveSharedRoom(roomId: string, targetAuthId?: string) {
      const active = await ensureService();
      return active == null ? disabled() : active.leaveSharedRoom(roomId, targetAuthId);
    },
    async noteAgentDeleted(agentId: string) {
      const active = await ensureService();
      if (active != null) await active.noteAgentDeleted(agentId);
    },
    async publishRoomEntry(roomId: string, entry: Loose) {
      const active = await ensureService();
      if (active == null) return false;
      active.buildManagerDelegate().publishRoomEntry(roomId, entry as any);
      return true;
    },
    requestRelayDrain() { service?.requestRelayDrain(); },
    dispose() { service?.stop(); service = null; starting = null; },
  };
}
