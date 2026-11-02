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

async function fixture(t) {
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
    })).catch((error) => ({ unexpected: [String(error)] }));
    await writeFile(resolve(artifacts, `${name}.browser.json`), JSON.stringify({ diagnostics, rows, ...state }, null, 2));
    await page.close();
    assert.deepEqual(state.unexpected, [], "undeclared boundary calls must not be hidden by the fixture");
    assert.deepEqual(diagnostics, [], "real renderer mount must not hide browser/runtime errors");
  });
  await page.goto(url);
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
