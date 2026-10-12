import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

function required(name) {
  const value = process.env[name]?.trim();
  if (!value) throw new Error(`${name} is required`);
  return value;
}
const executable = required("FABUSHI_RELEASE_IDENTITY_EXECUTABLE");
const expectedVersion = required("FABUSHI_RELEASE_IDENTITY_VERSION");
const expectedSourceSha = required("FABUSHI_RELEASE_IDENTITY_SOURCE_SHA");
if (!/^[0-9a-f]{40}$/u.test(expectedSourceSha)) throw new Error("FABUSHI_RELEASE_IDENTITY_SOURCE_SHA must be a lowercase full SHA");
const evidenceDir = process.env.FABUSHI_RELEASE_IDENTITY_EVIDENCE_DIR?.trim()
  || path.join(tmpdir(), `fabushi-release-identity-${Date.now()}`);
const userDataDir = process.env.FABUSHI_RELEASE_IDENTITY_USER_DATA_DIR?.trim()
  || path.join(tmpdir(), `fabushi-release-identity-user-data-${Date.now()}`);

function shippingRenderer(url) {
  if (url.startsWith("app://bundle/")) return true;
  try {
    const parsed = new URL(url);
    return parsed.protocol === "file:" && parsed.pathname.endsWith("/dist/renderer/index.html");
  } catch {
    return false;
  }
}
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function waitForPage() {
  const deadline = Date.now() + 120_000;
  let last = "";
  while (Date.now() < deadline) {
    try {
      const active = await readFile(path.join(userDataDir, "DevToolsActivePort"), "utf8");
      const port = Number(active.trim().split(/\r?\n/u)[0]);
      if (!Number.isInteger(port) || port <= 0 || port > 65535) throw new Error("invalid DevToolsActivePort");
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      if (!response.ok) throw new Error(`CDP registry HTTP ${response.status}`);
      const targets = await response.json();
      if (!Array.isArray(targets) || !targets.some((target) => target?.type === "page" && shippingRenderer(target.url || ""))) {
        throw new Error("shipping renderer target absent");
      }
      const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 30_000 });
      const page = browser.contexts().flatMap((context) => context.pages()).find((candidate) => shippingRenderer(candidate.url()));
      if (!page) {
        await browser.close().catch(() => undefined);
        throw new Error("shipping page absent after CDP attach");
      }
      return { browser, page, port };
    } catch (error) {
      last = error instanceof Error ? error.message : String(error);
      await sleep(250);
    }
  }
  throw new Error(`updated candidate did not expose shipping renderer: ${last}`);
}

async function main() {
  await rm(evidenceDir, { recursive: true, force: true });
  await rm(userDataDir, { recursive: true, force: true });
  await mkdir(evidenceDir, { recursive: true });
  await mkdir(userDataDir, { recursive: true });
  const child = spawn(executable, ["--remote-debugging-port=0", `--user-data-dir=${userDataDir}`], {
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, SAND_DISABLE_UPDATES: "0" },
  });
  const logs = [];
  child.stdout?.on("data", (chunk) => logs.push({ at: Date.now(), stream: "stdout", text: String(chunk) }));
  child.stderr?.on("data", (chunk) => logs.push({ at: Date.now(), stream: "stderr", text: String(chunk) }));
  let browser;
  try {
    const binding = await waitForPage();
    browser = binding.browser;
    const identity = await binding.page.evaluate(async () => {
      const update = window.desktop?.update;
      if (typeof update?.getStatus !== "function") throw new Error("shipping updater identity bridge unavailable");
      const status = await update.getStatus();
      return {
        currentVersion: status?.currentVersion ?? null,
        currentSourceSha: status?.currentSourceSha ?? null,
        state: status?.state ?? null,
      };
    });
    if (identity.currentVersion !== expectedVersion) {
      throw new Error(`updated candidate version mismatch: expected ${expectedVersion}, got ${String(identity.currentVersion)}`);
    }
    if (identity.currentSourceSha !== expectedSourceSha) {
      throw new Error(`updated candidate source mismatch: expected ${expectedSourceSha}, got ${String(identity.currentSourceSha)}`);
    }
    await writeFile(path.join(evidenceDir, "release-identity.json"), JSON.stringify({
      ...identity,
      executable,
      cdpPort: binding.port,
    }, null, 2) + "\n");
  } finally {
    await browser?.close().catch(() => undefined);
    if (child.exitCode == null && child.signalCode == null) child.kill("SIGTERM");
    await writeFile(path.join(evidenceDir, "runtime.log"), logs.map((entry) => JSON.stringify(entry)).join("\n") + (logs.length ? "\n" : ""));
  }
}
main().catch((error) => {
  console.error(error instanceof Error ? error.stack || error.message : String(error));
  process.exitCode = 1;
});
