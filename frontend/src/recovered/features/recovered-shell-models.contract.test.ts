import assert from "node:assert/strict";
import test from "node:test";

import {
  mergeToolResultCard,
  projectToolResultCard,
} from "./conversation/tool-results/model.ts";
import {
  createGlobalKeyboardShortcutController,
  createRootShellShortcutActions,
  resolveGlobalShortcutAction,
  type KeyboardShortcutEvent,
} from "./window-chrome/global-keyboard-shortcuts.ts";
import {
  computerCursorPresentation,
  computerStageCopy,
  firstSelectedMonitor,
  handoffStatusLabel,
  parseVncLiveness,
  parseVncSession,
  projectComputerCursor,
  projectComputerMonitors,
  projectComputerStatus,
  retainWarmVncSources,
  stepSelectedMonitor,
  vncDimensions,
  vncIdentity,
  vncViewerUrl,
} from "./computer/shell/model.ts";

test("tool-result projection normalizes shell deltas, terminal results, and file errors", () => {
  const running = projectToolResultCard({
    kind: "shell",
    tool_call_id: "tool-1",
    args: { command: "printf hi", working_directory: "/tmp" },
    delta: { stdout_delta: "a\r\nb" },
  });
  assert.ok(running);
  assert.equal(running.kind, "shell");
  assert.equal(running.status, "running");
  assert.equal(running.output, "a\nb");
  assert.equal(running.command, "printf hi");
  assert.equal(running.workingDirectory, "/tmp");
  assert.equal(running.isStreaming, true);

  const merged = mergeToolResultCard(running, {
    delta: { stdoutDelta: "c" },
  });
  assert.equal(merged.output, "a\nbc");
  assert.equal(merged.toolCallId, "tool-1");

  const success = projectToolResultCard({
    tool: "bash",
    result: {
      success: { stdout: "done" },
      is_background: true,
    },
  });
  assert.ok(success);
  assert.equal(success.status, "background");
  assert.equal(success.isBackground, true);
  assert.equal(success.output, "done");

  const denied = projectToolResultCard({
    tool: "edit",
    args: { path: "/repo/a.ts", edits: [] },
    result: {
      write_permission_denied: { message: "no" },
    },
  });
  assert.ok(denied);
  assert.equal(denied.kind, "file-edit");
  assert.equal(denied.status, "denied");
  assert.equal(denied.path, "/repo/a.ts");

  assert.equal(projectToolResultCard({ hello: "world" }), null);
});

function keyboardEvent(
  key: string,
  overrides: Partial<KeyboardShortcutEvent> = {},
): KeyboardShortcutEvent & { prevented: boolean } {
  const state = {
    prevented: false,
    key,
    defaultPrevented: false,
    altKey: false,
    shiftKey: false,
    metaKey: false,
    ctrlKey: false,
    preventDefault() { state.prevented = true; },
    ...overrides,
  };
  return state;
}

test("global shortcut registry preserves order, editable policy, alternate prompt hotkey, and agent indexes", () => {
  const calls: string[] = [];
  const actions = createRootShellShortcutActions({
    toggleCommandPalette: () => calls.push("palette"),
    openSearch: () => calls.push("search"),
    newAgent: () => calls.push("new"),
    openSettings: () => calls.push("settings"),
    openTools: () => calls.push("tools"),
    focusPrompt: () => calls.push("prompt"),
    findInChat: () => calls.push("find"),
    previousAgent: () => calls.push("prev"),
    nextAgent: () => calls.push("next"),
    navigateBack: () => calls.push("back"),
    navigateForward: () => calls.push("forward"),
    focusAgent: (index) => calls.push(`agent:${index}`),
    toggleSidebar: () => calls.push("sidebar"),
  });

  assert.deepEqual(actions.slice(0, 7).map((item) => item.id), [
    "sand.newAgent",
    "sand.commandPalette",
    "sand.openSettings",
    "sand.openTools",
    "sand.focusInput",
    "sand.findInChat",
    "sand.focusSearch",
  ]);

  const modN = keyboardEvent("n", { metaKey: true });
  assert.equal(resolveGlobalShortcutAction(modN, actions)?.id, "sand.newAgent");

  const alternateFocus = keyboardEvent("l", { ctrlKey: true });
  assert.equal(resolveGlobalShortcutAction(alternateFocus, actions)?.id, "sand.focusInput");

  const editable = {
    matches: () => true,
  } as unknown as EventTarget;
  assert.equal(
    resolveGlobalShortcutAction(keyboardEvent("l", { ctrlKey: true, target: editable }), actions),
    null,
  );
  assert.equal(
    resolveGlobalShortcutAction(keyboardEvent("k", { ctrlKey: true, target: editable }), actions)?.id,
    "sand.commandPalette",
  );

  const focusNine = resolveGlobalShortcutAction(keyboardEvent("9", { metaKey: true }), actions);
  assert.equal(focusNine?.id, "sand.focusAgent9");
  void focusNine?.run();
  assert.equal(calls.at(-1), "agent:9");
});

test("global shortcut controller reference-counts listener and honors unstacked Escape ownership", () => {
  const calls: string[] = [];
  let listener: ((event: KeyboardEvent) => void) | null = null;
  let adds = 0;
  let removes = 0;
  const target = {
    addEventListener(_type: "keydown", next: (event: KeyboardEvent) => void) {
      adds += 1;
      listener = next;
    },
    removeEventListener(_type: "keydown", next: (event: KeyboardEvent) => void) {
      removes += 1;
      if (listener === next) listener = null;
    },
  };
  const controller = createGlobalKeyboardShortcutController([{
    id: "sand.openSettings",
    label: "Settings",
    hotkey: "mod+comma",
    run: () => calls.push("settings"),
  }]);
  const offA = controller.subscribe(target);
  const offB = controller.subscribe(target);
  assert.equal(adds, 1);

  controller.acceptOverlayState({
    isArmed: true,
    isOverlayStacked: false,
    close: () => calls.push("close"),
  });
  listener?.(keyboardEvent("Escape") as unknown as KeyboardEvent);
  assert.deepEqual(calls, ["close"]);

  const rootSearchInput = {
    closest: (selector: string) => selector.includes('role="dialog"')
      ? { querySelector: () => null }
      : null,
  } as unknown as EventTarget;
  listener?.(keyboardEvent("Escape", {
    defaultPrevented: true,
    target: rootSearchInput,
  }) as unknown as KeyboardEvent);
  assert.deepEqual(calls, ["close", "close"]);

  const nestedSearchInput = {
    closest: (selector: string) => selector.includes('role="dialog"')
      ? { querySelector: () => ({ ariaLabel: "Back" }) }
      : null,
  } as unknown as EventTarget;
  listener?.(keyboardEvent("Escape", {
    defaultPrevented: true,
    target: nestedSearchInput,
  }) as unknown as KeyboardEvent);
  assert.deepEqual(calls, ["close", "close"]);

  listener?.(keyboardEvent(",", { ctrlKey: true }) as unknown as KeyboardEvent);
  assert.deepEqual(calls, ["close", "settings"]);

  offA();
  assert.equal(removes, 0);
  offB();
  assert.equal(removes, 1);
});

test("computer shell model projects status, monitor inventory, VNC identity, cursor, and handoff copy", () => {
  const projected = projectComputerStatus({
    state: "running",
    vncUrl: "https://box.example/vnc.html?path=proxy%3Ftoken%3Ddisplay-7",
    windows: [{ title: "Browser" }],
    handoff: { requestId: "h1", instruction: "Sign in" },
  }, "known");
  assert.equal(projected.phase, "running");
  assert.equal(projected.isStatusKnown, true);
  assert.equal(projected.windows.length, 1);
  assert.equal(projected.handoff?.instruction, "Sign in");

  const pulling = projectComputerStatus({ state: "running", pull: { percent: 42 } }, "unknown");
  assert.equal(pulling.phase, "pulling");
  assert.equal(pulling.pullPercent, 42);

  const monitors = projectComputerMonitors([
    { status: "running", subagentType: "computerUse", subagentId: "a", title: "  Research  " },
    { status: "done", subagentType: "computerUse", subagentId: "b" },
  ], (id) => id === "a" ? { state: "running", vncUrl: "https://a.example/vnc" } : null);
  assert.deepEqual(monitors, [{
    subagentId: "a",
    title: "Research",
    vncUrl: "https://a.example/vnc",
    handoff: null,
  }]);

  assert.deepEqual(computerStageCopy({
    isScreenLoading: false,
    isScreenUnavailable: true,
    subjectLabel: "Bot",
    isEmptyLoading: false,
    pullPercent: null,
  }), {
    message: "Can't reach Bot's screen",
    progressPercent: null,
    isBusy: false,
    hasRetry: true,
  });

  const interactiveUrl = vncViewerUrl("https://host/vnc?x=1", true);
  const parsedUrl = new URL(interactiveUrl);
  assert.equal(parsedUrl.searchParams.get("autoconnect"), "true");
  assert.equal(parsedUrl.searchParams.get("sandInteractive"), "1");
  assert.deepEqual(vncIdentity("https://box.example/vnc?path=ws%3Ftoken%3Ddisplay-7"), {
    host: "box.example",
    display: "display-7",
  });
  assert.deepEqual(vncDimensions("https://box.example/sand-special-treatment-v1/vnc.html"), {
    width: 2048,
    height: 2048,
  });
  assert.deepEqual(retainWarmVncSources(["a", "b"], "c", 2), ["c", "a"]);

  assert.deepEqual(parseVncSession('{"phase":"rfb_disconnect","clean":true}'), {
    phase: "rfb_disconnect",
    clean: true,
  });
  assert.deepEqual(parseVncLiveness({
    phase: "post_connect",
    stallMs: 10,
    keys: 1,
    clicks: 2,
    moves: 3,
    inBytes: 4,
  }), {
    phase: "post_connect",
    stallMs: 10,
    keys: 1,
    clicks: 2,
    moves: 3,
    inBytes: 4,
  });

  const firstCursor = projectComputerCursor({
    agentId: "a",
    type: "move",
    x: 10,
    y: 20,
  }, null, 1_000);
  assert.ok(firstCursor);
  const clickCursor = projectComputerCursor({
    agentId: "a",
    type: "click",
    x: 11,
    y: 20,
  }, firstCursor, 1_200);
  assert.ok(clickCursor);
  assert.equal(clickCursor.clickSeq, 1);
  assert.equal(clickCursor.lastMovedAtMs, 1_200);
  assert.deepEqual(computerCursorPresentation(clickCursor, true), {
    isGliding: true,
    isVisible: true,
    press: { key: 1, delayMs: 500 },
  });

  const selectable = [
    { subagentId: "a", title: "A", vncUrl: "a", handoff: null },
    { subagentId: "b", title: "B", vncUrl: "b", handoff: { requestId: "h", instruction: "help" } },
  ];
  assert.equal(firstSelectedMonitor(selectable, null), "b");
  assert.equal(stepSelectedMonitor(selectable, "b", 1), "a");
  assert.deepEqual(handoffStatusLabel("dismissed"), { label: "Skipped", muted: true });
});
