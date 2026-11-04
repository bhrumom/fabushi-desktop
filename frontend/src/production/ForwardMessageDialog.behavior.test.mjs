import assert from "node:assert/strict";
import test, { after, before } from "node:test";
import { createRequire } from "node:module";
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { mkdir, writeFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
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
async function fixture(t, variant = "") {
  const page = await browser.newPage();
  const diagnostics = [];
  const name = `${t.name}${variant}`.replace(/[^a-z0-9]+/gi, "-");
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
  const settleSearch = async (recipient = "human:b", name = recipient) => {
    const found = await request("search");
    await call("resolve", found.id, [{ id: recipient, name }]);
    await page.getByRole("option", { name, exact: true }).waitFor();
    return found;
  };
  const select = async (recipient = "human:b") => {
    await page.getByRole("option", { name: recipient }).locator('input[type="checkbox"]').check();
  };
  const submit = async () => page.getByRole("button", { name: "Forward", exact: true }).click();
  return { page, call, request, settleSearch, select, submit };
}

async function assertPreReadyForwardRetired(t, retirement) {
  const f = await fixture(t, `-readiness-${retirement}`);
  await f.settleSearch(); await f.select();
  await f.call("holdCoordinatorReadiness");
  await f.submit();
  await f.page.getByRole("button", { name: "Forwarding…", exact: true }).waitFor();
  assert.deepEqual(await f.call("forwardFrames"), []);
  if (retirement === "close") {
    await f.page.getByRole("button", { name: "Close", exact: true }).click();
    await f.page.getByRole("dialog", { name: "Forward message" }).waitFor({ state: "hidden" });
    await f.call("reopen");
  } else if (retirement === "source") {
    await f.call("replaceSource", "message-2");
    await f.request("search", 2);
  } else if (retirement === "unmount") {
    const scope = await f.call("unmount");
    assert.equal(scope.after, scope.before + 1);
  } else {
    await f.call("switchAccount", retirement === "logout" ? null : "B");
    await f.page.getByRole("dialog", { name: "Forward message" }).waitFor({ state: "hidden" });
    if (retirement === "A-B-A") await f.call("switchAccount", "A");
    if (retirement !== "logout") await f.call("reopen");
  }
  // The real client had an unresolved reconnect-ready promise. Retiring the
  // original dialog must reject its call before any mutation frame is posted.
  await f.call("releaseCoordinatorReadiness");
  assert.deepEqual(await f.call("forwardFrames"), []);
  assert.equal((await f.call("requests")).filter((request) => request.kind === "forward").length, 0);
  assert.equal((await f.call("events")).filter((event) => event.kind === "settled").length, 0);
  assert.equal(await f.page.getByText("Coordinator request scope expired", { exact: false }).count(), 0);
  if (retirement === "unmount") return;
  if (retirement === "logout") {
    await f.call("switchAccount", "B");
    await f.call("reopen");
  }
  // The current account/source/dialog can still send once; the retired request
  // must not revive when the same transport or account becomes ready again.
  await f.request("search", 2);
  await f.settleSearch("human:current"); await f.select("human:current"); await f.submit();
  const sent = await f.request("forward");
  const frames = await f.call("forwardFrames");
  assert.equal(frames.length, 1);
  assert.equal(frames[0].method, "forwardHumanMessage");
  assert.equal(sent.args.sourceEntryId, retirement === "source" ? "message-2" : "message-1");
  assert.deepEqual(sent.args.destinationConversationIds, ["human:current"]);
  assert.deepEqual(Object.keys(frames[0].args).sort(), [
    "clientNonce", "destinationConversationIds", "dropCaptions",
    "dropSenderNames", "sourceConversationId", "sourceEntryId",
  ]);
  await f.call("resolve", sent.id, [{ conversationId: "human:current", status: "sent" }]);
  await f.page.getByRole("dialog", { name: "Forward message" }).waitFor({ state: "hidden" });
  await f.call("releaseCoordinatorReadiness");
  assert.equal((await f.call("forwardFrames")).length, 1);
  assert.equal((await f.call("events")).filter((event) => event.kind === "settled").length, 1);
}

test("current forward closes and settles exactly once", async (t) => {
  const f = await fixture(t); await f.settleSearch();
  await f.page.waitForFunction(() => document.activeElement === document.querySelector('input[type="search"]'));
  await f.page.keyboard.press("Enter");
  assert.equal(await f.page.getByRole("option", { name: "human:b" }).getAttribute("aria-selected"), "true");
  await f.page.locator("[data-ui-dialog-backdrop]").click({ position: { x: 4, y: 4 } });
  assert.equal(await f.page.getByRole("dialog", { name: "Forward message" }).count(), 1);
  assert.equal((await f.call("events")).filter((e) => e.kind === "closed").length, 0);
  await f.page.getByRole("searchbox").focus();
  await f.page.keyboard.press("Enter");
  assert.equal(await f.page.getByRole("option", { name: "human:b" }).getAttribute("aria-selected"), "false");
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
  await f.call("holdCoordinatorReadiness");
  await f.page.keyboard.press("Control+Enter");
  await f.page.getByRole("button", { name: "Forwarding…", exact: true }).waitFor();
  assert.deepEqual(await f.call("forwardFrames"), []);
  await f.call("releaseCoordinatorReadiness");
  const sent = await f.request("forward"); await f.call("resolve", sent.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  assert.equal((await f.call("events")).filter((e) => e.kind === "settled").length, 1);
  assert.deepEqual((await f.call("events")).find((e) => e.kind === "settled").recipientNames, [["human:b", "human:b"]]);
});
test("account switch prevents old success from closing B dialog", async (t) => {
  const f = await fixture(t); await f.settleSearch("human:b", "Account A recipient"); await f.select("Account A recipient"); await f.submit();
  const old = await f.request("forward"); await f.call("switchAccount", "B"); await f.call("reopen");
  await f.call("resolve", old.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.getByRole("heading", { name: "Forward message" }).waitFor();
  assert.deepEqual(await f.call("events"), []);
  const currentSearch = await f.request("search", 2);
  await f.call("resolve", currentSearch.id, [{ id: "human:c", name: "Account B recipient" }]);
  await f.select("Account B recipient"); await f.submit();
  const current = await f.request("forward", 2);
  await f.call("resolve", current.id, [{ conversationId: "human:c", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  const settled = (await f.call("events")).filter((e) => e.kind === "settled");
  assert.equal(settled.length, 1);
  assert.equal(settled[0].account, "B");
  assert.deepEqual(settled[0].recipientNames, [["human:c", "Account B recipient"]]);
  await assertPreReadyForwardRetired(t, "account");
});
test("logout prevents late error from being displayed", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const old = await f.request("forward"); await f.call("switchAccount", null);
  await f.call("reject", old.id, "private-old-token");
  assert.equal(await f.page.getByText("private-old-token").count(), 0);
  await assertPreReadyForwardRetired(t, "logout");
});
test("A B A reuse still rejects old settlement", async (t) => {
  const f = await fixture(t); await f.settleSearch("human:b", "Previous A recipient"); await f.select("Previous A recipient"); await f.submit();
  const old = await f.request("forward"); await f.call("switchAccount", "B"); await f.call("switchAccount", "A"); await f.call("reopen");
  await f.call("resolve", old.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.getByRole("heading", { name: "Forward message" }).waitFor();
  assert.deepEqual(await f.call("events"), []);
  const currentSearch = await f.request("search", 2);
  await f.call("resolve", currentSearch.id, [{ id: "human:b", name: "Current A recipient" }]);
  await f.select("Current A recipient"); await f.submit();
  const current = await f.request("forward", 2);
  assert.notEqual(current.args.clientNonce, old.args.clientNonce);
  await f.call("resolve", current.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  const settled = (await f.call("events")).filter((e) => e.kind === "settled");
  assert.equal(settled.length, 1);
  assert.equal(settled[0].account, "A");
  assert.equal(settled[0].generation, 3);
  assert.deepEqual(settled[0].recipientNames, [["human:b", "Current A recipient"]]);
  await assertPreReadyForwardRetired(t, "A-B-A");
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
  assert.equal(replacement.args.sourceEntryId, "message-2");
  assert.equal(await f.page.getByText("OLD_SECRET").count(), 0);
  await f.call("resolve", replacement.id, [{ id: "human:b", name: "Previous source recipient" }]);
  await f.select("Previous source recipient"); await f.submit();
  const oldForward = await f.request("forward");
  await f.call("replaceSource", "message-3");
  const currentSearch = await f.request("search", 3);
  assert.equal(currentSearch.args.sourceEntryId, "message-3");
  await f.call("resolve", currentSearch.id, [{ id: "human:c", name: "Current source recipient" }]);
  await f.select("Current source recipient"); await f.submit();
  const current = await f.request("forward", 2);
  assert.equal(current.args.sourceEntryId, "message-3");
  await f.call("resolve", oldForward.id, [{ conversationId: "human:b", status: "sent" }]);
  assert.deepEqual(await f.call("events"), []);
  await f.call("resolve", current.id, [{ conversationId: "human:c", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  const settled = (await f.call("events")).filter((e) => e.kind === "settled");
  assert.equal(settled.length, 1);
  assert.deepEqual(settled[0].recipientNames, [["human:c", "Current source recipient"]]);
  await assertPreReadyForwardRetired(t, "source");
});
test("unmount synchronously invalidates root generation", async (t) => {
  const f = await fixture(t); const old = await f.request("search");
  const scope = await f.call("unmount");
  assert.equal(scope.after, scope.before + 1);
  await f.call("resolve", old.id, [{ id: "old", name: "OLD_SECRET" }]);
  await assertPreReadyForwardRetired(t, "unmount");
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
  // Source-eligibility clamp: when forcedForwardInfo becomes true on the open
  // source message before snapshot lock, both privacy checkboxes clamp to false
  // and disable; when restored to false, the user can re-enable them.
  await f.call("setForcedForwardInfo", true);
  await f.page.waitForFunction(() => {
    const boxes = [...document.querySelectorAll('.fabushi-forward-dialog__privacy input[type="checkbox"]')];
    return boxes.length === 2 && boxes.every((box) => box.disabled && !box.checked);
  });
  assert.equal(await sender.isChecked(), false);
  assert.equal(await captions.isChecked(), false);
  assert.equal(await sender.isDisabled(), true);
  assert.equal(await captions.isDisabled(), true);
  await f.call("setForcedForwardInfo", false);
  await f.page.waitForFunction(() => {
    const boxes = [...document.querySelectorAll('.fabushi-forward-dialog__privacy input[type="checkbox"]')];
    return boxes.length === 2 && boxes.every((box) => !box.disabled);
  });
  assert.equal(await sender.isDisabled(), false);
  assert.equal(await captions.isDisabled(), false);
  await captions.check();
  assert.equal(await sender.isChecked(), true);
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
  assert.deepEqual((await f.call("events")).filter((e) => e.kind === "settled").map((e) => e.recipientNames), [
    [["human:b", "human:b"], ["human:c", "human:c"]],
    [["human:c", "human:c"]],
  ]);
});

test("hidden selected recipients keep submission names across renamed retries and consumer mutation", async (t) => {
  const f = await fixture(t); const initial = await f.request("search");
  await f.call("resolve", initial.id, [{ id: "human:b", name: "Beatrice old" }, { id: "human:c", name: "Camilo" }]);
  await f.select("Beatrice old"); await f.select("Camilo");
  await f.page.getByRole("searchbox").fill("updated name");
  const updated = await f.request("search", 2);
  await f.call("resolve", updated.id, [{ id: "human:b", name: "Beatrice at submission" }, { id: "human:d", name: "Unselected recipient" }]);
  await f.page.getByRole("option", { name: "Beatrice at submission", exact: true }).waitFor();
  await f.page.getByRole("searchbox").fill("only unselected");
  const hidden = await f.request("search", 3);
  await f.call("resolve", hidden.id, [{ id: "human:d", name: "Unselected recipient" }]);
  assert.equal(await f.page.getByRole("option").count(), 1);
  assert.equal(await f.page.getByText("2 selected", { exact: true }).count(), 1);
  await f.submit(); const first = await f.request("forward");
  assert.deepEqual(first.args.destinationConversationIds, ["human:b", "human:c"]);
  assert.deepEqual(Object.keys(first.args).sort(), [
    "clientNonce", "destinationConversationIds", "dropCaptions", "dropSenderNames", "sourceConversationId", "sourceEntryId",
  ]);
  await f.call("resolve", first.id, [{ conversationId: "human:b", status: "sent" }, { conversationId: "human:c", status: "failed", error: "retry" }]);
  await f.page.getByRole("alert").waitFor();
  const firstEvent = (await f.call("events")).find((e) => e.kind === "settled");
  assert.deepEqual(firstEvent.recipientNames, [["human:b", "Beatrice at submission"], ["human:c", "Camilo"]]);
  await f.call("mutateSettledRecipientName", "human:c", "Consumer replacement");
  await f.call("mutateSettledRecipientName", "foreign", "Consumer foreign recipient");
  await f.page.getByRole("searchbox").fill("renamed after submission");
  const renamed = await f.request("search", 4);
  await f.call("resolve", renamed.id, [
    { id: "human:b", name: "Beatrice renamed" },
    { id: "human:c", name: "Camilo renamed" },
    { id: "human:d", name: "New destination" },
  ]);
  assert.equal(await f.page.getByRole("option", { name: "New destination", exact: true }).locator('input[type="checkbox"]').isDisabled(), true);
  assert.equal(await f.page.getByRole("checkbox", { name: "Hide sender name", exact: true }).isDisabled(), true);
  assert.equal(await f.page.getByRole("checkbox", { name: "Hide media caption", exact: true }).isDisabled(), true);
  await f.submit(); const retry = await f.request("forward", 2);
  assert.deepEqual(retry.args, { ...first.args, destinationConversationIds: ["human:c"] });
  await f.call("resolve", retry.id, [{ conversationId: "human:c", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  const settled = (await f.call("events")).filter((e) => e.kind === "settled");
  assert.equal(settled.length, 2);
  assert.deepEqual(settled[1].recipientNames, [["human:c", "Camilo"]]);
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
  assert.equal(await f.page.getByRole("alert").textContent(), "fixture-forward: Forward outcome is unknown");
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
  const f = await fixture(t); await f.settleSearch("human:b", "Before close recipient"); await f.select("Before close recipient"); await f.submit();
  const first = await f.request("forward");
  const close = f.page.getByRole("button", { name: "Close", exact: true });
  assert.equal(await close.isEnabled(), true);
  await close.click();
  await f.page.getByRole("dialog", { name: "Forward message" }).waitFor({ state: "hidden" });
  assert.equal((await f.call("pending")).includes(first.id), true);
  await f.call("reopen");
  const search = await f.request("search", 2);
  await f.call("resolve", search.id, [{ id: "human:c", name: "After reopen recipient" }]);
  await f.select("After reopen recipient"); await f.submit();
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
  assert.deepEqual((await f.call("events")).find((e) => e.kind === "settled").recipientNames, [["human:c", "After reopen recipient"]]);
  await assertPreReadyForwardRetired(t, "close");
});

test("late searches preserve send failure and retry destinations even when selected rows are hidden", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const first = await f.request("forward");
  await f.page.getByRole("searchbox").fill("another recipient");
  const searching = await f.request("search", 2);
  await f.call("reject", first.id, "Forward service unavailable");
  await f.page.getByRole("alert").waitFor();
  await f.call("resolve", searching.id, [{ id: "human:c", name: "human:c" }]);
  assert.equal(await f.page.getByRole("alert").textContent(), "fixture-forward: Forward service unavailable");
  assert.equal(await f.page.getByText("1 selected", { exact: true }).count(), 1);
  assert.equal(await f.page.getByRole("option", { name: "human:c" }).locator('input[type="checkbox"]').isDisabled(), true);
  await f.page.getByRole("searchbox").fill("offline search");
  const offline = await f.request("search", 3);
  await f.call("reject", offline.id, "Search is offline");
  assert.equal(await f.page.getByRole("alert").textContent(), "fixture-forward: Forward service unavailable");
  await f.submit();
  const retry = await f.request("forward", 2);
  assert.deepEqual(retry.args, first.args);
  await f.call("resolve", retry.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
  assert.deepEqual((await f.call("events")).find((e) => e.kind === "settled").recipientNames, [["human:b", "human:b"]]);
});

test("typing comment transmits formatted comment with forward request and locks on submission", async (t) => {
  const f = await fixture(t);
  await f.settleSearch("human:b", "Comment recipient");
  await f.select("Comment recipient");
  const commentInput = f.page.getByPlaceholder("Add a comment…");
  assert.equal(await commentInput.isVisible(), true);
  await commentInput.fill("Check out this link!");
  await f.submit();
  const first = await f.request("forward");
  assert.deepEqual(first.args.comment, { text: "Check out this link!", entities: [] });
  assert.equal(await commentInput.isDisabled(), true);
  await f.call("resolve", first.id, [{ conversationId: "human:b", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
});

test("selecting topic recipient displays topic title and transmits destinationTopics in forward request", async (t) => {
  const f = await fixture(t);
  const search = await f.request("search");
  await f.call("resolve", search.id, [
    { id: "human:forum-group", name: "Engineering Forum", topicId: "topic-42", topicTitle: "Architecture" },
  ]);
  const topicLabel = "Engineering Forum › Architecture";
  await f.page.getByRole("option", { name: topicLabel, exact: true }).waitFor();
  await f.select(topicLabel);
  await f.submit();
  const first = await f.request("forward");
  assert.deepEqual(first.args.destinationConversationIds, ["human:forum-group"]);
  assert.deepEqual(first.args.destinationTopics, { "human:forum-group": "topic-42" });

  const frames = await f.call("forwardFrames");
  assert.equal(frames.length, 1);
  assert.deepEqual(frames[0].args.destinationTopics, { "human:forum-group": "topic-42" });

  await f.call("resolve", first.id, [{ conversationId: "human:forum-group", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
});

test("destroyed or closed topic recipient automatically clears selection without falling back to parent conversation", async (t) => {
  const f = await fixture(t);
  const search1 = await f.request("search");
  await f.call("resolve", search1.id, [
    { id: "topic:alpha", name: "Alpha Group", topicId: "t-100", topicTitle: "Alpha Topic" },
    { id: "human:normal", name: "Normal Peer" },
  ]);
  const alphaLabel = "Alpha Group › Alpha Topic";
  await f.page.getByRole("option", { name: alphaLabel, exact: true }).waitFor();
  await f.page.getByRole("option", { name: "Normal Peer", exact: true }).waitFor();

  await f.select(alphaLabel);
  await f.select("Normal Peer");
  assert.equal(await f.page.getByText("2 selected", { exact: false }).count(), 1);

  // Trigger search refresh with topicClosed
  await f.page.getByRole("searchbox").fill("alpha");
  const search2 = await f.request("search", 2);
  await f.call("resolve", search2.id, [
    { id: "topic:alpha", name: "Alpha Group", topicId: "t-100", topicTitle: "Alpha Topic", topicClosed: true },
    { id: "human:other", name: "Other Peer" },
  ]);

  // topic:alpha is closed, so it must be filtered out and removed from selected
  await f.page.waitForFunction(() => !document.querySelector('[data-forward-recipient-index="1"]'));
  assert.equal(await f.page.getByText("1 selected", { exact: false }).count(), 1);
  assert.equal(await f.page.getByRole("option", { name: alphaLabel }).count(), 0);

  // Submit forwards only to the remaining selected recipient without falling back to parent conversation
  await f.submit();
  const first = await f.request("forward");
  assert.deepEqual(first.args.destinationConversationIds, ["human:normal"]);
  assert.equal(first.args.destinationTopics, undefined);

  await f.call("resolve", first.id, [{ conversationId: "human:normal", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
});

test("topicDeleted recipient in refreshed search automatically clears selection", async (t) => {
  const f = await fixture(t);
  const search1 = await f.request("search");
  await f.call("resolve", search1.id, [
    { id: "topic:del", name: "Forum Beta", topicId: "t-200", topicTitle: "Beta Topic" },
  ]);
  const betaLabel = "Forum Beta › Beta Topic";
  await f.page.getByRole("option", { name: betaLabel, exact: true }).waitFor();
  await f.select(betaLabel);
  assert.equal(await f.page.getByText("1 selected", { exact: false }).count(), 1);

  // Trigger search refresh with topicDeleted
  await f.page.getByRole("searchbox").fill("beta");
  const search2 = await f.request("search", 2);
  await f.call("resolve", search2.id, [
    { id: "topic:del", name: "Forum Beta", topicId: "t-200", topicTitle: "Beta Topic", topicDeleted: true },
  ]);

  // Topic is deleted, so selection is cleared
  await f.page.waitForFunction(() => document.querySelectorAll('[role="option"]').length === 0);
  assert.equal(await f.page.getByText("No recipients selected").count(), 1);
  assert.equal(await f.page.getByRole("button", { name: "Forward", exact: true }).isDisabled(), true);
});

test("partial retry only retains destinationTopics for unconfirmed destinations", async (t) => {
  const f = await fixture(t);
  const search1 = await f.request("search");
  await f.call("resolve", search1.id, [
    { id: "topic:first", name: "Forum A", topicId: "t-1", topicTitle: "Topic 1" },
    { id: "topic:second", name: "Forum B", topicId: "t-2", topicTitle: "Topic 2" },
  ]);
  await f.select("Forum A › Topic 1");
  await f.select("Forum B › Topic 2");
  await f.submit();

  const first = await f.request("forward");
  assert.deepEqual(first.args.destinationTopics, {
    "topic:first": "t-1",
    "topic:second": "t-2",
  });

  // Fail only topic:first
  await f.call("resolve", first.id, [
    { conversationId: "topic:first", status: "failed", error: "Rate limit on topic:first" },
    { conversationId: "topic:second", status: "sent" },
  ]);

  await f.page.getByRole("alert").waitFor();
  assert.equal(await f.page.getByText("1 selected", { exact: false }).count(), 1);

  // Retry
  await f.submit();
  const retry = await f.request("forward", 2);
  assert.deepEqual(retry.args.destinationConversationIds, ["topic:first"]);
  // Only the failed destination remains in destinationTopics on retry
  assert.deepEqual(retry.args.destinationTopics, { "topic:first": "t-1" });

  await f.call("resolve", retry.id, [{ conversationId: "topic:first", status: "sent" }]);
  await f.page.waitForFunction(() => window.__forwardHarness.events().some((e) => e.kind === "closed"));
});

