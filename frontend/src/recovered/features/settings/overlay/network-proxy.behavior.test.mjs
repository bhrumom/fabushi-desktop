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
const artifacts = resolve(root, "artifacts/native-message-ui");
const fixtureUrl = "http://127.0.0.1:14329/network-proxy-settings-test.html";
const panelSelector = '[aria-label="Network proxy preferences"]';
const timeoutLabel = "Proxy rotation timeout seconds";
const unavailableText = "Proxy connections are unavailable. Your preferences can still be saved.";
const unconfirmedText = "Proxy connection availability has not been confirmed.";
const saveError = "Could not save proxy preferences. Your changes are ready to retry.";
const privateError = "PRIVATE_TRANSPORT_FAILURE_WITH_TOKEN";
const privatePublication = "PRIVATE_PUBLICATION_REASON";
let server;
let browser;
let serverLog;
let serverClosed = false;

before(async () => {
  await mkdir(artifacts, { recursive: true });
  serverLog = createWriteStream(resolve(artifacts, "network-proxy-vite.log"));
  server = spawn(resolve(desktop, "node_modules/.bin/vite"), ["--host", "127.0.0.1", "--port", "14329", "--strictPort"], {
    cwd: desktop, stdio: "pipe", env: { ...process.env, CI: "1" },
  });
  server.stdout.pipe(serverLog, { end: false });
  server.stderr.pipe(serverLog, { end: false });
  server.on("error", (error) => serverLog.write(`Vite process error: ${error.message}\n`));
  server.once("close", () => { serverClosed = true; });
  let ready = false;
  for (let attempt = 0; attempt < 100; attempt++) {
    if (server.exitCode != null) throw new Error("Settings fixture Vite server exited before readiness");
    try {
      const response = await fetch(fixtureUrl, { signal: AbortSignal.timeout(2_000) });
      if (response.ok) { ready = true; break; }
    } catch {}
    await new Promise((resolveWait) => setTimeout(resolveWait, 100));
  }
  if (!ready) throw new Error("Settings fixture Vite server did not start");
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

async function fixture(t, options = {}) {
  const page = await browser.newPage({ viewport: { width: 1280, height: 1000 } });
  page.setDefaultTimeout(5_000);
  const diagnostics = [];
  const name = `network-proxy-${t.name.replace(/[^a-z0-9]+/gi, "-")}`;
  page.on("pageerror", (error) => diagnostics.push({ kind: "pageerror", message: error.message }));
  page.on("console", (message) => { if (message.type() === "error") diagnostics.push({ kind: "console", message: message.text() }); });
  page.on("requestfailed", (request) => diagnostics.push({ kind: "requestfailed", url: request.url(), error: request.failure()?.errorText }));
  page.on("response", (response) => { if (response.status() >= 400) diagnostics.push({ kind: "http", url: response.url(), status: response.status() }); });
  const call = (method, ...args) => page.evaluate(([name, values]) => window.__proxySettingsHarness[name](...values), [method, args]);
  t.after(async () => {
    let rejections = [];
    try {
      rejections = await page.evaluate(() => window.__proxySettingsHarness?.rejections() ?? []);
      await page.screenshot({ path: resolve(artifacts, `${name}.png`), fullPage: true });
      await writeFile(resolve(artifacts, `${name}.browser.json`), JSON.stringify({ diagnostics, rejections }, null, 2));
    } finally {
      await page.close();
    }
    assert.deepEqual(rejections, [], "failed or retired saves must not create unhandled promise rejections");
    assert.deepEqual(diagnostics.filter((item) => item.kind === "pageerror"), [], "the real Settings surface must mount and settle without browser exceptions");
  });
  await page.goto(fixtureUrl + (options.failInitialProxyRead ? "?failInitialProxyRead=1" : ""));
  await page.waitForFunction(() => !!window.__proxySettingsHarness);
  const panel = page.locator(panelSelector);
  const timeout = page.getByLabel(timeoutLabel, { exact: true });
  const ready = async (expectedTimeout) => {
    await panel.waitFor({ state: "visible" });
    await timeout.waitFor({ state: "visible" });
    if (expectedTimeout !== undefined) await page.waitForFunction(([label, expected]) => {
      const input = document.querySelector(`input[aria-label="${label}"]`);
      return input?.value === expected;
    }, [timeoutLabel, String(expectedTimeout)]);
  };
  if (!options.failInitialProxyRead) await ready();
  const records = async (kind, bridge) => (await call("requests")).filter((entry) => entry.kind === kind && (bridge == null || entry.bridge === bridge));
  const request = async (kind = "proxy-mutate", count = 1, bridge) => {
    await page.waitForFunction(([kind, count, bridge]) => window.__proxySettingsHarness.requests()
      .filter((entry) => entry.kind === kind && (bridge == null || entry.bridge === bridge)).length >= count, [kind, count, bridge]);
    return (await records(kind, bridge)).at(-1);
  };
  const saveTimeout = async (seconds) => {
    await timeout.fill(String(seconds));
    await timeout.press("Tab");
  };
  const resolveWithTimeout = async (pending, seconds, publication = { available: false }) => {
    const canonical = await call("snapshot", pending.bridge);
    canonical.network.proxy.rotation.timeoutSeconds = seconds;
    await call("resolveMutation", pending.id, canonical, publication);
    return canonical;
  };
  const close = () => page.getByRole("button", { name: "Close", exact: true }).click();
  const proxyNotices = async () => (await call("notices")).filter((item) => item.event.operation === "settings-network-proxy");
  return { page, call, panel, timeout, ready, records, request, saveTimeout, resolveWithTimeout, close, proxyNotices };
}

test("PS-UI-01 canonical timeout normalizes 10 to 0 to the same returned 10", async (t) => {
  const f = await fixture(t);
  assert.equal(await f.timeout.inputValue(), "10");
  await f.saveTimeout(0);
  const pending = await f.request();
  assert.deepEqual(pending.args, { type: "setRotationTimeout", seconds: 0 });
  assert.equal(await f.timeout.inputValue(), "0", "the draft remains visible while the write is unresolved");
  await f.resolveWithTimeout(pending, 10);
  assert.equal(await f.timeout.inputValue(), "10", "success must apply the returned value even when the previous canonical value was also 10");
  assert.equal((await f.records("proxy-mutate")).length, 1);
  assert.equal((await f.call("snapshot", "A")).network.proxy.rotation.timeoutSeconds, 10);
  assert.equal(await f.panel.getAttribute("aria-busy"), "false");
});

for (const entry of [
  { name: "explicit unavailable", publication: { available: false, reason: privatePublication, credentials: { password: "PRIVATE_PUBLICATION_PASSWORD" } }, expected: unavailableText },
  { name: "absent", publication: null, omitted: true, expected: unconfirmedText },
  { name: "null", publication: null, expected: unconfirmedText },
  { name: "unknown", publication: { available: "false", reason: privatePublication }, expected: unconfirmedText },
  { name: "available true", publication: { available: true, connected: true, reason: privatePublication }, expected: unconfirmedText },
]) {
  test(`PS-UI-02 ${entry.name} publication preserves honest accessible availability`, async (t) => {
    const f = await fixture(t);
    await f.page.getByRole("status").filter({ hasText: unconfirmedText }).waitFor({ state: "visible" });
    await f.saveTimeout(22);
    const pending = await f.request();
    const canonical = await f.call("snapshot", "A");
    canonical.network.proxy.rotation.timeoutSeconds = 17;
    await f.call("resolveMutation", pending.id, canonical, entry.publication, entry.omitted === true);
    await f.page.getByRole("status").filter({ hasText: entry.expected }).waitFor({ state: "visible" });
    assert.equal(await f.timeout.inputValue(), "17", "committed presentation follows the returned snapshot, not the submitted draft");
    const body = await f.page.locator("body").innerText();
    assert.equal(body.includes(privatePublication), false);
    assert.equal(body.includes("PRIVATE_PUBLICATION_PASSWORD"), false);
    assert.equal(body.includes("telegramClientUnavailable"), false);
    const notices = await f.proxyNotices();
    assert.ok(notices.every((item) => !JSON.stringify(item.event).includes(privatePublication)));
    assert.equal((await f.records("proxy-mutate")).length, 1);
  });
}

test("PS-UI-03 pending disables all proxy controls and rapid duplicate save while Theme remains usable", async (t) => {
  const f = await fixture(t);
  const host = f.page.getByLabel("Proxy 1 host", { exact: true });
  await f.timeout.focus();
  await host.click();
  await f.call("paint");
  assert.equal((await f.records("proxy-mutate")).length, 0, "blurring an unchanged timeout must not seize the mutation slot from another control");
  await host.fill("draft.example");
  // Two genuine pointer clicks represent a rapid duplicate interaction. Do not
  // force a click through a disabled control or replace the mutation callback.
  await f.panel.getByRole("button", { name: "Save", exact: true }).dblclick();
  const pending = await f.request();
  assert.equal(pending.args.type, "replace");
  assert.equal(pending.args.proxy.host, "draft.example");
  assert.equal(await f.panel.getAttribute("aria-busy"), "true");
  const controls = f.panel.locator("input, button, select, textarea");
  assert.ok(await controls.count() >= 10, "the test must exercise the real proxy form");
  for (const control of await controls.all()) assert.equal(await control.isDisabled(), true);
  await f.call("paint");
  assert.equal((await f.records("proxy-mutate")).length, 1);
  const theme = f.page.getByRole("button", { name: "Theme", exact: true });
  assert.equal(await theme.isDisabled(), false);
  await theme.click();
  await f.page.getByRole("option", { name: "Dark", exact: true }).click();
  const themeRequest = await f.request("theme-set");
  assert.deepEqual(themeRequest.args, { preference: "dark" });
  assert.equal(await theme.textContent(), "Dark");
  assert.equal((await f.records("proxy-mutate")).length, 1);
  assert.equal(await f.page.getByLabel("Proxy 1 host", { exact: true }).inputValue(), "draft.example");
  const canonical = await f.call("snapshot", "A");
  canonical.network.proxy.list[0] = { ...pending.args.proxy };
  await f.call("resolveMutation", pending.id, canonical, { available: false });
  assert.equal(await f.panel.getAttribute("aria-busy"), "false");
  assert.equal(await f.page.getByLabel("Proxy 1 host", { exact: true }).isDisabled(), false);
});

test("PS-UI-04 endpoint failure retains the exact draft and retries through the real mutation boundary", async (t) => {
  const f = await fixture(t);
  const host = f.page.getByLabel("Proxy 1 host", { exact: true });
  const user = f.page.getByLabel("Proxy 1 user", { exact: true });
  const password = f.page.getByLabel("Proxy 1 password or secret", { exact: true });
  await host.fill("edited.example");
  await user.fill("edited-user");
  await password.fill("EDITED_PASSWORD_FIXTURE");
  await f.panel.getByRole("button", { name: "Save", exact: true }).click();
  const first = await f.request();
  await f.call("rejectMutation", first.id, privateError);
  await f.page.getByRole("alert").filter({ hasText: saveError }).waitFor({ state: "visible" });
  assert.equal(await host.inputValue(), "edited.example");
  assert.equal(await user.inputValue(), "edited-user");
  assert.equal(await password.inputValue(), "EDITED_PASSWORD_FIXTURE");
  assert.equal((await f.call("snapshot", "A")).network.proxy.list[0].host, "alpha.example");
  assert.equal((await f.page.locator("body").innerText()).includes(privateError), false);
  assert.equal(await host.isDisabled(), false);
  await f.panel.getByRole("button", { name: "Save", exact: true }).click();
  const retry = await f.request("proxy-mutate", 2);
  assert.deepEqual(retry.args, first.args, "the retry must retain both the original replace target and the unsaved endpoint draft");
  const canonical = await f.call("snapshot", "A");
  canonical.network.proxy.list[0] = { ...retry.args.proxy };
  await f.call("resolveMutation", retry.id, canonical, { available: false });
  assert.equal(await host.inputValue(), "edited.example");
  assert.equal((await f.call("snapshot", "A")).network.proxy.list[0].host, "edited.example");
  assert.equal((await f.records("proxy-mutate")).length, 2);
  const errors = (await f.proxyNotices()).filter((item) => item.event.kind === "error");
  assert.equal(errors.length, 1);
  assert.equal(errors[0].event.message, saveError);
});

test("PS-UI-04 timeout failure retains its draft until a successful normalized retry", async (t) => {
  const f = await fixture(t);
  await f.saveTimeout(0);
  const first = await f.request();
  await f.call("rejectMutation", first.id, privateError);
  await f.page.getByRole("alert").filter({ hasText: saveError }).waitFor({ state: "visible" });
  assert.equal(await f.timeout.inputValue(), "0");
  assert.equal((await f.call("snapshot", "A")).network.proxy.rotation.timeoutSeconds, 10);
  await f.timeout.focus();
  await f.timeout.press("Tab");
  const retry = await f.request("proxy-mutate", 2);
  assert.deepEqual(retry.args, first.args);
  await f.resolveWithTimeout(retry, 10);
  assert.equal(await f.timeout.inputValue(), "10");
  assert.equal((await f.records("proxy-mutate")).length, 2);
});

for (const retirement of ["close", "unmount"]) {
  for (const outcome of ["success", "error"]) {
    test(`PS-UI-04 ${retirement} retires a late proxy ${outcome} without changing the next surface`, async (t) => {
      const f = await fixture(t);
      await f.saveTimeout(23);
      const pending = await f.request();
      if (retirement === "close") await f.close();
      else await f.call("unmount");
      await f.panel.waitFor({ state: "detached" });
      if (outcome === "success") await f.resolveWithTimeout(pending, 61);
      else await f.call("rejectMutation", pending.id, privateError);
      assert.deepEqual(await f.proxyNotices(), [], "a retired save must publish neither success nor error into the next lifetime");
      await f.call(retirement === "close" ? "reopen" : "mount");
      await f.ready();
      assert.equal(await f.timeout.inputValue(), outcome === "success" ? "61" : "10");
      await f.page.getByRole("status").filter({ hasText: unconfirmedText }).waitFor({ state: "visible" });
      assert.equal((await f.records("proxy-read", "A")).length, 2);
      assert.equal((await f.records("proxy-mutate", "A")).length, 1);
      assert.deepEqual(await f.proxyNotices(), []);
      assert.equal((await f.page.locator("body").innerText()).includes(privateError), false);
    });
  }
}

for (const outcome of ["success", "error"]) {
  test(`PS-UI-04 bridge A B A isolates a retired proxy ${outcome} and independent B write`, async (t) => {
    const f = await fixture(t);
    await f.saveTimeout(23);
    const oldA = await f.request("proxy-mutate", 1, "A");
    await f.call("switchBridge", "B");
    await f.ready("30");
    assert.equal(await f.timeout.inputValue(), "30");
    await f.saveTimeout(41);
    const oldB = await f.request("proxy-mutate", 1, "B");
    await f.call("switchBridge", "A");
    await f.call("paint");
    assert.equal((await f.records("proxy-read", "A")).length, 1, "A's new read must wait for its dispatched write");
    assert.equal(await f.panel.count(), 0);
    if (outcome === "success") await f.resolveWithTimeout(oldA, 64);
    else await f.call("rejectMutation", oldA.id, privateError);
    await f.ready();
    const expectedA = outcome === "success" ? "64" : "10";
    assert.equal(await f.timeout.inputValue(), expectedA);
    await f.resolveWithTimeout(oldB, 41);
    assert.equal(await f.timeout.inputValue(), expectedA, "late B cannot overwrite reopened A");
    assert.equal((await f.call("snapshot", "B")).network.proxy.rotation.timeoutSeconds, 41);
    assert.equal((await f.records("proxy-mutate")).length, 2);
    assert.equal((await f.records("proxy-read", "A")).length, 2);
    assert.deepEqual(await f.proxyNotices(), []);
    await f.page.getByRole("status").filter({ hasText: unconfirmedText }).waitFor({ state: "visible" });
  });
}

test("PS-UI-03 reopen waits for an accepted pending write before its authoritative get", async (t) => {
  const f = await fixture(t);
  for (const [index, retirement] of ["close", "unmount"].entries()) {
    await f.saveTimeout(23);
    const pending = await f.request("proxy-mutate", index + 1);
    if (retirement === "close") { await f.close(); await f.call("reopen"); }
    else { await f.call("unmount"); await f.call("mount"); }
    await f.call("paint");
    assert.equal((await f.records("proxy-read")).length, index + 1);
    assert.equal(await f.panel.count(), 0, "a reopened or remounted lifetime cannot display a stale pre-write snapshot");
    assert.equal((await f.records("proxy-mutate")).length, index + 1);
    await f.resolveWithTimeout(pending, 70 + index);
    await f.ready();
    assert.equal(await f.timeout.inputValue(), String(70 + index));
    assert.equal((await f.records("proxy-read")).length, index + 2);
    assert.equal((await f.records("proxy-mutate")).length, index + 1, "reopen must not automatically repeat an uncertain write");
    assert.deepEqual(await f.proxyNotices(), []);
    await f.page.getByRole("status").filter({ hasText: unconfirmedText }).waitFor({ state: "visible" });
  }
});

test("PS-UI-03 rejected pending write releases the reopened read and permits a new explicit save", async (t) => {
  const f = await fixture(t);
  await f.saveTimeout(23);
  const pending = await f.request();
  await f.close();
  await f.call("reopen");
  await f.call("paint");
  assert.equal((await f.records("proxy-read")).length, 1);
  await f.call("rejectMutation", pending.id, privateError);
  await f.ready();
  assert.equal(await f.timeout.inputValue(), "10");
  assert.equal((await f.records("proxy-read")).length, 2);
  assert.deepEqual(await f.proxyNotices(), []);
  assert.equal((await f.records("proxy-mutate")).length, 1);
  await f.saveTimeout(24);
  const retry = await f.request("proxy-mutate", 2);
  await f.resolveWithTimeout(retry, 24);
  assert.equal(await f.timeout.inputValue(), "24");
  assert.equal((await f.records("proxy-mutate")).length, 2);
});

test("PS-UI-04 initial proxy read failure shows a fixed fallback and Retry rereads without mutation", async (t) => {
  const f = await fixture(t, { failInitialProxyRead: true });
  const loadError = "Could not load proxy preferences. Please try again.";
  await f.page.locator(".sand-settings-panel").getByRole("alert").filter({ hasText: loadError }).waitFor({ state: "visible" });
  assert.equal(await f.panel.count(), 0, "a failed authoritative read must not expose default editable preferences");
  assert.equal((await f.page.locator("body").innerText()).includes("PRIVATE_PROXY_READ_FAILURE_WITH_SECRET"), false);
  assert.equal((await f.records("proxy-read")).length, 1);
  assert.equal((await f.records("proxy-mutate")).length, 0);
  const loadNotices = (await f.call("notices")).filter((item) => item.event.operation === "settings-load");
  assert.equal(loadNotices.length, 1);
  assert.equal(loadNotices[0].event.message, loadError);
  await f.page.getByRole("button", { name: "Retry", exact: true }).click();
  await f.ready("10");
  assert.equal((await f.records("proxy-read")).length, 2);
  assert.equal((await f.records("proxy-mutate")).length, 0);
  assert.equal(await f.page.getByLabel("Proxy 1 host", { exact: true }).inputValue(), "alpha.example");
});
