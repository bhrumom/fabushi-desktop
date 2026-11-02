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
  const submit = async () => page.getByRole("button", { name: "Forward", exact: true }).click();
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
  assert.equal(replacement.args.sourceEntryId, "message-2");
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
  await f.submit(); const first = await f.request("forward");
  await f.call("resolve", first.id, [{ conversationId: "human:b", status: "sent" }, { conversationId: "human:c", status: "failed", error: "retry" }]);
  await f.page.getByRole("alert").waitFor(); await f.submit(); const second = await f.request("forward", 2);
  assert.equal(second.args.clientNonce, first.args.clientNonce);
  assert.deepEqual(second.args.destinationConversationIds, ["human:c"]);
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

