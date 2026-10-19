import { createHash } from "node:crypto";

export const DandelionCandidate = Object.freeze({
  namespace: "bhrumom", name: "global-dandelion", version: "0.2.1",
  digest: "afc45ad273e4965fc272a89e775291444c683513ecbc3fa31769858431e8a6b6",
  appId: "org.fabushi.global-dandelion",
});
const API = "https://api.ombhrum.com";
const SAFE_ERRORS = new Set([
  "PUBLISHER_ACCOUNT_CHANGED", "PUBLISHER_ACCOUNT_REQUIRED", "PUBLISHER_BUSY",
  "PUBLISHER_CLOSED", "PUBLISHER_FILE_DIALOG_UNAVAILABLE", "PUBLISHER_PACKAGE_CHANGED",
  "PUBLISHER_PACKAGE_LIMIT", "PUBLISHER_IMMUTABLE_CONFLICT", "PUBLISHER_INVALID_RESPONSE",
  "PUBLISHER_INVALID_REVIEW_STATE", "PUBLISHER_NAMESPACE_REQUIRED",
  "PUBLISHER_NETWORK_UNCERTAIN_NO_RETRY", "PUBLISHER_OPERATION_FAILED",
  "PUBLISHER_RELEASE_FROZEN", "PUBLISHER_RESPONSE_LIMIT", "PUBLISHER_SENDER_DENIED",
  "PUBLISHER_UNREVIEWED_PACKAGE",
]);
type Handler = (event: unknown) => Promise<unknown>;
export interface MiniAppPublisherPorts {
  ipc: { handle(channel: string, handler: Handler): void };
  trustedSender(): unknown;
  accessToken(): Promise<string | null>;
  selectPackage(): Promise<Uint8Array | null>;
  fetch: typeof globalThis.fetch;
}
function trusted(ports: MiniAppPublisherPorts, event: unknown): void {
  const sender = typeof event === "object" && event != null ? Reflect.get(event, "sender") : undefined;
  if (sender == null || sender !== ports.trustedSender()) throw new Error("PUBLISHER_SENDER_DENIED");
}
export function registerMiniAppPublisher(ports: MiniAppPublisherPorts): { dispose(): void } {
  let disposed = false;
  let publishing = false;
  const handle = (name: string, handler: Handler) => ports.ipc.handle(name, async (event) => {
    try { return await handler(event); }
    catch (error) {
      const message = error instanceof Error ? error.message : "";
      throw new Error(SAFE_ERRORS.has(message) || /^PUBLISHER_HTTP_[1-5][0-9]{2}$/.test(message) ? message : "PUBLISHER_OPERATION_FAILED");
    }
  });
  const begin = async (event: unknown) => {
    if (disposed) throw new Error("PUBLISHER_CLOSED");
    trusted(ports, event);
    const token = await ports.accessToken();
    if (!token) throw new Error("PUBLISHER_ACCOUNT_REQUIRED");
    const fence = async () => {
      if (disposed || await ports.accessToken() !== token) throw new Error("PUBLISHER_ACCOUNT_CHANGED");
    };
    const request = async (path: string, method = "GET", body?: unknown, headers: Record<string,string> = {}) => {
      await fence();
      let response: Response;
      try {
        response = await ports.fetch(API + path, {
          method, redirect: "manual", signal: AbortSignal.timeout(30_000),
          credentials: "omit", cache: "no-store", referrerPolicy: "no-referrer",
          headers: { Authorization: `Bearer ${token}`, ...headers },
          ...(body === undefined ? {} : { body: body instanceof Uint8Array ? new Uint8Array(body) : JSON.stringify(body) }),
        });
      } catch { throw new Error("PUBLISHER_NETWORK_UNCERTAIN_NO_RETRY"); }
      if (!response.ok) throw new Error(`PUBLISHER_HTTP_${response.status}`);
      const reader = response.body?.getReader();
      if (!reader) throw new Error("PUBLISHER_INVALID_RESPONSE");
      let text = "";
      let count = 0;
      const decoder = new TextDecoder();
      for (;;) {
        const part = await reader.read();
        if (part.done) break;
        count += part.value.length;
        if (count > 1024 * 1024) { await reader.cancel(); throw new Error("PUBLISHER_RESPONSE_LIMIT"); }
        text += decoder.decode(part.value, { stream: true });
      }
      text += decoder.decode();
      await fence();
      try { return JSON.parse(text); } catch { throw new Error("PUBLISHER_INVALID_RESPONSE"); }
    };
    return { request, fence };
  };
  handle("sand:marketplace-publisher-owned", async (event) => {
    const { request } = await begin(event);
    const data = await request("/v1/extensions/publisher/namespaces");
    return { namespaces: (Array.isArray(data.namespaces) ? data.namespaces : [])
      .map((item: { namespace?: unknown }) => item.namespace)
      .filter((name: unknown) => typeof name === "string" && /^[a-z0-9][a-z0-9._-]{0,127}$/.test(name)) };
  });
  handle("sand:marketplace-publisher-claim", async (event) => {
    const { request } = await begin(event);
    await request("/v1/extensions/publisher/namespaces", "POST",
      { namespace: DandelionCandidate.namespace, displayName: "bhrumom" }, { "Content-Type": "application/json" });
    return { namespace: DandelionCandidate.namespace };
  });
  handle("sand:marketplace-submit-dandelion", async (event) => {
    if (publishing) throw new Error("PUBLISHER_BUSY");
    publishing = true;
    try {
      const { request, fence } = await begin(event);
      const bytes = await ports.selectPackage();
      await fence();
      if (bytes == null) return { state: "cancelled" };
      if (bytes.length === 0 || bytes.length > 25 * 1024 * 1024) throw new Error("PUBLISHER_PACKAGE_LIMIT");
      if (createHash("sha256").update(bytes).digest("hex") !== DandelionCandidate.digest) throw new Error("PUBLISHER_UNREVIEWED_PACKAGE");
      const owned = await request("/v1/extensions/publisher/namespaces");
      if (!owned.namespaces?.some((item: { namespace: string }) => item.namespace === DandelionCandidate.namespace)) throw new Error("PUBLISHER_NAMESPACE_REQUIRED");
      const listed = await request("/v1/extensions/publisher/releases");
      const existing = listed.releases?.find((item: { namespace: string; name: string; kind: string; version: string }) =>
        item.namespace === DandelionCandidate.namespace && item.name === DandelionCandidate.name && item.kind === "web-miniapp" && item.version === DandelionCandidate.version);
      if (existing && existing.descriptor?.release?.bundleDigest !== DandelionCandidate.digest) throw new Error("PUBLISHER_IMMUTABLE_CONFLICT");
      if (existing?.state === "approved" || existing?.state === "pending_review") return { state: existing.state, digest: DandelionCandidate.digest };
      if (existing && existing.state !== "draft") throw new Error("PUBLISHER_RELEASE_FROZEN");
      if (!existing) await request("/v1/extensions/publisher/releases", "POST", { descriptor: {
        identity: { namespace: DandelionCandidate.namespace, name: DandelionCandidate.name, kind: "web-miniapp" },
        origin: "public", trustDomain: "catalog-authorized", displayName: "Global Dandelion 离线工作台",
        description: "复用原生 Rust GDAN v2/BLAKE3 的离线文件分块与逐包完整性校验工作台。",
        publisher: { id: DandelionCandidate.namespace, displayName: "bhrumom", verified: false },
        release: { version: DandelionCandidate.version, channel: "preview", platforms: ["android", "desktop", "ios"],
          requestedCapabilities: [], minHostApi: 1, bundleDigest: DandelionCandidate.digest },
        categories: ["developer-tools", "utilities"], tags: ["cross-platform", "file-integrity", "offline", "rust", "wasm"],
        runtime: { owner: "host-plugins", installMode: "fabapp", appId: DandelionCandidate.appId,
          publisherId: DandelionCandidate.namespace, offlineCore: true, license: "MIT OR Apache-2.0",
          nativeTransferAvailable: false, networkPolicy: { mode: "deny", allowedOrigins: [] } },
      } }, { "Content-Type": "application/json" });
      await request("/v1/extensions/publisher/artifacts", "POST", bytes, {
        "Content-Type": "application/octet-stream", "x-fabushi-extension-kind": "web-miniapp",
        "x-fabushi-extension-namespace": DandelionCandidate.namespace, "x-fabushi-extension-name": DandelionCandidate.name,
        "x-fabushi-extension-version": DandelionCandidate.version, "x-fabushi-artifact-id": "desktop-fabapp",
        "x-fabushi-artifact-platform": "desktop", "x-fabushi-artifact-architecture": "any", "x-fabushi-artifact-format": "fabapp",
      });
      const submitted = await request(`/v1/extensions/publisher/releases/web-miniapp/${DandelionCandidate.namespace}/${DandelionCandidate.name}/${DandelionCandidate.version}/submit`, "POST", {}, { "Content-Type": "application/json" });
      if (submitted.release?.state !== "pending_review") throw new Error("PUBLISHER_INVALID_REVIEW_STATE");
      return { state: "pending_review", digest: DandelionCandidate.digest };
    } finally { publishing = false; }
  });
  return { dispose() { disposed = true; } };
}
