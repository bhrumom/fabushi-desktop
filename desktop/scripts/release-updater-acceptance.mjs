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

const executable = required("FABUSHI_UPDATER_BASELINE_EXECUTABLE");
const baselineVersion = required("FABUSHI_UPDATER_BASELINE_VERSION");
const targetVersion = required("FABUSHI_UPDATER_TARGET_VERSION");
const evidenceDir = process.env.FABUSHI_UPDATER_EVIDENCE_DIR?.trim()
  || path.join(tmpdir(), `fabushi-updater-acceptance-${Date.now()}`);
const userDataDir = process.env.FABUSHI_UPDATER_USER_DATA_DIR?.trim()
  || path.join(tmpdir(), `fabushi-updater-user-data-${Date.now()}`);
const timeoutMs = Number(process.env.FABUSHI_UPDATER_TIMEOUT_MS || 15 * 60_000);

function parseSemver(value) {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/u.exec(value);
  if (!match) throw new Error(`invalid semantic version: ${value}`);
  return { core: [Number(match[1]), Number(match[2]), Number(match[3])], prerelease: match[4]?.split(".") ?? [] };
}

function compareSemver(left, right) {
  const a = parseSemver(left);
  const b = parseSemver(right);
  for (let index = 0; index < 3; index += 1) {
    if (a.core[index] !== b.core[index]) return a.core[index] < b.core[index] ? -1 : 1;
  }
  if (a.prerelease.length === 0 || b.prerelease.length === 0) {
    if (a.prerelease.length === b.prerelease.length) return 0;
    return a.prerelease.length === 0 ? 1 : -1;
  }
  const length = Math.max(a.prerelease.length, b.prerelease.length);
  for (let index = 0; index < length; index += 1) {
    const av = a.prerelease[index], bv = b.prerelease[index];
    if (av === undefined) return -1;
    if (bv === undefined) return 1;
    if (av === bv) continue;
    const an = /^\d+$/u.test(av), bn = /^\d+$/u.test(bv);
    if (an && bn) return Number(av) < Number(bv) ? -1 : 1;
    if (an !== bn) return an ? -1 : 1;
    return av < bv ? -1 : 1;
  }
  return 0;
}

function shippingRenderer(url) {
  if (url.startsWith("app://bundle/")) return true;
  try {
    const parsed = new URL(url);
    return parsed.protocol === "file:" && parsed.pathname.endsWith("/dist/renderer/index.html");
  } catch {
    return false;
  }
}

async function sleep(ms) {
  await new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForRenderer() {
  const deadline = Date.now() + 120_000;
  let lastError = "";
  while (Date.now() < deadline) {
    try {
      const activePort = await readFile(path.join(userDataDir, "DevToolsActivePort"), "utf8");
      const port = Number(activePort.trim().split(/\r?\n/u)[0]);
      if (!Number.isInteger(port) || port <= 0 || port > 65535) throw new Error("invalid DevToolsActivePort");
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      if (!response.ok) throw new Error(`CDP registry returned HTTP ${response.status}`);
      const targets = await response.json();
      if (!Array.isArray(targets) || !targets.some((target) => target?.type === "page" && typeof target.url === "string" && shippingRenderer(target.url))) {
        throw new Error("shipping renderer target is not present");
      }
      const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 30_000 });
      const page = browser.contexts().flatMap((context) => context.pages()).find((candidate) => shippingRenderer(candidate.url()));
      if (!page) {
        await browser.close().catch(() => undefined);
        throw new Error("Playwright could not bind the shipping renderer");
      }
      return { browser, page, port };
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
      await sleep(250);
    }
  }
  throw new Error(`baseline did not expose the shipping renderer: ${lastError}`);
}

async function updaterStatus(page) {
  return await page.evaluate(async () => {
    const update = window.desktop?.update;
    if (typeof update?.getStatus !== "function") throw new Error("shipping updater preload bridge is unavailable");
    return await update.getStatus();
  });
}

async function main() {
  if (compareSemver(baselineVersion, targetVersion) >= 0) {
    throw new Error(`baseline version ${baselineVersion} must be strictly older than target ${targetVersion}`);
  }
  await rm(evidenceDir, { recursive: true, force: true });
  await rm(userDataDir, { recursive: true, force: true });
  await mkdir(evidenceDir, { recursive: true });
  await mkdir(userDataDir, { recursive: true });

  const child = spawn(executable, [
    "--remote-debugging-port=0",
    `--user-data-dir=${userDataDir}`,
  ], {
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      FABUSHI_UPDATE_ACCEPT_PRERELEASE: "1",
      SAND_DISABLE_UPDATES: "0",
    },
  });
  const logs = [];
  child.stdout?.on("data", (chunk) => logs.push({ at: Date.now(), stream: "stdout", text: String(chunk) }));
  child.stderr?.on("data", (chunk) => logs.push({ at: Date.now(), stream: "stderr", text: String(chunk) }));

  let browser;
  try {
    const binding = await waitForRenderer();
    browser = binding.browser;
    const initial = await updaterStatus(binding.page);
    if (initial?.currentVersion !== baselineVersion) {
      throw new Error(`baseline version mismatch: expected ${baselineVersion}, got ${String(initial?.currentVersion)}`);
    }
    if (initial?.state?.type === "disabled") {
      throw new Error(`shipping updater is disabled: ${String(initial?.state?.reason)}`);
    }

    const checked = await binding.page.evaluate(async () => {
      const update = window.desktop?.update;
      if (typeof update?.check !== "function") throw new Error("shipping updater check() is unavailable");
      return await update.check();
    });
    await writeFile(path.join(evidenceDir, "check-result.json"), JSON.stringify(checked, null, 2) + "\n");

    const deadline = Date.now() + timeoutMs;
    let staged = checked;
    while (Date.now() < deadline) {
      staged = await updaterStatus(binding.page);
      if (staged?.state?.type === "ready" && staged?.state?.version === targetVersion) break;
      if (staged?.state?.type === "disabled") throw new Error(`shipping updater became disabled: ${String(staged?.state?.reason)}`);
      if (staged?.state?.type === "idle" && staged?.state?.lastCheck?.result === "error") {
        throw new Error(`shipping updater failed: ${String(staged?.state?.lastCheck?.errorMessage || "unknown error")}`);
      }
      await sleep(500);
    }
    if (staged?.state?.type !== "ready" || staged?.state?.version !== targetVersion) {
      throw new Error(`shipping updater did not stage exact target ${targetVersion}: ${JSON.stringify(staged)}`);
    }
    await writeFile(path.join(evidenceDir, "staged.json"), JSON.stringify(staged, null, 2) + "\n");

    await binding.page.evaluate(async () => {
      const update = window.desktop?.update;
      if (typeof update?.quitAndInstall !== "function") throw new Error("shipping updater quitAndInstall() is unavailable");
      await update.quitAndInstall();
    });

    const exit = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("baseline process did not exit after shipping updater apply request")), 120_000);
      child.once("exit", (code, signal) => {
        clearTimeout(timer);
        resolve({ code, signal });
      });
    });
    await writeFile(path.join(evidenceDir, "apply-request.json"), JSON.stringify({
      baselineVersion,
      targetVersion,
      executable,
      cdpPort: binding.port,
      exit,
    }, null, 2) + "\n");
  } finally {
    await browser?.close().catch(() => undefined);
    if (child.exitCode == null && child.signalCode == null) child.kill("SIGTERM");
    await writeFile(path.join(evidenceDir, "baseline-runtime.log"), logs.map((entry) => JSON.stringify(entry)).join("\n") + (logs.length ? "\n" : ""));
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack || error.message : String(error));
  process.exitCode = 1;
});
