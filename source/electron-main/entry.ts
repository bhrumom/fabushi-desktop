import * as electron from "electron";

import { parseAllowedExternalUrl } from "../shared/external-url-policy.js";
import { startElectronMainProduction } from "./main.js";
import {
  createElectronProductionNativeBindings,
} from "./main-production-services.js";
import {
  composeElectronProductionCoordinatorBindings,
  createElectronProductionServiceFactories,
  type ElectronProductionAdapterBindings,
} from "./production-adapters.js";
import {
  createElectronProductionSecureStorageBinding,
  createElectronProductionSettingsBinding,
  createElectronProductionUpdaterInstallerBinding,
  createElectronProductionNotificationsBinding,
  createElectronProductionStartupBinding,
  createElectronProductionMediaProtocolBinding,
} from "./production-binding-providers.js";
import {
  createElectronProductionAccountOAuthBinding,
} from "./adapters/account-oauth.js";
import {
  createElectronProductionAttachmentGatewayBinding,
} from "./adapters/attachment-gateway.js";
import {
  createElectronProductionAvatarImagesBinding,
  createElectronProductionImageContextMenuBinding,
} from "./adapters/avatar-images.js";
import {
  createElectronProductionCursorAccountBinding,
} from "./adapters/account-edge.js";
import {
  createElectronProductionExperimentsBinding,
} from "./adapters/production-experiments-binding.js";
import {
  createElectronProductionIpcBinding,
} from "./adapters/ipc.js";
import {
  createElectronProductionMainRpcBinding,
} from "./adapters/main-rpc.js";
import {
  createProductionMcpOAuthAdapter,
} from "./adapters/mcp-oauth.js";
import {
  createElectronProductionTelemetryBinding,
} from "./adapters/telemetry.js";
import {
  createElectronProductionCoordinatorBinding,
} from "./coordinator/production-root-provider.js";
import {
  reportDesktopEdgeFailure,
} from "./desktop-edge-failures.js";

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
  cursorAccount: createElectronProductionCursorAccountBinding(),
  mainRpc: createElectronProductionMainRpcBinding(),
  updaterInstaller: createElectronProductionUpdaterInstallerBinding(),
  mediaProtocol: createElectronProductionMediaProtocolBinding(),
  accountOAuth: createElectronProductionAccountOAuthBinding(),
  experiments: createElectronProductionExperimentsBinding(),
  mcpOAuth: createProductionMcpOAuthAdapter(),
  telemetry: createElectronProductionTelemetryBinding(),
  notifications: createElectronProductionNotificationsBinding(),
  coordinator: coordinator.coordinator,
  ipc: coordinator.ipc,
  imageContextMenu: createElectronProductionImageContextMenuBinding(),
};

startElectronMainProduction({
  native: createElectronProductionNativeBindings(electron as never),
  moduleDir: __dirname,
  env: process.env,
  platform: process.platform,
  startup: createElectronProductionStartupBinding(),
  services: createElectronProductionServiceFactories(adapters),
  parseAllowedExternalUrl,
  reportFailure: reportDesktopEdgeFailure,
});
