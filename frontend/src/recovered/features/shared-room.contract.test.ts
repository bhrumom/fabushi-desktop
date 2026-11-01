import assert from "node:assert/strict";
import test from "node:test";

import {
  projectInviteResult,
  projectSharedRoomAgent,
  projectSharingState,
} from "./agent-info/shared-room/model.ts";
import {
  typedAddOwnAgentToSharedRoom,
  typedCreateRoomInvite,
  typedGetSharingState,
  typedLeaveSharedRoom,
  typedRemoveOwnAgentFromSharedRoom,
  typedRespondToRoomJoinRequest,
} from "./agent-info/shared-room/bridge.ts";

const validState = {
  isEnabled: true,
  selfAuthId: "me",
  pendingJoinRequests: [{
    requestId: "req-1",
    roomId: "room-1",
    requesterAuthId: "other",
    requesterName: "Other",
  }],
  rooms: [{
    roomId: "room-1",
    name: "Room",
    hostAuthId: "me",
    members: [
      { kind: "human", authId: "me", displayName: "Me" },
      { kind: "agent", authId: "me", agentId: "a", displayName: "" },
    ],
  }],
  typingUsers: [{
    roomId: "room-1",
    authId: "other",
    name: "Other",
    expiresAtMs: 1000,
  }],
};

test("shared-room projection rejects malformed nested rows as a whole", () => {
  const state = projectSharingState(validState);
  assert.ok(state);
  assert.equal(state.rooms[0]?.members.length, 2);
  assert.equal(projectSharingState({
    ...validState,
    rooms: [{ ...validState.rooms[0], members: [{ kind: "human" }] }],
  }), null);

  assert.deepEqual(projectInviteResult({
    status: "ok",
    shareUrl: "https://example.test/invite",
    expiresAtMs: 42,
    roomId: "room-1",
  }), {
    status: "ok",
    shareUrl: "https://example.test/invite",
    expiresAtMs: 42,
    roomId: "room-1",
  });
  assert.deepEqual(projectInviteResult({
    status: "error",
    message: "disabled",
  }), { status: "error", message: "disabled" });
  assert.equal(projectInviteResult({ status: "error", message: "" }), null);

  assert.deepEqual(projectSharedRoomAgent({
    id: "agent",
    name: "Agent",
    isGroup: true,
    isSharedRoom: true,
    remoteRoom: { roomId: "room-1" },
  })?.remoteRoom, { roomId: "room-1" });
});

test("shared-room bridge uses closed RPC method set and validates identities/replies", async () => {
  const calls: Array<[string, unknown]> = [];
  const client = {
    async call(method: string, args?: unknown) {
      calls.push([method, args]);
      if (method === "createRoomInvite") {
        return {
          status: "ok",
          shareUrl: "https://example.test/i",
          expiresAtMs: 99,
          roomId: "room-1",
        };
      }
      return validState;
    },
  };

  assert.equal((await typedGetSharingState(client)).isEnabled, true);
  assert.equal((await typedCreateRoomInvite(client, { roomId: "room-1" })).status, "ok");
  await typedRespondToRoomJoinRequest(client, {
    requestId: "req-1",
    isApproved: true,
  });
  await typedAddOwnAgentToSharedRoom(client, {
    roomId: "room-1",
    agentId: "a",
    agentName: "Agent",
  });
  await typedRemoveOwnAgentFromSharedRoom(client, {
    roomId: "room-1",
    agentId: "a",
  });
  await typedLeaveSharedRoom(client, {
    roomId: "room-1",
    targetAuthId: "other",
  });
  assert.deepEqual(calls.map(([method]) => method), [
    "getSharingState",
    "createRoomInvite",
    "respondToRoomJoinRequest",
    "addOwnAgentToSharedRoom",
    "removeOwnAgentFromSharedRoom",
    "leaveSharedRoom",
  ]);

  await assert.rejects(
    () => typedCreateRoomInvite(client, { roomId: "" }),
    /room id/,
  );

  await assert.rejects(
    () => typedGetSharingState({
      call: async () => ({ malformed: true }),
    }),
    /malformed state/,
  );
});
