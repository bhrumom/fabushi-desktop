import {
  projectInviteResult,
  projectSharingState,
  type SharedInviteResult,
  type SharedSharingState,
} from "./model.ts";

export interface SharingCoordinatorClient {
  call(method: string, args?: unknown): Promise<unknown>;
}

function sharingState(value: unknown): SharedSharingState {
  const projected = projectSharingState(value);
  if (projected == null) {
    throw new TypeError("Sharing returned a malformed state");
  }
  return projected;
}

function inviteResult(value: unknown): SharedInviteResult {
  const projected = projectInviteResult(value);
  if (projected == null) {
    throw new TypeError("Sharing returned a malformed invite result");
  }
  return projected;
}

export async function typedGetSharingState(
  client: SharingCoordinatorClient,
): Promise<SharedSharingState> {
  return sharingState(await client.call("getSharingState"));
}

export async function typedCreateRoomInvite(
  client: SharingCoordinatorClient,
  args: { readonly roomId: string },
): Promise<SharedInviteResult> {
  if (args.roomId.length === 0) {
    throw new TypeError("createRoomInvite requires a room id");
  }
  return inviteResult(
    await client.call("createRoomInvite", { roomId: args.roomId }),
  );
}

export async function typedRespondToRoomJoinRequest(
  client: SharingCoordinatorClient,
  args: { readonly requestId: string; readonly isApproved: boolean },
): Promise<SharedSharingState> {
  if (args.requestId.length === 0) {
    throw new TypeError("respondToRoomJoinRequest requires a request id");
  }
  return sharingState(
    await client.call("respondToRoomJoinRequest", { ...args }),
  );
}

export async function typedAddOwnAgentToSharedRoom(
  client: SharingCoordinatorClient,
  args: {
    readonly roomId: string;
    readonly agentId: string;
    readonly agentName: string;
  },
): Promise<SharedSharingState> {
  if (
    args.roomId.length === 0
    || args.agentId.length === 0
    || args.agentName.length === 0
  ) {
    throw new TypeError(
      "addOwnAgentToSharedRoom requires room and agent identity",
    );
  }
  return sharingState(await client.call("addOwnAgentToSharedRoom", { ...args }));
}

export async function typedRemoveOwnAgentFromSharedRoom(
  client: SharingCoordinatorClient,
  args: { readonly roomId: string; readonly agentId: string },
): Promise<SharedSharingState> {
  if (args.roomId.length === 0 || args.agentId.length === 0) {
    throw new TypeError(
      "removeOwnAgentFromSharedRoom requires room and agent identity",
    );
  }
  return sharingState(
    await client.call("removeOwnAgentFromSharedRoom", { ...args }),
  );
}

export async function typedLeaveSharedRoom(
  client: SharingCoordinatorClient,
  args: { readonly roomId: string; readonly targetAuthId?: string },
): Promise<SharedSharingState> {
  if (args.roomId.length === 0) {
    throw new TypeError("leaveSharedRoom requires a room id");
  }
  return sharingState(await client.call("leaveSharedRoom", {
    roomId: args.roomId,
    ...(args.targetAuthId == null
      ? {}
      : { targetAuthId: args.targetAuthId }),
  }));
}
