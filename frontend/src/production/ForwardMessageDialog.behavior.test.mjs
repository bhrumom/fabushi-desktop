import assert from "node:assert/strict";
import test, { after, before } from "node:test";
import { createRequire } from "node:module";
import { spawn } from "node:child_process";
import { resolve } from "node:path";
const root = process.cwd();
const desktop = resolve(root, "desktop");
const require = createRequire(resolve(desktop, "package.json"));
const { chromium } = require("playwright");
let server;
let browser;
before(async () => {
  server = spawn(resolve(desktop, "node_modules/.bin/vite"), ["--host", "127.0.0.1", "--port", "14319", "--strictPort"], {
    cwd: desktop, stdio: "pipe", env: { ...process.env, CI: "1" },
  });
  let ready = false;
  for (let i = 0; i < 100; i++) {
    if (server.exitCode != null) throw new Error("fixture Vite server exited before readiness");
    try { const res = await fetch("http://127.0.0.1:14319/forward-dialog-test.html"); if (res.ok) { ready = true; break; } }
    catch {}
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (!ready) throw new Error("fixture server did not start");
  browser = await chromium.launch({ headless: true });
});
after(async () => { await browser?.close(); server?.kill(); });
async function fixture(t) {
  const page = await browser.newPage();
  t.after(() => page.close());
  await page.goto("http://127.0.0.1:14319/forward-dialog-test.html");
  await page.waitForFunction(() => !!window.__forwardHarness && document.querySelector('input[type="search"]'));
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
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
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
  const f = await fixture(t); await f.settleSearch(); await f.select();
  await f.submit(); const first = await f.request("forward");
  await f.call("resolve", first.id, [{ conversationId: "human:b", status: "failed", error: "retry" }]);
  await f.page.getByRole("alert").waitFor(); await f.submit(); const second = await f.request("forward", 2);
  assert.equal(second.args.clientNonce, first.args.clientNonce);
  assert.deepEqual(second.args.destinationConversationIds, ["human:b"]);
});
test("query change does not invalidate valid submit", async (t) => {
  const f = await fixture(t); await f.settleSearch(); await f.select(); await f.submit();
  const sent = await f.request("forward");
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
