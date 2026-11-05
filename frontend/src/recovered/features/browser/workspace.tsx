/**
 * Native Agent Browser Workspace (Phase 1)
 *
 * Upstream & Open PR References:
 * - Absorbed from PR #165 (`origin/feat/agent-browser-workspace-phase1-20261020:frontend/src/recovered/features/browser/workspace.tsx`
 *   and `source/shared/browser-workspace.ts`).
 * - Aligns with OpenClaw ACP session/turn ownership (`openclaw/src/acp/control-plane/manager.core.ts`)
 *   for user takeover (`available | paused | taken-over | stopped`) revoking local agent mutations,
 *   and `openclaw/src/plugins/registry.ts` for integrity-pinned unpacked extension lifecycle.
 * - Blueprint 08 (`docs/specs/fusion-blueprints/08-devices-computer-and-backend-parity.md`).
 */
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type FormEvent } from "react";
import "./workspace.css";

export const BROWSER_WORKSPACE_INVOKE_CHANNEL = "sand:browser-workspace";
export const BROWSER_WORKSPACE_STATE_CHANNEL = "sand:browser-workspace-state";

export const BROWSER_WORKSPACE_LIMITS = Object.freeze({
  maxConversationIdLength: 256,
  maxAddressLength: 4096,
  maxTabs: 20,
  maxBookmarks: 500,
  maxHistory: 500,
  maxDownloadBytes: 64 * 1024 * 1024,
  minZoomFactor: 0.5,
  maxZoomFactor: 3,
});

export interface BrowserWorkspaceBounds {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export type BrowserDownloadState = "progressing" | "completed" | "cancelled" | "interrupted";

export interface BrowserWorkspaceTabSnapshot {
  readonly id: string;
  readonly title: string;
  readonly url: string;
  readonly faviconUrl: string | null;
  readonly loading: boolean;
  readonly canGoBack: boolean;
  readonly canGoForward: boolean;
  readonly zoomFactor: number;
  readonly generation: number;
  readonly crashed: boolean;
  readonly error: { readonly code: string; readonly description: string } | null;
}

export interface BrowserWorkspaceBookmark {
  readonly id: string;
  readonly title: string;
  readonly url: string;
  readonly createdAtMs: number;
}

export interface BrowserWorkspaceHistoryEntry {
  readonly id: string;
  readonly title: string;
  readonly url: string;
  readonly visitedAtMs: number;
}

export interface BrowserWorkspaceDownload {
  readonly id: string;
  readonly tabId: string;
  readonly fileName: string;
  readonly url: string;
  readonly receivedBytes: number;
  readonly totalBytes: number;
  readonly state: BrowserDownloadState;
  readonly sha256: string | null;
  readonly startedAtMs: number;
}

export interface BrowserWorkspaceExtension {
  readonly key: string;
  readonly name: string;
  readonly version: string;
  readonly enabled: boolean;
  readonly permissions: readonly string[];
  readonly hostPermissions: readonly string[];
  readonly integrity: string;
  readonly error: string | null;
}

export interface BrowserWorkspaceExtensionCandidate {
  readonly candidateId: string;
  readonly name: string;
  readonly version: string;
  readonly permissions: readonly string[];
  readonly hostPermissions: readonly string[];
  readonly integrity: string;
}

export type BrowserAgentControlState = "available" | "paused" | "taken-over" | "stopped";

export type BrowserAgentTaskStatus = "idle" | "running" | "paused" | "taken-over" | "stopped";

export interface BrowserAgentControlSnapshot {
  readonly state: BrowserAgentControlState;
  readonly epoch: number;
  readonly taskId: string;
  readonly taskStatus: BrowserAgentTaskStatus;
  readonly activeOperationId: string | null;
}

export interface BrowserWorkspaceSnapshot {
  readonly workspaceId: string;
  readonly conversationId: string;
  readonly visible: boolean;
  readonly activeTabId: string;
  readonly tabs: readonly BrowserWorkspaceTabSnapshot[];
  readonly bookmarks: readonly BrowserWorkspaceBookmark[];
  readonly history: readonly BrowserWorkspaceHistoryEntry[];
  readonly downloads: readonly BrowserWorkspaceDownload[];
  readonly extensions: readonly BrowserWorkspaceExtension[];
  readonly extensionCandidate: BrowserWorkspaceExtensionCandidate | null;
  readonly agentControl: BrowserAgentControlSnapshot;
}

export type BrowserWorkspaceCommand =
  | { readonly type: "activate"; readonly conversationId: string; readonly bounds: BrowserWorkspaceBounds }
  | { readonly type: "hide"; readonly conversationId: string }
  | { readonly type: "set-bounds"; readonly conversationId: string; readonly bounds: BrowserWorkspaceBounds }
  | { readonly type: "new-tab"; readonly conversationId: string; readonly address?: string }
  | { readonly type: "select-tab"; readonly conversationId: string; readonly tabId: string }
  | { readonly type: "close-tab"; readonly conversationId: string; readonly tabId: string }
  | { readonly type: "navigate"; readonly conversationId: string; readonly tabId: string; readonly address: string }
  | { readonly type: "back" | "forward" | "reload" | "stop"; readonly conversationId: string; readonly tabId: string }
  | { readonly type: "set-zoom"; readonly conversationId: string; readonly tabId: string; readonly factor: number }
  | { readonly type: "add-bookmark"; readonly conversationId: string; readonly tabId: string }
  | { readonly type: "remove-bookmark"; readonly conversationId: string; readonly bookmarkId: string }
  | { readonly type: "extension-choose"; readonly conversationId: string }
  | { readonly type: "extension-install"; readonly conversationId: string; readonly candidateId: string }
  | { readonly type: "extension-set-enabled"; readonly conversationId: string; readonly extensionKey: string; readonly enabled: boolean }
  | { readonly type: "extension-update"; readonly conversationId: string; readonly extensionKey: string }
  | { readonly type: "extension-remove"; readonly conversationId: string; readonly extensionKey: string }
  | { readonly type: "agent-control"; readonly conversationId: string; readonly action: "pause" | "resume" | "takeover" | "stop" };

export interface BrowserWorkspaceDesktopBridge {
  activate(conversationId: string, bounds: BrowserWorkspaceBounds): Promise<BrowserWorkspaceSnapshot>;
  hide(conversationId: string): Promise<BrowserWorkspaceSnapshot>;
  setBounds(conversationId: string, bounds: BrowserWorkspaceBounds): Promise<BrowserWorkspaceSnapshot>;
  newTab(conversationId: string, address?: string): Promise<BrowserWorkspaceSnapshot>;
  selectTab(conversationId: string, tabId: string): Promise<BrowserWorkspaceSnapshot>;
  closeTab(conversationId: string, tabId: string): Promise<BrowserWorkspaceSnapshot>;
  navigate(conversationId: string, tabId: string, address: string): Promise<BrowserWorkspaceSnapshot>;
  back(conversationId: string, tabId: string): Promise<BrowserWorkspaceSnapshot>;
  forward(conversationId: string, tabId: string): Promise<BrowserWorkspaceSnapshot>;
  reload(conversationId: string, tabId: string): Promise<BrowserWorkspaceSnapshot>;
  stop(conversationId: string, tabId: string): Promise<BrowserWorkspaceSnapshot>;
  setZoom(conversationId: string, tabId: string, factor: number): Promise<BrowserWorkspaceSnapshot>;
  addBookmark(conversationId: string, tabId: string): Promise<BrowserWorkspaceSnapshot>;
  removeBookmark(conversationId: string, bookmarkId: string): Promise<BrowserWorkspaceSnapshot>;
  chooseExtension(conversationId: string): Promise<BrowserWorkspaceSnapshot>;
  installExtension(conversationId: string, candidateId: string): Promise<BrowserWorkspaceSnapshot>;
  setExtensionEnabled(conversationId: string, extensionKey: string, enabled: boolean): Promise<BrowserWorkspaceSnapshot>;
  updateExtension(conversationId: string, extensionKey: string): Promise<BrowserWorkspaceSnapshot>;
  removeExtension(conversationId: string, extensionKey: string): Promise<BrowserWorkspaceSnapshot>;
  setAgentControl(conversationId: string, action: "pause" | "resume" | "takeover" | "stop"): Promise<BrowserWorkspaceSnapshot>;
  onState(listener: (snapshot: BrowserWorkspaceSnapshot) => void): () => void;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function isBrowserWorkspaceDesktopBridge(value: unknown): value is BrowserWorkspaceDesktopBridge {
  if (!isRecord(value)) return false;
  return (
    typeof value.activate === "function" &&
    typeof value.hide === "function" &&
    typeof value.setBounds === "function" &&
    typeof value.newTab === "function" &&
    typeof value.navigate === "function" &&
    typeof value.onState === "function"
  );
}

export function normalizeBrowserWorkspaceBounds(value: unknown): BrowserWorkspaceBounds | null {
  if (!isRecord(value)) return null;
  const numbers = ["x", "y", "width", "height"].map((key) => value[key]);
  if (!numbers.every((item) => typeof item === "number" && Number.isFinite(item))) return null;
  const x = Math.max(0, Math.floor(numbers[0] as number));
  const y = Math.max(0, Math.floor(numbers[1] as number));
  const width = Math.floor(numbers[2] as number);
  const height = Math.floor(numbers[3] as number);
  if (width < 1 || height < 1 || width > 16_384 || height > 16_384 || x > 16_384 || y > 16_384) return null;
  return { x, y, width, height };
}

export function normalizeBrowserWorkspaceConversationId(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const id = value.trim();
  return id.length > 0 && id.length <= BROWSER_WORKSPACE_LIMITS.maxConversationIdLength ? id : null;
}

export function normalizeBrowserWorkspaceAddress(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const input = value.trim();
  if (input.length === 0 || input.length > BROWSER_WORKSPACE_LIMITS.maxAddressLength || /[\u0000-\u001f\u007f]/u.test(input)) return null;
  let candidate = input;
  if (!/^[a-zA-Z][a-zA-Z\d+.-]*:/.test(candidate)) {
    const looksLikeHost = !/\s/u.test(candidate) && /(?:^localhost(?::\d+)?$|^\d{1,3}(?:\.\d{1,3}){3}(?::\d+)?$|^[\w.-]+\.[a-zA-Z]{2,}(?::\d+)?(?:\/.*)?$)/u.test(candidate);
    candidate = looksLikeHost ? `https://${candidate}` : `https://www.google.com/search?q=${encodeURIComponent(candidate)}`;
  }
  let url: URL;
  try { url = new URL(candidate); } catch { return null; }
  if (url.username.length > 0 || url.password.length > 0) return null;
  if (url.protocol === "https:") return url.toString();
  if (url.protocol !== "http:") return null;
  const host = url.hostname.toLowerCase();
  if (host !== "localhost" && host !== "127.0.0.1" && host !== "::1") return null;
  return url.toString();
}

export function parseBrowserWorkspaceCommand(value: unknown): BrowserWorkspaceCommand | null {
  if (!isRecord(value) || typeof value.type !== "string") return null;
  const conversationId = normalizeBrowserWorkspaceConversationId(value.conversationId);
  if (conversationId == null) return null;
  const tabId = typeof value.tabId === "string" && value.tabId.length > 0 && value.tabId.length <= 128 ? value.tabId : null;
  switch (value.type) {
    case "activate":
    case "set-bounds": {
      const bounds = normalizeBrowserWorkspaceBounds(value.bounds);
      return bounds == null ? null : { type: value.type, conversationId, bounds };
    }
    case "hide":
      return { type: "hide", conversationId };
    case "new-tab":
      return value.address === undefined || (typeof value.address === "string" && value.address.length <= BROWSER_WORKSPACE_LIMITS.maxAddressLength)
        ? { type: "new-tab", conversationId, ...(value.address === undefined ? {} : { address: value.address }) }
        : null;
    case "select-tab":
    case "close-tab":
    case "back":
    case "forward":
    case "reload":
    case "stop":
      return tabId == null ? null : { type: value.type, conversationId, tabId } as BrowserWorkspaceCommand;
    case "navigate":
      return tabId == null || typeof value.address !== "string" || value.address.length > BROWSER_WORKSPACE_LIMITS.maxAddressLength
        ? null
        : { type: "navigate", conversationId, tabId, address: value.address };
    case "set-zoom":
      return tabId == null || typeof value.factor !== "number" || !Number.isFinite(value.factor)
        ? null
        : { type: "set-zoom", conversationId, tabId, factor: value.factor };
    case "add-bookmark":
      return tabId == null ? null : { type: "add-bookmark", conversationId, tabId };
    case "remove-bookmark":
      return typeof value.bookmarkId === "string" && value.bookmarkId.length > 0 && value.bookmarkId.length <= 128
        ? { type: "remove-bookmark", conversationId, bookmarkId: value.bookmarkId }
        : null;
    case "extension-choose":
      return { type: "extension-choose", conversationId };
    case "extension-install":
      return typeof value.candidateId === "string" && value.candidateId.length > 0 && value.candidateId.length <= 128
        ? { type: "extension-install", conversationId, candidateId: value.candidateId }
        : null;
    case "extension-set-enabled":
      return typeof value.extensionKey === "string" && value.extensionKey.length > 0 && value.extensionKey.length <= 128 && typeof value.enabled === "boolean"
        ? { type: "extension-set-enabled", conversationId, extensionKey: value.extensionKey, enabled: value.enabled }
        : null;
    case "extension-update":
    case "extension-remove":
      return typeof value.extensionKey === "string" && value.extensionKey.length > 0 && value.extensionKey.length <= 128
        ? { type: value.type, conversationId, extensionKey: value.extensionKey }
        : null;
    case "agent-control":
      return ["pause", "resume", "takeover", "stop"].includes(String(value.action))
        ? { type: "agent-control", conversationId, action: value.action as "pause" | "resume" | "takeover" | "stop" }
        : null;
    default:
      return null;
  }
}

export interface BrowserWorkspacePaneProps {
  readonly bridge: BrowserWorkspaceDesktopBridge;
  readonly conversationId: string;
  readonly onClose: () => void;
}

type BrowserSidebar = "bookmarks" | "history" | "downloads" | "extensions" | null;

function boundsFor(element: HTMLElement): { x: number; y: number; width: number; height: number } | null {
  const rect = element.getBoundingClientRect();
  const width = Math.floor(rect.width), height = Math.floor(rect.height);
  if (width < 1 || height < 1) return null;
  return { x: Math.max(0, Math.floor(rect.x)), y: Math.max(0, Math.floor(rect.y)), width, height };
}

function displayHost(url: string): string {
  if (url === "about:blank") return "New tab";
  try { return new URL(url).hostname || url; } catch { return url; }
}

export function BrowserWorkspacePane({ bridge, conversationId, onClose }: BrowserWorkspacePaneProps) {
  const contentRef = useRef<HTMLDivElement | null>(null);
  const addressFocused = useRef(false);
  const [snapshot, setSnapshot] = useState<BrowserWorkspaceSnapshot | null>(null);
  const [address, setAddress] = useState("");
  const [sidebar, setSidebar] = useState<BrowserSidebar>(null);
  const [error, setError] = useState<string | null>(null);

  const activeTab = useMemo(
    () => snapshot?.tabs.find((tab) => tab.id === snapshot.activeTabId) ?? null,
    [snapshot],
  );

  useEffect(() => {
    if (!addressFocused.current) setAddress(activeTab?.url === "about:blank" ? "" : activeTab?.url ?? "");
  }, [activeTab?.id, activeTab?.url]);

  const settle = useCallback(async (operation: Promise<BrowserWorkspaceSnapshot>) => {
    try {
      const next = await operation;
      setSnapshot(next);
      setError(null);
      return next;
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      return null;
    }
  }, []);

  useLayoutEffect(() => {
    let disposed = false;
    const element = contentRef.current;
    if (element == null) return;
    const publishBounds = (activate: boolean) => {
      const bounds = boundsFor(element);
      if (bounds == null || disposed) return;
      const operation = activate
        ? bridge.activate(conversationId, bounds)
        : bridge.setBounds(conversationId, bounds);
      void operation.then((next) => {
        if (!disposed) {
          setSnapshot(next);
          setError(null);
        }
      }).catch((cause: unknown) => {
        if (!disposed) setError(cause instanceof Error ? cause.message : String(cause));
      });
    };
    const unsubscribe = bridge.onState((next) => {
      if (!disposed && next.conversationId === conversationId) setSnapshot(next);
    });
    const observer = new ResizeObserver(() => publishBounds(false));
    observer.observe(element);
    publishBounds(true);
    return () => {
      disposed = true;
      observer.disconnect();
      unsubscribe();
      void bridge.hide(conversationId).catch(() => {});
    };
  }, [bridge, conversationId]);

  const submitAddress = (event: FormEvent) => {
    event.preventDefault();
    if (activeTab == null || address.trim().length === 0) return;
    void settle(bridge.navigate(conversationId, activeTab.id, address));
  };

  const setSidebarMode = (next: Exclude<BrowserSidebar, null>) => {
    setSidebar((current) => current === next ? null : next);
  };

  return (
    <section aria-label="Browser workspace" className="sand-browser-workspace" data-testid="browser-workspace">
      <div className="sand-browser-workspace__tabs" role="tablist" aria-label="Browser tabs">
        {(snapshot?.tabs ?? []).map((tab) => (
          <button
            aria-selected={tab.id === snapshot?.activeTabId}
            className="sand-browser-workspace__tab"
            data-active={tab.id === snapshot?.activeTabId ? "true" : undefined}
            key={tab.id}
            onClick={() => void settle(bridge.selectTab(conversationId, tab.id))}
            role="tab"
            type="button"
          >
            <span className="sand-browser-workspace__favicon" aria-hidden="true" />
            <span className="sand-browser-workspace__tab-title">{tab.title || displayHost(tab.url)}</span>
            <span
              aria-label={`Close ${tab.title || "tab"}`}
              className="sand-browser-workspace__tab-close"
              onClick={(event) => {
                event.stopPropagation();
                void settle(bridge.closeTab(conversationId, tab.id));
              }}
              role="button"
              tabIndex={0}
            >×</span>
          </button>
        ))}
        <button aria-label="New browser tab" className="sand-browser-workspace__icon-button" onClick={() => void settle(bridge.newTab(conversationId))} type="button">+</button>
        <button aria-label="Close browser workspace" className="sand-browser-workspace__icon-button sand-browser-workspace__close" onClick={onClose} type="button">×</button>
      </div>

      <div className="sand-browser-workspace__toolbar">
        <button aria-label="Back" disabled={!activeTab?.canGoBack} onClick={() => activeTab && void settle(bridge.back(conversationId, activeTab.id))} type="button">←</button>
        <button aria-label="Forward" disabled={!activeTab?.canGoForward} onClick={() => activeTab && void settle(bridge.forward(conversationId, activeTab.id))} type="button">→</button>
        <button
          aria-label={activeTab?.loading ? "Stop loading" : "Reload"}
          disabled={activeTab == null}
          onClick={() => {
            if (activeTab == null) return;
            void settle(activeTab.loading ? bridge.stop(conversationId, activeTab.id) : bridge.reload(conversationId, activeTab.id));
          }}
          type="button"
        >{activeTab?.loading ? "■" : "↻"}</button>
        <form className="sand-browser-workspace__address-form" onSubmit={submitAddress}>
          <input
            aria-label="Address and search"
            autoComplete="off"
            onBlur={() => { addressFocused.current = false; }}
            onChange={(event) => setAddress(event.currentTarget.value)}
            onFocus={() => { addressFocused.current = true; }}
            placeholder="Search or enter address"
            spellCheck={false}
            value={address}
          />
        </form>
        <button aria-label="Bookmark current page" disabled={activeTab == null || activeTab.url === "about:blank"} onClick={() => activeTab && void settle(bridge.addBookmark(conversationId, activeTab.id))} type="button">☆</button>
        <button aria-label="Zoom out" disabled={activeTab == null} onClick={() => activeTab && void settle(bridge.setZoom(conversationId, activeTab.id, activeTab.zoomFactor - 0.1))} type="button">−</button>
        <span className="sand-browser-workspace__zoom">{Math.round((activeTab?.zoomFactor ?? 1) * 100)}%</span>
        <button aria-label="Zoom in" disabled={activeTab == null} onClick={() => activeTab && void settle(bridge.setZoom(conversationId, activeTab.id, activeTab.zoomFactor + 0.1))} type="button">+</button>
        <button aria-pressed={sidebar === "bookmarks"} onClick={() => setSidebarMode("bookmarks")} type="button">Bookmarks</button>
        <button aria-pressed={sidebar === "history"} onClick={() => setSidebarMode("history")} type="button">History</button>
        <button aria-pressed={sidebar === "downloads"} onClick={() => setSidebarMode("downloads")} type="button">Downloads</button>
        <button aria-pressed={sidebar === "extensions"} onClick={() => setSidebarMode("extensions")} type="button">Extensions</button>
        {snapshot == null ? null : (
          <>
            <span className="sand-browser-workspace__agent-state" aria-live="polite">
              Agent: {snapshot.agentControl.activeOperationId != null ? "working" : snapshot.agentControl.state} · task {snapshot.agentControl.taskStatus}
            </span>
            {snapshot.agentControl.state === "available" ? (
              <>
                <button onClick={() => void settle(bridge.setAgentControl(conversationId, "pause"))} type="button">Pause agent</button>
                <button onClick={() => void settle(bridge.setAgentControl(conversationId, "takeover"))} type="button">Take over</button>
                <button onClick={() => void settle(bridge.setAgentControl(conversationId, "stop"))} type="button">Stop agent</button>
              </>
            ) : snapshot.agentControl.state === "paused" ? (
              <button onClick={() => void settle(bridge.setAgentControl(conversationId, "resume"))} type="button">Resume agent</button>
            ) : (
              <button onClick={() => void settle(bridge.setAgentControl(conversationId, "resume"))} type="button">Allow agent control</button>
            )}
          </>
        )}
      </div>

      {error == null && activeTab?.error == null ? null : (
        <div className="sand-browser-workspace__error" role="status">
          {error ?? `${activeTab?.error?.code}: ${activeTab?.error?.description}`}
        </div>
      )}

      {sidebar == null ? null : (
        <div className="sand-browser-workspace__drawer" aria-label={sidebar}>
          {sidebar === "bookmarks" ? (
            (snapshot?.bookmarks.length ?? 0) === 0 ? <p>No bookmarks yet.</p> :
            snapshot?.bookmarks.map((bookmark) => (
              <div className="sand-browser-workspace__drawer-row" key={bookmark.id}>
                <button onClick={() => activeTab && void settle(bridge.navigate(conversationId, activeTab.id, bookmark.url))} type="button">
                  <strong>{bookmark.title}</strong><small>{bookmark.url}</small>
                </button>
                <button aria-label={`Remove bookmark ${bookmark.title}`} onClick={() => void settle(bridge.removeBookmark(conversationId, bookmark.id))} type="button">×</button>
              </div>
            ))
          ) : sidebar === "history" ? (
            (snapshot?.history.length ?? 0) === 0 ? <p>No history yet.</p> :
            [...(snapshot?.history ?? [])].reverse().slice(0, 100).map((entry) => (
              <button className="sand-browser-workspace__history-row" key={entry.id} onClick={() => activeTab && void settle(bridge.navigate(conversationId, activeTab.id, entry.url))} type="button">
                <strong>{entry.title}</strong><small>{entry.url}</small>
              </button>
            ))
          ) : sidebar === "downloads" ? (
            (snapshot?.downloads.length ?? 0) === 0 ? <p>No downloads yet.</p> :
            [...(snapshot?.downloads ?? [])].reverse().map((download) => (
              <div className="sand-browser-workspace__download-row" key={download.id}>
                <strong>{download.fileName}</strong>
                <small>{download.state} · {download.receivedBytes}/{download.totalBytes || "?"} bytes</small>
              </div>
            ))
          ) : (
            <>
              <button onClick={() => void settle(bridge.chooseExtension(conversationId))} type="button">Choose unpacked extension…</button>
              {snapshot?.extensionCandidate == null ? null : (
                <div className="sand-browser-workspace__drawer-row" role="group" aria-label="Extension install confirmation">
                  <div>
                    <strong>{snapshot.extensionCandidate.name} {snapshot.extensionCandidate.version}</strong>
                    <small>Permissions: {snapshot.extensionCandidate.permissions.join(", ") || "none"}</small>
                    <small>Host access: {snapshot.extensionCandidate.hostPermissions.join(", ") || "none"}</small>
                    <small>{snapshot.extensionCandidate.integrity}</small>
                  </div>
                  <button onClick={() => void settle(bridge.installExtension(conversationId, snapshot.extensionCandidate!.candidateId))} type="button">Install</button>
                </div>
              )}
              {(snapshot?.extensions.length ?? 0) === 0 ? <p>No browser extensions installed.</p> :
                snapshot?.extensions.map((extension) => (
                  <div className="sand-browser-workspace__drawer-row" key={extension.key}>
                    <div>
                      <strong>{extension.name} {extension.version}</strong>
                      <small>{extension.enabled ? "Enabled" : "Disabled"} · {extension.integrity}</small>
                      {extension.error == null ? null : <small role="status">{extension.error}</small>}
                    </div>
                    <button onClick={() => void settle(bridge.setExtensionEnabled(conversationId, extension.key, !extension.enabled))} type="button">
                      {extension.enabled ? "Disable" : "Enable"}
                    </button>
                    <button onClick={() => void settle(bridge.updateExtension(conversationId, extension.key))} type="button">Update</button>
                    <button onClick={() => void settle(bridge.removeExtension(conversationId, extension.key))} type="button">Uninstall</button>
                  </div>
                ))
              }
            </>
          )}
        </div>
      )}

      <div className="sand-browser-workspace__content" data-testid="browser-workspace-content" ref={contentRef}>
        {snapshot == null ? <div className="sand-browser-workspace__placeholder">Opening browser…</div> : null}
      </div>
    </section>
  );
}
