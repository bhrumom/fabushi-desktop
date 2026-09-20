import { invokeNativeDesktop, subscribeNativeDesktopEvents } from "./native-desktop";

export type CollaborationScope = "local-device" | "fabushi-platform";

export interface SharedRoomSummary {
  readonly id: string;
  readonly name: string;
  readonly ownerAgentId: string | null;
  readonly memberAgentIds: readonly string[];
  readonly scope: CollaborationScope;
  readonly createdAtMs: number;
  readonly updatedAtMs: number;
  readonly ownAgentIds?: readonly string[];
  readonly memberCount?: number;
  readonly isOwner?: boolean;
}

export interface SharedRoomInvite {
  readonly token: string;
  readonly roomId: string;
  readonly expiresAtMs: number;
}

export interface SharedRoomJoinRequest {
  readonly id: string;
  readonly roomId: string;
  readonly agentId: string;
  readonly displayName: string;
  readonly status: "pending" | "accepted" | "rejected";
  readonly createdAtMs: number;
  readonly resolvedAtMs?: number;
  readonly isOwnRequest?: boolean;
}

export interface SharedRoomTypingState {
  readonly roomId: string;
  readonly participantId: string;
  readonly isTyping: boolean;
  readonly updatedAtMs: number;
}

export interface SharingState {
  readonly scope: CollaborationScope;
  readonly rooms: readonly SharedRoomSummary[];
  readonly joinRequests: readonly SharedRoomJoinRequest[];
  readonly typing?: readonly SharedRoomTypingState[];
  readonly fetchedAtMs?: number;
}

export type CollaborationEvent =
  | { readonly type: "state.changed"; readonly state: SharingState }
  | {
      readonly type: "typing.changed";
      readonly roomId: string;
      readonly participantId: string;
      readonly isTyping: boolean;
      readonly updatedAtMs: number;
    };

type StoredState = {
  rooms: SharedRoomSummary[];
  invites: SharedRoomInvite[];
  joinRequests: SharedRoomJoinRequest[];
};

const STORAGE_KEY = "fabushi.collaboration.local.v1";
const CHANNEL_NAME = "fabushi-collaboration-local-v1";
const INVITE_TTL_MS = 24 * 60 * 60 * 1_000;
const MAX_ROOMS = 100;
const MAX_INVITES = 300;
const MAX_JOIN_REQUESTS = 500;

function localStorageOrNull(): Storage | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function randomId(prefix: string): string {
  const suffix = typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`;
  return `${prefix}-${suffix}`;
}

function uniqueIds(values: readonly string[]): string[] {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))];
}

function normalizeStoredState(value: unknown): StoredState {
  if (!value || typeof value !== "object") return { rooms: [], invites: [], joinRequests: [] };
  const candidate = value as Partial<StoredState>;
  const rooms = Array.isArray(candidate.rooms)
    ? candidate.rooms.filter((room): room is SharedRoomSummary =>
        Boolean(
          room
          && typeof room.id === "string"
          && typeof room.name === "string"
          && (room.ownerAgentId === null || typeof room.ownerAgentId === "string")
          && Array.isArray(room.memberAgentIds)
          && room.scope === "local-device"
          && Number.isFinite(room.createdAtMs)
          && Number.isFinite(room.updatedAtMs),
        ),
      ).slice(-MAX_ROOMS)
    : [];
  const invites = Array.isArray(candidate.invites)
    ? candidate.invites.filter((invite): invite is SharedRoomInvite =>
        Boolean(
          invite
          && typeof invite.token === "string"
          && typeof invite.roomId === "string"
          && Number.isFinite(invite.expiresAtMs),
        ),
      ).slice(-MAX_INVITES)
    : [];
  const joinRequests = Array.isArray(candidate.joinRequests)
    ? candidate.joinRequests.filter((request): request is SharedRoomJoinRequest =>
        Boolean(
          request
          && typeof request.id === "string"
          && typeof request.roomId === "string"
          && typeof request.agentId === "string"
          && typeof request.displayName === "string"
          && ["pending", "accepted", "rejected"].includes(request.status)
          && Number.isFinite(request.createdAtMs),
        ),
      ).slice(-MAX_JOIN_REQUESTS)
    : [];
  return { rooms, invites, joinRequests };
}

function loadState(): StoredState {
  const storage = localStorageOrNull();
  if (!storage) return { rooms: [], invites: [], joinRequests: [] };
  try {
    return normalizeStoredState(JSON.parse(storage.getItem(STORAGE_KEY) ?? "null"));
  } catch {
    return { rooms: [], invites: [], joinRequests: [] };
  }
}

function saveState(state: StoredState): void {
  const storage = localStorageOrNull();
  if (!storage) return;
  try {
    storage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Collaboration remains usable in-memory when persistence is unavailable.
  }
}

function isPlatformRoom(value: unknown): value is SharedRoomSummary {
  if (!value || typeof value !== "object") return false;
  const room = value as Partial<SharedRoomSummary>;
  return Boolean(
    typeof room.id === "string"
    && typeof room.name === "string"
    && (room.ownerAgentId === null || typeof room.ownerAgentId === "string")
    && Array.isArray(room.memberAgentIds)
    && room.memberAgentIds.every((agentId) => typeof agentId === "string")
    && room.scope === "fabushi-platform"
    && Number.isFinite(room.createdAtMs)
    && Number.isFinite(room.updatedAtMs),
  );
}

function normalizePlatformState(value: unknown): SharingState {
  const candidate = value && typeof value === "object" ? value as Partial<SharingState> : {};
  const rooms = Array.isArray(candidate.rooms) ? candidate.rooms.filter(isPlatformRoom) : [];
  const roomIds = new Set(rooms.map((room) => room.id));
  const joinRequests = Array.isArray(candidate.joinRequests)
    ? candidate.joinRequests.filter((request): request is SharedRoomJoinRequest => Boolean(
        request
        && typeof request.id === "string"
        && typeof request.roomId === "string"
        && typeof request.agentId === "string"
        && typeof request.displayName === "string"
        && ["pending", "accepted", "rejected"].includes(request.status)
        && Number.isFinite(request.createdAtMs),
      ))
    : [];
  const typing = Array.isArray(candidate.typing)
    ? candidate.typing.filter((entry): entry is SharedRoomTypingState => Boolean(
        entry
        && typeof entry.roomId === "string"
        && roomIds.has(entry.roomId)
        && typeof entry.participantId === "string"
        && typeof entry.isTyping === "boolean"
        && Number.isFinite(entry.updatedAtMs),
      ))
    : [];
  return {
    scope: "fabushi-platform",
    rooms,
    joinRequests,
    typing,
    fetchedAtMs: typeof candidate.fetchedAtMs === "number" && Number.isFinite(candidate.fetchedAtMs)
      ? candidate.fetchedAtMs
      : Date.now(),
  };
}

export class PlatformCollaborationProvider {
  private state: SharingState = {
    scope: "fabushi-platform",
    rooms: [],
    joinRequests: [],
    typing: [],
    fetchedAtMs: 0,
  };
  private readonly listeners = new Set<(event: CollaborationEvent) => void>();
  private nativeUnsubscribe: (() => void) | null = null;
  private pollTimer: ReturnType<typeof setTimeout> | null = null;
  private refreshPromise: Promise<SharingState> | null = null;
  private closed = false;

  snapshot(agentId?: string): SharingState {
    if (!agentId) return this.state;
    const rooms = this.state.rooms.filter((room) => room.memberAgentIds.includes(agentId));
    const roomIds = new Set(rooms.map((room) => room.id));
    return {
      ...this.state,
      rooms,
      joinRequests: this.state.joinRequests.filter((request) => roomIds.has(request.roomId) || request.agentId === agentId),
      typing: this.state.typing?.filter((entry) => roomIds.has(entry.roomId)),
    };
  }

  async refresh(agentId?: string): Promise<SharingState> {
    if (this.closed) throw new Error("Platform collaboration provider is closed");
    if (!this.refreshPromise) {
      this.refreshPromise = invokeNativeDesktop<SharingState>("getSharingState", {})
        .then((state) => {
          const next = normalizePlatformState(state);
          this.applyState(next);
          return next;
        })
        .finally(() => { this.refreshPromise = null; });
    }
    await this.refreshPromise;
    return this.snapshot(agentId);
  }

  createRoomFromAgent(agentId: string, name?: string): Promise<SharedRoomSummary> {
    return this.mutate<SharedRoomSummary>("createRoomFromAgent", { agentId, name });
  }

  createSharedRoom(name: string, memberAgentIds: readonly string[], ownerAgentId: string | null = null): Promise<SharedRoomSummary> {
    return this.mutate<SharedRoomSummary>("createSharedRoom", { name, memberAgentIds: [...memberAgentIds], ownerAgentId });
  }

  createRoomInvite(roomId: string): Promise<SharedRoomInvite> {
    return this.mutate<SharedRoomInvite>("createRoomInvite", { roomId }, false);
  }

  joinSharedRoom(token: string, agentId: string, displayName?: string): Promise<SharedRoomJoinRequest> {
    return this.mutate<SharedRoomJoinRequest>("joinSharedRoom", { token, agentId, displayName });
  }

  respondToRoomJoinRequest(requestId: string, accept: boolean): Promise<SharedRoomJoinRequest> {
    return this.mutate<SharedRoomJoinRequest>("respondToRoomJoinRequest", { requestId, accept });
  }

  addOwnAgentToSharedRoom(roomId: string, agentId: string): Promise<SharedRoomSummary> {
    return this.mutate<SharedRoomSummary>("addOwnAgentToSharedRoom", { roomId, agentId });
  }

  removeOwnAgentFromSharedRoom(roomId: string, agentId: string): Promise<SharedRoomSummary | null> {
    return this.mutate<SharedRoomSummary | null>("removeOwnAgentFromSharedRoom", { roomId, agentId });
  }

  async setSharedRoomTyping(roomId: string, participantId: string, isTyping: boolean): Promise<void> {
    const typing = await invokeNativeDesktop<SharedRoomTypingState>("setSharedRoomTyping", { roomId, participantId, isTyping });
    this.emit({ type: "typing.changed", ...typing });
  }

  leaveSharedRoom(roomId: string, agentId: string): Promise<SharedRoomSummary | null> {
    return this.mutate<SharedRoomSummary | null>("leaveSharedRoom", { roomId, agentId });
  }

  subscribe(listener: (event: CollaborationEvent) => void): () => void {
    this.listeners.add(listener);
    this.ensureLiveRefresh();
    return () => {
      this.listeners.delete(listener);
      if (!this.listeners.size) this.stopPolling();
    };
  }

  close(): void {
    this.closed = true;
    this.nativeUnsubscribe?.();
    this.nativeUnsubscribe = null;
    this.stopPolling();
    this.listeners.clear();
  }

  private async mutate<T>(method: string, params: Record<string, unknown>, refresh = true): Promise<T> {
    const result = await invokeNativeDesktop<T>(method, params);
    if (refresh) await this.refresh().catch(() => undefined);
    return result;
  }

  private applyState(next: SharingState): void {
    const previousTyping = new Map((this.state.typing ?? []).map((entry) => [`${entry.roomId}:${entry.participantId}`, entry]));
    const nextTyping = new Map((next.typing ?? []).map((entry) => [`${entry.roomId}:${entry.participantId}`, entry]));
    this.state = next;
    this.emit({ type: "state.changed", state: next });
    for (const [key, entry] of nextTyping) {
      const previous = previousTyping.get(key);
      if (!previous || previous.isTyping !== entry.isTyping || previous.updatedAtMs !== entry.updatedAtMs) {
        this.emit({ type: "typing.changed", ...entry });
      }
    }
    for (const [key, entry] of previousTyping) {
      if (!nextTyping.has(key) && entry.isTyping) {
        this.emit({ type: "typing.changed", ...entry, isTyping: false, updatedAtMs: Date.now() });
      }
    }
  }

  private ensureLiveRefresh(): void {
    if (!this.nativeUnsubscribe) {
      this.nativeUnsubscribe = subscribeNativeDesktopEvents({
        "shared-room-changed": () => { void this.refresh().catch(() => undefined); },
      });
    }
    void this.refresh().catch(() => undefined);
    this.schedulePoll();
  }

  private schedulePoll(): void {
    if (this.pollTimer || !this.listeners.size || this.closed) return;
    this.pollTimer = setTimeout(() => {
      this.pollTimer = null;
      void this.refresh().catch(() => undefined).finally(() => this.schedulePoll());
    }, 7_500);
  }

  private stopPolling(): void {
    if (this.pollTimer) clearTimeout(this.pollTimer);
    this.pollTimer = null;
  }

  private emit(event: CollaborationEvent): void {
    for (const listener of this.listeners) listener(event);
  }
}

export class LocalCollaborationProvider {
  private state = loadState();
  private readonly listeners = new Set<(event: CollaborationEvent) => void>();
  private channel: BroadcastChannel | null = null;

  getSharingState(agentId?: string): SharingState {
    const rooms = agentId
      ? this.state.rooms.filter((room) => room.memberAgentIds.includes(agentId))
      : this.state.rooms;
    const roomIds = new Set(rooms.map((room) => room.id));
    return {
      scope: "local-device",
      rooms: rooms.map((room) => ({ ...room, memberAgentIds: [...room.memberAgentIds] })),
      joinRequests: this.state.joinRequests.filter((request) => roomIds.has(request.roomId)),
    };
  }

  createRoomFromAgent(agentId: string, name?: string): SharedRoomSummary {
    return this.createSharedRoom(name?.trim() || "Fabushi shared room", [agentId], agentId);
  }

  createSharedRoom(name: string, memberAgentIds: readonly string[], ownerAgentId: string | null = null): SharedRoomSummary {
    const cleanName = name.replace(/\s+/gu, " ").trim().slice(0, 96);
    if (!cleanName) throw new Error("Room name is required");
    const members = uniqueIds(memberAgentIds);
    if (ownerAgentId && !members.includes(ownerAgentId)) members.unshift(ownerAgentId);
    const now = Date.now();
    const room: SharedRoomSummary = {
      id: randomId("room"),
      name: cleanName,
      ownerAgentId,
      memberAgentIds: members,
      scope: "local-device",
      createdAtMs: now,
      updatedAtMs: now,
    };
    this.state = { ...this.state, rooms: [...this.state.rooms, room].slice(-MAX_ROOMS) };
    this.commit();
    return room;
  }

  createRoomInvite(roomId: string): SharedRoomInvite {
    this.requireRoom(roomId);
    const invite: SharedRoomInvite = {
      token: `fabushi-local-${randomId("invite")}`,
      roomId,
      expiresAtMs: Date.now() + INVITE_TTL_MS,
    };
    this.state = {
      ...this.state,
      invites: [...this.state.invites.filter((item) => item.expiresAtMs > Date.now()), invite].slice(-MAX_INVITES),
    };
    this.commit();
    return invite;
  }

  joinSharedRoom(token: string, agentId: string, displayName?: string): SharedRoomJoinRequest {
    const invite = this.state.invites.find((item) => item.token === token && item.expiresAtMs > Date.now());
    if (!invite) throw new Error("Invite is invalid or expired");
    this.requireRoom(invite.roomId);
    const cleanAgentId = agentId.trim();
    if (!cleanAgentId) throw new Error("Agent ID is required");
    const request: SharedRoomJoinRequest = {
      id: randomId("join"),
      roomId: invite.roomId,
      agentId: cleanAgentId,
      displayName: displayName?.replace(/\s+/gu, " ").trim().slice(0, 96) || cleanAgentId,
      status: "pending",
      createdAtMs: Date.now(),
    };
    this.state = {
      ...this.state,
      joinRequests: [...this.state.joinRequests, request].slice(-MAX_JOIN_REQUESTS),
    };
    this.commit();
    return request;
  }

  respondToRoomJoinRequest(requestId: string, accept: boolean): SharedRoomJoinRequest {
    const current = this.state.joinRequests.find((request) => request.id === requestId);
    if (!current) throw new Error(`Unknown join request: ${requestId}`);
    if (current.status !== "pending") return current;
    const resolved: SharedRoomJoinRequest = { ...current, status: accept ? "accepted" : "rejected" };
    let rooms = this.state.rooms;
    if (accept) {
      rooms = rooms.map((room) => room.id === current.roomId
        ? {
            ...room,
            memberAgentIds: uniqueIds([...room.memberAgentIds, current.agentId]),
            updatedAtMs: Date.now(),
          }
        : room);
    }
    this.state = {
      ...this.state,
      rooms,
      joinRequests: this.state.joinRequests.map((request) => request.id === requestId ? resolved : request),
    };
    this.commit();
    return resolved;
  }

  addOwnAgentToSharedRoom(roomId: string, agentId: string): SharedRoomSummary {
    return this.updateMembers(roomId, (members) => uniqueIds([...members, agentId]));
  }

  removeOwnAgentFromSharedRoom(roomId: string, agentId: string): SharedRoomSummary {
    return this.updateMembers(roomId, (members) => members.filter((id) => id !== agentId));
  }

  leaveSharedRoom(roomId: string, agentId: string): SharedRoomSummary | null {
    const room = this.requireRoom(roomId);
    const members = room.memberAgentIds.filter((id) => id !== agentId);
    if (!members.length) {
      this.state = {
        ...this.state,
        rooms: this.state.rooms.filter((candidate) => candidate.id !== roomId),
        invites: this.state.invites.filter((invite) => invite.roomId !== roomId),
        joinRequests: this.state.joinRequests.filter((request) => request.roomId !== roomId),
      };
      this.commit();
      return null;
    }
    return this.updateMembers(roomId, () => members);
  }

  setSharedRoomTyping(roomId: string, participantId: string, isTyping: boolean): void {
    this.requireRoom(roomId);
    const event: CollaborationEvent = {
      type: "typing.changed",
      roomId,
      participantId,
      isTyping,
      updatedAtMs: Date.now(),
    };
    this.emitLocal(event);
    this.ensureChannel()?.postMessage(event);
  }

  subscribe(listener: (event: CollaborationEvent) => void): () => void {
    this.ensureChannel();
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  close(): void {
    this.channel?.close();
    this.channel = null;
    this.listeners.clear();
  }

  private updateMembers(roomId: string, update: (members: string[]) => string[]): SharedRoomSummary {
    const current = this.requireRoom(roomId);
    const next: SharedRoomSummary = {
      ...current,
      memberAgentIds: uniqueIds(update([...current.memberAgentIds])),
      updatedAtMs: Date.now(),
    };
    this.state = {
      ...this.state,
      rooms: this.state.rooms.map((room) => room.id === roomId ? next : room),
    };
    this.commit();
    return next;
  }

  private requireRoom(roomId: string): SharedRoomSummary {
    const room = this.state.rooms.find((candidate) => candidate.id === roomId);
    if (!room) throw new Error(`Unknown shared room: ${roomId}`);
    return room;
  }

  private commit(): void {
    saveState(this.state);
    const event: CollaborationEvent = { type: "state.changed", state: this.getSharingState() };
    this.emitLocal(event);
    this.ensureChannel()?.postMessage(event);
  }

  private ensureChannel(): BroadcastChannel | null {
    if (this.channel || typeof window === "undefined" || typeof BroadcastChannel !== "function") {
      return this.channel;
    }
    this.channel = new BroadcastChannel(CHANNEL_NAME);
    this.channel.onmessage = (message: MessageEvent<CollaborationEvent>) => {
      if (message.data?.type === "state.changed") {
        this.state = loadState();
        this.emitLocal({ type: "state.changed", state: this.getSharingState() });
        return;
      }
      if (message.data?.type === "typing.changed") this.emitLocal(message.data);
    };
    return this.channel;
  }

  private emitLocal(event: CollaborationEvent): void {
    for (const listener of this.listeners) listener(event);
  }
}
