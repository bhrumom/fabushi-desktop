import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, type FormEvent } from "react";
import type {
  BrowserWorkspaceDesktopBridge,
  BrowserWorkspaceSnapshot,
} from "../../../../../source/shared/browser-workspace";
import "./workspace.css";

export interface BrowserWorkspacePaneProps {
  readonly bridge: BrowserWorkspaceDesktopBridge;
  readonly conversationId: string;
  readonly onClose: () => void;
}

type BrowserSidebar = "bookmarks" | "history" | "downloads" | null;

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
        {snapshot == null ? null : (
          <>
            <span className="sand-browser-workspace__agent-state" aria-live="polite">
              Agent: {snapshot.agentControl.activeOperationId != null ? "working" : snapshot.agentControl.state}
            </span>
            {snapshot.agentControl.state === "available" ? (
              <>
                <button onClick={() => void settle(bridge.setAgentControl(conversationId, "pause"))} type="button">Pause agent</button>
                <button onClick={() => void settle(bridge.setAgentControl(conversationId, "takeover"))} type="button">Take over</button>
                <button onClick={() => void settle(bridge.setAgentControl(conversationId, "stop"))} type="button">Stop agent</button>
              </>
            ) : (
              <button onClick={() => void settle(bridge.setAgentControl(conversationId, "resume"))} type="button">Resume agent</button>
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
          ) : (
            (snapshot?.downloads.length ?? 0) === 0 ? <p>No downloads yet.</p> :
            [...(snapshot?.downloads ?? [])].reverse().map((download) => (
              <div className="sand-browser-workspace__download-row" key={download.id}>
                <strong>{download.fileName}</strong>
                <small>{download.state} · {download.receivedBytes}/{download.totalBytes || "?"} bytes</small>
              </div>
            ))
          )}
        </div>
      )}

      <div className="sand-browser-workspace__content" data-testid="browser-workspace-content" ref={contentRef}>
        {snapshot == null ? <div className="sand-browser-workspace__placeholder">Opening browser…</div> : null}
      </div>
    </section>
  );
}
