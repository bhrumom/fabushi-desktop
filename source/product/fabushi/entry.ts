import * as electron from "electron";

import { parseAllowedExternalUrl } from "../../shared/external-url-policy.js";
import { startElectronMainProduction } from "../../electron-main/main.js";
import {
  createElectronProductionNativeBindings,
} from "../../electron-main/main-production-services.js";
import {
  composeElectronProductionCoordinatorBindings,
  createElectronProductionServiceFactories,
  type ElectronProductionAdapterBindings,
} from "../../electron-main/production-adapters.js";
import {
  createElectronProductionSecureStorageBinding,
  createElectronProductionSettingsBinding,
  createElectronProductionUpdaterInstallerBinding,
  createElectronProductionNotificationsBinding,
  createElectronProductionStartupBinding,
  createElectronProductionMediaProtocolBinding,
} from "../../electron-main/production-binding-providers.js";
import {
  createElectronProductionAttachmentGatewayBinding,
} from "../../electron-main/adapters/attachment-gateway.js";
import {
  createElectronProductionAvatarImagesBinding,
  createElectronProductionImageContextMenuBinding,
} from "../../electron-main/adapters/avatar-images.js";
import {
  createElectronProductionExperimentsBinding,
} from "../../electron-main/adapters/production-experiments-binding.js";
import {
  createElectronProductionIpcBinding,
} from "../../electron-main/adapters/ipc.js";
import {
  createElectronProductionMainRpcBinding,
} from "../../electron-main/adapters/main-rpc.js";
import {
  createProductionMcpOAuthAdapter,
} from "../../electron-main/adapters/mcp-oauth.js";
import {
  createElectronProductionTelemetryBinding,
} from "../../electron-main/adapters/telemetry.js";
import {
  createElectronProductionCoordinatorBinding,
} from "../../electron-main/coordinator/production-root-provider.js";
import {
  reportDesktopEdgeFailure,
} from "../../electron-main/desktop-edge-failures.js";
import {
  createFabushiProductionAccountOAuthBinding,
  createFabushiProductionCursorAccountBinding,
} from "./fabushi-account-adapters.js";

const baseIpc = createElectronProductionIpcBinding();
const coordinator = composeElectronProductionCoordinatorBindings(
  createElectronProductionCoordinatorBinding(),
  baseIpc,
);

const adapters: ElectronProductionAdapterBindings = {
  secureStorage: createElectronProductionSecureStorageBinding(),
  settings: createElectronProductionSettingsBinding(),
  attachmentGateway: createElectronProductionAttachmentGatewayBinding(),
  avatarImages: createElectronProductionAvatarImagesBinding(),
  cursorAccount: createFabushiProductionCursorAccountBinding(),
  mainRpc: createElectronProductionMainRpcBinding(),
  updaterInstaller: createElectronProductionUpdaterInstallerBinding(),
  mediaProtocol: createElectronProductionMediaProtocolBinding(),
  accountOAuth: createFabushiProductionAccountOAuthBinding(),
  experiments: createElectronProductionExperimentsBinding(),
  mcpOAuth: createProductionMcpOAuthAdapter(),
  telemetry: createElectronProductionTelemetryBinding(),
  notifications: createElectronProductionNotificationsBinding(),
  coordinator: coordinator.coordinator,
  ipc: coordinator.ipc,
  imageContextMenu: createElectronProductionImageContextMenuBinding(),
};

const nativeBindings = createElectronProductionNativeBindings(electron as never);
const e2eAuthWebsite = process.env.FABUSHI_E2E === "1"
  ? process.env.SAND_CURSOR_WEBSITE_URL
  : undefined;
const entryNativeBindings = e2eAuthWebsite == null
  ? nativeBindings
  : {
      ...nativeBindings,
      shell: {
        openExternal: async (url: string) => {
          try {
            const expected = new URL(e2eAuthWebsite);
            const actual = new URL(url);
            if (actual.origin === expected.origin && actual.pathname === "/loginDeepControl") return;
          } catch {
            // Preserve the shipping external-url path for malformed/unrelated URLs.
          }
          return await nativeBindings.shell.openExternal(url);
        },
      },
    };

startElectronMainProduction({
  native: entryNativeBindings,
  moduleDir: __dirname,
  env: process.env,
  platform: process.platform,
  startup: createElectronProductionStartupBinding(),
  services: createElectronProductionServiceFactories(adapters),
  parseAllowedExternalUrl,
  reportFailure: reportDesktopEdgeFailure,
});
