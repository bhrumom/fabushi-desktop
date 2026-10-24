import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  deepLinkRoute,
  deepLinkSourceLabel,
} from "./deep-links/overlay/model.ts";
import {
  hiddenChatNameId,
  hiddenChatsEmptyLabel,
  openHiddenChat,
} from "./hidden-chats/overlay/model.ts";
import {
  AGENT_NETWORK_TRIGGER,
  createAgentNetworkTrigger,
} from "./org-chart/workspace/network-trigger.ts";
import {
  SIDEBAR_PROFILE_ACTION,
  createSidebarProfileAction,
} from "./conversation/workspace/sidebar-profile-action.ts";

test("deep link model keeps protocol route identity stable", () => {
  assert.equal(
    deepLinkRoute({
      version: 1,
      source: "protocol",
      route: "info",
      topic: "deep-links",
    }),
    "sand://app/v1/info?topic=deep-links",
  );
  assert.equal(deepLinkSourceLabel("protocol"), "Custom protocol (sand://)");
  assert.equal(deepLinkSourceLabel("https"), "HTTPS link");
});

test("hidden chat model preserves ids, empty label and close-before-open order", () => {
  assert.equal(hiddenChatNameId("agent-7"), "sand-hidden-chat-agent-7-name");
  assert.equal(hiddenChatsEmptyLabel([]), "No hidden bots");
  assert.equal(
    hiddenChatsEmptyLabel([{ id: "agent-7", name: "Seven" }]),
    null,
  );

  const calls: string[] = [];
  openHiddenChat(
    () => calls.push("close"),
    (agentId) => calls.push(`open:${agentId}`),
    "agent-7",
  );
  assert.deepEqual(calls, ["close", "open:agent-7"]);
});

test("agent network trigger is fail-closed behind availability gate", () => {
  const blockedCalls: string[] = [];
  const blocked = createAgentNetworkTrigger({
    isAvailable: false,
    closeChooser: () => blockedCalls.push("close"),
    openOrgChart: () => blockedCalls.push("open"),
  });
  assert.equal(blocked(), false);
  assert.deepEqual(blockedCalls, []);

  const calls: string[] = [];
  const available = createAgentNetworkTrigger({
    isAvailable: true,
    closeChooser: () => calls.push("close"),
    openOrgChart: () => calls.push("open"),
  });
  assert.equal(available(), true);
  assert.deepEqual(calls, ["close", "open"]);
  assert.equal(AGENT_NETWORK_TRIGGER.ariaLabel, "Agent network");
  assert.equal(AGENT_NETWORK_TRIGGER.icon, "cube-nodes");
});

test("agent network graph keeps responsive measurement and live wheel geometry in sync", () => {
  const graphSource = readFileSync(
    new URL("./org-chart/workspace/graph.tsx", import.meta.url),
    "utf8",
  );

  assert.match(graphSource, /ref=\{containerRef\}/);
  assert.match(
    graphSource,
    /style=\{\{ width: width \?\? "100%", height: height \?\? "100%" \}\}/,
  );
  assert.match(
    graphSource,
    /observer\.observe\(container\);[\s\S]*?\}, \[height, width\]\);/,
  );
  assert.match(
    graphSource,
    /scene\.addEventListener\("wheel", onWheel, \{ passive: false \}\);[\s\S]*?\}, \[layoutHeight, layoutWidth\]\);/,
  );
  assert.match(
    graphSource,
    /zoomViewport\(viewportRef\.current,[\s\S]*?\{ width: layoutWidth, height: layoutHeight \}\)/,
  );
});

test("sidebar profile action forwards exactly the selected agent identity", () => {
  const opened: string[] = [];
  const action = createSidebarProfileAction({
    openProfile: (agentId) => opened.push(agentId),
  });
  action.onSelect("agent-a");
  action.onSelect("agent-b");
  assert.deepEqual(opened, ["agent-a", "agent-b"]);
  assert.equal(SIDEBAR_PROFILE_ACTION.label, "Edit Profile");
});
