import assert from "node:assert/strict";
import test, { after, before } from "node:test";
import { createRequire } from "node:module";
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { mkdir, writeFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { intersectForwardRecipientResults } from "./forward-recipient-navigation.ts";
const root = process.cwd();
const desktop = resolve(root, "desktop");
const require = createRequire(resolve(desktop, "package.json"));
const { chromium } = require("playwright");
let server;
let browser;
let serverLog;
let serverClosed = false;
const artifacts = resolve(root, "artifacts/native-message-ui");
before(async () => {
  await mkdir(artifacts, { recursive: true });
  serverLog = createWriteStream(resolve(artifacts, "forward-vite.log"));
  server = spawn(resolve(desktop, "node_modules/.bin/vite"), ["--host", "127.0.0.1", "--port", "14319", "--strictPort"], {
    cwd: desktop, stdio: "pipe", env: { ...process.env, CI: "1" },
  });
  server.stdout.pipe(serverLog, { end: false });
  server.stderr.pipe(serverLog, { end: false });
  server.on("error", (error) => serverLog.write(`Vite process error: ${error.message}\n`));
  server.once("close", () => { serverClosed = true; });
  let ready = false;
  for (let i = 0; i < 100; i++) {
    if (server.exitCode != null) throw new Error("fixture Vite server exited before readiness");
    try { const res = await fetch("http://127.0.0.1:14319/forward-dialog-test.html", { signal: AbortSignal.timeout(2_000) }); if (res.ok) { ready = true; break; } }
    catch {}
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (!ready) throw new Error("fixture server did not start");
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
async function fixture(t) {
  const page = await browser.newPage();
  const diagnostics = [];
  const name = t.name.replace(/[^a-z0-9]+/gi, "-");
  page.on("pageerror", (error) => diagnostics.push({ kind: "pageerror", message: error.message }));
  page.on("console", (message) => { if (message.type() === "error") diagnostics.push({ kind: "console", message: message.text() }); });
  page.on("requestfailed", (request) => diagnostics.push({ kind: "requestfailed", url: request.url(), error: request.failure()?.errorText }));
  page.on("response", (response) => { if (response.status() >= 400) diagnostics.push({ kind: "http", url: response.url(), status: response.status() }); });
  t.after(async () => {
    await writeFile(resolve(artifacts, `${name}.browser.json`), JSON.stringify(diagnostics, null, 2));
    await page.close();
  });
  await page.goto("http://127.0.0.1:14319/forward-dialog-test.html");
  try {
    await page.waitForFunction(() => !!window.__forwardHarness && document.querySelector('input[type="search"]'));
  } catch (error) {
    await page.screenshot({ path: resolve(artifacts, `${name}.bootstrap.png`), fullPage: true });
    throw new Error(`Forward fixture failed to mount: ${JSON.stringify(diagnostics)}`, { cause: error });
  }
  const call = (method, ...args) => page.evaluate(([m, a]) => window.__forwardHarness[m](...a), [method, args]);
  const request = async (kind, count = 1) => {
    await page.waitForFunction(([k, c]) => window.__forwardHarness.requests().filter((r) => r.kind === k).length >= c, [kind, count]);
    return (await call("requests")).filter((r) => r.kind === kind).at(-1);
  };
  const settleSearch = async (recipient = "human:b") => {
    const found = await request("search");
    await call("resolve", found.id, [{ id: recipient, name: recipient }]);
    await page.getByRole("option", { name: recipient }).waitFor();
    return found;
  };
  const select = async (recipient = "human:b") => {
    await page.getByRole("option", { name: recipient }).locator('input[type="checkbox"]').check();
  };
  const submit = async () => page.getByRole("button", { name: /^(Forward|Retry)$/ }).click();
  return { page, call, request, settleSearch, select, submit };
}
test("current forward closes and settles exactly once", async (t) => {
  const f = await fixture(t); await f.settleSearch();
  await f.page.waitForFunction(() => document.activeElement === document.querySelector('input[type="search"]'));
  await f.page.keyboard.press("Tab");
  assert.equal(await f.page.getByRole("listbox").evaluate((node) => node === document.activeElement), true);
  await f.page.keyboard.press("Space");
  assert.equal(await f.page.getByRole("option", { name: "human:b" }).getAttribute("aria-selected"), "true");
  const themeColors = [];
  for (const mode of ["light", "dark"]) {
    await f.call("setTheme", mode);
    assert.equal(await f.page.locator("html").getAttribute("data-theme"), `cursor-${mode}`);
    const colors = await f.page.getByRole("dialog", { name: "Forward message" }).evaluate((node) => {
      const style = getComputedStyle(node);
      return { foreground: style.color, background: style.backgroundColor };
    });
    assert.notEqual(colors.background, "rgba(0, 0, 0, 0)");
    themeColors.push(colors);
    for (const [size, width, height] of [["wide", 1280, 800], ["narrow", 390, 844]]) {
      await f.page.setViewportSize({ width, height });
      const bounds = await f.page.getByRole("dialog", { name: "Forward message" }).boundingBox();
      assert.ok(bounds != null && bounds.x >= 0 && bounds.y >= 0 && bounds.x + bounds.width <= width + 1 && bounds.y + bounds.height <= height + 1);
      await f.page.screenshot({ path: resolve(artifacts, `forward-${mode}-${size}.png`), fullPage: true });
    }
  }
  assert.notEqual(themeColors[0].foreground, themeColors[1].foreground);
  assert.notEqual(themeColors[0].background, themeColors[1].background);
  await f.page.getByRole("listbox").focus();
  await f.page.keyboard.press("Control+Enter");
  const sent = await f.request("forward"); await f.call("resolve", sent.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  assert.equal((await f.call("events")).filter((e) => e.kind === "settled").length, 1);
});
test("account switch prevents old success from closing B dialog", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const old = await f.request("forward"); await f.call("switchAccount", "B"); await f.call("reopen");
  await f.call("resolve", old.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.getByRole("heading", { name: "Forward message" }).waitFor();
  assert.deepEqual(await f.call("events"), []);
});
test("logout prevents late error from being displayed", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const old = await f.request("forward"); await f.call("switchAccount", null);
  await f.call("reject", old.id, "private-old-token");
  assert.equal(await f.page.getByText("private-old-token").count(), 0);
});
test("A B A reuse still rejects old settlement", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const old = await f.request("forward"); await f.call("switchAccount", "B"); await f.call("switchAccount", "A"); await f.call("reopen");
  await f.call("resolve", old.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.getByRole("heading", { name: "Forward message" }).waitFor();
  assert.deepEqual(await f.call("events"), []);
});
test("closed dialog discards old search", async (t) => {
  const f = await fixture(t); const old = await f.request("search"); await f.call("close");
  await f.call("resolve", old.id, [{ id: "old", name: "OLD_SECRET" }]); await f.call("reopen");
  assert.equal(await f.page.getByText("OLD_SECRET").count(), 0);
});
test("replaced source discards old search", async (t) => {
  const f = await fixture(t); const old = await f.request("search"); await f.call("replaceSource", "message-2");
  await f.call("resolve", old.id, [{ id: "old", name: "OLD_SECRET" }]);
  const replacement = await f.request("search", 2);
  assert.deepEqual(replacement.args.sourceEntryIds, ["message-2"]);
  assert.equal(await f.page.getByText("OLD_SECRET").count(), 0);
});
test("unmount synchronously invalidates root generation", async (t) => {
  const f = await fixture(t); const old = await f.request("search");
  const scope = await f.call("unmount");
  assert.equal(scope.after, scope.before + 1);
  await f.call("resolve", old.id, [{ id: "old", name: "OLD_SECRET" }]);
});
test("late rejected search cannot display error in reopened dialog", async (t) => {
  const f = await fixture(t); const old = await f.request("search"); await f.call("close"); await f.call("reopen");
  await f.call("reject", old.id, "private-old-search");
  assert.equal(await f.page.getByText("private-old-search").count(), 0);
});
test("rapid double-submit dispatches only once", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select();
  await f.page.evaluate(() => { const form = document.querySelector(".fabushi-forward-dialog form"); form.requestSubmit(); form.requestSubmit(); });
  await f.request("forward");
  assert.equal((await f.call("requests")).filter((r) => r.kind === "forward").length, 1);
});
test("partial retry only sends failed destination with stable identity", async (t) => {
  const f = await fixture(t); const recipients = await f.request("search");
  await f.call("resolve", recipients.id, [{ id: "human:b", name: "human:b" }, { id: "human:c", name: "human:c" }]);
  await f.page.getByRole("option", { name: "human:b" }).waitFor();
  await f.select("human:b"); await f.select("human:c");
  const captions = f.page.getByRole("checkbox", { name: "Hide media caption", exact: true });
  const sender = f.page.getByRole("checkbox", { name: "Hide sender name", exact: true });
  await captions.check();
  assert.equal(await sender.isChecked(), true);
  assert.equal(await sender.isEnabled(), true);
  await sender.uncheck();
  assert.equal(await captions.isChecked(), false);
  await captions.check();
  await f.call("refreshMessage");
  assert.equal(await f.page.getByText("2 selected", { exact: true }).count(), 1);
  assert.equal(await captions.isChecked(), true);
  assert.equal((await f.call("requests")).filter((r) => r.kind === "search").length, 1);
  await f.submit(); const first = await f.request("forward");
  assert.equal(first.args.dropSenderNames, true);
  assert.equal(first.args.dropCaptions, true);
  await f.call("resolve", first.id, [{ conversationId: "human:b", status: "sent" }, { conversationId: "human:c", status: "failed", error: "retry" }]);
  await f.page.getByRole("alert").waitFor();
  await f.call("refreshMessage");
  assert.equal(await f.page.getByRole("alert").textContent(), "retry");
  assert.equal(await sender.isDisabled(), true);
  assert.equal(await captions.isDisabled(), true);
  assert.equal(await f.page.getByRole("option", { name: "human:b" }).locator('input[type="checkbox"]').isDisabled(), true);
  assert.equal(await f.page.getByRole("option", { name: "human:b" }).getAttribute("aria-selected"), "false");
  assert.equal(await f.page.getByText("1 selected", { exact: true }).count(), 1);
  await f.page.screenshot({ path: resolve(artifacts, "forward-immutable-retry-wide.png"), fullPage: true });
  await f.page.setViewportSize({ width: 390, height: 844 });
  const retryButton = await f.page.getByRole("button", { name: "Forward", exact: true }).boundingBox();
  assert.ok(retryButton != null && retryButton.x >= 0 && retryButton.y >= 0
    && retryButton.x + retryButton.width <= 391 && retryButton.y + retryButton.height <= 845);
  await f.page.screenshot({ path: resolve(artifacts, "forward-immutable-retry-narrow.png"), fullPage: true });
  // The keyboard path must obey the frozen intent as well as the disabled
  // checkbox; the confirmed destination cannot reenter the retry set.
  await f.page.getByRole("listbox").focus();
  await f.page.keyboard.press("Home");
  await f.page.keyboard.press("Space");
  assert.equal(await f.page.getByRole("option", { name: "human:b" }).getAttribute("aria-selected"), "false");
  await f.submit(); const second = await f.request("forward", 2);
  assert.deepEqual(second.args, { ...first.args, destinationConversationIds: ["human:c"] });
  await f.call("resolve", second.id, [{ conversationId: "human:c", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
});
test("query change does not invalidate valid submit", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const sent = await f.request("forward");
  await f.page.getByRole("searchbox").fill("another recipient");
  const changedQuery = await f.request("search", 2);
  assert.equal(changedQuery.args.query, "another recipient");
  await f.call("resolve", changedQuery.id, [{ id: "human:c", name: "human:c" }]);
  await f.page.getByRole("option", { name: "human:c" }).waitFor();
  await f.call("resolve", sent.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "settled"));
  assert.equal((await f.call("events")).length, 2);
});
test("credential refresh does not replace account dialog lifetime", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const sent = await f.request("forward"); await f.call("refreshCredential");
  await f.call("resolve", sent.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "settled"));
  assert.equal((await f.call("events")).filter((e) => e.kind === "settled").length, 1);
});

test("ambiguous missing duplicate and unexpected settlements retain the frozen retry intent", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const first = await f.request("forward");
  await f.call("reject", first.id, "Forward outcome is unknown");
  await f.page.getByRole("alert").waitFor();
  assert.equal(await f.page.getByRole("alert").textContent(), "Forward outcome is unknown");
  assert.equal(await f.page.getByRole("checkbox", { name: "Hide sender name", exact: true }).isDisabled(), true);

  const ambiguousResults = [
    [],
    [{ conversationId: "human:b", status: "sent" }, { conversationId: "human:b", status: "failed", error: "conflicting row" }],
    [{ conversationId: "human:b", status: "sent" }, { conversationId: "not-requested", status: "sent" }],
  ];
  for (const [index, result] of ambiguousResults.entries()) {
    await f.submit();
    const retry = await f.request("forward", index + 2);
    assert.deepEqual(retry.args, first.args);
    await f.call("resolve", retry.id, result);
    await f.page.getByRole("alert").waitFor();
    assert.match(await f.page.getByRole("alert").textContent(), /Delivery was not confirmed/);
    assert.equal(await f.page.getByText("1 selected", { exact: true }).count(), 1);
    assert.equal((await f.call("events")).filter((e) => e.kind === "closed").length, 0);
    assert.equal((await f.call("events")).at(-1).settlement[0].status, "failed");
  }
  await f.submit();
  const confirmed = await f.request("forward", 5);
  assert.deepEqual(confirmed.args, first.args);
  await f.call("resolve", confirmed.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  assert.equal((await f.call("events")).filter((e) => e.kind === "closed").length, 1);
});

test("recipient limit stays editable and counts selections outside the current search", async (t) => {
  const f = await fixture(t);
  const recipients = Array.from({ length: 33 }, (_, index) => ({ id: `destination-${index}`, name: `Recipient ${index + 1}` }));
  const initialSearch = await f.request("search");
  await f.call("resolve", initialSearch.id, recipients);
  await f.page.getByRole("option", { name: "Recipient 1", exact: true }).waitFor();
  for (let index = 0; index < 31; index++) {
    await f.page.getByRole("option", { name: `Recipient ${index + 1}`, exact: true }).locator('input[type="checkbox"]').check();
  }
  // The limit must also hold for two input events before React has repainted.
  await f.page.getByRole("listbox").evaluate((list) => {
    const inputs = list.querySelectorAll('input[type="checkbox"]');
    inputs[31].click();
    inputs[32].click();
  });
  await f.page.getByRole("alert").waitFor();
  assert.match(await f.page.getByRole("alert").textContent(), /Select up to 32 conversations/);
  assert.equal(await f.page.getByRole("option", { name: "Recipient 33", exact: true }).locator('input[type="checkbox"]').isChecked(), false);
  assert.equal(await f.page.getByText("32 selected", { exact: true }).count(), 1);
  assert.equal((await f.call("requests")).filter((r) => r.kind === "forward").length, 0);

  await f.page.getByRole("searchbox").fill("Recipient 33");
  const filtered = await f.request("search", 2);
  await f.call("resolve", filtered.id, [recipients[32]]);
  assert.equal(await f.page.getByText("32 selected", { exact: true }).count(), 1);
  await f.page.getByRole("option", { name: "Recipient 33", exact: true }).locator('input[type="checkbox"]').click();
  assert.equal(await f.page.getByRole("option", { name: "Recipient 33", exact: true }).locator('input[type="checkbox"]').isChecked(), false);

  await f.page.getByRole("searchbox").fill("");
  const all = await f.request("search", 3);
  await f.call("resolve", all.id, recipients);
  await f.page.getByRole("option", { name: "Recipient 1", exact: true }).locator('input[type="checkbox"]').uncheck();
  await f.page.getByRole("option", { name: "Recipient 33", exact: true }).locator('input[type="checkbox"]').check();
  assert.equal(await f.page.getByRole("alert").count(), 0);
  await f.submit(); const first = await f.request("forward");
  assert.equal(first.args.destinationConversationIds.length, 32);
  assert.equal(new Set(first.args.destinationConversationIds).size, 32);
  assert.equal(first.args.destinationConversationIds.includes("destination-0"), false);
  assert.equal(first.args.destinationConversationIds.includes("destination-32"), true);
  await f.call("resolve", first.id, first.args.destinationConversationIds.map((conversationId) => ({ conversationId, status: "sent" })));
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
});

test("closing a pending forward retires presentation without cancelling or settling a reopened intent", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const first = await f.request("forward");
  const close = f.page.getByRole("button", { name: "Close", exact: true });
  assert.equal(await close.isEnabled(), true);
  await close.click();
  await f.page.getByRole("dialog", { name: "Forward message" }).waitFor({ state: "hidden" });
  assert.equal((await f.call("pending")).includes(first.id), true);
  await f.call("reopen");
  const search = await f.request("search", 2);
  await f.call("resolve", search.id, [{ id: "human:c", name: "human:c" }]);
  await f.select("human:c"); await f.submit();
  const second = await f.request("forward", 2);
  assert.notEqual(second.args.clientNonce, first.args.clientNonce);
  await f.call("resolve", first.id, [{ conversationId: "human:b", status: "sent" }]);
  assert.equal((await f.call("events")).filter((e) => e.kind === "settled").length, 0);
  assert.equal((await f.call("events")).filter((e) => e.kind === "closed").length, 1);
  assert.equal(await f.page.getByRole("button", { name: "Forwarding…", exact: true }).isDisabled(), true);
  await f.call("resolve", second.id, [{ conversationId: "human:c", status: "sent" }]);
  await f.page.getByRole("dialog", { name: "Forward message" }).waitFor({ state: "hidden" });
  assert.equal((await f.call("events")).filter((e) => e.kind === "closed").length, 2);
  assert.equal((await f.call("events")).filter((e) => e.kind === "settled").length, 1);
});

test("late searches preserve send failure and retry destinations even when selected rows are hidden", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const first = await f.request("forward");
  await f.page.getByRole("searchbox").fill("another recipient");
  const searching = await f.request("search", 2);
  await f.call("reject", first.id, "Forward service unavailable");
  await f.page.getByRole("alert").waitFor();
  await f.call("resolve", searching.id, [{ id: "human:c", name: "human:c" }]);
  assert.equal(await f.page.getByRole("alert").textContent(), "Forward service unavailable");
  assert.equal(await f.page.getByText("1 selected", { exact: true }).count(), 1);
  assert.equal(await f.page.getByRole("option", { name: "human:c" }).locator('input[type="checkbox"]').isDisabled(), true);
  await f.page.getByRole("searchbox").fill("offline search");
  const offline = await f.request("search", 3);
  await f.call("reject", offline.id, "Search is offline");
  assert.equal(await f.page.getByRole("alert").textContent(), "Forward service unavailable");
  await f.submit();
  const retry = await f.request("forward", 2);
  assert.deepEqual(retry.args, first.args);
  await f.call("resolve", retry.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
});


for (const retirement of ["close", "source", "account-aba", "logout", "unmount"]) {
  test(`Coordinator readiness refuses a forward after ${retirement} retires its original dialog`, async (t) => {
    const f = await fixture(t);
    await f.settleSearch(); await f.select();
    await f.call("enableCoordinator");
    await f.submit();
    await f.page.waitForFunction(() => window.__forwardHarness.coordinatorState().attempts === 1);
    assert.equal((await f.call("requests")).filter((request) => request.kind === "forward").length, 0);
    if (retirement === "close") await f.page.getByRole("button", { name: "Close", exact: true }).click();
    else if (retirement === "source") await f.call("replaceSource", "message-replacement");
    else if (retirement === "account-aba") {
      await f.call("switchAccount", "B");
      await f.call("switchAccount", "A");
      await f.call("reopen");
    } else if (retirement === "logout") await f.call("switchAccount", null);
    else await f.call("unmount");
    // Source/visibility changes must commit their layout cleanup before the
    // external port becomes ready. Account/unmount also invalidate synchronously.
    await f.page.evaluate(() => new Promise((resolve) => requestAnimationFrame(resolve)));
    const eventsBeforeReady = await f.call("events");
    await f.call("readyCoordinator");
    await f.page.waitForFunction(() => window.__forwardHarness.coordinatorState().completions === 1);
    assert.equal((await f.call("requests")).filter((request) => request.kind === "forward").length, 0);
    assert.deepEqual(await f.call("events"), eventsBeforeReady);
    assert.deepEqual(await f.call("pending"), (await f.call("pending")).filter((id) => id.startsWith("search-")));
  });
}

test("current dialog forwards only after actual Coordinator readiness and settles once", async (t) => {
  const f = await fixture(t);
  await f.settleSearch(); await f.select();
  await f.call("enableCoordinator"); await f.submit();
  await f.page.waitForFunction(() => window.__forwardHarness.coordinatorState().attempts === 1);
  assert.equal((await f.call("requests")).filter((request) => request.kind === "forward").length, 0);
  await f.call("readyCoordinator");
  const request = await f.request("forward");
  assert.deepEqual(Object.keys(request.args).sort(), ["clientNonce", "destinationConversationIds", "dropCaptions", "dropSenderNames", "sourceConversationId", "sourceEntryId"]);
  assert.deepEqual(request.args.destinationConversationIds, ["human:b"]);
  await f.call("resolve", request.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.getByRole("dialog", { name: "Forward message" }).waitFor({ state: "hidden" });
  const events = await f.call("events");
  assert.equal(events.filter((event) => event.kind === "settled").length, 1);
  assert.equal(events.filter((event) => event.kind === "closed").length, 1);
  assert.equal((await f.call("requests")).filter((request) => request.kind === "forward").length, 1);
});

test("closing after the actual Coordinator frame preserves admitted work without settling a new dialog", async (t) => {
  const f = await fixture(t);
  await f.settleSearch(); await f.select();
  await f.call("enableCoordinator"); await f.call("readyCoordinator");
  await f.submit();
  const request = await f.request("forward");
  await f.page.getByRole("button", { name: "Close", exact: true }).click();
  await f.call("reopen");
  await f.page.getByRole("dialog", { name: "Forward message" }).waitFor();
  const eventsBeforeReply = await f.call("events");
  await f.call("resolve", request.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.coordinatorState().completions === 1);
  assert.deepEqual(await f.call("events"), eventsBeforeReply);
  assert.equal(await f.page.getByRole("dialog", { name: "Forward message" }).count(), 1);
  assert.equal((await f.call("requests")).filter((item) => item.kind === "forward").length, 1);
  assert.equal((await f.call("pending")).includes(request.id), false);
});
 
test("multi-source forwarding intersects recipients, preserves source order, and retries only failed pairs", async (t) => {
  const f = await fixture(t);
  const staleSearch = await f.request("search");
  await f.call("addSource", "message-2");
  const search = await f.request("search", 2);
  assert.deepEqual(search.args.sourceEntryIds, ["message-1", "message-2"]);
  await f.call("resolve", staleSearch.id, [{ id: "old", name: "OLD_SECRET" }]);
  await f.call("resolve", search.id, [{ id: "human:b", name: "human:b" }]);
  await f.page.getByRole("option", { name: "human:b" }).waitFor();
  await f.select();
  await f.submit();
  const first = await f.request("forward");
  assert.equal(first.args.sourceEntryId, "message-1");
  await f.call("resolve", first.id, [{ conversationId: "human:b", status: "sent" }]);
  const second = await f.request("forward", 2);
  assert.equal(second.args.sourceEntryId, "message-2");
  assert.notEqual(second.args.clientNonce, first.args.clientNonce);
  await f.call("resolve", second.id, [{ conversationId: "human:b", status: "failed", error: "temporary delivery failure" }]);
  await f.page.getByRole("alert").waitFor();
  assert.equal(await f.page.getByText("1 selected", { exact: true }).count(), 1);
  await f.submit();
  const retry = await f.request("forward", 3);
  assert.equal(retry.args.sourceEntryId, "message-2");
  assert.equal(retry.args.clientNonce, second.args.clientNonce);
  assert.deepEqual(retry.args.destinationConversationIds, ["human:b"]);
  await f.call("resolve", retry.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((event) => event.kind === "closed"));
  assert.equal((await f.call("requests")).filter((request) => request.kind === "forward").length, 3);
});

test("removing a source message updates the draft and recipient search scope", async (t) => {
  const f = await fixture(t);
  const staleSearch = await f.request("search");
  await f.call("addSource", "message-2");
  await f.request("search", 2);
  await f.page.getByRole("button", { name: "Remove message message-1", exact: true }).click();
  const search = await f.request("search", 3);
  assert.deepEqual(search.args.sourceEntryIds, ["message-2"]);
  await f.call("resolve", staleSearch.id, [{ id: "old", name: "OLD_SECRET" }]);
  await f.call("resolve", search.id, [{ id: "human:b", name: "human:b" }]);
  await f.select();
  await f.submit();
  const sent = await f.request("forward");
  assert.equal(sent.args.sourceEntryId, "message-2");
  await f.call("resolve", sent.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((event) => event.kind === "closed"));
});

test("recipient result intersection fails closed on any source denial and removes duplicate identities", () => {
  assert.deepEqual(intersectForwardRecipientResults([]), []);
  assert.deepEqual(intersectForwardRecipientResults([
    [{ id: "human:a", name: "Alpha" }, { id: "human:b", name: "Beta" }, { id: "human:b", name: "Duplicate Beta" }],
    [{ id: "human:b", name: "Beta from source two" }, { id: "human:c", name: "Gamma" }],
  ]), [{ id: "human:b", name: "Beta" }]);
  assert.deepEqual(intersectForwardRecipientResults([
    [{ id: "human:a", name: "Alpha" }],
    [],
  ]), []);
});
