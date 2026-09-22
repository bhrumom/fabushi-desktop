export interface SharedRoomMember {
  readonly kind: "human" | "agent";
  readonly authId: string;
  readonly agentId?: string;
  readonly displayName: string;
  readonly avatarUrl?: string;
}

export interface SharedRoom {
  readonly roomId: string;
  readonly name: string;
  readonly hostAuthId: string;
  readonly members: readonly SharedRoomMember[];
  readonly avatarDataUrl?: string;
}

export interface SharedJoinRequest {
  readonly requestId: string;
  readonly roomId: string;
  readonly requesterAuthId: string;
  readonly requesterName: string;
  readonly requesterAvatarUrl?: string;
}

export interface SharedTypingUser {
  readonly roomId: string;
  readonly authId: string;
  readonly name: string;
  readonly avatarUrl?: string;
  readonly expiresAtMs: number;
}

export interface SharedSharingState {
  readonly isEnabled: boolean;
  readonly selfAuthId: string | null;
  readonly pendingJoinRequests: readonly SharedJoinRequest[];
  readonly rooms: readonly SharedRoom[];
  readonly typingUsers: readonly SharedTypingUser[];
}

export interface SharedRoomAgent {
  readonly id: string;
  readonly name: string;
  readonly isGroup: boolean;
  readonly remoteRoom?: { readonly roomId: string } | null;
  readonly isSharedRoom?: boolean;
}

export type SharedInviteResult =
  | {
      readonly status: "ok";
      readonly shareUrl: string;
      readonly expiresAtMs: number;
      readonly roomId: string;
    }
  | { readonly status: "error"; readonly message: string };

export interface SharedRoomContext {
  readonly roomId: string;
  readonly agentId: string;
  readonly accountGeneration: number;
  readonly agents: readonly SharedRoomAgent[];
}

export type SharedRoomAction = "refresh" | "invite" | "respond" | "add" | "remove" | "leave";

export interface SharedRoomSnapshot {
  readonly context: SharedRoomContext | null;
  readonly state: SharedSharingState | null;
  readonly room: SharedRoom | null;
  readonly isHost: boolean;
  readonly selfAgentIds: readonly string[];
  readonly candidates: readonly SharedRoomAgent[];
  readonly requests: readonly SharedJoinRequest[];
  readonly pending: ReadonlySet<string>;
  readonly pendingAction: SharedRoomAction | null;
  readonly invite: SharedInviteResult | null;
  readonly isLoading: boolean;
  readonly transport: "connected" | "down" | "unknown";
  readonly failure: unknown | null;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function nonEmpty(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function member(value: unknown): SharedRoomMember | null {
  const row = asRecord(value);
  if (
    row == null
    || (row.kind !== "human" && row.kind !== "agent")
    || !nonEmpty(row.authId)
    || (row.agentId !== undefined && !nonEmpty(row.agentId))
  ) {
    return null;
  }
  const displayName =
    typeof row.displayName === "string" ? row.displayName : "";
  if (row.kind === "human" && displayName.length === 0) return null;
  if (
    row.avatarDataUrl !== undefined
    && typeof row.avatarDataUrl !== "string"
  ) {
    return null;
  }
  return {
    kind: row.kind,
    authId: row.authId,
    ...(row.agentId === undefined ? {} : { agentId: row.agentId as string }),
    displayName,
    ...(typeof row.avatarDataUrl === "string" && row.avatarDataUrl.length > 0
      ? { avatarUrl: row.avatarDataUrl }
      : {}),
  };
}

function room(value: unknown): SharedRoom | null {
  const row = asRecord(value);
  if (
    row == null
    || !nonEmpty(row.roomId)
    || !nonEmpty(row.name)
    || !nonEmpty(row.hostAuthId)
    || !Array.isArray(row.members)
  ) {
    return null;
  }
  const members = row.members.map(member);
  if (members.some((item) => item == null)) return null;
  return {
    roomId: row.roomId,
    name: row.name,
    hostAuthId: row.hostAuthId,
    members: members as SharedRoomMember[],
    ...(typeof row.avatarDataUrl === "string" && row.avatarDataUrl.length > 0
      ? { avatarDataUrl: row.avatarDataUrl }
      : {}),
  };
}

function joinRequest(value: unknown): SharedJoinRequest | null {
  const row = asRecord(value);
  if (
    row == null
    || !nonEmpty(row.requestId)
    || !nonEmpty(row.roomId)
    || !nonEmpty(row.requesterAuthId)
    || !nonEmpty(row.requesterName)
    || (
      row.requesterAvatarUrl !== undefined
      && typeof row.requesterAvatarUrl !== "string"
    )
  ) {
    return null;
  }
  return {
    requestId: row.requestId,
    roomId: row.roomId,
    requesterAuthId: row.requesterAuthId,
    requesterName: row.requesterName,
    ...(nonEmpty(row.requesterAvatarUrl)
      ? { requesterAvatarUrl: row.requesterAvatarUrl }
      : {}),
  };
}

function typingUser(value: unknown): SharedTypingUser | null {
  const row = asRecord(value);
  if (
    row == null
    || !nonEmpty(row.roomId)
    || !nonEmpty(row.authId)
    || !nonEmpty(row.name)
    || typeof row.expiresAtMs !== "number"
    || !Number.isFinite(row.expiresAtMs)
  ) {
    return null;
  }
  return {
    roomId: row.roomId,
    authId: row.authId,
    name: row.name,
    expiresAtMs: row.expiresAtMs,
    ...(nonEmpty(row.avatarUrl) ? { avatarUrl: row.avatarUrl } : {}),
  };
}

export function projectSharingState(value: unknown): SharedSharingState | null {
  const row = asRecord(value);
  if (
    row == null
    || typeof row.isEnabled !== "boolean"
    || !(row.selfAuthId === null || nonEmpty(row.selfAuthId))
    || !Array.isArray(row.pendingJoinRequests)
    || !Array.isArray(row.rooms)
    || !Array.isArray(row.typingUsers)
  ) {
    return null;
  }

  const requests = row.pendingJoinRequests.map(joinRequest);
  const rooms = row.rooms.map(room);
  const typing = row.typingUsers.map(typingUser);
  if (
    requests.some((item) => item == null)
    || rooms.some((item) => item == null)
    || typing.some((item) => item == null)
  ) {
    return null;
  }

  return {
    isEnabled: row.isEnabled,
    selfAuthId: row.selfAuthId,
    pendingJoinRequests: requests as SharedJoinRequest[],
    rooms: rooms as SharedRoom[],
    typingUsers: typing as SharedTypingUser[],
  };
}

export function projectInviteResult(value: unknown): SharedInviteResult | null {
  const row = asRecord(value);
  if (row == null) return null;
  if (row.status === "error") {
    return nonEmpty(row.message)
      ? { status: "error", message: row.message }
      : null;
  }
  if (
    row.status !== "ok"
    || !nonEmpty(row.shareUrl)
    || !nonEmpty(row.roomId)
    || typeof row.expiresAtMs !== "number"
    || !Number.isFinite(row.expiresAtMs)
  ) {
    return null;
  }
  return {
    status: "ok",
    shareUrl: row.shareUrl,
    expiresAtMs: row.expiresAtMs,
    roomId: row.roomId,
  };
}

export function projectSharedRoomAgent(value: unknown): SharedRoomAgent | null {
  const row = asRecord(value);
  if (
    row == null
    || !nonEmpty(row.id)
    || !nonEmpty(row.name)
    || typeof row.isGroup !== "boolean"
    || (
      row.isSharedRoom !== undefined
      && typeof row.isSharedRoom !== "boolean"
    )
  ) {
    return null;
  }

  let remoteRoom: { roomId: string } | null = null;
  if (row.remoteRoom !== undefined && row.remoteRoom !== null) {
    const remote = asRecord(row.remoteRoom);
    if (remote == null || !nonEmpty(remote.roomId)) return null;
    remoteRoom = { roomId: remote.roomId };
  }
  return {
    id: row.id,
    name: row.name,
    isGroup: row.isGroup,
    remoteRoom,
    ...(row.isSharedRoom === undefined
      ? {}
      : { isSharedRoom: row.isSharedRoom as boolean }),
  };
}
