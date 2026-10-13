import fs from "node:fs";
import { describe, expect, it } from "vitest";

import {
  COORDINATOR_METHOD_TABLE,
  parseCoordinatorSavedSublistDraft,
} from "../../../source/shared/rpc/coordinator";

const root = new URL("../../", import.meta.url);
const read = (relative: string) => fs.readFileSync(new URL(relative, root), "utf8");

describe("canonical SavedSublist cloud draft contract", () => {
  it("parses only exact typed SavedSublist draft projections", () => {
    const valid = {
      destination: {
        conversationId: "conversation:human:1:2",
        child: { kind: "savedSublist", participantId: "2" },
      },
      text: "draft",
      replyToMessageId: "human-server-message:7",
      revision: 3,
      updatedAt: "2026-10-13T01:00:00.000Z",
    };
    expect(parseCoordinatorSavedSublistDraft(valid)).toEqual(valid);
    expect(parseCoordinatorSavedSublistDraft({
      ...valid,
      destination: { ...valid.destination, child: { kind: "topic", participantId: "2" } },
    })).toBeNull();
    expect(parseCoordinatorSavedSublistDraft({ ...valid, revision: -1 })).toBeNull();
    expect(parseCoordinatorSavedSublistDraft({ ...valid, revision: Number.MAX_SAFE_INTEGER + 1 })).toBeNull();
    expect(parseCoordinatorSavedSublistDraft({ ...valid, replyToMessageId: 7 })).toBeNull();
  });

  it("uses the existing Coordinator and Host Session owner instead of renderer HTTP", () => {
    expect(COORDINATOR_METHOD_TABLE.getHumanSavedSublistDraft).toEqual({ args: "object", reply: "record" });
    expect(COORDINATOR_METHOD_TABLE.setHumanSavedSublistDraft).toEqual({ args: "object", reply: "record" });

    const client = read("src/production/coordinator-client.ts");
    const gatewayApi = read("../source/host/src/host_gateway_api.rs");
    const gateway = read("../source/host/src/extensions/session/gateway.rs");
    const production = read("../source/host/src/extensions/session/production.rs");
    const native = read("../source/host/src/extensions/session/native_messaging.rs");
    const service = read("../native/mahayana-messaging/src/service.rs");

    for (const method of ["getHumanSavedSublistDraft", "setHumanSavedSublistDraft"]) {
      expect(client).toContain(method);
      expect(gatewayApi).toContain(`"${method}"`);
      expect(gateway).toContain(`"${method}" =>`);
    }
    expect(client).toContain("parseCoordinatorSavedSublistDraft");
    expect(production).toContain("reconcile_remote_saved_sublist_draft");
    expect(native).toContain('"/api/social/conversation-child-draft"');
    expect(service).toContain("reconcile_conversation_child_draft");
    expect(client).not.toContain("/api/social/conversation-child-draft");
  });
});
