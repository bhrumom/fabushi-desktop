import test from "node:test";
import assert from "node:assert/strict";
import { access, mkdir, mkdtemp, readdir, rm, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createInstalledNativeRuntime, parseNativeRuntime, type NativeRuntimeDefinition } from "./miniapp-native-runtime.js";
import type { VerifiedMiniAppResource } from "./miniapp-window.js";

const definition: NativeRuntimeDefinition = { schema: "fabushi.native-runtime.v1", platform: "darwin", architecture: "arm64", entrypoint: "native/worker.cjs", executables: ["native/engine.bin"], hostCapabilities: ["confirmation"] };
const scope = { appId: "org.test.native", version: "1.0.0", packageDigest: "a".repeat(64), accountScope: "b".repeat(64), accountGeneration: 1 };
const carrierPath = resolve("source/electron-main/mcp/miniapp-native-carrier.ts");
function resources(worker = ""): Map<string, VerifiedMiniAppResource> {
  return new Map([
    ["native/worker.cjs", { mime: "text/javascript", bytes: Buffer.from(worker) }],
    ["native/engine.bin", { mime: "application/octet-stream", bytes: Buffer.from("fixture; not executed") }],
  ]);
}
test("native runtime declaration rejects traversal, unknown permissions and mismatched resources", () => {
  assert.deepEqual(parseNativeRuntime(definition, resources()), definition);
  for (const invalid of [
    { ...definition, entrypoint: "native/../worker.cjs" },
    { ...definition, platform: "linux" },
    { ...definition, architecture: "x64" },
    { ...definition, hostCapabilities: ["shell"] },
    { ...definition, executables: ["native/missing.bin"] },
    { ...definition, unknown: true },
  ]) assert.throws(() => parseNativeRuntime(invalid, resources()), /MINIAPP_POLICY_DENIED/);
});

test("native-code denial creates no runtime and starts no worker", { skip: process.platform !== "darwin" || process.arch !== "arm64" }, async () => {
  const root = await mkdtemp(join(tmpdir(), "miniapp-denied-"));
  try {
    await assert.rejects(createInstalledNativeRuntime({ carrierPath, codeRoot: root, resources: resources(), definition, scope, current: () => true, confirm: async () => false, pick: async () => null }), /MINIAPP_POLICY_DENIED/);
    assert.deepEqual(await readdir(root), []);
  } finally { await rm(root, { recursive: true, force: true }); }
});

test("installed worker is scope-bound, environment-sanitized and removed after termination", { skip: process.platform !== "darwin" || process.arch !== "arm64" }, async () => {
  const root = await mkdtemp(join(tmpdir(), "miniapp-owned-"));
  let current = true;
  const worker = `
const readline = require('node:readline');
readline.createInterface({input:process.stdin}).on('line', line => {
 const m=JSON.parse(line);
 process.stdout.write(JSON.stringify({kind:'result',id:m.id,result:{ok:true,data:{method:m.method,secret:process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE??null}}})+'\\n');
});`;
  const previous = process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE;
  process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE = "must-not-be-inherited";
  const runtime = await createInstalledNativeRuntime({ carrierPath, codeRoot: root, resources: resources(worker), definition, scope, current: () => current, confirm: async () => true, pick: async () => null });
  try {
    assert.deepEqual(await runtime.router.invoke(scope, "test.echo", {}), { ok: true, data: { method: "test.echo", secret: null } });
    assert.equal((await runtime.router.invoke({ ...scope, packageDigest: "c".repeat(64) }, "test.echo", {})).ok, false);
    current = false;
    assert.equal((await runtime.router.invoke(scope, "test.echo", {})).ok, false);
    runtime.cancelAll();
    await new Promise(resolve => setTimeout(resolve, 3500));
    await assert.rejects(access(runtime.directory));
    assert.equal((await runtime.router.invoke(scope, "test.echo", {})).ok, false);
  } finally {
    runtime.cancelAll();
    if (previous === undefined) delete process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE;
    else process.env.FABUSHI_CI_ACCOUNT_SESSION_FILE = previous;
    await rm(root, { recursive: true, force: true });
  }
});

test("Host pipe loss cleans temporary code and kills native descendants that ignore TERM", { skip: process.platform !== "darwin" || process.arch !== "arm64" }, async () => {
  const directory = await mkdtemp(join(tmpdir(), "runtime-crash-"));
  await mkdir(join(directory, "native"), { mode: 0o700 });
  const entrypoint = join(directory, "native", "worker.cjs");
  await writeFile(entrypoint, `
process.on('SIGTERM',()=>{});
const descendant=require('node:child_process').spawn(process.execPath,['-e',"process.on('SIGTERM',()=>{});setInterval(()=>{},1000)"],{stdio:'ignore'});
console.log(JSON.stringify({worker:process.pid,descendant:descendant.pid}));
setInterval(()=>{},1000);
`, { mode: 0o400 });
  const carrier = spawn(process.execPath, [carrierPath, entrypoint, directory], { detached: true, stdio: ["pipe", "pipe", "pipe"] });
  try {
    const observed = await new Promise<{ worker: number; descendant: number }>((resolveResult, reject) => {
      let input = "";
      const timer = setTimeout(() => reject(new Error("carrier fixture startup timeout")), 10000);
      carrier.stdout.on("data", chunk => { input += chunk.toString(); if (input.includes("\n")) { clearTimeout(timer); resolveResult(JSON.parse(input.split("\n")[0]!)); } });
      carrier.once("error", reject);
    });
    const exited = new Promise<void>(resolveExit => carrier.once("exit", () => resolveExit()));
    carrier.stdin.end();
    await Promise.race([exited, new Promise<never>((_resolve, reject) => setTimeout(() => reject(new Error("carrier did not settle on parent pipe loss")), 10000))]);
    await assert.rejects(access(directory));
    let remaining = [observed.worker, observed.descendant];
    for (let attempt = 0; attempt < 40 && remaining.length; attempt++) {
      remaining = remaining.filter(pid => { try { process.kill(pid, 0); return true; } catch { return false; } });
      if (remaining.length) await new Promise(resolveDelay => setTimeout(resolveDelay, 50));
    }
    assert.deepEqual(remaining, [], "no process in the native fixture group may survive Host loss");
  } finally {
    if (carrier.exitCode === null && carrier.signalCode === null && carrier.pid) { try { process.kill(-carrier.pid, "SIGKILL"); } catch { /* fixture exited */ } }
    await rm(directory, { recursive: true, force: true });
  }
});
