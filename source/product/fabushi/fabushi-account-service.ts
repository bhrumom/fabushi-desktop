import { randomBytes, randomUUID } from "node:crypto";
import { promises as fs } from "node:fs";
import { dirname } from "node:path";

import type { AuthServicePort, FabushiDeviceAgentSession, SandAuthStatus } from "../../electron-main/account/auth-service-contract.js";
import { settleOptionalRemoteComputerTarget } from "./remote-target-availability.js";
import { deleteSecret, readSecret, writeSecret } from "../../electron-main/secrets/secret-store.js";

export const FABUSHI_ACCOUNT_SESSION_SECRET = "fabushi-account-session-v1";
export const FABUSHI_CI_ACCOUNT_SESSION_FILE_ENV = "FABUSHI_CI_ACCOUNT_SESSION_FILE";
export const FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV = "FABUSHI_HOST_ACCESS_CREDENTIAL_FILE";
export const FABUSHI_REMOTE_COMPUTER_DEVICE_CREDENTIAL_FILE_ENV = "FABUSHI_REMOTE_COMPUTER_DEVICE_CREDENTIAL_FILE";
const FABUSHI_REMOTE_COMPUTER_DEVICE_SECRET_PREFIX = "fabushi-remote-computer-device-secret-v1";
const REFRESH_LEEWAY_MS = 60_000;
const MIN_POLL_MS = 250;
const MAX_POLL_MS = 5_000;
const REMOTE_COMPUTER_HEARTBEAT_MS = 25_000;

interface BrowserAttempt {
  readonly attemptId: string;
  readonly loginUrl: string;
  readonly pollSecret: string;
  readonly expiresAt: number;
  readonly pollAfterMs: number;
}
export {
  DEFAULT_FABUSHI_API_BASE_URL,
  DEFAULT_FABUSHI_RESPONSES_URL,
  normalizeFabushiApiBaseUrl,
  normalizeFabushiCiSession,
  normalizeFabushiSession,
} from "./fabushi-account-policy.js";
import {
  boundedText,
  isRecord,
  normalizeExpiryMs,
  normalizeFabushiApiBaseUrl,
  normalizeFabushiCiSession,
  normalizeFabushiSession,
  FABUSHI_CI_ISOLATED_AUTH_BASE_URL,
  isProductionFabushiIssuer,
  nativeRemoteComputerDeviceScope,
  nativeAuthIdForIssuer,
  type FabushiSession,
} from "./fabushi-account-policy.js";

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
    authId: nativeAuthIdForIssuer(session.userId, session.issuer),
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
  readonly hostRemoteComputerCredentialFile?: string;
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
  private logoutInProgress = false;
  private readonly activeOperations = new Set<Promise<unknown>>();
  private ciSessionPromise: Promise<FabushiSession | null> | undefined;
  private ciSessionDisabled = false;
  private refreshPromise: Promise<FabushiSession | null> | undefined;
  private remoteComputerRegistrationPromise: Promise<void> | undefined;
  private remoteComputerRegisteredIdentity: string | undefined;
  private remoteComputerHeartbeatIdentity: string | undefined;
  private remoteComputerHeartbeatTimer: ReturnType<typeof setTimeout> | undefined;

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

  private isOperationCurrent(epoch: number): boolean {
    return !this.logoutInProgress && this.operationEpoch === epoch;
  }

  private trackOperation<T>(operation: Promise<T>): Promise<T> {
    let tracked: Promise<T>;
    tracked = operation.finally(() => this.activeOperations.delete(tracked));
    this.activeOperations.add(tracked);
    return tracked;
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
    if (this.ciSessionDisabled) return null;
    const source = this.options.env[FABUSHI_CI_ACCOUNT_SESSION_FILE_ENV]?.trim();
    if (!source) return null;
    this.ciSessionPromise ??= fs.readFile(source, "utf8").then((raw) => {
      const parsed: unknown = JSON.parse(raw);
      const session = normalizeFabushiCiSession(parsed, { issuer: this.baseUrl, bindMissingIssuer: true });
      if (session == null) {
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
      return normalizeFabushiSession(JSON.parse(raw), { issuer: this.baseUrl });
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
      issuer: session.issuer,
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

  private remoteComputerAccountIdentity(session: FabushiSession): string {
    return `${nativeAuthIdForIssuer(session.userId, session.issuer)}\0${session.deviceId}`;
  }

  private remoteComputerDeviceSecretKey(session: FabushiSession): string {
    const scope = nativeRemoteComputerDeviceScope(session.userId, session.issuer, session.deviceId);
    return `${FABUSHI_REMOTE_COMPUTER_DEVICE_SECRET_PREFIX}:${scope}`;
  }

  private async writeHostRemoteComputerCredential(session: FabushiSession, deviceSecret: string): Promise<void> {
    const path = this.options.hostRemoteComputerCredentialFile;
    if (!path) return;
    await fs.mkdir(dirname(path), { recursive: true, mode: 0o700 });
    const temporary = `${path}.${process.pid}.${randomUUID()}.tmp`;
    await fs.writeFile(temporary, JSON.stringify({
      version: 1,
      userId: session.userId,
      deviceId: session.deviceId,
      deviceSecret,
    }), { encoding: "utf8", mode: 0o600, flag: "wx" });
    try {
      await fs.rename(temporary, path);
      await fs.chmod(path, 0o600).catch(() => {});
    } catch (error) {
      await fs.rm(temporary, { force: true }).catch(() => {});
      throw error;
    }
  }

  private async clearHostRemoteComputerCredential(): Promise<void> {
    const path = this.options.hostRemoteComputerCredentialFile;
    if (path) await fs.rm(path, { force: true }).catch(() => {});
  }

  private stopRemoteComputerHeartbeat(): void {
    if (this.remoteComputerHeartbeatTimer != null) clearTimeout(this.remoteComputerHeartbeatTimer);
    this.remoteComputerHeartbeatTimer = undefined;
    this.remoteComputerHeartbeatIdentity = undefined;
  }

  private async remoteComputerHeartbeat(session: FabushiSession): Promise<void> {
    const deviceSecret = await readSecret(this.remoteComputerDeviceSecretKey(session));
    if (deviceSecret == null || deviceSecret.length < 48 || deviceSecret.length > 256) {
      throw new Error("Fabushi Remote Computer device authority is unavailable.");
    }
    const response = await this.requestJson("/v1/computers/heartbeat", {
      method: "POST",
      headers: { Authorization: `Bearer ${session.accessToken}` },
      body: JSON.stringify({ deviceId: session.deviceId, deviceSecret }),
    });
    if (!isRecord(response) || response.ok !== true || typeof response.lastSeenAt !== "number") {
      throw new Error("Fabushi Remote Computer heartbeat failed validation.");
    }
  }

  private scheduleRemoteComputerHeartbeat(identity: string): void {
    if (this.remoteComputerHeartbeatIdentity === identity && this.remoteComputerHeartbeatTimer != null) return;
    this.stopRemoteComputerHeartbeat();
    this.remoteComputerHeartbeatIdentity = identity;
    const scheduleNext = () => {
      if (this.remoteComputerHeartbeatIdentity !== identity || this.remoteComputerHeartbeatTimer != null) return;
      this.remoteComputerHeartbeatTimer = setTimeout(() => {
        this.remoteComputerHeartbeatTimer = undefined;
        const epoch = this.operationEpoch;
        void this.trackOperation((async () => {
          try {
            const session = await this.currentSession();
            if (!this.isOperationCurrent(epoch)
              || session == null
              || this.remoteComputerAccountIdentity(session) !== identity) {
              this.stopRemoteComputerHeartbeat();
              return;
            }
            await this.remoteComputerHeartbeat(session);
            if (!this.isOperationCurrent(epoch)) return;
          } catch {
            if (!this.isOperationCurrent(epoch)) return;
            this.remoteComputerRegisteredIdentity = undefined;
            await this.clearHostRemoteComputerCredential();
          } finally {
            if (this.isOperationCurrent(epoch)
              && this.remoteComputerHeartbeatIdentity === identity
              && this.remoteComputerHeartbeatTimer == null) scheduleNext();
          }
        })());
      }, REMOTE_COMPUTER_HEARTBEAT_MS);
      this.remoteComputerHeartbeatTimer.unref?.();
    };
    scheduleNext();
  }

  private async ensureRemoteComputerRegistration(session: FabushiSession): Promise<void> {
    const identity = this.remoteComputerAccountIdentity(session);
    if (this.remoteComputerRegisteredIdentity === identity) {
      this.scheduleRemoteComputerHeartbeat(identity);
      return;
    }
    if (this.remoteComputerRegistrationPromise != null) {
      await this.remoteComputerRegistrationPromise;
      if (this.remoteComputerRegisteredIdentity === identity) return;
    }
    this.remoteComputerRegistrationPromise = (async () => {
      const secretKey = this.remoteComputerDeviceSecretKey(session);
      let deviceSecret = await readSecret(secretKey);
      if (deviceSecret == null || deviceSecret.length < 48 || deviceSecret.length > 256) {
        deviceSecret = randomBytes(48).toString("base64url");
        await writeSecret(secretKey, deviceSecret);
      }
      const response = await this.requestJson("/v1/computers/register", {
        method: "POST",
        headers: { Authorization: `Bearer ${session.accessToken}` },
        body: JSON.stringify({
          deviceId: session.deviceId,
          label: "Fabushi Desktop",
          deviceSecret,
          provider: "fabushi-desktop",
          platform: process.platform,
          capabilities: [],
        }),
      });
      if (!isRecord(response) || response.deviceId !== session.deviceId) {
        throw new Error("Fabushi Remote Computer registration failed identity validation.");
      }
      await this.writeHostRemoteComputerCredential(session, deviceSecret);
      this.remoteComputerRegisteredIdentity = identity;
      this.scheduleRemoteComputerHeartbeat(identity);
    })().finally(() => {
      this.remoteComputerRegistrationPromise = undefined;
    });
    await this.remoteComputerRegistrationPromise;
  }

  private async settleRemoteComputerTarget(session: FabushiSession): Promise<void> {
    if (!isProductionFabushiIssuer(session.issuer)) {
      this.remoteComputerRegisteredIdentity = undefined;
      this.stopRemoteComputerHeartbeat();
      await this.clearHostRemoteComputerCredential().catch(() => {});
      return;
    }
    await settleOptionalRemoteComputerTarget(
      async () => await this.ensureRemoteComputerRegistration(session),
      async () => {
        this.remoteComputerRegisteredIdentity = undefined;
        this.stopRemoteComputerHeartbeat();
        await this.clearHostRemoteComputerCredential().catch(() => {});
      },
    );
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

  private async refreshSession(session: FabushiSession, epoch: number): Promise<FabushiSession | null> {
    if (!this.isOperationCurrent(epoch)) return null;
    if (session.refreshToken == null) return null;
    if (this.refreshPromise != null) return await this.refreshPromise;
    this.refreshPromise = (async () => {
      const raw = await this.requestJson("/api/auth/refresh", {
        method: "POST",
        body: JSON.stringify({ refreshToken: session.refreshToken, deviceId: session.deviceId }),
      });
      if (!this.isOperationCurrent(epoch)) return null;
      const next = normalizeFabushiSession(raw, { issuer: session.issuer, bindMissingIssuer: true });
      if (next == null
        || next.deviceId !== session.deviceId
        || String(next.userId) !== String(session.userId)) {
        throw new Error("Fabushi refreshed account session failed identity validation.");
      }
      const verified = await this.resolveIdentity(next);
      if (!this.isOperationCurrent(epoch)) return null;
      await this.saveStoredSession(verified);
      if (!this.isOperationCurrent(epoch)) return null;
      await this.settleRemoteComputerTarget(verified);
      if (!this.isOperationCurrent(epoch)) return null;
      await this.writeHostCredential(verified);
      if (!this.isOperationCurrent(epoch)) return null;
      return verified;
    })().catch(async (error) => {
      if (!this.isOperationCurrent(epoch)) return null;
      await this.clearStoredSession().catch(() => {});
      await this.clearHostCredential();
      await this.clearHostRemoteComputerCredential();
      this.remoteComputerRegisteredIdentity = undefined;
      this.stopRemoteComputerHeartbeat();
      throw error;
    }).finally(() => {
      this.refreshPromise = undefined;
    });
    return await this.refreshPromise;
  }

  private currentSession(): Promise<FabushiSession | null> {
    if (this.logoutInProgress) return Promise.resolve(null);
    const epoch = this.operationEpoch;
    return this.trackOperation(this.resolveCurrentSession(epoch));
  }

  private async resolveCurrentSession(epoch: number): Promise<FabushiSession | null> {
    const ci = await this.readCiSession();
    if (!this.isOperationCurrent(epoch)) return null;
    let session = ci ?? await this.readStoredSession();
    if (!this.isOperationCurrent(epoch)) return null;
    if (session == null) {
      await this.clearHostCredential();
      if (!this.isOperationCurrent(epoch)) return null;
      await this.clearHostRemoteComputerCredential();
      if (!this.isOperationCurrent(epoch)) return null;
      this.remoteComputerRegisteredIdentity = undefined;
      this.stopRemoteComputerHeartbeat();
      return null;
    }
    if (session.accessTokenExpiresAt <= this.now() + REFRESH_LEEWAY_MS) {
      if (ci != null || session.refreshToken == null) {
        await this.clearHostCredential();
        if (!this.isOperationCurrent(epoch)) return null;
        await this.clearHostRemoteComputerCredential();
        if (!this.isOperationCurrent(epoch)) return null;
        this.remoteComputerRegisteredIdentity = undefined;
        this.stopRemoteComputerHeartbeat();
        return null;
      }
      session = await this.refreshSession(session, epoch);
      if (!this.isOperationCurrent(epoch)) return null;
      if (session == null) return null;
    }
    if (!this.isOperationCurrent(epoch)) return null;
    await this.settleRemoteComputerTarget(session);
    if (!this.isOperationCurrent(epoch)) return null;
    await this.writeHostCredential(session);
    return this.isOperationCurrent(epoch) ? session : null;
  }

  async getStatus(): Promise<SandAuthStatus> {
    const epoch = this.operationEpoch;
    try {
      const session = await this.currentSession();
      if (!this.isOperationCurrent(epoch)) return { kind: "logged-out" };
      return session == null ? { kind: "logged-out" } : statusFromSession(session);
    } catch (error) {
      if (!this.isOperationCurrent(epoch)) return { kind: "logged-out" };
      return { kind: "logged-out", errorMessage: error instanceof Error ? error.message : "Fabushi sign-in is unavailable." };
    }
  }

  async getValidAccessToken(options?: { readonly backendUrl?: string }): Promise<string> {
    if (this.baseUrl === FABUSHI_CI_ISOLATED_AUTH_BASE_URL && options?.backendUrl == null) {
      throw new Error("An isolated Fabushi credential requires an explicit same-issuer target.");
    }
    if (options?.backendUrl != null) {
      const requested = normalizeFabushiApiBaseUrl(options.backendUrl);
      if (new URL(requested).origin !== new URL(this.baseUrl).origin) {
        throw new Error("Refusing to send a Fabushi account token to a non-Fabushi backend.");
      }
    }
    const epoch = this.operationEpoch;
    const session = await this.currentSession();
    if (!this.isOperationCurrent(epoch) || session == null) throw new Error("Sign in to Fabushi to continue.");
    return session.accessToken;
  }

  async peekAccessToken(): Promise<string | null> {
    const epoch = this.operationEpoch;
    try {
      const session = await this.currentSession();
      if (!this.isOperationCurrent(epoch)) return null;
      return session?.issuer === this.baseUrl && session.issuer !== FABUSHI_CI_ISOLATED_AUTH_BASE_URL
        ? session.accessToken
        : null;
    } catch {
      return null;
    }
  }

  async deviceAgentSession(): Promise<FabushiDeviceAgentSession | null> {
    const epoch = this.operationEpoch;
    try {
      const session = await this.currentSession();
      if (!this.isOperationCurrent(epoch)
        || session == null
        || !isProductionFabushiIssuer(session.issuer)
        || session.accessTokenExpiresAt <= this.now() + REFRESH_LEEWAY_MS) {
        return null;
      }
      return {
        accessToken: session.accessToken,
        accessTokenExpiresAt: session.accessTokenExpiresAt,
        deviceId: session.deviceId,
        sessionId: session.sessionId,
        username: session.username,
      };
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

  login(): Promise<SandAuthStatus> {
    if (this.logoutInProgress) return Promise.resolve({ kind: "logged-out" });
    this.loginAbort?.abort();
    const epoch = ++this.operationEpoch;
    const controller = new AbortController();
    this.loginAbort = controller;
    this.emit({ kind: "logging-in" });
    return this.trackOperation(this.runLogin(epoch, controller));
  }

  private async runLogin(epoch: number, controller: AbortController): Promise<SandAuthStatus> {
    if (!this.isOperationCurrent(epoch) || controller.signal.aborted) return await this.getStatus();
    try {
      const start = this.parseBrowserAttempt(await this.requestJson("/api/auth/browser/start", {
        method: "POST",
        body: JSON.stringify({ deviceId: this.options.machineId, platform: "desktop" }),
        signal: controller.signal,
      }));
      this.loginAttempt = start;
      await this.options.openExternal(start.loginUrl);
      while (!controller.signal.aborted && this.isOperationCurrent(epoch) && this.now() < start.expiresAt) {
        const result = await this.requestJson(`/api/auth/browser/attempts/${encodeURIComponent(start.attemptId)}`, {
          method: "POST",
          body: JSON.stringify({ pollSecret: start.pollSecret }),
          signal: controller.signal,
        });
        if (!isRecord(result)) throw new Error("Fabushi browser sign-in poll response is invalid.");
        const state = boundedText(result.status, 80);
        if (state === "completed") {
          const session = normalizeFabushiSession(result.session, { issuer: this.baseUrl, bindMissingIssuer: true });
          if (session == null) throw new Error("Fabushi browser sign-in returned an invalid durable session.");
          const verified = await this.resolveIdentity(session);
          if (!this.isOperationCurrent(epoch) || controller.signal.aborted) return await this.getStatus();
          await this.saveStoredSession(verified);
          if (!this.isOperationCurrent(epoch) || controller.signal.aborted) return await this.getStatus();
          await this.settleRemoteComputerTarget(verified);
          if (!this.isOperationCurrent(epoch) || controller.signal.aborted) return await this.getStatus();
          await this.writeHostCredential(verified);
          if (!this.isOperationCurrent(epoch) || controller.signal.aborted) return await this.getStatus();
          const status = statusFromSession(verified);
          this.emit(status);
          return this.isOperationCurrent(epoch) ? status : { kind: "logged-out" };
        }
        if (state === "failed" || state === "expired" || state === "cancelled") {
          throw new Error(`Fabushi browser sign-in ${state}.`);
        }
        await delay(start.pollAfterMs, controller.signal);
      }
      throw new Error("Fabushi browser sign-in timed out.");
    } catch (error) {
      if (!this.isOperationCurrent(epoch) || controller.signal.aborted) return await this.getStatus();
      const status: SandAuthStatus = { kind: "logged-out", errorMessage: error instanceof Error ? error.message : "Fabushi sign-in failed." };
      this.emit(status);
      return status;
    } finally {
      if (this.loginAbort === controller) this.loginAbort = undefined;
      if (epoch === this.operationEpoch) this.loginAttempt = undefined;
    }
  }

  async cancelLogin(): Promise<SandAuthStatus> {
    if (!this.logoutInProgress) ++this.operationEpoch;
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
    const selectedCiBootstrap = !this.ciSessionDisabled
      && Boolean(this.options.env[FABUSHI_CI_ACCOUNT_SESSION_FILE_ENV]?.trim());
    this.ciSessionDisabled = true;
    this.logoutInProgress = true;
    const logoutEpoch = ++this.operationEpoch;
    this.loginAbort?.abort();
    this.loginAbort = undefined;
    const pendingOperations = [...this.activeOperations];
    try {
      await Promise.allSettled(pendingOperations);
      // CI bootstrap has precedence over stored sessions. Its refreshless
      // token is revoked by the existing CI lease owner, never by selecting an
      // unrelated stored account after disabling bootstrap.
      const session = selectedCiBootstrap ? null : await this.readStoredSession().catch(() => null);
      await this.clearStoredSession().catch(() => {});
      await this.clearHostCredential();
      await this.clearHostRemoteComputerCredential();
      this.remoteComputerRegisteredIdentity = undefined;
      this.stopRemoteComputerHeartbeat();
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
    } finally {
      if (this.operationEpoch === logoutEpoch) this.logoutInProgress = false;
    }
  }

  async revokeForAccountRefusal(): Promise<{ readonly kind: "completed"; readonly status: SandAuthStatus }> {
    const status = await this.logout();
    return { kind: "completed", status };
  }

  async updateDisplayName(_name: string): Promise<SandAuthStatus> {
    return await this.getStatus();
  }
}
