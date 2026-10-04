import { randomUUID } from "node:crypto";
import { promises as fs } from "node:fs";
import { dirname } from "node:path";

import type { AuthServicePort } from "../../electron-main/account/cursor-auth-wiring.js";
import type { SandAuthStatus } from "../../electron-main/account/cursor-auth.js";
import { deleteSecret, readSecret, writeSecret } from "../../electron-main/secrets/secret-store.js";

export const FABUSHI_ACCOUNT_SESSION_SECRET = "fabushi-account-session-v1";
export const FABUSHI_CI_ACCOUNT_SESSION_FILE_ENV = "FABUSHI_CI_ACCOUNT_SESSION_FILE";
export const FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV = "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE";
export const DEFAULT_FABUSHI_API_BASE_URL = "https://api.ombhrum.com";
const REFRESH_LEEWAY_MS = 60_000;
const MIN_POLL_MS = 250;
const MAX_POLL_MS = 5_000;

interface FabushiSession {
  readonly accessToken: string;
  readonly refreshToken?: string;
  readonly accessTokenExpiresAt: number;
  readonly refreshTokenExpiresAt?: number;
  readonly sessionId: string;
  readonly deviceId: string;
  readonly username: string;
  readonly userId: string | number;
  readonly user?: Readonly<Record<string, unknown>>;
  readonly provider?: string;
  readonly ciRunner?: boolean;
}

interface BrowserAttempt {
  readonly attemptId: string;
  readonly loginUrl: string;
  readonly pollSecret: string;
  readonly expiresAt: number;
  readonly pollAfterMs: number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function boundedText(value: unknown, max: number): string | undefined {
  if (typeof value !== "string") return undefined;
  const trimmed = value.trim();
  return trimmed.length > 0 && trimmed.length <= max ? trimmed : undefined;
}

function normalizeExpiryMs(value: unknown): number | undefined {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) return undefined;
  return value < 10_000_000_000 ? Math.trunc(value * 1_000) : Math.trunc(value);
}

export function normalizeFabushiApiBaseUrl(raw: string | undefined): string {
  const candidate = (raw ?? DEFAULT_FABUSHI_API_BASE_URL).trim().replace(/\/+$/, "");
  const url = new URL(candidate);
  const loopback = ["localhost", "127.0.0.1", "::1", "[::1]"].includes(url.hostname);
  if (url.protocol !== "https:" && !(loopback && url.protocol === "http:")) {
    throw new Error("Fabushi account API must use HTTPS outside loopback development.");
  }
  if (url.username || url.password || url.search || url.hash) {
    throw new Error("Fabushi account API base URL must not contain credentials, query, or fragment.");
  }
  return url.toString().replace(/\/+$/, "");
}

export function normalizeFabushiSession(value: unknown, options: { readonly allowRefreshless?: boolean } = {}): FabushiSession | null {
  if (!isRecord(value)) return null;
  const accessToken = boundedText(value.accessToken, 16_384);
  const refreshToken = boundedText(value.refreshToken, 16_384);
  const accessTokenExpiresAt = normalizeExpiryMs(value.accessTokenExpiresAt ?? value.expiresAtMs);
  const refreshTokenExpiresAt = normalizeExpiryMs(value.refreshTokenExpiresAt);
  const sessionId = boundedText(value.sessionId, 200);
  const deviceId = boundedText(value.deviceId, 200);
  const username = boundedText(value.username, 320);
  const userId = typeof value.userId === "string" && value.userId.length > 0 && value.userId.length <= 200
    ? value.userId
    : typeof value.userId === "number" && Number.isFinite(value.userId)
      ? value.userId
      : undefined;
  if (accessToken == null || accessTokenExpiresAt == null || sessionId == null || deviceId == null || username == null || userId == null) return null;
  if (refreshToken == null && options.allowRefreshless !== true) return null;
  if (refreshToken != null && refreshTokenExpiresAt == null) return null;
  const user = isRecord(value.user) ? value.user : undefined;
  const provider = boundedText(value.provider, 100);
  const ciRunner = typeof value.ciRunner === "boolean" ? value.ciRunner : undefined;
  return {
    accessToken,
    ...(refreshToken == null ? {} : { refreshToken }),
    accessTokenExpiresAt,
    ...(refreshTokenExpiresAt == null ? {} : { refreshTokenExpiresAt }),
    sessionId,
    deviceId,
    username,
    userId,
    ...(user == null ? {} : { user }),
    ...(provider == null ? {} : { provider }),
    ...(ciRunner == null ? {} : { ciRunner }),
  };
}

function statusFromSession(session: FabushiSession): SandAuthStatus {
  const user = session.user;
  const email = boundedText(user?.email, 320) ?? (session.username.includes("@") ? session.username : undefined);
  const displayName = boundedText(user?.displayName, 200)
    ?? boundedText(user?.name, 200)
    ?? boundedText(user?.username, 200)
    ?? session.username;
  const profilePictureUrl = boundedText(user?.profilePictureUrl, 2048)
    ?? boundedText(user?.avatarUrl, 2048)
    ?? boundedText(user?.avatar, 2048);
  return {
    kind: "logged-in",
    authId: String(session.userId),
    ...(email == null ? {} : { email }),
    expiresAt: session.accessTokenExpiresAt,
    displayName,
    ...(profilePictureUrl == null ? {} : { profilePictureUrl }),
    isAnysphereUser: false,
  };
}

async function delay(ms: number, signal?: AbortSignal): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    if (signal?.aborted) {
      reject(signal.reason ?? new Error("Fabushi sign-in cancelled."));
      return;
    }
    const timer = setTimeout(resolve, ms);
    timer.unref?.();
    signal?.addEventListener("abort", () => {
      clearTimeout(timer);
      reject(signal.reason ?? new Error("Fabushi sign-in cancelled."));
    }, { once: true });
  });
}

export interface FabushiAuthServiceOptions {
  readonly openExternal: (url: string) => void | Promise<void>;
  readonly env: NodeJS.ProcessEnv;
  readonly machineId: string;
  readonly hostAccessCredentialFile: string;
  readonly fetchImpl?: typeof fetch;
  readonly now?: () => number;
}

export class FabushiAuthService implements AuthServicePort {
  private readonly listeners = new Set<(status: SandAuthStatus) => void>();
  private readonly baseUrl: string;
  private readonly fetchImpl: typeof fetch;
  private readonly now: () => number;
  private loginAbort: AbortController | undefined;
  private loginAttempt: BrowserAttempt | undefined;
  private operationEpoch = 0;
  private ciSessionPromise: Promise<FabushiSession | null> | undefined;
  private refreshPromise: Promise<FabushiSession | null> | undefined;

  constructor(private readonly options: FabushiAuthServiceOptions) {
    this.baseUrl = normalizeFabushiApiBaseUrl(options.env.FABUSHI_API_BASE_URL);
    this.fetchImpl = options.fetchImpl ?? fetch;
    this.now = options.now ?? Date.now;
  }

  subscribe(listener: (status: SandAuthStatus) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private emit(status: SandAuthStatus): void {
    for (const listener of this.listeners) listener(status);
  }

  private async requestJson(path: string, init: RequestInit = {}): Promise<unknown> {
    const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
      ...init,
      headers: {
        Accept: "application/json",
        ...(init.body == null ? {} : { "Content-Type": "application/json" }),
        ...(init.headers ?? {}),
      },
      cache: "no-store",
      credentials: "omit",
      redirect: "error",
      referrerPolicy: "no-referrer",
    });
    const raw = await response.text();
    let body: unknown = null;
    if (raw.trim().length > 0) {
      try { body = JSON.parse(raw); } catch { body = null; }
    }
    if (!response.ok) {
      const code = isRecord(body) && isRecord(body.error) ? boundedText(body.error.code, 120) : undefined;
      throw new Error(`Fabushi account request failed (${response.status}${code == null ? "" : ` ${code}`}).`);
    }
    return body;
  }

  private async readCiSession(): Promise<FabushiSession | null> {
    const source = this.options.env[FABUSHI_CI_ACCOUNT_SESSION_FILE_ENV]?.trim();
    if (!source) return null;
    this.ciSessionPromise ??= fs.readFile(source, "utf8").then((raw) => {
      const parsed: unknown = JSON.parse(raw);
      const session = normalizeFabushiSession(parsed, { allowRefreshless: true });
      if (session == null
        || session.provider !== "github-actions"
        || session.ciRunner !== true
        || session.refreshToken != null
        || !session.sessionId.startsWith("ci-runner:")) {
        throw new Error("Fabushi CI account session failed bounded-session validation.");
      }
      return session;
    });
    return await this.ciSessionPromise;
  }

  private async readStoredSession(): Promise<FabushiSession | null> {
    const raw = await readSecret(FABUSHI_ACCOUNT_SESSION_SECRET);
    if (raw == null) return null;
    try {
      return normalizeFabushiSession(JSON.parse(raw));
    } catch {
      return null;
    }
  }

  private async saveStoredSession(session: FabushiSession): Promise<void> {
    await writeSecret(FABUSHI_ACCOUNT_SESSION_SECRET, JSON.stringify(session));
  }

  private async clearStoredSession(): Promise<void> {
    await deleteSecret(FABUSHI_ACCOUNT_SESSION_SECRET);
  }

  private async writeHostCredential(session: FabushiSession): Promise<void> {
    const path = this.options.hostAccessCredentialFile;
    await fs.mkdir(dirname(path), { recursive: true, mode: 0o700 });
    const temporary = `${path}.${process.pid}.${randomUUID()}.tmp`;
    await fs.writeFile(temporary, JSON.stringify({
      version: 1,
      accessToken: session.accessToken,
      expiresAtMs: session.accessTokenExpiresAt,
      sessionId: session.sessionId,
      deviceId: session.deviceId,
      userId: session.userId,
    }), { encoding: "utf8", mode: 0o600, flag: "wx" });
    try {
      await fs.rename(temporary, path);
      await fs.chmod(path, 0o600).catch(() => {});
    } catch (error) {
      await fs.rm(temporary, { force: true }).catch(() => {});
      throw error;
    }
  }

  private async clearHostCredential(): Promise<void> {
    await fs.rm(this.options.hostAccessCredentialFile, { force: true }).catch(() => {});
  }

  private async resolveIdentity(session: FabushiSession): Promise<FabushiSession> {
    const body = await this.requestJson("/api/auth/user-info", {
      headers: { Authorization: `Bearer ${session.accessToken}` },
    });
    if (!isRecord(body)) throw new Error("Fabushi account identity response is invalid.");
    const userId = typeof body.id === "string" || typeof body.id === "number" ? body.id : session.userId;
    const username = boundedText(body.username, 320) ?? boundedText(body.email, 320) ?? session.username;
    if (String(userId) !== String(session.userId)) {
      throw new Error("Fabushi account identity changed while settling the session.");
    }
    return { ...session, username, user: body };
  }

  private async refreshSession(session: FabushiSession): Promise<FabushiSession | null> {
    if (session.refreshToken == null) return null;
    if (this.refreshPromise != null) return await this.refreshPromise;
    this.refreshPromise = (async () => {
      const raw = await this.requestJson("/api/auth/refresh", {
        method: "POST",
        body: JSON.stringify({ refreshToken: session.refreshToken, deviceId: session.deviceId }),
      });
      const next = normalizeFabushiSession(raw);
      if (next == null
        || next.deviceId !== session.deviceId
        || String(next.userId) !== String(session.userId)) {
        throw new Error("Fabushi refreshed account session failed identity validation.");
      }
      const verified = await this.resolveIdentity(next);
      await this.saveStoredSession(verified);
      await this.writeHostCredential(verified);
      return verified;
    })().catch(async (error) => {
      await this.clearStoredSession().catch(() => {});
      await this.clearHostCredential();
      throw error;
    }).finally(() => {
      this.refreshPromise = undefined;
    });
    return await this.refreshPromise;
  }

  private async currentSession(): Promise<FabushiSession | null> {
    const ci = await this.readCiSession();
    let session = ci ?? await this.readStoredSession();
    if (session == null) {
      await this.clearHostCredential();
      return null;
    }
    if (session.accessTokenExpiresAt <= this.now() + REFRESH_LEEWAY_MS) {
      if (ci != null || session.refreshToken == null) {
        await this.clearHostCredential();
        return null;
      }
      session = await this.refreshSession(session);
      if (session == null) return null;
    }
    await this.writeHostCredential(session);
    return session;
  }

  async getStatus(): Promise<SandAuthStatus> {
    try {
      const session = await this.currentSession();
      return session == null ? { kind: "logged-out" } : statusFromSession(session);
    } catch (error) {
      return { kind: "logged-out", errorMessage: error instanceof Error ? error.message : "Fabushi sign-in is unavailable." };
    }
  }

  async getValidAccessToken(options?: { readonly backendUrl?: string }): Promise<string> {
    if (options?.backendUrl != null) {
      const requested = normalizeFabushiApiBaseUrl(options.backendUrl);
      if (new URL(requested).origin !== new URL(this.baseUrl).origin) {
        throw new Error("Refusing to send a Fabushi account token to a non-Fabushi backend.");
      }
    }
    const session = await this.currentSession();
    if (session == null) throw new Error("Sign in to Fabushi to continue.");
    return session.accessToken;
  }

  async peekAccessToken(): Promise<string | null> {
    try {
      return (await this.currentSession())?.accessToken ?? null;
    } catch {
      return null;
    }
  }

  private parseBrowserAttempt(value: unknown): BrowserAttempt {
    if (!isRecord(value)) throw new Error("Fabushi browser sign-in start response is invalid.");
    const attemptId = boundedText(value.attemptId, 200);
    const loginUrl = boundedText(value.loginUrl, 4096);
    const pollSecret = boundedText(value.pollSecret, 4096);
    const expiresAt = normalizeExpiryMs(value.expiresAt);
    const pollAfterMs = typeof value.pollAfterMs === "number" && Number.isFinite(value.pollAfterMs)
      ? Math.min(MAX_POLL_MS, Math.max(MIN_POLL_MS, Math.trunc(value.pollAfterMs)))
      : 750;
    if (attemptId == null || loginUrl == null || pollSecret == null || expiresAt == null) {
      throw new Error("Fabushi browser sign-in start response is incomplete.");
    }
    const login = new URL(loginUrl);
    if (login.origin !== new URL(this.baseUrl).origin) {
      throw new Error("Fabushi browser sign-in returned an untrusted login origin.");
    }
    return { attemptId, loginUrl, pollSecret, expiresAt, pollAfterMs };
  }

  async login(): Promise<SandAuthStatus> {
    this.loginAbort?.abort();
    const epoch = ++this.operationEpoch;
    const controller = new AbortController();
    this.loginAbort = controller;
    this.emit({ kind: "logging-in" });
    try {
      const start = this.parseBrowserAttempt(await this.requestJson("/api/auth/browser/start", {
        method: "POST",
        body: JSON.stringify({ deviceId: this.options.machineId, platform: "desktop" }),
        signal: controller.signal,
      }));
      this.loginAttempt = start;
      await this.options.openExternal(start.loginUrl);
      while (!controller.signal.aborted && epoch === this.operationEpoch && this.now() < start.expiresAt) {
        const result = await this.requestJson(`/api/auth/browser/attempts/${encodeURIComponent(start.attemptId)}`, {
          method: "POST",
          body: JSON.stringify({ pollSecret: start.pollSecret }),
          signal: controller.signal,
        });
        if (!isRecord(result)) throw new Error("Fabushi browser sign-in poll response is invalid.");
        const state = boundedText(result.status, 80);
        if (state === "completed") {
          const session = normalizeFabushiSession(result.session);
          if (session == null) throw new Error("Fabushi browser sign-in returned an invalid durable session.");
          const verified = await this.resolveIdentity(session);
          if (epoch !== this.operationEpoch || controller.signal.aborted) return await this.getStatus();
          await this.saveStoredSession(verified);
          await this.writeHostCredential(verified);
          const status = statusFromSession(verified);
          this.emit(status);
          return status;
        }
        if (state === "failed" || state === "expired" || state === "cancelled") {
          throw new Error(`Fabushi browser sign-in ${state}.`);
        }
        await delay(start.pollAfterMs, controller.signal);
      }
      throw new Error("Fabushi browser sign-in timed out.");
    } catch (error) {
      if (epoch !== this.operationEpoch || controller.signal.aborted) return await this.getStatus();
      const status: SandAuthStatus = { kind: "logged-out", errorMessage: error instanceof Error ? error.message : "Fabushi sign-in failed." };
      this.emit(status);
      return status;
    } finally {
      if (this.loginAbort === controller) this.loginAbort = undefined;
      if (epoch === this.operationEpoch) this.loginAttempt = undefined;
    }
  }

  async cancelLogin(): Promise<SandAuthStatus> {
    ++this.operationEpoch;
    this.loginAbort?.abort();
    this.loginAbort = undefined;
    const attempt = this.loginAttempt;
    this.loginAttempt = undefined;
    if (attempt != null) {
      void this.requestJson(`/api/auth/browser/attempts/${encodeURIComponent(attempt.attemptId)}/cancel`, {
        method: "POST",
        body: JSON.stringify({ pollSecret: attempt.pollSecret }),
      }).catch(() => {});
    }
    const status: SandAuthStatus = { kind: "logged-out" };
    this.emit(status);
    return status;
  }

  async logout(): Promise<SandAuthStatus> {
    ++this.operationEpoch;
    this.loginAbort?.abort();
    this.loginAbort = undefined;
    const session = await this.currentSession().catch(() => null);
    await this.clearStoredSession().catch(() => {});
    await this.clearHostCredential();
    if (session?.refreshToken != null) {
      void this.requestJson("/api/auth/logout", {
        method: "POST",
        headers: { Authorization: `Bearer ${session.accessToken}` },
        body: JSON.stringify({ refreshToken: session.refreshToken, deviceId: session.deviceId }),
      }).catch(() => {});
    }
    const status: SandAuthStatus = { kind: "logged-out" };
    this.emit(status);
    return status;
  }

  async revokeForAccountRefusal(): Promise<{ readonly kind: "completed"; readonly status: SandAuthStatus }> {
    const status = await this.logout();
    return { kind: "completed", status };
  }

  async updateDisplayName(_name: string): Promise<SandAuthStatus> {
    return await this.getStatus();
  }
}
