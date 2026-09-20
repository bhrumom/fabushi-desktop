import {
  mahayanaHostFeatures,
  type MahayanaHostFeatureId,
  type MahayanaHostFeatureState,
} from "@fabushi/shared";
import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
} from "react";
import styles from "./host.module.css";
import type {
  AgentMode,
  AgentPeerMessage,
  AsyncTaskSummary,
  ApprovalRequestedEvent,
  AttachmentContext,
  AutomationSummary,
  AuthState,
  BrowserLoginAttempt,
  BotSummary,
  CapabilitySummary,
  ComputerSnapshot,
  ComputerStatus,
  GroupSummary,
  ConnectorSummary,
  ConversationSummary,
  ErrorTray,
  ListenerIntegrationSummary,
  ListenerPlatform,
  MemoryKind,
  MemoryRecord,
  MessageDraft,
  ProductHostSettings,
  RuntimeCommand,
  SearchMediaMatch,
  SearchMessageMatch,
  SkillSummary,
  SkillTeamSummary,
  SubagentSummary,
  TeachRecordingResult,
  TeachRecordingStatus,
  UpdateState,
  WorkflowSummary,
  WorkflowTrigger,
} from "../../lib/mahayana-host/contracts";
import { ElectronMahayanaHostTransport, isElectronMahayanaHostAvailable } from "../../lib/mahayana-host/electron-transport";
import { MockMahayanaHostTransport } from "../../lib/mahayana-host/mock-transport";
import { WasmMahayanaHostTransport } from "../../lib/mahayana-host/wasm-transport";
import type { MahayanaHostTransport } from "../../lib/mahayana-host/transport";
import { MahayanaCoordinator } from "../../lib/mahayana-host/coordinator";
import {
  RemoteComputerDesktopController,
  type RemoteComputerDesktopState,
} from "../../lib/remote-computer/desktop-peer";
import {
  buildModeTransitionNote,
  clampAgentMessage,
  estimateTextTokens,
  formatNumberedLines,
  normalizeAutomationSchedule,
} from "../../lib/fabushi-runtime/agent-utils";
import {
  AgentNotificationPolicy,
  buildAgentNotification,
  type AgentNotificationSnapshot,
} from "../../lib/fabushi-runtime/agent-notifications";
import type {
  BoxSecretsStatus,
  CloudAgentInfo,
  ForeverBoxStatus,
} from "../../lib/fabushi-runtime/capability-provider";
import type {
  SharingState,
  SharedRoomInvite,
} from "../../lib/fabushi-runtime/collaboration";
import {
  invokeNativeDesktop,
  markNativeDeepLinksReady,
  nativeDesktopBridge,
  nativeOnboardingSeen,
  rememberNativeOnboarding,
  subscribeNativeDesktopEvents,
  syncNativeTheme,
  type NativeDeepLink,
  type NativeDesktopEnvironment,
  type NativeDiskAudit,
  type NativeOfflineAsrProgress,
  type NativeOfflineAsrStatus,
} from "../../lib/fabushi-runtime/native-desktop";
import {
  ExtensionStudio,
  MarketplaceTabs,
  type MarketplaceSection,
} from "./extension-studio";
import {
  RichMessage,
  TranscriptCardView,
  type TranscriptCardEntry,
} from "./rich-transcript";
import {
  BotMark,
  botMarkStateFromActivity,
  type BotMarkColor,
  type BotMarkShape,
  type BotMarkState,
} from "./bot-mark";
import { marketplaceApps as marketplaceCatalog } from "../../lib/marketplace";
import {
  installMarketplaceApp,
  marketplaceAppInstallAction,
  marketplaceInstallActionLabel,
  readMarketplaceInstallRecords,
  subscribeMarketplaceInstallState,
  type MarketplaceInstallRecord,
} from "../../lib/marketplace-install-state";
import { GroupAvatarStack, GroupChatPanel, GroupEditor } from "./group-chat-panel";
import { AgentWorkflowPanel } from "./agent-workflow-panel";

const defaultMiniAppId = "global-dharma";
const ONBOARDING_COMPLETE_KEY = "fabushi.host.onboarding-complete.v1";
const ATTACHMENT_BYTE_LIMIT = 25 * 1024 * 1024;
const VIDEO_ATTACHMENT_BYTE_LIMIT = 200 * 1024 * 1024;
const ATTACHMENT_TEXT_PREVIEW_BYTES = 64 * 1024;

function attachmentLooksLikeVideo(name: string): boolean {
  return /\.(?:mp4|mov|m4v|webm|mkv|avi|mpg|mpeg)$/i.test(name);
}

function attachmentLooksTextPreviewable(file: File): boolean {
  if (file.type.startsWith("text/")) return true;
  return /\.(?:txt|md|markdown|mdc|csv|tsv|json|jsonl|ya?ml|toml|xml|html?|css|jsx?|tsx?|py|rs|go|java|kt|swift|c|h|cpp|hpp|sh|bash|zsh|fish|log|sql|ini|conf)$/i.test(file.name);
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error ?? new Error(`无法读取 ${file.name}`));
    reader.onload = () => {
      const result = typeof reader.result === "string" ? reader.result : "";
      const comma = result.indexOf(",");
      if (comma < 0) reject(new Error(`无法编码 ${file.name}`));
      else resolve(result.slice(comma + 1));
    };
    reader.readAsDataURL(file);
  });
}

const marketplaceApps = marketplaceCatalog.map((app) => ({
  ...app,
  title: app.name,
  description: app.subtitle,
  glyph: app.icon,
}));

type FeatureStates = Record<
  MahayanaHostFeatureId,
  MahayanaHostFeatureState
>;

type AgentActivity = {
  id: string;
  operationId?: string;
  kind: string;
  title: string;
  detail?: string;
  status: "running" | "completed" | "failed";
};

type HostTranscriptEntry = {
  id?: string;
  kind: "message" | "action" | "thinking";
  role: "user" | "assistant";
  text: string;
  operationId?: string;
  streaming?: boolean;
  title?: string;
  detail?: string;
  status?: "running" | "completed" | "failed";
};

type SettingsSection = "general" | "mcp" | "usage" | "updates";
type ThemePreference = "system" | "light" | "dark";

type HostPreferences = {
  theme: ThemePreference;
  notifyOnUpdates: boolean;
  autoUpdateWhenIdle: boolean;
  localExecution: boolean;
  routeEgressLocally: boolean;
  securityKeys: boolean;
  webauthnProxyEnabled: boolean;
  localToolPermission: "never" | "ask" | "always";
  remoteControlEnabled: boolean;
  aiComputerControlEnabled: boolean;
  autoReviewRules: Array<{ id: string; behavior: "allow" | "ask"; text: string }>;
};

const defaultPreferences: HostPreferences = {
  theme: "system",
  notifyOnUpdates: true,
  autoUpdateWhenIdle: true,
  localExecution: true,
  routeEgressLocally: false,
  securityKeys: false,
  webauthnProxyEnabled: false,
  localToolPermission: "ask",
  remoteControlEnabled: false,
  aiComputerControlEnabled: true,
  autoReviewRules: [],
};

function serializeProductHostSettings(settings: ProductHostSettings): string {
  return JSON.stringify({
    notifications: settings.notifications,
    autoUpdateWhenIdle: settings.autoUpdateWhenIdle,
    localExecution: settings.localExecution,
    routeEgressLocally: settings.routeEgressLocally,
    securityKeys: settings.securityKeys,
    webauthnProxyEnabled: settings.webauthnProxyEnabled,
    localToolPermission: settings.localToolPermission,
    remoteControlEnabled: settings.remoteControlEnabled,
    aiComputerControlEnabled: settings.aiComputerControlEnabled,
    autoReviewRules: settings.autoReviewRules,
    inferenceProvider: settings.inferenceProvider,
    sandboxRuntime: settings.sandboxRuntime,
  });
}

const modelOptions = [
  { value: "auto", label: "自动选择" },
  { value: "deepseek-chat", label: "DeepSeek Chat" },
  { value: "deepseek-reasoner", label: "DeepSeek Reasoner" },
] as const;

const listenerOptions: Array<{
  value: ListenerPlatform;
  label: string;
  defaultEvent: string;
}> = [
  { value: "github", label: "GitHub", defaultEvent: "pull_request.opened" },
  { value: "git", label: "Git", defaultEvent: "commit.created" },
  { value: "slack", label: "Slack", defaultEvent: "mention.created" },
  { value: "teams", label: "Microsoft Teams", defaultEvent: "mention.created" },
  { value: "linear", label: "Linear", defaultEvent: "issue.updated" },
  { value: "sentry", label: "Sentry", defaultEvent: "issue.regressed" },
  { value: "pagerduty", label: "PagerDuty", defaultEvent: "incident.triggered" },
];

function automationTriggerLabel(automation: AutomationSummary): string {
  const trigger = automation.trigger;
  if (trigger?.kind === "event") {
    const source = listenerOptions.find(
      (option) => option.value === trigger.source,
    )?.label ?? trigger.source;
    return `${source} · ${trigger.event}`;
  }
  return automation.trigger?.kind === "schedule"
    ? automation.trigger.schedule
    : automation.schedule;
}

function automationRunLabel(automation: AutomationSummary): string {
  if (!automation.enabled) return "已暂停";
  if (automation.trigger?.kind === "event") return "等待事件";
  return automation.nextRunAtMs
    ? `下次 ${new Date(automation.nextRunAtMs).toLocaleString()}`
    : "等待调度";
}

function objectValue(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function mcpServerName(value: unknown): string {
  const server = objectValue(value);
  for (const key of ["name", "server", "id", "serverName", "identifier"]) {
    if (typeof server[key] === "string" && String(server[key]).trim()) return String(server[key]);
  }
  return "MCP Server";
}

function mcpServerStatus(value: unknown): string {
  const server = objectValue(value);
  for (const key of ["status", "state", "authStatus", "connectionStatus"]) {
    if (typeof server[key] === "string" && String(server[key]).trim()) return String(server[key]);
  }
  return "unknown";
}

function mcpServerTools(value: unknown): string[] {
  const tools = objectValue(value).tools;
  if (!Array.isArray(tools)) return [];
  return tools.map((tool) => {
    if (typeof tool === "string") return tool;
    const record = objectValue(tool);
    return typeof record.name === "string" ? record.name : "tool";
  });
}

function formatElapsedMs(value: number): string {
  const seconds = Math.max(0, Math.floor(value / 1000));
  const minutes = Math.floor(seconds / 60);
  return `${String(minutes).padStart(2, "0")}:${String(seconds % 60).padStart(2, "0")}`;
}

function updateStateCopy(state: UpdateState): {
  title: string;
  detail: string;
  action: "check" | "install" | null;
} {
  switch (state.type) {
    case "loading":
      return { title: "正在读取更新状态", detail: "请稍候…", action: null };
    case "disabled":
      return { title: "当前构建不支持自动更新", detail: state.reason, action: null };
    case "checking":
      return { title: "正在检查更新", detail: "正在联系更新服务…", action: null };
    case "available":
      return { title: `发现版本 ${state.version}`, detail: state.notes || "可以下载并安装更新。", action: "install" };
    case "downloading":
      return { title: `正在下载 ${state.version}`, detail: state.progress === undefined ? "下载中…" : `${state.progress}%`, action: null };
    case "staging":
      return { title: `正在准备 ${state.version}`, detail: "安装包正在验证。", action: null };
    case "ready":
      return { title: `${state.version} 已可安装`, detail: "重新启动客户端以完成更新。", action: "install" };
    case "upToDate":
      return { title: "当前已是最新版本", detail: `Fabushi ${state.version} · Mahayana Runtime`, action: "check" };
    case "error":
      return { title: "检查更新失败", detail: state.message, action: "check" };
  }
}

function createInitialFeatureStates(): FeatureStates {
  return Object.fromEntries(
    mahayanaHostFeatures.map((feature) => [feature.id, "pending"]),
  ) as FeatureStates;
}

function Icon({
  name,
  size = 18,
}: {
  name:
    | "plus"
    | "search"
    | "plugins"
    | "settings"
    | "send"
    | "close"
    | "stop"
    | "shield"
    | "attach"
    | "network"
    | "automation"
    | "computer"
    | "bell";
  size?: number;
}) {
  const paths = {
    plus: <path d="M12 5v14M5 12h14" />,
    search: <><circle cx="11" cy="11" r="7" /><path d="m20 20-4-4" /></>,
    plugins: <path d="M8 3v4H4v4h4v4H4v4h4v2h4v-4h4v4h4v-4h-4v-4h4V9h-4V5h-4v4h-2V3H8Zm2 8h4v4h-4v-4Z" />,
    settings: <><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.83 2.83-.06-.06A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 .6 1.7 1.7 0 0 0-.4 1.1V21h-4v-.1A1.7 1.7 0 0 0 8.6 19.4a1.7 1.7 0 0 0-1.88.34l-.06.06-2.83-2.83.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.5-1H3v-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.34-1.88l-.06-.06 2.83-2.83.06.06A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.5V3h4v.1A1.7 1.7 0 0 0 15 4.6a1.7 1.7 0 0 0 1.88-.34l.06-.06 2.83 2.83-.06.06A1.7 1.7 0 0 0 19.4 9a1.7 1.7 0 0 0 1.5 1h.1v4h-.1a1.7 1.7 0 0 0-1.5 1Z" /></>,
    send: <path d="m4 4 17 8-17 8 3-8-3-8Zm3 8h14" />,
    close: <path d="m6 6 12 12M18 6 6 18" />,
    stop: <rect x="6" y="6" width="12" height="12" rx="2" />,
    shield: <path d="M12 3 5 6v5c0 4.5 2.8 8.3 7 10 4.2-1.7 7-5.5 7-10V6l-7-3Z" />,
    attach: <path d="m20.5 11.5-8.9 8.9a5 5 0 0 1-7.1-7.1l9.6-9.6a3.5 3.5 0 0 1 5 5l-9.7 9.7a2 2 0 0 1-2.8-2.8l8.9-8.9" />,
    network: <><circle cx="12" cy="5" r="2.5" /><circle cx="5" cy="18" r="2.5" /><circle cx="19" cy="18" r="2.5" /><path d="m10.8 7.2-4.5 8.4m6.9-8.4 4.5 8.4M7.5 18h9" /></>,
    automation: <><circle cx="12" cy="12" r="8" /><path d="M12 8v4l3 2M5.5 5.5 3 3m15.5 2.5L21 3" /></>,
    computer: <><rect x="3" y="4" width="18" height="13" rx="2" /><path d="M8 21h8m-4-4v4" /></>,
    bell: <><path d="M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9" /><path d="M10 21h4" /></>,
  };
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={name === "plugins" ? "currentColor" : "none"}
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {paths[name]}
    </svg>
  );
}

interface ConfirmAction {
  title: string;
  message: string;
  confirmLabel: string;
  tone: "danger" | "warning";
  action: () => void | Promise<void>;
}

interface HostClientProps {
  onAuthStateChange?: (state: AuthState) => void;
}

export default function HostClient({ onAuthStateChange }: HostClientProps) {
  const screenshotMode = new URLSearchParams(window.location.search).get("screenshot");
  const screenshotHasMiniApp = screenshotMode === "miniapp";
  const screenshotComputerOpen = ["computer", "running", "miniapp"].includes(screenshotMode ?? "");
  const configuredHostMode =
    typeof process !== "undefined"
      ? process.env.NEXT_PUBLIC_MAHAYANA_HOST_MODE
      : undefined;
  const hostTestMode =
    configuredHostMode === "test" ||
    (window as Window & { __FABUSHI_HOST_TEST__?: boolean }).__FABUSHI_HOST_TEST__ === true;
  const transport = useMemo<MahayanaHostTransport>(() => {
    // Screenshot fixtures intentionally stay deterministic, but the real desktop
    // login surface must authenticate the same Electron/Rust Host that the
    // Messenger shell will use after the handoff. Using the mock here makes the
    // browser flow appear successful while the authoritative Host remains signed out.
    if (screenshotMode !== null) return new MockMahayanaHostTransport({ authenticated: true });
    // The standalone Host journey uses deterministic fixtures for auth and
    // marketplace contract coverage. Production browser sessions always use
    // Mahayana WebAssembly below, so this branch cannot hide a real runtime.
    if (hostTestMode) return new MockMahayanaHostTransport({ authenticated: false });
    if (isElectronMahayanaHostAvailable()) return new ElectronMahayanaHostTransport();
    return new WasmMahayanaHostTransport();
  }, [hostTestMode, screenshotMode]);
  const coordinator = useMemo(() => new MahayanaCoordinator(transport), [transport]);
  const requestSequence = useRef(0);
  const attachmentInput = useRef<HTMLInputElement>(null);
  const [hostStatus, setHostStatus] = useState("initializing");
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<HostTranscriptEntry[]>([]);
  const [installedMiniAppRecords, setInstalledMiniAppRecords] = useState<Record<string, MarketplaceInstallRecord>>(
    () => readMarketplaceInstallRecords(),
  );
  const [installedMiniApps, setInstalledMiniApps] = useState<Set<string>>(
    () => new Set([
      ...Object.keys(readMarketplaceInstallRecords()),
      ...(screenshotHasMiniApp ? [defaultMiniAppId] : []),
    ]),
  );
  useEffect(() => subscribeMarketplaceInstallState((records) => {
    setInstalledMiniAppRecords(records);
    setInstalledMiniApps(new Set([
      ...Object.keys(records),
      ...(screenshotHasMiniApp ? [defaultMiniAppId] : []),
    ]));
  }), [screenshotHasMiniApp]);
  const [openedMiniApp, setOpenedMiniApp] = useState<string | null>(
    screenshotHasMiniApp ? defaultMiniAppId : null,
  );
  const [openedMiniAppHtml, setOpenedMiniAppHtml] = useState<string | null>(
    screenshotHasMiniApp
      ? '<!doctype html><html lang="zh-CN"><body style="font:15px system-ui;background:#101722;color:#edf3ff;padding:20px"><h1>全球法流</h1><p>MiniApp 已在隔离容器中打开，可通过受控能力与宿主通信。</p></body></html>'
      : null,
  );
  const [approval, setApproval] = useState<ApprovalRequestedEvent | null>(null);
  const [approvalState, setApprovalState] = useState("not-requested");
  const [activeOperationId, setActiveOperationId] = useState<string | null>(
    screenshotMode === "running" ? "operation-screenshot" : null,
  );
  const [operationState, setOperationState] = useState(screenshotMode === "running" ? "running" : "idle");
  const [chatDispatching, setChatDispatching] = useState(false);
  const [sessionState, setSessionState] = useState("active");
  const [featureStates, setFeatureStates] = useState<FeatureStates>(
    createInitialFeatureStates,
  );
  const [error, setError] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);
  const [confirmBusy, setConfirmBusy] = useState(false);
  const overlayReturnFocusRef = useRef<HTMLElement | null>(null);
  const overlayWasOpenRef = useRef(false);
  const [marketplaceOpen, setMarketplaceOpen] = useState(screenshotMode === "marketplace");
  const [marketplaceSection, setMarketplaceSection] = useState<MarketplaceSection>("apps");
  const [networkOpen, setNetworkOpen] = useState(false);
  const [networkView, setNetworkView] = useState<"agents" | "rooms" | "workspace">("agents");
  const [networkTargetId, setNetworkTargetId] = useState("");
  const [networkMessage, setNetworkMessage] = useState("");
  const [networkPriority, setNetworkPriority] = useState(false);
  const [peerMessages, setPeerMessages] = useState<AgentPeerMessage[]>([]);
  const [sharingState, setSharingState] = useState<SharingState>({ scope: "local-device", rooms: [], joinRequests: [] });
  const [sharingBusy, setSharingBusy] = useState(false);
  const [sharingError, setSharingError] = useState<string | null>(null);
  const [sharedRoomName, setSharedRoomName] = useState("");
  const [sharedInviteToken, setSharedInviteToken] = useState("");
  const [lastSharedInvite, setLastSharedInvite] = useState<SharedRoomInvite | null>(null);
  const [workspaceStatus, setWorkspaceStatus] = useState<ForeverBoxStatus | null>(null);
  const [workspaceSecrets, setWorkspaceSecrets] = useState<BoxSecretsStatus | null>(null);
  const [workspaceDiskAudit, setWorkspaceDiskAudit] = useState<NativeDiskAudit | null>(null);
  const [workspaceEgressAvailable, setWorkspaceEgressAvailable] = useState(false);
  const [workspaceAgentNetworkEnabled, setWorkspaceAgentNetworkEnabled] = useState(false);
  const [workspaceBusy, setWorkspaceBusy] = useState(false);
  const [workspaceError, setWorkspaceError] = useState<string | null>(null);
  const [backgroundWorkingAgents, setBackgroundWorkingAgents] = useState<Set<string>>(() => new Set());
  const [backgroundPreviews, setBackgroundPreviews] = useState<Record<string, { agentId: string; text: string }>>({});
  const [lastBroadcastResult, setLastBroadcastResult] = useState<{ total: number; scheduled: number } | null>(null);
  const [subagentAgentId, setSubagentAgentId] = useState<string | null>(null);
  const [subagents, setSubagents] = useState<SubagentSummary[]>([]);
  const [asyncTasks, setAsyncTasks] = useState<AsyncTaskSummary[]>([]);
  const [teachStatus, setTeachStatus] = useState<TeachRecordingStatus>({ state: "idle", maxDurationMs: 600_000 });
  const [teachResult, setTeachResult] = useState<TeachRecordingResult | null>(null);
  const [teachClockMs, setTeachClockMs] = useState(() => Date.now());
  const [trayOpen, setTrayOpen] = useState(false);
  const [trays, setTrays] = useState<ErrorTray[]>([]);
  const [automationOpen, setAutomationOpen] = useState(screenshotMode === "automation");
  const [computerOpen, setComputerOpen] = useState(screenshotComputerOpen);
  const [computerStatus, setComputerStatus] = useState<ComputerStatus | null>(null);
  const [computerSnapshot, setComputerSnapshot] = useState<ComputerSnapshot | null>(null);
  const [computerRefreshing, setComputerRefreshing] = useState(false);
  const [remoteDesktopState, setRemoteDesktopState] = useState<RemoteComputerDesktopState | null>(null);
  const [agentSettingsOpen, setAgentSettingsOpen] = useState(false);
  const [agentName, setAgentName] = useState("");
  const [agentTitle, setAgentTitle] = useState("");
  const [agentDescription, setAgentDescription] = useState("");
  const [agentNotifications, setAgentNotifications] = useState(true);
  const [agentMemories, setAgentMemories] = useState<MemoryRecord[]>([]);
  const [agentMemoryForId, setAgentMemoryForId] = useState<string | null>(null);
  const [agentMemoryCount, setAgentMemoryCount] = useState(0);
  const [agentMemoryLocation, setAgentMemoryLocation] = useState<string | null>(null);
  const [agentMemoryDraft, setAgentMemoryDraft] = useState("");
  const [agentMemoryKind, setAgentMemoryKind] = useState<MemoryKind>("profile");
  const [agentWorkflows, setAgentWorkflows] = useState<WorkflowSummary[]>([]);
  const [agentWorkflowForId, setAgentWorkflowForId] = useState<string | null>(null);
  const [marketplaceSearch, setMarketplaceSearch] = useState("");
  const [busyMiniApp, setBusyMiniApp] = useState<string | null>(null);
  const [auth, setAuth] = useState<AuthState | null>(null);
  const authRef = useRef<AuthState | null>(null);
  const commitAuth = useCallback((next: AuthState) => {
    authRef.current = next;
    setAuth(next);
  }, []);
  const [authResolved, setAuthResolved] = useState(false);
  const [accountAvatarUrl, setAccountAvatarUrl] = useState<string | null>(null);
  const [loginBusy, setLoginBusy] = useState(false);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [browserLoginAttempt, setBrowserLoginAttempt] = useState<BrowserLoginAttempt | null>(null);
  const [browserLoginWakeNonce, setBrowserLoginWakeNonce] = useState(0);
  useEffect(() => {
    if (authResolved && auth) onAuthStateChange?.(auth);
  }, [auth, authResolved, onAuthStateChange]);
  const [onboardingStep, setOnboardingStep] = useState(() =>
    screenshotMode !== null || window.localStorage.getItem(ONBOARDING_COMPLETE_KEY) === "1"
      ? 3
      : 0,
  );
  useEffect(() => {
    let cancelled = false;
    void nativeOnboardingSeen().then((seen) => {
      if (!cancelled && seen === true) setOnboardingStep(3);
    });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    if (!networkOpen) return;
    let cancelled = false;
    const applyState = (state: SharingState) => {
      if (!cancelled) {
        setSharingState(state);
        setSharingError(null);
      }
    };
    void coordinator.getSharingState()
      .then(applyState)
      .catch((error) => { if (!cancelled) setSharingError(error instanceof Error ? error.message : String(error)); });
    const unsubscribe = coordinator.subscribeCollaboration((event) => {
      if (event.type === "state.changed") applyState(event.state);
      if (event.type === "typing.changed") {
        void coordinator.getSharingState().then(applyState).catch(() => undefined);
      }
    });
    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, [coordinator, networkOpen]);

  const [accountOpen, setAccountOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(screenshotMode === "settings");
  const [settingsSection, setSettingsSection] = useState<SettingsSection>("general");
  const [feedbackOpen, setFeedbackOpen] = useState(false);
  const [feedbackText, setFeedbackText] = useState("");
  const [feedbackState, setFeedbackState] = useState<"idle" | "sending" | "sent" | "error">("idle");
  const [aboutOpen, setAboutOpen] = useState(false);
  const [aboutEnvironment, setAboutEnvironment] = useState<NativeDesktopEnvironment | null>(null);
  const [widgetGalleryOpen, setWidgetGalleryOpen] = useState(false);
  const [nativeZoomFactor, setNativeZoomFactor] = useState(1);
  const [offlineAsrOpen, setOfflineAsrOpen] = useState(false);
  const [offlineAsrStatus, setOfflineAsrStatus] = useState<NativeOfflineAsrStatus | null>(null);
  const [offlineAsrModelUrl, setOfflineAsrModelUrl] = useState("");
  const [offlineAsrSha256, setOfflineAsrSha256] = useState("");
  const [offlineAsrProgress, setOfflineAsrProgress] = useState<NativeOfflineAsrProgress | null>(null);
  const [offlineAsrBusy, setOfflineAsrBusy] = useState(false);
  const [offlineAsrError, setOfflineAsrError] = useState<string | null>(null);
  const [cloudAgentInfo, setCloudAgentInfo] = useState<CloudAgentInfo | null>(null);
  const [cloudAgentFailure, setCloudAgentFailure] = useState<string | null>(null);
  const [cloudAgentActionPending, setCloudAgentActionPending] = useState(false);
  const [mcpServers, setMcpServers] = useState<unknown[]>([]);
  const [mcpApps, setMcpApps] = useState<unknown[]>([]);
  const [mcpToolServer, setMcpToolServer] = useState("");
  const [mcpToolName, setMcpToolName] = useState("");
  const [mcpToolArguments, setMcpToolArguments] = useState("{}");
  const [mcpToolResult, setMcpToolResult] = useState("");
  const [preferences, setPreferences] = useState<HostPreferences>(defaultPreferences);
  const [hostSettingsHydrated, setHostSettingsHydrated] = useState(false);
  const lastHostSettingsJsonRef = useRef("");
  const hostRouterSettingsRef = useRef<Pick<ProductHostSettings, "inferenceProvider" | "sandboxRuntime">>({
    inferenceProvider: "fabushi",
    sandboxRuntime: "host",
  });
  const [auditRecords, setAuditRecords] = useState<unknown[]>([]);
  const [auditAgentId, setAuditAgentId] = useState<string | null>(null);
  const [ruleDraft, setRuleDraft] = useState("");
  const [ruleBehavior, setRuleBehavior] = useState<"allow" | "ask">("allow");
  const [activeAgentId, setActiveAgentId] = useState("mahayana-assistant");

  useEffect(() => {
    if (!networkOpen || networkView !== "workspace") return;
    void refreshWorkspaceState();
  }, [networkOpen, networkView, activeAgentId]);

  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [capabilities, setCapabilities] = useState<CapabilitySummary[]>([]);
  const [automations, setAutomations] = useState<AutomationSummary[]>([]);
  const [connectors, setConnectors] = useState<ConnectorSummary[]>([]);
  const [skills, setSkills] = useState<SkillSummary[]>([]);
  const [skillTeams, setSkillTeams] = useState<SkillTeamSummary[]>([]);
  const [bots, setBots] = useState<BotSummary[]>([]);
  const [groups, setGroups] = useState<GroupSummary[]>([]);
  const [activeGroupId, setActiveGroupId] = useState<string | null>(null);
  const [groupEditorId, setGroupEditorId] = useState<"new" | string | null>(null);
  const [groupPreviews, setGroupPreviews] = useState<Record<string, { groupId: string; memberId: string; memberName: string; text: string }>>({});
  const [listenerIntegrations, setListenerIntegrations] = useState<ListenerIntegrationSummary[]>([]);
  const [connectingListeners, setConnectingListeners] = useState<Set<ListenerPlatform>>(
    () => new Set(),
  );
  const [updateState, setUpdateState] = useState<UpdateState>({ type: "loading" });
  const [transcriptCards, setTranscriptCards] = useState<TranscriptCardEntry[]>([]);
  const [automationName, setAutomationName] = useState("");
  const [automationPrompt, setAutomationPrompt] = useState("");
  const [automationSchedule, setAutomationSchedule] = useState("@daily");
  const [automationTriggerKind, setAutomationTriggerKind] = useState<"schedule" | "event">("schedule");
  const [automationEventSource, setAutomationEventSource] = useState<ListenerPlatform>("github");
  const [automationEventName, setAutomationEventName] = useState("pull_request.opened");
  const [automationEventFilter, setAutomationEventFilter] = useState("");
  const [editingAutomationId, setEditingAutomationId] = useState<string | null>(null);
  const [conversationSearch, setConversationSearch] = useState("");
  const [searchMessageMatches, setSearchMessageMatches] = useState<SearchMessageMatch[]>([]);
  const [searchMediaMatches, setSearchMediaMatches] = useState<SearchMediaMatch[]>([]);
  const [searchPending, setSearchPending] = useState(false);
  const searchQueryRef = useRef("");
  const botsRef = useRef<BotSummary[]>([]);
  const preferencesRef = useRef(preferences);
  const activeAgentIdRef = useRef(activeAgentId);
  const activeGroupIdRef = useRef(activeGroupId);
  const agentWorkflowForIdRef = useRef<string | null>(null);
  const notificationDeciderRef = useRef(new AgentNotificationPolicy());
  const notificationSnapshotsRef = useRef(new Map<string, AgentNotificationSnapshot>());
  const notificationOperationAgentsRef = useRef(new Map<string, string>());
  const agentRequestPendingRef = useRef(false);
  const streamedOperationIdRef = useRef<string | null>(null);
  const notificationQueueRef = useRef<Promise<void>>(Promise.resolve());
  const computerRefreshInFlightRef = useRef(false);
  const remoteDesktopControllerRef = useRef<RemoteComputerDesktopController | null>(null);

  const claimStreamedOperation = (operationId?: string): boolean => {
    if (!operationId) return false;
    if (streamedOperationIdRef.current === operationId) return true;
    if (!agentRequestPendingRef.current) return false;
    agentRequestPendingRef.current = false;
    streamedOperationIdRef.current = operationId;
    return true;
  };

  const clearStreamedOperation = (operationId: string, terminalStatus: "completed" | "failed" = "completed") => {
    if (streamedOperationIdRef.current !== operationId) return;
    streamedOperationIdRef.current = null;
    agentRequestPendingRef.current = false;
    setMessages((current) => current
      .filter((entry) => !(entry.kind === "thinking" && entry.operationId === operationId))
      .map((entry) => entry.kind === "action" &&
        entry.operationId === operationId &&
        entry.status === "running"
        ? { ...entry, status: terminalStatus }
        : entry));
  };

  const appendThinkingEntry = (operationId: string, title: string) => {
    setMessages((current) => [
      ...current.filter((entry) => !(entry.kind === "thinking" && entry.operationId === operationId)),
      { kind: "thinking", role: "assistant", text: "", operationId, title: title || "正在思考", status: "running" },
    ]);
  };

  const upsertStreamAction = (entry: {
    id: string;
    operationId: string;
    title: string;
    detail?: string;
    status: "running" | "completed" | "failed";
  }) => {
    setMessages((current) => {
      const next: HostTranscriptEntry = {
        id: entry.id,
        kind: "action",
        role: "assistant",
        text: "",
        operationId: entry.operationId,
        title: entry.title,
        detail: entry.detail,
        status: entry.status,
      };
      const index = current.findIndex((candidate) => candidate.kind === "action" && candidate.id === entry.id);
      return index < 0
        ? [...current, next]
        : current.map((candidate, candidateIndex) => candidateIndex === index ? { ...candidate, ...next } : candidate);
    });
  };

  useEffect(() => {
    const unsubscribe = subscribeNativeDesktopEvents({
      "cloud-agent-open": (payload) => {
        const info = payload as CloudAgentInfo;
        if (!info?.id) return;
        setCloudAgentFailure(null);
        setCloudAgentInfo(info);
      },
      "open-offline-asr": () => {
        setOfflineAsrError(null);
        setOfflineAsrOpen(true);
      },
      "offline-asr-progress": (payload) => {
        setOfflineAsrProgress(payload as NativeOfflineAsrProgress);
      },
      "focus-agent": (payload) => {
        const agentId = typeof payload === "object" && payload !== null && "agentId" in payload
          ? String((payload as { agentId?: unknown }).agentId ?? "").trim()
          : "";
        if (!agentId) return;
        setActiveGroupId(null);
        setActiveAgentId(agentId);
      },
      "deep-link": (payload) => {
        const link = payload as NativeDeepLink;
        if (link?.route === "auth" && link.attemptId) {
          setLoginBusy(true);
          setLoginError(null);
          setBrowserLoginAttempt((current) => current?.attemptId === link.attemptId
            ? current
            : { attemptId: link.attemptId!, loginUrl: "", pollAfterMs: 150 });
          setBrowserLoginWakeNonce((value) => value + 1);
          return;
        }
        if (link?.route === "settings" && link.section) {
          setSettingsSection(link.section);
          setSettingsOpen(true);
        }
      },
      "open-feedback": () => {
        setFeedbackState("idle");
        setFeedbackOpen(true);
      },
      "open-about": () => setAboutOpen(true),
      "widget-gallery": () => setWidgetGalleryOpen(true),
      "force-onboarding": () => {
        window.localStorage.removeItem(ONBOARDING_COMPLETE_KEY);
        setOnboardingStep(0);
      },
      "skip-onboarding": () => {
        window.localStorage.setItem(ONBOARDING_COMPLETE_KEY, "1");
        rememberNativeOnboarding();
        setOnboardingStep(3);
      },
      "zoom-factor-changed": (payload) => {
        const factor = typeof payload === "object" && payload !== null && "factor" in payload
          ? Number((payload as { factor?: unknown }).factor)
          : Number.NaN;
        if (Number.isFinite(factor) && factor > 0) setNativeZoomFactor(factor);
      },
    });
    markNativeDeepLinksReady();
    return unsubscribe;
  }, []);

  useEffect(() => {
    if (!aboutOpen) return;
    let cancelled = false;
    void invokeNativeDesktop<NativeDesktopEnvironment>("getDesktopEnvironment")
      .then((environment) => { if (!cancelled) setAboutEnvironment(environment); })
      .catch(() => { if (!cancelled) setAboutEnvironment(null); });
    return () => { cancelled = true; };
  }, [aboutOpen]);

  useEffect(() => {
    if (!offlineAsrOpen) return;
    let cancelled = false;
    void invokeNativeDesktop<NativeOfflineAsrStatus>("getOfflineAsrStatus")
      .then((status) => {
        if (!cancelled) {
          setOfflineAsrStatus(status);
          setOfflineAsrModelUrl((current) => current || status.modelUrl || "");
          setOfflineAsrSha256((current) => current || status.expectedSha256 || "");
          setOfflineAsrError(null);
        }
      })
      .catch((error) => {
        if (!cancelled) setOfflineAsrError(error instanceof Error ? error.message : String(error));
      });
    return () => { cancelled = true; };
  }, [offlineAsrOpen]);

  useEffect(() => {
    const runId = cloudAgentInfo?.runId ?? cloudAgentInfo?.id;
    if (!runId || ["finished", "error", "expired"].includes(cloudAgentInfo?.status ?? "unknown")) return;
    let cancelled = false;
    let timer: number | null = null;
    const schedule = (delayMs: number) => {
      if (!cancelled) timer = window.setTimeout(() => void refresh(), delayMs);
    };
    const refresh = async () => {
      try {
        const info = await invokeNativeDesktop<CloudAgentInfo>("getCloudAgentInfo", { bcId: runId, includeFiles: false });
        if (cancelled) return;
        setCloudAgentFailure(null);
        setCloudAgentInfo(info);
        if (!["finished", "error", "expired"].includes(info.status)) schedule(5_000);
      } catch (error) {
        if (cancelled) return;
        setCloudAgentFailure(error instanceof Error ? error.message : String(error));
        schedule(60_000);
      }
    };
    schedule(5_000);
    return () => {
      cancelled = true;
      if (timer) window.clearTimeout(timer);
    };
  }, [cloudAgentInfo?.id, cloudAgentInfo?.runId, cloudAgentInfo?.status]);

  const [agentMode, setAgentMode] = useState<AgentMode>("agent");
  const [selectedModel, setSelectedModel] = useState("auto");
  const [attachments, setAttachments] = useState<AttachmentContext[]>([]);
  const [activity, setActivity] = useState<AgentActivity[]>([]);
  const [usage, setUsage] = useState<{
    totalTokens: number;
    contextWindow?: number;
    model: string;
  } | null>(null);

  const notificationSnapshot = (
    agentId: string,
    patch: Partial<AgentNotificationSnapshot> = {},
  ): AgentNotificationSnapshot => {
    const bot = botsRef.current.find((candidate) => candidate.id === agentId);
    const current = notificationSnapshotsRef.current.get(agentId);
    return {
      id: agentId,
      name: bot?.name ?? current?.name ?? agentId,
      isRunning: current?.isRunning ?? false,
      awaitingReason: current?.awaitingReason ?? null,
      lastMessageId: current?.lastMessageId ?? null,
      lastMessagePreview: current?.lastMessagePreview ?? null,
      notifyEnabled:
        preferencesRef.current.notifyOnUpdates
        && (bot?.notificationsEnabled ?? current?.notifyEnabled ?? true)
        && (bot?.notifyOnUpdates ?? true),
      isHiddenFromSidebar: bot?.hidden ?? current?.isHiddenFromSidebar ?? false,
      ...patch,
    };
  };

  const queueNotificationSnapshot = (
    agentId: string,
    patch: Partial<AgentNotificationSnapshot>,
    allowNotification: boolean,
  ) => {
    const snapshot = notificationSnapshot(agentId, patch);
    notificationSnapshotsRef.current.set(agentId, snapshot);
    notificationQueueRef.current = notificationQueueRef.current
      .catch(() => {})
      .then(async () => {
        const isWindowFocused = allowNotification
          ? await transport.windowFocused().catch(() => document.hasFocus())
          : true;
        const transitions = notificationDeciderRef.current.evaluate(snapshot, {
          isWindowFocused,
          nowMs: Date.now(),
        });
        if (!allowNotification) return;
        for (const transition of transitions) {
          const content = buildAgentNotification(transition);
          await transport.showNotification(content.title, content.body).catch(() => {});
        }
      });
  };

  useEffect(() => {
    try {
      const stored = window.localStorage.getItem("fabushi.host.preferences.v1");
      if (stored) {
        setPreferences({ ...defaultPreferences, ...JSON.parse(stored) });
      }
    } catch {
      // A damaged preference should never prevent the host from starting.
    }
  }, []);

  useEffect(() => {
    preferencesRef.current = preferences;
    window.localStorage.setItem(
      "fabushi.host.preferences.v1",
      JSON.stringify(preferences),
    );
    syncNativeTheme(preferences.theme);
    if (!hostSettingsHydrated) return;
    const settings: ProductHostSettings = {
      notifications: preferences.notifyOnUpdates,
      autoUpdateWhenIdle: preferences.autoUpdateWhenIdle,
      localExecution: preferences.localExecution,
      routeEgressLocally: preferences.routeEgressLocally,
      securityKeys: preferences.securityKeys,
      webauthnProxyEnabled: preferences.webauthnProxyEnabled,
      localToolPermission: preferences.localToolPermission,
      remoteControlEnabled: preferences.remoteControlEnabled,
      aiComputerControlEnabled: preferences.aiComputerControlEnabled,
      autoReviewRules: preferences.autoReviewRules,
      ...hostRouterSettingsRef.current,
    };
    const serialized = serializeProductHostSettings(settings);
    if (serialized === lastHostSettingsJsonRef.current) return;
    lastHostSettingsJsonRef.current = serialized;
    void transport.execute({
      type: "settings.update",
      requestId: `settings-update-${Date.now()}`,
      settings,
    }).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
  }, [preferences, hostSettingsHydrated, transport]);

  useEffect(() => {
    const existing = remoteDesktopControllerRef.current;
    if (
      !hostSettingsHydrated ||
      hostStatus !== "ready" ||
      !auth?.loggedIn ||
      !(auth.user?.id ?? auth.user?.username ?? auth.user?.email)
    ) {
      remoteDesktopControllerRef.current = null;
      if (existing) void existing.stop();
      setRemoteDesktopState(null);
      return;
    }

    let cancelled = false;
    const controller = new RemoteComputerDesktopController({
      transport,
      label: `${auth.user?.nickname || auth.user?.username || "Fabushi"} 的 Mac`,
      identityScope: String(auth.user?.id ?? auth.user?.username ?? auth.user?.email),
      controlEnabled: preferences.remoteControlEnabled,
      resolveAgentId: (requestedAgentId) => requestedAgentId === "mahayana-assistant"
        || botsRef.current.some((bot) => bot.id === requestedAgentId)
        ? requestedAgentId
        : null,
      onState: (state) => {
        if (!cancelled) setRemoteDesktopState(state);
      },
    });
    remoteDesktopControllerRef.current = controller;
    const settings: ProductHostSettings = {
      notifications: preferences.notifyOnUpdates,
      autoUpdateWhenIdle: preferences.autoUpdateWhenIdle,
      localExecution: preferences.localExecution,
      routeEgressLocally: preferences.routeEgressLocally,
      securityKeys: preferences.securityKeys,
      webauthnProxyEnabled: preferences.webauthnProxyEnabled,
      localToolPermission: preferences.localToolPermission,
      remoteControlEnabled: preferences.remoteControlEnabled,
      aiComputerControlEnabled: preferences.aiComputerControlEnabled,
      autoReviewRules: preferences.autoReviewRules,
      ...hostRouterSettingsRef.current,
    };
    void (async () => {
      await transport.execute({
        type: "settings.update",
        requestId: `remote-settings-${Date.now()}`,
        settings,
      });
      if (!cancelled) await controller.start();
    })().catch((cause: unknown) => {
      if (!cancelled) setError(cause instanceof Error ? cause.message : String(cause));
    });
    return () => {
      cancelled = true;
      if (remoteDesktopControllerRef.current === controller) {
        remoteDesktopControllerRef.current = null;
      }
      void controller.stop();
    };
  }, [
    auth?.loggedIn,
    auth?.user?.id,
    auth?.user?.nickname,
    auth?.user?.username,
    auth?.user?.email,
    hostSettingsHydrated,
    hostStatus,
    preferences.remoteControlEnabled,
    transport,
  ]);

  useEffect(() => {
    botsRef.current = bots;
  }, [bots]);

  useEffect(() => {
    activeAgentIdRef.current = activeAgentId;
  }, [activeAgentId]);

  useEffect(() => {
    activeGroupIdRef.current = activeGroupId;
  }, [activeGroupId]);

  useEffect(() => {
    agentWorkflowForIdRef.current = agentWorkflowForId;
  }, [agentWorkflowForId]);

  useEffect(() => {
    if (teachStatus.state !== "recording") return;
    setTeachClockMs(Date.now());
    const timer = window.setInterval(() => setTeachClockMs(Date.now()), 500);
    return () => window.clearInterval(timer);
  }, [teachStatus.state, teachStatus.startedAtMs]);

  useEffect(() => {
    if (!computerOpen) return;
    const agentId = bots.some((bot) => bot.id === activeAgentId)
      ? activeAgentId
      : "mahayana-assistant";
    setSubagentAgentId(agentId);
    const stamp = Date.now();
    void Promise.all([
      transport.execute({ type: "subagent.list", requestId: `subagent-list-${stamp}`, agentId }),
      transport.execute({ type: "asyncTask.list", requestId: `async-task-list-${stamp}`, agentId }),
      transport.execute({ type: "teach.status", requestId: `teach-status-${stamp}` }),
      transport.execute({ type: "computer.status", requestId: `computer-status-${stamp}` }),
    ]).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
  }, [computerOpen, activeAgentId, bots, transport]);

  useEffect(() => {
    if (!computerOpen) return;
    let stopped = false;
    const refreshStatus = async () => {
      if (stopped) return;
      try {
        await transport.execute({ type: "computer.status", requestId: `computer-permission-status-${Date.now()}` });
      } catch (cause) {
        if (!stopped) setError(cause instanceof Error ? cause.message : String(cause));
      }
    };
    void refreshStatus();
    const timer = window.setInterval(() => void refreshStatus(), 2_000);
    return () => {
      stopped = true;
      window.clearInterval(timer);
    };
  }, [computerOpen, transport]);

  useEffect(() => {
    if (!computerOpen || !computerStatus?.captureSupported || !computerStatus.screenRecordingGranted) return;
    let stopped = false;
    const refresh = async () => {
      if (stopped || computerRefreshInFlightRef.current) return;
      computerRefreshInFlightRef.current = true;
      setComputerRefreshing(true);
      try {
        await transport.execute({
          type: "computer.screenshot",
          requestId: `computer-snapshot-${Date.now()}`,
          origin: "local-ui",
        });
      } catch (cause) {
        if (!stopped) setError(cause instanceof Error ? cause.message : String(cause));
      } finally {
        computerRefreshInFlightRef.current = false;
        if (!stopped) setComputerRefreshing(false);
      }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), 900);
    return () => {
      stopped = true;
      window.clearInterval(timer);
    };
  }, [computerOpen, computerStatus?.captureSupported, computerStatus?.screenRecordingGranted, transport]);

  useEffect(() => {
    if (!settingsOpen || settingsSection !== "mcp") return;
    const stamp = Date.now();
    void Promise.all([
      transport.execute({ type: "mcp.list", requestId: `mcp-list-${stamp}` }),
      transport.execute({ type: "mcp.apps", requestId: `mcp-apps-${stamp}` }),
    ]).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
  }, [settingsOpen, settingsSection, transport]);

  useEffect(() => {
    const query = conversationSearch.trim().toLowerCase();
    searchQueryRef.current = query;
    if (!query) {
      setSearchMessageMatches([]);
      setSearchMediaMatches([]);
      setSearchPending(false);
      return;
    }
    setSearchPending(true);
    const timer = window.setTimeout(() => {
      void Promise.all([
        coordinator.searchMessages(query, 50),
        coordinator.searchMedia(query, 50),
      ]).then(([messageMatches, mediaMatches]) => {
        if (searchQueryRef.current !== query) return;
        setSearchMessageMatches(messageMatches);
        setSearchMediaMatches(mediaMatches);
        setSearchPending(false);
      }).catch((cause: unknown) => {
        setSearchPending(false);
        setError(cause instanceof Error ? cause.message : String(cause));
      });
    }, 180);
    return () => window.clearTimeout(timer);
  }, [conversationSearch, coordinator]);

  useEffect(() => {
    const pass = (featureId: MahayanaHostFeatureId) => {
      setFeatureStates((current) => ({
        ...current,
        [featureId]: "passed",
      }));
    };

    const unsubscribe = coordinator.subscribe((event) => {
      switch (event.type) {
        case "host.ready":
          setHostStatus("ready");
          pass("runtime.boot");
          break;
        case "chat.message":
          if (event.role === "assistant" && claimStreamedOperation(event.operationId)) {
            setMessages((current) => current.filter((entry) => !(entry.kind === "thinking" && entry.operationId === event.operationId)));
          }
          setMessages((current) => {
            if (event.role === "assistant" && event.operationId) {
              const index = current.findIndex(
                (message) => message.kind === "message" && message.operationId === event.operationId && message.streaming,
              );
              if (index >= 0) {
                return current.map((message, messageIndex) =>
                  messageIndex === index
                    ? { ...message, kind: "message", text: event.text, streaming: false }
                    : message,
                );
              }
            }
            if (event.role === "user" && current.some((message) =>
              message.role === "user" && message.text === event.text &&
              (!event.operationId || message.operationId === event.operationId))) return current;
            return [
              ...current,
              {
                id: nextRequestId("message"),
                kind: "message",
                role: event.role,
                text: event.text,
                operationId: event.operationId,
                streaming: false,
              },
            ];
          });
          if (event.role === "assistant") {
            pass("chat.send");
            if (event.operationId) {
              const agentId = notificationOperationAgentsRef.current.get(event.operationId) ?? activeAgentIdRef.current;
              if (botsRef.current.some((bot) => bot.id === agentId)) {
                queueNotificationSnapshot(agentId, {
                  lastMessageId: event.operationId,
                  lastMessagePreview: event.text,
                }, false);
              }
            }
          }
          break;
        case "chat.delta":
          claimStreamedOperation(event.operationId);
          setMessages((current) => current.filter((entry) => !(entry.kind === "thinking" && entry.operationId === event.operationId)));
          setMessages((current) => {
            const index = current.findIndex(
              (message) => message.kind === "message" && message.operationId === event.operationId && message.streaming,
            );
            if (index < 0) {
              return [
                ...current,
                {
                  id: `${event.operationId}:stream`,
                  kind: "message",
                  role: "assistant",
                  text: event.delta,
                  operationId: event.operationId,
                  streaming: true,
                },
              ];
            }
            return current.map((message, messageIndex) =>
              messageIndex === index
                ? { ...message, kind: "message", text: `${message.text}${event.delta}`, streaming: true }
                : message,
            );
          });
          break;
        case "transcript.card":
          setTranscriptCards((current) => {
            const entry = {
              entryId: event.entryId,
              operationId: event.operationId,
              card: event.card,
            };
            const index = current.findIndex((item) => item.entryId === event.entryId);
            return index < 0
              ? [...current, entry]
              : current.map((item, itemIndex) => itemIndex === index ? entry : item);
          });
          break;
        case "draft.changed":
          setTranscriptCards((current) => current.map((entry) => {
            if (
              (entry.card.kind === "emailDraft" || entry.card.kind === "slackDraft") &&
              entry.card.draft.id === event.draftId
            ) {
              return {
                ...entry,
                card: {
                  ...entry.card,
                  draft: {
                    ...entry.card.draft,
                    status: event.status,
                    error: event.error,
                  },
                },
              } as TranscriptCardEntry;
            }
            return entry;
          }));
          break;
        case "secret.provided":
          setTranscriptCards((current) => current.map((entry) =>
            entry.card.kind === "secretRequest" &&
            entry.card.requestId === event.secretRequestId
              ? { ...entry, card: { ...entry.card, provided: true } }
              : entry,
          ));
          break;
        case "conversation.listed":
          setConversations(event.conversations);
          break;
        case "capability.listed":
          setCapabilities(event.capabilities);
          break;
        case "automation.listed":
          setAutomations(event.automations);
          break;
        case "automation.changed":
          setAutomations((current) => {
            if (event.action === "deleted") {
              return current.filter((item) => item.id !== event.automation.id);
            }
            const index = current.findIndex((item) => item.id === event.automation.id);
            return index < 0
              ? [...current, event.automation]
              : current.map((item, itemIndex) => itemIndex === index ? event.automation : item);
          });
          break;
        case "connector.listed":
          setConnectors(event.connectors);
          break;
        case "connector.changed":
          setConnectors((current) => {
            const index = current.findIndex((item) => item.id === event.connector.id);
            return index < 0
              ? [...current, event.connector]
              : current.map((item, itemIndex) => itemIndex === index ? event.connector : item);
          });
          break;
        case "connector.oauthRequested":
          void transport.openExternal(event.authorizationUrl).catch((cause: unknown) => {
            setError(cause instanceof Error ? cause.message : String(cause));
          });
          break;
        case "skill.listed":
          setSkills(event.skills);
          setSkillTeams(event.teams);
          break;
        case "skill.changed":
          setSkills((current) => {
            if (event.action === "deleted") {
              return current.filter((item) => item.id !== event.skill.id);
            }
            const index = current.findIndex((item) => item.id === event.skill.id);
            return index < 0
              ? [...current, event.skill]
              : current.map((item, itemIndex) => itemIndex === index ? event.skill : item);
          });
          break;
        case "bot.listed": {
          botsRef.current = event.bots;
          setBots(event.bots);
          const baselines = event.bots.map((bot) => ({
            id: bot.id,
            name: bot.name,
            isRunning: false,
            awaitingReason: null,
            lastMessageId: null,
            lastMessagePreview: null,
            notifyEnabled: preferencesRef.current.notifyOnUpdates && bot.notificationsEnabled,
            isHiddenFromSidebar: bot.hidden,
          } satisfies AgentNotificationSnapshot));
          for (const baseline of baselines) notificationSnapshotsRef.current.set(baseline.id, baseline);
          notificationDeciderRef.current.seed(baselines);
          break;
        }
        case "bot.changed":
          setBots((current) => {
            const next = event.action === "deleted"
              ? current.filter((item) => item.id !== event.bot.id)
              : current.some((item) => item.id === event.bot.id)
                ? current.map((item) => item.id === event.bot.id ? event.bot : item)
                : [...current, event.bot];
            botsRef.current = next;
            return next;
          });
          if (event.action === "deleted") {
            notificationSnapshotsRef.current.delete(event.bot.id);
            notificationDeciderRef.current.forgetAgent(event.bot.id);
          }
          break;
        case "group.listed":
          setGroups(event.groups);
          break;
        case "group.changed":
          setGroups((current) => {
            if (event.action === "deleted") {
              return current.filter((item) => item.id !== event.group.id);
            }
            const index = current.findIndex((item) => item.id === event.group.id);
            return index < 0
              ? [...current, event.group]
              : current.map((item, itemIndex) => itemIndex === index ? event.group : item);
          });
          if (event.action === "deleted") {
            setActiveGroupId((current) => current === event.group.id ? null : current);
          }
          if (event.action === "message" || event.action.startsWith("turnFailed:")) {
            setGroupPreviews((current) => Object.fromEntries(
              Object.entries(current).filter(([, preview]) => preview.groupId !== event.group.id),
            ));
          }
          break;
        case "group.delta":
          setGroupPreviews((current) => ({
            ...current,
            [event.operationId]: {
              groupId: event.groupId,
              memberId: event.memberId,
              memberName: event.memberName,
              text: `${current[event.operationId]?.text ?? ""}${event.delta}`,
            },
          }));
          break;
        case "agent.peerMessage":
          setPeerMessages((current) => [...current.filter((message) => message.id !== event.message.id), event.message].slice(-300));
          break;
        case "agent.peerHistory":
          setPeerMessages(event.messages.slice(-300));
          break;
        case "agent.broadcasted":
          setLastBroadcastResult(event.result);
          break;
        case "agent.backgroundStarted":
          setBackgroundWorkingAgents((current) => new Set(current).add(event.agentId));
          queueNotificationSnapshot(event.agentId, {
            isRunning: true,
            awaitingReason: null,
          }, false);
          break;
        case "agent.backgroundDelta":
          setBackgroundWorkingAgents((current) => new Set(current).add(event.agentId));
          setBackgroundPreviews((current) => ({
            ...current,
            [event.operationId]: {
              agentId: event.agentId,
              text: `${current[event.operationId]?.text ?? ""}${event.delta}`,
            },
          }));
          if (activeAgentIdRef.current === event.agentId && !activeGroupIdRef.current) {
            setMessages((current) => {
              const index = current.findIndex((message) => message.operationId === event.operationId);
              if (index < 0) {
                return [...current, { kind: "message", role: "assistant", text: event.delta, operationId: event.operationId }];
              }
              return current.map((message, messageIndex) => messageIndex === index
                ? { ...message, text: `${message.text}${event.delta}` }
                : message);
            });
          }
          break;
        case "agent.backgroundMessage":
          setBackgroundWorkingAgents((current) => new Set(current).add(event.agentId));
          setBackgroundPreviews((current) => ({ ...current, [event.operationId]: { agentId: event.agentId, text: event.text } }));
          queueNotificationSnapshot(event.agentId, {
            isRunning: true,
            lastMessageId: event.operationId,
            lastMessagePreview: event.text,
          }, false);
          if (activeAgentIdRef.current === event.agentId && !activeGroupIdRef.current) {
            setMessages((current) => {
              const index = current.findIndex((message) => message.operationId === event.operationId);
              return index < 0
                ? [...current, { kind: "message", role: "assistant", text: event.text, operationId: event.operationId }]
                : current.map((message, messageIndex) => messageIndex === index ? { ...message, text: event.text } : message);
            });
          }
          break;
        case "agent.backgroundFinished":
          setBackgroundWorkingAgents((current) => {
            const next = new Set(current);
            next.delete(event.agentId);
            return next;
          });
          setBackgroundPreviews((current) => {
            const next = { ...current };
            delete next[event.operationId];
            return next;
          });
          queueNotificationSnapshot(event.agentId, {
            isRunning: false,
          }, !event.error);
          if (event.error) setError(`${event.agentName}: ${event.error}`);
          break;
        case "subagent.listed":
          setSubagentAgentId(event.agentId);
          setSubagents(event.subagents);
          break;
        case "subagent.changed":
          if (event.subagent.parentAgentId === subagentAgentId || event.subagent.parentAgentId === activeAgentIdRef.current) {
            setSubagentAgentId(event.subagent.parentAgentId);
            setSubagents((current) => {
              const index = current.findIndex((subagent) => subagent.id === event.subagent.id);
              return index < 0
                ? [...current, event.subagent]
                : current.map((subagent, subagentIndex) => subagentIndex === index ? event.subagent : subagent);
            });
          }
          break;
        case "asyncTask.listed":
        case "asyncTask.changed":
          if (event.agentId === subagentAgentId || event.agentId === activeAgentIdRef.current) {
            setSubagentAgentId(event.agentId);
            setAsyncTasks(event.tasks);
          }
          break;
        case "teach.changed":
          setTeachStatus(event.status);
          if (event.result) {
            setTeachResult(event.result);
            if (event.result.saved) {
              void transport.execute({ type: "workflow.list", requestId: `teach-workflow-refresh-${Date.now()}`, agentId: event.result.agentId }).catch(() => {});
            }
          }
          break;
        case "computer.status":
          setComputerStatus(event.status);
          break;
        case "computer.snapshot":
          setComputerSnapshot(event.snapshot);
          break;
        case "computer.result":
          setComputerSnapshot(event.result.snapshot);
          break;
        case "memory.listed":
          setAgentMemoryForId(event.agentId);
          setAgentMemories(event.memories);
          setAgentMemoryCount(event.count);
          setAgentMemoryLocation(event.location ?? null);
          break;
        case "memory.changed":
          void transport.execute({
            type: "memory.list",
            requestId: `memory-refresh-${Date.now()}`,
            agentId: event.agentId,
            limit: 1000,
          }).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
          break;
        case "workflow.listed":
          setAgentWorkflowForId(event.agentId);
          setAgentWorkflows(event.workflows);
          break;
        case "workflow.changed":
          if (event.agentId === agentWorkflowForIdRef.current) {
            setAgentWorkflows((current) => {
              if (event.action === "deleted" && event.id) {
                return current.filter((workflow) => workflow.id !== event.id);
              }
              if (event.workflow) {
                const index = current.findIndex((workflow) => workflow.id === event.workflow!.id && workflow.source === event.workflow!.source);
                return index < 0
                  ? [...current, event.workflow]
                  : current.map((workflow, workflowIndex) => workflowIndex === index ? event.workflow! : workflow);
              }
              return current;
            });
          }
          if (event.workflow?.source === "automation" || event.action === "deleted") {
            void transport.execute({
              type: "automation.list",
              requestId: `automation-refresh-${Date.now()}`,
            }).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
          }
          break;
        case "search.messages":
          if (event.query === searchQueryRef.current) {
            setSearchMessageMatches(event.matches);
            setSearchPending(false);
          }
          break;
        case "search.media":
          if (event.query === searchQueryRef.current) {
            setSearchMediaMatches(event.matches);
          }
          break;
        case "mcp.listed":
          setMcpServers(event.servers);
          if (event.servers.length) setMcpToolServer((current) => current || mcpServerName(event.servers[0]));
          break;
        case "mcp.apps":
          setMcpApps(event.apps);
          break;
        case "mcp.oauth":
          if (event.authorizationUrl) {
            void transport.openExternal(event.authorizationUrl).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
          }
          void transport.execute({ type: "mcp.list", requestId: `mcp-list-${Date.now()}` }).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
          break;
        case "mcp.refreshed":
          void Promise.all([
            transport.execute({ type: "mcp.list", requestId: `mcp-list-${Date.now()}` }),
            transport.execute({ type: "mcp.apps", requestId: `mcp-apps-${Date.now()}` }),
          ]).catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
          break;
        case "mcp.toolResult":
          setMcpToolResult(JSON.stringify(event.result, null, 2));
          break;
        case "settings.changed":
          lastHostSettingsJsonRef.current = serializeProductHostSettings(event.settings);
          hostRouterSettingsRef.current = {
            inferenceProvider: event.settings.inferenceProvider,
            sandboxRuntime: event.settings.sandboxRuntime,
          };
          setHostSettingsHydrated(true);
          setPreferences((current) => ({
            ...current,
            notifyOnUpdates: event.settings.notifications,
            autoUpdateWhenIdle: event.settings.autoUpdateWhenIdle,
            localExecution: event.settings.localExecution,
            routeEgressLocally: event.settings.routeEgressLocally,
            securityKeys: event.settings.securityKeys,
            webauthnProxyEnabled: event.settings.webauthnProxyEnabled,
            localToolPermission: event.settings.localToolPermission,
            remoteControlEnabled: event.settings.remoteControlEnabled,
            aiComputerControlEnabled: event.settings.aiComputerControlEnabled,
            autoReviewRules: event.settings.autoReviewRules,
          }));
          break;
        case "audit.listed":
          setAuditAgentId(event.agentId);
          setAuditRecords(event.records);
          break;
        case "tray.listed":
          setTrays(event.trays);
          break;
        case "tray.changed":
          setTrays((current) => {
            if (event.action === "cleared") return [];
            if (event.action === "dismissed" && event.id) return current.filter((tray) => tray.id !== event.id);
            if (event.action === "pushed" && event.tray) {
              const index = current.findIndex((tray) => tray.id === event.tray!.id);
              return index < 0
                ? [...current, event.tray]
                : current.map((tray, trayIndex) => trayIndex === index ? event.tray! : tray);
            }
            return current;
          });
          break;
        case "listener.listed":
          setListenerIntegrations(event.integrations);
          break;
        case "listener.changed":
          setListenerIntegrations((current) => {
            const index = current.findIndex((item) => item.platform === event.integration.platform);
            return index < 0
              ? [...current, event.integration]
              : current.map((item, itemIndex) => itemIndex === index ? event.integration : item);
          });
          setConnectingListeners((current) => {
            if (!current.has(event.integration.platform)) return current;
            const next = new Set(current);
            next.delete(event.integration.platform);
            return next;
          });
          setTranscriptCards((current) => current.map((entry) =>
            entry.card.kind === "listenerConnect" &&
            entry.card.platform === event.integration.platform
              ? {
                  ...entry,
                  card: {
                    ...entry.card,
                    connected: event.integration.isConnected,
                    pending: false,
                  },
                }
              : entry,
          ));
          break;
        case "update.changed":
          setUpdateState(event.state);
          break;
        case "conversation.opened":
          setActiveConversationId(event.conversationId);
          setActiveAgentId(
            botsRef.current.find((bot) => bot.conversationId === event.conversationId)?.id ?? "mahayana-assistant",
          );
          setTranscriptCards([]);
          setMessages(
            event.messages.map((message) => ({
              kind: "message" as const,
              role: message.role,
              text: message.text,
            })),
          );
          break;
        case "agent.step":
          if (event.operationId && claimStreamedOperation(event.operationId)) {
            upsertStreamAction({
              id: `${event.operationId}:${event.stepId}`,
              operationId: event.operationId,
              title: event.title,
              detail: event.detail,
              status: event.status,
            });
          }
          setActivity((current) => {
            const next: AgentActivity = {
              id: event.stepId,
              operationId: event.operationId,
              kind: event.kind,
              title: event.title,
              detail: event.detail,
              status: event.status,
            };
            const index = current.findIndex(
              (item) =>
                item.id === event.stepId &&
                item.operationId === event.operationId,
            );
            return index < 0
              ? [...current.slice(-11), next]
              : current.map((item, itemIndex) =>
                  itemIndex === index ? next : item,
                );
          });
          break;
        case "model.routed":
          if (claimStreamedOperation(event.operationId)) {
            upsertStreamAction({
              id: `${event.operationId}:model`,
              operationId: event.operationId,
              title: event.model === "auto" ? "选择模型" : `模型：${event.model}`,
              detail: `${event.provider} · ${event.mode}`,
              status: "completed",
            });
          }
          setUsage((current) => ({
            totalTokens: current?.totalTokens ?? 0,
            contextWindow: current?.contextWindow,
            model: event.model,
          }));
          setActivity((current) => [
            ...current.slice(-11),
            {
              id: `${event.operationId}:model`,
              operationId: event.operationId,
              kind: "model",
              title: event.model === "auto" ? "自动选择模型" : event.model,
              detail: `${event.provider} · ${event.mode}`,
              status: "completed",
            },
          ]);
          break;
        case "usage.updated":
          setUsage((current) => ({
            totalTokens: event.totalTokens,
            contextWindow: event.contextWindow,
            model: current?.model ?? "auto",
          }));
          break;
        case "marketplace.installed":
          {
            const record = installMarketplaceApp(event.miniAppId, "github-metadata");
            if (record) setInstalledMiniAppRecords(readMarketplaceInstallRecords());
          }
          setInstalledMiniApps((current) => {
            const next = new Set(current);
            next.add(event.miniAppId);
            return next;
          });
          setBusyMiniApp(null);
          void transport.execute({
            type: "capability.list",
            requestId: `capability-refresh-${Date.now()}`,
          });
          pass("marketplace.install");
          break;
        case "miniapp.opened":
          setOpenedMiniApp(event.miniAppId);
          setOpenedMiniAppHtml(event.html ?? null);
          setActiveAgentId(event.miniAppId);
          setMarketplaceOpen(false);
          setComputerOpen(true);
          setBusyMiniApp(null);
          pass("miniapp.open");
          break;
        case "approval.requested":
          setApproval(event);
          setApprovalState("pending");
          if (botsRef.current.some((bot) => bot.id === activeAgentIdRef.current)) {
            queueNotificationSnapshot(activeAgentIdRef.current, {
              isRunning: false,
              awaitingReason: event.detail || event.reason || "Waiting for your input.",
            }, true);
          }
          break;
        case "approval.resolved":
          setApproval(null);
          for (const [agentId, snapshot] of notificationSnapshotsRef.current) {
            if (snapshot.awaitingReason) queueNotificationSnapshot(agentId, { awaitingReason: null }, false);
          }
          setApprovalState(
            event.decision === "allow-once"
              ? "allowed-once"
              : event.decision === "allow-session"
                ? "allowed-for-session"
                : "denied",
          );
          pass("capability.approval");
          break;
        case "operation.started": {
          if (claimStreamedOperation(event.operationId)) {
            setChatDispatching(true);
            appendThinkingEntry(event.operationId, event.label || "正在思考");
          }
          setOperationState("running");
          if (event.interruptible) {
            setActiveOperationId(event.operationId);
          }
          const agentId = activeAgentIdRef.current;
          if (botsRef.current.some((bot) => bot.id === agentId)) {
            notificationOperationAgentsRef.current.set(event.operationId, agentId);
            queueNotificationSnapshot(agentId, { isRunning: true, awaitingReason: null }, false);
          }
          break;
        }
        case "operation.interrupted": {
          clearStreamedOperation(event.operationId, "failed");
          setChatDispatching(false);
          setActiveOperationId(null);
          setOperationState("interrupted");
          const agentId = notificationOperationAgentsRef.current.get(event.operationId);
          if (agentId) queueNotificationSnapshot(agentId, { isRunning: false }, false);
          notificationOperationAgentsRef.current.delete(event.operationId);
          pass("operation.interrupt");
          break;
        }
        case "operation.completed": {
          clearStreamedOperation(event.operationId);
          setChatDispatching(false);
          setActiveOperationId((current) =>
            current === event.operationId ? null : current,
          );
          setOperationState((current) =>
            current === "running" ? "completed" : current,
          );
          const agentId = notificationOperationAgentsRef.current.get(event.operationId);
          if (agentId) queueNotificationSnapshot(agentId, { isRunning: false }, true);
          notificationOperationAgentsRef.current.delete(event.operationId);
          break;
        }
        case "operation.failed": {
          clearStreamedOperation(event.operationId, "failed");
          setChatDispatching(false);
          setActiveOperationId((current) =>
            current === event.operationId ? null : current,
          );
          setOperationState("failed");
          const agentId = notificationOperationAgentsRef.current.get(event.operationId);
          if (agentId) queueNotificationSnapshot(agentId, { isRunning: false }, false);
          notificationOperationAgentsRef.current.delete(event.operationId);
          setError(`${event.code}: ${event.message}`);
          break;
        }
        case "session.cleared":
          setSessionState("cleared");
          commitAuth({ loggedIn: false });
          pass("session.clear");
          break;
        case "host.closed":
          setHostStatus("closed");
          break;
      }
    });

    const configuredMode = configuredHostMode;
    const mode =
      configuredMode === "production" || configuredMode === "test"
        ? configuredMode
        : "production";

    let disposed = false;
    void transport
      .initialize({ profileId: "default", mode })
      .then(async () => {
        // Resolve authentication first. Conversation/tool/workspace hydration is background
        // work and must never hold a full-screen returning-user gate.
        let restoredLoggedIn = false;
        try {
          const authState = await Promise.race([
            transport.authStatus(),
            new Promise<never>((_, reject) =>
              window.setTimeout(
                () => reject(new Error("账号服务响应超时，请重新登录")),
                8_000,
              ),
            ),
          ]);
          if (disposed) return;
          restoredLoggedIn = authState.loggedIn;
          // Browser login can complete while the initial authStatus request is
          // still in flight. Never let that older signed-out snapshot overwrite
          // the newer authenticated session.
          if (!authRef.current?.loggedIn) commitAuth(authState);
          if (authState.loggedIn || authRef.current?.loggedIn) {
            setFeatureStates((current) => ({ ...current, "auth.login": "passed" }));
          }
        } catch (cause: unknown) {
          if (disposed) return;
          if (!authRef.current?.loggedIn) {
            commitAuth({ loggedIn: false });
            setLoginError(
              `无法恢复账号会话：${cause instanceof Error ? cause.message : String(cause)}`,
            );
          }
        } finally {
          if (!disposed) setAuthResolved(true);
        }
        // Only a session restored by this initialization owns HostClient hydration.
        // If browser login won the race, the parent immediately switches to
        // MessengerWorkspace; letting this stale signed-out surface hydrate after
        // unmount would monopolize the single Rust Host queue and delay Messenger.
        if (disposed || !restoredLoggedIn) return;
        await transport.execute({
          type: "conversation.list",
          requestId: "conversation-list-initial",
        });
        await transport.execute({
          type: "capability.list",
          requestId: "capability-list-initial",
        });
        await coordinator.listAutomations();
        await Promise.all([
          transport.execute({
            type: "connector.list",
            requestId: "connector-list-initial",
          }),
          coordinator.listSkills(),
          coordinator.listAgents(),
          coordinator.listGroups(),
          coordinator.listTrays(),
          transport.execute({
            type: "settings.get",
            requestId: "settings-get-initial",
          }),
          coordinator.listListenerIntegrations(),
          transport.execute({
            type: "update.status",
            requestId: "update-status-initial",
          }),
        ]);
        const stored = readMarketplaceInstallRecords();
        setInstalledMiniAppRecords(stored);
        setInstalledMiniApps(new Set([
          ...Object.keys(stored),
          ...(screenshotHasMiniApp ? [defaultMiniAppId] : []),
        ]));
        // Rehydrate the native Host from the same immutable release metadata
        // recorded by the Marketplace. Browser-only WASM keeps the metadata
        // record but cannot execute a package without a native Host.
        for (const record of Object.values(stored)) {
          if (!record.version || record.version === "0.0.0") continue;
          try {
            const release = await transport.marketplaceRelease(record.id, record.version);
            const releaseManifest = release.releaseManifest;
            if (Array.isArray(releaseManifest?.artifacts) && releaseManifest.artifacts.length > 0) {
              await transport.pluginInstall(releaseManifest, "desktop");
            }
          } catch {
            // A removed or unavailable release should not prevent the Host
            // from starting; the Marketplace state remains visible for repair.
          }
        }
      })
      .catch((cause: unknown) => {
        if (!authRef.current?.loggedIn) commitAuth({ loggedIn: false });
        setAuthResolved(true);
        setHostStatus("failed");
        setError(cause instanceof Error ? cause.message : String(cause));
      });

    return () => {
      disposed = true;
      unsubscribe();
      void transport.close();
    };
  }, [transport, coordinator]);

  const nextRequestId = (prefix: string) => {
    requestSequence.current += 1;
    return `${prefix}-${requestSequence.current}`;
  };

  const run = async (action: () => Promise<unknown>) => {
    setError(null);
    try {
      await action();
    } catch (cause: unknown) {
      setBusyMiniApp(null);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const execute = (command: RuntimeCommand) => coordinator.execute(command);

  const sendMessage = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const text = input.trim();
    if (!text) return;
    setInput("");
    setChatDispatching(true);
    agentRequestPendingRef.current = true;
    const outgoingAttachments = attachments;
    setAttachments([]);
    setError(null);
    try {
      const accepted = await execute({
        type: "chat.send",
        requestId: nextRequestId("chat"),
        text,
        agentId: activeAgentId,
        conversationId: activeConversationId ?? undefined,
        mode: agentMode,
        modeStatement: buildModeTransitionNote(
          agentMode === "ask" ? "chat" : agentMode,
        ),
        model: selectedModel === "auto" ? undefined : selectedModel,
        attachments: outgoingAttachments,
      });
      if (accepted.operationId && claimStreamedOperation(accepted.operationId)) {
        appendThinkingEntry(accepted.operationId, "正在思考");
      }
    } catch (cause: unknown) {
      agentRequestPendingRef.current = false;
      streamedOperationIdRef.current = null;
      setChatDispatching(false);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
    if (!streamedOperationIdRef.current) agentRequestPendingRef.current = false;
  };

  const attachFiles = async (files: FileList | null) => {
    if (!files?.length) return;
    const ownerId = bots.some((bot) => bot.id === activeAgentId)
      ? activeAgentId
      : "mahayana-assistant";
    const next: AttachmentContext[] = [];
    for (const file of [...files].slice(0, 6)) {
      const byteLimit = attachmentLooksLikeVideo(file.name)
        ? VIDEO_ATTACHMENT_BYTE_LIMIT
        : ATTACHMENT_BYTE_LIMIT;
      if (!file.size) {
        setError(`“${file.name}” 是空文件，未添加。`);
        continue;
      }
      if (file.size > byteLimit) {
        setError(`“${file.name}” 超过附件上限（${Math.round(byteLimit / 1024 / 1024)} MB）。`);
        continue;
      }
      try {
        let cancelStoredWait = () => {};
        const stored = new Promise<AttachmentContext>((resolve, reject) => {
          let unsubscribe = () => {};
          const timeout = window.setTimeout(() => {
            unsubscribe();
            reject(new Error(`等待 ${file.name} 写入 Host 超时`));
          }, 20_000);
          cancelStoredWait = () => {
            window.clearTimeout(timeout);
            unsubscribe();
          };
          unsubscribe = coordinator.subscribe((event) => {
            if (
              event.type !== "attachment.stored" ||
              event.attachment.agentId !== ownerId ||
              event.attachment.name !== file.name
            ) return;
            cancelStoredWait();
            resolve({
              id: event.attachment.id,
              name: event.attachment.name,
              mimeType: event.attachment.mimeType,
              path: event.attachment.path,
              sizeBytes: event.attachment.sizeBytes,
            });
          });
        });
        try {
          await execute({
            type: "attachment.upload",
            requestId: nextRequestId("attachment-upload"),
            agentId: ownerId,
            filename: file.name,
            mimeType: file.type || undefined,
            bytesBase64: await fileToBase64(file),
          });
        } catch (cause) {
          cancelStoredWait();
          void stored.catch(() => {});
          throw cause;
        }
        const attachment = await stored;
        if (attachmentLooksTextPreviewable(file)) {
          attachment.text = await file.slice(0, ATTACHMENT_TEXT_PREVIEW_BYTES).text();
        }
        next.push(attachment);
      } catch (cause: unknown) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
    }
    if (next.length) {
      setAttachments((current) => [...current, ...next].slice(0, 6));
    }
    if (attachmentInput.current) attachmentInput.current.value = "";
  };

  const beginBrowserLogin = async () => {
    if (loginBusy) return;
    setLoginBusy(true);
    setLoginError(null);
    try {
      const attempt = await transport.browserLoginStart();
      setBrowserLoginAttempt(attempt);
      await transport.openExternal(attempt.loginUrl);
    } catch (cause: unknown) {
      setBrowserLoginAttempt(null);
      setLoginBusy(false);
      setLoginError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const reopenBrowserLogin = async () => {
    const attempt = browserLoginAttempt;
    if (!attempt) return;
    try {
      const reopened = await transport.browserLoginReopen(attempt.attemptId);
      if (reopened.status !== "pending") {
        setBrowserLoginWakeNonce((value) => value + 1);
        return;
      }
      const loginUrl = reopened.loginUrl?.trim();
      if (!loginUrl) throw new Error("账号服务没有返回新的登录地址");
      setBrowserLoginAttempt((current) => current?.attemptId === attempt.attemptId
        ? {
            ...current,
            loginUrl,
            pollAfterMs: reopened.pollAfterMs ?? current.pollAfterMs,
          }
        : current);
      await transport.openExternal(loginUrl);
      setLoginError(null);
      setBrowserLoginWakeNonce((value) => value + 1);
    } catch (cause: unknown) {
      setLoginError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const cancelBrowserLogin = async () => {
    const attempt = browserLoginAttempt;
    if (!attempt) return;
    setLoginError(null);
    try {
      const result = await transport.browserLoginCancel(attempt.attemptId);
      if (result.status === "completed") {
        setBrowserLoginWakeNonce((value) => value + 1);
        return;
      }
      setBrowserLoginAttempt(null);
      setLoginBusy(false);
      setLoginError(result.status === "failed" ? "登录流程已失败，请重新开始" : null);
    } catch (cause: unknown) {
      setLoginError(`取消登录失败：${cause instanceof Error ? cause.message : String(cause)}`);
    }
  };

  useEffect(() => {
    if (!auth?.loggedIn) {
      setAccountAvatarUrl(null);
      return;
    }
    let cancelled = false;
    void invokeNativeDesktop<{ avatar?: string | null; displayName?: string | null }>("getAccountAvatar")
      .then((profile) => {
        if (!cancelled) setAccountAvatarUrl(profile?.avatar?.trim() || null);
      })
      .catch(() => {
        if (!cancelled) setAccountAvatarUrl(auth.user?.avatar?.trim() || null);
      });
    return () => { cancelled = true; };
  }, [auth?.loggedIn, auth?.user?.avatar]);

  useEffect(() => {
    const attempt = browserLoginAttempt;
    if (!attempt) return undefined;
    let cancelled = false;
    let timer: number | null = null;
    const pollDelay = Math.max(150, Math.min(2_000, attempt.pollAfterMs ?? 750));
    const schedule = (delay = pollDelay) => {
      if (!cancelled) timer = window.setTimeout(() => void poll(), delay);
    };
    const poll = async () => {
      if (cancelled) return;
      if (attempt.expiresAt && Date.now() / 1000 >= attempt.expiresAt) {
        setBrowserLoginAttempt(null);
        setLoginBusy(false);
        setLoginError("登录链接已过期，请重新开始");
        return;
      }
      try {
        const result = await transport.browserLoginPoll(attempt.attemptId);
        if (cancelled) return;
        if (result.status === "completed" && result.auth) {
          const completedAuth = { ...result.auth, loggedIn: true };
          // Resolve the parent shell immediately; do not wait for the initial
          // auth restoration request or leave the authenticated Host surface
          // mounted in place of Messenger.
          setAuthResolved(true);
          commitAuth(completedAuth);
          setFeatureStates((current) => ({ ...current, "auth.login": "passed" }));
          setBrowserLoginAttempt(null);
          setLoginBusy(false);
          setLoginError(null);
          return;
        }
        if (result.status === "expired" || result.status === "cancelled" || result.status === "failed") {
          setBrowserLoginAttempt(null);
          setLoginBusy(false);
          setLoginError(
            result.status === "cancelled"
              ? "登录已取消"
              : result.status === "failed"
                ? "登录流程未完成，请重新开始"
                : "登录链接已过期，请重新开始",
          );
          return;
        }
        setLoginError(null);
        schedule();
      } catch (cause: unknown) {
        if (cancelled) return;
        setLoginError(`正在重新连接账号服务：${cause instanceof Error ? cause.message : String(cause)}`);
        schedule(1_500);
      }
    };
    void poll();
    return () => {
      cancelled = true;
      if (timer) window.clearTimeout(timer);
    };
  }, [browserLoginAttempt?.attemptId, browserLoginWakeNonce]);

  const logout = async () => {
    await run(async () => {
      const state = await transport.logout();
      commitAuth({ ...state, loggedIn: false });
      setAccountOpen(false);
      setBrowserLoginAttempt(null);
      setLoginBusy(false);
    });
  };

  const hasManagedModal = Boolean(
    confirmAction ||
    approval ||
    browserLoginAttempt ||
    accountOpen ||
    feedbackOpen ||
    aboutOpen ||
    widgetGalleryOpen ||
    offlineAsrOpen ||
    cloudAgentInfo ||
    settingsOpen ||
    automationOpen ||
    networkOpen ||
    marketplaceOpen ||
    agentSettingsOpen
  );

  useEffect(() => {
    if (!hasManagedModal) {
      if (overlayWasOpenRef.current) {
        overlayWasOpenRef.current = false;
        const returnTarget = overlayReturnFocusRef.current;
        overlayReturnFocusRef.current = null;
        window.requestAnimationFrame(() => {
          if (returnTarget?.isConnected) {
            returnTarget.focus({ preventScroll: true });
            return;
          }
          const browserLoginButton = document.querySelector<HTMLElement>('[data-testid="browser-login-start"]');
          browserLoginButton?.focus({ preventScroll: true });
        });
      }
      return undefined;
    }

    if (!overlayWasOpenRef.current) {
      const activeElement = document.activeElement;
      overlayReturnFocusRef.current = activeElement instanceof HTMLElement ? activeElement : null;
      overlayWasOpenRef.current = true;
    }

    const visibleDialogs = () => Array.from(
      document.querySelectorAll<HTMLElement>('[role="dialog"][aria-modal="true"]'),
    ).filter((dialog) => dialog.getClientRects().length > 0);
    const topDialog = () => {
      const dialogs = visibleDialogs();
      return dialogs.length ? dialogs[dialogs.length - 1] : null;
    };
    const focusableWithin = (dialog: HTMLElement) => Array.from(
      dialog.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    ).filter((element) => element.getClientRects().length > 0 && element.getAttribute('aria-hidden') !== 'true');

    const focusFrame = window.requestAnimationFrame(() => {
      const dialog = topDialog();
      if (!dialog || dialog.contains(document.activeElement)) return;
      const focusable = focusableWithin(dialog);
      (focusable[0] ?? dialog).focus({ preventScroll: true });
    });

    const handleKeyDown = (event: KeyboardEvent) => {
      const dialog = topDialog();
      if (!dialog) return;
      if (event.key === 'Tab') {
        const focusable = focusableWithin(dialog);
        if (!focusable.length) {
          event.preventDefault();
          dialog.focus({ preventScroll: true });
          return;
        }
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        const active = document.activeElement;
        if (event.shiftKey && (active === first || !dialog.contains(active))) {
          event.preventDefault();
          last.focus({ preventScroll: true });
        } else if (!event.shiftKey && active === last) {
          event.preventDefault();
          first.focus({ preventScroll: true });
        }
        return;
      }
      if (event.key !== 'Escape') return;
      event.preventDefault();
      event.stopPropagation();
      if (confirmAction) {
        if (!confirmBusy) setConfirmAction(null);
        return;
      }
      // Approval is an explicit security decision; Escape must never silently
      // deny or dismiss it.
      if (approval) return;
      if (accountOpen) return void setAccountOpen(false);
      if (widgetGalleryOpen) return void setWidgetGalleryOpen(false);
      if (aboutOpen) return void setAboutOpen(false);
      if (feedbackOpen) return void setFeedbackOpen(false);
      if (cloudAgentInfo) return void setCloudAgentInfo(null);
      if (offlineAsrOpen) return void setOfflineAsrOpen(false);
      if (settingsOpen) return void setSettingsOpen(false);
      if (automationOpen) return void setAutomationOpen(false);
      if (networkOpen) return void setNetworkOpen(false);
      if (marketplaceOpen) return void setMarketplaceOpen(false);
      if (agentSettingsOpen) return void setAgentSettingsOpen(false);
      if (browserLoginAttempt) void cancelBrowserLogin();
    };

    document.addEventListener('keydown', handleKeyDown, true);
    return () => {
      window.cancelAnimationFrame(focusFrame);
      document.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [
    hasManagedModal,
    confirmAction,
    confirmBusy,
    approval,
    browserLoginAttempt?.attemptId,
    accountOpen,
    feedbackOpen,
    aboutOpen,
    widgetGalleryOpen,
    offlineAsrOpen,
    cloudAgentInfo,
    settingsOpen,
    automationOpen,
    networkOpen,
    marketplaceOpen,
    agentSettingsOpen,
  ]);

  const installMiniApp = (miniAppId: string) => {
    const app = marketplaceApps.find((candidate) => candidate.id === miniAppId);
    if (!app) return;
    const installed = installedMiniAppRecords[miniAppId];
    const action = installed
      ? marketplaceAppInstallAction(miniAppId, installed)
      : installedMiniApps.has(miniAppId)
        ? "current"
        : marketplaceAppInstallAction(miniAppId);
    if (action === "blocked") {
      setError(`${app.title} 的本地版本高于市场版本，已阻止静默降级。`);
      return;
    }
    if (action === "unavailable") {
      setError(`${app.title} 尚未发布可验证的 GitHub 版本。`);
      return;
    }
    if (action === "current") return;
    setBusyMiniApp(miniAppId);
    void run(async () => {
      const release = await transport.marketplaceRelease(miniAppId, app.version);
      const releaseManifest = release.releaseManifest;
      const installContract = release.install
        ?? (releaseManifest?.install as Record<string, unknown> | undefined);
      const installSource = installContract?.source as Record<string, unknown> | undefined;
      if (releaseManifest?.protocol !== "mahayana.external-release.v1"
        || installContract?.protocol !== "fabushi.marketplace.install.v1"
        || installContract.strategy !== "github-immutable"
        || installSource?.marketplaceHostsPackage === true
        || typeof installSource?.repository !== "string"
        || typeof installSource?.sourceRef !== "string"
        || !installSource.sourceRef.trim()) {
        throw new Error("市场版本没有提供可验证的 GitHub 安装合同。");
      }
      await transport.pluginInstall(releaseManifest, "desktop");
      const nativeBridge = nativeDesktopBridge();
      if (nativeBridge) {
        try {
          await nativeBridge.invoke("addMiniAppToAccount", { pluginId: miniAppId });
        } catch (cause) {
          await transport.pluginUninstall(miniAppId).catch(() => undefined);
          throw cause;
        }
      }
      const record = installMarketplaceApp(miniAppId, "github-sha256");
      if (!record) throw new Error("市场没有可记录的 GitHub 版本。");
      setInstalledMiniAppRecords(readMarketplaceInstallRecords());
      setInstalledMiniApps((current) => new Set(current).add(miniAppId));
      setFeatureStates((current) => ({
        ...current,
        "marketplace.install": "passed",
      }));
    }).finally(() => setBusyMiniApp(null));
  };

  const openMiniApp = (miniAppId: string) => {
    setBusyMiniApp(miniAppId);
    void run(() =>
      execute({
        type: "miniapp.open",
        requestId: nextRequestId("open"),
        miniAppId,
      }),
    );
  };

  const resolveDraft = (draft: MessageDraft, action: "send" | "discard") => {
    void run(() =>
      execute({
        type: "draft.resolve",
        requestId: nextRequestId("draft"),
        draft,
        action,
      }),
    );
  };

  const provideSecret = (secretRequestId: string, value: string) => {
    void run(() =>
      execute({
        type: "secret.provide",
        requestId: nextRequestId("secret"),
        secretRequestId,
        value,
      }),
    );
  };

  const connectListener = (platform: ListenerPlatform) => {
    setConnectingListeners((current) => new Set(current).add(platform));
    setTranscriptCards((current) => current.map((entry) =>
      entry.card.kind === "listenerConnect" && entry.card.platform === platform
        ? { ...entry, card: { ...entry.card, pending: true } }
        : entry,
    ));
    void run(async () => {
      try {
        await execute({
          type: "listener.connect",
          requestId: nextRequestId("listener-connect"),
          platform,
        });
      } finally {
        setConnectingListeners((current) => {
          if (!current.has(platform)) return current;
          const next = new Set(current);
          next.delete(platform);
          return next;
        });
        setTranscriptCards((current) => current.map((entry) =>
          entry.card.kind === "listenerConnect" && entry.card.platform === platform
            ? { ...entry, card: { ...entry.card, pending: false } }
            : entry,
        ));
      }
    });
  };

  const visibleMarketplaceApps = marketplaceApps.filter((app) =>
    `${app.title} ${app.id} ${app.description}`
      .toLowerCase()
      .includes(marketplaceSearch.toLowerCase()),
  );
  const visibleConversations = conversations.filter((conversation) =>
    `${conversation.title} ${conversation.id}`
      .toLowerCase()
      .includes(conversationSearch.toLowerCase()),
  );
  const activeMiniApp = marketplaceApps.find((app) => app.id === openedMiniApp);
  const activeAgent =
    activeAgentId === "mahayana-assistant"
      ? undefined
      : marketplaceApps.find((app) => app.id === activeAgentId);
  const displayName =
    auth?.user?.nickname || auth?.user?.username || "Fabushi 用户";
  const accountAvatar = accountAvatarUrl || auth?.user?.avatar?.trim() || null;
  const accountProvider = auth?.provider?.trim() || "browser";
  const attachmentTokens = attachments.reduce(
    (total, attachment) => total + estimateTextTokens(attachment.text ?? ""),
    0,
  );
  const mentionMatch = input.match(/(?:^|\s)@([^\s@]*)$/);
  const mentionQuery = mentionMatch?.[1]?.toLowerCase() ?? null;
  const mentionSuggestions = mentionQuery === null
    ? []
    : capabilities
        .filter((capability) => capability.availability !== "unavailable")
        .filter((capability) =>
          `${capability.id} ${capability.title} ${capability.description}`
            .toLowerCase()
            .includes(mentionQuery),
        )
        .slice(0, 6);
  const effectiveTheme =
    preferences.theme === "system" ? "dark" : preferences.theme;
  const updateCopy = updateStateCopy(updateState);
  const selectedListenerIntegration = listenerIntegrations.find(
    (integration) => integration.platform === automationEventSource,
  );
  const selectedListenerNeededByCount = automations.filter(
    (automation) =>
      automation.enabled &&
      automation.trigger?.kind === "event" &&
      automation.trigger.source === automationEventSource,
  ).length;
  const selectedListenerState = connectingListeners.has(automationEventSource)
    ? "connecting"
    : selectedListenerIntegration?.error
      ? "error"
      : selectedListenerIntegration?.isConnected
        ? "listening"
        : "idle";
  const selectedListenerStateCopy = {
    idle: "尚未连接",
    connecting: "正在连接",
    listening: "正在监听",
    error: "连接异常",
  }[selectedListenerState];
  const visibleSidebarBots = bots.filter(
    (bot) => !bot.hidden && bot.id !== "mahayana-assistant",
  );
  const currentRunningActivity = [...activity]
    .reverse()
    .find((item) => item.status === "running");
  const activeBotState: BotMarkState = chatDispatching
    ? "thinking"
    : activeOperationId || operationState === "running"
      ? botMarkStateFromActivity(currentRunningActivity)
      : operationState === "failed" || operationState === "interrupted"
      ? "waking"
      : "idle";
  const activeBotMarkId = activeAgentId || "mahayana-assistant";
  const activeBotProfile = bots.find((bot) => bot.id === activeBotMarkId);
  const primaryBotProfile = bots.find((bot) => bot.id === "mahayana-assistant");
  const activeBotShape = activeBotProfile?.avatarShape as BotMarkShape | undefined;
  const activeBotColor = activeBotProfile?.avatarColor as BotMarkColor | undefined;
  const activeGroup = groups.find((group) => group.id === activeGroupId) ?? null;
  const editingGroup = groupEditorId && groupEditorId !== "new"
    ? groups.find((group) => group.id === groupEditorId)
    : undefined;
  const activeGroupPreviews = Object.fromEntries(
    Object.entries(groupPreviews)
      .filter(([, preview]) => preview.groupId === activeGroupId)
      .map(([operationId, preview]) => [operationId, {
        memberId: preview.memberId,
        memberName: preview.memberName,
        text: preview.text,
      }]),
  );
  const networkSender = activeBotProfile ?? primaryBotProfile;
  const networkTargets = networkSender
    ? [
        ...bots.filter((bot) => bot.id !== networkSender.id),
        ...groups
          .filter((group) => group.memberIds.includes(networkSender.id))
          .map((group) => ({
            id: group.id,
            name: group.name,
            description: group.description || `${group.memberIds.length} 个成员`,
            title: "Group",
            hidden: false,
            notificationsEnabled: false,
            notifyOnUpdates: false,
            unread: false,
          } satisfies BotSummary)),
      ]
    : [];

  const openAgentNetwork = () => {
    if (networkSender) {
      setNetworkTargetId((current) => current && current !== networkSender.id ? current : networkTargets[0]?.id ?? "");
      void run(() => execute({
        type: "agent.peerHistory",
        requestId: nextRequestId("peer-history"),
        agentId: networkSender.id,
        limit: 300,
      }));
    }
    setNetworkOpen(true);
  };

  const sendNetworkMessage = async () => {
    if (!networkSender || !networkTargetId || !networkMessage.trim()) return;
    const text = clampAgentMessage(networkMessage);
    if (!text) return;
    setNetworkMessage("");
    await run(() => execute({
      type: "agent.send",
      requestId: nextRequestId("agent-send"),
      fromAgentId: networkSender.id,
      targetId: networkTargetId,
      text,
      priority: networkPriority,
    }));
  };

  const requestConfirmation = (confirmation: ConfirmAction) => {
    setConfirmBusy(false);
    setConfirmAction(confirmation);
  };

  const executeConfirmedAction = async () => {
    const confirmation = confirmAction;
    if (!confirmation || confirmBusy) return;
    setConfirmBusy(true);
    try {
      await confirmation.action();
      setConfirmAction(null);
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setConfirmBusy(false);
    }
  };

  const refreshWorkspaceState = async () => {
    if (!activeAgentId || workspaceBusy) return;
    setWorkspaceBusy(true);
    try {
      const [box, secrets, audit, egressAvailable, agentNetworkEnabled] = await Promise.all([
        coordinator.getForeverBoxStatus(activeAgentId),
        coordinator.getBoxSecretsStatus(activeAgentId),
        coordinator.requestDiskSaverAudit(),
        coordinator.isEgressTunnelAvailable(),
        coordinator.isAgentNetworkEnabled(activeAgentId),
      ]);
      setWorkspaceStatus(box);
      setWorkspaceSecrets(secrets);
      setWorkspaceDiskAudit(audit);
      setWorkspaceEgressAvailable(egressAvailable);
      setWorkspaceAgentNetworkEnabled(agentNetworkEnabled);
      setWorkspaceError(null);
    } catch (error) {
      setWorkspaceError(error instanceof Error ? error.message : String(error));
    } finally {
      setWorkspaceBusy(false);
    }
  };

  const ensureActiveWorkspace = async () => {
    if (!activeAgentId || workspaceBusy) return;
    setWorkspaceBusy(true);
    try {
      const box = await coordinator.ensureForeverBox(activeAgentId);
      setWorkspaceStatus(box);
      setWorkspaceSecrets(await coordinator.getBoxSecretsStatus(activeAgentId));
      setWorkspaceError(null);
    } catch (error) {
      setWorkspaceError(error instanceof Error ? error.message : String(error));
    } finally {
      setWorkspaceBusy(false);
    }
  };

  const handBackActiveWorkspace = () => {
    if (!activeAgentId || workspaceBusy) return;
    requestConfirmation({
      title: "归档并释放 Workspace？",
      message: "当前 Workspace 会移动到可恢复归档区并退出活跃状态。已保存的 Box secrets 不会写入明文，也不会被这个动作删除。",
      confirmLabel: "归档并释放",
      tone: "warning",
      action: async () => {
        setWorkspaceBusy(true);
        try {
          const box = await coordinator.handBackForeverBox(activeAgentId);
          setWorkspaceStatus(box);
          setWorkspaceSecrets(await coordinator.getBoxSecretsStatus(activeAgentId));
          setWorkspaceError(null);
        } finally {
          setWorkspaceBusy(false);
        }
      },
    });
  };

  const refreshSharingState = async () => {
    if (sharingBusy) return;
    setSharingBusy(true);
    try {
      setSharingState(await coordinator.getSharingState());
      setSharingError(null);
    } catch (error) {
      setSharingError(error instanceof Error ? error.message : String(error));
    } finally {
      setSharingBusy(false);
    }
  };

  const createSharedRoomForActiveAgent = async () => {
    const name = sharedRoomName.trim();
    if (!name || !activeAgentId || sharingBusy) return;
    setSharingBusy(true);
    try {
      await coordinator.createSharedRoom(name, [activeAgentId], activeAgentId);
      setSharedRoomName("");
      setSharingState(await coordinator.getSharingState());
      setSharingError(null);
    } catch (error) {
      setSharingError(error instanceof Error ? error.message : String(error));
    } finally {
      setSharingBusy(false);
    }
  };

  const joinSharedRoomFromInvite = async () => {
    const token = sharedInviteToken.trim();
    if (!token || !activeAgentId || sharingBusy) return;
    setSharingBusy(true);
    try {
      await coordinator.joinSharedRoom(token, activeAgentId, activeBotProfile?.name ?? activeAgentId);
      setSharedInviteToken("");
      setSharingState(await coordinator.getSharingState());
      setSharingError(null);
    } catch (error) {
      setSharingError(error instanceof Error ? error.message : String(error));
    } finally {
      setSharingBusy(false);
    }
  };

  const createSharedInvite = async (roomId: string) => {
    if (sharingBusy) return;
    setSharingBusy(true);
    try {
      const invite = await coordinator.createRoomInvite(roomId);
      setLastSharedInvite(invite);
      setSharingError(null);
    } catch (error) {
      setSharingError(error instanceof Error ? error.message : String(error));
    } finally {
      setSharingBusy(false);
    }
  };

  const mutateSharedRoom = async (action: () => Promise<unknown>) => {
    if (sharingBusy) return;
    setSharingBusy(true);
    try {
      await action();
      setSharingState(await coordinator.getSharingState());
      setSharingError(null);
    } catch (error) {
      setSharingError(error instanceof Error ? error.message : String(error));
    } finally {
      setSharingBusy(false);
    }
  };

  const confirmRemoveAgentFromSharedRoom = (roomId: string, roomName: string) => {
    if (!activeAgentId || sharingBusy) return;
    const agentId = activeAgentId;
    requestConfirmation({
      title: `从「${roomName}」移除当前 Agent？`,
      message: "只会移除当前 Agent 在这个共享房间里的成员关系；房间、其他成员和本机 Agent 本身都会保留。",
      confirmLabel: "移除当前 Agent",
      tone: "warning",
      action: () => mutateSharedRoom(() => coordinator.removeOwnAgentFromSharedRoom(roomId, agentId)),
    });
  };

  const confirmLeaveSharedRoom = (roomId: string, roomName: string) => {
    if (!activeAgentId || sharingBusy) return;
    const agentId = activeAgentId;
    requestConfirmation({
      title: `离开共享房间「${roomName}」？`,
      message: "当前 Agent 的最后一个本方成员关系会被移除。之后需要新的邀请码或房主审批才能重新加入。",
      confirmLabel: "离开房间",
      tone: "warning",
      action: () => mutateSharedRoom(() => coordinator.leaveSharedRoom(roomId, agentId)),
    });
  };

  const pulseSharedRoomTyping = (roomId: string) => {
    if (!activeAgentId) return;
    void coordinator.setSharedRoomTyping(roomId, activeAgentId, true)
      .then(() => setTimeout(() => { void coordinator.setSharedRoomTyping(roomId, activeAgentId, false); }, 1_500))
      .catch((error) => setSharingError(error instanceof Error ? error.message : String(error)));
  };

  const broadcastNetworkMessage = async () => {
    if (!networkMessage.trim()) return;
    const message = clampAgentMessage(networkMessage);
    if (!message) return;
    setNetworkMessage("");
    setLastBroadcastResult(null);
    await run(() => execute({
      type: "agent.broadcast",
      requestId: nextRequestId("agent-broadcast"),
      message,
    }));
  };

  const saveGroup = async (draft: { name: string; description: string; memberIds: string[] }) => {
    if (groupEditorId && groupEditorId !== "new") {
      await run(() => execute({
        type: "group.update",
        requestId: nextRequestId("group-update"),
        id: groupEditorId,
        ...draft,
      }));
    } else {
      await run(() => execute({
        type: "group.create",
        requestId: nextRequestId("group-create"),
        ...draft,
      }));
    }
    setGroupEditorId(null);
  };

  const sendGroupMessage = async (text: string) => {
    if (!activeGroup) return;
    await run(() => execute({
      type: "group.send",
      requestId: nextRequestId("group-send"),
      id: activeGroup.id,
      text,
    }));
  };

  const deleteActiveGroup = () => {
    if (!activeGroup) return;
    const groupId = activeGroup.id;
    const groupName = activeGroup.name;
    requestConfirmation({
      title: `删除群聊「${groupName}」？`,
      message: "这个群聊及其本地上下文会从当前工作空间移除。这个操作不能从群聊列表直接撤销。",
      confirmLabel: "永久删除",
      tone: "danger",
      action: () => run(() => execute({
        type: "group.delete",
        requestId: nextRequestId("group-delete"),
        id: groupId,
      })),
    });
  };

  const openAgentSettings = () => {
    setAgentName(activeBotProfile?.name ?? (activeAgent ? `${activeAgent.title}机器人` : "大乘助手"));
    setAgentTitle(activeBotProfile?.title ?? "");
    setAgentDescription(activeBotProfile?.description ?? "");
    setAgentNotifications(activeBotProfile?.notificationsEnabled ?? true);
    setAgentSettingsOpen(true);
    if (activeBotProfile) {
      setAgentMemoryForId(activeBotProfile.id);
      void run(() => execute({
        type: "memory.list",
        requestId: nextRequestId("memory-list"),
        agentId: activeBotProfile.id,
        limit: 1000,
      }));
      setAgentWorkflowForId(activeBotProfile.id);
      void run(() => execute({
        type: "workflow.list",
        requestId: nextRequestId("workflow-list"),
        agentId: activeBotProfile.id,
      }));
      setAuditAgentId(activeBotProfile.id);
      void run(() => execute({
        type: "audit.list",
        requestId: nextRequestId("audit-list"),
        agentId: activeBotProfile.id,
        limit: 100,
      }));
    }
  };

  const updateActiveBotProfile = (patch: Partial<Pick<BotSummary, "name" | "title" | "description" | "notificationsEnabled">>) => {
    if (!activeBotProfile) return;
    void run(() => execute({
      type: "bot.update",
      requestId: nextRequestId("bot-profile"),
      id: activeBotProfile.id,
      ...patch,
    }));
  };

  const addAgentMemory = () => {
    if (!activeBotProfile || !agentMemoryDraft.trim()) return;
    const content = agentMemoryDraft;
    setAgentMemoryDraft("");
    void run(() => execute({
      type: "memory.add",
      requestId: nextRequestId("memory-add"),
      agentId: activeBotProfile.id,
      content,
      kind: agentMemoryKind,
    }));
  };

  const removeAgentMemory = (id: string) => {
    if (!activeBotProfile) return;
    void run(() => execute({
      type: "memory.remove",
      requestId: nextRequestId("memory-remove"),
      agentId: activeBotProfile.id,
      id,
    }));
  };

  const clearAgentMemory = () => {
    if (!activeBotProfile) return;
    const agentId = activeBotProfile.id;
    const name = activeBotProfile.name;
    requestConfirmation({
      title: `清空 ${name} 的全部记忆？`,
      message: "这会删除这个 Agent 的持久记忆条目。对话记录和工作流不会被这个动作删除。",
      confirmLabel: "清空记忆",
      tone: "danger",
      action: () => run(() => execute({
        type: "memory.clear",
        requestId: nextRequestId("memory-clear"),
        agentId,
      })),
    });
  };

  const refreshAgentWorkflows = () => {
    if (!activeBotProfile) return;
    setAgentWorkflowForId(activeBotProfile.id);
    void run(() => execute({
      type: "workflow.list",
      requestId: nextRequestId("workflow-list"),
      agentId: activeBotProfile.id,
    }));
  };

  const saveAgentWorkflow = async (draft: {
    id?: string;
    name: string;
    description: string;
    body: string;
    trigger?: WorkflowTrigger;
    sourceRef?: string;
  }) => {
    if (!activeBotProfile) return;
    await run(() => execute({
      type: "workflow.upsert",
      requestId: nextRequestId("workflow-upsert"),
      agentId: activeBotProfile.id,
      ...draft,
    }));
    refreshAgentWorkflows();
  };

  const setAgentWorkflowEnabled = async (id: string, enabled: boolean) => {
    if (!activeBotProfile) return;
    await run(() => execute({
      type: "workflow.setEnabled",
      requestId: nextRequestId("workflow-toggle"),
      agentId: activeBotProfile.id,
      id,
      enabled,
    }));
    refreshAgentWorkflows();
  };

  const runAgentWorkflow = async (id: string) => {
    if (!activeBotProfile) return;
    setChatDispatching(true);
    await run(() => execute({
      type: "workflow.run",
      requestId: nextRequestId("workflow-run"),
      agentId: activeBotProfile.id,
      id,
    }));
  };

  const deleteAgentWorkflow = (id: string, workflowName: string) => {
    if (!activeBotProfile) return;
    const agentId = activeBotProfile.id;
    requestConfirmation({
      title: `删除 Workflow「${workflowName}」？`,
      message: "这个可复用工作流会从当前 Agent 的工作流目录移除；关联的历史审计记录不会被改写。",
      confirmLabel: "删除 Workflow",
      tone: "danger",
      action: async () => {
        await run(() => execute({
          type: "workflow.delete",
          requestId: nextRequestId("workflow-delete"),
          agentId,
          id,
        }));
        refreshAgentWorkflows();
      },
    });
  };

  const importAgentWorkflowMarkdown = async (markdown: string, fallbackName?: string) => {
    if (!activeBotProfile) return;
    await run(() => execute({
      type: "workflow.importMarkdown",
      requestId: nextRequestId("workflow-import"),
      agentId: activeBotProfile.id,
      markdown,
      fallbackName,
    }));
    refreshAgentWorkflows();
  };

  const importAgentWorkflowLiveSource = async (source: string, fallbackName?: string) => {
    if (!activeBotProfile) return;
    await run(() => execute({
      type: "workflow.importLiveSource",
      requestId: nextRequestId("workflow-live"),
      agentId: activeBotProfile.id,
      source,
      fallbackName,
    }));
    refreshAgentWorkflows();
  };

  const callMcpTool = async () => {
    if (!mcpToolServer.trim() || !mcpToolName.trim()) return;
    let argumentsValue: unknown = {};
    try {
      argumentsValue = mcpToolArguments.trim() ? JSON.parse(mcpToolArguments) : {};
    } catch {
      setError("MCP 工具参数必须是有效 JSON。");
      return;
    }
    setMcpToolResult("");
    await run(() => execute({
      type: "mcp.toolCall",
      requestId: nextRequestId("mcp-tool"),
      server: mcpToolServer.trim(),
      tool: mcpToolName.trim(),
      arguments: argumentsValue,
    }));
  };

  const setPreference = <Key extends keyof HostPreferences>(
    key: Key,
    value: HostPreferences[Key],
  ) => setPreferences((current) => ({ ...current, [key]: value }));

  const clickComputerSnapshot = (event: React.MouseEvent<HTMLImageElement>) => {
    if (!computerSnapshot || !computerStatus?.inputSupported || !computerStatus.accessibilityGranted) return;
    const bounds = event.currentTarget.getBoundingClientRect();
    if (bounds.width <= 0 || bounds.height <= 0) return;
    const sourceWidth = computerSnapshot.width ?? Math.round(bounds.width);
    const sourceHeight = computerSnapshot.height ?? Math.round(bounds.height);
    const x = Math.max(0, Math.min(sourceWidth - 1, Math.round((event.clientX - bounds.left) / bounds.width * sourceWidth)));
    const y = Math.max(0, Math.min(sourceHeight - 1, Math.round((event.clientY - bounds.top) / bounds.height * sourceHeight)));
    void run(() => execute({
      type: "computer.action",
      requestId: nextRequestId("computer-click"),
      origin: "local-ui",
      agentId: activeBotMarkId,
      action: { action: "click", x, y, button: "left", count: 1, description: "User clicked the local computer preview" },
    }));
  };

  const addAutoReviewRule = () => {
    const text = ruleDraft.trim();
    if (!text) return;
    setPreferences((current) => ({
      ...current,
      autoReviewRules: [
        ...current.autoReviewRules,
        { id: `rule-${Date.now()}`, behavior: ruleBehavior, text },
      ],
    }));
    setRuleDraft("");
  };

  const resetAutomationDraft = () => {
    setEditingAutomationId(null);
    setAutomationName("");
    setAutomationPrompt("");
    setAutomationSchedule("@daily");
    setAutomationTriggerKind("schedule");
    setAutomationEventSource("github");
    setAutomationEventName("pull_request.opened");
    setAutomationEventFilter("");
  };

  const editAutomation = (automation: AutomationSummary) => {
    setEditingAutomationId(automation.id);
    setAutomationName(automation.name);
    setAutomationPrompt(automation.prompt);
    if (automation.trigger?.kind === "event") {
      setAutomationTriggerKind("event");
      setAutomationEventSource(automation.trigger.source);
      setAutomationEventName(automation.trigger.event);
      setAutomationEventFilter(automation.trigger.filter ?? "");
      setAutomationSchedule("@daily");
    } else {
      setAutomationTriggerKind("schedule");
      setAutomationSchedule(
        automation.trigger?.kind === "schedule"
          ? automation.trigger.schedule
          : automation.schedule,
      );
    }
  };

  const saveAutomation = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const trigger = automationTriggerKind === "event"
      ? {
          kind: "event" as const,
          source: automationEventSource,
          event: automationEventName.trim(),
          filter: automationEventFilter.trim() || undefined,
        }
      : {
          kind: "schedule" as const,
          schedule: normalizeAutomationSchedule(automationSchedule),
        };
    await run(() => execute({
      type: "automation.upsert",
      requestId: nextRequestId("automation"),
      id: editingAutomationId ?? undefined,
      name: automationName.trim(),
      prompt: automationPrompt.trim(),
      schedule:
        trigger.kind === "schedule"
          ? trigger.schedule
          : `event:${trigger.source}:${trigger.event}`,
      trigger,
      enabled: true,
    }));
    resetAutomationDraft();
  };

  async function configureOfflineAsr() {
    if (offlineAsrBusy) return;
    setOfflineAsrBusy(true);
    setOfflineAsrError(null);
    try {
      const status = await invokeNativeDesktop<NativeOfflineAsrStatus>("configureOfflineAsrModel", {
        modelUrl: offlineAsrModelUrl.trim(),
        sha256: offlineAsrSha256.trim().toLowerCase(),
      });
      setOfflineAsrStatus(status);
    } catch (error) {
      setOfflineAsrError(error instanceof Error ? error.message : String(error));
    } finally {
      setOfflineAsrBusy(false);
    }
  }

  async function downloadOfflineAsr() {
    const modelUrl = offlineAsrModelUrl.trim();
    const sha256 = offlineAsrSha256.trim().toLowerCase();
    if (!modelUrl || !sha256 || offlineAsrBusy) return;
    setOfflineAsrBusy(true);
    setOfflineAsrError(null);
    setOfflineAsrProgress({ phase: "model-download", downloadedBytes: 0, totalBytes: null });
    try {
      await invokeNativeDesktop<NativeOfflineAsrStatus>("configureOfflineAsrModel", { modelUrl, sha256 });
      const status = await invokeNativeDesktop<NativeOfflineAsrStatus>("downloadOfflineAsrModel", { modelUrl, sha256 });
      setOfflineAsrStatus(status);
      setOfflineAsrProgress({ phase: "ready", progress: 1, status });
    } catch (error) {
      setOfflineAsrError(error instanceof Error ? error.message : String(error));
    } finally {
      setOfflineAsrBusy(false);
    }
  }

  async function openCloudRun(runId: string) {
    const id = runId.trim();
    if (!id || cloudAgentActionPending) return;
    setCloudAgentActionPending(true);
    setCloudAgentFailure(null);
    try {
      const result = await invokeNativeDesktop<{ info?: CloudAgentInfo }>("openCloudAgent", { bcId: id });
      if (result?.info) setCloudAgentInfo(result.info);
    } catch (error) {
      setCloudAgentFailure(error instanceof Error ? error.message : String(error));
    } finally {
      setCloudAgentActionPending(false);
    }
  }

  async function refreshCloudRun() {
    const runId = cloudAgentInfo?.runId ?? cloudAgentInfo?.id;
    if (!runId || cloudAgentActionPending) return;
    setCloudAgentActionPending(true);
    try {
      const info = await invokeNativeDesktop<CloudAgentInfo>("getCloudAgentInfo", { bcId: runId, includeFiles: false });
      setCloudAgentFailure(null);
      setCloudAgentInfo(info);
    } catch (error) {
      setCloudAgentFailure(error instanceof Error ? error.message : String(error));
    } finally {
      setCloudAgentActionPending(false);
    }
  }

  const confirmBotDelete = (bot: BotSummary) => {
    requestConfirmation({
      title: `永久删除 Bot「${bot.name}」？`,
      message: "这个 Bot 的独立身份、配置和会话关联会从 Fabushi 工作空间移除。主助手不会受这个操作影响。",
      confirmLabel: "永久删除 Bot",
      tone: "danger",
      action: () => run(() => execute({
        type: "bot.delete",
        requestId: nextRequestId("bot-delete"),
        id: bot.id,
      })),
    });
  };

  const confirmAutomationDelete = (id: string, name: string) => {
    requestConfirmation({
      title: `删除例程「${name}」？`,
      message: "这个自动化例程会停止触发，并从当前 Agent 的例程列表移除。",
      confirmLabel: "删除例程",
      tone: "danger",
      action: () => run(() => execute({
        type: "automation.delete",
        requestId: nextRequestId("automation-delete"),
        id,
      })),
    });
  };

  function cancelCloudRun() {
    const runId = cloudAgentInfo?.runId ?? cloudAgentInfo?.id;
    if (!runId || cloudAgentActionPending) return;
    requestConfirmation({
      title: "取消这个 Cloud Run？",
      message: "正在执行的云端任务会收到取消请求。已经产生的工具调用或外部副作用不会被自动回滚。",
      confirmLabel: "取消运行",
      tone: "warning",
      action: async () => {
        setCloudAgentActionPending(true);
        try {
          const result = await invokeNativeDesktop<{ info?: CloudAgentInfo }>("cancelCloudAgent", { bcId: runId });
          if (result?.info) setCloudAgentInfo(result.info);
          setCloudAgentFailure(null);
        } finally {
          setCloudAgentActionPending(false);
        }
      },
    });
  }

  async function submitDesktopFeedback(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const message = feedbackText.trim();
    if (!message || feedbackState === "sending") return;
    setFeedbackState("sending");
    try {
      await invokeNativeDesktop("submitFeedback", {
        category: "product",
        message,
        context: {
          activeAgentId,
          activeGroupId,
          settingsSection,
          zoomFactor: nativeZoomFactor,
        },
      });
      setFeedbackText("");
      setFeedbackState("sent");
    } catch {
      setFeedbackState("error");
    }
  }

  return (
    <main className={styles.shell} data-theme={effectiveTheme} data-testid="mahayana-host">
      <aside className={styles.sidebar}>
        <div className={styles.titlebar}>
          <div className={styles.trafficLights} aria-hidden="true">
            <span />
            <span />
            <span />
          </div>
          <div className={styles.titleBrand}>
            <BotMark botId="fabushi-brand" state={operationState === "running" ? "working" : "idle"} size={21} color="black" className={styles.titleBrandMark} />
            <strong>Fabushi</strong>
          </div>
          <div className={styles.titleActions}>
            <button className={styles.iconButton} type="button" aria-label="通知与错误" data-has-trays={trays.length > 0} onClick={() => setTrayOpen((open) => !open)}>
              <Icon name="bell" />
              {trays.length > 0 ? <span className={styles.trayBadge}>{Math.min(trays.length, 99)}</span> : null}
            </button>
            <button className={styles.iconButton} type="button" aria-label="智能体网络" onClick={openAgentNetwork}>
              <Icon name="network" />
            </button>
            <button
              className={styles.iconButton}
              type="button"
              aria-label="新建会话"
              onClick={() => {
                setMessages([]);
                setTranscriptCards([]);
                setActivity([]);
                setUsage(null);
                setActiveConversationId(null);
                setActiveGroupId(null);
                setActiveAgentId("mahayana-assistant");
              }}
            >
              <Icon name="plus" />
            </button>
          </div>
        </div>

        {trayOpen ? (
          <section className={styles.trayPopover} aria-label="通知与错误">
            <header>
              <div><strong>通知</strong><small>{trays.length ? `${trays.length} 个需要注意` : "没有需要处理的项目"}</small></div>
              {trays.length ? <button type="button" onClick={() => void run(() => execute({ type: "tray.clear", requestId: nextRequestId("tray-clear") }))}>全部清除</button> : null}
            </header>
            <div className={styles.trayList}>
              {[...trays].reverse().map((tray) => {
                const bot = bots.find((item) => item.id === tray.agentId);
                return (
                  <article key={tray.id}>
                    <BotMark
                      botId={tray.agentId}
                      state="waking"
                      size={30}
                      shape={bot?.avatarShape as BotMarkShape | undefined}
                      color={bot?.avatarColor as BotMarkColor | undefined}
                      label={bot?.name || tray.agentId}
                    />
                    <div>
                      <strong>{tray.title}{tray.count && tray.count > 1 ? ` ×${tray.count}` : ""}</strong>
                      {tray.detail ? <p>{tray.detail}</p> : null}
                      <small>{bot?.name || tray.agentId}</small>
                      {tray.actions?.length ? (
                        <div className={styles.trayActions}>
                          {tray.actions.map((action, index) => (
                            <button key={`${tray.id}-${action.kind}-${index}`} type="button" onClick={() => {
                              if (action.kind === "open-url") window.open(action.url, "_blank", "noopener,noreferrer");
                              if (action.kind === "switch-model") setSelectedModel("auto");
                            }}>
                              {action.kind === "open-url" ? action.label : action.kind === "switch-model" ? "切换模型" : action.label}
                            </button>
                          ))}
                        </div>
                      ) : null}
                    </div>
                    <button type="button" aria-label={`关闭 ${tray.title}`} onClick={() => void run(() => execute({ type: "tray.dismiss", requestId: nextRequestId("tray-dismiss"), id: tray.id }))}>×</button>
                  </article>
                );
              })}
              {!trays.length ? <p className={styles.trayEmpty}>任务失败、需要恢复的后台工作和其他错误会显示在这里。</p> : null}
            </div>
          </section>
        ) : null}

        <label className={styles.searchBox}>
          <Icon name="search" size={16} />
          <input
            aria-label="搜索会话"
            placeholder="搜索"
            value={conversationSearch}
            onChange={(event) => setConversationSearch(event.target.value)}
          />
        </label>

        {conversationSearch.trim() ? (
          <section className={styles.globalSearchResults} aria-label="全局搜索结果">
            <header>
              <strong>全局搜索</strong>
              <small>{searchPending ? "搜索中…" : `${searchMessageMatches.length} 条消息 · ${searchMediaMatches.length} 个文件`}</small>
            </header>
            {searchMessageMatches.length ? (
              <div className={styles.globalSearchSection}>
                <span>消息</span>
                {searchMessageMatches.map((match) => (
                  <button
                    type="button"
                    key={`${match.agentId}:${match.entryId}`}
                    onClick={() => {
                      setActiveGroupId(null);
                      setActiveAgentId(match.agentId);
                      void run(() => execute({
                        type: "conversation.open",
                        requestId: nextRequestId("search-open"),
                        conversationId: match.conversationId,
                      }));
                    }}
                  >
                    <BotMark botId={match.agentId} size={24} className={styles.globalSearchBotMark} label={match.agentName} />
                    <span><strong>{match.agentName}</strong><small>{match.snippet}</small></span>
                  </button>
                ))}
              </div>
            ) : null}
            {searchMediaMatches.length ? (
              <div className={styles.globalSearchSection}>
                <span>文件</span>
                {searchMediaMatches.map((match) => (
                  <article key={match.path} title={match.path}>
                    <span className={styles.globalSearchFileIcon}>⌑</span>
                    <span><strong>{match.name}</strong><small>{match.agentName} · {Math.max(1, Math.round(match.sizeBytes / 1024))} KB</small></span>
                  </article>
                ))}
              </div>
            ) : null}
            {!searchPending && !searchMessageMatches.length && !searchMediaMatches.length ? <p>没有找到匹配的消息或文件。</p> : null}
          </section>
        ) : null}

        <nav className={styles.agentList} aria-label="智能体会话">
          <button
            className={activeAgentId === "mahayana-assistant" ? styles.agentActive : styles.agentItem}
            type="button"
            onClick={() => { setActiveGroupId(null); setActiveAgentId("mahayana-assistant"); }}
          >
            <BotMark botId="mahayana-assistant" state={backgroundWorkingAgents.has("mahayana-assistant") ? "working" : activeAgentId === "mahayana-assistant" && !activeGroupId ? activeBotState : "idle"} size={38} shape={primaryBotProfile?.avatarShape as BotMarkShape | undefined} color={primaryBotProfile?.avatarColor as BotMarkColor | undefined} className={styles.sidebarBotMark} label={primaryBotProfile?.name || "大乘助手"} />
            <span className={styles.agentCopy}>
              <span><strong>大乘助手</strong><time>现在</time></span>
              <small>{chatDispatching || operationState === "running" ? "正在工作…" : "Mahayana Runtime 已连接"}</small>
            </span>
          </button>
          {visibleConversations
            .filter((conversation) => conversation.id !== "mahayana-ai:agent:assistant")
            .map((conversation) => (
              <button
                key={conversation.id}
                className={activeConversationId === conversation.id ? styles.agentActive : styles.agentItem}
                type="button"
                onClick={() => {
                  setActiveGroupId(null);
                  void run(() =>
                    execute({
                      type: "conversation.open",
                      requestId: nextRequestId("conversation"),
                      conversationId: conversation.id,
                    }),
                  );
                }}
              >
                <span className={styles.avatarAlt}>聊</span>
                <span className={styles.agentCopy}>
                  <span><strong>{conversation.title}</strong></span>
                  <small>{conversation.kind} · {conversation.unreadCount ? `${conversation.unreadCount} 条未读` : "历史会话"}</small>
                </span>
              </button>
            ))}
          {visibleSidebarBots.map((bot) => (
            <button
              key={bot.id}
              className={!activeGroupId && activeConversationId === bot.conversationId ? styles.agentActive : styles.agentItem}
              type="button"
              onClick={() => {
                setActiveGroupId(null);
                setActiveAgentId(bot.id);
                const existingConversation = bot.conversationId
                  ? conversations.find((conversation) => conversation.id === bot.conversationId)
                  : undefined;
                if (existingConversation) {
                  void run(() =>
                    execute({
                      type: "conversation.open",
                      requestId: nextRequestId("bot-conversation"),
                      conversationId: existingConversation.id,
                    }),
                  );
                } else {
                  setActiveConversationId(null);
                  setMessages([]);
                  setTranscriptCards([]);
                  setActivity([]);
                  setUsage(null);
                }
              }}
            >
              <BotMark
                botId={bot.id}
                state={backgroundWorkingAgents.has(bot.id) ? "working" : activeAgentId === bot.id && !activeGroupId ? activeBotState : "idle"}
                size={38}
                shape={bot.avatarShape as BotMarkShape | undefined}
                color={bot.avatarColor as BotMarkColor | undefined}
                className={styles.sidebarBotMark}
                label={bot.name}
              />
              <span className={styles.agentCopy}>
                <span><strong>{bot.name}</strong></span>
                <small>{bot.title ? `${bot.title} · ` : ""}{bot.description}</small>
              </span>
            </button>
          ))}
          <div className={styles.groupListHeading}>
            <span>群聊</span>
            <button type="button" aria-label="新建群聊" onClick={() => setGroupEditorId("new")}>＋</button>
          </div>
          {groups.map((group) => (
            <button
              key={group.id}
              className={activeGroupId === group.id ? styles.agentActive : styles.agentItem}
              type="button"
              onClick={() => {
                setActiveGroupId(group.id);
                setComputerOpen(false);
                setAgentSettingsOpen(false);
              }}
            >
              <GroupAvatarStack group={group} bots={bots} size={26} />
              <span className={styles.agentCopy}>
                <span><strong>{group.name}</strong></span>
                <small>{group.memberIds.length} 个 Bot · {group.messages.length ? `${group.messages.length} 条消息` : "新群聊"}</small>
              </span>
            </button>
          ))}
          {[...installedMiniApps].map((miniAppId) => {
            const app = marketplaceApps.find((item) => item.id === miniAppId);
            if (!app || app.id === "mahayana-assistant") return null;
            return (
              <button
                key={app.id}
                data-testid={`agent-${app.id}`}
                className={activeAgentId === app.id ? styles.agentActive : styles.agentItem}
                type="button"
                onClick={() => {
                  setActiveGroupId(null);
                  setActiveAgentId(app.id);
                  if (openedMiniApp !== app.id) openMiniApp(app.id);
                }}
              >
                <BotMark botId={app.id} state={activeAgentId === app.id ? activeBotState : "idle"} size={38} className={styles.sidebarBotMark} label={`${app.title}机器人`} />
                <span className={styles.agentCopy}>
                  <span><strong>{app.title}机器人</strong></span>
                  <small>{openedMiniApp === app.id ? "应用已打开" : app.description}</small>
                </span>
              </button>
            );
          })}
        </nav>

        <div className={styles.sidebarFooter}>
          <button className={styles.footerButton} type="button" onClick={() => setAutomationOpen(true)}>
            <Icon name="automation" />
            <span>自动化例程</span>
            <em>{automations.length}</em>
          </button>
          <button
            data-testid="open-marketplace"
            className={styles.footerButton}
            type="button"
            onClick={() => setMarketplaceOpen(true)}
          >
            <Icon name="plugins" />
            <span>插件市场</span>
            <em>{installedMiniApps.size}</em>
          </button>
          <button
            className={styles.profileButton}
            data-testid="account-menu"
            type="button"
            onClick={() => setAccountOpen(true)}
          >
            <span className={`${styles.profileAvatar} ${styles.profileAvatarLiving}`}>
              {accountAvatar ? <img src={accountAvatar} alt="" /> : <BotMark botId={`account-${auth?.user?.id ?? displayName}`} state="idle" size={29} color="violet" label={displayName} />}
            </span>
            <span>{displayName}</span>
            <Icon name="settings" size={17} />
          </button>
        </div>
      </aside>

      <section className={styles.workspace}>
        {activeGroup ? (
          <GroupChatPanel
            group={activeGroup}
            bots={bots}
            previews={activeGroupPreviews}
            disabled={hostStatus !== "ready"}
            onSend={sendGroupMessage}
            onEdit={() => setGroupEditorId(activeGroup.id)}
            onDelete={deleteActiveGroup}
          />
        ) : (
          <>
        <header className={styles.chatHeader}>
          <div className={styles.headerIdentity}>
            <BotMark botId={activeBotMarkId} state={activeBotState} size={22} shape={activeBotShape} color={activeBotColor} className={styles.headerBotMark} label={activeBotProfile?.name || (activeAgent ? `${activeAgent.title}机器人` : "大乘助手")} />
            <div>
              <h1>{activeAgent ? `${activeAgent.title}机器人` : "大乘助手"}</h1>
              <p>{activeAgent ? `${activeAgent.description} · Mahayana MiniApp` : "Mahayana Rust Core · 本地优先"}</p>
            </div>
          </div>
          <output
            className={styles.runtimeStatus}
            data-state={hostStatus}
            data-testid="host-status"
            aria-live="polite"
          >
            <span />
            {hostStatus === "ready"
              ? "Host 已连接"
              : hostStatus === "initializing"
                ? "Host 启动中"
                : hostStatus === "closed"
                  ? "Host 已断开"
                  : "Host 连接失败"}
          </output>
          <button
            className={styles.computerButton}
            type="button"
            aria-label={remoteDesktopState?.channelOpen ? "这台电脑正在被已配对手机远程控制" : "大乘助手的电脑"}
            aria-expanded={computerOpen}
            data-remote-live={remoteDesktopState?.channelOpen ? "true" : "false"}
            onClick={() => setComputerOpen((current) => !current)}
          >
            <Icon name="computer" size={16} />
            {remoteDesktopState?.channelOpen ? <span className={styles.remoteLiveDot} aria-hidden="true" /> : null}
          </button>
        </header>

        {error ? <p role="alert" className={styles.error}>{error}</p> : null}

        <div className={styles.conversation}>
          <div className={styles.welcome}>
            <BotMark botId={activeBotMarkId} state={activeBotState} size={66} shape={activeBotShape} color={activeBotColor} className={styles.welcomeBotMark} label={activeBotProfile?.name || (activeAgent ? `${activeAgent.title}机器人` : "大乘助手")} />
            <h2>{activeAgent ? `${activeAgent.title}已连接` : "有什么可以帮你？"}</h2>
            <p>{activeAgent ? "可以在右侧打开应用，也可以通过大乘助手调用它的能力。" : "发送消息、运行任务，或从插件市场为助手添加能力。"}</p>
          </div>
          <div className={styles.messages} data-testid="messages" aria-live="polite">
            {messages.map((message, index) => message.kind === "thinking" ? (
              <article key={`${message.operationId ?? "stream"}-${index}`} className={styles.agentThinkingRow} data-testid="agent-thinking" data-operation-id={message.operationId}>
                <BotMark botId={activeBotMarkId} state="thinking" size={28} shape={activeBotShape} color={activeBotColor} className={styles.messageBotMark} label={activeAgent ? `${activeAgent.title}机器人` : "大乘助手"} />
                <div><strong>{message.title || "正在思考"}</strong><span>大乘助手正在处理这条消息…</span></div>
              </article>
            ) : message.kind === "action" ? (
              <article key={message.id ?? `${message.operationId ?? "stream"}-${index}`} className={styles.agentActionRow} data-testid="agent-step" data-operation-id={message.operationId} data-status={message.status}>
                <BotMark botId={activeBotMarkId} state={message.status === "running" ? "working" : message.status === "failed" ? "error" : "result"} size={25} shape={activeBotShape} color={activeBotColor} className={styles.messageBotMark} label={activeAgent ? `${activeAgent.title}机器人` : "大乘助手"} />
                <div><strong>{message.title}</strong>{message.detail ? <span>{message.detail}</span> : null}</div>
                <small>{message.status === "running" ? "进行中" : message.status === "failed" ? "失败" : "完成"}</small>
              </article>
            ) : (
              <article
                key={`${message.role}-${index}`}
                data-testid={`message-${message.role}`}
                className={message.role === "user" ? styles.userMessage : styles.assistantMessage}
              >
                {message.role === "assistant" ? (
                  <BotMark
                    botId={activeBotMarkId}
                    state={(chatDispatching || operationState === "running") && (!activeOperationId || !message.operationId || message.operationId === activeOperationId) ? activeBotState : "idle"}
                    size={28}
                    shape={activeBotShape}
                    color={activeBotColor}
                    className={styles.messageBotMark}
                    label={activeAgent ? `${activeAgent.title}机器人` : "大乘助手"}
                  />
                ) : null}
                <div>
                  <strong>{message.role === "user" ? "你" : activeAgent ? `${activeAgent.title}机器人` : "大乘助手"}</strong>
                  {message.role === "assistant"
                    ? <RichMessage text={message.text} />
                    : <p>{message.text}</p>}
                </div>
              </article>
            ))}
            {transcriptCards.length ? (
              <div className={styles.transcriptCards}>
                {transcriptCards.map((entry) => (
                  <TranscriptCardView
                    key={entry.entryId}
                    entry={entry}
                    onResolveDraft={resolveDraft}
                    onProvideSecret={provideSecret}
                    onConnectListener={connectListener}
                  />
                ))}
              </div>
            ) : null}
          </div>

          <form className={styles.composer} onSubmit={(event) => void sendMessage(event)}>
            {mentionSuggestions.length ? (
              <div className={styles.mentionMenu} role="listbox" aria-label="能力引用建议">
                {mentionSuggestions.map((capability) => (
                  <button
                    key={capability.id}
                    type="button"
                    role="option"
                    onClick={() => {
                      setInput((current) =>
                        current.replace(/@[^\s@]*$/, `${capability.mention} `),
                      );
                    }}
                  >
                    <span>{capability.title.slice(0, 1)}</span>
                    <div><strong>{capability.title}</strong><small>{capability.mention} · {capability.kind}</small></div>
                    {capability.requiredPermissions.length ? <em>需权限</em> : null}
                  </button>
                ))}
              </div>
            ) : null}
            {attachments.length ? (
              <div className={styles.attachmentList}>
                {attachments.map((attachment) => (
                  <span key={attachment.id}>
                    <Icon name="attach" size={13} /> {attachment.name}
                    <button
                      type="button"
                      aria-label={`移除 ${attachment.name}`}
                      onClick={() =>
                        setAttachments((current) =>
                          current.filter((item) => item.id !== attachment.id),
                        )
                      }
                    >
                      ×
                    </button>
                  </span>
                ))}
                <small>约 {attachmentTokens.toLocaleString()} tokens</small>
              </div>
            ) : null}
            <textarea
              data-testid="chat-input"
              aria-label="消息内容"
              value={input}
              rows={1}
              onChange={(event) => setInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  event.currentTarget.form?.requestSubmit();
                }
              }}
              placeholder={activeAgent ? `给${activeAgent.title}机器人发消息…` : "描述任务，@ 引用文件，或让大乘助手执行操作…"}
              disabled={hostStatus !== "ready"}
            />
            <div className={styles.composerToolbar}>
              <input
                ref={attachmentInput}
                className={styles.fileInput}
                type="file"
                multiple
                onChange={(event) => void attachFiles(event.target.files)}
              />
              <button
                className={styles.toolButton}
                type="button"
                aria-label="添加附件"
                onClick={() => attachmentInput.current?.click()}
              >
                <Icon name="attach" size={17} />
              </button>
              <select
                data-testid="agent-mode"
                aria-label="Agent 模式"
                value={agentMode}
                onChange={(event) => setAgentMode(event.target.value as AgentMode)}
              >
                <option value="agent">Agent</option>
                <option value="ask">问答</option>
                <option value="plan">计划</option>
              </select>
              <select
                data-testid="agent-model"
                aria-label="模型"
                value={selectedModel}
                onChange={(event) => setSelectedModel(event.target.value)}
              >
                {modelOptions.map((model) => (
                  <option key={model.value} value={model.value}>{model.label}</option>
                ))}
              </select>
              <span className={styles.toolbarSpacer} />
              {activeOperationId ? (
                <button
                  className={styles.sendButton}
                  type="button"
                  aria-label="停止任务"
                  onClick={() => void run(() => transport.interrupt(activeOperationId))}
                >
                  <Icon name="stop" />
                </button>
              ) : (
                <button
                  className={styles.sendButton}
                  data-testid="send-message"
                  type="submit"
                  disabled={hostStatus !== "ready" || !input.trim()}
                  aria-label="发送消息"
                >
                  <Icon name="send" />
                </button>
              )}
            </div>
          </form>
        </div>
          </>
        )}
      </section>

      {groupEditorId ? (
        <GroupEditor
          group={editingGroup}
          bots={bots}
          onSave={saveGroup}
          onClose={() => setGroupEditorId(null)}
        />
      ) : null}

      {computerOpen && !activeGroup ? <aside className={styles.activityPanel}>
        <div className={styles.computerPanelHeader}>
          {agentSettingsOpen ? <strong>设置</strong> : <button type="button" aria-label="智能体设置" onClick={openAgentSettings}><Icon name="settings" size={15} /></button>}
          <button type="button" aria-label="关闭电脑面板" onClick={() => setComputerOpen(false)}><Icon name="close" size={16} /></button>
        </div>
        {agentSettingsOpen ? (
          <section className={styles.agentSettingsPanel}>
            <button type="button" onClick={() => setAgentSettingsOpen(false)}>← 返回电脑与例程</button>
            <BotMark botId={activeBotMarkId} state={activeBotState} size={66} shape={activeBotShape} color={activeBotColor} className={styles.settingsBotMark} label={activeBotProfile?.name || (activeAgent ? `${activeAgent.title}机器人` : "大乘助手")} />
            <label><span>名称</span><input value={agentName} onChange={(event) => setAgentName(event.target.value)} onBlur={() => updateActiveBotProfile({ name: agentName })} /></label>
            <label><span>职衔</span><input value={agentTitle} onChange={(event) => setAgentTitle(event.target.value)} onBlur={() => updateActiveBotProfile({ title: agentTitle })} placeholder="描述这个智能体的工作" /></label>
            <label><span>描述</span><textarea value={agentDescription} onChange={(event) => setAgentDescription(event.target.value)} onBlur={() => updateActiveBotProfile({ description: agentDescription })} placeholder="这个智能体用于什么" rows={4} /></label>
            <label className={styles.agentNotification}><span><strong>通知</strong><small>智能体完成工作或需要输入时通知我</small></span><input type="checkbox" checked={agentNotifications} onChange={(event) => { const enabled = event.target.checked; setAgentNotifications(enabled); updateActiveBotProfile({ notificationsEnabled: enabled }); }} /></label>
            {activeBotProfile ? (
              <section className={styles.agentMemoryPanel}>
                <header>
                  <div><strong>Memory</strong><small>{agentMemoryForId === activeBotProfile.id ? `${agentMemoryCount} 条持久记忆` : "正在读取…"}</small></div>
                  {agentMemoryCount > 0 ? <button type="button" onClick={clearAgentMemory}>清空</button> : null}
                </header>
                <div className={styles.agentMemoryComposer}>
                  <select value={agentMemoryKind} onChange={(event) => setAgentMemoryKind(event.target.value as MemoryKind)}>
                    <option value="profile">Profile</option>
                    <option value="log">Log</option>
                  </select>
                  <input value={agentMemoryDraft} maxLength={500} onChange={(event) => setAgentMemoryDraft(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") { event.preventDefault(); addAgentMemory(); } }} placeholder="记录一条持久事实…" />
                  <button type="button" onClick={addAgentMemory} disabled={!agentMemoryDraft.trim()}>＋</button>
                </div>
                <div className={styles.agentMemoryList}>
                  {agentMemories.map((memory) => (
                    <article key={memory.id}>
                      <span data-kind={memory.kind}>{memory.kind}</span>
                      <p>{memory.content}</p>
                      <button type="button" aria-label={`忘记 ${memory.content}`} onClick={() => removeAgentMemory(memory.id)}>×</button>
                    </article>
                  ))}
                  {agentMemoryForId === activeBotProfile.id && !agentMemories.length ? <p className={styles.agentMemoryEmpty}>还没有记忆。Profile 会每轮带入；Log 按重要性与最近时间召回。</p> : null}
                </div>
                {agentMemoryLocation ? <small className={styles.agentMemoryLocation}>{agentMemoryLocation}</small> : null}
              </section>
            ) : null}
            {activeBotProfile ? (
              <AgentWorkflowPanel
                agentId={activeBotProfile.id}
                workflows={agentWorkflowForId === activeBotProfile.id ? agentWorkflows : []}
                onRefresh={refreshAgentWorkflows}
                onSave={saveAgentWorkflow}
                onSetEnabled={setAgentWorkflowEnabled}
                onRun={runAgentWorkflow}
                onDelete={deleteAgentWorkflow}
                onImportMarkdown={importAgentWorkflowMarkdown}
                onImportLiveSource={importAgentWorkflowLiveSource}
              />
            ) : null}
            {activeBotProfile ? (
              <section className={styles.agentAuditPanel}>
                <header>
                  <div><strong>行动审计</strong><small>Shell、MCP 和自动审批的持久记录</small></div>
                  <button type="button" onClick={() => void run(() => execute({ type: "audit.list", requestId: nextRequestId("audit-refresh"), agentId: activeBotProfile.id, limit: 100 }))}>刷新</button>
                </header>
                <div className={styles.agentAuditList}>
                  {(auditAgentId === activeBotProfile.id ? auditRecords : []).slice(-30).reverse().map((record, index) => {
                    const row = objectValue(record);
                    const action = objectValue(row.action);
                    const title = String(action.kind ?? "action");
                    const detail = String(action.command ?? action.toolName ?? action.capability ?? action.status ?? "");
                    return <article key={`${String(row.eventId ?? index)}-${index}`}><span>{title.slice(0, 1).toUpperCase()}</span><div><strong>{title}</strong><p>{detail || JSON.stringify(action)}</p><small>{String(row.ts ?? "")}</small></div></article>;
                  })}
                  {auditAgentId === activeBotProfile.id && !auditRecords.length ? <p className={styles.agentAuditEmpty}>还没有可审计的操作。</p> : null}
                </div>
              </section>
            ) : null}
          </section>
        ) : (
          <>
            <section className={styles.computerPreview} data-ready={computerSnapshot ? "true" : "false"}>
              <header>
                <div>
                  <strong>这台电脑</strong>
                  <small>{computerStatus?.platform === "macos" ? "用户的 Mac" : computerStatus?.platform ?? "正在读取…"}</small>
                </div>
                <span data-active={remoteDesktopState?.channelOpen ? "true" : "false"}>{remoteDesktopState?.channelOpen ? "REMOTE ACTIVE" : preferences.remoteControlEnabled ? "REMOTE READY" : "REMOTE OFF"}</span>
              </header>
              <div className={styles.computerScreen}>
                {computerSnapshot ? (
                  <img
                    src={computerSnapshot.dataUrl}
                    alt="这台电脑的实时屏幕"
                    draggable={false}
                    onClick={clickComputerSnapshot}
                    data-controllable={computerStatus?.accessibilityGranted ? "true" : "false"}
                  />
                ) : (
                  <span className={computerRefreshing || operationState === "running" ? styles.computerPulse : undefined} />
                )}
              </div>
              <div className={styles.computerPermissionStrip}>
                {computerStatus?.screenRecordingGranted ? (
                  <span data-ok="true">屏幕录制 ✓</span>
                ) : (
                  <button type="button" onClick={() => void transport.openSystemSettings("screen-recording").catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)))}>授权屏幕录制</button>
                )}
                {computerStatus?.accessibilityGranted ? (
                  <span data-ok="true">辅助功能 ✓</span>
                ) : (
                  <button type="button" onClick={() => void transport.openSystemSettings("accessibility").catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)))}>授权辅助功能</button>
                )}
                <span data-ok={preferences.aiComputerControlEnabled ? "true" : "false"}>AI {preferences.aiComputerControlEnabled ? "可控制" : "已关闭"}</span>
              </div>
              <small>{computerSnapshot ? "点击画面即可控制本机；手机与 AI 使用同一套动作。" : computerStatus?.screenRecordingGranted === false ? "请在 macOS 隐私与安全中允许屏幕录制。" : "正在连接这台电脑的屏幕…"}</small>
              {preferences.remoteControlEnabled ? (
                <section className={styles.remotePairingPanel}>
                  <header>
                    <div>
                      <strong>手机远程控制</strong>
                      <small>{remoteDesktopState?.channelOpen ? "已建立端到端控制通道" : auth?.loggedIn ? "等待已配对手机连接" : "请先登录同一大乘账户"}</small>
                    </div>
                    {remoteDesktopState?.channelOpen ? <i data-live="true" /> : <i />}
                  </header>
                  {remoteDesktopState?.registration ? (
                    <div className={styles.remotePairingCode}>
                      <span>配对码</span>
                      <strong>{remoteDesktopState.registration.pairingCode}</strong>
                      <small>有效至 {new Date(remoteDesktopState.registration.pairingExpiresAt * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</small>
                      <button type="button" onClick={() => void remoteDesktopControllerRef.current?.refreshPairingCode().catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)))}>刷新</button>
                    </div>
                  ) : null}
                  {remoteDesktopState?.activeSessionId ? (
                    <div className={styles.remoteActiveSession}>
                      <span><strong>正在远控</strong><small>{remoteDesktopState.activeClientId ?? "已配对手机"} · {remoteDesktopState.connectionState}</small></span>
                      <button type="button" onClick={() => void remoteDesktopControllerRef.current?.disconnectActive()}>断开</button>
                    </div>
                  ) : null}
                  {remoteDesktopState?.clients.length ? (
                    <div className={styles.remoteClientList}>
                      {remoteDesktopState.clients.map((client) => (
                        <div key={client.clientId}>
                          <span><strong>{client.label}</strong><small>已配对 · {new Date(client.pairedAt * 1000).toLocaleDateString()}</small></span>
                          <button type="button" onClick={() => void remoteDesktopControllerRef.current?.revokeClient(client.clientId).catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)))}>撤销</button>
                        </div>
                      ))}
                    </div>
                  ) : auth?.loggedIn ? <p>手机打开“远程电脑”，输入上面的 8 位配对码即可绑定。</p> : null}
                  {remoteDesktopState?.error ? <p className={styles.remoteError}>{remoteDesktopState.error}</p> : null}
                </section>
              ) : null}
            </section>
            <section className={styles.routineShortcut}>
              <p>例程是这个智能体按计划反复执行的任务。</p>
              <button type="button" onClick={() => setAutomationOpen(true)}>创建例程</button>
            </section>
          </>
        )}
        <div className={styles.activityHeader}>
          <div>
            <span className={styles.activityKicker}>WORKSPACE</span>
            <h2>运行与能力</h2>
          </div>
          <span className={styles.secureBadge}><Icon name="shield" size={14} /> 本地</span>
        </div>

        <section className={styles.agentRunCard}>
          <div className={styles.cardHeading}>
            <div>
              <strong>Agent 活动</strong>
              <small>{activity.length ? `${activity.length} 个步骤` : "等待任务"}</small>
            </div>
            {usage ? <span className={styles.tokenBadge}>{usage.totalTokens.toLocaleString()} tokens</span> : null}
          </div>
          {activity.length ? (
            <ol className={styles.activityTimeline} data-testid="agent-activity">
              {activity.map((item, index) => (
                <li key={`${item.operationId ?? "runtime"}-${item.id}-${index}`} data-state={item.status}>
                  <i />
                  <div>
                    <strong>{item.title}</strong>
                    {item.detail ? (
                      <details>
                        <summary>{item.kind === "shell" ? "查看终端输出" : "查看详情"}</summary>
                        <pre>{item.kind === "shell" ? formatNumberedLines({ compact: true }, item.detail, 1) : item.detail}</pre>
                      </details>
                    ) : null}
                  </div>
                </li>
              ))}
            </ol>
          ) : (
            <p className={styles.emptyActivity}>模型选择、工具调用和执行进度会显示在这里。</p>
          )}
          {usage?.contextWindow ? (
            <div className={styles.contextUsage}>
              <span style={{ width: `${Math.min(100, usage.totalTokens / usage.contextWindow * 100)}%` }} />
              <small>{usage.model} · 上下文 {Math.round(usage.totalTokens / usage.contextWindow * 100)}%</small>
            </div>
          ) : null}
        </section>

        <section className={styles.teachCard}>
          <div className={styles.cardHeading}>
            <div>
              <strong>Teach</strong>
              <small>录一次操作，保存后自动学习成 Workflow</small>
            </div>
            <span className={styles.teachState} data-state={teachStatus.state}>{teachStatus.state === "recording" ? "REC" : "IDLE"}</span>
          </div>
          {teachStatus.state === "recording" ? (
            <div className={styles.teachRecordingBody}>
              <div className={styles.teachTimer}>
                <i />
                <strong>{formatElapsedMs(Math.min(teachStatus.maxDurationMs, teachClockMs - (teachStatus.startedAtMs ?? teachClockMs)))}</strong>
                <span>/ {formatElapsedMs(teachStatus.maxDurationMs)}</span>
              </div>
              <p>{teachStatus.agentId === activeBotMarkId ? "正在录制当前 Bot 的演示。" : `正在为 ${bots.find((bot) => bot.id === teachStatus.agentId)?.name ?? teachStatus.agentId} 录制。`}</p>
              <div>
                <button
                  type="button"
                  disabled={teachStatus.agentId !== activeBotMarkId}
                  onClick={() => void run(() => execute({ type: "teach.stop", requestId: nextRequestId("teach-save"), agentId: teachStatus.agentId ?? activeBotMarkId, save: true }))}
                >保存并学习</button>
                <button
                  type="button"
                  disabled={teachStatus.agentId !== activeBotMarkId}
                  onClick={() => void run(() => execute({ type: "teach.stop", requestId: nextRequestId("teach-cancel"), agentId: teachStatus.agentId ?? activeBotMarkId, save: false }))}
                >取消录制</button>
              </div>
            </div>
          ) : (
            <div className={styles.teachIdleBody}>
              <p>开始后会录制当前屏幕，最长 10 分钟。停止并保存后，该 Bot 会在后台分析演示并生成可复用的 SKILL.md。</p>
              <button
                type="button"
                onClick={() => { setTeachResult(null); void run(() => execute({ type: "teach.start", requestId: nextRequestId("teach-start"), agentId: activeBotMarkId, entryPoint: "screen_hover" })); }}
              >开始教我操作</button>
            </div>
          )}
          {teachResult ? (
            <div className={styles.teachLastResult} data-saved={teachResult.saved}>
              <strong>{teachResult.saved ? "演示已保存并开始学习" : "演示已丢弃"}</strong>
              <small>{formatElapsedMs(teachResult.durationMs)}{teachResult.saved ? ` · ${teachResult.videoPath}` : ""}</small>
            </div>
          ) : null}
        </section>

        <section className={styles.subagentCard}>
          <div className={styles.cardHeading}>
            <div>
              <strong>Subagents</strong>
              <small>{subagents.length ? `${subagents.length} 个子 Agent · ${asyncTasks.length} 个后台任务运行中` : "Codex 多 Agent"}</small>
            </div>
            <button
              type="button"
              onClick={() => {
                const agentId = subagentAgentId ?? (bots.some((bot) => bot.id === activeAgentId) ? activeAgentId : "mahayana-assistant");
                const stamp = Date.now();
                void run(() => Promise.all([
                  execute({ type: "subagent.list", requestId: `subagent-refresh-${stamp}`, agentId }),
                  execute({ type: "asyncTask.list", requestId: `async-task-refresh-${stamp}`, agentId }),
                ]).then(() => undefined));
              }}
            >刷新</button>
          </div>
          {asyncTasks.length ? (
            <div className={styles.asyncTaskStrip}>
              {asyncTasks.map((task) => {
                const cloudRunId = task.kind === "cloud-agent" ? task.resourceId : undefined;
                const content = (
                  <>
                    <span data-kind={task.kind}>↻</span>
                    <div><strong>{task.label}</strong><small>{task.kind}{task.subagentType ? ` · ${task.subagentType}` : ""}{cloudRunId ? " · 打开 Cloud Run" : ""}</small></div>
                  </>
                );
                return cloudRunId ? (
                  <button className={styles.asyncTaskCard} key={task.id} type="button" onClick={() => void openCloudRun(cloudRunId)}>{content}</button>
                ) : (
                  <article className={styles.asyncTaskCard} key={task.id}>{content}</article>
                );
              })}
            </div>
          ) : null}
          <div className={styles.subagentList}>
            {subagents.map((subagent) => (
              <article key={subagent.id} data-status={subagent.status}>
                <span><i /></span>
                <div>
                  <strong>{subagent.title}</strong>
                  <small>{subagent.subagentType} · {subagent.status} · {subagent.id.slice(0, 10)}</small>
                  {subagent.detail ? <p>{subagent.detail}</p> : null}
                </div>
              </article>
            ))}
            {!subagents.length ? <p className={styles.subagentEmpty}>模型调用 spawn_agent 后，子 Agent 的身份、状态和后台任务会显示在这里。</p> : null}
          </div>
        </section>

        <section className={styles.controlCard}>
          <div className={styles.cardHeading}>
            <div>
              <strong>Runtime 任务</strong>
              <small data-testid="operation-state">{operationState}</small>
            </div>
            <span data-state={operationState} className={styles.stateDot} />
          </div>
          <div className={styles.controlActions}>
            <button
              data-testid="start-long-operation"
              type="button"
              onClick={() =>
                void run(() =>
                  execute({
                    type: "runtime.longTask",
                    requestId: nextRequestId("long-task"),
                    label: "长任务测试",
                  }),
                )
              }
            >
              启动长任务
            </button>
            <button
              data-testid="interrupt-operation"
              type="button"
              disabled={!activeOperationId}
              onClick={() =>
                activeOperationId
                  ? void run(() => transport.interrupt(activeOperationId))
                  : undefined
              }
            >
              <Icon name="stop" size={15} /> 中断
            </button>
          </div>
        </section>

        <section className={styles.controlCard}>
          <div className={styles.cardHeading}>
            <div>
              <strong>安全会话</strong>
              <small data-testid="session-state">{sessionState}</small>
            </div>
          </div>
          <button
            className={styles.fullButton}
            data-testid="clear-session"
            type="button"
            onClick={() =>
              void run(() =>
                execute({
                  type: "session.clear",
                  requestId: nextRequestId("session"),
                }),
              )
            }
          >
            清除本地会话
          </button>
        </section>

        {activeMiniApp ? (
          <section className={styles.miniAppPanel} data-testid="miniapp-panel">
            <div className={`${styles.marketIcon} ${styles[activeMiniApp.tone]}`}>
              {activeMiniApp.glyph}
            </div>
            <div>
              <strong>{activeMiniApp.title}</strong>
              <small>{activeMiniApp.id}</small>
            </div>
            <p>隔离 MiniApp 容器已打开。</p>
            {openedMiniAppHtml ? (
              <iframe
                className={styles.miniAppFrame}
                data-testid="miniapp-frame"
                title={`${activeMiniApp.title}应用`}
                sandbox="allow-scripts"
                srcDoc={openedMiniAppHtml}
              />
            ) : null}
            <button
              data-testid="request-capability"
              type="button"
              onClick={() =>
                void run(() =>
                  execute({
                    type: "capability.request",
                    requestId: nextRequestId("capability"),
                    miniAppId: activeMiniApp.id,
                    capability: "microphone.request",
                    reason: "为语音布施功能录制音频",
                  }),
                )
              }
            >
              请求麦克风权限
            </button>
            <output data-testid="approval-state">{approvalState}</output>
          </section>
        ) : (
          <button
            className={styles.emptyMiniApp}
            type="button"
            onClick={() => setMarketplaceOpen(true)}
          >
            <Icon name="plugins" />
            <span><strong>添加插件</strong><small>扩展助手能力</small></span>
          </button>
        )}

        <section className={styles.coverageCard}>
          <div className={styles.coverageTitle}>
            <strong>自动化覆盖</strong>
            <span>{Object.values(featureStates).filter((state) => state === "passed").length}/{mahayanaHostFeatures.length}</span>
          </div>
          <ul data-testid="feature-coverage">
            {mahayanaHostFeatures.map((feature) => (
              <li
                key={feature.id}
                data-testid={`feature-result-${feature.id}`}
                data-state={featureStates[feature.id]}
              >
                <span>{feature.label}</span>
                <i />
              </li>
            ))}
          </ul>
        </section>
      </aside> : null}

      {marketplaceOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setMarketplaceOpen(false)}>
          <section
            className={styles.marketplace}
            role="dialog"
            aria-modal="true"
            aria-labelledby="marketplace-title"
            onMouseDown={(event) => event.stopPropagation()}
          >
            <header>
              <div>
                <p>FABUSHI EXTENSIONS</p>
                <h2 id="marketplace-title">插件市场</h2>
                <span>为所有智能体安装官方 Mahayana 插件。</span>
              </div>
              <button
                className={styles.iconButton}
                type="button"
                aria-label="关闭插件市场"
                onClick={() => setMarketplaceOpen(false)}
              >
                <Icon name="close" />
              </button>
            </header>

            <MarketplaceTabs
              section={marketplaceSection}
              onChange={setMarketplaceSection}
              counts={{
                apps: marketplaceApps.length,
                connectors: connectors.length,
                skills: skills.length,
                bots: bots.length,
              }}
            />

            <label className={styles.marketSearch}>
              <Icon name="search" size={16} />
              <input
                value={marketplaceSearch}
                onChange={(event) => setMarketplaceSearch(event.target.value)}
                placeholder={marketplaceSection === "apps" ? "搜索插件" : `搜索 ${marketplaceSection}`}
              />
            </label>

            {marketplaceSection === "apps" ? (
              <div className={styles.marketList}>
              {visibleMarketplaceApps.map((app) => {
                const installed = installedMiniApps.has(app.id);
                const installRecord = installedMiniAppRecords[app.id];
                const action = installRecord
                  ? marketplaceAppInstallAction(app.id, installRecord)
                  : installed
                    ? "current"
                    : marketplaceAppInstallAction(app.id);
                const busy = busyMiniApp === app.id;
                return (
                  <article key={app.id} className={styles.marketRow}>
                    <div className={`${styles.marketIcon} ${styles[app.tone]}`}>
                      {app.glyph}
                    </div>
                    <div className={styles.marketCopy}>
                      <div>
                        <strong>{app.title}</strong>
                        {installed ? <span className={styles.connectedDot} /> : null}
                      </div>
                      <p>{app.description}</p>
                      <div className={styles.marketMeta}>
                        <span>{app.developer}</span>
                        <a href={`/apps/${app.slug}`}>详情 / 内容</a>
                      </div>
                    </div>
                    <button
                      data-testid={app.id === defaultMiniAppId ? "install-miniapp" : `install-${app.id}`}
                      type="button"
                      disabled={busy || action === "current" || action === "blocked" || action === "unavailable"}
                      onClick={() => installMiniApp(app.id)}
                    >
                      {busy ? "处理中…" : marketplaceInstallActionLabel(action)}
                    </button>
                    <button
                      data-testid={app.id === defaultMiniAppId ? "open-miniapp" : `open-${app.id}`}
                      type="button"
                      disabled={!installed || busy}
                      onClick={() => openMiniApp(app.id)}
                    >
                      打开
                    </button>
                    {app.id === defaultMiniAppId ? (
                      <output className={styles.srOnly} data-testid="marketplace-state">
                        {installed ? "installed" : "not-installed"}
                      </output>
                    ) : null}
                  </article>
                );
              })}
              {visibleMarketplaceApps.length === 0 ? (
                <p className={styles.noResults}>没有匹配的插件。</p>
              ) : null}
              </div>
            ) : (
              <ExtensionStudio
                section={marketplaceSection}
                connectors={connectors}
                skills={skills}
                skillTeams={skillTeams}
                bots={bots}
                search={marketplaceSearch}
                execute={execute}
                nextRequestId={nextRequestId}
                run={run}
                onDeleteBot={confirmBotDelete}
              />
            )}
          </section>
        </div>
      ) : null}

      {networkOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setNetworkOpen(false)}>
          <section className={styles.networkDialog} role="dialog" aria-modal="true" aria-labelledby="network-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div><p>AGENT NETWORK</p><h2 id="network-title">智能体网络</h2><span>查看 Agents、Bots、插件和联系人之间的能力关系。</span></div>
              <button className={styles.iconButton} type="button" aria-label="关闭智能体网络" onClick={() => setNetworkOpen(false)}><Icon name="close" /></button>
            </header>
            <nav className={styles.networkTabs} aria-label="智能体网络视图">
              <button type="button" data-active={networkView === "agents"} onClick={() => setNetworkView("agents")}>Agent Network</button>
              <button type="button" data-active={networkView === "rooms"} onClick={() => setNetworkView("rooms")}>Shared Rooms</button>
              <button type="button" data-active={networkView === "workspace"} onClick={() => setNetworkView("workspace")}>Workspace</button>
            </nav>
            {networkView === "agents" ? (
              <>
            <div className={styles.networkCanvas}>
              <span className={styles.networkLine} />
              <article className={styles.networkHub}>
                <BotMark
                  botId={networkSender?.id ?? "mahayana-assistant"}
                  state={networkSender && backgroundWorkingAgents.has(networkSender.id) ? "working" : activeBotState}
                  size={48}
                  shape={networkSender?.avatarShape as BotMarkShape | undefined}
                  color={networkSender?.avatarColor as BotMarkColor | undefined}
                  className={styles.networkHubBotMark}
                  label={networkSender?.name ?? "大乘助手"}
                />
                <strong>{networkSender?.name ?? "大乘助手"}</strong><small>当前发送方 · Agent</small>
              </article>
              <div className={styles.networkNodes}>
                {bots.filter((bot) => bot.id !== networkSender?.id).map((bot) => (
                  <button
                    key={bot.id}
                    type="button"
                    data-selected={networkTargetId === bot.id}
                    onClick={() => setNetworkTargetId(bot.id)}
                  >
                    <BotMark botId={bot.id} state={backgroundWorkingAgents.has(bot.id) ? "working" : "idle"} size={31} shape={bot.avatarShape as BotMarkShape | undefined} color={bot.avatarColor as BotMarkColor | undefined} className={styles.networkNodeBotMark} label={bot.name} />
                    <strong>{bot.name}</strong>
                    <small>{backgroundWorkingAgents.has(bot.id) ? "正在后台工作" : bot.description || "Agent"}</small>
                  </button>
                ))}
                {networkSender ? groups.filter((group) => group.memberIds.includes(networkSender.id)).map((group) => (
                  <button key={group.id} type="button" data-selected={networkTargetId === group.id} onClick={() => setNetworkTargetId(group.id)}>
                    <GroupAvatarStack group={group} bots={bots} size={31} />
                    <strong>{group.name}</strong>
                    <small>群聊 · {group.memberIds.length} 个成员</small>
                  </button>
                )) : null}
              </div>
              {!networkTargets.length ? (
                <div className={styles.networkEmpty}><Icon name="network" size={26} /><strong>还没有其他智能体</strong><p>创建几个 Bot 后，可以让它们异步互相协作。</p><button type="button" onClick={() => { setNetworkOpen(false); setMarketplaceOpen(true); setMarketplaceSection("bots"); }}>创建 Bot</button></div>
              ) : null}
            </div>
            <section className={styles.networkMessaging}>
              <header>
                <div><strong>Agent-to-Agent</strong><small>异步发送；目标 Bot 会在自己的会话中被隐藏 turn 唤醒。</small></div>
                {lastBroadcastResult ? <em>{lastBroadcastResult.scheduled}/{lastBroadcastResult.total} 已调度</em> : null}
              </header>
              <div className={styles.networkComposer}>
                <select value={networkTargetId} onChange={(event) => setNetworkTargetId(event.target.value)}>
                  <option value="">选择目标</option>
                  {networkTargets.map((target) => <option key={target.id} value={target.id}>{target.name}</option>)}
                </select>
                <textarea value={networkMessage} onChange={(event) => setNetworkMessage(event.target.value)} rows={3} maxLength={8000} placeholder="给另一个 Agent 或群聊发送信息…" />
                <label><input type="checkbox" checked={networkPriority} onChange={(event) => setNetworkPriority(event.target.checked)} /><span>优先消息</span></label>
                <div>
                  <button type="button" disabled={!networkSender || !networkTargetId || !networkMessage.trim()} onClick={() => void sendNetworkMessage()}>发送给目标</button>
                  <button type="button" disabled={!networkMessage.trim()} onClick={() => void broadcastNetworkMessage()}>广播给全部 Bot</button>
                </div>
              </div>
              <div className={styles.peerMessageList}>
                {peerMessages
                  .filter((message) => !networkSender || message.fromAgentId === networkSender.id || message.targetId === networkSender.id)
                  .slice(-20)
                  .reverse()
                  .map((message) => (
                    <article key={message.id}>
                      <span>{message.priority ? "!" : "↗"}</span>
                      <div><strong>{message.fromAgentName} → {message.targetName}</strong><p>{message.text}</p><small>{new Date(message.createdAtMs).toLocaleString()}</small></div>
                    </article>
                  ))}
                {!peerMessages.length ? <p className={styles.networkNoMessages}>还没有 Agent 间消息。</p> : null}
              </div>
            </section>
              </>
            ) : networkView === "rooms" ? (
              <section className={styles.sharedRoomsPanel} aria-label="共享房间">
                <header>
                  <div>
                    <strong>跨设备共享房间</strong>
                    <small>{sharingState.scope === "fabushi-platform" ? "Fabushi Platform · 已持久化" : "本机离线模式 · 网络恢复后可切回平台"}</small>
                  </div>
                  <button type="button" disabled={sharingBusy} onClick={() => void refreshSharingState()}>{sharingBusy ? "同步中…" : "刷新"}</button>
                </header>
                {sharingError ? <p className={styles.sharedRoomsError}>{sharingError}</p> : null}
                <div className={styles.sharedRoomsForms}>
                  <form onSubmit={(event) => { event.preventDefault(); void createSharedRoomForActiveAgent(); }}>
                    <label><span>创建共享房间</span><input value={sharedRoomName} onChange={(event) => setSharedRoomName(event.target.value)} maxLength={96} placeholder="例如：发布协作室" /></label>
                    <button type="submit" disabled={sharingBusy || !sharedRoomName.trim()}>用当前 Agent 创建</button>
                  </form>
                  <form onSubmit={(event) => { event.preventDefault(); void joinSharedRoomFromInvite(); }}>
                    <label><span>加入邀请码</span><input value={sharedInviteToken} onChange={(event) => setSharedInviteToken(event.target.value)} maxLength={1000} placeholder="fabushi_…" /></label>
                    <button type="submit" disabled={sharingBusy || !sharedInviteToken.trim()}>申请加入</button>
                  </form>
                </div>
                {lastSharedInvite ? (
                  <div className={styles.sharedInviteResult}>
                    <span>邀请码 · {new Date(lastSharedInvite.expiresAtMs).toLocaleString()} 前有效</span>
                    <code>{lastSharedInvite.token}</code>
                    <button type="button" onClick={() => void navigator.clipboard?.writeText(lastSharedInvite.token)}>复制</button>
                  </div>
                ) : null}
                <div className={styles.sharedRoomList}>
                  {sharingState.rooms.map((room) => {
                    const ownAgentIds = room.ownAgentIds ?? room.memberAgentIds.filter((id) => bots.some((bot) => bot.id === id) || id === "mahayana-assistant");
                    const activeIsOwnMember = ownAgentIds.includes(activeAgentId);
                    const typing = (sharingState.typing ?? []).filter((entry) => entry.roomId === room.id && entry.isTyping);
                    return (
                      <article key={room.id}>
                        <header>
                          <div><strong>{room.name}</strong><small>{room.memberCount ?? room.memberAgentIds.length} 个成员 · {room.isOwner ? "你创建的" : "已加入"}</small></div>
                          <span data-scope={room.scope}>{room.scope === "fabushi-platform" ? "CLOUD" : "LOCAL"}</span>
                        </header>
                        <p>{room.memberAgentIds.slice(0, 8).join(" · ") || "暂无成员"}</p>
                        {typing.length ? <small className={styles.sharedTyping}>{typing.map((entry) => entry.participantId).join("、")} 正在输入…</small> : null}
                        <div className={styles.sharedRoomActions}>
                          <button type="button" disabled={sharingBusy} onClick={() => void createSharedInvite(room.id)}>邀请</button>
                          {!activeIsOwnMember ? <button type="button" disabled={sharingBusy} onClick={() => void mutateSharedRoom(() => coordinator.addOwnAgentToSharedRoom(room.id, activeAgentId))}>加入当前 Agent</button> : null}
                          {activeIsOwnMember ? <button type="button" disabled={sharingBusy} onClick={() => pulseSharedRoomTyping(room.id)}>输入状态</button> : null}
                          {activeIsOwnMember && ownAgentIds.length > 1 ? <button type="button" disabled={sharingBusy} onClick={() => confirmRemoveAgentFromSharedRoom(room.id, room.name)}>移除当前 Agent</button> : null}
                          {activeIsOwnMember && ownAgentIds.length <= 1 ? <button type="button" disabled={sharingBusy} onClick={() => confirmLeaveSharedRoom(room.id, room.name)}>离开房间</button> : null}
                        </div>
                      </article>
                    );
                  })}
                  {!sharingState.rooms.length ? <div className={styles.sharedRoomsEmpty}><Icon name="network" /><strong>还没有共享房间</strong><p>创建一个房间，或用邀请码把当前 Agent 加入别人的房间。</p></div> : null}
                </div>
                {sharingState.joinRequests.some((request) => request.status === "pending") ? (
                  <section className={styles.sharedJoinRequests}>
                    <h3>加入请求</h3>
                    {sharingState.joinRequests.filter((request) => request.status === "pending").map((request) => (
                      <article key={request.id}>
                        <div><strong>{request.displayName}</strong><small>{request.agentId} · {request.isOwnRequest ? "等待房主审批" : "请求加入"}</small></div>
                        {!request.isOwnRequest ? <div><button type="button" disabled={sharingBusy} onClick={() => void mutateSharedRoom(() => coordinator.respondToRoomJoinRequest(request.id, true))}>接受</button><button type="button" disabled={sharingBusy} onClick={() => void mutateSharedRoom(() => coordinator.respondToRoomJoinRequest(request.id, false))}>拒绝</button></div> : null}
                      </article>
                    ))}
                  </section>
                ) : null}
              </section>
            ) : (
              <section className={styles.workspacePanel} aria-label="Agent workspace">
                <header>
                  <div>
                    <strong>{activeBotProfile?.name ?? activeAgentId} · Workspace</strong>
                    <small>持久工作区、密钥状态、managed egress 与存储审计</small>
                  </div>
                  <button type="button" disabled={workspaceBusy} onClick={() => void refreshWorkspaceState()}>{workspaceBusy ? "检查中…" : "刷新"}</button>
                </header>
                {workspaceError ? <p className={styles.sharedRoomsError}>{workspaceError}</p> : null}
                <div className={styles.workspaceStatusGrid}>
                  <article><span>Workspace</span><strong>{workspaceStatus?.status ?? "unknown"}</strong><small>{workspaceStatus?.provider ?? "尚未配置"}</small></article>
                  <article><span>Box secrets</span><strong>{workspaceSecrets?.secretCount ?? 0}</strong><small>{workspaceSecrets?.configured ? "已加密保存" : "暂无 Box secret"}</small></article>
                  <article><span>Egress relay</span><strong>{workspaceEgressAvailable ? "available" : "unavailable"}</strong><small>Fabushi managed HTTPS relay</small></article>
                  <article><span>Agent network</span><strong>{workspaceAgentNetworkEnabled ? "enabled" : "not routed"}</strong><small>只有 runtime 真接入 relay 才会启用</small></article>
                </div>
                <div className={styles.workspaceActions}>
                  {workspaceStatus?.status !== "ready" ? <button type="button" disabled={workspaceBusy} onClick={() => void ensureActiveWorkspace()}>创建持久 Workspace</button> : null}
                  {workspaceStatus?.status === "ready" ? <button type="button" disabled={workspaceBusy} onClick={() => void handBackActiveWorkspace()}>归档并释放 Workspace</button> : null}
                  <button type="button" disabled={workspaceBusy} onClick={() => void refreshWorkspaceState()}>重新审计磁盘</button>
                </div>
                {workspaceStatus?.boxId ? <code className={styles.workspaceBoxId}>{workspaceStatus.boxId}</code> : null}
                <section className={styles.workspaceDiskAudit}>
                  <header><strong>桌面存储审计</strong><small>{workspaceDiskAudit ? `${Math.round(workspaceDiskAudit.totalBytes / 1024 / 1024)} MiB 总占用 · ${Math.round(workspaceDiskAudit.reclaimableBytes / 1024 / 1024)} MiB 保守可回收` : "尚未审计"}</small></header>
                  <div>
                    {(workspaceDiskAudit?.entries ?? []).slice(0, 8).map((entry) => (
                      <article key={entry.name}><span>{entry.name}</span><strong>{Math.max(0, Math.round(entry.bytes / 1024 / 1024))} MiB</strong><small>{entry.reclaimable ? "cache/log/temp" : "保留"}</small></article>
                    ))}
                    {workspaceDiskAudit?.truncated ? <p>目录过大，审计达到安全扫描上限；结果是保守估计。</p> : null}
                  </div>
                </section>
              </section>
            )}
          </section>
        </div>
      ) : null}

      {automationOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setAutomationOpen(false)}>
          <section className={styles.automationDialog} role="dialog" aria-modal="true" aria-labelledby="automation-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div><p>ALWAYS-ON ROUTINES</p><h2 id="automation-title">自动化例程</h2><span>按计划或外部事件唤醒大乘助手，执行保存的 standing instruction。</span></div>
              <button className={styles.iconButton} type="button" aria-label="关闭自动化" onClick={() => setAutomationOpen(false)}><Icon name="close" /></button>
            </header>
            <div className={styles.automationBody}>
              <div className={styles.automationList}>
                {automations.map((automation) => (
                  <article key={automation.id}>
                    <span data-enabled={automation.enabled}><Icon name="automation" size={15} /></span>
                    <div><strong>{automation.name}</strong><p>{automation.prompt}</p><small>{automationTriggerLabel(automation)} · {automationRunLabel(automation)}</small></div>
                    <div className={styles.automationActions}>
                      <button type="button" onClick={() => void run(() => execute({ type: "automation.run", requestId: nextRequestId("automation-run"), id: automation.id }))}>运行</button>
                      <button type="button" onClick={() => void run(() => execute({ type: "automation.setEnabled", requestId: nextRequestId("automation-toggle"), id: automation.id, enabled: !automation.enabled }))}>{automation.enabled ? "暂停" : "恢复"}</button>
                      <button type="button" onClick={() => editAutomation(automation)}>编辑</button>
                      <button type="button" onClick={() => confirmAutomationDelete(automation.id, automation.name)}>删除</button>
                    </div>
                  </article>
                ))}
                {!automations.length ? <div className={styles.automationEmpty}><Icon name="automation" size={25} /><strong>还没有例程</strong><p>创建一个日报、周期检查或提醒。</p></div> : null}
              </div>
              <form className={styles.automationForm} onSubmit={(event) => void saveAutomation(event)}>
                <h3>{editingAutomationId ? "编辑例程" : "新建例程"}</h3>
                <label><span>名称</span><input value={automationName} onChange={(event) => setAutomationName(event.target.value)} placeholder="例如：早间摘要" required /></label>
                <label><span>执行指令</span><textarea value={automationPrompt} onChange={(event) => setAutomationPrompt(event.target.value)} placeholder="告诉智能体每次唤醒后要完成什么" rows={6} required /></label>
                <fieldset className={styles.triggerPicker}>
                  <legend>触发方式</legend>
                  <label><input type="radio" name="automation-trigger" checked={automationTriggerKind === "schedule"} onChange={() => setAutomationTriggerKind("schedule")} /><span>计划</span></label>
                  <label><input type="radio" name="automation-trigger" checked={automationTriggerKind === "event"} onChange={() => setAutomationTriggerKind("event")} /><span>事件</span></label>
                </fieldset>
                {automationTriggerKind === "schedule" ? (
                  <label><span>计划</span><input value={automationSchedule} onChange={(event) => setAutomationSchedule(event.target.value)} placeholder="@daily 或 0 9 * * 1-5" required /><small>支持 5 段 cron、@hourly、@daily、@weekly 与 @every 5m。</small></label>
                ) : (
                  <div className={styles.eventTriggerFields}>
                    <label><span>事件来源</span><select value={automationEventSource} onChange={(event) => {
                      const source = event.target.value as ListenerPlatform;
                      setAutomationEventSource(source);
                      const option = listenerOptions.find((item) => item.value === source);
                      if (option) setAutomationEventName(option.defaultEvent);
                    }}>{listenerOptions.map((option) => <option value={option.value} key={option.value}>{option.label}</option>)}</select></label>
                    <label><span>事件</span><input value={automationEventName} onChange={(event) => setAutomationEventName(event.target.value)} placeholder="pull_request.opened" required /></label>
                    <label><span>过滤条件（可选）</span><input value={automationEventFilter} onChange={(event) => setAutomationEventFilter(event.target.value)} placeholder="例如：repository == org/repo" /></label>
                    <div className={styles.listenerStatus} data-state={selectedListenerState}>
                      <span aria-hidden="true" />
                      <div>
                        <strong>{selectedListenerIntegration?.displayName ?? automationEventSource}</strong>
                        <small>{selectedListenerIntegration?.isConnected ? selectedListenerIntegration.accountLabel || "监听器已连接" : selectedListenerIntegration?.blurb || "需要连接事件来源"}</small>
                        <em>{selectedListenerStateCopy} · {selectedListenerNeededByCount ? `${selectedListenerNeededByCount} 个启用例程依赖此来源` : "暂无启用例程依赖此来源"}</em>
                        {selectedListenerIntegration?.error ? <b>{selectedListenerIntegration.error}</b> : null}
                      </div>
                      {!selectedListenerIntegration?.isConnected ? <button type="button" disabled={selectedListenerState === "connecting"} onClick={() => connectListener(automationEventSource)}>{selectedListenerState === "connecting" ? "连接中…" : "连接"}</button> : null}
                    </div>
                  </div>
                )}
                <div><button type="button" onClick={resetAutomationDraft}>清空</button><button type="submit">{editingAutomationId ? "保存修改" : "创建例程"}</button></div>
              </form>
            </div>
          </section>
        </div>
      ) : null}

      {settingsOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setSettingsOpen(false)}>
          <section
            className={styles.settingsDialog}
            role="dialog"
            aria-modal="true"
            aria-labelledby="settings-title"
            onMouseDown={(event) => event.stopPropagation()}
          >
            <aside className={styles.settingsNav}>
              <div className={styles.settingsBrand}>
                <span className={`${styles.profileAvatar} ${styles.profileAvatarLiving}`}>
                  {accountAvatar ? <img src={accountAvatar} alt="" /> : <BotMark botId={`account-${auth?.user?.id ?? displayName}`} state="idle" size={30} color="violet" label={displayName} />}
                </span>
                <div><strong>{displayName}</strong><small>{auth?.user?.email || `${accountProvider} · 安全会话`}</small></div>
              </div>
              <p>设置分区</p>
              {([
                ["general", "通用", "General"],
                ["mcp", "MCP", "MCP Servers"],
                ["usage", "用量与计费", "Usage & Billing"],
                ["updates", "更新", "Updates"],
              ] as const).map(([id, label, english]) => (
                <button
                  key={id}
                  type="button"
                  className={settingsSection === id ? styles.settingsNavActive : undefined}
                  onClick={() => setSettingsSection(id)}
                >
                  <span>{label}</span><small>{english}</small>
                </button>
              ))}
              <span className={styles.settingsNavSpacer} />
              <button type="button" onClick={() => { setSettingsOpen(false); setAccountOpen(true); }}>
                <span>账户</span><small>Account</small>
              </button>
            </aside>

            <div className={styles.settingsContent}>
              <header>
                <div>
                  <p>FABUSHI AGENT SETTINGS</p>
                  <h2 id="settings-title">
                    {settingsSection === "general" ? "通用设置" : settingsSection === "mcp" ? "MCP 与 Apps" : settingsSection === "usage" ? "用量与计费" : "全球法布施更新"}
                  </h2>
                </div>
                <button className={styles.iconButton} type="button" aria-label="关闭设置" onClick={() => setSettingsOpen(false)}>
                  <Icon name="close" />
                </button>
              </header>

              {settingsSection === "general" ? (
                <div className={styles.settingsSections}>
                  <SettingsGroup title="外观" description="Appearance">
                    <label className={styles.settingsRow}>
                      <span><strong>主题</strong><small>跟随系统，或固定使用浅色/深色主题。</small></span>
                      <select value={preferences.theme} onChange={(event) => setPreference("theme", event.target.value as ThemePreference)}>
                        <option value="system">跟随系统</option><option value="light">浅色</option><option value="dark">深色</option>
                      </select>
                    </label>
                  </SettingsGroup>

                  <SettingsGroup title="通知" description="Notifications">
                    <ToggleRow
                      checked={preferences.notifyOnUpdates}
                      label="任务状态通知"
                      description="智能体完成任务或需要输入时通知我。"
                      onChange={(value) => setPreference("notifyOnUpdates", value)}
                    />
                  </SettingsGroup>

                  <SettingsGroup title="本机能力" description="Execution on Local Computer">
                    <ToggleRow
                      checked={preferences.localExecution}
                      label="允许在本机执行"
                      description="允许助手打开文件并在这台电脑上运行任务；自动审批仍会检查每个操作。"
                      onChange={(value) => setPreference("localExecution", value)}
                    />
                    <ToggleRow
                      checked={preferences.routeEgressLocally}
                      label="通过此桌面路由网络出口"
                      description="让智能体的新网络连接通过这台电脑，而不是云端出口。"
                      onChange={(value) => setPreference("routeEgressLocally", value)}
                    />
                    <ToggleRow
                      checked={preferences.remoteControlEnabled}
                      label="允许已配对手机远程控制此电脑"
                      description="默认关闭。开启后，只有当前账户明确配对的设备才能建立加密远控会话；桌面会持续显示远控状态。"
                      onChange={(value) => setPreference("remoteControlEnabled", value)}
                    />
                    <ToggleRow
                      checked={preferences.aiComputerControlEnabled}
                      label="允许 AI 控制此电脑"
                      description="AI 使用与手机完全相同的截图、点击、移动、拖拽、输入、按键、滚动和等待动作，并受本机工具审批规则约束。"
                      onChange={(value) => setPreference("aiComputerControlEnabled", value)}
                    />
                    <ToggleRow
                      checked={preferences.securityKeys}
                      label="使用硬件安全密钥"
                      description="允许智能体请求使用连接到电脑的安全密钥，每次使用仍需确认。"
                      onChange={(value) => setPreference("securityKeys", value)}
                    />
                    <label className={styles.settingsRow}>
                      <span><strong>本机工具权限</strong><small>Never 会拒绝本机命令；Ask 每次审批；Always 只对本机工具建立会话级授权。</small></span>
                      <select value={preferences.localToolPermission} onChange={(event) => setPreference("localToolPermission", event.target.value as HostPreferences["localToolPermission"])}>
                        <option value="never">Never</option><option value="ask">Ask</option><option value="always">Always</option>
                      </select>
                    </label>
                    <ToggleRow
                      checked={preferences.webauthnProxyEnabled}
                      label="WebAuthn 代理"
                      description="允许网页认证请求转交给桌面安全密钥代理；实际使用仍受审批和平台能力限制。"
                      onChange={(value) => setPreference("webauthnProxyEnabled", value)}
                    />
                  </SettingsGroup>

                  <SettingsGroup title="自动审批" description="Auto-review Rules">
                    <p className={styles.settingsHelp}>内置安全检查始终生效；“先询问”规则在冲突时优先。</p>
                    <div className={styles.ruleComposer}>
                      <input value={ruleDraft} onChange={(event) => setRuleDraft(event.target.value)} placeholder="例如：允许为我运行只读测试" />
                      <select value={ruleBehavior} onChange={(event) => setRuleBehavior(event.target.value as "allow" | "ask")}>
                        <option value="allow">自动允许</option><option value="ask">先询问</option>
                      </select>
                      <button type="button" disabled={!ruleDraft.trim()} onClick={addAutoReviewRule}>添加规则</button>
                    </div>
                    {preferences.autoReviewRules.length ? (
                      <ul className={styles.ruleList}>
                        {preferences.autoReviewRules.map((rule) => (
                          <li key={rule.id}>
                            <span data-behavior={rule.behavior}>{rule.behavior === "allow" ? "允许" : "询问"}</span>
                            <p>{rule.text}</p>
                            <button type="button" aria-label={`删除规则 ${rule.text}`} onClick={() => setPreference("autoReviewRules", preferences.autoReviewRules.filter((item) => item.id !== rule.id))}>×</button>
                          </li>
                        ))}
                      </ul>
                    ) : <p className={styles.settingsEmpty}>尚未添加自定义规则。</p>}
                  </SettingsGroup>
                </div>
              ) : settingsSection === "mcp" ? (
                <div className={styles.settingsSections}>
                  <SettingsGroup title="MCP Servers" description="Model Context Protocol">
                    <div className={styles.mcpToolbar}>
                      <p className={styles.settingsHelp}>服务器状态和 OAuth 直接来自 Mahayana/Codex Runtime。</p>
                      <button type="button" onClick={() => void run(() => execute({ type: "mcp.refresh", requestId: nextRequestId("mcp-refresh") }))}>刷新服务器</button>
                    </div>
                    <div className={styles.mcpServerList}>
                      {mcpServers.map((server, index) => {
                        const name = mcpServerName(server);
                        const status = mcpServerStatus(server);
                        const tools = mcpServerTools(server);
                        const connected = /connected|ready|authenticated|ok/i.test(status);
                        return (
                          <article key={`${name}-${index}`} data-state={status}>
                            <span className={styles.mcpStatusDot} />
                            <div><strong>{name}</strong><small>{status} · {tools.length} tools</small>{tools.length ? <p>{tools.slice(0, 8).join(" · ")}</p> : null}</div>
                            <div>
                              <button type="button" onClick={() => { setMcpToolServer(name); setMcpToolName(tools[0] ?? ""); }}>使用</button>
                              <button type="button" onClick={() => void run(() => execute({ type: connected ? "mcp.oauthLogout" : "mcp.oauthLogin", requestId: nextRequestId("mcp-oauth"), server: name }))}>{connected ? "退出授权" : "OAuth 登录"}</button>
                            </div>
                          </article>
                        );
                      })}
                      {!mcpServers.length ? <p className={styles.settingsEmpty}>当前 Runtime 没有报告 MCP Server。</p> : null}
                    </div>
                  </SettingsGroup>

                  <SettingsGroup title="MCP Apps" description="Connector directory">
                    <div className={styles.mcpAppsList}>
                      {mcpApps.map((app, index) => {
                        const record = objectValue(app);
                        const name = String(record.name ?? record.title ?? record.id ?? `App ${index + 1}`);
                        const detail = String(record.description ?? record.status ?? record.id ?? "MCP App");
                        return <article key={`${name}-${index}`}><span>{name.slice(0, 1)}</span><div><strong>{name}</strong><small>{detail}</small></div></article>;
                      })}
                      {!mcpApps.length ? <p className={styles.settingsEmpty}>当前账户没有可用的 MCP Apps。</p> : null}
                    </div>
                  </SettingsGroup>

                  <SettingsGroup title="直接工具调用" description="Direct MCP tool call">
                    <div className={styles.mcpToolForm}>
                      <label><span>Server</span><input value={mcpToolServer} onChange={(event) => setMcpToolServer(event.target.value)} placeholder="github" /></label>
                      <label><span>Tool</span><input value={mcpToolName} onChange={(event) => setMcpToolName(event.target.value)} placeholder="search_repositories" /></label>
                      <label><span>Arguments (JSON)</span><textarea value={mcpToolArguments} onChange={(event) => setMcpToolArguments(event.target.value)} rows={4} /></label>
                      <button type="button" disabled={!mcpToolServer.trim() || !mcpToolName.trim()} onClick={() => void callMcpTool()}>调用工具</button>
                      {mcpToolResult ? <pre>{mcpToolResult}</pre> : null}
                    </div>
                  </SettingsGroup>
                </div>
              ) : settingsSection === "usage" ? (
                <div className={styles.settingsSections}>
                  <SettingsGroup title="当前用量" description="Usage & Billing">
                    <div className={styles.usageSummary}>
                      <strong>{usage ? usage.totalTokens.toLocaleString() : "0"}</strong>
                      <span>本次会话 tokens</span>
                      <i><b style={{ width: usage?.contextWindow ? `${Math.min(100, usage.totalTokens / usage.contextWindow * 100)}%` : "0%" }} /></i>
                      <small>{usage?.model || selectedModel} · {usage?.contextWindow ? `${usage.contextWindow.toLocaleString()} 上下文窗口` : "等待首次模型调用"}</small>
                    </div>
                  </SettingsGroup>
                  <SettingsGroup title="按需用量" description="On-demand usage">
                    <p className={styles.settingsHelp}>当前运行时由本地大乘 CLI 管理模型账户与配额；这里显示实际运行事件返回的用量。</p>
                  </SettingsGroup>
                </div>
              ) : (
                <div className={styles.settingsSections}>
                  <SettingsGroup title="全球法布施更新" description="Updates">
                    <div className={styles.updateCard} data-state={updateState.type}>
                      <span className={styles.updateIcon}>{updateState.type === "error" ? "!" : updateState.type === "checking" || updateState.type === "downloading" || updateState.type === "staging" ? "↻" : "✓"}</span>
                      <div><strong>{updateCopy.title}</strong><small>{updateCopy.detail}</small></div>
                      {updateCopy.action ? (
                        <button
                          type="button"
                          onClick={() => void run(() => execute({
                            type: updateCopy.action === "check" ? "update.check" : "update.install",
                            requestId: nextRequestId(`update-${updateCopy.action}`),
                          }))}
                        >
                          {updateCopy.action === "check" ? "检查更新" : updateState.type === "ready" ? "重新安装" : "下载并安装"}
                        </button>
                      ) : null}
                    </div>
                    <ToggleRow
                      checked={preferences.autoUpdateWhenIdle}
                      label="空闲时自动更新"
                      description="在不影响进行中任务时自动安装客户端更新。"
                      onChange={(value) => setPreference("autoUpdateWhenIdle", value)}
                    />
                  </SettingsGroup>
                </div>
              )}
            </div>
          </section>
        </div>
      ) : null}

      {approval ? (
        <div className={styles.backdrop}>
          <section className={styles.approvalDialog} role="dialog" aria-modal="true" aria-labelledby="approval-title" tabIndex={-1}>
            <span className={styles.approvalIcon}><Icon name="shield" size={24} /></span>
            <h2 id="approval-title">
              {approval.kind === "command"
                ? "运行命令？"
                : approval.kind === "connectedService"
                  ? "连接外部服务？"
                  : approval.kind === "automation"
                    ? "允许自动化操作？"
                    : "能力审批"}
            </h2>
            <p><strong>{approval.miniAppId}</strong> 请求 {approval.capability}</p>
            {approval.subject ? <pre className={styles.approvalSubject}>{approval.subject}</pre> : null}
            <small>{approval.detail || approval.reason}</small>
            {approval.location ? <span className={styles.approvalLocation}>执行位置：{approval.location}</span> : null}
            {approval.proposedRule ? <div className={styles.approvalRule}><strong>以后自动允许</strong><code>{approval.proposedRule}</code></div> : null}
            <div>
              <button
                data-testid="deny-capability"
                type="button"
                onClick={() =>
                  void run(() =>
                    transport.resolveApproval({
                      approvalId: approval.approvalId,
                      decision: "deny",
                    }),
                  )
                }
              >
                拒绝
              </button>
              <button
                data-testid="approve-capability"
                type="button"
                onClick={() =>
                  void run(() =>
                    transport.resolveApproval({
                      approvalId: approval.approvalId,
                      decision: "allow-once",
                    }),
                  )
                }
              >
                本次允许
              </button>
              <button
                data-testid="approve-capability-session"
                type="button"
                onClick={() =>
                  void run(() =>
                    transport.resolveApproval({
                      approvalId: approval.approvalId,
                      decision: "allow-session",
                    }),
                  )
                }
              >
                本会话始终允许
              </button>
            </div>
          </section>
        </div>
      ) : null}

      {offlineAsrOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setOfflineAsrOpen(false)}>
          <section className={`${styles.nativeUtilityDialog} ${styles.offlineAsrDialog}`} role="dialog" aria-modal="true" aria-labelledby="offline-asr-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div>
                <small>FABUSHI OFFLINE ASR</small>
                <h2 id="offline-asr-title">离线语音转写</h2>
                <p>优先使用 Mahayana runtime 转写工具；本页配置 whisper.cpp 离线兜底。</p>
              </div>
              <button type="button" className={styles.iconButton} aria-label="关闭离线语音转写" onClick={() => setOfflineAsrOpen(false)}><Icon name="close" /></button>
            </header>
            <div className={styles.offlineAsrStatusGrid}>
              <div><span>本地引擎</span><strong>{offlineAsrStatus?.binaryPath ? "已就绪" : "未部署"}</strong></div>
              <div><span>模型</span><strong>{offlineAsrStatus?.modelPath ? "已缓存" : "未下载"}</strong></div>
              <div><span>完整性</span><strong>{offlineAsrStatus?.modelVerified ? "SHA-256 已验证" : "待验证"}</strong></div>
              <div><span>可用状态</span><strong>{offlineAsrStatus?.available ? "离线可用" : "等待组件"}</strong></div>
            </div>
            {offlineAsrStatus?.missing?.length ? <p className={styles.offlineAsrHint}>缺少：{offlineAsrStatus.missing.join(" · ")}</p> : null}
            <div className={styles.offlineAsrConfig}>
              <label><span>模型 HTTPS URL</span><input value={offlineAsrModelUrl} onChange={(event) => setOfflineAsrModelUrl(event.target.value)} placeholder="https://…/model.bin" /></label>
              <label><span>SHA-256</span><input value={offlineAsrSha256} onChange={(event) => setOfflineAsrSha256(event.target.value)} maxLength={64} spellCheck={false} placeholder="64 位十六进制摘要" /></label>
            </div>
            {offlineAsrProgress?.phase === "model-download" ? (
              <div className={styles.offlineAsrProgress}>
                <span>下载模型</span>
                <progress max={offlineAsrProgress.totalBytes ?? 1} value={offlineAsrProgress.downloadedBytes ?? 0} />
                <small>{offlineAsrProgress.downloadedBytes ? `${Math.round(offlineAsrProgress.downloadedBytes / 1024 / 1024)} MiB` : "准备中"}</small>
              </div>
            ) : null}
            {offlineAsrError ? <p className={styles.offlineAsrError}>{offlineAsrError}</p> : null}
            <footer className={styles.cloudRunActions}>
              <button type="button" disabled={offlineAsrBusy} onClick={() => void configureOfflineAsr()}>保存配置</button>
              <button type="button" disabled={offlineAsrBusy || !offlineAsrModelUrl.trim() || !/^[0-9a-fA-F]{64}$/.test(offlineAsrSha256.trim())} onClick={() => void downloadOfflineAsr()}>{offlineAsrBusy ? "处理中…" : "下载并校验模型"}</button>
            </footer>
          </section>
        </div>
      ) : null}

      {cloudAgentInfo ? (
        <div className={styles.backdrop} onMouseDown={() => setCloudAgentInfo(null)}>
          <section className={`${styles.nativeUtilityDialog} ${styles.cloudRunDialog}`} role="dialog" aria-modal="true" aria-labelledby="cloud-run-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div>
                <small>FABUSHI CLOUD RUN</small>
                <h2 id="cloud-run-title">{cloudAgentInfo.name ?? `Cloud run ${cloudAgentInfo.id.slice(-8)}`}</h2>
                <p>{cloudAgentInfo.provider ?? "unknown provider"} · {cloudAgentInfo.status}</p>
              </div>
              <button type="button" className={styles.iconButton} aria-label="关闭 Cloud Run" onClick={() => setCloudAgentInfo(null)}><Icon name="close" /></button>
            </header>
            <div className={styles.cloudRunGrid}>
              <div><span>Run ID</span><strong>{cloudAgentInfo.runId ?? cloudAgentInfo.id}</strong></div>
              <div><span>模型</span><strong>{cloudAgentInfo.model ?? "自动"}</strong></div>
              <div><span>输入 Tokens</span><strong>{cloudAgentInfo.inputTokens ?? 0}</strong></div>
              <div><span>输出 Tokens</span><strong>{cloudAgentInfo.outputTokens ?? 0}</strong></div>
              <div><span>工具调用</span><strong>{cloudAgentInfo.toolCallCount ?? 0}</strong></div>
              <div><span>开始时间</span><strong>{cloudAgentInfo.startedAt ? new Date(cloudAgentInfo.startedAt).toLocaleString() : "—"}</strong></div>
            </div>
            {cloudAgentInfo.errorMessage || cloudAgentFailure ? <p className={styles.cloudRunError}>{cloudAgentInfo.errorMessage ?? cloudAgentFailure}</p> : null}
            <footer className={styles.cloudRunActions}>
              <button type="button" disabled={cloudAgentActionPending} onClick={() => void refreshCloudRun()}>刷新状态</button>
              {!(["finished", "error", "expired"] as string[]).includes(cloudAgentInfo.status) ? <button type="button" disabled={cloudAgentActionPending} onClick={() => void cancelCloudRun()}>取消运行</button> : null}
            </footer>
          </section>
        </div>
      ) : null}

      {feedbackOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setFeedbackOpen(false)}>
          <section className={styles.nativeUtilityDialog} role="dialog" aria-modal="true" aria-labelledby="feedback-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div><small>FABUSHI DESKTOP</small><h2 id="feedback-title">发送反馈</h2><p>反馈保存在本机诊断目录；敏感上下文会在写入前脱敏。</p></div>
              <button type="button" className={styles.iconButton} aria-label="关闭反馈" onClick={() => setFeedbackOpen(false)}><Icon name="close" /></button>
            </header>
            <form onSubmit={(event) => void submitDesktopFeedback(event)}>
              <textarea autoFocus rows={7} maxLength={12000} value={feedbackText} onChange={(event) => { setFeedbackText(event.target.value); if (feedbackState !== "idle") setFeedbackState("idle"); }} placeholder="告诉我们哪里可以更快、更稳或更好用…" />
              <footer>
                <span>{feedbackState === "sent" ? "已保存反馈" : feedbackState === "error" ? "保存失败，请重试" : "不会把密码、Token 或 Cookie 写入诊断上下文"}</span>
                <button type="submit" disabled={!feedbackText.trim() || feedbackState === "sending"}>{feedbackState === "sending" ? "保存中…" : "提交反馈"}</button>
              </footer>
            </form>
          </section>
        </div>
      ) : null}

      {aboutOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setAboutOpen(false)}>
          <section className={styles.nativeUtilityDialog} role="dialog" aria-modal="true" aria-labelledby="about-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div><small>FABUSHI · MAHAYANA</small><h2 id="about-title">关于 Fabushi</h2><p>面向多智能体工作流的原生桌面运行时。</p></div>
              <button type="button" className={styles.iconButton} aria-label="关闭关于" onClick={() => setAboutOpen(false)}><Icon name="close" /></button>
            </header>
            <div className={styles.aboutGrid}>
              <div><span>应用版本</span><strong>{aboutEnvironment?.appVersion ?? "Web"}</strong></div>
              <div><span>平台</span><strong>{aboutEnvironment ? `${aboutEnvironment.platform} · ${aboutEnvironment.arch}` : "Browser"}</strong></div>
              <div><span>Electron</span><strong>{aboutEnvironment?.electronVersion ?? "—"}</strong></div>
              <div><span>界面缩放</span><strong>{Math.round(nativeZoomFactor * 100)}%</strong></div>
              <div><span>运行模式</span><strong>{aboutEnvironment?.packaged ? "Production" : "Development"}</strong></div>
              <div><span>Host</span><strong>Mahayana Feature Host</strong></div>
            </div>
          </section>
        </div>
      ) : null}

      {widgetGalleryOpen ? (
        <div className={styles.backdrop} onMouseDown={() => setWidgetGalleryOpen(false)}>
          <section className={`${styles.nativeUtilityDialog} ${styles.widgetGalleryDialog}`} role="dialog" aria-modal="true" aria-labelledby="widget-gallery-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div><small>DESIGN SYSTEM</small><h2 id="widget-gallery-title">Widget Gallery</h2><p>Fabushi 自己的实时状态组件与 Bot 视觉语义。</p></div>
              <button type="button" className={styles.iconButton} aria-label="关闭组件画廊" onClick={() => setWidgetGalleryOpen(false)}><Icon name="close" /></button>
            </header>
            <div className={styles.widgetGalleryGrid}>
              {(["idle", "listening", "thinking", "working", "happy", "alerting"] as BotMarkState[]).map((state, index) => (
                <article key={state}>
                  <BotMark botId={`gallery-${state}`} state={state} size={62} color={(["violet", "cyan", "blue", "orange", "green", "red"] as BotMarkColor[])[index]} label={`${state} state`} />
                  <strong>{state}</strong>
                  <small>{state === "idle" ? "待命" : state === "listening" ? "监听" : state === "thinking" ? "推理" : state === "working" ? "执行" : state === "happy" ? "完成" : "提醒"}</small>
                </article>
              ))}
            </div>
          </section>
        </div>
      ) : null}

      {onboardingStep < 3 || !authResolved || auth?.loggedIn === false ? (
        <div
          className={styles.loginBackdrop}
          data-testid={onboardingStep < 3 ? "onboarding-gate" : "login-gate"}
        >
          {onboardingStep < 3 ? (
            <section className={styles.onboarding} role="dialog" aria-modal="true" aria-labelledby="onboarding-title">
              <header className={styles.onboardingHeader}>
                <div className={styles.onboardingBrand}><BotMark botId="fabushi-onboarding" state={onboardingStep === 0 ? "waking" : onboardingStep === 1 ? "working" : "listening"} size={30} color="black" /><span>FABUSHI</span></div>
                <div className={styles.onboardingProgress} aria-label={`引导第 ${onboardingStep + 1} 步，共 3 步`}>
                  {[0, 1, 2].map((step) => <i key={step} data-active={step <= onboardingStep} />)}
                </div>
              </header>
              <div className={styles.onboardingStage} data-step={onboardingStep}>
                {onboardingStep === 0 ? (
                  <>
                    <div className={styles.onboardingMarkStage}>
                      <BotMark botId="mahayana-assistant" state="waking" shape="blob" color="black" size={112} followPointer emphasis className={styles.onboardingBotMark} label="Fabushi 助手" />
                      <span aria-hidden="true" />
                    </div>
                    <p className={styles.onboardingEyebrow}>YOUR AGENT WORKSPACE</p>
                    <h2 id="onboarding-title">不是聊天窗口。<br />是一支会继续工作的团队。</h2>
                    <p className={styles.onboardingLead}>把目标交给 Fabushi。智能体会拆解任务、调用工具、持续执行，并在需要你决定时回来找你。</p>
                    <div className={styles.onboardingPrompt}><span>“整理今天的客户反馈，找出最值得修的三个问题”</span><i>＋</i><b>↑</b></div>
                  </>
                ) : null}
                {onboardingStep === 1 ? (
                  <>
                    <p className={styles.onboardingEyebrow}>SPECIALIZED AGENTS</p>
                    <h2 id="onboarding-title">每个 Bot 都有自己的工作、记忆和节奏。</h2>
                    <p className={styles.onboardingLead}>不用把所有上下文塞给一个助手。让不同智能体长期负责不同领域，它们会在同一个工作空间协作。</p>
                    <div className={styles.onboardingBots}>
                      <article>
                        <BotMark botId="onboarding-billing" state="working" color="red" size={74} label="账单跟进 Bot" />
                        <div><b>账单跟进</b><small>核对 · 催办 · 汇总</small></div><em>执行中</em>
                      </article>
                      <article>
                        <BotMark botId="onboarding-standup" state="listening" color="cyan" size={74} label="团队站会 Bot" />
                        <div><b>团队站会</b><small>收集 · 提炼 · 跟进</small></div><em>监听中</em>
                      </article>
                      <article>
                        <BotMark botId="onboarding-forecast" state="thinking" color="blue" size={74} label="销售预测 Bot" />
                        <div><b>销售预测</b><small>研究 · 推理 · 更新</small></div><em>思考中</em>
                      </article>
                    </div>
                  </>
                ) : null}
                {onboardingStep === 2 ? (
                  <>
                    <p className={styles.onboardingEyebrow}>CONNECT YOUR WORK</p>
                    <h2 id="onboarding-title">工具留在原位，Fabushi 去那里工作。</h2>
                    <p className={styles.onboardingLead}>连接器可以稍后配置。这里先告诉 Fabushi 你每天在哪些系统里工作。</p>
                    <label className={styles.onboardingSearch}><Icon name="search" size={15} /><input aria-label="搜索工作工具" placeholder="搜索工具" /></label>
                    <div className={styles.onboardingTools}>
                      {["Workspace", "Slack", "Notion", "Salesforce", "Microsoft 365", "LinkedIn", "Zoom", "GitHub", "Jira", "Figma", "HubSpot", "Canva"].map((tool, index) => <button type="button" key={tool}><span data-tone={index % 4}>{tool.slice(0, 1)}</span><b>{tool}</b><small>稍后连接</small></button>)}
                    </div>
                  </>
                ) : null}
              </div>
              <div className={styles.onboardingNav}>
                {onboardingStep ? <button data-testid="onboarding-back" className={styles.onboardingBack} type="button" onClick={() => setOnboardingStep((step) => Math.max(0, step - 1))}>返回</button> : <span />}
                <button
                  data-testid="onboarding-next"
                  className={styles.onboardingNext}
                  type="button"
                  onClick={() => setOnboardingStep((step) => {
                    const nextStep = Math.min(3, step + 1);
                    if (nextStep === 3) {
                      window.localStorage.setItem(ONBOARDING_COMPLETE_KEY, "1");
                      rememberNativeOnboarding();
                    }
                    return nextStep;
                  })}
                >
                  <span>{onboardingStep === 2 ? "开始使用 Fabushi" : "继续"}</span><b>→</b>
                </button>
              </div>
            </section>
          ) : !authResolved ? (
            <section className={`${styles.fabushiWelcome} ${styles.loginExperience}`} role="status" aria-live="polite">
              <BotMark botId="fabushi-account" state="waking" size={96} className={styles.loginHeroMark} paused={false} />
              <p className={styles.loginEyebrow}>FABUSHI ACCOUNT</p>
              <h2>正在验证账号</h2>
              <p>界面不会等待工作空间同步；这里只确认本机安全会话是否仍有效。</p>
              <div className={styles.loginLoadingRail}><i /><i /><i /></div>
            </section>
          ) : browserLoginAttempt ? (
            <section className={`${styles.fabushiWelcome} ${styles.loginExperience} ${styles.browserLoginWaiting}`} role="dialog" aria-modal="true" aria-labelledby="browser-login-title">
              <div className={styles.loginMarkStage}>
                <BotMark botId="fabushi-account" state="orbit" size={108} followPointer emphasis className={styles.loginHeroMark} label="Fabushi 登录助手" />
                <span className={styles.loginOrbitDot} aria-hidden="true" />
              </div>
              <p className={styles.loginEyebrow}>SECURE BROWSER LOGIN</p>
              <h2 id="browser-login-title">在浏览器完成登录</h2>
              <p>登录方式和密码都只在 Fabushi Account Portal 中处理。完成后会自动回到这里。</p>
              <div className={styles.browserLoginSteps} aria-label="登录进度">
                <span data-active="true"><i />已创建一次性会话</span>
                <span data-active="true"><i />等待浏览器授权</span>
                <span><i />安全领取会话</span>
              </div>
              {loginError ? <output className={styles.loginInlineStatus} role="status">{loginError}</output> : null}
              <div className={styles.browserLoginActions}>
                <button data-testid="browser-login-reopen" type="button" onClick={() => void reopenBrowserLogin()}>重新打开浏览器</button>
                <button data-testid="browser-login-cancel" type="button" onClick={() => void cancelBrowserLogin()}>取消等待</button>
              </div>
              <small className={styles.loginPrivacyNote}>Deep link 只携带 attempt ID，不包含 access token、refresh token 或密码。</small>
            </section>
          ) : (
            <section className={`${styles.fabushiWelcome} ${styles.loginExperience} ${styles.loginLanding}`} role="dialog" aria-modal="true" aria-labelledby="login-title">
              <div className={styles.loginMarkStage}>
                <BotMark botId="fabushi-account" state="idle" size={118} followPointer emphasis className={styles.loginHeroMark} label="Fabushi" />
                <span className={styles.loginPresenceHalo} aria-hidden="true" />
              </div>
              <p className={styles.loginEyebrow}>WELCOME TO FABUSHI</p>
              <h2 id="login-title">一支真正会继续工作的智能体团队。</h2>
              <p>桌面只负责发起安全会话。Google、Microsoft、GitHub 和 Fabushi 账号等登录方式统一在浏览器中选择。</p>
              <div className={styles.loginTrustRow}>
                <span>PKCE</span><span>一次性会话</span><span>本机安全存储</span>
              </div>
              <button className={styles.browserLoginPrimary} data-testid="browser-login-start" type="button" disabled={loginBusy} onClick={() => void beginBrowserLogin()}>
                <span>{loginBusy ? "正在打开浏览器…" : "在浏览器中登录"}</span><b>↗</b>
              </button>
              {loginError ? <output className={styles.loginInlineStatus} role="alert">{loginError}</output> : null}
              <small className={styles.loginPrivacyNote}>Fabushi 桌面不会显示或接收第三方登录表单中的密码。</small>
            </section>
          )}
        </div>
      ) : null}

      {accountOpen && auth?.loggedIn ? (
        <div className={styles.backdrop} onMouseDown={() => setAccountOpen(false)}>
          <section className={styles.accountDialog} role="dialog" aria-modal="true" aria-labelledby="account-title" onMouseDown={(event) => event.stopPropagation()}>
            <header>
              <div className={styles.accountAvatarStage}>
                {accountAvatar ? <img src={accountAvatar} alt="" /> : <BotMark botId={`account-${auth.user?.id ?? displayName}`} state="happy" size={88} color="violet" followPointer emphasis label={displayName} />}
                <i aria-hidden="true" />
              </div>
              <div className={styles.accountIdentity}>
                <p>FABUSHI ACCOUNT</p>
                <h2 id="account-title">{displayName}</h2>
                <span>{auth.user?.email || auth.user?.username || "已登录"}</span>
              </div>
              <button className={styles.iconButton} type="button" aria-label="关闭账户" onClick={() => setAccountOpen(false)}><Icon name="close" /></button>
            </header>
            <div className={styles.accountSessionGrid}>
              <article><span>登录方式</span><strong>{accountProvider}</strong><small>Browser Account Portal</small></article>
              <article><span>会话</span><strong>本机安全存储</strong><small>renderer 不持有 refresh token</small></article>
              <article><span>工作空间</span><strong>已同步</strong><small>账户资料与协作状态</small></article>
            </div>
            <div className={styles.accountSecurityNote}><i /><span>登录授权发生在系统浏览器；Fabushi 桌面只领取一次性会话结果。</span></div>
            <footer>
              <button type="button" onClick={() => { setAccountOpen(false); setSettingsOpen(true); setSettingsSection("general"); }}>账户设置</button>
              <button className={styles.accountLogout} data-testid="logout" type="button" onClick={() => void logout()}>退出登录</button>
            </footer>
          </section>
        </div>
      ) : null}

      {confirmAction ? (
        <div className={`${styles.backdrop} ${styles.confirmBackdrop}`} onMouseDown={() => { if (!confirmBusy) setConfirmAction(null); }}>
          <section
            className={styles.confirmDialog}
            data-tone={confirmAction.tone}
            data-testid="confirm-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="confirm-action-title"
            tabIndex={-1}
            onMouseDown={(event) => event.stopPropagation()}
          >
            <div className={styles.confirmMarkStage}>
              <BotMark
                botId={`confirm-${confirmAction.tone}`}
                state={confirmAction.tone === "danger" ? "alerting" : "thinking"}
                size={82}
                color={confirmAction.tone === "danger" ? "red" : "violet"}
                emphasis
                label={confirmAction.tone === "danger" ? "危险操作确认" : "操作确认"}
              />
              <i aria-hidden="true" />
            </div>
            <p className={styles.confirmEyebrow}>CONFIRM ACTION</p>
            <h2 id="confirm-action-title">{confirmAction.title}</h2>
            <p>{confirmAction.message}</p>
            <footer>
              <button type="button" disabled={confirmBusy} onClick={() => setConfirmAction(null)}>返回</button>
              <button
                type="button"
                data-tone={confirmAction.tone}
                disabled={confirmBusy}
                onClick={() => void executeConfirmedAction()}
              >
                {confirmBusy ? "处理中…" : confirmAction.confirmLabel}
              </button>
            </footer>
          </section>
        </div>
      ) : null}
    </main>
  );
}

function SettingsGroup({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <section className={styles.settingsGroup}>
      <header><div><h3>{title}</h3><small>{description}</small></div></header>
      <div>{children}</div>
    </section>
  );
}

function ToggleRow({
  checked,
  label,
  description,
  onChange,
}: {
  checked: boolean;
  label: string;
  description: string;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className={styles.settingsRow}>
      <span><strong>{label}</strong><small>{description}</small></span>
      <input className={styles.switchInput} type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
    </label>
  );
}
