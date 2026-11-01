import { lstat, readFile } from "node:fs/promises";
import { resolve } from "node:path";

const MAX_DISCOVERY_BYTES = 64 * 1024;
const MAX_RESPONSE_BYTES = 4 * 1024 * 1024;
const DEFAULT_TIMEOUT_MS = 35_000;
const MAX_TIMEOUT_MS = 40_000;
const OPERATIONS = new Set(["status", "snapshot", "find", "action", "wait", "assert"]);
const TOKEN = /^[A-Za-z0-9_-]{48,256}$/u;

export class AppAgentSurfaceUnavailableError extends Error {
  constructor(message = "Fabushi App Agent Surface is unavailable.") {
    super(message);
    this.name = "AppAgentSurfaceUnavailableError";
    this.code = "app_surface_unavailable";
  }
}

function cleanDiscoveryPath(value) {
  const configured = String(value || "").trim();
  return configured ? resolve(configured) : "";
}

function loopbackOrigin(value) {
  const url = new URL(String(value || ""));
  const hostname = url.hostname.toLowerCase();
  const loopback = ["127.0.0.1", "localhost", "::1", "[::1]"].includes(hostname);
  if (url.protocol !== "http:" || !loopback || url.username || url.password || url.search || url.hash || url.pathname !== "/") {
    throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface discovery origin is not loopback HTTP.");
  }
  return url.origin;
}

async function readBoundedResponse(response) {
  const advertised = Number(response.headers.get("content-length") || 0);
  if (Number.isFinite(advertised) && advertised > MAX_RESPONSE_BYTES) {
    throw new Error("Fabushi App Agent Surface response is too large.");
  }
  const text = await response.text();
  if (Buffer.byteLength(text) > MAX_RESPONSE_BYTES) throw new Error("Fabushi App Agent Surface response is too large.");
  let payload = {};
  try { payload = text ? JSON.parse(text) : {}; }
  catch { throw new Error("Fabushi App Agent Surface returned invalid JSON."); }
  if (!response.ok || payload?.ok === false) {
    const message = String(payload?.error || `App Agent Surface HTTP ${response.status}`).slice(0, 1000);
    if ([401, 403, 404, 503, 504].includes(response.status) || /unavailable|timeout|unauthorized|policy_denied/iu.test(message)) {
      throw new AppAgentSurfaceUnavailableError(message);
    }
    throw new Error(message);
  }
  return payload?.result ?? payload;
}

export function createAppAgentSurfaceClient(options = {}) {
  const discoveryPath = cleanDiscoveryPath(options.discoveryPath ?? process.env.FABUSHI_APP_AGENT_DISCOVERY_FILE);
  const fetchImpl = options.fetchImpl ?? globalThis.fetch;
  const timeoutMs = Math.max(1_000, Math.min(Number(options.timeoutMs ?? DEFAULT_TIMEOUT_MS), MAX_TIMEOUT_MS));
  if (typeof fetchImpl !== "function") throw new TypeError("A fetch implementation is required.");

  async function discovery() {
    if (!discoveryPath) throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface discovery is not configured.");
    let metadata;
    try { metadata = await lstat(discoveryPath); }
    catch { throw new AppAgentSurfaceUnavailableError("Fabushi desktop is not currently publishing an App Agent Surface."); }
    if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.size <= 0 || metadata.size > MAX_DISCOVERY_BYTES) {
      throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface discovery file is invalid.");
    }
    if (process.platform !== "win32" && (metadata.mode & 0o077) !== 0) {
      throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface discovery file is not private.");
    }
    let payload;
    try { payload = JSON.parse(await readFile(discoveryPath, "utf8")); }
    catch { throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface discovery file is unreadable."); }
    const token = String(payload?.token || "");
    const appId = String(payload?.appId || "");
    const pid = Number(payload?.pid || 0);
    if (payload?.version !== 1 || appId !== "fabushi.desktop" || !TOKEN.test(token) || !Number.isSafeInteger(pid) || pid <= 0) {
      throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface discovery contract is invalid.");
    }
    return { origin: loopbackOrigin(payload.origin), token, appId, pid };
  }

  async function call(operation, input = {}, callOptions = {}) {
    if (!OPERATIONS.has(operation)) throw new Error(`Unsupported App Agent Surface operation: ${operation}`);
    if (!input || typeof input !== "object" || Array.isArray(input)) throw new TypeError("App Agent Surface input must be an object.");
    const current = await discovery();
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    timer.unref?.();
    const abort = () => controller.abort(callOptions.signal?.reason);
    callOptions.signal?.addEventListener?.("abort", abort, { once: true });
    try {
      const response = await fetchImpl(`${current.origin}/v1/${operation}`, {
        method: "POST",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
          Authorization: `Bearer ${current.token}`,
        },
        body: JSON.stringify({ input }),
        cache: "no-store",
        redirect: "error",
        referrerPolicy: "no-referrer",
        signal: controller.signal,
      });
      return await readBoundedResponse(response);
    } catch (error) {
      if (error?.name === "AbortError") throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface request timed out.");
      if (error instanceof AppAgentSurfaceUnavailableError) throw error;
      if (error instanceof TypeError) throw new AppAgentSurfaceUnavailableError("Fabushi App Agent Surface loopback connection failed.");
      throw error;
    } finally {
      clearTimeout(timer);
      callOptions.signal?.removeEventListener?.("abort", abort);
    }
  }

  async function status() {
    try { return await call("status", {}); }
    catch (error) {
      if (!(error instanceof AppAgentSurfaceUnavailableError)) throw error;
      return {
        version: 1,
        appId: "fabushi.desktop",
        available: false,
        reason: error.message,
      };
    }
  }

  return Object.freeze({ discoveryPath, discovery, call, status });
}
