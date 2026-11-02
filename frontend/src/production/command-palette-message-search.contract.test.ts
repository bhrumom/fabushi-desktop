import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import { setImmediate as nextTurn } from "node:timers/promises";
import type * as TypeScript from "../../../desktop/node_modules/typescript/lib/typescript.js";

import {
  commandPaletteMessagesFromSearch,
  createCommandPaletteMessageProvider,
  isCurrentCommandPaletteMessageResult,
  type CommandPaletteMessage,
  type CommandPaletteMessageSnapshot,
} from "./command-palette-message-provider.ts";
import {
  activateCommandPaletteEntry,
  commandPaletteEntries,
  fuzzyPaletteScore,
  normalizePaletteSearch,
  paletteSearchTokens,
  type CommandPaletteEntry,
  type CommandPaletteTab,
} from "./command-palette-model.ts";

interface GatewayCase {
  id: string;
  query: string;
  results: CommandPaletteMessage[];
}

const requiredCases = [
  ["final-sigma", "ος"],
  ["unicode-offsets", "needle"],
  ["partial-expansion", "\u0307y"],
  ["combining-only", "\u0307"],
  ["emoji-only", "🧭"],
  ["host-order", "rank"],
  ["nonfinal-sigma", "οσ"],
  ["no-match", "missing-fixture"],
  ["empty-query", " \t "],
];
let rustCases: GatewayCase[] | undefined;

function gatewayCases(): GatewayCase[] {
  if (rustCases != null) return rustCases;
  const path = process.env.OCRM_PALETTE_FIXTURE_PATH;
  assert.ok(path, "OCRM_PALETTE_FIXTURE_PATH must name the same-H Rust Gateway export; no mock fallback");
  const fixture = JSON.parse(readFileSync(path, "utf8"));
  assert.equal(fixture.contract, "PMS-v1");
  assert.equal(fixture.formatVersion, 1);
  assert.ok(Array.isArray(fixture.cases));
  assert.deepEqual(fixture.cases.map(({ id, query }: GatewayCase) => [id, query]), requiredCases);
  for (const candidate of fixture.cases) {
    assert.ok(Array.isArray(candidate.results));
    for (const result of candidate.results) {
      assert.deepEqual(Object.keys(result).sort(), ["agentId", "entryId", "role", "snippet", "timestampMs"]);
      assert.equal(typeof result.agentId, "string");
      assert.ok(result.agentId.length > 0);
      assert.equal(typeof result.entryId, "string");
      assert.equal(typeof result.snippet, "string");
      assert.ok(result.role === "user" || result.role === "assistant");
      assert.ok(Number.isFinite(result.timestampMs));
    }
  }
  rustCases = fixture.cases;
  return rustCases!;
}

function gatewayCase(id: string): GatewayCase {
  const result = gatewayCases().find((candidate) => candidate.id === id);
  assert.ok(result, `required real Gateway case: ${id}`);
  return result;
}

interface PendingCall {
  input: unknown;
  signal: AbortSignal | undefined;
  resolve(value: unknown): void;
  reject(error: unknown): void;
}

function controlledProvider() {
  const calls: PendingCall[] = [];
  const queued: PendingCall[] = [];
  const waiting: ((call: PendingCall) => void)[] = [];
  // This transport only controls settlement. It never searches or invents a
  // positive result, and deliberately ignores AbortSignal just like the caller.
  const provider = createCommandPaletteMessageProvider({
    searchAgents: (input, options) => new Promise((resolve, reject) => {
      const signal = options?.signal;
      assert.ok(signal instanceof AbortSignal);
      const call = { input, signal, resolve, reject };
      calls.push(call);
      const waiter = waiting.shift();
      if (waiter != null) waiter(call);
      else queued.push(call);
    }),
  });
  provider.setAvailable(true);
  return {
    provider,
    calls,
    nextCall(): Promise<PendingCall> {
      const call = queued.shift();
      return call != null ? Promise.resolve(call) : new Promise((resolve) => waiting.push(resolve));
    },
  };
}

type Harness = ReturnType<typeof controlledProvider>;

async function acceptGatewayCase(harness: Harness, id: string, query = gatewayCase(id).query) {
  const fixture = gatewayCase(id);
  assert.equal(query.trim(), fixture.query.trim(), "transport reply must correspond to its real Rust query");
  const pending = harness.provider.setQuery(query);
  const call = await harness.nextCall();
  assert.deepEqual(call.input, { query: query.trim() });
  call.resolve(fixture.results);
  const snapshot = await pending;
  assert.equal(snapshot.status, fixture.results.length === 0 ? "empty" : "ready");
  assert.deepEqual(snapshot.value, fixture.results);
  return snapshot;
}

function displayedMessages(
  snapshot: CommandPaletteMessageSnapshot,
  query: string,
  tab: CommandPaletteTab = "messages",
): readonly CommandPaletteMessage[] {
  return commandPaletteEntries({
    agents: [], commands: [], messages: snapshot.value, messageStatus: snapshot.status, query, tab,
  }).flatMap((entry) => entry.kind === "message" ? [entry.message] : []);
}

function assertRevoked(snapshot: CommandPaletteMessageSnapshot, query: string) {
  assert.equal(isCurrentCommandPaletteMessageResult(snapshot.value, query), false);
  assert.deepEqual(displayedMessages({ status: "ready", value: snapshot.value }, query), []);
}

test("pms_requires_real_rust_gateway_fixture", () => {
  const cases = gatewayCases();
  assert.deepEqual(cases.slice(0, 6).map(({ results }) => results.length), [2, 2, 1, 2, 1, 2]);
  assert.ok(cases.slice(6).every(({ results }) => results.length === 0));
  assert.ok(cases.every(({ results }) => results.every(({ entryId }) => entryId !== "hidden")));
});

test("pms_rust_sigma_expansion_contraction_and_partial_originals_reach_entries", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  try {
    for (const id of ["final-sigma", "unicode-offsets", "partial-expansion"]) {
      const fixture = gatewayCase(id);
      const snapshot = await acceptGatewayCase(harness, id);
      assert.deepEqual(displayedMessages(snapshot, fixture.query), fixture.results);
      assert.deepEqual(displayedMessages(snapshot, fixture.query, "all"), fixture.results);
    }
    assert.deepEqual(gatewayCase("final-sigma").results.map(({ snippet }) => snippet), ["prefix ΟΣ suffix", "ΟΣ"]);
    assert.deepEqual(gatewayCase("unicode-offsets").results.map(({ snippet }) => snippet), [
      `…${"K".repeat(10)}Needle`, `…${"İ".repeat(15)}Needle`,
    ]);
    assert.equal(gatewayCase("partial-expansion").results[0].snippet, `…${"z".repeat(30)}İY${"界".repeat(20)}…`);
    for (const id of ["nonfinal-sigma", "no-match"]) {
      const snapshot = await acceptGatewayCase(harness, id);
      assert.deepEqual(displayedMessages(snapshot, gatewayCase(id).query), []);
    }
  } finally { harness.provider.dispose(); }
});

test("pms_nonempty_empty_token_queries_keep_rust_messages_in_all", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  const agent = { id: "visible", name: "Visible", isGroup: false, isHidden: false };
  const command = { id: "action", label: "Action", keywords: [], run() {} };
  try {
    for (const id of ["combining-only", "emoji-only"]) {
      const fixture = gatewayCase(id);
      const snapshot = await acceptGatewayCase(harness, id);
      assert.deepEqual(paletteSearchTokens(fixture.query), []);
      const entries = commandPaletteEntries({
        agents: [agent], commands: [command], messages: snapshot.value, messageStatus: snapshot.status,
        query: fixture.query, tab: "all",
      });
      assert.deepEqual(entries, [
        { kind: "agent", agent }, { kind: "command", command },
        ...fixture.results.map((message) => ({ kind: "message", message })),
      ]);
      assert.deepEqual(displayedMessages(snapshot, fixture.query), fixture.results);
      assert.deepEqual(commandPaletteEntries({
        agents: [agent], commands: [command], messages: snapshot.value, messageStatus: "ready", query: " \t ", tab: "all",
      }), [{ kind: "agent", agent }, { kind: "command", command }]);
    }
  } finally { harness.provider.dispose(); }
});

function otherCategoryInputs() {
  return {
    agents: [
      { id: "person", name: "stale", isGroup: false, isHidden: true },
      { id: "group", name: "rank", isGroup: true, isHidden: false },
      { id: "hidden", name: "rank", isGroup: false, isHidden: true },
      { id: "person", name: "rank", isGroup: false, isHidden: false },
    ],
    commands: [{ id: "action", label: "rank", keywords: [], run() {} }],
    files: [{ agentId: "person", entryId: "file", fileName: "rank", ext: "txt", mime: "text/plain", kind: "text" as const, timestampMs: 1, width: null, height: null }],
    links: [{ url: "https://rank/" }],
    routines: [{ agentId: "person", automation: { id: "routine", name: "rank", triggerDescription: "weekly", createdAt: 1, lastRunAt: null } }],
  };
}

function entryIdentity(entry: CommandPaletteEntry): string {
  if (entry.kind === "agent") return `${entry.kind}:${entry.agent.id}`;
  if (entry.kind === "command") return `${entry.kind}:${entry.command.id}`;
  if (entry.kind === "file") return `${entry.kind}:${entry.file.entryId}`;
  if (entry.kind === "link") return `${entry.kind}:${entry.link.url}`;
  if (entry.kind === "routine") return `${entry.kind}:${entry.routine.automation.id}`;
  return `${entry.kind}:${entry.message.entryId}`;
}

test("pms_host_order_and_navigation_identity_survive_all_tab_placement", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  try {
    const fixture = gatewayCase("host-order");
    const snapshot = await acceptGatewayCase(harness, fixture.id);
    assert.ok(fuzzyPaletteScore("rank", "rank")! > fuzzyPaletteScore("a longer rank context", "rank")!, "fixture distinguishes Host order from the old fuzzy order");
    const entries = commandPaletteEntries({
      ...otherCategoryInputs(), messages: snapshot.value, messageStatus: snapshot.status, query: fixture.query, tab: "all",
    });
    assert.deepEqual(entries.map(entryIdentity), [
      "agent:person", "agent:group", "routine:routine", "file:file", "link:https://rank/", "command:action",
      "message:rank-context", "message:rank-short", "agent:hidden",
    ]);
    assert.deepEqual(displayedMessages(snapshot, fixture.query), fixture.results);
    const messageEntries = entries.filter((entry) => entry.kind === "message");
    let opened: CommandPaletteMessage | undefined;
    activateCommandPaletteEntry(messageEntries[1], () => assert.fail("message must use the message navigation callback"), undefined, undefined, (message) => { opened = message; });
    assert.equal(opened, snapshot.value[1], "navigation retains the exact accepted message object");
    assert.deepEqual(opened, fixture.results[1], "original agent/entry/role/time/snippet identity survives");
    for (const tab of ["agents", "groups", "actions", "files", "links", "routines"] as const) {
      assert.deepEqual(displayedMessages(snapshot, fixture.query, tab), []);
    }
  } finally { harness.provider.dispose(); }
});

test("pms_other_categories_keep_matching_deduplication_tabs_and_order", () => {
  const inputs = otherCategoryInputs();
  const expected = {
    all: ["agent:person", "agent:group", "routine:routine", "file:file", "link:https://rank/", "command:action", "agent:hidden"],
    agents: ["agent:person", "agent:hidden"], groups: ["agent:group"], actions: ["command:action"],
    files: ["file:file"], links: ["link:https://rank/"], routines: ["routine:routine"], messages: [],
  };
  for (const tab of Object.keys(expected) as CommandPaletteTab[]) {
    assert.deepEqual(commandPaletteEntries({ ...inputs, query: "rank", tab }).map(entryIdentity), expected[tab]);
  }
  assert.deepEqual(commandPaletteEntries({ ...inputs, query: "", tab: "all" }).map(entryIdentity), ["agent:person", "agent:group", "command:action"]);
  assert.equal(normalizePaletteSearch("Café 𠀀 𐐀"), "cafe 𠀀 𐐨");
  assert.notEqual(fuzzyPaletteScore("𠀀xxa", "𠀀a"), null);
  assert.equal(fuzzyPaletteScore("𠀀xxxxxxa", "𠀀a"), null);
  const alias = { id: "alias", name: "Unrelated", title: "CAFÉ", isGroup: false, isHidden: false };
  assert.deepEqual(commandPaletteEntries({ agents: [alias], commands: [], query: "cafe", tab: "agents" }), [{ kind: "agent", agent: alias }]);
});

test("pms_ready_array_provenance_rejects_forgery_clones_parser_output_and_query_aliases", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  try {
    const fixture = gatewayCase("final-sigma");
    const snapshot = await acceptGatewayCase(harness, fixture.id, " \tος\n");
    assert.deepEqual(displayedMessages(snapshot, " \nος\t "), fixture.results);
    assert.ok(Object.isFrozen(snapshot.value));
    assert.ok(snapshot.value.every(Object.isFrozen));
    const forged = fixture.results.map((message) => ({ ...message, query: fixture.query, status: "ready", requestId: 1, isCurrent: true }));
    for (const value of [fixture.results, forged, commandPaletteMessagesFromSearch(fixture.results), [...snapshot.value], snapshot.value.map((message) => ({ ...message })), JSON.parse(JSON.stringify(snapshot.value))]) {
      assertRevoked({ status: "ready", value }, fixture.query);
    }
    for (const query of ["ΟΣ", "οσ", "ος extra", "", " \t "]) {
      assert.deepEqual(displayedMessages(snapshot, query), []);
    }
    for (const status of ["loading", "failed", "cancelled", "unavailable", "empty"] as const) {
      assert.deepEqual(commandPaletteEntries({ agents: [], commands: [], messages: snapshot.value, messageStatus: status, query: fixture.query, tab: "messages" }), []);
    }
    assert.deepEqual(commandPaletteEntries({ agents: [], commands: [], messages: snapshot.value, query: fixture.query, tab: "messages" }), []);
  } finally { harness.provider.dispose(); }
});

test("pms_same_query_retry_failure_and_capability_failure_revoke_retained_arrays", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  const fixture = gatewayCase("final-sigma");
  try {
    const old = await acceptGatewayCase(harness, fixture.id);
    const retry = harness.provider.setQuery(fixture.query);
    assert.equal(harness.provider.getSnapshot().status, "loading");
    assert.equal(harness.provider.getSnapshot().value, old.value, "exercise the actual reused-array status transition");
    assertRevoked(old, fixture.query);
    const failedCall = await harness.nextCall();
    failedCall.reject(new Error("transport failed"));
    const failed = await retry;
    assert.equal(failed.status, "failed");
    assert.equal(failed.value, old.value);
    assertRevoked(failed, fixture.query);
    const current = await acceptGatewayCase(harness, fixture.id);
    assert.notEqual(current.value, old.value);
    assertRevoked(old, fixture.query);
    const unavailable = harness.provider.setQuery(fixture.query);
    (await harness.nextCall()).reject({ code: "source/capability-unavailable" });
    assert.equal((await unavailable).status, "unavailable");
    assertRevoked(current, fixture.query);
  } finally { harness.provider.dispose(); }
});

test("pms_cancel_reset_unavailable_empty_and_dispose_revoke_ready_and_inflight_results", { timeout: 15000 }, async () => {
  const fixture = gatewayCase("final-sigma");
  const revoke = {
    cancel: (h: Harness) => h.provider.cancel(),
    reset: (h: Harness) => h.provider.reset(),
    unavailable: (h: Harness) => h.provider.setAvailable(false),
    empty: (h: Harness) => { void h.provider.setQuery(" \t "); },
    dispose: (h: Harness) => h.provider.dispose(),
  };
  for (const [name, invalidate] of Object.entries(revoke)) {
    for (const inFlight of [false, true]) {
      const harness = controlledProvider();
      try {
        const old = await acceptGatewayCase(harness, fixture.id);
        const pending = inFlight ? harness.provider.setQuery(fixture.query) : undefined;
        const call = inFlight ? await harness.nextCall() : undefined;
        invalidate(harness);
        assertRevoked(old, fixture.query);
        const revoked = harness.provider.getSnapshot();
        assert.deepEqual(displayedMessages(revoked, fixture.query), [], name);
        if (call != null && pending != null) {
          assert.equal(call.signal?.aborted, true, name);
          assert.equal((await pending).status, "cancelled");
          call.resolve(fixture.results);
          await nextTurn();
          assert.equal(harness.provider.getSnapshot(), revoked, `${name}: ignored abort reply cannot publish`);
        }
        if (name === "dispose") {
          const count = harness.calls.length;
          assert.equal((await harness.provider.setQuery(fixture.query)).status, "unavailable");
          assert.equal(harness.calls.length, count);
        } else {
          harness.provider.setAvailable(true);
          const reopened = await acceptGatewayCase(harness, fixture.id);
          assert.deepEqual(displayedMessages(reopened, fixture.query), fixture.results);
          assertRevoked(old, fixture.query);
        }
      } finally { harness.provider.dispose(); }
    }
  }
});

test("pms_ignored_abort_A_B_A_replies_cannot_replace_the_current_ready_array", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  let publications = 0;
  harness.provider.subscribe(() => { publications += 1; });
  try {
    const a = gatewayCase("final-sigma");
    const b = gatewayCase("host-order");
    const firstA = harness.provider.setQuery(a.query);
    const oldA = await harness.nextCall();
    const requestB = harness.provider.setQuery(b.query);
    const oldB = await harness.nextCall();
    const currentA = harness.provider.setQuery(a.query);
    const currentCall = await harness.nextCall();
    assert.equal(oldA.signal?.aborted, true);
    assert.equal(oldB.signal?.aborted, true);
    currentCall.resolve(a.results);
    const ready = await currentA;
    const count = publications;
    oldA.resolve(a.results);
    oldB.reject(new Error("late B failure"));
    await nextTurn();
    assert.equal((await firstA).status, "cancelled");
    assert.equal((await requestB).status, "cancelled");
    assert.equal(harness.provider.getSnapshot(), ready);
    assert.equal(publications, count);
    assert.deepEqual(displayedMessages(ready, a.query), a.results);
  } finally { harness.provider.dispose(); }
});

test("pms_synchronous_loading_reentry_keeps_only_the_latest_request", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  const a = gatewayCase("final-sigma");
  const b = gatewayCase("host-order");
  let replacement: Promise<CommandPaletteMessageSnapshot> | undefined;
  let replaced = false;
  const unsubscribe = harness.provider.subscribe(() => {
    if (!replaced && harness.provider.getSnapshot().status === "loading") {
      replaced = true;
      replacement = harness.provider.setQuery(b.query);
    }
  });
  try {
    assert.equal((await harness.provider.setQuery(a.query)).status, "cancelled");
    const call = await harness.nextCall();
    assert.deepEqual(call.input, { query: b.query });
    call.resolve(b.results);
    assert.ok(replacement);
    const ready = await replacement;
    assert.equal(harness.calls.length, 1);
    assert.deepEqual(displayedMessages(ready, b.query), b.results);
    assert.deepEqual(displayedMessages(ready, a.query), []);
  } finally { unsubscribe(); harness.provider.dispose(); }
});

test("pms_malformed_rows_and_duplicate_transport_provenance_do_not_grant_authority", { timeout: 10000 }, async () => {
  const harness = controlledProvider();
  const fixture = gatewayCase("final-sigma");
  try {
    const malformed = [
      null, [], {}, { ...fixture.results[0], agentId: "" }, { ...fixture.results[0], entryId: "" },
      { ...fixture.results[0], role: "system" }, { ...fixture.results[0], timestampMs: Infinity },
      { ...fixture.results[0], timestampMs: "200", query: fixture.query, status: "ready" },
      { ...fixture.results[0], snippet: null },
    ];
    assert.deepEqual(commandPaletteMessagesFromSearch(malformed), []);
    const payload = [...malformed, ...fixture.results, { ...fixture.results[0], snippet: "forged duplicate", query: fixture.query, requestId: 1 }];
    const parsed = commandPaletteMessagesFromSearch(payload);
    assert.deepEqual(parsed, fixture.results);
    assertRevoked({ status: "ready", value: parsed }, fixture.query);
    const pending = harness.provider.setQuery(fixture.query);
    (await harness.nextCall()).resolve(payload);
    const ready = await pending;
    assert.deepEqual(displayedMessages(ready, fixture.query), fixture.results);
    const invalid = harness.provider.setQuery(fixture.query);
    (await harness.nextCall()).resolve({ results: fixture.results, query: fixture.query, status: "ready" });
    assert.equal((await invalid).status, "empty");
    assertRevoked(ready, fixture.query);
  } finally { harness.provider.dispose(); }
});

test("pms_component_and_production_caller_wire_current_message_status", () => {
  // AST assertions inspect executable declarations/JSX. Comments cannot satisfy
  // these checks; full rendered-browser acceptance remains a separate gate.
  const ts: typeof TypeScript = createRequire(new URL("../../../desktop/package.json", import.meta.url))("typescript");
  function source(name: string) {
    return ts.createSourceFile(name, readFileSync(new URL(name, import.meta.url), "utf8"), ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  }
  function nodes(root: TypeScript.Node, predicate: (node: TypeScript.Node) => boolean): TypeScript.Node[] {
    const found: TypeScript.Node[] = [];
    const visit = (node: TypeScript.Node) => {
      if (predicate(node)) found.push(node);
      ts.forEachChild(node, visit);
    };
    visit(root);
    return found;
  }
  function variable(root: TypeScript.Node, name: string): TypeScript.VariableDeclaration {
    const matches = nodes(root, (node) => ts.isVariableDeclaration(node) && ts.isIdentifier(node.name) && node.name.text === name);
    assert.equal(matches.length, 1, `one executable declaration of ${name}`);
    return matches[0] as TypeScript.VariableDeclaration;
  }
  function namedCalls(root: TypeScript.Node, name: string): TypeScript.CallExpression[] {
    return nodes(root, (node) => ts.isCallExpression(node) && node.expression.getText() === name) as TypeScript.CallExpression[];
  }
  function jsxExpression(node: TypeScript.JsxSelfClosingElement, name: string): string {
    const attributes = node.attributes.properties.filter((property) => ts.isJsxAttribute(property) && property.name.getText() === name);
    assert.equal(attributes.length, 1, `actual JSX attribute ${name}`);
    const attribute = attributes[0] as TypeScript.JsxAttribute;
    assert.ok(attribute.initializer != null && ts.isJsxExpression(attribute.initializer));
    assert.ok(attribute.initializer.expression);
    return attribute.initializer.expression.getText();
  }
  const palette = source("CommandPalette.tsx");
  const components = nodes(palette, (node) => ts.isFunctionDeclaration(node) && node.name?.text === "CommandPalette");
  assert.equal(components.length, 1);
  const entries = variable(components[0], "entries").initializer;
  assert.ok(entries != null && ts.isCallExpression(entries) && entries.expression.getText() === "useMemo");
  const calls = namedCalls(entries.arguments[0], "commandPaletteEntries");
  assert.equal(calls.length, 1);
  const input = calls[0].arguments[0];
  assert.ok(ts.isObjectLiteralExpression(input));
  for (const name of ["messages", "messageStatus"]) {
    assert.ok(input.properties.some((property) => ts.isShorthandPropertyAssignment(property) && property.name.text === name));
  }
  const dependencies = entries.arguments[1];
  assert.ok(ts.isArrayLiteralExpression(dependencies));
  for (const name of ["messages", "messageStatus", "query", "tab"]) {
    assert.ok(dependencies.elements.some((element) => ts.isIdentifier(element) && element.text === name), `memo tracks ${name}`);
  }

  const renderer = source("ProductionRenderer.tsx");
  const paletteNodes = nodes(renderer, (node) => ts.isJsxSelfClosingElement(node) && node.tagName.getText() === "CommandPalette");
  assert.equal(paletteNodes.length, 1);
  const mounted = paletteNodes[0] as TypeScript.JsxSelfClosingElement;
  assert.equal(jsxExpression(mounted, "messages"), "messageSnapshot.value");
  assert.equal(jsxExpression(mounted, "messageStatus"), "messageSnapshot.status");
  assert.equal(jsxExpression(mounted, "onSearchQueryChange"), "updatePaletteSearchQuery");
  assert.equal(jsxExpression(mounted, "onOpenMessage"), "openPaletteMessage");
  const providers = namedCalls(renderer, "createCommandPaletteMessageProvider");
  assert.equal(providers.length, 1);
  const transports = namedCalls(providers[0], "client.call");
  assert.equal(transports.length, 1);
  assert.equal(transports[0].arguments[0].getText(), '"searchAgents"');
  assert.equal(transports[0].arguments[1].getText(), "input");
  const snapshot = variable(renderer, "messageSnapshot").initializer;
  assert.ok(snapshot != null && ts.isCallExpression(snapshot));
  assert.equal(snapshot.expression.getText(), "useSyncExternalStore");
  assert.equal(snapshot.arguments[0].getText(), "messageProvider?.subscribe ?? emptyMessageSubscribe");
  assert.equal(snapshot.arguments[1].getText(), "messageProvider?.getSnapshot ?? readEmptyMessageSnapshot");
  const queryCallback = variable(renderer, "updatePaletteSearchQuery").initializer;
  assert.ok(queryCallback);
  const queryCalls = namedCalls(queryCallback, "messageProvider.setQuery");
  assert.equal(queryCalls.length, 1);
  assert.equal(queryCalls[0].arguments[0].getText(), "query");
  const lifecycle = namedCalls(renderer, "useEffect").filter((call) => namedCalls(call.arguments[0], "messageProvider.reset").length > 0);
  assert.equal(lifecycle.length, 1);
  assert.ok(namedCalls(lifecycle[0].arguments[0], "messageProvider.setAvailable").some((call) => call.arguments[0].kind === ts.SyntaxKind.FalseKeyword));
  assert.ok(namedCalls(lifecycle[0].arguments[0], "messageProvider.cancel").length > 0);
  const lifecycleDependencies = lifecycle[0].arguments[1];
  assert.ok(ts.isArrayLiteralExpression(lifecycleDependencies));
  assert.ok(lifecycleDependencies.elements.some((element) => element.getText() === "paletteAccountIdentity"));
  assert.ok(namedCalls(renderer, "useStrictModeSafeDisposal").some((call) => call.arguments[0].getText() === "messageProvider"));
  const openMessage = variable(renderer, "openPaletteMessage").initializer;
  assert.ok(openMessage);
  const targets = namedCalls(openMessage, "setPaletteMessageTarget");
  assert.equal(targets.length, 1);
  assert.equal(targets[0].arguments[0].getText(), "{ agentId: message.agentId, entryId: message.entryId }");
  assert.equal(namedCalls(openMessage, "openAgent")[0].arguments[0].getText(), "message.agentId");
});
