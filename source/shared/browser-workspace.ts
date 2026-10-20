export const BROWSER_WORKSPACE_INVOKE_CHANNEL = "sand:browser-workspace";
export const BROWSER_WORKSPACE_STATE_CHANNEL = "sand:browser-workspace-state";

export const BROWSER_WORKSPACE_LIMITS = Object.freeze({
  maxConversationIdLength: 256,
  maxAddressLength: 4096,
  maxTabs: 20,
  maxBookmarks: 500,
  maxHistory: 500,
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
  readonly startedAtMs: number;
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
  | { readonly type: "remove-bookmark"; readonly conversationId: string; readonly bookmarkId: string };

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
  onState(listener: (snapshot: BrowserWorkspaceSnapshot) => void): () => void;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
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
    default:
      return null;
  }
}
