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
const artifacts = resolve(root, "artifacts/call-media");
const origin = "http://127.0.0.1:14320";
let server;
let serverLog;
let browser;
let serverClosed = false;

before(async () => {
  await mkdir(artifacts, { recursive: true });
  serverLog = createWriteStream(resolve(artifacts, "call-media-vite.log"));
  let spawnError;
  server = spawn(resolve(desktop, "node_modules/.bin/vite"), ["--host", "127.0.0.1", "--port", "14320", "--strictPort"], {
    cwd: desktop, stdio: "pipe", env: { ...process.env, CI: "1" },
  });
  server.stdout.pipe(serverLog, { end: false });
  server.stderr.pipe(serverLog, { end: false });
  server.once("error", (error) => { spawnError = error; });
  server.once("close", () => { serverClosed = true; });
  const deadline = Date.now() + 30_000;
  let ready = false;
  while (Date.now() < deadline) {
    if (spawnError != null) throw spawnError;
    if (serverClosed || server.exitCode != null) throw new Error("Call-media Vite server exited before readiness");
    try {
      const response = await fetch(`${origin}/human-call-media-test.html`, { signal: AbortSignal.timeout(1_000) });
      if (response.ok) { ready = true; break; }
    } catch {}
    await new Promise((done) => setTimeout(done, 100));
  }
  if (!ready) throw new Error("Call-media Vite readiness deadline expired");
  browser = await chromium.launch({ headless: true });
}, { timeout: 60_000 });

after(async () => {
  let browserError;
  try { await browser?.close(); } catch (error) { browserError = error; }
  try {
    if (server != null && !serverClosed) {
      await new Promise((done, reject) => {
        const force = setTimeout(() => server.kill("SIGKILL"), 5_000);
        const deadline = setTimeout(() => { clearTimeout(force); reject(new Error("Owned Vite process did not close")); }, 10_000);
        server.once("close", () => { clearTimeout(force); clearTimeout(deadline); done(); });
        if (server.exitCode == null && server.signalCode == null) server.kill("SIGTERM");
      });
    }
  } finally {
    await new Promise((done) => serverLog == null ? done() : serverLog.end(done));
  }
  if (browserError != null) throw browserError;
}, { timeout: 20_000 });

async function fixture(t, suffix = "") {
  const page = await browser.newPage();
  page.setDefaultTimeout(10_000);
  const diagnostics = [];
  const name = `${t.name}-${suffix}`.replace(/[^a-z0-9]+/gi, "-");
  page.on("pageerror", (error) => diagnostics.push({ kind: "pageerror", message: error.message }));
  page.on("console", (message) => { if (message.type() === "error") diagnostics.push({ kind: "console", message: message.text() }); });
  page.on("requestfailed", (request) => diagnostics.push({ kind: "requestfailed", url: request.url(), error: request.failure()?.errorText }));
  page.on("response", (response) => { if (response.status() >= 400) diagnostics.push({ kind: "http", url: response.url(), status: response.status() }); });
  const call = (method, ...args) => page.evaluate(([m, values]) => window.__callMediaHarness[m](...values), [method, args]);
  const snapshot = () => call("snapshot");
  t.after(async () => {
    let lastSnapshot;
    try {
      lastSnapshot = await snapshot();
      await call("dispose");
    } catch (error) {
      diagnostics.push({ kind: "cleanup", message: String(error) });
    }
    try {
      await writeFile(resolve(artifacts, `${name}.browser.json`), JSON.stringify({ diagnostics, snapshot: lastSnapshot }, null, 2));
    } finally {
      await page.close();
    }
    assert.deepEqual(diagnostics.filter((item) => item.kind === "pageerror" || item.kind === "cleanup"), [], "No unhandled browser continuation or fixture cleanup error");
  });
  await page.goto(`${origin}/human-call-media-test.html`);
  try {
    await page.getByRole("button", { name: "Accept voice call", exact: true }).waitFor();
    await page.waitForFunction(() => !!window.__callMediaHarness);
  } catch (error) {
    await page.screenshot({ path: resolve(artifacts, `${name}.bootstrap.png`), fullPage: true });
    throw new Error(`Call-media component did not mount: ${JSON.stringify(diagnostics)}`, { cause: error });
  }
  const request = async (kind, { method, after = 0, pending = true } = {}) => {
    await page.waitForFunction(({ kind, method, after, pending }) => {
      const value = window.__callMediaHarness.snapshot();
      return value.requests.slice(after).some((item) => item.kind === kind && (method == null || item.method === method) && (!pending || value.pending.includes(item.id)));
    }, { kind, method, after, pending });
    const value = await snapshot();
    return value.requests.slice(after).find((item) => item.kind === kind && (method == null || item.method === method) && (!pending || value.pending.includes(item.id)));
  };
  const accept = async (video, label, double = false) => {
    const before = (await snapshot()).requests.length;
    const button = page.getByRole("button", { name: video ? "Accept video call" : "Accept voice call", exact: true });
    if (double) await button.evaluate((node) => { node.click(); node.click(); });
    else await button.click();
    const capture = await request("user", { after: before });
    await call("resolveCapture", capture.id, { label, audio: 1, video: video ? 1 : 0 });
    await page.waitForFunction(() => {
      const value = window.__callMediaHarness.snapshot();
      return value.localPreview != null && value.peers.some((peer) => !peer.closed);
    });
    return snapshot();
  };
  const share = async (label) => {
    const before = (await snapshot()).requests.length;
    await page.getByRole("button", { name: "Share screen", exact: true }).click();
    const capture = await request("display", { after: before });
    await call("resolveCapture", capture.id, { label, video: 1 });
    await page.getByRole("button", { name: "Stop sharing", exact: true }).waitFor();
    return snapshot();
  };
  const start = async (video, label) => {
    for (const current of (await snapshot()).calls) {
      if (current.state !== "ended" && current.state !== "failed") await call("remoteTerminal", current.id);
    }
    await call("poll");
    const before = (await snapshot()).requests.length;
    await page.getByRole("button", { name: video ? "Start video call" : "Start voice call", exact: true }).click();
    const capture = await request("user", { after: before });
    await call("resolveCapture", capture.id, { label, audio: 1, video: video ? 1 : 0 });
    const ringing = await snapshot();
    const outgoing = ringing.calls.find((current) => current.creatorId === "local-human" && current.state === "ringing");
    assert.ok(outgoing, "Outgoing create/ring/capture reached the real component");
    assert.equal(ringing.requests.slice(before).filter((item) => item.method === "createCallSession").length, 1);
    assert.equal(ringing.requests.slice(before).filter((item) => item.method === "transitionCallSession" && item.args.action === "ring").length, 1);
    await call("remoteNegotiating", outgoing.id);
    await call("poll");
    await page.waitForFunction(() => window.__callMediaHarness.snapshot().peers.some((peer) => !peer.closed));
    const connected = await snapshot();
    assert.equal(connected.requests.slice(before).filter((item) => item.method === "sendCallSignal" && item.args.kind === "offer").length, 1);
    return connected;
  };
  const freshIncoming = async (scope = "scope-a") => {
    await call("installIncoming", scope);
    await call("poll");
    await page.getByRole("button", { name: "Accept voice call", exact: true }).waitFor();
  };
  return { page, call, snapshot, request, accept, share, start, freshIncoming };
}

function media(value, label) {
  const selected = value.tracks.filter((track) => track.label.startsWith(`${label}:`));
  assert.ok(selected.length > 0, `Native tracks exist for ${label}`);
  return selected;
}

function track(value, label, kind = "video") {
  const selected = value.tracks.find((item) => item.label === `${label}:${kind}:0`);
  assert.ok(selected, `Native ${kind} track exists for ${label}`);
  return selected;
}

function live(value, label) {
  for (const item of media(value, label)) {
    assert.equal(item.readyState, "live", `${item.label} remains live`);
    assert.equal(item.stopCalls, 0, `${item.label} was not released`);
  }
}

function ended(value, label, exactlyOnce = true) {
  for (const item of media(value, label)) {
    assert.equal(item.readyState, "ended", `${item.label} is physically ended`);
    if (exactlyOnce) assert.equal(item.stopCalls, 1, `${item.label} is released exactly once`);
  }
}

function activePeer(value) {
  const selected = value.peers.filter((peer) => !peer.closed);
  assert.equal(selected.length, 1, "Exactly one peer is active");
  return selected[0];
}

function coordinatorAfter(value, offset) {
  return value.requests.slice(offset).filter((item) => item.kind === "coordinator");
}

function attachmentAttemptsAfter(value, offset) {
  return value.peerEvents.slice(offset).filter((item) => ["construct", "addTrack", "replace-dispatch"].includes(item.kind));
}

function mediaUpdates(value) {
  return value.requests.filter((item) => item.kind === "coordinator" && item.method === "updateCallMedia");
}

test("video start accept and camera mute rerenders retain the active media owner", { timeout: 90_000 }, async (t) => {
  for (const entry of ["accept", "start"]) {
    const f = await fixture(t, entry);
    const initial = await f[entry](true, "local");
    const peer = activePeer(initial);
    const camera = track(initial, "local");
    const microphone = track(initial, "local", "audio");
    live(initial, "local");
    assert.equal(peer.senders.length, 2);
    await f.call("rerender");
    await f.page.getByRole("button", { name: "Camera off", exact: true }).click();
    await f.call("flush");
    let value = await f.snapshot();
    live(value, "local");
    assert.equal(track(value, "local").enabled, false);
    assert.equal(activePeer(value).id, peer.id);
    await f.page.getByRole("button", { name: "Mute", exact: true }).click();
    await f.call("flush");
    value = await f.snapshot();
    assert.equal(track(value, "local", "audio").enabled, false);
    await f.page.getByRole("button", { name: "Unmute", exact: true }).click();
    await f.page.getByRole("button", { name: "Camera on", exact: true }).click();
    await f.call("flush");
    value = await f.snapshot();
    live(value, "local");
    assert.equal(track(value, "local").enabled, true);
    assert.equal(track(value, "local", "audio").enabled, true);
    assert.equal(activePeer(value).id, peer.id);
    assert.deepEqual(activePeer(value).senders.map((sender) => sender.trackId).sort(), [camera.id, microphone.id].sort());
    assert.equal(value.requests.filter((request) => request.kind === "user").length, 1);
    assert.equal(value.requests.filter((request) => request.method === "getCallIceServers").length, 1);
    assert.equal(value.pollingOwners, 1);
    assert.ok(value.localPreview.trackIds.includes(camera.id));
    await f.page.getByRole("button", { name: "End call", exact: true }).waitFor();
  }
});

test("voice camera acquisition serializes overlap and preserves its original microphone", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  const initial = await f.accept(false, "voice", true);
  assert.equal(initial.requests.filter((request) => request.kind === "user").length, 1);
  const peer = activePeer(initial);
  const before = initial.requests.length;
  await f.call("hold", "preferences");
  await f.page.getByRole("button", { name: "Camera on", exact: true }).evaluate((button) => { button.click(); button.click(); });
  const preferences = await f.request("preferences", { after: before });
  await f.call("resolve", preferences.id, { microphoneId: null, cameraId: "removed-camera" });
  const capture = await f.request("user", { after: before });
  let value = await f.snapshot();
  assert.equal(value.requests.slice(before).filter((request) => request.kind === "user").length, 1);
  assert.deepEqual(capture.args, { audio: false, video: { deviceId: { exact: "removed-camera" } } });
  const fallbackOffset = value.requests.length;
  await f.call("reject", capture.id, "Fixture selected camera is absent", "OverconstrainedError");
  const fallback = await f.request("user", { after: fallbackOffset });
  assert.deepEqual(fallback.args, { audio: false, video: true });
  await f.call("resolveCapture", fallback.id, { label: "camera", video: 1 });
  await f.page.getByRole("button", { name: "Camera off", exact: true }).waitFor();
  value = await f.snapshot();
  live(value, "voice");
  live(value, "camera");
  assert.equal(activePeer(value).id, peer.id);
  assert.equal(activePeer(value).senders.length, 2);
  assert.equal(activePeer(value).senders.filter((sender) => sender.trackId === track(value, "camera").id).length, 1);
  await f.page.getByRole("button", { name: "Camera off", exact: true }).click();
  await f.page.getByRole("button", { name: "Camera on", exact: true }).click();
  await f.call("flush");
  value = await f.snapshot();
  live(value, "voice");
  live(value, "camera");
  assert.equal(activePeer(value).id, peer.id);
  assert.equal(value.requests.filter((request) => request.kind === "user").length, 3,
    "Voice capture plus the selected-camera request and its one recoverable fallback");
});

test("share stop and reshare release display tracks restore camera and ignore an old ended handler", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  const initial = await f.accept(true, "local");
  const camera = track(initial, "local");
  const originalSender = activePeer(initial).senders.find((sender) => sender.trackId === camera.id);
  let value = await f.share("display-one");
  const firstDisplay = track(value, "display-one");
  assert.equal(activePeer(value).senders.find((sender) => sender.id === originalSender.id).trackId, firstDisplay.id);
  await f.call("saveEnded", firstDisplay.id);
  await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
  await f.page.getByRole("button", { name: "Share screen", exact: true }).waitFor();
  await f.call("flush");
  value = await f.snapshot();
  ended(value, "display-one");
  live(value, "local");
  assert.equal(activePeer(value).senders.find((sender) => sender.id === originalSender.id).trackId, camera.id);
  assert.equal(mediaUpdates(value).at(-1).args.mediaCapabilities.screenShare, false);
  value = await f.share("display-two");
  const secondDisplay = track(value, "display-two");
  const before = value.requests.length;
  await f.call("fireSavedEnded", firstDisplay.id);
  value = await f.snapshot();
  live(value, "display-two");
  live(value, "local");
  assert.equal(activePeer(value).senders.length, 2);
  assert.equal(activePeer(value).senders.find((sender) => sender.id === originalSender.id).trackId, secondDisplay.id);
  assert.deepEqual(coordinatorAfter(value, before), [], "Old ended callback cannot publish into a newer share");
  await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
  await f.call("flush");
  value = await f.snapshot();
  ended(value, "display-one");
  ended(value, "display-two");
  assert.equal(activePeer(value).senders.find((sender) => sender.id === originalSender.id).trackId, camera.id);
  const attachOffset = value.requests.length;
  await f.call("hold", "replace");
  await f.page.getByRole("button", { name: "Share screen", exact: true }).click();
  const heldCapture = await f.request("display", { after: attachOffset });
  await f.call("resolveCapture", heldCapture.id, { label: "pending-display", video: 1 });
  const heldReplace = await f.request("replace", { after: attachOffset });
  await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
  await f.call("flush");
  value = await f.snapshot();
  ended(value, "pending-display");
  live(value, "local");
  await f.call("resolve", heldReplace.id);
  await f.call("flush");
  value = await f.snapshot();
  ended(value, "pending-display");
  assert.equal(activePeer(value).senders.find((sender) => sender.id === originalSender.id).trackId, camera.id);
  assert.equal(value.requests.slice(attachOffset).filter((request) => request.method === "updateCallMedia" && request.args.mediaCapabilities.screenShare === true).length, 0,
    "Cancelled attachment cannot publish a late sharing=true after Stop");
  assert.equal(mediaUpdates(value).at(-1).args.mediaCapabilities.screenShare, false);
});

test("voice sharing clears its sender on device ended and reuses that sender for the next capture", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  await f.accept(false, "voice");
  let value = await f.share("display-one");
  const display = track(value, "display-one");
  const sender = activePeer(value).senders.find((item) => item.trackId === display.id);
  assert.ok(sender);
  await f.call("endTrack", display.id);
  await f.page.getByRole("button", { name: "Share screen", exact: true }).waitFor();
  value = await f.snapshot();
  ended(value, "display-one", false);
  live(value, "voice");
  assert.equal(activePeer(value).senders.find((item) => item.id === sender.id).trackId, null);
  assert.equal(mediaUpdates(value).at(-1).args.mediaCapabilities.screenShare, false);
  value = await f.share("display-two");
  assert.equal(activePeer(value).senders.length, 2, "A null video sender is reused instead of accumulating senders");
  assert.equal(activePeer(value).senders.find((item) => item.id === sender.id).trackId, track(value, "display-two").id);
  await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
  await f.call("flush");
  value = await f.snapshot();
  ended(value, "display-two");
  live(value, "voice");
  assert.equal(activePeer(value).senders.find((item) => item.id === sender.id).trackId, null);
});

test("display denial missing video and replacement refusal release capture and permit retry", { timeout: 90_000 }, async (t) => {
  for (const fault of ["denied", "missing-video", "replace"]) {
    const f = await fixture(t, fault);
    const initial = await f.accept(true, "local");
    const camera = track(initial, "local");
    const peer = activePeer(initial);
    const before = initial.requests.length;
    if (fault === "replace") await f.call("hold", "replace");
    await f.page.getByRole("button", { name: "Share screen", exact: true }).click();
    const capture = await f.request("display", { after: before });
    if (fault === "denied") {
      await f.call("reject", capture.id, "Fixture capture denied", "NotAllowedError");
    } else {
      await f.call("resolveCapture", capture.id, { label: "failed-display", audio: fault === "missing-video" ? 1 : 0, video: fault === "missing-video" ? 0 : 1 });
      if (fault === "replace") {
        const replacement = await f.request("replace", { after: before });
        await f.call("reject", replacement.id, "Fixture replacement refused");
      }
    }
    await f.page.getByRole("alert").waitFor();
    let value = await f.snapshot();
    if (fault !== "denied") ended(value, "failed-display");
    live(value, "local");
    assert.equal(activePeer(value).id, peer.id);
    assert.ok(activePeer(value).senders.some((sender) => sender.trackId === camera.id));
    assert.equal(value.requests.slice(before).filter((request) => request.method === "updateCallMedia" && request.args.mediaCapabilities.screenShare === true).length, 0);
    value = await f.share("retry-display");
    live(value, "retry-display");
    live(value, "local");
    await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
    await f.call("flush");
    ended(await f.snapshot(), "retry-display");
  }
});

test("metadata refusal cannot retain display capture on either share or stop", { timeout: 90_000 }, async (t) => {
  const f = await fixture(t);
  const initial = await f.accept(true, "local");
  const camera = track(initial, "local");
  const sender = activePeer(initial).senders.find((item) => item.trackId === camera.id);
  let before = initial.requests.length;
  await f.call("hold", "coordinator", "updateCallMedia");
  await f.page.getByRole("button", { name: "Share screen", exact: true }).click();
  const capture = await f.request("display", { after: before });
  await f.call("resolveCapture", capture.id, { label: "failed-metadata-display", video: 1 });
  const metadata = await f.request("coordinator", { method: "updateCallMedia", after: before });
  await f.call("reject", metadata.id, "Fixture media metadata refused");
  await f.page.getByRole("alert").waitFor();
  let value = await f.snapshot();
  ended(value, "failed-metadata-display");
  live(value, "local");
  assert.equal(activePeer(value).senders.find((item) => item.id === sender.id).trackId, camera.id);
  value = await f.share("retry-display");
  before = value.requests.length;
  await f.call("hold", "coordinator", "updateCallMedia");
  await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
  const stopping = await f.request("coordinator", { method: "updateCallMedia", after: before });
  value = await f.snapshot();
  ended(value, "retry-display");
  live(value, "local");
  assert.equal(activePeer(value).senders.find((item) => item.id === sender.id).trackId, camera.id);
  await f.call("reject", stopping.id, "Fixture stop metadata refused");
  await f.page.getByRole("button", { name: "Share screen", exact: true }).waitFor();
  value = await f.snapshot();
  ended(value, "retry-display");
  live(value, "local");
  value = await f.share("second-retry-display");
  live(value, "second-retry-display");
});

test("End releases local and display tracks before a held Host hangup response", { timeout: 60_000 }, async (t) => {
  const f = await fixture(t);
  await f.accept(true, "local");
  const initial = await f.share("display");
  const before = initial.requests.length;
  await f.call("hold", "coordinator", "transitionCallSession");
  await f.page.getByRole("button", { name: "End call", exact: true }).click();
  const ending = await f.request("coordinator", { method: "transitionCallSession", after: before });
  assert.equal(ending.args.action, "hangup");
  let value = await f.snapshot();
  ended(value, "display");
  ended(value, "local");
  assert.equal(value.peers.filter((peer) => !peer.closed).length, 0);
  assert.equal(value.localPreview, null);
  assert.equal(value.remotePreview, null);
  const afterRelease = value.requests.length;
  await f.call("resolve", ending.id);
  value = await f.snapshot();
  assert.deepEqual(coordinatorAfter(value, afterRelease), []);
  await f.freshIncoming();
  value = await f.accept(true, "retry-local");
  live(value, "retry-local");
  ended(value, "local");
  ended(value, "display");
});

test("late permission preferences and display-source results cannot acquire after End", { timeout: 90_000 }, async (t) => {
  for (const boundary of ["permission", "preferences", "sources"]) {
    const f = await fixture(t, boundary);
    if (boundary === "sources") await f.accept(true, "local");
    const before = (await f.snapshot()).requests.length;
    await f.call("hold", boundary);
    await f.page.getByRole("button", { name: boundary === "sources" ? "Share screen" : "Accept video call", exact: true }).click();
    const held = await f.request(boundary, { after: before });
    await f.page.getByRole("button", { name: "End call", exact: true }).click();
    await f.call("flush");
    let value = await f.snapshot();
    if (boundary === "sources") ended(value, "local");
    const offset = value.requests.length;
    const events = value.peerEvents.length;
    await f.call("resolve", held.id);
    value = await f.snapshot();
    assert.equal(value.requests.slice(offset).filter((item) => item.kind === "user" || item.kind === "display").length, 0,
      `${boundary} completion cannot start capture in a cancelled operation`);
    assert.deepEqual(coordinatorAfter(value, offset), []);
    assert.deepEqual(attachmentAttemptsAfter(value, events), []);
    await f.freshIncoming();
    value = await f.accept(true, "retry-local");
    live(value, "retry-local");
    assert.equal(value.peers.filter((peer) => !peer.closed).length, 1);
  }
});

test("late local camera display and metadata completion after End preserve a newer call", { timeout: 120_000 }, async (t) => {
  for (const acquisition of ["local", "camera", "display", "metadata"]) {
    const f = await fixture(t, acquisition);
    let held;
    if (acquisition === "local" || acquisition === "metadata") {
      if (acquisition === "metadata") await f.call("hold", "coordinator", "updateCallMedia");
      const before = (await f.snapshot()).requests.length;
      await f.page.getByRole("button", { name: "Accept video call", exact: true }).click();
      held = await f.request("user", { after: before });
      if (acquisition === "metadata") {
        await f.call("resolveCapture", held.id, { label: "old-base", audio: 1, video: 1 });
        held = await f.request("coordinator", { method: "updateCallMedia", after: before });
      }
    } else {
      await f.accept(acquisition === "display", "old-base");
      const before = (await f.snapshot()).requests.length;
      await f.page.getByRole("button", { name: acquisition === "display" ? "Share screen" : "Camera on", exact: true }).click();
      held = await f.request(acquisition === "display" ? "display" : "user", { after: before });
    }
    await f.page.getByRole("button", { name: "End call", exact: true }).click();
    await f.call("flush");
    if (acquisition !== "local") ended(await f.snapshot(), "old-base");
    await f.freshIncoming();
    const newer = await f.accept(true, "new-local");
    const newerPeer = activePeer(newer);
    const newerPreview = newer.localPreview;
    const offset = newer.requests.length;
    const events = newer.peerEvents.length;
    if (acquisition === "metadata") await f.call("resolve", held.id);
    else await f.call("resolveCapture", held.id, { label: "late-capture", audio: acquisition === "local" ? 1 : 0, video: 1 });
    const value = await f.snapshot();
    if (acquisition !== "metadata") ended(value, "late-capture");
    live(value, "new-local");
    assert.equal(activePeer(value).id, newerPeer.id);
    assert.deepEqual(activePeer(value).senders, newerPeer.senders);
    assert.deepEqual(value.localPreview, newerPreview);
    assert.deepEqual(coordinatorAfter(value, offset), [], "A stale completion cannot send commands for the old or newer call");
    assert.deepEqual(attachmentAttemptsAfter(value, events), []);
    await f.page.getByRole("button", { name: "End call", exact: true }).waitFor();
  }
});

test("unmount and same-instance scope A B A dispose pending camera and display acquisitions", { timeout: 120_000 }, async (t) => {
  for (const cancellation of ["unmount", "scope-aba"]) {
    for (const acquisition of ["camera", "display"]) {
      const f = await fixture(t, `${cancellation}-${acquisition}`);
      await f.accept(acquisition === "display", "old-local");
      const before = (await f.snapshot()).requests.length;
      await f.page.getByRole("button", { name: acquisition === "display" ? "Share screen" : "Camera on", exact: true }).click();
      const held = await f.request(acquisition === "display" ? "display" : "user", { after: before });
      if (cancellation === "unmount") {
        await f.call("unmount");
        ended(await f.snapshot(), "old-local");
        await f.call("installIncoming", "scope-a");
        await f.call("mount");
      } else {
        await f.call("installIncoming", "scope-b");
        await f.call("switchScope", "scope-b");
        ended(await f.snapshot(), "old-local");
        await f.call("installIncoming", "scope-a");
        await f.call("switchScope", "scope-a");
      }
      const newer = await f.accept(true, "new-local");
      const offset = newer.requests.length;
      const events = newer.peerEvents.length;
      await f.call("resolveCapture", held.id, { label: "late-capture", video: 1 });
      const value = await f.snapshot();
      ended(value, "late-capture");
      ended(value, "old-local");
      live(value, "new-local");
      assert.equal(value.scope, "scope-a");
      assert.equal(value.pollingOwners, 1);
      assert.equal(activePeer(value).id, activePeer(newer).id);
      assert.deepEqual(activePeer(value).senders, activePeer(newer).senders);
      assert.deepEqual(value.localPreview, newer.localPreview);
      assert.deepEqual(coordinatorAfter(value, offset), []);
      assert.deepEqual(attachmentAttemptsAfter(value, events), []);
    }
  }
});

test("late ICE and admitted camera or display replacement cannot reopen ended or unmounted peers", { timeout: 120_000 }, async (t) => {
  for (const cancellation of ["end", "unmount"]) {
    for (const boundary of ["ice", "display-replace", "camera-replace"]) {
      const f = await fixture(t, `${cancellation}-${boundary}`);
      let held;
      if (boundary === "ice") {
        await f.call("hold", "coordinator", "getCallIceServers");
        const before = (await f.snapshot()).requests.length;
        await f.page.getByRole("button", { name: "Accept video call", exact: true }).click();
        const capture = await f.request("user", { after: before });
        await f.call("resolveCapture", capture.id, { label: "old-local", audio: 1, video: 1 });
        held = await f.request("coordinator", { method: "getCallIceServers", after: before });
      } else if (boundary === "display-replace") {
        await f.accept(true, "old-local");
        const before = (await f.snapshot()).requests.length;
        await f.call("hold", "replace");
        await f.page.getByRole("button", { name: "Share screen", exact: true }).click();
        const capture = await f.request("display", { after: before });
        await f.call("resolveCapture", capture.id, { label: "old-display", video: 1 });
        held = await f.request("replace", { after: before });
      } else {
        await f.accept(false, "old-local");
        await f.share("prior-display");
        await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
        await f.call("flush");
        const before = (await f.snapshot()).requests.length;
        await f.call("hold", "replace");
        await f.page.getByRole("button", { name: "Camera on", exact: true }).click();
        const capture = await f.request("user", { after: before });
        await f.call("resolveCapture", capture.id, { label: "old-camera", video: 1 });
        held = await f.request("replace", { after: before });
      }
      if (cancellation === "end") {
        await f.page.getByRole("button", { name: "End call", exact: true }).click();
        await f.call("flush");
      } else {
        await f.call("unmount");
      }
      let value = await f.snapshot();
      ended(value, "old-local");
      if (boundary === "display-replace") ended(value, "old-display");
      if (boundary === "camera-replace") ended(value, "old-camera");
      assert.equal(value.peers.filter((peer) => !peer.closed).length, 0);
      if (cancellation === "unmount") {
        await f.call("installIncoming", "scope-a");
        await f.call("mount");
      } else {
        await f.freshIncoming();
      }
      const newer = await f.accept(true, "new-local");
      const offset = newer.requests.length;
      const events = newer.peerEvents.length;
      await f.call("resolve", held.id);
      value = await f.snapshot();
      live(value, "new-local");
      ended(value, "old-local");
      if (boundary === "display-replace") ended(value, "old-display");
      if (boundary === "camera-replace") ended(value, "old-camera");
      assert.equal(activePeer(value).id, activePeer(newer).id);
      assert.deepEqual(activePeer(value).senders, activePeer(newer).senders);
      assert.deepEqual(value.localPreview, newer.localPreview);
      assert.deepEqual(coordinatorAfter(value, offset), []);
      assert.deepEqual(attachmentAttemptsAfter(value, events), [],
        "Completing an old admitted replacement permits no new production attach or restore attempt");
    }
  }
});

test("remote terminal client replacement and call failure release media and permit fresh acquisition", { timeout: 90_000 }, async (t) => {
  for (const ending of ["remote-terminal", "client-replacement", "ice-failure"]) {
    const f = await fixture(t, ending);
    if (ending === "ice-failure") {
      await f.call("hold", "coordinator", "getCallIceServers");
      const before = (await f.snapshot()).requests.length;
      await f.page.getByRole("button", { name: "Accept video call", exact: true }).click();
      const capture = await f.request("user", { after: before });
      await f.call("resolveCapture", capture.id, { label: "old-local", audio: 1, video: 1 });
      const ice = await f.request("coordinator", { method: "getCallIceServers", after: before });
      await f.call("reject", ice.id, "Fixture ICE refused");
      await f.page.getByRole("alert").waitFor();
    } else {
      await f.accept(true, "old-local");
      const initial = await f.share("old-display");
      if (ending === "remote-terminal") {
        const active = initial.calls.find((call) => call.scopeId === initial.scope && call.state !== "ended" && call.state !== "failed");
        assert.ok(active);
        await f.call("remoteTerminal", active.id);
        await f.call("poll");
      } else {
        await f.call("replaceClient");
      }
    }
    let value = await f.snapshot();
    ended(value, "old-local");
    if (ending !== "ice-failure") ended(value, "old-display");
    assert.equal(value.peers.filter((peer) => !peer.closed).length, 0);
    assert.equal(value.localPreview, null);
    assert.equal(value.remotePreview, null);
    if (ending !== "client-replacement") await f.freshIncoming();
    value = await f.accept(true, "new-local");
    live(value, "new-local");
    value = await f.share("new-display");
    live(value, "new-display");
    live(value, "new-local");
    assert.equal(value.peers.filter((peer) => !peer.closed).length, 1);
  }
});

test("stopped sharing reconciles current-generation metadata after reconnect or remote generation change", { timeout: 120_000 }, async (t) => {
  for (const replacement of ["local-reconnect", "remote-generation"]) {
    for (const successor of ["new-share", "new-call"]) {
      const f = await fixture(t, `${replacement}-${successor}`);
      await f.accept(true, "local");
      const shared = await f.share("old-display");
      const originalCall = shared.calls.find((call) => call.scopeId === shared.scope && call.state === "negotiating");
      assert.ok(originalCall);
      assert.equal(originalCall.generation, 0);
      assert.equal(originalCall.mediaCapabilities.screenShare, true,
        "The original sharing=true was acknowledged before Stop begins");
      const originalPeer = activePeer(shared);
      const camera = track(shared, "local");
      const microphone = track(shared, "local", "audio");
      const offset = shared.requests.length;
      await f.call("hold", "replace");
      await f.page.getByRole("button", { name: "Stop sharing", exact: true }).click();
      const restore = await f.request("replace", { after: offset });
      assert.equal(restore.args.peerId, originalPeer.id);
      assert.equal(restore.args.trackId, camera.id,
        "The held operation is Stop's restoration of the retained camera");
      let value = await f.snapshot();
      ended(value, "old-display");
      live(value, "local");
      assert.ok(value.pending.includes(restore.id));
      if (replacement === "local-reconnect") {
        await f.call("setPeerConnectionState", originalPeer.id, "disconnected");
      } else {
        const advanced = await f.call("advanceRemoteGeneration", originalCall.id);
        assert.equal(advanced.generation, originalCall.generation + 1);
        await f.call("poll");
      }
      await f.page.waitForFunction(({ callId, oldPeer, generation }) => {
        const value = window.__callMediaHarness.snapshot();
        const current = value.calls.find((call) => call.id === callId);
        return current?.generation === generation
          && current.mediaCapabilities.screenShare === false
          && value.peers.some((peer) => !peer.closed && peer.id !== oldPeer)
          && value.requests.some((request) => request.method === "updateCallMedia"
            && request.args.callId === callId && request.args.generation === generation
            && request.args.mediaCapabilities.screenShare === false && request.settled);
      }, { callId: originalCall.id, oldPeer: originalPeer.id, generation: originalCall.generation + 1 });
      value = await f.snapshot();
      ended(value, "old-display");
      live(value, "local");
      assert.equal(value.calls.find((call) => call.id === originalCall.id).mediaCapabilities.screenShare, false);
      assert.equal(value.calls.find((call) => call.id === originalCall.id).generation, 1);
      assert.ok(value.peers.find((peer) => peer.id === originalPeer.id).closed);
      assert.notEqual(activePeer(value).id, originalPeer.id);
      assert.deepEqual(activePeer(value).senders.map((sender) => sender.trackId).sort(), [camera.id, microphone.id].sort());
      assert.ok(value.pending.includes(restore.id), "Current-generation false settles while the old restoration is still held");
      const reconciled = value.requests.slice(offset).filter((request) => request.method === "updateCallMedia"
        && request.args.callId === originalCall.id && request.args.generation === 1
        && request.args.mediaCapabilities.screenShare === false && request.settled);
      assert.ok(reconciled.length >= 1, "A current-generation command acknowledged the local stop obligation");
      if (replacement === "local-reconnect") {
        const transitions = value.requests.slice(offset).filter((request) => request.method === "transitionCallSession");
        assert.deepEqual(transitions.map((request) => [request.args.action, request.args.generation]), [["reconnect", 0], ["resume", 1]]);
      }
      if (successor === "new-call") {
        await f.page.getByRole("button", { name: "End call", exact: true }).click();
        await f.call("flush");
        ended(await f.snapshot(), "local");
        await f.freshIncoming();
        await f.accept(true, "new-local");
      }
      const newer = await f.share("new-display");
      const newerPeer = activePeer(newer);
      const newerCall = newer.calls.find((call) => call.scopeId === newer.scope && call.state === "negotiating");
      assert.ok(newerCall);
      assert.equal(newerCall.mediaCapabilities.screenShare, true);
      if (successor === "new-call") assert.notEqual(newerCall.id, originalCall.id);
      else {
        assert.equal(newerCall.id, originalCall.id);
        assert.equal(newerCall.generation, 1);
      }
      const requestOffset = newer.requests.length;
      const eventOffset = newer.peerEvents.length;
      await f.call("resolve", restore.id);
      value = await f.snapshot();
      ended(value, "old-display");
      live(value, "new-display");
      live(value, successor === "new-call" ? "new-local" : "local");
      assert.equal(activePeer(value).id, newerPeer.id);
      assert.deepEqual(activePeer(value).senders, newerPeer.senders);
      assert.deepEqual(value.localPreview, newer.localPreview);
      assert.equal(value.calls.find((call) => call.id === newerCall.id).mediaCapabilities.screenShare, true);
      assert.deepEqual(coordinatorAfter(value, requestOffset), [],
        "Old restoration completion cannot publish false into a newer share or call");
      assert.deepEqual(attachmentAttemptsAfter(value, eventOffset), []);
      await f.page.getByRole("button", { name: "Stop sharing", exact: true }).waitFor();
      assert.equal(await f.page.getByRole("alert").count(), 0);
    }
  }
});

test("obsolete Accept ICE rejection preserves the current peer in same and newer generations", { timeout: 90_000 }, async (t) => {
  for (const replacement of ["same-generation", "remote-generation"]) {
    const f = await fixture(t, replacement);
    await f.call("hold", "coordinator", "getCallIceServers");
    const before = (await f.snapshot()).requests.length;
    await f.page.getByRole("button", { name: "Accept video call", exact: true }).click();
    const capture = await f.request("user", { after: before });
    await f.call("resolveCapture", capture.id, { label: "local", audio: 1, video: 1 });
    const obsoleteIce = await f.request("coordinator", { method: "getCallIceServers", after: before });
    const held = await f.snapshot();
    const originalCall = held.calls.find((call) => call.scopeId === held.scope && call.state === "negotiating");
    assert.ok(originalCall);
    assert.equal(originalCall.generation, 0);
    assert.equal(held.peers.length, 0, "The first Accept peer attempt is waiting before construction");
    live(held, "local");
    assert.ok(held.localPreview);
    if (replacement === "remote-generation") {
      const advanced = await f.call("advanceRemoteGeneration", originalCall.id);
      assert.equal(advanced.generation, originalCall.generation + 1);
    }
    // Polling enters the real configurePeer path with the retained local stream.
    // In the first variant the call ID and generation are deliberately unchanged.
    await f.call("poll");
    await f.page.waitForFunction((obsoleteId) => {
      const value = window.__callMediaHarness.snapshot();
      return value.pending.includes(obsoleteId)
        && value.requests.filter((request) => request.method === "getCallIceServers").length === 2
        && value.peers.some((peer) => !peer.closed);
    }, obsoleteIce.id);
    const newer = await f.snapshot();
    const currentCall = newer.calls.find((call) => call.id === originalCall.id);
    const expectedGeneration = replacement === "same-generation" ? 0 : 1;
    assert.equal(currentCall.generation, expectedGeneration);
    assert.equal(currentCall.state, "negotiating");
    assert.equal(currentCall.id, originalCall.id);
    assert.equal(newer.peers.length, 1, "The second ICE request installed the only current peer");
    const newerPeer = activePeer(newer);
    live(newer, "local");
    assert.deepEqual(newer.localPreview, held.localPreview);
    assert.equal(newer.requests.filter((request) => request.method === "getCallIceServers" && request.settled).length, 1);
    assert.ok(newer.pending.includes(obsoleteIce.id));
    const requestOffset = newer.requests.length;
    const eventOffset = newer.peerEvents.length;
    await f.call("reject", obsoleteIce.id, "Obsolete Accept ICE rejection");
    let value = await f.snapshot();
    assert.ok(!value.pending.includes(obsoleteIce.id));
    live(value, "local");
    assert.equal(activePeer(value).id, newerPeer.id);
    assert.deepEqual(activePeer(value).senders, newerPeer.senders);
    assert.deepEqual(value.localPreview, newer.localPreview);
    assert.deepEqual(value.calls.find((call) => call.id === originalCall.id), currentCall,
      "An obsolete failure cannot fail or mutate the still-current call");
    assert.deepEqual(coordinatorAfter(value, requestOffset), [],
      "Obsolete rejection emits no fail, media, signaling, or follow-on Coordinator request");
    assert.deepEqual(attachmentAttemptsAfter(value, eventOffset), []);
    assert.deepEqual(value.peerEvents.slice(eventOffset), [], "The newer peer is not closed by the older rejection");
    assert.equal(await f.page.getByRole("alert").count(), 0);
    await f.page.getByRole("button", { name: "End call", exact: true }).waitFor();
    await f.page.getByRole("button", { name: "Camera off", exact: true }).click();
    await f.call("flush");
    value = await f.snapshot();
    live(value, "local");
    assert.equal(activePeer(value).id, newerPeer.id);
    assert.equal(track(value, "local").enabled, false);
    assert.equal(mediaUpdates(value).at(-1).args.generation, expectedGeneration,
      "The preserved current owner still accepts a normal camera command");
  }
});
