import assert from "node:assert/strict";
import test from "node:test";

import {
  SIDEBAR_SEARCH_TRIGGER,
  createSidebarSearchTrigger,
  type SidebarSearchKeyEvent,
} from "./conversation/workspace/sidebar-search-trigger.ts";
import {
  GROUP_INFO_PANE_HEADER,
  isCurrentGroupInfoPaneRoute,
  isLocalGroupAgent,
  projectGroupInfoPaneRoute,
} from "./agent-info/group-members/route.ts";

function keyEvent(key: string, modifiers: Partial<SidebarSearchKeyEvent> = {}) {
  const calls: string[] = [];
  return {
    calls,
    event: {
      key,
      defaultPrevented: false,
      metaKey: false,
      ctrlKey: false,
      altKey: false,
      preventDefault: () => calls.push("preventDefault"),
      stopPropagation: () => calls.push("stopPropagation"),
      ...modifiers,
    } satisfies SidebarSearchKeyEvent,
  };
}

test("sidebar search opens for click and plain printable typing only", () => {
  const calls: string[] = [];
  const trigger = createSidebarSearchTrigger({
    openSearch: () => calls.push("open"),
  });

  trigger.onClick();
  assert.deepEqual(calls, ["open"]);
  assert.equal(SIDEBAR_SEARCH_TRIGGER.ariaLabel, "Search");

  const printable = keyEvent("x");
  trigger.onKeyDown(printable.event);
  assert.deepEqual(printable.calls, ["preventDefault"]);
  assert.deepEqual(calls, ["open", "open"]);

  for (const input of [
    keyEvent(" "),
    keyEvent("Enter"),
    keyEvent("x", { metaKey: true }),
    keyEvent("x", { ctrlKey: true }),
    keyEvent("x", { altKey: true }),
    keyEvent("x", { defaultPrevented: true }),
  ]) {
    trigger.onKeyDown(input.event);
    assert.deepEqual(input.calls, []);
  }
  assert.deepEqual(calls, ["open", "open"]);

  for (const key of ["Delete", "Backspace"]) {
    const deletion = keyEvent(key);
    trigger.onKeyDown(deletion.event);
    assert.deepEqual(deletion.calls, ["stopPropagation"]);
  }
});

test("group-info route mounts only for current account-bound local groups", () => {
  const local = {
    id: "group-1",
    isGroup: true,
    raw: {},
  };
  const shared = {
    id: "group-shared",
    isGroup: true,
    raw: { isSharedRoom: true },
  };
  const direct = {
    id: "direct",
    isGroup: false,
    raw: {},
  };

  assert.equal(isLocalGroupAgent(local), true);
  assert.equal(isLocalGroupAgent(shared), false);
  assert.equal(isLocalGroupAgent(direct), false);

  const opened: string[] = [];
  const route = projectGroupInfoPaneRoute({
    agent: local,
    accountKey: "account-1",
    accountGeneration: 7,
    onOpenAgentChat: (agentId) => opened.push(agentId),
  });
  assert.ok(route);
  assert.equal(route.route, "overview");
  assert.equal(route.header, GROUP_INFO_PANE_HEADER);
  route.onOpenAgentChat("member-1");
  assert.deepEqual(opened, ["member-1"]);

  assert.equal(projectGroupInfoPaneRoute({
    agent: shared,
    accountKey: "account-1",
    accountGeneration: 7,
    onOpenAgentChat: () => {},
  }), null);
  assert.equal(projectGroupInfoPaneRoute({
    agent: local,
    accountKey: "",
    accountGeneration: 7,
    onOpenAgentChat: () => {},
  }), null);
  assert.equal(projectGroupInfoPaneRoute({
    agent: local,
    accountKey: "account-1",
    accountGeneration: -1,
    onOpenAgentChat: () => {},
  }), null);

  assert.equal(isCurrentGroupInfoPaneRoute(route, {
    agent: local,
    accountKey: "account-1",
    accountGeneration: 7,
  }), true);
  assert.equal(isCurrentGroupInfoPaneRoute(route, {
    agent: local,
    accountKey: "account-1",
    accountGeneration: 8,
  }), false);
  assert.equal(isCurrentGroupInfoPaneRoute(route, {
    agent: shared,
    accountKey: "account-1",
    accountGeneration: 7,
  }), false);
});
