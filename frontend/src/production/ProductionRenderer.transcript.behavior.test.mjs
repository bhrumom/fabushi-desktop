import assert from "node:assert/strict";
import test, { after, before } from "node:test";
import { createRequire } from "node:module";
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { mkdir, writeFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";

// Actions-only harness. It uses the same locked desktop Vite/Chromium tooling
// as ForwardMessageDialog.behavior.test.mjs, and mounts the production root.
const root = process.cwd();
const desktop = resolve(root, "desktop");
const require = createRequire(resolve(desktop, "package.json"));
const { chromium } = require("playwright");
const artifacts = resolve(root, "artifacts/native-message-ui");
const url = "http://127.0.0.1:14329/transcript-test.html";
const A = "human:alpha";
const B = "human:beta";
const TAIL = "getHumanConversationTranscriptTail";
const FULL = "getHumanConversationTranscript";
const ERROR_NAME = "Older messages unavailable";
// Controlled Host-shaped walk tokens remain opaque to every renderer owner.
// All eight Human cases use the stable path; legacy numeric-only compatibility
// remains in the existing shared RPC and pagination contract gates.
const ALPHA_41 = "htp1:11111111111111111111111111111111:23:41";
const ALPHA_21 = "htp1:11111111111111111111111111111111:43:21";
const ALPHA_1 = "htp1:11111111111111111111111111111111:63:1";
const ACCOUNT_B_101 = "htp1:22222222222222222222222222222222:0:101";
let server;
let browser;
let serverLog;
let serverClosed = false;
before(async () => {
  await mkdir(artifacts, { recursive: true });
  serverLog = createWriteStream(resolve(artifacts, "transcript-vite.log"));
  server = spawn(resolve(desktop, "node_modules/.bin/vite"), ["--host", "127.0.0.1", "--port", "14329", "--strictPort"], {
    cwd: desktop, stdio: "pipe", env: { ...process.env, CI: "1" },
  });
  server.stdout.pipe(serverLog, { end: false });
  server.stderr.pipe(serverLog, { end: false });
  server.on("error", (error) => serverLog.write(`Vite process error: ${error.message}\n`));
  server.once("close", () => { serverClosed = true; });
  let ready = false;
  for (let i = 0; i < 100; i++) {
    if (server.exitCode != null) throw new Error("transcript Vite server exited before readiness");
    try { const response = await fetch(url, { signal: AbortSignal.timeout(2_000) }); if (response.ok) { ready = true; break; } }
    catch {}
    await new Promise((resolveReady) => setTimeout(resolveReady, 100));
  }
  if (!ready) throw new Error("transcript fixture server did not start");
  browser = await chromium.launch({ headless: true });
});
after(async () => {
  await browser?.close();
  if (server != null && !serverClosed) {
    await new Promise((resolveExit) => {
      const force = setTimeout(() => server.kill("SIGKILL"), 5_000);
      server.once("close", () => { clearTimeout(force); resolveExit(); });
      if (server.exitCode == null && server.signalCode == null) server.kill("SIGTERM");
    });
  }
  await new Promise((resolveClose) => serverLog == null ? resolveClose() : serverLog.end(resolveClose));
});
const ids = (first, last, prefix = "m") => Array.from({ length: last - first + 1 }, (_, offset) => `${prefix}${first + offset}`);
const message = (id, text = `Fixture ${id}: a stable transcript row.\n`.repeat(5)) => ({
  id, kind: "message", role: "user", content: text, timestampMs: 1_700_000_000_000, delivery: "sent",
});
const messages = (first, last, prefix = "m") => ids(first, last, prefix).map((id) => message(id));
const historyPage = (entries, nextBeforeSeq, nextBeforeCursor) => {
  assert.equal(nextBeforeSeq == null, nextBeforeCursor == null, "Human fixture pages must carry both continuation fields or neither");
  return { entries, ...(nextBeforeSeq == null ? {} : { nextBeforeSeq, nextBeforeCursor }) };
};

async function fixture(t, { globalSearch = false } = {}) {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const diagnostics = [];
  const name = t.name.replace(/[^a-z0-9]+/gi, "-");
  page.on("pageerror", (error) => diagnostics.push({ kind: "pageerror", message: error.message }));
  page.on("console", (message) => { if (message.type() === "error") diagnostics.push({ kind: "console", message: message.text() }); });
  page.on("requestfailed", (request) => diagnostics.push({ kind: "requestfailed", url: request.url(), error: request.failure()?.errorText }));
  page.on("response", (response) => { if (response.status() >= 400) diagnostics.push({ kind: "http", url: response.url(), status: response.status() }); });
  t.after(async () => {
    const rows = await page.locator('.sand-virtual-transcript > [data-entry-id]').evaluateAll((nodes) => nodes.map((node) => node.getAttribute("data-entry-id"))).catch(() => []);
    await page.screenshot({ path: resolve(artifacts, `${name}.png`), fullPage: true }).catch(() => {});
    await page.evaluate(async () => {
      window.__transcriptHarness?.unmount();
      for (let frame = 0; frame < 2; frame++) await new Promise(requestAnimationFrame);
    }).catch((error) => diagnostics.push({ kind: "cleanup", message: String(error) }));
    const state = await page.evaluate(() => ({
      requests: window.__transcriptHarness?.requests() ?? [],
      unexpected: window.__transcriptHarness?.unexpected() ?? ["fixture did not mount"],
      ports: window.__transcriptHarness?.ports() ?? [],
      resources: window.__transcriptHarness?.resources() ?? [],
    })).catch((error) => ({ unexpected: [String(error)] }));
    await writeFile(resolve(artifacts, `${name}.browser.json`), JSON.stringify({ diagnostics, rows, ...state }, null, 2));
    await page.close();
    assert.deepEqual(state.unexpected, [], "undeclared boundary calls must not be hidden by the fixture");
    assert.deepEqual(diagnostics, [], "real renderer mount must not hide browser/runtime errors");
  });
  await page.goto(globalSearch ? `${url}?globalSearch=1` : url);
  try {
    await page.getByTestId("messenger-workspace").waitFor();
    await page.getByRole("button", { name: "Alpha", exact: true }).waitFor();
    await page.waitForFunction(() => !!window.__transcriptHarness);
  } catch (error) {
    throw new Error(`ProductionRenderer fixture failed to mount: ${JSON.stringify(diagnostics)}`, { cause: error });
  }
  const call = (method, ...args) => page.evaluate(([m, a]) => window.__transcriptHarness[m](...a), [method, args]);
  const matching = (records, method, where) => records.filter((request) => request.method === method && Object.entries(where).every(([key, value]) =>
    (key === "account" ? request.account : request.args[key] ?? null) === value));
  const count = async (method, where = {}) => matching(await call("requests"), method, where).length;
  const request = async (method, where = {}, after = 0) => {
    await page.waitForFunction(({ method, where, after }) => window.__transcriptHarness.requests().some((request) =>
      request.id > after && !request.settled && request.method === method
      && Object.entries(where).every(([key, value]) => (key === "account" ? request.account : request.args[key] ?? null) === value)), { method, where, after });
    const found = matching(await call("requests"), method, where).filter((request) => request.id > after && !request.settled).at(-1);
    if (method === TAIL) {
      const continuation = where.beforeSeq == null ? {} : { beforeSeq: where.beforeSeq, beforeCursor: where.beforeCursor };
      if (where.beforeSeq != null) assert.equal(typeof where.beforeCursor, "string", "each Human continuation must declare its exact walk token");
      assert.deepEqual(found.args, { conversationId: where.conversationId, limit: 200, ...continuation });
    }
    return found;
  };
  // Wait for real browser commit/effect opportunities, not a copied cache tick.
  const frames = () => page.evaluate(async () => {
    for (let frame = 0; frame < 8; frame++) await new Promise(requestAnimationFrame);
  });
  const rows = () => page.locator('.sand-virtual-transcript > [data-entry-id]').evaluateAll((nodes) => nodes.map((node) => node.getAttribute("data-entry-id")));
  const expectRows = async (expected) => {
    await page.waitForFunction((expected) => JSON.stringify([...document.querySelectorAll('.sand-virtual-transcript > [data-entry-id]')].map((row) => row.getAttribute("data-entry-id"))) === JSON.stringify(expected), expected);
    assert.deepEqual(await rows(), expected);
  };
  const resolveRequest = (request, value) => call("resolve", request.id, value);
  const rejectRequest = (request, reason = "PRIVATE_UPSTREAM_TRACE_token=fixture-secret") => call("reject", request.id, reason);
  const scroll = async (top) => {
    await page.getByRole("log", { name: "Conversation transcript" }).evaluate((node, value) => {
      node.scrollTop = value;
      node.dispatchEvent(new Event("scroll"));
    }, top);
    await frames();
  };
  const error = () => page.getByRole("alert", { name: ERROR_NAME });
  const retry = () => page.getByRole("button", { name: "Retry loading older messages" });
  const loadInitial = async (entries = messages(41, 64)) => {
    const initial = await request(TAIL, { conversationId: A, beforeSeq: null });
    await resolveRequest(initial, historyPage(entries, 41, ALPHA_41));
    await expectRows(entries.map((entry) => entry.id));
    // unprobed -> more retains hasOlder=true. Request history through the
    // mounted viewport's scroll listener, without assuming an effect reruns.
    await frames();
    await scroll(0);
    return initial;
  };
  const failedRetry = async () => {
    await loadInitial();
    const old = await request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 });
    await frames();
    await rejectRequest(old);
    await error().waitFor();
    await retry().click();
    const retrying = await request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 }, old.id);
    await frames();
    return retrying;
  };
  return { page, call, count, request, frames, rows, expectRows, resolveRequest, rejectRequest, scroll, error, retry, loadInitial, failedRetry };
}

test("UI-NMR-OLDER-RETRY-001 failure pauses automatic pages; keyboard and single-flight retries preserve the root cache", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  await f.loadInitial();
  const first = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 });
  await f.frames();
  // This row is not loaded yet: only the real pagination read overlay can
  // retain it across a failed read and two retries of the same opaque walk.
  await f.call("event", "transcript", { type: "updated", agentId: A, entry: message("m22", "IN_FLIGHT_UNLOADED_UPDATE") });
  await f.rejectRequest(first);
  await f.error().waitFor();
  await f.call("event", "transcript", { type: "updated", agentId: A, entry: message("m22", "LATEST_UNLOADED_UPDATE") });
  // Reselecting the current chat must keep its failed cursor and the pending
  // unloaded-row overlay; the existing m22 assertion after Retry verifies it.
  const initialReadsBeforeReselect = await f.count(TAIL, { conversationId: A, beforeSeq: null });
  await f.page.getByRole("button", { name: "Alpha", exact: true }).click();
  await f.frames();
  assert.equal(await f.error().isVisible(), true, "same-chat reselection must retain the explicit older-page error");
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 41 }), 1, "same-chat reselection must not automatically retry the failed cursor");
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: null }), initialReadsBeforeReselect, "same-chat reselection must not restart its cached initial read");
  assert.equal(await f.page.getByText("PRIVATE_UPSTREAM_TRACE", { exact: false }).count(), 0);
  for (let attempt = 0; attempt < 3; attempt++) {
    await f.scroll(attempt % 2 === 0 ? 0 : 700);
    await f.call("event", "transcript", { type: "updated", agentId: A, entry: message("m64", `Live row ${attempt}`) });
  }
  await f.frames();
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 41 }), 1);
  await f.expectRows(ids(41, 64));
  await f.retry().focus();
  await f.page.keyboard.press("Enter");
  const second = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 }, first.id);
  await f.frames();
  await f.rejectRequest(second, "PRIVATE_SECOND_FAILURE");
  await f.error().waitFor();
  await f.scroll(0);
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 41 }), 2);
  await f.retry().evaluate((button) => { button.click(); button.click(); });
  const third = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 }, second.id);
  await f.frames();
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 41 }), 3);
  assert.deepEqual(third.args, { conversationId: A, limit: 200, beforeSeq: 41, beforeCursor: ALPHA_41 });
  assert.equal(third.account, "fixture-a");
  await f.resolveRequest(third, historyPage([...messages(21, 40), message("m40"), message("m41", "stale overlap")], 21, ALPHA_21));
  await f.expectRows(ids(21, 64));
  assert.match(await f.page.locator('[data-entry-id="m22"]').innerText(), /LATEST_UNLOADED_UPDATE/);
  assert.equal(await f.error().count(), 0);
  assert.equal(await f.page.getByText("stale overlap", { exact: false }).count(), 0);
  const initialReads = await f.count(TAIL, { conversationId: A, beforeSeq: null });
  await f.page.getByRole("button", { name: "Beta", exact: true }).click();
  const beta = await f.request(TAIL, { conversationId: B, beforeSeq: null });
  await f.resolveRequest(beta, historyPage(messages(1, 3, "b")));
  await f.expectRows(ids(1, 3, "b"));
  await f.page.getByRole("button", { name: "Alpha", exact: true }).click();
  await f.expectRows(ids(21, 64));
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: null }), initialReads);
  // Normal successful automatic pagination remains reachable after reopening.
  await f.scroll(0);
  const resumed = await f.request(TAIL, { conversationId: A, beforeSeq: 21, beforeCursor: ALPHA_21 }, third.id);
  await f.resolveRequest(resumed, historyPage(messages(1, 20)));
  await f.expectRows(ids(1, 64));
  await f.scroll(0);
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 21 }), 1);
});

test("UI-NMR-INITIAL-RETRY-VIEWPORT-001 first-load Retry binds the replacement DOM viewport before prepend", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  const initial = await f.request(TAIL, { conversationId: A, beforeSeq: null });
  await f.rejectRequest(initial);
  await f.page.getByRole("heading", { name: "Couldn't load conversation" }).waitFor();
  assert.equal(await f.page.getByRole("log", { name: "Conversation transcript" }).count(), 0);
  await f.page.getByRole("button", { name: "Retry", exact: true }).click();
  const retry = await f.request(TAIL, { conversationId: A, beforeSeq: null }, initial.id);
  await f.resolveRequest(retry, historyPage(messages(41, 64), 41, ALPHA_41));
  await f.expectRows(ids(41, 64));
  await f.frames();
  await f.scroll(0);
  const older = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 }, retry.id);
  await f.frames();
  const before = await f.page.locator('[data-entry-id="m41"]').evaluate((row) => {
    const viewport = row.closest(".sand-virtual-transcript");
    return { top: row.getBoundingClientRect().top - viewport.getBoundingClientRect().top, scrollTop: viewport.scrollTop, height: viewport.scrollHeight, connected: viewport.isConnected };
  });
  assert.equal(before.connected, true);
  await f.resolveRequest(older, historyPage(messages(21, 40), 21, ALPHA_21));
  await f.expectRows(ids(21, 64));
  await f.frames();
  const after = await f.page.locator('[data-entry-id="m41"]').evaluate((row) => {
    const viewport = row.closest(".sand-virtual-transcript");
    return { top: row.getBoundingClientRect().top - viewport.getBoundingClientRect().top, scrollTop: viewport.scrollTop, height: viewport.scrollHeight };
  });
  assert.ok(after.height > before.height, "older rows must increase real scroll geometry");
  assert.ok(after.scrollTop > before.scrollTop, "the new attached viewport must receive the prepend correction");
  assert.ok(Math.abs(after.top - before.top) <= 2, `visible anchor moved: ${JSON.stringify({ before, after })}`);

  // An already loaded older row belongs to the pagination snapshot as well
  // as the root cache. Updating it commits while the next page is pending;
  // pendingAnchor.ready must prevent that commit from consuming the anchor.
  await f.scroll(0);
  const next = await f.request(TAIL, { conversationId: A, beforeSeq: 21, beforeCursor: ALPHA_21 }, older.id);
  const beforeNext = await f.page.locator('[data-entry-id="m21"]').evaluate((row) => {
    const viewport = row.closest(".sand-virtual-transcript");
    return { top: row.getBoundingClientRect().top - viewport.getBoundingClientRect().top, scrollTop: viewport.scrollTop, height: viewport.scrollHeight };
  });
  await f.call("event", "transcript", { type: "updated", agentId: A, entry: message("m21", "COMMITTED_DURING_PAGE: stable transcript row.\n".repeat(5)) });
  await f.frames();
  assert.match(await f.page.locator('[data-entry-id="m21"]').innerText(), /COMMITTED_DURING_PAGE/);
  assert.equal(await f.page.getByRole("log", { name: "Conversation transcript" }).evaluate((node) => node.scrollTop), beforeNext.scrollTop);
  await f.resolveRequest(next, historyPage([...messages(1, 20), message("m21", "STALE_PAGE_OVERLAP")]));
  await f.expectRows(ids(1, 64));
  await f.frames();
  const afterNext = await f.page.locator('[data-entry-id="m21"]').evaluate((row) => {
    const viewport = row.closest(".sand-virtual-transcript");
    return { top: row.getBoundingClientRect().top - viewport.getBoundingClientRect().top, scrollTop: viewport.scrollTop, height: viewport.scrollHeight };
  });
  assert.ok(afterNext.height > beforeNext.height, "the second prepend must change real scroll geometry");
  assert.ok(afterNext.scrollTop > beforeNext.scrollTop, "an intermediate row commit must not consume the pending anchor");
  assert.ok(Math.abs(afterNext.top - beforeNext.top) <= 2, `pending anchor moved: ${JSON.stringify({ beforeNext, afterNext })}`);
  assert.match(await f.page.locator('[data-entry-id="m21"]').innerText(), /COMMITTED_DURING_PAGE/);
  assert.equal(await f.page.getByText("STALE_PAGE_OVERLAP", { exact: false }).count(), 0);
});

test("UI-NMR-REFRESH-PAGE-RACE-001 a full reconnect refresh preserves live events and fences the late older response", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  await f.loadInitial();
  const old = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 });
  const marker = Math.max(...(await f.call("requests")).map((request) => request.id));
  await f.call("event", "coordinator-transport-state", { state: "connected" });
  const refresh = await f.request(FULL, { conversationId: A }, marker);
  await f.call("event", "transcript", { type: "updated", agentId: A, entry: message("m41", "LIVE_EDIT_WINS") });
  await f.call("event", "transcript", { type: "appended", agentId: A, entry: message("m65", "LIVE_APPEND_SURVIVES") });
  await f.resolveRequest(refresh, { entries: messages(1, 64) });
  await f.expectRows(ids(1, 65));
  await f.resolveRequest(old, historyPage([message("stale-old", "STALE_PAGE_MUST_NOT_APPEAR")], 1, ALPHA_1));
  await f.frames();
  await f.expectRows(ids(1, 65));
  assert.match(await f.page.locator('[data-entry-id="m41"]').innerText(), /LIVE_EDIT_WINS/);
  assert.match(await f.page.locator('[data-entry-id="m65"]').innerText(), /LIVE_APPEND_SURVIVES/);
  assert.equal(await f.page.getByText("STALE_PAGE_MUST_NOT_APPEAR", { exact: false }).count(), 0);
  const count = await f.count(TAIL, { conversationId: A });
  await f.scroll(0);
  assert.equal(await f.count(TAIL, { conversationId: A }), count, "complete refresh exhausts the old cursor");
});

test("UI-NMR-CLEAR-PAGE-001 authoritative clear fences an in-flight retry while a fresh first page remains usable", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  const old = await f.failedRetry();
  await f.call("event", "transcript", { type: "cleared", agentId: A });
  const fresh = await f.request(TAIL, { conversationId: A, beforeSeq: null }, old.id);
  await f.resolveRequest(fresh, historyPage([message("after-clear")]));
  await f.expectRows(["after-clear"]);
  await f.resolveRequest(old, historyPage([message("before-clear")], 1, ALPHA_1));
  await f.frames();
  await f.expectRows(["after-clear"]);
  assert.equal(await f.error().count(), 0);
});

test("UI-NMR-ACCOUNT-PAGE-001 another account reusing the conversation ID rejects the old retry failure", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  const old = await f.failedRetry();
  await f.call("switchAccount", "fixture-b");
  const fresh = await f.request(TAIL, { conversationId: A, beforeSeq: null, account: "fixture-b" }, old.id);
  await f.resolveRequest(fresh, historyPage([message("account-b-row")], 101, ACCOUNT_B_101));
  await f.expectRows(["account-b-row"]);
  await f.frames();
  await f.scroll(0);
  const freshOlder = await f.request(TAIL, { conversationId: A, beforeSeq: 101, beforeCursor: ACCOUNT_B_101, account: "fixture-b" }, fresh.id);
  await f.rejectRequest(old, "PRIVATE_ACCOUNT_A_FAILURE");
  await f.frames();
  await f.expectRows(["account-b-row"]);
  assert.equal(await f.error().count(), 0);
  assert.equal(await f.page.getByText("PRIVATE_ACCOUNT_A_FAILURE", { exact: false }).count(), 0);
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 41, account: "fixture-b" }), 0);
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 101, account: "fixture-b" }), 1);
  await f.resolveRequest(freshOlder, historyPage([message("account-b-older")]));
  await f.expectRows(["account-b-older", "account-b-row"]);
  assert.equal(await f.error().count(), 0);
});

test("UI-NMR-CONVERSATION-PAGE-001 switching conversations discards a late retry and retains the original cached cursor", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  const old = await f.failedRetry();
  await f.page.getByRole("button", { name: "Beta", exact: true }).click();
  const fresh = await f.request(TAIL, { conversationId: B, beforeSeq: null }, old.id);
  await f.resolveRequest(fresh, historyPage([message("beta-only")]));
  await f.expectRows(["beta-only"]);
  await f.resolveRequest(old, historyPage([message("old-alpha-page")], 1, ALPHA_1));
  await f.frames();
  await f.expectRows(["beta-only"]);
  await f.page.getByRole("button", { name: "Alpha", exact: true }).click();
  await f.expectRows(ids(41, 64));
  await f.scroll(0);
  const resumed = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 }, fresh.id);
  await f.resolveRequest(resumed, historyPage(messages(21, 40)));
  await f.expectRows(ids(21, 64));
  assert.equal(await f.page.locator('[data-entry-id="old-alpha-page"]').count(), 0);
});

test("UI-NMR-UNMOUNT-PAGE-001 unmount disposes the actual client and ignores a late retry rejection", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  const old = await f.failedRetry();
  await f.call("unmount");
  await f.frames();
  const count = await f.count(TAIL);
  await f.rejectRequest(old, "PRIVATE_UNMOUNTED_FAILURE");
  await f.frames();
  assert.equal(await f.page.getByTestId("messenger-workspace").count(), 0);
  assert.equal(await f.count(TAIL), count);
  assert.ok((await f.call("ports")).every((port) => port.closed), "actual Coordinator disposal closes its claimed port");
});

test("UI-NMR-FIND-PAGE-FAILURE-001 finding an unloaded match stops after one failed page and permits explicit Retry", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  await f.loadInitial();
  const first = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 });
  await f.resolveRequest(first, historyPage(messages(21, 40), 21, ALPHA_21));
  await f.expectRows(ids(21, 64));
  await f.scroll(10_000);
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 21 }), 0);
  await f.page.getByRole("log", { name: "Conversation transcript" }).focus();
  await f.page.keyboard.press("Control+f");
  await f.page.getByRole("textbox", { name: "Find in chat" }).fill("UNLOADED_NEEDLE");
  const search = await f.request("searchHumanMessages", { conversationId: A, query: "UNLOADED_NEEDLE" });
  await f.resolveRequest(search, { entries: [message("m1", "UNLOADED_NEEDLE")] });
  const page = await f.request(TAIL, { conversationId: A, beforeSeq: 21, beforeCursor: ALPHA_21 }, first.id);
  await f.frames();
  await f.rejectRequest(page, "PRIVATE_FIND_FAILURE");
  await f.error().waitFor();
  await f.frames();
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 21 }), 1, "Find's page budget must not replay a failed cursor");
  await f.page.getByRole("button", { name: "Next match" }).click();
  await f.frames();
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 21 }), 1);
  await f.retry().click();
  const retry = await f.request(TAIL, { conversationId: A, beforeSeq: 21, beforeCursor: ALPHA_21 }, page.id);
  await f.resolveRequest(retry, historyPage([message("m1", "UNLOADED_NEEDLE"), ...messages(2, 20)]));
  await f.expectRows(ids(1, 64));
  assert.equal(await f.error().count(), 0);
});


// NMR-UI-ENTRY-REFRESH-01 uses committed Host-shaped updates, never a fixture
// copy of the root's invalidation rules or a new transcript/cache owner.
const RESOURCE_CAP = 25 * 1024 * 1024;
const humanRefreshMessage = (remoteId, text, attachments = []) => ({
  ...message("human-server-message:" + remoteId, text),
  authorKind: "human", authorId: "peer-alpha", remoteMessageId: String(remoteId),
  richText: "", attachments, reactions: [],
});
const refreshAttachment = (path, name, mimeType, size = 16) => ({ path, fileName: name, mimeType, byteSize: size });
async function refreshRoot(t, initialRows) {
  const f = await fixture(t);
  const initial = await f.request(TAIL, { conversationId: A, beforeSeq: null });
  await f.resolveRequest(initial, historyPage(initialRows));
  await f.expectRows(initialRows.map((entry) => entry.id));
  await f.frames();
  const editor = f.page.locator('.sand-chat-input-dock [contenteditable="true"]');
  assert.equal(await editor.count(), 1);
  // promptEditorContent("", undefined) is one empty paragraph. Its trailing
  // ProseMirror BR is a cursor placeholder, not a serialized prompt newline.
  // Check the complete empty shape so whitespace, extra paragraphs, and empty
  // inline tokens cannot be mistaken for an empty account/conversation draft.
  const expectEmptyEditor = async (reason) => {
    const shape = await editor.evaluate((element) => ({
      text: element.textContent,
      children: [...element.childNodes].map((node) => ({
        name: node.nodeName, children: [...node.childNodes].map((child) => child.nodeName),
      })),
      trailingBreaks: element.querySelectorAll("br.ProseMirror-trailingBreak").length,
    }));
    assert.deepEqual(shape, {
      text: "", children: [{ name: "P", children: ["BR"] }], trailingBreaks: 1,
    }, reason);
  };
  const row = (id) => f.page.locator('[data-entry-id="' + id + '"]');
  const update = (entry, conversationId = A) => f.call("event", "transcript", { type: "updated", agentId: conversationId, entry });
  const openFind = async (query) => {
    if (await f.page.getByRole("textbox", { name: "Find in chat" }).count() === 0) {
      await f.page.getByRole("log", { name: "Conversation transcript" }).focus();
      await f.page.keyboard.press("Control+f");
    }
    await f.page.getByRole("textbox", { name: "Find in chat" }).fill(query);
  };
  const findState = async (query, value) => {
    await f.page.waitForFunction(([query, value]) => {
      const input = document.querySelector('input[aria-label="Find in chat"]');
      const status = document.querySelector('.sand-chat-find [role="status"]');
      return input?.value === query && status?.textContent === value;
    }, [query, value]);
    assert.equal(await f.page.locator('.sand-chat-find [role="status"]').textContent(), value);
  };
  const resourceRequests = () => f.call("resources");
  const resourceRequest = async (method, source, after = 0) => {
    await f.page.waitForFunction(({ method, source, after }) => window.__transcriptHarness.resources().some((request) =>
      request.id > after && !request.settled && request.method === method && request.args.source === source), { method, source, after });
    const found = (await resourceRequests()).find((request) => request.id > after && !request.settled
      && request.method === method && request.args.source === source);
    assert.ok(found);
    if (method === "readAttachmentBytes") assert.deepEqual(found.args, { source, maxBytes: RESOURCE_CAP });
    return found;
  };
  const resourceCount = async (method, source) => (await resourceRequests()).filter((request) =>
    request.method === method && (source === undefined || request.args.source === source)).length;
  const declareBytes = async (attachment) => {
    await f.call("declareResource", "readAttachmentBytes", { source: attachment.path, maxBytes: RESOURCE_CAP });
    await f.call("declareResource", "downloadAttachment", { source: attachment.path, suggestedName: attachment.fileName });
  };
  const resolveResource = (request, value) => f.call("resolveResource", request.id, value);
  const boundedRead = (request) => resolveResource(request, { kind: "too-large", size: RESOURCE_CAP + 1 });
  const noSend = async () => {
    assert.equal(await f.count("sendHumanMessage"), 0);
    assert.equal(await f.count("sendMessage"), 0);
    assert.equal(await f.count("forwardHumanMessage"), 0);
  };
  return { ...f, initial, editor, expectEmptyEditor, row, update, openFind, findState, resourceRequests,
    resourceRequest, resourceCount, declareBytes, resolveResource, boundedRead, noSend };
}

test("human_entry_refresh_requeries_search_and_discards_older_results", { timeout: 60_000 }, async (t) => {
  const firstRow = humanRefreshMessage(9001, "needle before update");
  const retainedRow = humanRefreshMessage(9002, "retained needle");
  const f = await refreshRoot(t, [firstRow, retainedRow]);
  await f.editor.fill("SEARCH_DRAFT_MUST_SURVIVE");
  await f.openFind("needle");
  const oldSuccess = await f.request("searchHumanMessages", { conversationId: A, query: "needle" });
  assert.deepEqual(oldSuccess.args, { conversationId: A, query: "needle", limit: 200 });
  await f.update({ ...firstRow, content: "edited without the queried word" });
  const current = await f.request("searchHumanMessages", { conversationId: A, query: "needle" }, oldSuccess.id);
  assert.deepEqual(current.args, oldSuccess.args);
  assert.equal(current.account, "fixture-a");
  await f.resolveRequest(current, { entries: [retainedRow] });
  await f.findState("needle", "1/1");
  await f.resolveRequest(oldSuccess, { entries: [{ ...firstRow, content: "needle needle needle" }] });
  await f.frames();
  await f.findState("needle", "1/1");
  assert.equal(await f.count("searchHumanMessages"), 2, "ordinary render/result commits cannot create search requests");
  assert.equal(await f.editor.innerText(), "SEARCH_DRAFT_MUST_SURVIVE");
  assert.equal(await f.row(firstRow.id).locator(".sand-message-prose").innerText(), "edited without the queried word");

  // A later accepted update retires a pending error as well as an old success.
  await f.update({ ...retainedRow, content: "retained needle edited once" });
  const oldFailure = await f.request("searchHumanMessages", { conversationId: A, query: "needle" }, current.id);
  await f.update({ ...retainedRow, content: "" });
  const empty = await f.request("searchHumanMessages", { conversationId: A, query: "needle" }, oldFailure.id);
  await f.resolveRequest(empty, { entries: [] });
  await f.findState("needle", "0/0");
  await f.rejectRequest(oldFailure, "RETIRED_SEARCH_FAILURE");
  await f.frames();
  await f.findState("needle", "0/0");
  assert.equal(await f.page.getByText("RETIRED_SEARCH_FAILURE", { exact: false }).count(), 0);
  assert.equal(await f.editor.innerText(), "SEARCH_DRAFT_MUST_SURVIVE");

  const beforeForeign = await f.count("searchHumanMessages");
  assert.equal(beforeForeign, 4);
  await f.update(humanRefreshMessage(9090, "needle in another conversation"), B);
  await f.frames();
  assert.equal(await f.count("searchHumanMessages"), beforeForeign, "another conversation cannot invalidate the current query");

  // Reusing the conversation and query under a new account cannot admit A's reply.
  await f.update({ ...firstRow, content: "needle old account held" });
  const accountARead = await f.request("searchHumanMessages", { conversationId: A, query: "needle" }, empty.id);
  const beforeAccountSwitch = await f.count("searchHumanMessages");
  await f.call("switchAccount", "fixture-b");
  const accountBTail = await f.request(TAIL, { conversationId: A, beforeSeq: null, account: "fixture-b" }, accountARead.id);
  const accountBRow = humanRefreshMessage(9010, "needle current account");
  await f.resolveRequest(accountBTail, historyPage([accountBRow]));
  await f.expectRows([accountBRow.id]);
  await f.frames();
  await f.expectEmptyEditor("an old account's composer does not enter the new account");
  assert.equal(await f.count("searchHumanMessages"), beforeAccountSwitch,
    "scope reset must reject an old render's query before any request enters account B");
  await f.editor.fill("ACCOUNT_B_DRAFT");
  await f.openFind("needle");
  const accountBRead = await f.request("searchHumanMessages", { conversationId: A, query: "needle", account: "fixture-b" }, accountARead.id);
  await f.resolveRequest(accountBRead, { entries: [accountBRow] });
  await f.findState("needle", "1/1");
  await f.resolveRequest(accountARead, { entries: [{ ...firstRow, content: "needle needle needle needle" }] });
  await f.frames();
  await f.findState("needle", "1/1");
  assert.equal(await f.editor.innerText(), "ACCOUNT_B_DRAFT");
  await f.expectRows([accountBRow.id]);

  // The same query in another chat has a separate result/error lifetime.
  await f.update({ ...accountBRow, content: "needle pending before chat switch" });
  const alphaRead = await f.request("searchHumanMessages", { conversationId: A, query: "needle", account: "fixture-b" }, accountBRead.id);
  const beforeChatSwitch = await f.count("searchHumanMessages");
  await f.page.getByRole("button", { name: "Beta", exact: true }).click();
  const betaTail = await f.request(TAIL, { conversationId: B, beforeSeq: null, account: "fixture-b" }, alphaRead.id);
  const betaRow = humanRefreshMessage(9020, "needle in Beta");
  await f.resolveRequest(betaTail, historyPage([betaRow]));
  await f.expectRows([betaRow.id]);
  await f.frames();
  assert.equal(await f.count("searchHumanMessages"), beforeChatSwitch,
    "scope reset must reject an old render's query before any request enters Beta");
  await f.openFind("needle");
  const betaRead = await f.request("searchHumanMessages", { conversationId: B, query: "needle", account: "fixture-b" }, alphaRead.id);
  await f.resolveRequest(betaRead, { entries: [betaRow] });
  await f.findState("needle", "1/1");
  await f.rejectRequest(alphaRead, "RETIRED_OTHER_CHAT_SEARCH_FAILURE");
  await f.frames();
  await f.findState("needle", "1/1");
  assert.equal(await f.page.getByText("RETIRED_OTHER_CHAT_SEARCH_FAILURE", { exact: false }).count(), 0);
  await f.expectRows([betaRow.id]);
  const beforeEmptyQuery = await f.count("searchHumanMessages");
  await f.page.getByRole("textbox", { name: "Find in chat" }).fill("");
  await f.frames();
  await f.update({ ...betaRow, content: "ordinary update with no active query" }, B);
  await f.frames();
  assert.equal(await f.count("searchHumanMessages"), beforeEmptyQuery);
  assert.equal(beforeEmptyQuery, 8, "only the eight explicit current-query stages may dispatch");
  assert.ok((await f.call("requests")).filter((request) => request.method === "searchHumanMessages").every((request) => request.settled));
  await f.noSend();
});

test("human_entry_refresh_retires_action_menus_without_hook_order_errors", { timeout: 60_000 }, async (t) => {
  const entry = humanRefreshMessage(9101, "ACTION_SOURCE");
  const f = await refreshRoot(t, [entry]);
  await f.call("holdActionRpc", "reactHumanMessage");
  await f.call("holdActionRpc", "searchHumanRecipients");
  await f.editor.fill("ACTION_DRAFT_MUST_SURVIVE");
  const row = f.row(entry.id);
  const menu = f.page.getByRole("menu", { name: "More message actions", exact: true });
  await row.locator(".sand-message-prose").click({ button: "right" });
  await menu.getByRole("menuitem", { name: "Forward", exact: true }).waitFor();
  await f.update({ ...entry, content: "", attachments: [], richText: "" });
  await menu.waitFor({ state: "detached" });
  await f.frames();
  assert.equal(await row.locator(".sand-message-prose").innerText(), "");
  assert.equal(await row.getByRole("button", { name: "More message actions", exact: true }).count(), 0);
  assert.equal(await f.count("searchHumanRecipients"), 0);
  assert.equal(await f.count("reactHumanMessage"), 0);
  await f.page.keyboard.press("Escape");
  assert.equal(await f.editor.innerText(), "ACTION_DRAFT_MUST_SURVIVE");

  const restored = { ...entry, content: "ACTION_RESTORED" };
  await f.update(restored);
  await f.frames();
  assert.equal(await menu.count(), 0, "eligibility restoration cannot reopen an old ordinary menu");
  await row.locator(".sand-message-prose").click({ button: "right" });
  await menu.getByRole("menuitem", { name: "Reply", exact: true }).click();
  await f.page.getByRole("button", { name: "Cancel reply", exact: true }).waitFor();
  assert.equal(await f.page.locator(".sand-prompt-reply-pill").innerText(), "ACTION_RESTORED");
  await f.page.getByRole("button", { name: "Cancel reply", exact: true }).click();
  assert.equal(await f.editor.innerText(), "ACTION_DRAFT_MUST_SURVIVE");

  // Keyboard interaction enters the real reaction control, without force clicks.
  await row.getByRole("button", { name: "Add reaction", exact: true }).focus();
  await row.getByRole("button", { name: "Add reaction", exact: true }).press("Enter");
  const picker = row.getByRole("group", { name: "Add reaction", exact: true });
  await picker.getByRole("button", { name: "React with 👍", exact: true }).waitFor();
  await f.update({ ...restored, content: "", attachments: [], richText: "" });
  await picker.waitFor({ state: "detached" });
  await f.frames();
  assert.equal(await f.count("reactHumanMessage"), 0, "retired menus cannot issue a reaction");
  await f.update(restored);
  await f.frames();
  assert.equal(await row.getByRole("group", { name: "Add reaction", exact: true }).count(), 0);
  await row.getByRole("button", { name: "Add reaction", exact: true }).focus();
  await row.getByRole("button", { name: "Add reaction", exact: true }).press("Enter");
  await row.getByRole("button", { name: "React with 👍", exact: true }).click();
  const reaction = await f.request("reactHumanMessage", { conversationId: A, entryId: entry.id, emoji: "👍" });
  assert.deepEqual(reaction.args, { conversationId: A, entryId: entry.id, emoji: "👍" });
  await f.resolveRequest(reaction, { ...restored, reactions: [{ emoji: "👍", by: "me" }] });
  await row.getByRole("button", { name: /Toggle your 👍 reaction/ }).waitFor();
  assert.equal(await f.count("reactHumanMessage"), 1);

  await row.locator(".sand-message-prose").click({ button: "right" });
  await menu.getByRole("menuitem", { name: "Forward", exact: true }).click();
  const forward = f.page.getByRole("dialog", { name: "Forward message", exact: true });
  await forward.waitFor();
  const recipients = await f.request("searchHumanRecipients", { sourceConversationId: A, sourceEntryId: entry.id, query: "" });
  assert.deepEqual(recipients.args, { sourceConversationId: A, sourceEntryId: entry.id, query: "", limit: 50 });
  await f.resolveRequest(recipients, []);
  await f.page.keyboard.press("Escape");
  await forward.waitFor({ state: "detached" });
  assert.equal(await f.count("searchHumanRecipients"), 1);
  assert.equal(await f.count("reactHumanMessage"), 1);
  assert.equal(await f.editor.innerText(), "ACTION_DRAFT_MUST_SURVIVE");
  await f.noSend();
});


test("human_entry_refresh_retires_removed_pdf_and_late_resource_reads", { timeout: 60_000 }, async (t) => {
  const pdfA = refreshAttachment("fixture-entry-refresh-a.pdf", "Original.pdf", "application/pdf");
  const pdfB = refreshAttachment("fixture-entry-refresh-b.pdf", "Retained.pdf", "application/pdf");
  const entry = humanRefreshMessage(9201, "PDF_SOURCE", [pdfA, pdfB]);
  const f = await refreshRoot(t, [entry]);
  await f.declareBytes(pdfA);
  await f.declareBytes(pdfB);
  await f.editor.fill("RESOURCE_DRAFT_MUST_SURVIVE");
  const row = f.row(entry.id);
  const pdf = (name) => f.page.getByRole("dialog", { name, exact: true });
  const openPdf = async (attachment) => {
    await row.getByRole("button", { name: "Open " + attachment.fileName, exact: true }).click();
    await pdf(attachment.fileName).waitFor();
  };
  await openPdf(pdfA);
  const retiredRead = await f.resourceRequest("readAttachmentBytes", pdfA.path);
  // The Linux fixture mounts the actual production WindowChrome. Reserving
  // its existing titlebar band must keep the real button pointer-reachable;
  // inspect hit testing without invoking any window-state operation.
  const minimize = f.page.locator(".sand-window-controls").getByRole("button", { name: "Minimize", exact: true });
  assert.equal(await minimize.count(), 1);
  const chromeHit = await minimize.evaluate((button) => {
    const viewer = document.querySelector(".sand-file-viewer");
    const carrier = document.querySelector(".sand-cover-drag");
    if (!(viewer instanceof HTMLElement) || !(carrier instanceof HTMLElement)) {
      throw new Error("Actual file viewer and WindowChrome carrier must be mounted");
    }
    const buttonRect = button.getBoundingClientRect();
    const chromeRect = carrier.getBoundingClientRect();
    const viewerRect = viewer.getBoundingClientRect();
    const centerX = buttonRect.left + buttonRect.width / 2;
    const centerY = buttonRect.top + buttonRect.height / 2;
    const hit = document.elementFromPoint(centerX, centerY);
    return {
      buttonWidth: buttonRect.width, buttonHeight: buttonRect.height, centerY,
      chromeTop: chromeRect.top, chromeHeight: chromeRect.height, chromeBottom: chromeRect.bottom,
      viewerTop: viewerRect.top, hitsMinimize: hit != null && button.contains(hit),
      hitTag: hit?.tagName ?? null, hitClass: hit?.getAttribute("class") ?? null,
    };
  });
  assert.ok(chromeHit.buttonWidth > 0 && chromeHit.buttonHeight > 0);
  assert.equal(chromeHit.chromeTop, 0);
  assert.equal(chromeHit.chromeHeight, 52);
  assert.ok(chromeHit.centerY >= chromeHit.chromeTop && chromeHit.centerY < chromeHit.chromeBottom);
  assert.equal(chromeHit.viewerTop, chromeHit.chromeBottom, "the actual preview begins below the existing titlebar band");
  assert.equal(chromeHit.hitsMinimize, true, "the open preview must preserve actual Minimize pointer hit testing: " + JSON.stringify(chromeHit));
  const handoffSelect = f.page.getByRole("combobox", { name: "Agent for Human handoff", exact: true });
  assert.equal(await handoffSelect.count(), 1);
  const handoffHit = await handoffSelect.evaluate((select) => {
    const header = select.closest(".sand-chat-header");
    const controls = document.querySelector(".sand-window-controls");
    if (!(header instanceof HTMLElement) || !(controls instanceof HTMLElement)) {
      throw new Error("Actual Human header and WindowChrome controls must be mounted");
    }
    const selectRect = select.getBoundingClientRect();
    const selectHit = document.elementFromPoint(selectRect.left + selectRect.width / 2, selectRect.top + selectRect.height / 2);
    const buttons = [...controls.querySelectorAll("button")].map((button) => {
      const rect = button.getBoundingClientRect();
      const hit = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2);
      return {
        label: button.getAttribute("aria-label"), width: rect.width, height: rect.height,
        hitsButton: hit != null && button.contains(hit),
        overlapsSelect: selectRect.left < rect.right && selectRect.right > rect.left
          && selectRect.top < rect.bottom && selectRect.bottom > rect.top,
      };
    });
    return {
      selectTag: select.tagName, selectWidth: selectRect.width, selectHeight: selectRect.height,
      hitsSelect: selectHit != null && select.contains(selectHit),
      headerRight: header.getBoundingClientRect().right, controlsLeft: controls.getBoundingClientRect().left,
      buttons,
    };
  });
  assert.equal(handoffHit.selectTag, "SELECT");
  assert.ok(handoffHit.selectWidth > 0 && handoffHit.selectHeight > 0);
  assert.equal(handoffHit.hitsSelect, true, "reserving window controls must preserve the real Human handoff target: " + JSON.stringify(handoffHit));
  assert.ok(handoffHit.headerRight <= handoffHit.controlsLeft, "the entire header hitbox must leave the actual window-control band clear: " + JSON.stringify(handoffHit));
  assert.deepEqual(handoffHit.buttons.map((button) => button.label), ["Minimize", "Maximize", "Close"]);
  for (const button of handoffHit.buttons) {
    assert.ok(button.width > 0 && button.height > 0);
    assert.equal(button.overlapsSelect, false, button.label + " must not overlap the actual handoff select");
    assert.equal(button.hitsButton, true, button.label + " must remain a real pointer target while preview is open");
  }
  // These are actual browser hit/geometry checks, without changing a window or
  // selecting an Agent. They do not claim native Electron drag-region behavior.
  await pdf(pdfA.fileName).getByRole("button", { name: "Download file", exact: true }).click();
  const retiredDownload = await f.resourceRequest("downloadAttachment", pdfA.path);
  assert.deepEqual(retiredDownload.args, { source: pdfA.path, suggestedName: pdfA.fileName });
  await f.update({ ...entry, content: "unrelated text edit", attachments: [pdfB, { ...pdfA }] });
  await f.frames();
  assert.equal(await pdf(pdfA.fileName).count(), 1, "same resource survives attachment reorder and text edits");
  assert.equal(await f.resourceCount("readAttachmentBytes", pdfA.path), 1);
  assert.equal(await f.resourceCount("readAttachmentBytes", pdfB.path), 0);
  assert.equal(await f.editor.innerText(), "RESOURCE_DRAFT_MUST_SURVIVE");

  await f.update({ ...entry, attachments: [pdfB] });
  await pdf(pdfA.fileName).waitFor({ state: "detached" });
  assert.equal(await row.getByRole("button", { name: "Open " + pdfB.fileName, exact: true }).count(), 1);
  await f.update({ ...entry, attachments: [pdfB, pdfA] });
  await openPdf(pdfA);
  const currentRead = await f.resourceRequest("readAttachmentBytes", pdfA.path, retiredRead.id);
  assert.equal(await pdf(pdfA.fileName).getByText("Loading PDF…", { exact: true }).count(), 1);
  await f.boundedRead(retiredRead);
  await f.call("rejectResource", retiredDownload.id, "RETIRED_PDF_DOWNLOAD_FAILURE");
  await f.frames();
  assert.equal(await pdf(pdfA.fileName).getByText("Loading PDF…", { exact: true }).count(), 1,
    "old same-path read completion cannot populate the new viewer");
  assert.equal(await pdf(pdfA.fileName).getByText("PDF too large to preview", { exact: true }).count(), 0);
  await f.boundedRead(currentRead);
  await pdf(pdfA.fileName).getByText("PDF too large to preview", { exact: true }).waitFor();
  // This is the real viewer's documented bounded-read branch, not PDF parsing.
  await pdf(pdfA.fileName).getByRole("button", { name: "Download file", exact: true }).click();
  const currentDownload = await f.resourceRequest("downloadAttachment", pdfA.path, retiredDownload.id);
  await f.resolveResource(currentDownload, true);

  const changedPdf = { ...pdfA, byteSize: pdfA.byteSize + 1 };
  await f.update({ ...entry, attachments: [pdfB, changedPdf] });
  await pdf(pdfA.fileName).waitFor({ state: "detached" });
  await openPdf(changedPdf);
  const changedRead = await f.resourceRequest("readAttachmentBytes", pdfA.path, currentRead.id);
  await f.boundedRead(changedRead);
  await pdf(pdfA.fileName).getByText("PDF too large to preview", { exact: true }).waitFor();
  await f.update({ ...entry, attachments: [] });
  await pdf(pdfA.fileName).waitFor({ state: "detached" });
  await f.update({ ...entry, attachments: [pdfB] });
  await openPdf(pdfB);
  const retainedRead = await f.resourceRequest("readAttachmentBytes", pdfB.path);
  await f.boundedRead(retainedRead);
  await pdf(pdfB.fileName).getByText("PDF too large to preview", { exact: true }).waitFor();
  await pdf(pdfB.fileName).getByRole("button", { name: "Download file", exact: true }).click();
  const retainedDownload = await f.resourceRequest("downloadAttachment", pdfB.path);
  await f.resolveResource(retainedDownload, true);
  await pdf(pdfB.fileName).getByRole("button", { name: "Close preview", exact: true }).click();
  await pdf(pdfB.fileName).waitFor({ state: "detached" });

  // Images exercise the actual media viewer: opening index 1 must remain B
  // when that same attachment moves to index 0.
  const imageA = refreshAttachment("fixture-entry-refresh-a.png", "Image A.png", "image/png");
  const imageB = refreshAttachment("fixture-entry-refresh-b.png", "Image B.png", "image/png");
  for (const [attachment, color] of [[imageA, "red"], [imageB, "blue"]]) {
    const svg = '<svg xmlns="http://www.w3.org/2000/svg" width="96" height="64"><rect width="96" height="64" fill="' + color + '"/></svg>';
    await f.call("declareResource", "resolveAttachmentMedia", { source: attachment.path }, true,
      { kind: "image", dataUrl: "data:image/svg+xml," + encodeURIComponent(svg), width: 96, height: 64 });
  }
  await f.update({ ...entry, attachments: [imageA, imageB] });
  await row.getByRole("img", { name: imageB.fileName, exact: true }).waitFor();
  await row.getByRole("button", { name: "Media preview", exact: true }).nth(1).click();
  const media = f.page.locator('.sand-media-viewer[role="dialog"]');
  await media.getByRole("img", { name: imageB.fileName, exact: true }).waitFor();
  await f.update({ ...entry, content: "media reordered", attachments: [{ ...imageB }, imageA] });
  await f.frames();
  assert.equal(await media.getByRole("img", { name: imageB.fileName, exact: true }).count(), 1);
  assert.equal(await media.getByRole("img", { name: imageA.fileName, exact: true }).count(), 0);
  await f.update({ ...entry, attachments: [imageA] });
  await media.waitFor({ state: "detached" });
  await f.update({ ...entry, attachments: [imageB, imageA] });
  await row.getByRole("img", { name: imageB.fileName, exact: true }).waitFor();
  await row.getByRole("button", { name: "Media preview", exact: true }).first().click();
  await media.getByRole("img", { name: imageB.fileName, exact: true }).waitFor();
  await f.update({ ...entry, attachments: [] });
  await media.waitFor({ state: "detached" });

  // Warm both chat caches with the same row ID and attachment tuple. This
  // exercises the real root mount scope key, not an incidental empty page.
  await f.update({ ...entry, attachments: [pdfB] });
  await f.page.getByRole("button", { name: "Beta", exact: true }).click();
  const betaTail = await f.request(TAIL, { conversationId: B, beforeSeq: null }, f.initial.id);
  const betaEntry = { ...entry, content: "BETA_SAME_ID_RESOURCE", attachments: [pdfB] };
  await f.resolveRequest(betaTail, historyPage([betaEntry]));
  await f.expectRows([entry.id]);
  await f.frames();
  await f.page.getByRole("button", { name: "Alpha", exact: true }).click();
  await f.frames();
  assert.equal(await row.locator(".sand-message-prose").innerText(), entry.content);
  assert.equal(await f.editor.innerText(), "RESOURCE_DRAFT_MUST_SURVIVE");
  await openPdf(pdfB);
  const alphaRead = await f.resourceRequest("readAttachmentBytes", pdfB.path, retainedRead.id);
  // The modal covers the sidebar. Use the existing real next-conversation
  // keyboard command, whose root handler includes Human conversations.
  await pdf(pdfB.fileName).getByRole("button", { name: "Close preview", exact: true }).focus();
  await f.page.keyboard.press("Alt+ArrowDown");
  await pdf(pdfB.fileName).waitFor({ state: "detached" });
  await f.frames();
  assert.equal(await row.locator(".sand-message-prose").innerText(), betaEntry.content);
  await f.expectEmptyEditor("the cached Beta conversation does not inherit Alpha's draft");
  await openPdf(pdfB);
  const betaRead = await f.resourceRequest("readAttachmentBytes", pdfB.path, alphaRead.id);
  await f.boundedRead(alphaRead);
  await f.frames();
  assert.equal(await pdf(pdfB.fileName).getByText("Loading PDF…", { exact: true }).count(), 1,
    "a cached chat with the same tuple cannot inherit another chat's resource lifetime");
  await f.boundedRead(betaRead);
  await pdf(pdfB.fileName).getByText("PDF too large to preview", { exact: true }).waitFor();
  await pdf(pdfB.fileName).getByRole("button", { name: "Close preview", exact: true }).click();
  await f.page.getByRole("button", { name: "Alpha", exact: true }).click();
  await f.frames();
  assert.equal(await f.editor.innerText(), "RESOURCE_DRAFT_MUST_SURVIVE");
  assert.ok((await f.resourceRequests()).every((request) => request.settled));
  await f.noSend();
});

test("human_entry_refresh_retires_spreadsheet_mount_when_source_changes", { timeout: 60_000 }, async (t) => {
  // A data URL makes the genuine browser download link self-contained. Reads
  // remain held at the real DesktopBridge byte boundary with its exact cap.
  const sheet = refreshAttachment("data:text/csv,Column%0Afixture", "Source.csv", "text/csv");
  const spare = refreshAttachment("fixture-retained.txt", "Retained.txt", "text/plain");
  const original = humanRefreshMessage(9301, "ORIGINAL_SHEET_OWNER", [sheet, spare]);
  const shadow = humanRefreshMessage(9302, "ANOTHER_ROW_WITH_THE_SAME_RESOURCE", [sheet]);
  const other = humanRefreshMessage(9303, "UNRELATED_ROW");
  const f = await refreshRoot(t, [original, shadow, other]);
  await f.declareBytes(sheet);
  await f.editor.fill("SHEET_DRAFT_MUST_SURVIVE");
  const viewer = f.page.getByRole("dialog", { name: sheet.fileName, exact: true });
  const sheetReads = async () => (await f.resourceRequests()).filter((request) =>
    request.method === "readAttachmentBytes" && request.args.source === sheet.path);
  const openSheet = async (entryId, account = "fixture-a") => {
    const before = await sheetReads();
    const chip = f.row(entryId).getByRole("button", { name: "Open " + sheet.fileName, exact: true });
    await chip.focus();
    await chip.press("Enter");
    await viewer.waitFor();
    // The real development bootstrap retains StrictMode. The current public
    // provider coalesces its consumers into one external read for this fresh
    // activation; observe that boundary before any source or row update.
    await f.page.waitForFunction(({ source, count }) => window.__transcriptHarness.resources().filter((request) =>
      request.method === "readAttachmentBytes" && request.args.source === source).length === count,
    { source: sheet.path, count: before.length + 1 });
    await f.frames();
    const all = await sheetReads();
    assert.equal(all.length, before.length + 1, "one fresh provider activation creates exactly one external read");
    assert.deepEqual(all.slice(0, before.length).map((request) => request.id), before.map((request) => request.id));
    const currentRead = all[before.length];
    assert.deepEqual(currentRead.args, { source: sheet.path, maxBytes: RESOURCE_CAP });
    assert.equal(currentRead.account, account);
    assert.equal(currentRead.settled, false);
    assert.equal(await viewer.getByText("Loading spreadsheet…", { exact: true }).count(), 1);
    return currentRead;
  };
  const retiredRead = await openSheet(original.id);
  const initialReadCount = await f.resourceCount("readAttachmentBytes", sheet.path);
  await viewer.getByRole("link", { name: "Download file", exact: true }).click();
  const retiredDownload = await f.resourceRequest("downloadAttachment", sheet.path);
  assert.deepEqual(retiredDownload.args, { source: sheet.path, suggestedName: sheet.fileName });
  await f.update({ ...other, content: "UNRELATED_ROW_UPDATED" });
  await f.update({ ...original, content: "TEXT_EDIT_WITH_SAME_RESOURCE", attachments: [spare, { ...sheet }] });
  await f.frames();
  assert.equal(await viewer.count(), 1);
  assert.equal(await f.resourceCount("readAttachmentBytes", sheet.path), initialReadCount,
    "unrelated updates and source attachment reorder create no additional reads");
  await f.call("event", "transcript", { type: "snapshot", activeAgentId: A,
    entries: [shadow, { ...other, content: "UNRELATED_ROW_UPDATED" }, { ...original, content: "TEXT_EDIT_WITH_SAME_RESOURCE", attachments: [spare, sheet] }] });
  await f.expectRows([shadow.id, other.id, original.id]);
  assert.equal(await viewer.count(), 1, "entry reorder cannot change a valid resource owner");
  assert.equal(await f.resourceCount("readAttachmentBytes", sheet.path), initialReadCount,
    "full transcript row reorder creates no additional reads");
  assert.equal(await f.editor.innerText(), "SHEET_DRAFT_MUST_SURVIVE");

  await f.update({ ...original, attachments: [spare] });
  await viewer.waitFor({ state: "detached" });
  assert.equal(await f.row(shadow.id).getByRole("button", { name: "Open " + sheet.fileName, exact: true }).count(), 1,
    "the same path remains present in a different row but cannot keep the old source authorized");
  const shadowRead = await openSheet(shadow.id);
  await f.boundedRead(retiredRead);
  await f.resolveResource(retiredDownload, true);
  await f.frames();
  assert.equal(await viewer.getByText("Loading spreadsheet…", { exact: true }).count(), 1);
  await f.boundedRead(shadowRead);
  await viewer.getByText("Spreadsheet too large to preview", { exact: true }).waitFor();
  // No parser substitute is provided; this positive control qualifies the
  // actual bounded-read window and download lifetime, not CSV/XLSX parsing.
  await viewer.getByRole("link", { name: "Download file", exact: true }).click();
  const shadowDownload = await f.resourceRequest("downloadAttachment", sheet.path, retiredDownload.id);
  await f.resolveResource(shadowDownload, true);

  const changedSheet = { ...sheet, byteSize: sheet.byteSize + 1 };
  const changedShadow = { ...shadow, attachments: [changedSheet] };
  await f.update(changedShadow);
  await viewer.waitFor({ state: "detached" });
  const changedRead = await openSheet(shadow.id);
  await f.boundedRead(changedRead);
  await viewer.getByText("Spreadsheet too large to preview", { exact: true }).waitFor();
  await viewer.getByRole("button", { name: "Close preview", exact: true }).click();
  await viewer.waitFor({ state: "detached" });
  const accountARead = await openSheet(shadow.id);
  await f.call("switchAccount", "fixture-b");
  await viewer.waitFor({ state: "detached" });
  const accountBTail = await f.request(TAIL, { conversationId: A, beforeSeq: null, account: "fixture-b" }, f.initial.id);
  const accountBEntry = { ...changedShadow, content: "ACCOUNT_B_SAME_ID_AND_RESOURCE" };
  await f.resolveRequest(accountBTail, historyPage([accountBEntry]));
  await f.expectRows([shadow.id]);
  await f.frames();
  await f.expectEmptyEditor("a new account with the same row and resource does not inherit the old draft");
  await f.editor.fill("SHEET_ACCOUNT_B_DRAFT");
  const accountBRead = await openSheet(shadow.id, "fixture-b");
  assert.equal(accountARead.account, "fixture-a");
  assert.equal(accountBRead.account, "fixture-b");
  await f.boundedRead(accountARead);
  await f.frames();
  assert.equal(await viewer.getByText("Loading spreadsheet…", { exact: true }).count(), 1);
  await f.boundedRead(accountBRead);
  await viewer.getByText("Spreadsheet too large to preview", { exact: true }).waitFor();
  assert.equal(await f.editor.innerText(), "SHEET_ACCOUNT_B_DRAFT");
  await f.update({ ...accountBEntry, attachments: [] });
  await viewer.waitFor({ state: "detached" });
  assert.ok((await f.resourceRequests()).every((request) => request.settled));
  await f.noSend();

  // Separately exercise the existing PUBLIC provider contract. This proves
  // its close/dispose callback fence; it does not pretend to capture private
  // callbacks from the mounted root or invoke detached React event handlers.
  const providerUrl = "/@fs" + resolve(root, "frontend/src/recovered/features/conversation/workspace/spreadsheet-viewer-provider.ts").replaceAll("\\", "/");
  const providerEvidence = await f.page.evaluate(async ({ providerUrl, cap }) => {
    const { createSpreadsheetViewerProvider } = await import(providerUrl);
    const calls = [];
    const pendingReads = [];
    const pendingDownloads = [];
    const provider = createSpreadsheetViewerProvider({
      readAttachmentBytes: (source, maxBytes) => {
        calls.push({ method: "read", source, maxBytes });
        return new Promise((resolveRead, rejectRead) => pendingReads.push({ resolve: resolveRead, reject: rejectRead }));
      },
      downloadAttachment: (source, suggestedName) => {
        calls.push({ method: "download", source, suggestedName });
        return new Promise((resolveDownload) => pendingDownloads.push(resolveDownload));
      },
    });
    const attachment = { path: "provider-source.csv", name: "Provider.csv", mimeType: "text/csv", size: 16 };
    const requireReadCount = (expected, reason) => {
      const actual = calls.filter((call) => call.method === "read").length;
      if (actual !== expected) throw new Error(reason + ": expected " + expected + " external reads, received " + actual);
      return actual;
    };
    const retired = provider.mount(attachment);
    if (retired == null) throw new Error("public spreadsheet provider did not mount");
    const oldRead = retired.readAttachmentBytes(attachment.path, cap);
    const oldReadTwin = retired.readAttachmentBytes(attachment.path, cap);
    const retiredReadCount = requireReadCount(1, "same pending activation must share one external read");
    const oldDownload = retired.onDownload();
    retired.close();
    const countAtClose = calls.length;
    const closedRead = await retired.readAttachmentBytes(attachment.path, cap);
    const closedDownload = await retired.onDownload();
    const noDispatchAfterClose = calls.length === countAtClose;
    const current = provider.mount({ ...attachment });
    if (current == null) throw new Error("same-path provider reopening failed");
    retired.close();
    const oldCloseCannotRetireCurrent = current.isOpen();
    const currentReadPromise = current.readAttachmentBytes(attachment.path, cap);
    const currentReadTwin = current.readAttachmentBytes(attachment.path, cap);
    const currentReadCount = requireReadCount(2, "fresh activation owns one new pending boundary");

    // Complete the OLD request while the NEW activation's slot is held.
    // A stale finally must not clear the new slot: its third consumer still
    // shares that pending boundary, regardless of adopting Promise identity.
    pendingReads.shift().resolve({ kind: "too-large", size: cap + 1 });
    pendingDownloads.shift()(true);
    const oldResults = { read: await oldRead, download: await oldDownload };
    const oldTwinResult = await oldReadTwin;
    const currentAfterOldCompletion = current.readAttachmentBytes(attachment.path, cap);
    const afterOldCompletionCount = requireReadCount(2, "old completion must not evict the current pending slot");
    pendingReads.shift().resolve({ kind: "too-large", size: cap + 1 });
    const [currentRead, currentTwinResult, currentThirdResult] = await Promise.all([
      currentReadPromise, currentReadTwin, currentAfterOldCompletion,
    ]);

    // A settled success/null/failure is not a lifetime cache. Each explicit
    // subsequent read dispatches once and only its concurrent consumer shares it.
    const nullRead = current.readAttachmentBytes(attachment.path, cap);
    const nullReadTwin = current.readAttachmentBytes(attachment.path, cap);
    const afterSuccessCount = requireReadCount(3, "a completed success must release its pending slot");
    pendingReads.shift().resolve(null);
    const nullResults = await Promise.all([nullRead, nullReadTwin]);

    const failure = new Error("CONTROLLED_PROVIDER_READ_FAILURE");
    const failedRead = current.readAttachmentBytes(attachment.path, cap);
    const failedReadTwin = current.readAttachmentBytes(attachment.path, cap);
    const afterNullCount = requireReadCount(4, "a completed null result must release its pending slot");
    // Attach BOTH error consumers before rejection; the pageerror oracle remains
    // strict and the real provider must propagate the original current error.
    const failuresObserved = Promise.all([failedRead, failedReadTwin].map((operation) =>
      operation.then(() => false, (error) => error === failure)));
    pendingReads.shift().reject(failure);
    const failureResults = await failuresObserved;

    const recoveredRead = current.readAttachmentBytes(attachment.path, cap);
    const recoveredReadTwin = current.readAttachmentBytes(attachment.path, cap);
    const afterErrorCount = requireReadCount(5, "a completed error must allow a fresh external read");
    pendingReads.shift().resolve({ kind: "too-large", size: cap + 2 });
    const recoveredResults = await Promise.all([recoveredRead, recoveredReadTwin]);
    const currentDownloadPromise = current.onDownload();
    pendingDownloads.shift()(true);
    const currentDownload = await currentDownloadPromise;
    provider.dispose();
    const countAtDispose = calls.length;
    const disposedRead = await current.readAttachmentBytes(attachment.path, cap);
    const disposedDownload = await current.onDownload();
    return { calls, closedRead, closedDownload, noDispatchAfterClose, oldCloseCannotRetireCurrent,
      oldResults, oldTwinResult, currentRead, currentTwinResult, currentThirdResult,
      nullResults, failureResults, recoveredResults, currentDownload, disposedRead, disposedDownload,
      readCounts: { retiredReadCount, currentReadCount, afterOldCompletionCount, afterSuccessCount, afterNullCount, afterErrorCount },
      allPublicRequestsSettled: pendingReads.length === 0 && pendingDownloads.length === 0,
      noDispatchAfterDispose: calls.length === countAtDispose, currentClosed: !current.isOpen() };
  }, { providerUrl, cap: RESOURCE_CAP });
  assert.deepEqual(providerEvidence, {
    calls: [
      { method: "read", source: "provider-source.csv", maxBytes: RESOURCE_CAP },
      { method: "download", source: "provider-source.csv", suggestedName: "Provider.csv" },
      { method: "read", source: "provider-source.csv", maxBytes: RESOURCE_CAP },
      { method: "read", source: "provider-source.csv", maxBytes: RESOURCE_CAP },
      { method: "read", source: "provider-source.csv", maxBytes: RESOURCE_CAP },
      { method: "read", source: "provider-source.csv", maxBytes: RESOURCE_CAP },
      { method: "download", source: "provider-source.csv", suggestedName: "Provider.csv" },
    ],
    closedRead: null, closedDownload: false, noDispatchAfterClose: true,
    oldCloseCannotRetireCurrent: true, oldResults: { read: null, download: false }, oldTwinResult: null,
    currentRead: { kind: "too-large", size: RESOURCE_CAP + 1 },
    currentTwinResult: { kind: "too-large", size: RESOURCE_CAP + 1 },
    currentThirdResult: { kind: "too-large", size: RESOURCE_CAP + 1 },
    nullResults: [null, null], failureResults: [true, true],
    recoveredResults: [{ kind: "too-large", size: RESOURCE_CAP + 2 }, { kind: "too-large", size: RESOURCE_CAP + 2 }],
    readCounts: { retiredReadCount: 1, currentReadCount: 2, afterOldCompletionCount: 2, afterSuccessCount: 3, afterNullCount: 4, afterErrorCount: 5 },
    allPublicRequestsSettled: true, currentDownload: true,
    disposedRead: null, disposedDownload: false, noDispatchAfterDispose: true, currentClosed: true,
  });
});

test("command_palette_searches_human_conversations_messages_and_files_and_paginates_older_targets", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t, { globalSearch: true });
  const initial = await f.request(TAIL, { conversationId: A, beforeSeq: null });
  await f.resolveRequest(initial, historyPage(messages(41, 64), 41, ALPHA_41));
  await f.expectRows(ids(41, 64));
  await f.frames();

  const palette = f.page.getByRole("dialog", { name: "Search", exact: true });
  await f.page.getByRole("button", { name: "Search", exact: true }).click();
  await palette.waitFor();
  await palette.getByRole("option").filter({ hasText: "Alpha" }).first().waitFor();
  await palette.getByRole("option").filter({ hasText: "Beta" }).first().waitFor();

  await f.call("holdActionRpc", "searchAgents");
  await f.call("holdActionRpc", "searchMedia");
  await palette.getByRole("combobox", { name: "Search", exact: true }).fill("needle");

  const mediaSearch = await f.request("searchMedia", { query: "needle", limit: 50 });
  const messageSearch = await f.request("searchAgents", { query: "needle" });
  await f.resolveRequest(mediaSearch, [
    {
      agentId: B,
      entryId: "b-file-1",
      fileName: "needle-report.pdf",
      ext: "pdf",
      mime: "application/pdf",
      kind: "pdf",
      timestampMs: 1_700_000_000_500,
      width: null,
      height: null,
    },
  ]);
  await f.resolveRequest(messageSearch, [
    {
      agentId: A,
      entryId: "m15",
      role: "assistant",
      timestampMs: 1_700_000_000_900,
      snippet: "Alpha older needle on page three",
    },
  ]);

  const messageOption = palette.getByRole("option").filter({ hasText: "Alpha older needle on page three" });
  const fileOption = palette.getByRole("option").filter({ hasText: "needle-report.pdf" });
  await messageOption.waitFor();
  await fileOption.waitFor();
  assert.match(await messageOption.innerText(), /Alpha to you/);
  assert.match(await fileOption.innerText(), /Beta/);
  assert.equal(await f.count(TAIL, { conversationId: A, beforeSeq: 41 }), 0);

  await messageOption.click();
  await palette.waitFor({ state: "detached" });

  const page2 = await f.request(TAIL, { conversationId: A, beforeSeq: 41, beforeCursor: ALPHA_41 }, initial.id);
  await f.resolveRequest(page2, historyPage(messages(21, 40), 21, ALPHA_21));
  const page3 = await f.request(TAIL, { conversationId: A, beforeSeq: 21, beforeCursor: ALPHA_21 }, page2.id);
  await f.resolveRequest(page3, historyPage(messages(1, 20)));
  await f.expectRows(ids(1, 64));
  await f.frames();
  assert.equal(await f.page.locator('[data-entry-id="m15"]').count(), 1);

  await f.page.getByRole("button", { name: "Search", exact: true }).click();
  await palette.waitFor();
  const emptyMediaSearch = await f.request("searchMedia", { query: "", limit: 50 }, mediaSearch.id);
  await f.resolveRequest(emptyMediaSearch, [
    {
      agentId: B,
      entryId: "b10",
      fileName: "beta-archive.pdf",
      ext: "pdf",
      mime: "application/pdf",
      kind: "pdf",
      timestampMs: 1_700_000_000_800,
      width: null,
      height: null,
    },
  ]);
  await palette.getByRole("tab", { name: "Files", exact: true }).click();
  const betaFileOption = palette.getByRole("option").filter({ hasText: "beta-archive.pdf" });
  await betaFileOption.waitFor();
  await betaFileOption.click();
  await palette.waitFor({ state: "detached" });

  const betaInitial = await f.request(TAIL, { conversationId: B, beforeSeq: null }, page3.id);
  await f.resolveRequest(betaInitial, historyPage(messages(21, 40, "b"), 21, ALPHA_21));
  const betaOlder = await f.request(TAIL, { conversationId: B, beforeSeq: 21, beforeCursor: ALPHA_21 }, betaInitial.id);

  await f.call("switchAccount", "fixture-b");
  const accountBMedia = await f.request("searchMedia", { query: "", limit: 50, account: "fixture-b" }, emptyMediaSearch.id);
  const accountBTail = await f.request(TAIL, { conversationId: A, beforeSeq: null, account: "fixture-b" }, betaOlder.id);
  await f.rejectRequest(betaOlder, "STALE_ACCOUNT_A_SEARCH_NAVIGATION");
  await f.resolveRequest(accountBMedia, []);
  await f.resolveRequest(accountBTail, historyPage(messages(101, 102, "acctb-")));
  await f.expectRows(["acctb-101", "acctb-102"]);
  await f.frames();
  assert.equal(await f.page.getByText("STALE_ACCOUNT_A_SEARCH_NAVIGATION", { exact: false }).count(), 0);
});
