export type NativeDesktopEvent =
  | "app-agent-surface-request"
  | "mcp-auth-completed"
  | "focus-agent"
  | "cloud-agent-open"
  | "shared-room-changed"
  | "deep-link"
  | "compute-migration"
  | "dev-compute-rebuild"
  | "open-feedback"
  | "open-about"
  | "widget-gallery"
  | "force-onboarding"
  | "account-auth-changed"
  | "experiments-changed"
  | "window-state"
  | "zoom-factor-changed"
  | "update-computer-dispatched"
  | "offline-asr-progress"
  | "open-offline-asr"
  | "remote-desktop-user-presence"
  | "dev-compute-pull-progress"
  | "egress-tunnel-changed"
  | "egress-tunnel-status-changed"
  | "webauthn-proxy-changed"
  | "skip-onboarding"
  | "theme-changed"
  | "update-status"
  | "messaging-call-signal"
  | "messaging-call-status"
;

export interface NativeDesktopBridge {
  invoke<T>(method: string, params?: Record<string, unknown>): Promise<T>;
  subscribe(listeners: Partial<Record<NativeDesktopEvent, (payload: unknown) => void>>): () => void;
}

declare global {
  interface Window {
    fabushiNative?: NativeDesktopBridge;
  }
}

export function nativeDesktopBridge(): NativeDesktopBridge | null {
  if (typeof window === "undefined") return null;
  return typeof window.fabushiNative?.invoke === "function" ? window.fabushiNative : null;
}


export interface NativeDeepLink {
  readonly version: 1;
  readonly route: "agent" | "auth" | "settings" | "feedback" | "about" | "widgets" | "onboarding";
  readonly source?: string;
  readonly canonicalUrl?: string;
  readonly agentId?: string;
  readonly section?: "general" | "mcp" | "usage" | "updates";
  readonly action?: "start" | "skip" | "complete";
  readonly attemptId?: string;
  readonly status?: "completed" | "cancelled" | "failed";
}

export interface NativeDesktopEnvironment {
  readonly platform: string;
  readonly arch: string;
  readonly appVersion: string;
  readonly electronVersion: string;
  readonly packaged: boolean;
}

export function invokeNativeDesktop<T>(method: string, params?: Record<string, unknown>): Promise<T> {
  const bridge = nativeDesktopBridge();
  return bridge
    ? bridge.invoke<T>(method, params)
    : Promise.reject(new Error("Native desktop bridge is unavailable."));
}

export function subscribeNativeDesktopEvents(
  listeners: Partial<Record<NativeDesktopEvent, (payload: unknown) => void>>,
): () => void {
  return nativeDesktopBridge()?.subscribe(listeners) ?? (() => undefined);
}

export function markNativeDeepLinksReady(): void {
  const bridge = nativeDesktopBridge();
  if (!bridge) return;
  void bridge.invoke("markDeepLinksReady").catch(() => undefined);
}

export async function nativeOnboardingSeen(): Promise<boolean | null> {
  const bridge = nativeDesktopBridge();
  if (!bridge) return null;
  return bridge.invoke<boolean>("getOnboardingSeen").catch(() => null);
}

export function rememberNativeOnboarding(): void {
  const bridge = nativeDesktopBridge();
  if (!bridge) return;
  void bridge.invoke<boolean>("setOnboardingSeen", { seen: true }).catch(() => undefined);
}

export function syncNativeTheme(preference: "system" | "light" | "dark"): void {
  const bridge = nativeDesktopBridge();
  if (!bridge) return;
  void bridge.invoke("setThemePreference", { preference }).catch(() => undefined);
}

export interface NativeOfflineAsrStatus {
  readonly available: boolean;
  readonly engine: string;
  readonly binaryPath: string | null;
  readonly modelPath: string | null;
  readonly modelUrlConfigured: boolean;
  readonly modelUrl?: string | null;
  readonly expectedSha256?: string | null;
  readonly expectedSizeBytes?: number | null;
  readonly modelSha256: string | null;
  readonly modelVerified: boolean;
  readonly missing: readonly string[];
}

export interface NativeOfflineAsrProgress {
  readonly phase: "model-download" | "ready" | "transcribing" | "complete" | "unavailable";
  readonly downloadedBytes?: number;
  readonly totalBytes?: number | null;
  readonly progress?: number;
  readonly status?: NativeOfflineAsrStatus;
}

export interface NativeDiskAuditEntry {
  readonly name: string;
  readonly bytes: number;
  readonly reclaimable: boolean;
}

export interface NativeDiskAudit {
  readonly root: string;
  readonly totalBytes: number;
  readonly reclaimableBytes: number;
  readonly scannedNodes: number;
  readonly truncated: boolean;
  readonly scannedAtMs: number;
  readonly entries: readonly NativeDiskAuditEntry[];
}

export async function requestNativeDiskSaverAudit(): Promise<NativeDiskAudit | null> {
  const bridge = nativeDesktopBridge();
  if (!bridge) return null;
  return bridge.invoke<NativeDiskAudit>("requestDiskSaverAudit").catch(() => null);
}
