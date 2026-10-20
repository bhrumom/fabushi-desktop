import test from "node:test";
import assert from "node:assert/strict";
import { access, mkdtemp, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createInstalledNativeRuntime, parseNativeRuntime, type NativeRuntimeDefinition } from "./miniapp-native-runtime.js";
import type { VerifiedMiniAppResource } from "./miniapp-window.js";

const definition: NativeRuntimeDefinition = { schema: "fabushi.native-runtime.v1", platform: "darwin", architecture: "arm64", entrypoint: "native/worker.cjs", executables: ["native/engine.bin"], hostCapabilities: ["confirmation"] };
const scope = { appId: "org.test.native", version: "1.0.0", packageDigest: "a".repeat(64), accountScope: "b".repeat(64), accountGeneration: 1 };
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
    await assert.rejects(createInstalledNativeRuntime({ codeRoot: root, resources: resources(), definition, scope, current: () => true, confirm: async () => false, pick: async () => null }), /MINIAPP_POLICY_DENIED/);
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
  const runtime = await createInstalledNativeRuntime({ codeRoot: root, resources: resources(worker), definition, scope, current: () => current, confirm: async () => true, pick: async () => null });
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
