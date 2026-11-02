import { useCallback, useState } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";
import type {
  DesktopBridge,
  ProductNetworkProxyMutation,
  ProductNetworkProxyMutationResult,
  ProductSettingsSnapshot,
  ThemePreference,
  ThemeState,
} from "../frontend/src/recovered/contracts/desktop-bridge";
import type { SettingsNoticeEvent } from "../frontend/src/recovered/contracts/surface-notice";
import { SettingsDesktopSurface } from "../frontend/src/recovered/features/settings/overlay/desktop-surface";
import { createRuntimeThemeInstaller, type ThemeDocument } from "../frontend/src/recovered/features/runtime-theme-token-installer";
import "../frontend/src/production/production.css";

// The fixture supplies only the DesktopBridge boundary. Settings loading,
// mutation ordering, availability, lifetime and all controls are production code.
// Mutation outcomes are explicit test inputs, never a duplicate Settings reducer.
type BridgeId = "A" | "B";
type RequestKind = "proxy-read" | "proxy-mutate" | "theme-set";
type RequestRecord = { id: string; bridge: BridgeId; kind: RequestKind; args: unknown };
type PendingRequest = {
  bridge: BridgeId;
  resolve(value: unknown): void;
  reject(error: Error): void;
};
const clone = <T,>(value: T): T => structuredClone(value);
const requests: RequestRecord[] = [];
const notices: Array<{ bridge: BridgeId; instance: number; event: SettingsNoticeEvent }> = [];
const rejections: string[] = [];
const pending = new Map<string, PendingRequest>();
let nextRequestId = 0;
const themeInstaller = createRuntimeThemeInstaller(document as unknown as ThemeDocument, "light");

function initialSnapshot(bridge: BridgeId): ProductSettingsSnapshot {
  return {
    version: 2,
    network: {
      proxy: {
        mode: "system",
        selected: null,
        list: [{ type: "socks5", host: bridge === "A" ? "alpha.example" : "beta.example", port: 1080, user: "", password: "" }],
        tryIpv6: true,
        useProxyForCalls: false,
        checkIpWarningShown: false,
        rotation: { enabled: false, timeoutSeconds: bridge === "A" ? 10 : 30, preferredIndices: [] },
      },
    },
  };
}
const persisted: Record<BridgeId, ProductSettingsSnapshot> = { A: initialSnapshot("A"), B: initialSnapshot("B") };
const remainingReadFailures: Record<BridgeId, number> = {
  A: new URLSearchParams(location.search).get("failInitialProxyRead") === "1" ? 1 : 0,
  B: 0,
};
const record = (bridge: BridgeId, kind: RequestKind, args: unknown): string => {
  const id = `${kind}-${++nextRequestId}`;
  requests.push({ id, bridge, kind, args: clone(args) });
  return id;
};
const unsubscribe = () => {};
const subscribe = () => unsubscribe;

function bridgeFor(bridgeId: BridgeId): DesktopBridge {
  let theme: ThemeState = { preference: "light", resolved: "light" };
  const themeListeners = new Set<(value: ThemeState) => void>();
  const uiPreferences = { locale: "en", direction: "ltr" as const, reducedMotion: false, highContrast: false, textScale: 1 };
  return {
    platform: "darwin",
    cursorAccount: {
      getStatus: async () => ({ kind: "logged-in", authId: bridgeId, displayName: `Fixture ${bridgeId}`, email: `${bridgeId.toLowerCase()}@example.test` }),
      getAvatar: async () => null,
      getPrivacyModeEnabled: async () => true,
      getUsageSummary: async () => null,
      onStatusChanged: subscribe,
    },
    autoReviewInstructions: {
      get: async () => ({ isEnabled: true, allowInstructions: [], blockInstructions: [] }),
      set: async (value: unknown) => clone(value),
    },
    theme: {
      get: async () => clone(theme),
      set: async (preference: ThemePreference) => {
        record(bridgeId, "theme-set", { preference });
        theme = { preference, resolved: preference === "dark" ? "dark" : "light" };
        themeInstaller.update(theme.resolved);
        for (const listener of themeListeners) listener(clone(theme));
        return clone(theme);
      },
      onChanged: (listener: (value: ThemeState) => void) => {
        themeListeners.add(listener);
        return () => { themeListeners.delete(listener); };
      },
    },
    timeZone: {
      get: async () => ({ detectedTimeZone: "UTC", overrideTimeZone: null }),
      setOverride: async (overrideTimeZone: string | null) => ({ detectedTimeZone: "UTC", overrideTimeZone }),
    },
    uiPreferences: { get: async () => clone(uiPreferences), set: async (value: unknown) => clone(value) },
    callMedia: {
      getPreferences: async () => ({ microphoneId: null, cameraId: null }),
      setPreferences: async (value: unknown) => clone(value),
      requestPermissions: async () => ({ audio: false, video: false }),
    },
    localToolPermission: { get: async () => "ask", ceiling: async () => null, set: async (value: string) => value },
    foreverBox: {
      webauthnProxy: { initial: false, get: async () => false, set: async (value: boolean) => value, onChanged: subscribe },
      egressTunnel: { initial: false, initialStatus: { state: "off" }, onChanged: subscribe, onStatusChanged: subscribe },
    },
    update: { getStatus: async () => null, onStatusEvent: subscribe },
    experiments: { initialSnapshot: { featureGates: {} }, getSnapshot: async () => ({ featureGates: {} }), onChanged: subscribe },
    agent: { clientPersistence: { read: async () => null, write: async () => {} } },
    networkProxy: {
      get: async () => {
        record(bridgeId, "proxy-read", {});
        if (remainingReadFailures[bridgeId] > 0) {
          remainingReadFailures[bridgeId] -= 1;
          throw new Error("PRIVATE_PROXY_READ_FAILURE_WITH_SECRET");
        }
        return clone(persisted[bridgeId]);
      },
      mutate: (mutation: ProductNetworkProxyMutation) => {
        const id = record(bridgeId, "proxy-mutate", mutation);
        return new Promise<ProductNetworkProxyMutationResult>((resolve, reject) => {
          pending.set(id, { bridge: bridgeId, resolve: (value) => resolve(value as ProductNetworkProxyMutationResult), reject });
        });
      },
    },
  } as unknown as DesktopBridge;
}
const bridges: Record<BridgeId, DesktopBridge> = { A: bridgeFor("A"), B: bridgeFor("B") };
type Controls = {
  close(): void;
  reopen(): void;
  unmount(): void;
  mount(): void;
  switchBridge(bridge: BridgeId): void;
};
let controls: Controls | null = null;
const paint = () => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));

function Fixture() {
  const [bridgeId, setBridgeId] = useState<BridgeId>("A");
  const [open, setOpen] = useState(true);
  const [mounted, setMounted] = useState(true);
  const [instance, setInstance] = useState(1);
  const onClose = useCallback(() => setOpen(false), []);
  const onNotice = useCallback((event: SettingsNoticeEvent) => {
    notices.push({ bridge: bridgeId, instance, event: clone(event) });
  }, [bridgeId, instance]);
  controls = {
    close: () => flushSync(() => setOpen(false)),
    reopen: () => flushSync(() => setOpen(true)),
    unmount: () => flushSync(() => setMounted(false)),
    mount: () => flushSync(() => { setInstance((value) => value + 1); setMounted(true); setOpen(true); }),
    switchBridge: (nextBridge) => flushSync(() => setBridgeId(nextBridge)),
  };
  return <div data-fixture-bridge={bridgeId} data-fixture-instance={instance}>
    {mounted ? <SettingsDesktopSurface
      bridge={bridges[bridgeId]}
      initialSection="general"
      isOpen={open}
      onClose={onClose}
      onNotice={onNotice}
    /> : null}
  </div>;
}

window.addEventListener("unhandledrejection", (event) => {
  rejections.push(event.reason instanceof Error ? event.reason.message : String(event.reason));
});
Object.assign(window, {
  __proxySettingsHarness: {
    requests: () => clone(requests),
    notices: () => clone(notices),
    rejections: () => [...rejections],
    snapshot: (bridge: BridgeId = "A") => clone(persisted[bridge]),
    paint,
    resolveMutation: async (id: string, snapshot: ProductSettingsSnapshot, publication: unknown, omitPublication = false) => {
      const request = pending.get(id);
      if (request == null) throw new Error(`Unknown fixture mutation: ${id}`);
      pending.delete(id);
      // A supplied result models an accepted durable write, even if its surface
      // has closed. Reopened get() must read this explicit authoritative fixture.
      persisted[request.bridge] = clone(snapshot);
      request.resolve({ settings: clone(snapshot), ...(omitPublication ? {} : { publication: clone(publication) }) });
      await paint();
    },
    rejectMutation: async (id: string, reason: string) => {
      const request = pending.get(id);
      if (request == null) throw new Error(`Unknown fixture mutation: ${id}`);
      pending.delete(id);
      request.reject(new Error(reason));
      await paint();
    },
    close: () => controls?.close(),
    reopen: () => controls?.reopen(),
    unmount: () => controls?.unmount(),
    mount: () => controls?.mount(),
    switchBridge: (bridge: BridgeId) => controls?.switchBridge(bridge),
    setTheme: (mode: "light" | "dark") => themeInstaller.update(mode),
  },
});
createRoot(document.getElementById("root")!).render(<Fixture />);
