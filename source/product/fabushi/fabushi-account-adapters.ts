import { join } from "node:path";

import {
  createCursorAccountEdgePort,
  type AccountRuntime,
} from "../../electron-main/account/cursor-auth-wiring.js";
import {
  createProductionAccountOAuthAdapter,
} from "../../electron-main/adapters/account-oauth.js";
import type {
  ProductionServiceContext,
} from "../../electron-main/main-production-services.js";
import type {
  ElectronProductionAdapterBindings,
} from "../../electron-main/production-adapters.js";
import {
  DEFAULT_FABUSHI_API_BASE_URL,
  FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV,
  FabushiAuthService,
} from "./fabushi-account-service.js";

export const FABUSHI_RESPONSES_URL_ENV = "FABUSHI_RESPONSES_URL";
export const FABUSHI_PRODUCT_MODE_ENV = "FABUSHI_PRODUCT_MODE";
export const SAND_PRODUCT_FEATURE_GATE_DEFAULTS_ENV = "SAND_PRODUCT_FEATURE_GATE_DEFAULTS";
export const DEFAULT_FABUSHI_RESPONSES_URL = `${DEFAULT_FABUSHI_API_BASE_URL}/v1/ai/responses`;

function accountRuntimeOf(
  context: Pick<ProductionServiceContext, "requireCoordinator">,
): AccountRuntime | null | undefined {
  try {
    const runtime = context.requireCoordinator().getAccountRuntime?.();
    if (
      runtime != null
      && typeof (runtime as { observe?: unknown }).observe === "function"
      && typeof (runtime as { whenIdle?: unknown }).whenIdle === "function"
    ) {
      return runtime as AccountRuntime;
    }
  } catch {
    // Root composition creates account before coordinator; lookup stays lazy.
  }
  return null;
}

function productHostCredentialPath(context: ProductionServiceContext): string {
  return join(context.native.app.getPath("userData"), "fabushi-host-access-credential-v1.json");
}

function installProductEnvironment(context: ProductionServiceContext): string {
  const credentialPath = productHostCredentialPath(context);
  context.env[FABUSHI_PRODUCT_MODE_ENV] = "1";
  context.env[FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV] = credentialPath;
  // Signed production must never inherit a shell-provided inference origin:
  // the account bearer is scoped to Fabushi's first-party control plane.
  if (context.native.app.isPackaged || context.env[FABUSHI_RESPONSES_URL_ENV]?.trim().length === 0 || context.env[FABUSHI_RESPONSES_URL_ENV] == null) {
    context.env[FABUSHI_RESPONSES_URL_ENV] = DEFAULT_FABUSHI_RESPONSES_URL;
  }
  // Grok keeps the capability gate; Fabushi's product composition owns the
  // shipping fallback until a first-party live gate source is available.
  context.env[SAND_PRODUCT_FEATURE_GATE_DEFAULTS_ENV] = "sand_agent_network=true";
  // Preserve an explicit user choice, but make first-party Fabushi inference
  // the default for fresh Fabushi profiles rather than Cursor private auth.
  if (context.settings.settingsStore.load().inferenceProvider == null) {
    context.settings.settingsStore.setInferenceProvider("fabushi");
  }
  return credentialPath;
}

export function createFabushiProductionAccountOAuthBinding(): ElectronProductionAdapterBindings["accountOAuth"] {
  const binding = createProductionAccountOAuthAdapter({
    resolveWiringDeps(context) {
      const hostAccessCredentialFile = installProductEnvironment(context);
      return {
        openExternal: async (url) => {
          await context.native.shell.openExternal(url);
        },
        createAuthService: () => new FabushiAuthService({
          openExternal: async (url) => {
            await context.native.shell.openExternal(url);
          },
          env: context.env,
          machineId: context.machineId,
          hostAccessCredentialFile,
        }),
        fetchProfile: async () => null,
        updateProfileName: async () => {},
        getAccountRuntime: () => accountRuntimeOf(context),
        emitAuthStatus: (status) => context.requireMainEdge().emit("cursor-auth-changed", status),
        sentryEnabled: false,
        fetchUserPrivacyMode: async () => true,
        fetchLocalToolPermissionCeiling: async () => undefined,
        settingsStore: context.settings.settingsStore,
        syncHostSettingsToBox: async (settings) => {
          const setHostSettings = context.coordinatorLegs.legs.setHostSettings;
          if (typeof setHostSettings !== "function") {
            throw new Error("Fabushi production account requires coordinator host-settings synchronization.");
          }
          await setHostSettings(settings);
        },
        reportFailure: (domain, operation, error) => {
          context.reportFailure?.("account", `${domain}:${operation}`, error);
        },
      };
    },
  });
  return {
    async create(context) {
      const account = await binding.create(context);
      // Settle the product session before Coordinator/Host startup so the child
      // process observes both the credential-file path and its first credential.
      await account.getStatus();
      return account;
    },
  };
}

export function createFabushiProductionCursorAccountBinding(): ElectronProductionAdapterBindings["cursorAccount"] {
  return {
    create(context) {
      return createCursorAccountEdgePort({
        ensureCursorAuthService: () => context.requireAccount().getAuthService(),
        currentAuthStatusFreshness: () => context.requireAccount().currentAuthStatusFreshness(),
        getAccountRuntime: () => accountRuntimeOf(context),
        readSandAccess: async () => ({ state: "granted", reason: "none" }),
        resetMcpManager: () => context.requireMcp().resetMcpManager(),
        refreshHostMcp: () => context.requireMcp().refreshHostMcp(),
        resolveAvatar: async (_authId, preferredUrl) => preferredUrl ?? null,
        fetchWeeklyUsage: async () => null,
        isUsagePageEnabled: () => false,
        fetchUsageSummary: async () => null,
        fetchPrReviewPreferences: async () => ({ user: undefined, team: undefined }),
        fetchPrivacyModeEnabled: async () => true,
        cancelTrial: async () => ({ ok: false, message: "This isn’t available in Fabushi." }),
        invokeDashboardAction: async () => ({ ok: false, message: "This action isn’t available in Fabushi." }),
        productDisplayName: "Fabushi",
      });
    },
    createTranscriptionManager() {
      return async () => ({
        async transcribe(): Promise<never> {
          throw new Error("Fabushi cloud transcription is unavailable; use the packaged offline ASR path.");
        },
      });
    },
  };
}
