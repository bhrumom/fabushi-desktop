import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createServer } from "node:http";
import { createSocket } from "node:dgram";
import { createServer as createTcpServer } from "node:net";
import { access, mkdtemp, rm, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createRequire } from "node:module";
import { after, test } from "node:test";
import { _electron as electron } from "playwright";
import {
  createMiniAppWindowFactory,
  FABUSHI_MINI_APP_PRIVILEGED_SCHEME,
  type MiniAppDownloadItem,
  type MiniAppSession,
  type MiniAppWindow,
  type MiniAppWindowOptions,
  type VerifiedMiniAppLaunch,
} from "./miniapp-window.js";

const require = createRequire(import.meta.url);
const accountA = "a".repeat(64);
const accountB = "b".repeat(64);
const digest = "1".repeat(64);
const bytes = (text: string) => new TextEncoder().encode(text);
const localRequest = (url: string) => ({ url, method: "GET" }) as Request;
const launchFor = (accountScope = accountA, accountGeneration = 4, appId = "org.fabushi.global-dandelion"): VerifiedMiniAppLaunch => ({
  appId, version: "1.0.0", packageDigest: digest, entrypoint: "index.html", accountScope, accountGeneration,
  resources: new Map([["index.html", { bytes: bytes("<html><head><meta charset=utf-8></head><body><h1 id=result>loaded</h1><script src=app.js></script></body></html>"), mime: "text/html" }], ["LICENSE", { bytes: bytes("license"), mime: "text/plain" }]]),
});

const testSessions: Array<{ session: MiniAppSession; options?: MiniAppWindowOptions }> = [];
let active = { accountScope: accountA, generation: 4 };
let factory: ReturnType<typeof createMiniAppWindowFactory>;
let opens = 0;
let lastWindow: FakeWindow | undefined;

class FakeWindow implements MiniAppWindow {
  readonly webContents: MiniAppWindow["webContents"];
  #listeners = new Map<string, Array<() => void>>();
  #destroyed = false;
  closed = false;
  focused = false;
  webrtcPolicy: string | undefined;
  constructor(readonly options: MiniAppWindowOptions) {
    const session = testSessions.at(-1)!.session;
    this.webContents = {
      session,
      setWebRTCIPHandlingPolicy: async (policy: string) => { this.webrtcPolicy = policy; },
      setWindowOpenHandler: () => {},
      on: (event: string, listener: (...args: never[]) => void) => {
        const all = this.#listeners.get(event) ?? [];
        all.push(listener as () => void);
        this.#listeners.set(event, all);
      },
      loadURL: async () => { opens += 1; },
    } as MiniAppWindow["webContents"];
    lastWindow = this;
  }
  isDestroyed() { return this.#destroyed; }
  show() {}
  focus() { this.focused = true; }
  close() { this.closed = true; this.#destroyed = true; for (const listener of this.#listeners.get("closed") ?? []) listener(); }
  destroy() { this.close(); }
  on(event: "closed", listener: () => void) { const all = this.#listeners.get(event) ?? []; all.push(listener); this.#listeners.set(event, all); }
}

function fakeSession() {
  let protocolHandler: ((request: Request) => Response | Promise<Response>) | undefined;
  let requestListener: ((details: { url: string }, callback: (result: { cancel: boolean }) => void) => void) | null = null;
  let downloadListener: ((event: { preventDefault(): void }, item: MiniAppDownloadItem, contents: MiniAppWindow["webContents"]) => void) | undefined;
  return {
    protocol: { handle: async (_scheme: string, handler: (request: Request) => Response | Promise<Response>) => { protocolHandler = handler; }, unhandle: () => { protocolHandler = undefined; } },
    webRequest: { onBeforeRequest: (_filter: unknown, listener: typeof requestListener) => { requestListener = listener; } },
    setProxy: async () => {}, setPermissionRequestHandler: () => {}, setPermissionCheckHandler: () => {},
    on: (_event: string, listener: typeof downloadListener) => { downloadListener = listener; },
    removeListener: () => { downloadListener = undefined; },
    protocolRequest: (request: Request) => protocolHandler?.(request),
    networkAllowed: (url: string) => new Promise<boolean>((resolve) => requestListener?.({ url }, (result) => resolve(!result.cancel))),
    download: (values: Partial<MiniAppDownloadItem> = {}, contents?: MiniAppWindow["webContents"]) => {
      let prevented = false;
      const item = { getURL: () => values.getURL?.() ?? "https://example.com/file.json", getMimeType: () => values.getMimeType?.() ?? "application/json", getFilename: () => values.getFilename?.() ?? "report.json", getTotalBytes: () => values.getTotalBytes?.() ?? 10, getReceivedBytes: () => values.getReceivedBytes?.() ?? 0, hasUserGesture: () => values.hasUserGesture?.() ?? false, cancel: () => values.cancel?.(), on: (event: "updated" | "done", listener: () => void) => values.on?.(event, listener) } as MiniAppDownloadItem;
      downloadListener?.({ preventDefault() { prevented = true; } }, item, contents ?? lastWindow?.webContents!);
      return prevented;
    },
  } as unknown as MiniAppSession & { protocolRequest(request: Request): Response | Promise<Response> | undefined; networkAllowed(url: string): Promise<boolean>; download(values?: Partial<MiniAppDownloadItem>, contents?: MiniAppWindow["webContents"]): boolean };
}

test("scheme declaration is ready for the existing single pre-ready Electron registration", () => {
  assert.deepEqual(FABUSHI_MINI_APP_PRIVILEGED_SCHEME, { scheme: "fabushi-miniapp", privileges: {
    standard: true, secure: true, supportFetchAPI: true, corsEnabled: false, allowServiceWorkers: false,
  } });
});

test("Host resources are copied, bounded, app-scoped, and all non-package requests are denied", async () => {
  testSessions.length = 0;
  factory = createMiniAppWindowFactory({
    BrowserWindow: class extends FakeWindow { constructor(options: MiniAppWindowOptions) { super(options); testSessions.at(-1)!.options = options; } },
    session: { fromPartition: () => { const value = fakeSession(); testSessions.push({ session: value }); return value; } },
    currentAccount: () => active,
  });
  const source = launchFor();
  const original = source.resources.get("index.html")!.bytes;
  await factory.open(source);
  original.fill(0);
  const options = testSessions.at(-1)!.options!;
  assert.equal(options.webPreferences.nodeIntegration, false);
  assert.equal(options.webPreferences.contextIsolation, true);
  assert.equal(options.webPreferences.sandbox, true);
  assert.equal(options.webPreferences.webSecurity, true);
  assert.equal(options.webPreferences.webviewTag, false);
  assert.equal("preload" in options.webPreferences, false);
  assert.equal(lastWindow?.webrtcPolicy, "disable_non_proxied_udp");
  const session = testSessions.at(-1)!.session as ReturnType<typeof fakeSession>;
  const host = `${createHash("sha256").update(accountA).update("\0").update("org.fabushi.global-dandelion").digest("hex")}.app`;
  const response = await session.protocolRequest(localRequest(`fabushi-miniapp://${host}/index.html`));
  assert.ok(response instanceof Response);
  assert.equal(response.status, 200);
  assert.match(response.headers.get("content-security-policy") ?? "", /worker-src 'none'/);
  assert.match(response.headers.get("content-security-policy") ?? "", /wasm-unsafe-eval/);
  assert.equal(await response.text().then((text) => text.includes("loaded")), true);
  const license = await session.protocolRequest(localRequest(`fabushi-miniapp://${host}/LICENSE`));
  assert.equal(license instanceof Response ? license.headers.get("content-type") : null, "text/plain");
  assert.equal(await session.networkAllowed("https://example.com/data"), false);
  assert.equal(await session.networkAllowed("wss://example.com/socket"), false);
  assert.equal(await session.networkAllowed(`fabushi-miniapp://${host}/unknown.js`), true); // Request gate permits origin; resource handler still returns 404.
  assert.equal(session.download(), true);
  const reportBlob = `blob:fabushi-miniapp://${host}/opaque-id`;
  assert.equal(session.download({ getURL: () => reportBlob, getMimeType: () => "application/json", getFilename: () => "global-dandelion-local-report.json", hasUserGesture: () => true, getTotalBytes: () => 2 * 1024 * 1024 }), true);
  assert.equal(session.download({ getURL: () => reportBlob, getMimeType: () => "application/json", getFilename: () => "global-dandelion-local-report.json", hasUserGesture: () => true }), false);
  const missing = await session.protocolRequest(localRequest(`fabushi-miniapp://${host}/missing.js`));
  assert.equal(missing instanceof Response ? missing.status : 500, 404);
  const foreign = await session.protocolRequest(localRequest("fabushi-miniapp://other-app.app/index.html"));
  assert.equal(foreign instanceof Response ? foreign.status : 500, 404);
  assert.equal(await session.networkAllowed("fabushi-miniapp://other-app.app/index.html"), false);
  factory.closeAll();
});

test("stale account generations are denied, and account teardown closes windows before reuse", async () => {
  const before = opens;
  active = { accountScope: accountA, generation: 4 };
  await assert.rejects(factory.open(launchFor(accountB, 4)), /MINIAPP_STALE_ACCOUNT/);
  assert.equal(opens, before);
  await factory.open(launchFor());
  const current = lastWindow!;
  factory.closeAccountScope(accountA, 5);
  assert.equal(current.closed, true);
  active = { accountScope: accountA, generation: 5 };
  await assert.rejects(factory.open(launchFor(accountA, 4)), /MINIAPP_STALE_ACCOUNT/);
  factory.closeAll();
});

test("actual Electron uses the isolated protocol and blocks real network, worker, WebRTC, popup and navigation attempts", { skip: process.env.FABUSHI_MINIAPP_ELECTRON !== "1", timeout: 120_000 }, async () => {
  const bundle = process.env.FABUSHI_MINIAPP_WINDOW_BUNDLE;
  assert.ok(bundle, "workflow must provide the esbuild output");
  let networkHits = 0;
  let udpHits = 0, tcpHits = 0;
  const udp = createSocket("udp4");
  udp.on("message", () => { udpHits += 1; });
  await new Promise<void>((resolve) => udp.bind(0, "127.0.0.1", resolve));
  const udpAddress = udp.address();
  const tcp = createTcpServer((socket) => { tcpHits += 1; socket.destroy(); });
  await new Promise<void>((resolve) => tcp.listen(0, "127.0.0.1", resolve));
  const tcpAddress = tcp.address();
  assert.ok(tcpAddress && typeof tcpAddress !== "string");
  const iceServers = [{ urls: `stun:127.0.0.1:${udpAddress.port}` },
    { urls: `turn:127.0.0.1:${tcpAddress.port}?transport=tcp`, username: "fixture", credential: "fixture" }];
  const server = createServer((_request, response) => { networkHits += 1; response.end("unexpected egress"); });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const endpoint = `http://127.0.0.1:${address.port}/probe`;
  const directory = await mkdtemp(join(tmpdir(), "fabushi-miniapp-electron-"));
  const appJs = `
    const endpoint = ${JSON.stringify(endpoint)};
    const record = (name, value) => { window.__results ||= {}; window.__results[name] = value; };
    window.__exportReport = () => { const link = document.createElement('a'); link.href = URL.createObjectURL(new Blob([JSON.stringify({ report: 'offline' })], {type:'application/json'})); link.download = 'global-dandelion-local-report.json'; link.click(); setTimeout(() => URL.revokeObjectURL(link.href), 1000); };
    document.addEventListener('DOMContentLoaded', () => document.getElementById('export').addEventListener('click', window.__exportReport));
    Promise.all([
      fetch(endpoint).then(() => record('fetch', 'allowed')).catch(() => record('fetch', 'blocked')),
      new Promise(resolve => { try { const ws = new WebSocket(endpoint.replace('http:', 'ws:')); ws.onerror = () => resolve(record('websocket', 'blocked')); ws.onopen = () => resolve(record('websocket', 'allowed')); setTimeout(() => resolve(record('websocket', 'blocked')), 1000); } catch { resolve(record('websocket', 'blocked')); } }),
      new Promise(resolve => { try { const worker = new Worker(URL.createObjectURL(new Blob(['postMessage(1)']))); worker.onmessage = () => resolve(record('worker', 'allowed')); worker.onerror = () => resolve(record('worker', 'blocked')); setTimeout(() => resolve(record('worker', 'blocked')), 500); } catch { resolve(record('worker', 'blocked')); } }),
      new Promise(resolve => { if (!navigator.serviceWorker) return resolve(record('serviceWorker', 'blocked')); navigator.serviceWorker.register('sw.js').then(() => resolve(record('serviceWorker', 'allowed'))).catch(() => resolve(record('serviceWorker', 'blocked'))); }),
      new Promise(resolve => { try { const pc = new RTCPeerConnection({iceServers:${JSON.stringify(iceServers)}}); const candidates=[]; pc.createDataChannel('x'); pc.onicecandidate = e => { if (e.candidate) candidates.push(e.candidate.type); else { record('webrtc', candidates.some(type => type==='srflx' || type==='relay') ? 'egress' : 'blocked'); pc.close(); resolve(); } }; pc.createOffer().then(o => pc.setLocalDescription(o)); setTimeout(() => { pc.close(); record('webrtc', 'timed-out'); resolve(); }, 2500); } catch { resolve(record('webrtc', 'blocked')); } }),
      new Promise(resolve => { if (!navigator.geolocation) return resolve(record('permission', 'blocked')); navigator.geolocation.getCurrentPosition(() => resolve(record('permission', 'allowed')), () => resolve(record('permission', 'blocked'))); setTimeout(() => resolve(record('permission', 'blocked')), 1000); }),
    ]).then(() => record('ready', true));
    window.__domReady = true;
  `;
  const html = `<meta charset="utf-8"><body><h1>Mini App</h1><button id="export" onclick="window.__exportReport()">Export report</button><script src="app.js"></script><script>window.__domReady=true</script>`;
  const harness = `
    const { app, protocol, BrowserWindow, session } = require('electron');
    const crypto = require('node:crypto');
    const mini = require(process.env.FABUSHI_MINIAPP_WINDOW_BUNDLE);
    const scope = 'c'.repeat(64), otherScope = 'd'.repeat(64);
    const NativeBrowserWindow = BrowserWindow;
    function TrackingBrowserWindow(options) {
      // Electron's getAllWindows filters constructor.name; retain the native object.
      const window = new NativeBrowserWindow(options);
      const applyPolicy = window.webContents.setWebRTCIPHandlingPolicy.bind(window.webContents);
      window.webContents.setWebRTCIPHandlingPolicy = async (policy) => { globalThis.__miniAppWebRTCPolicy = policy; return applyPolicy(policy); };
      return window;
    }
    protocol.registerSchemesAsPrivileged([mini.FABUSHI_MINI_APP_PRIVILEGED_SCHEME]);
    app.whenReady().then(() => {
      globalThis.__noSandboxSwitch = app.commandLine.hasSwitch('no-sandbox');
      globalThis.__active = {accountScope:scope,generation:7};
      globalThis.__mini = mini.createMiniAppWindowFactory({ BrowserWindow: TrackingBrowserWindow, session, currentAccount: () => globalThis.__active });
      const scoped = crypto.createHash('sha256').update(scope).update('\\0').update('org.fabushi.global-dandelion').digest('hex');
      const fixtureSession = session.fromPartition('persist:fabushi-miniapp-' + scoped);
      fixtureSession.setDownloadPath(process.env.FABUSHI_MINIAPP_DOWNLOAD_DIR);
      globalThis.__downloadFacts = [];
      fixtureSession.on('will-download', (event,item,contents) => {
        globalThis.__downloadFacts.push({url:item.getURL(),mime:item.getMimeType(),filename:item.getFilename(),size:item.getTotalBytes(),gesture:item.hasUserGesture(),contents:contents && contents.id});
      });
      globalThis.__snapshotFor = (appId, accountScope=scope, accountGeneration=7) => ({ appId, version:'1.0.0', packageDigest:'${digest}', entrypoint:'index.html', accountScope, accountGeneration,
        resources:new Map([['index.html',{bytes:new TextEncoder().encode(${JSON.stringify(html)}),mime:'text/html'}],['app.js',{bytes:new TextEncoder().encode(${JSON.stringify(appJs)}),mime:'text/javascript'}],['module.wasm',{bytes:new Uint8Array([0,97,115,109,1,0,0,0]),mime:'application/wasm'}]]) });
      globalThis.__open = async (snapshot) => { globalThis.__handle = await globalThis.__mini.open(snapshot); };
      globalThis.__close = () => globalThis.__handle.close();
    });
  `;
  await writeFile(join(directory, "package.json"), JSON.stringify({ name: "miniapp-window-test", version: "1.0.0", main: "main.cjs" }));
  await writeFile(join(directory, "main.cjs"), harness);
  const downloadDirectory = join(directory, "downloads");
  await (await import("node:fs/promises")).mkdir(downloadDirectory);
  const electronBinary = require("electron") as string;
  let application: Awaited<ReturnType<typeof electron.launch>> | undefined;
  try {
    const app = application = await electron.launch({ executablePath: electronBinary, args: [directory], chromiumSandbox: true, timeout: 30_000, env: { ...process.env, FABUSHI_MINIAPP_WINDOW_BUNDLE: resolve(bundle), FABUSHI_MINIAPP_DOWNLOAD_DIR: downloadDirectory } });
    await app.evaluate(async () => { await globalThis.__open(globalThis.__snapshotFor('org.fabushi.global-dandelion')); });
    assert.equal(await app.evaluate(() => globalThis.__noSandboxSwitch), false, "sandbox acceptance must not launch Electron with --no-sandbox");
    assert.equal(await app.evaluate(() => globalThis.__miniAppWebRTCPolicy), "disable_non_proxied_udp");
    const page = await app.firstWindow();
    await page.waitForFunction(() => window.__results?.ready === true, null, { timeout: 20_000 });
    const browserFacts = await page.evaluate(async () => {
      const wasm = await fetch("module.wasm").then((response) => response.arrayBuffer()).then((buffer) => WebAssembly.instantiate(buffer));
      return { results: window.__results, hasNode: typeof process !== "undefined" || typeof require !== "undefined", origin: location.origin, wasm: Boolean(wasm.instance) };
    });
    assert.equal(browserFacts.hasNode, false);
    if (process.platform === "linux") {
      const pid = await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].webContents.getOSProcessId());
      const status = await (await import("node:fs/promises")).readFile(`/proc/${pid}/status`, "utf8");
      assert.match(status, /^Seccomp:\s+2$/m, "actual renderer must use kernel seccomp filtering");
    }
    assert.equal(browserFacts.wasm, true);
    assert.match(browserFacts.origin, /^fabushi-miniapp:\/\//);
    assert.equal(browserFacts.results.fetch, "blocked");
    assert.equal(browserFacts.results.websocket, "blocked");
    assert.equal(browserFacts.results.worker, "blocked");
    assert.equal(browserFacts.results.serviceWorker, "blocked");
    assert.equal(browserFacts.results.permission, "blocked");
    assert.ok(["blocked", "timed-out"].includes(browserFacts.results.webrtc));
    const downloadEvent = page.waitForEvent("download", { timeout: 10_000 });
    await page.getByRole("button", { name: "Export report" }).click();
    const reportPath = join(downloadDirectory, "global-dandelion-local-report.json");
    // Playwright owns accepted download paths in automation; this is not OS dialog acceptance.
    const reportDownload = await downloadEvent.catch(async (error) => {
      const facts = await app.evaluate(() => globalThis.__downloadFacts);
      throw new Error('report download denied; native fixture facts=' + JSON.stringify(facts), {cause:error});
    });
    assert.equal(reportDownload.suggestedFilename(), "global-dandelion-local-report.json");
    assert.equal(await reportDownload.failure(), null, "native policy must allow the real user-gesture export");
    await reportDownload.saveAs(reportPath);
    assert.equal((await (await import("node:fs/promises")).readFile(reportPath, "utf8")).includes('"report":"offline"'), true);
    await page.waitForTimeout(500);
    await unlink(reportPath);
    let unauthorizedDownloads = 0;
    page.on("download", () => { unauthorizedDownloads += 1; });
    await page.evaluate(() => setTimeout(() => window.__exportReport(), 6000));
    await page.waitForTimeout(6500);
    await assert.rejects(access(reportPath));
    assert.equal(unauthorizedDownloads, 0, "gesture-free exports must not reach Playwright's intercepted download storage either");
    await page.evaluate((url) => { const link = document.createElement('a'); link.href = url; link.download = 'untrusted.json'; link.click(); }, endpoint);
    await page.waitForTimeout(500);
    await assert.rejects(access(reportPath));
    assert.equal(unauthorizedDownloads, 0, "foreign downloads must be denied before a download event");
    await assert.rejects(app.evaluate(async ({ BrowserWindow }, url) => {
      const window = BrowserWindow.getAllWindows()[0];
      await window.webContents.session.fetch(url);
    }, endpoint));
    await page.evaluate(() => { window.__popup = window.open("https://example.com"); location.href = "https://example.com"; });
    await page.waitForTimeout(500);
    assert.equal(await page.evaluate(() => window.__popup === null), true);
    assert.match(page.url(), /^fabushi-miniapp:\/\//);
    assert.equal(networkHits, 0);
    assert.equal(udpHits, 0, "WebRTC must not send direct UDP outside its offline proxy");
    assert.equal(tcpHits, 0, "WebRTC must not connect directly to a TURN TCP endpoint");

    await page.evaluate(() => localStorage.setItem("account-app-data", "preserve-me"));
    await app.evaluate(() => globalThis.__close());
    await app.evaluate(async () => { await globalThis.__open(globalThis.__snapshotFor('org.fabushi.global-dandelion')); });
    const reopened = await app.firstWindow();
    assert.equal(await reopened.evaluate(() => localStorage.getItem("account-app-data")), "preserve-me");
    const firstOrigin = await reopened.evaluate(() => location.origin);

    await app.evaluate(async () => { await globalThis.__open(globalThis.__snapshotFor('org.fabushi.otherapp')); });
    const pagesForTwoApps = await app.windows();
    assert.equal(pagesForTwoApps.length, 2);
    const secondApp = pagesForTwoApps.find((candidate) => candidate !== reopened)!;
    assert.notEqual(await secondApp.evaluate(() => location.origin), firstOrigin);
    assert.equal(await secondApp.evaluate(() => localStorage.getItem("account-app-data")), null);

    await app.evaluate(() => { globalThis.__mini.closeAccountScope('c'.repeat(64), 8); globalThis.__active = {accountScope:'d'.repeat(64),generation:8}; });
    await app.evaluate(async () => { await globalThis.__open(globalThis.__snapshotFor('org.fabushi.global-dandelion', 'd'.repeat(64), 8)); });
    const accountB = await app.firstWindow();
    assert.notEqual(await accountB.evaluate(() => location.origin), firstOrigin);
    assert.equal(await accountB.evaluate(() => localStorage.getItem("account-app-data")), null);
    const prefs = await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows().at(-1).webContents.getLastWebPreferences());
    assert.equal(prefs.nodeIntegration, false);
    assert.equal(prefs.contextIsolation, true);
    assert.equal(prefs.sandbox, true);
    assert.equal(prefs.webSecurity, true);
    assert.equal(prefs.preload, undefined);
  } finally {
    try {
      if (application) {
        await application.evaluate(() => globalThis.__mini?.closeAll()).catch(() => {});
        await application.close();
      }
    } finally {
      server.close();
      udp.close();
      tcp.close();
      await rm(directory, { recursive: true, force: true });
    }
  }
});

test("actual signed Dandelion bundle runs its Rust codec offline in the production isolated window", { skip: !process.env.FABUSHI_DANDELION_SIGNED_FIXTURE, timeout: 90_000 }, async () => {
  const bundle = process.env.FABUSHI_MINIAPP_WINDOW_BUNDLE;
  assert.ok(bundle);
  const directory = await mkdtemp(join(tmpdir(), "fabushi-dandelion-interop-"));
  const harness = `
    const {app, protocol, BrowserWindow, session} = require('electron');
    const fs = require('node:fs'), path = require('node:path'), crypto = require('node:crypto');
    const mini = require(process.env.FABUSHI_MINIAPP_WINDOW_BUNDLE);
    protocol.registerSchemesAsPrivileged([mini.FABUSHI_MINI_APP_PRIVILEGED_SCHEME]);
    app.whenReady().then(() => {
      globalThis.__factory = mini.createMiniAppWindowFactory({BrowserWindow, session, currentAccount:() => ({accountScope:'e'.repeat(64),generation:1})});
      globalThis.__openSignedFixture = async () => {
        const root = process.env.FABUSHI_DANDELION_SIGNED_FIXTURE;
        const manifest = JSON.parse(fs.readFileSync(path.join(root,'manifest.json'),'utf8'));
        if (manifest.appId !== 'org.fabushi.global-dandelion' || manifest.version !== '0.1.0' || manifest.signature.keyId !== 'fabushi-marketplace-managed-20261013-1') throw new Error('wrong pinned fixture identity');
        const resources = new Map();
        for (const [name, metadata] of Object.entries(manifest.files)) {
          if (!name.split('/').every(part => /^[A-Za-z0-9._-]+$/.test(part) && part!=='.' && part!=='..')) throw new Error('invalid fixture path');
          const bytes = fs.readFileSync(path.join(root,name));
          if (bytes.length !== metadata.bytes || crypto.createHash('sha256').update(bytes).digest('hex') !== metadata.sha256) throw new Error('fixture resource mismatch');
          resources.set(name,{bytes,mime:metadata.mime});
        }
        await globalThis.__factory.open({appId:manifest.appId,version:manifest.version,packageDigest:'de10e02115415908f827723457817e51cb4f39aa5ca68798da824fecc483118a',entrypoint:manifest.entrypoint,accountScope:'e'.repeat(64),accountGeneration:1,resources});
      };
    });
  `;
  await writeFile(join(directory, "main.cjs"), harness);
  await writeFile(join(directory, "package.json"), JSON.stringify({name:"dandelion-interoperability",version:"1.0.0",main:"main.cjs"}));
  let application: Awaited<ReturnType<typeof electron.launch>> | undefined;
  try {
    const app = application = await electron.launch({executablePath:require("electron") as string,args:[directory],chromiumSandbox:true,timeout:30_000,env:{...process.env,FABUSHI_MINIAPP_WINDOW_BUNDLE:resolve(bundle)}});
    await app.evaluate(async () => { await globalThis.__openSignedFixture(); });
    const page = await app.firstWindow();
    await page.waitForFunction(() => document.getElementById("status")?.textContent?.includes("已就绪"), null, {timeout:15_000});
    assert.equal(await page.evaluate(() => typeof process !== "undefined" || typeof require !== "undefined"), false);
    await page.locator("#files").setInputFiles([{name:"<img onerror=alert(1)>.txt",mimeType:"text/plain",buffer:Buffer.from("hello")},{name:"empty.txt",mimeType:"text/plain",buffer:Buffer.alloc(0)}]);
    assert.equal(await page.locator("#selection img").count(), 0);
    await page.locator("#start").click();
    await page.waitForFunction(() => document.getElementById("status")?.textContent?.includes("本地校验完成"), null, {timeout:15_000});
    assert.equal(await page.locator("#file-count").textContent(), "2");
    assert.equal(await page.locator("#packet-count").textContent(), "2");
    const pending = page.waitForEvent("download", {timeout:10_000});
    await page.locator("#export").click();
    const download = await pending;
    assert.equal(await download.failure(), null);
    const filename = await download.path();
    assert.ok(filename);
    const report = JSON.parse(await (await import("node:fs/promises")).readFile(filename, "utf8"));
    assert.equal(report.networkSent, false);
    assert.equal(report.verifiedBytes, 5);
    assert.equal(report.status, "complete");
    await page.reload();
    await page.waitForFunction(() => document.getElementById("status")?.textContent?.includes("已就绪"));
    assert.equal(await page.locator("#selection li").count(), 0);
    assert.equal(await app.evaluate(({BrowserWindow}) => BrowserWindow.getAllWindows()[0].webContents.getLastWebPreferences().sandbox), true);
  } finally {
    try { if (application) { await application.evaluate(() => globalThis.__factory?.closeAll()).catch(() => {}); await application.close(); } }
    finally { await rm(directory, {recursive:true,force:true}); }
  }
});

after(() => { factory?.closeAll(); });
