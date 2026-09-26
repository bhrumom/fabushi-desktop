import { createCursorAuthWiring, type AuthServicePort } from "../account/cursor-auth-wiring.js";
import type { CursorLoginManager } from "../account/cursor-auth.js";
import type { ElectronProductionAdapterBindings } from "../production-adapters.js";
import type { ProductionAccountService, ProductionServiceContext } from "../main-production-services.js";
import { requireFunction, requireObject } from "./provider-guards.js";

type CursorAuthWiringDeps = Parameters<typeof createCursorAuthWiring>[0];

const E2E_ACCOUNT_SUBJECT = "fabushi-e2e-account";
const E2E_ACCOUNT_EMAIL = "e2e@fabushi.local";
const E2E_ACCOUNT_EXP_SECONDS = 4_102_444_800;

function encodeE2eJwtPart(value: Readonly<Record<string, unknown>>): string {
  return Buffer.from(JSON.stringify(value), "utf8").toString("base64url");
}

function createE2eAccessToken(): string {
  return [
    encodeE2eJwtPart({ alg: "none", typ: "JWT" }),
    encodeE2eJwtPart({
      sub: E2E_ACCOUNT_SUBJECT,
      email: E2E_ACCOUNT_EMAIL,
      exp: E2E_ACCOUNT_EXP_SECONDS,
    }),
    "fabushi-e2e",
  ].join(".");
}

export function resolveAccountOAuthE2eOverrides(
  env: NodeJS.ProcessEnv,
): Pick<
  CursorAuthWiringDeps,
  "openExternal" | "serviceOptions" | "fetchProfile" | "fetchLocalToolPermissionCeiling" | "sentryEnabled"
> | undefined {
  if (env.FABUSHI_E2E !== "1") return undefined;

  const createLoginManager = (): CursorLoginManager => ({
    startLogin: () => ({
      metadata: {
        uuid: "fabushi-e2e-login",
        verifier: "fabushi-e2e-verifier",
      },
      loginUrl: "about:blank#fabushi-e2e-account-login",
    }),
    async waitForResult(_metadata, signal) {
      if (signal?.aborted === true) return null;
      const accessToken = createE2eAccessToken();
      return { accessToken, refreshToken: accessToken };
    },
  });

  return {
    // Focused Electron acceptance still traverses the shipping renderer ->
    // preload -> Main Account service path. Only the external browser leg is
    // replaced so CI never depends on an interactive Cursor OAuth session.
    openExternal: async () => {},
    serviceOptions: { createLoginManager },
    fetchProfile: async () => ({
      email: E2E_ACCOUNT_EMAIL,
      displayName: "Fabushi E2E",
      isAnysphereUser: false,
    }),
    fetchLocalToolPermissionCeiling: async () => undefined,
    sentryEnabled: false,
  };
}

export interface ProductionAccountOAuthPorts {
  readonly resolveWiringDeps?: (context: ProductionServiceContext) => CursorAuthWiringDeps;
}

function accountRuntimeOf(context: ProductionServiceContext): ReturnType<CursorAuthWiringDeps["getAccountRuntime"]> {
  try {
    const runtime = context.requireCoordinator().getAccountRuntime?.();
    if (runtime != null && typeof (runtime as { observe?: unknown }).observe === "function" && typeof (runtime as { whenIdle?: unknown }).whenIdle === "function") {
      return runtime as ReturnType<CursorAuthWiringDeps["getAccountRuntime"]>;
    }
  } catch {
    // Account construction precedes coordinator construction. The auth wiring
    // must deliver directly until the coordinator exposes its settled runtime.
  }
  return null;
}

function defaultWiringDeps(context: ProductionServiceContext): CursorAuthWiringDeps {
  requireFunction(context.native?.shell?.openExternal, "electron.shell.openExternal");
  requireFunction(context.settings?.settingsStore?.getLocalToolPermission, "account settings.getLocalToolPermission");
  requireFunction(context.settings?.settingsStore?.setLocalToolPermissionCeiling, "account settings.setLocalToolPermissionCeiling");
  requireFunction(context.requireMainEdge, "account main-edge");
  requireFunction(context.coordinatorLegs?.legs?.setHostSettings, "account coordinator.setHostSettings");
  const e2eOverrides = resolveAccountOAuthE2eOverrides(context.env);
  return {
    openExternal: async (url) => { await context.native.shell.openExternal(url); },
    getAccountRuntime: () => accountRuntimeOf(context),
    emitAuthStatus: (status) => context.requireMainEdge().emit("cursor-auth-changed", status),
    sentryEnabled: context.env.SAND_DISABLE_SENTRY !== "1",
    settingsStore: context.settings.settingsStore,
    syncHostSettingsToBox: async (settings) => {
      const setHostSettings = context.coordinatorLegs.legs.setHostSettings;
      if (typeof setHostSettings !== "function") throw new Error("Electron production account requires coordinator host-settings synchronization.");
      await setHostSettings(settings);
    },
    ...(e2eOverrides ?? {}),
  };
}

function validateAuthService(service: AuthServicePort): AuthServicePort {
  requireObject(service, "accountOAuth.service");
  for (const method of ["subscribe", "getStatus", "getValidAccessToken", "revokeForAccountRefusal", "login", "cancelLogin", "logout", "updateDisplayName"] as const) {
    requireFunction(service[method], `accountOAuth.service.${method}`);
  }
  return service;
}

/** Artifact anchor: main.cjs:505993, `var cursorAuthWiring = createCursorAuthWiring({`. */
export function createProductionAccountOAuthAdapter(
  ports: ProductionAccountOAuthPorts,
): ElectronProductionAdapterBindings["accountOAuth"] {
  return {
    async create(context): Promise<ProductionAccountService> {
      const wiring = createCursorAuthWiring((ports?.resolveWiringDeps ?? defaultWiringDeps)(context));
      const service = validateAuthService(await wiring.ensureCursorAuthService());
      const subscriptions = new Set<() => void>();
      let disposed = false;
      return {
        getStatus: () => service.getStatus(),
        currentAuthStatusFreshness: wiring.currentAuthStatusFreshness,
        deliverCursorAuthStatus(status) {
          if (disposed) throw new Error("Electron production account adapter is disposed.");
          wiring.deliverCursorAuthStatus(service, status);
        },
        async getAuthService() {
          if (disposed) throw new Error("Electron production account adapter is disposed.");
          return service;
        },
        async revokeForAccountRefusal() {
          if (disposed) throw new Error("Electron production account adapter is disposed.");
          return await service.revokeForAccountRefusal();
        },
        subscribe(listener) {
          if (disposed) throw new Error("Electron production account adapter is disposed.");
          const unsubscribe = service.subscribe(() => listener());
          subscriptions.add(unsubscribe);
          return () => { if (subscriptions.delete(unsubscribe)) unsubscribe(); };
        },
        async dispose() {
          if (disposed) return;
          disposed = true;
          const failures: unknown[] = [];
          for (const unsubscribe of [...subscriptions].reverse()) {
            subscriptions.delete(unsubscribe);
            try { unsubscribe(); } catch (error) { failures.push(error); }
          }
          try { wiring.dispose(); } catch (error) { failures.push(error); }
          if (failures.length === 1) throw failures[0];
          if (failures.length > 1) throw new AggregateError(failures, "Electron production account cleanup failed.");
        },
      };
    },
  };
}

/**
 * Exact desktop account composition: native browser callback, secure-store
 * defaults, generated profile clients, main-edge status, coordinator account
 * runtime, and host-settings synchronization remain real owner seams.
 */
export function createElectronProductionAccountOAuthBinding(): ElectronProductionAdapterBindings["accountOAuth"] {
  return createProductionAccountOAuthAdapter({});
}
