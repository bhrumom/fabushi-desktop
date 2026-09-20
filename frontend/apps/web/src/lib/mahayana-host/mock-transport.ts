import type {
  AgentPeerMessage,
  ApprovalResolution,
  AuthState,
  BrowserLoginAttempt,
  BrowserLoginPollResult,
  BrowserLoginReopenResult,
  AuthProvider,
  AuthProviderId,
  AutomationSummary,
  BotSummary,
  CommandAccepted,
  ConnectorSummary,
  GroupSummary,
  HostConfig,
  HostInfo,
  HostStatus,
  ListenerIntegrationSummary,
  ListenerPlatform,
  MemoryRecord,
  MessageDraft,
  ProductHostSettings,
  RuntimeCommand,
  RuntimeEvent,
  OAuthAttempt,
  OAuthPollResult,
  SkillSummary,
  SkillTeamSummary,
  TranscriptCard,
  UpdateState,
  WorkflowSummary,
} from "./contracts";
import {
  ElectronMahayanaHostTransport,
  isElectronMahayanaHostAvailable,
} from "./electron-transport";
import type {
  InstalledPluginList,
  InstalledPluginPointer,
  MahayanaHostTransport,
  MarketplaceBrowseResult,
  MarketplaceReleaseMetadata,
  PluginUiDocument,
  PluginUninstallResult,
  RuntimeEventListener,
} from "./transport";
import { makeMemoryId, memoryDedupeKey, normalizeMemoryContent } from "../fabushi-runtime/memory-store";
import { ErrorTrayQueue } from "../fabushi-runtime/error-trays";
import { getMarketplaceRelease } from "../marketplace";

const now = () => new Date().toISOString();
const mockComputerSnapshot = () => ({
  capturedAtMs: Date.now(),
  dataUrl: "data:image/svg+xml;charset=utf-8," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="800"><rect width="100%" height="100%" fill="#15171a"/><text x="50%" y="50%" fill="#c8ccd4" font-family="system-ui" font-size="34" text-anchor="middle">This Computer · Mock Desktop</text></svg>'),
  width: 1280,
  height: 800,
});

const connectorTool = (id: string, name: string, description: string) => ({
  id,
  name,
  description,
  enabled: true,
  requiresApproval: true,
});

const defaultConnectors = (): ConnectorSummary[] => [
  {
    id: "github",
    displayName: "GitHub",
    description: "Repositories, pull requests, issues, comments and CI.",
    status: "disconnected",
    isTeam: false,
    canAddAccount: true,
    transport: "http",
    source: "Built in",
    accounts: [],
    tools: [
      connectorTool("read_repository", "Read repository", "Read repository files and metadata."),
      connectorTool("create_issue", "Create issue", "Create and update GitHub issues."),
      connectorTool("comment_pull_request", "Comment on pull request", "Post review comments on pull requests."),
    ],
  },
  {
    id: "slack",
    displayName: "Slack",
    description: "Messages, mentions, reactions and approved drafts.",
    status: "disconnected",
    isTeam: false,
    canAddAccount: true,
    transport: "http",
    source: "Built in",
    accounts: [],
    tools: [
      connectorTool("search_messages", "Search messages", "Search workspace messages and threads."),
      connectorTool("post_message", "Post message", "Send an approved message or thread reply."),
      connectorTool("add_reaction", "Add reaction", "Add a reaction to a message."),
    ],
  },
  {
    id: "teams",
    displayName: "Microsoft Teams",
    description: "Teams messages, mentions, channels and approved drafts.",
    status: "disconnected",
    isTeam: false,
    canAddAccount: true,
    transport: "http",
    source: "Built in",
    accounts: [],
    tools: [
      connectorTool("search_messages", "Search messages", "Search Teams channels and chats."),
      connectorTool("post_message", "Post message", "Send an approved Teams message."),
    ],
  },
  {
    id: "linear",
    displayName: "Linear",
    description: "Issues, comments, status changes and projects.",
    status: "disconnected",
    isTeam: false,
    canAddAccount: true,
    transport: "http",
    source: "Built in",
    accounts: [],
    tools: [
      connectorTool("read_issues", "Read issues", "Read Linear issues and projects."),
      connectorTool("update_issue", "Update issue", "Update issue state, assignee and fields."),
    ],
  },
  {
    id: "sentry",
    displayName: "Sentry",
    description: "Errors, regressions, releases and issue ownership.",
    status: "disconnected",
    isTeam: false,
    canAddAccount: true,
    transport: "http",
    source: "Built in",
    accounts: [],
    tools: [
      connectorTool("read_issues", "Read issues", "Read Sentry issues and events."),
      connectorTool("resolve_issue", "Resolve issue", "Resolve or assign a Sentry issue."),
    ],
  },
  {
    id: "pagerduty",
    displayName: "PagerDuty",
    description: "Incidents, acknowledgements, responders and escalation.",
    status: "disconnected",
    isTeam: false,
    canAddAccount: true,
    transport: "http",
    source: "Built in",
    accounts: [],
    tools: [
      connectorTool("read_incidents", "Read incidents", "Read incident details and timelines."),
      connectorTool("acknowledge_incident", "Acknowledge incident", "Acknowledge an incident after approval."),
    ],
  },
  {
    id: "git",
    displayName: "Git",
    description: "Local commits, branches and repository state.",
    status: "disconnected",
    isTeam: false,
    canAddAccount: true,
    transport: "command",
    source: "Built in",
    accounts: [],
    tools: [
      connectorTool("read_status", "Read status", "Read local repository status."),
      connectorTool("read_history", "Read history", "Read commit and branch history."),
    ],
  },
];

const defaultTeams = (): SkillTeamSummary[] => [
  { id: "team-mahayana", name: "Mahayana Team" },
];

const defaultSkills = (): SkillSummary[] => [
  {
    id: "skill-research-brief",
    name: "Research brief",
    description: "Turn verified sources into a concise research brief.",
    useWhen: "Use when a task needs sourced research and a decision-ready summary.",
    instructions: "Verify sources, distinguish facts from inference, and end with actionable conclusions.",
    source: "private",
    publishState: "local",
    ownerAgentId: "mahayana-assistant",
    readOnly: false,
    updatedAtMs: 0,
  },
  {
    id: "skill-incident-response",
    name: "Incident response",
    description: "Coordinate incident triage across monitoring and communication tools.",
    useWhen: "Use when an alert or incident needs coordinated triage.",
    instructions: "Establish severity, collect evidence, propose actions, and request approval before external changes.",
    source: "team",
    publishState: "managed",
    teamId: "team-mahayana",
    teamName: "Mahayana Team",
    readOnly: true,
    updatedAtMs: 0,
  },
];

const defaultBots = (): BotSummary[] => [
  { id: "mahayana-assistant", name: "大乘助手", description: "General-purpose Mahayana assistant.", title: "", hidden: false, notificationsEnabled: true, notifyOnUpdates: true, unread: false, conversationId: "mahayana-ai:agent:assistant" },
  { id: "research-bot", name: "Research Bot", description: "Source verification and research synthesis.", title: "", hidden: false, notificationsEnabled: true, notifyOnUpdates: true, unread: false, conversationId: "codex:agent:research" },
  { id: "incident-bot", name: "Incident Bot", description: "Incident triage and operational coordination.", title: "", hidden: true, notificationsEnabled: true, notifyOnUpdates: true, unread: false, conversationId: "codex:agent:incident" },
];

const listenerCopy: Record<ListenerPlatform, [string, string]> = {
  github: ["GitHub", "Let automations watch a repo's PRs, comments, issues, and CI."],
  git: ["Git", "Wake automations on local commits, branches, tags, and repository changes."],
  slack: ["Slack", "Wake automations on Slack messages, mentions, and reactions."],
  teams: ["Microsoft Teams", "Wake automations on Teams messages, mentions, and reactions."],
  linear: ["Linear", "Wake automations on issues, comments, status changes, and assignments."],
  sentry: ["Sentry", "Wake automations on new, regressed, assigned, and resolved issues."],
  pagerduty: ["PagerDuty", "Wake automations when incidents are triggered, acknowledged, escalated, or resolved."],
};

const defaultListeners = (): ListenerIntegrationSummary[] =>
  (Object.keys(listenerCopy) as ListenerPlatform[]).map((platform) => ({
    platform,
    displayName: listenerCopy[platform][0],
    blurb: listenerCopy[platform][1],
    isConnected: false,
  }));

const connectorForPlatform = (platform: ListenerPlatform): string => platform;

const validateDraft = (draft: MessageDraft): void => {
  if (draft.kind === "email") {
    if (
      draft.to.length === 0 ||
      draft.to.some((recipient) => !recipient.trim() || !recipient.includes("@"))
    ) {
      throw new Error("Email draft requires valid recipients");
    }
    if (!draft.subject.trim()) throw new Error("Email subject must not be empty");
    if (!draft.body.trim()) throw new Error("Email body must not be empty");
    return;
  }
  if (!draft.target.trim()) throw new Error("Slack target must not be empty");
  if (!draft.body.trim()) throw new Error("Slack body must not be empty");
};

const RICH_RESULTS_CONVERSATION_ID = "mock:rich-results";

const richResultsMessages = () => {
  const createdAtMs = Date.now();
  return [
    {
      id: "rich-results-user",
      role: "user" as const,
      text: "Show the complete rich-result and approval-card gallery.",
      createdAtMs: createdAtMs - 1_000,
    },
    {
      id: "rich-results-assistant",
      role: "assistant" as const,
      createdAtMs,
      text: [
        "# Rich result gallery",
        "",
        "This browser-only conversation exercises **Markdown**, `inline code`, tables, code, diffs, and Mermaid without changing the native Tauri runtime.",
        "",
        "> The cards below use the same Host protocol consumed by production.",
        "",
        "| Surface | State |",
        "| --- | --- |",
        "| Markdown | Ready |",
        "| Approval cards | Interactive |",
        "| Documents | Previewable |",
        "",
        "- [x] Event routines",
        "- [x] Draft approvals",
        "- [ ] External OAuth (requires a real account)",
        "",
        "```typescript",
        "interface RoutineEvent {",
        "  source: \"slack\" | \"github\" | \"teams\";",
        "  event: string;",
        "}",
        "",
        "export const wake = async (event: RoutineEvent) => event;",
        "```",
        "",
        "```diff",
        "@@ listener state @@",
        "- status: disconnected",
        "+ status: listening",
        "+ neededByCount: 2",
        "```",
        "",
        "```mermaid",
        "flowchart TD",
        "  Event[External event] --> Match{Listener matches?}",
        "  Match -->|yes| Wake[Wake routine]",
        "  Match -->|no| Ignore[Ignore event]",
        "```",
        "",
        "```mermaid",
        "sequenceDiagram",
        "  participant Source",
        "  participant Host",
        "  participant Agent",
        "  Source->>Host: event payload",
        "  Host-->>Agent: structured wake context",
        "```",
      ].join("\n"),
    },
  ];
};

const demoPdfDataBase64 = (): string => {
  const stream = "BT /F1 18 Tf 72 720 Td (Mahayana rich result preview) Tj ET\n";
  const objects = [
    "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
    "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n",
    "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
    `4 0 obj\n<< /Length ${stream.length} >>\nstream\n${stream}endstream\nendobj\n`,
    "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
  ];
  let pdf = "%PDF-1.4\n";
  const offsets = [0];
  for (const object of objects) {
    offsets.push(pdf.length);
    pdf += object;
  }
  const xrefOffset = pdf.length;
  pdf += `xref\n0 ${objects.length + 1}\n`;
  pdf += "0000000000 65535 f \n";
  for (const offset of offsets.slice(1)) {
    pdf += `${String(offset).padStart(10, "0")} 00000 n \n`;
  }
  pdf += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xrefOffset}\n%%EOF`;
  return btoa(pdf);
};

const richResultsCards = (): TranscriptCard[] => {
  const occurredAtMs = Date.now();
  const events: Array<Extract<TranscriptCard, { kind: "event" }>> = [
    {
      kind: "event",
      event: {
        source: "slack",
        event: "mention",
        title: "Agent mentioned in #operations",
        summary: "A production readiness question matched the routine keyword filter.",
        actor: "@dana",
        fields: [{ label: "Channel", value: "#operations" }, { label: "Keyword", value: "release" }],
        occurredAtMs,
      },
    },
    {
      kind: "event",
      event: {
        source: "github",
        event: "pullRequestReviewRequested",
        title: "Review requested on PR #482",
        summary: "The repository listener matched the review-request event.",
        actor: "octocat",
        fields: [{ label: "Repository", value: "ombhrum/fabushi" }, { label: "Branch", value: "main" }],
        occurredAtMs,
      },
    },
    {
      kind: "event",
      event: {
        source: "git",
        event: "commit",
        title: "Local branch advanced",
        summary: "A local commit woke the repository maintenance routine.",
        fields: [{ label: "Branch", value: "codex/fix-mahayana-marketplace-helper" }],
        occurredAtMs,
      },
    },
    {
      kind: "event",
      event: {
        source: "teams",
        event: "message",
        title: "Teams deployment thread updated",
        summary: "A message in the release channel matched the configured filter.",
        actor: "Avery",
        fields: [{ label: "Team", value: "Engineering" }, { label: "Channel", value: "Releases" }],
        occurredAtMs,
      },
    },
    {
      kind: "event",
      event: {
        source: "linear",
        event: "statusChanged",
        title: "FAB-321 moved to In Progress",
        summary: "The Linear issue entered a watched status.",
        actor: "Morgan",
        fields: [{ label: "Project", value: "Desktop" }, { label: "Team", value: "Runtime" }],
        occurredAtMs,
      },
    },
    {
      kind: "event",
      event: {
        source: "sentry",
        event: "issueRegressed",
        title: "Renderer crash regressed",
        summary: "Sentry reported a regression in the signed desktop build.",
        fields: [{ label: "Project", value: "fabushi-desktop" }, { label: "Issue", value: "DESKTOP-91" }],
        occurredAtMs,
      },
    },
    {
      kind: "event",
      event: {
        source: "pagerduty",
        event: "incidentTriggered",
        title: "API latency incident triggered",
        summary: "PagerDuty woke the incident response routine.",
        fields: [{ label: "Service", value: "Mahayana API" }, { label: "Urgency", value: "high" }],
        occurredAtMs,
      },
    },
  ];

  return [
    ...events,
    {
      kind: "emailDraft",
      draft: {
        kind: "email",
        id: "demo-email-draft",
        from: "agent@example.com",
        to: ["release@example.com"],
        cc: ["ops@example.com"],
        subject: "Release readiness update",
        body: "The migration checks passed. Please review the attached rollout summary before approval.",
        status: "editable",
      },
    },
    {
      kind: "slackDraft",
      draft: {
        kind: "slack",
        id: "demo-slack-draft",
        workspace: "Mahayana",
        target: "#release-room",
        thread: "Release 0.16 migration",
        body: "Host protocol, rich cards, and event routines are ready for final verification.",
        status: "editable",
      },
    },
    {
      kind: "secretRequest",
      requestId: "demo-secret-request",
      label: "Deployment API token",
      description: "Provide the token needed by the deployment connector. The value is never echoed into the transcript.",
      provided: false,
    },
    {
      kind: "listenerConnect",
      platform: "slack",
      reason: "Connect Slack before enabling routines that depend on mentions or reactions.",
      connected: false,
      pending: false,
    },
    {
      kind: "pdf",
      name: "migration-summary.pdf",
      dataBase64: demoPdfDataBase64(),
      pageCount: 1,
    },
    {
      kind: "spreadsheet",
      name: "migration-checklist.xlsx",
      sheets: [
        {
          name: "Coverage",
          rows: [
            ["Surface", "Status", "Evidence"],
            ["Event routines", "Ready", "7 listener sources"],
            ["Draft approvals", "Ready", "Email + Slack"],
            ["Updater", "Blocked", "Signed endpoint required"],
          ],
        },
        {
          name: "Responsive",
          rows: [
            ["Viewport", "Layout"],
            ["Desktop", "Three-column shell"],
            ["Tablet", "Activity panel hidden"],
            ["Mobile", "Compact sidebar and stacked cards"],
          ],
        },
      ],
    },
  ];
};

/**
 * Deterministic browser transport used by fast UI tests.
 *
 * The historical class name is retained so existing pages need no migration
 * churn. Inside a native Electron or Tauri window it automatically delegates
 * every method to the real Rust feature Host; only an ordinary browser uses
 * the in-memory implementation below.
 */
export class MockMahayanaHostTransport implements MahayanaHostTransport {
  private readonly native: MahayanaHostTransport | null;
  private readonly listeners = new Set<RuntimeEventListener>();
  private readonly installedPlugins = new Map<string, InstalledPluginPointer>();
  private readonly approvals = new Set<string>();
  private readonly providedSecrets = new Set<string>();
  private status: HostStatus = "idle";
  private sequence = 0;
  private auth: AuthState = { loggedIn: false, provider: "test" };
  private oauthAttempt: OAuthAttempt | null = null;
  private browserLoginAttempt: BrowserLoginAttempt | null = null;
  private automations = new Map<string, AutomationSummary>();
  private connectors = new Map(defaultConnectors().map((connector) => [connector.id, connector]));
  private skills = new Map(defaultSkills().map((skill) => [skill.id, skill]));
  private bots = new Map(defaultBots().map((bot) => [bot.id, bot]));
  private groups = new Map<string, GroupSummary>();
  private peerMessages: AgentPeerMessage[] = [];
  private memories = new Map<string, MemoryRecord[]>();
  private trays = new ErrorTrayQueue();
  private workflows = new Map<string, WorkflowSummary>();
  private disabledWorkflows = new Map<string, Set<string>>();
  private attachmentData = new Map<string, { agentId: string; name: string; mimeType?: string; bytesBase64: string }>();
  private teachRecording: { agentId: string; startedAtMs: number } | null = null;
  private hostSettings: ProductHostSettings = {
    notifications: true,
    autoUpdateWhenIdle: true,
    localExecution: true,
    routeEgressLocally: false,
    securityKeys: false,
    webauthnProxyEnabled: false,
    localToolPermission: "ask",
    remoteControlEnabled: false,
    aiComputerControlEnabled: true,
    autoReviewRules: [],
    inferenceProvider: "fabushi",
    sandboxRuntime: "host",
  };
  private listenerIntegrations = new Map(defaultListeners().map((integration) => [integration.platform, integration]));
  private updateState: UpdateState = { type: "upToDate", version: "0.1.0" };
  private info: HostInfo = {
    runtimeVersion: "mahayana-mock-1.0.0",
    protocolVersion: "1",
    platform: "mock",
  };

  constructor(options: { authenticated?: boolean } = {}) {
    this.native = isElectronMahayanaHostAvailable()
      ? new ElectronMahayanaHostTransport()
      : null;
    if (!this.native && options.authenticated) {
      this.auth = {
        loggedIn: true,
        provider: "test",
        user: {
          id: "copyright-screenshot",
          username: "copyright-local",
          nickname: "本地测试用户",
        },
      };
    }
  }

  subscribe(listener: RuntimeEventListener): () => void {
    if (this.native) return this.native.subscribe(listener);
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  async initialize(config: HostConfig): Promise<HostInfo> {
    if (this.native) return this.native.initialize(config);
    if (this.status === "ready") return this.info;
    this.status = "initializing";
    this.status = "ready";
    this.emit({ type: "host.ready", timestamp: now(), info: this.info });
    return this.info;
  }

  async marketplaceBrowse(query?: string): Promise<MarketplaceBrowseResult> {
    if (this.native) return this.native.marketplaceBrowse(query);
    const term = query?.trim().toLocaleLowerCase() ?? "";
    const plugins = [
      ["global-dharma", "全球法布施", "任务、日志与部署"],
      ["faliu-flashcards", "法流记忆卡", "经文牌组与复习"],
      ["platform-publish", "平台发布", "内容发布与自动化"],
      ["bot-father", "Bot Father", "创建和管理机器人"],
    ].map(([pluginId, displayName, description]) => ({
      pluginId, displayName, description, latestVersion: "1.0.0",
      platforms: ["desktop"], releaseStatus: "approved",
    })).filter((plugin) => !term || `${plugin.pluginId} ${plugin.displayName} ${plugin.description}`.toLocaleLowerCase().includes(term));
    return { plugins };
  }

  async marketplaceRelease(pluginId: string, version: string): Promise<MarketplaceReleaseMetadata> {
    if (this.native) return this.native.marketplaceRelease(pluginId, version);
    const published = getMarketplaceRelease(pluginId);
    if (!published) throw new Error(`published marketplace release is missing for ${pluginId}`);
    const artifact = {
      id: published.artifactId,
      runtime: published.runtime,
      platforms: [...published.platforms],
      source: { type: "https", url: published.artifactUrl },
      sha256: published.artifactSha256,
      size: published.artifactSize,
      format: published.format,
    };
    return {
      pluginId,
      version: published.version,
      releaseStatus: "approved",
      releaseManifest: {
        schemaVersion: 1,
        protocol: "mahayana.external-release.v1",
        pluginId,
        version: published.version,
        source: {
          repository: published.repository,
          sourceRef: published.sourceRef,
        },
        permissions: [],
        artifacts: [artifact],
      },
      install: {
        protocol: published.protocol,
        strategy: "github-immutable",
        pluginId,
        version: published.version,
        source: {
          repository: published.repository,
          sourceRef: published.sourceRef,
          ...(published.manifestUrl ? { manifestUrl: published.manifestUrl } : {}),
          marketplaceHostsPackage: false,
        },
        artifacts: [artifact],
        update: {
          check: "marketplace-release",
          comparison: "version-then-artifact-sha256",
          allowDowngrade: false,
          rollback: "previous-active",
        },
      },
    };
  }

  async pluginInstall(release: Record<string, unknown>, platform = "desktop"): Promise<InstalledPluginPointer> {
    if (this.native) return this.native.pluginInstall(release, platform);
    const pluginId = String(release.pluginId ?? "");
    const version = String(release.version ?? "1.0.0");
    if (!pluginId) throw new Error("release pluginId is required");
    const pointer: InstalledPluginPointer = {
      pluginId, version, artifactId: "mock-ui", artifactSha256: "0".repeat(64),
      runtime: "local-web", entry: "index.html", installedPath: `/mock/plugins/${pluginId}/${version}`, requestedPermissions: [],
    };
    this.installedPlugins.set(pluginId, pointer);
    return pointer;
  }

  async pluginUninstall(pluginId: string): Promise<PluginUninstallResult> {
    if (this.native) return this.native.pluginUninstall(pluginId);
    return { pluginId, removed: this.installedPlugins.delete(pluginId), permissionsRemoved: true };
  }

  async pluginRollback(pluginId: string): Promise<InstalledPluginPointer | null> {
    if (this.native) return this.native.pluginRollback(pluginId);
    return this.installedPlugins.get(pluginId) ?? null;
  }

  async pluginActive(pluginId: string): Promise<InstalledPluginPointer | null> {
    if (this.native) return this.native.pluginActive(pluginId);
    return this.installedPlugins.get(pluginId) ?? null;
  }

  async pluginListInstalled(): Promise<InstalledPluginList> {
    if (this.native) return this.native.pluginListInstalled();
    return { plugins: [...this.installedPlugins.values()] };
  }

  async pluginUiDocument(pluginId: string): Promise<PluginUiDocument> {
    if (this.native) return this.native.pluginUiDocument(pluginId);
    if (!this.installedPlugins.has(pluginId)) throw new Error(`plugin ${pluginId} is not installed`);
    return { pluginId, html: `<!doctype html><meta charset="utf-8"><title>${pluginId}</title><main style="font-family:system-ui;padding:32px"><h1>${pluginId}</h1><p>Installed from the online Mahayana Marketplace.</p></main>` };
  }

  async execute(command: RuntimeCommand): Promise<CommandAccepted> {
    if (this.native) return this.native.execute(command);
    this.assertReady();

    switch (command.type) {
      case "conversation.list":
        this.emit({
          type: "conversation.listed",
          timestamp: now(),
          conversations: [
            {
              id: "mahayana-ai:agent:assistant",
              title: "大乘助手",
              kind: "codex",
              pinned: true,
              unreadCount: 0,
              updatedAtMs: Date.now(),
            },
            {
              id: RICH_RESULTS_CONVERSATION_ID,
              title: "富结果验收",
              kind: "mock gallery",
              pinned: false,
              unreadCount: 0,
              updatedAtMs: Date.now() - 1_000,
            },
          ].filter((conversation) =>
            command.query
              ? conversation.title.includes(command.query)
              : true,
          ),
        });
        return { requestId: command.requestId };
      case "conversation.open": {
        const isRichResultsGallery = command.conversationId === RICH_RESULTS_CONVERSATION_ID;
        this.emit({
          type: "conversation.opened",
          timestamp: now(),
          conversationId: command.conversationId,
          messages: isRichResultsGallery ? richResultsMessages() : [],
        });
        if (isRichResultsGallery) {
          richResultsCards()
            .map((card) => card.kind === "secretRequest" && this.providedSecrets.has(card.requestId)
              ? { ...card, provided: true }
              : card)
            .forEach((card, index) => {
              this.emit({
                type: "transcript.card",
                timestamp: now(),
                entryId: `rich-results-card-${index + 1}`,
                card,
              });
            });
        }
        return { requestId: command.requestId };
      }
      case "capability.list":
        this.emit({
          type: "capability.listed",
          timestamp: now(),
          capabilities: [
            {
              id: "agent.mahayana",
              title: "大乘助手",
              kind: "agent" as const,
              mention: "@agent.mahayana",
              conversationId: "mahayana-ai:agent:assistant",
              provider: "codex",
              description: "大乘共享智能代理",
              requiredPermissions: [],
              availability: "ready" as const,
            },
          ].filter((capability) =>
            command.query
              ? `${capability.title} ${capability.id}`.includes(command.query)
              : true,
          ),
        });
        return { requestId: command.requestId };
      case "automation.list":
        this.emit({
          type: "automation.listed",
          timestamp: now(),
          automations: [...this.automations.values()].filter((automation) =>
            command.agentId ? automation.agentId === command.agentId : true,
          ),
        });
        return { requestId: command.requestId };
      case "automation.upsert": {
        const previous = command.id ? this.automations.get(command.id) : undefined;
        if (command.agentId && !this.bots.has(command.agentId)) {
          throw new Error(`Unknown automation agent: ${command.agentId}`);
        }
        if (previous && command.agentId && previous.agentId !== command.agentId) {
          throw new Error(`Automation ${previous.id} does not belong to agent ${command.agentId}`);
        }
        const id = command.id ?? this.nextId("routine");
        const automation = {
          id,
          agentId: command.agentId ?? previous?.agentId,
          name: command.name,
          prompt: command.prompt,
          schedule: command.schedule,
          trigger: command.trigger ?? { kind: "schedule" as const, schedule: command.schedule },
          enabled: command.enabled ?? true,
          createdAtMs: previous?.createdAtMs ?? Date.now(),
          lastRunAtMs: previous?.lastRunAtMs,
          nextRunAtMs: command.trigger?.kind === "event" ? undefined : Date.now() + 60_000,
        };
        this.automations.set(id, automation);
        this.emit({ type: "automation.changed", timestamp: now(), action: previous ? "updated" : "created", automation });
        return { requestId: command.requestId };
      }
      case "automation.setEnabled": {
        const automation = this.automations.get(command.id);
        if (!automation) throw new Error(`Unknown automation: ${command.id}`);
        if (command.agentId && automation.agentId !== command.agentId) {
          throw new Error(`Automation ${automation.id} does not belong to agent ${command.agentId}`);
        }
        const next = {
          ...automation,
          enabled: command.enabled,
          nextRunAtMs:
            command.enabled && automation.trigger?.kind !== "event"
              ? Date.now() + 60_000
              : undefined,
        };
        this.automations.set(command.id, next);
        this.emit({ type: "automation.changed", timestamp: now(), action: command.enabled ? "resumed" : "paused", automation: next });
        return { requestId: command.requestId };
      }
      case "automation.delete": {
        const automation = this.automations.get(command.id);
        if (!automation) throw new Error(`Unknown automation: ${command.id}`);
        if (command.agentId && automation.agentId !== command.agentId) {
          throw new Error(`Automation ${automation.id} does not belong to agent ${command.agentId}`);
        }
        this.automations.delete(command.id);
        this.emit({ type: "automation.changed", timestamp: now(), action: "deleted", automation });
        return { requestId: command.requestId };
      }
      case "automation.run": {
        const automation = this.automations.get(command.id);
        if (!automation) throw new Error(`Unknown automation: ${command.id}`);
        if (command.agentId && automation.agentId !== command.agentId) {
          throw new Error(`Automation ${automation.id} does not belong to agent ${command.agentId}`);
        }
        const next = { ...automation, lastRunAtMs: Date.now() };
        this.automations.set(command.id, next);
        this.emit({ type: "automation.changed", timestamp: now(), action: "running", automation: next });
        if (next.trigger?.kind === "event") {
          this.emit({
            type: "transcript.card",
            timestamp: now(),
            entryId: this.nextId("event"),
            card: {
              kind: "event",
              event: {
                source: next.trigger.source,
                event: next.trigger.event,
                title: `${listenerCopy[next.trigger.source][0]} event`,
                summary: `${next.trigger.event} woke routine “${next.name}”.`,
                fields: next.trigger.filter
                  ? [{ label: "Filter", value: next.trigger.filter }]
                  : undefined,
                occurredAtMs: Date.now(),
              },
            },
          });
        }
        const triggerContext =
          next.trigger?.kind === "event"
            ? `\n触发事件：${listenerCopy[next.trigger.source][0]} / ${next.trigger.event}`
            : "";
        return this.execute({
          type: "chat.send",
          requestId: command.requestId,
          text: `[自动化例程：${next.name}]${triggerContext}\n${next.prompt}`,
        });
      }
      case "chat.send": {
        const operationId = this.nextId("chat");
        this.emit({
          type: "chat.message",
          timestamp: now(),
          role: "user",
          text: command.text,
        });
        this.emit({
          type: "operation.started",
          timestamp: now(),
          operationId,
          label: "chat-response",
          interruptible: true,
        });
        this.emit({
          type: "model.routed",
          timestamp: now(),
          operationId,
          provider: "mahayana-mock",
          model: command.model ?? "auto",
          mode: command.mode ?? "agent",
        });
        this.emit({
          type: "agent.step",
          timestamp: now(),
          operationId,
          stepId: `${operationId}:context`,
          kind: "context",
          title: command.attachments?.length
            ? `读取 ${command.attachments.length} 个附件`
            : "分析请求",
          status: "completed",
          progress: 1,
          total: 1,
        });
        this.emit({
          type: "chat.message",
          timestamp: now(),
          role: "assistant",
          text: `收到：${command.text}`,
          operationId,
        });
        this.emit({
          type: "usage.updated",
          timestamp: now(),
          operationId,
          inputTokens: command.text.length,
          cachedInputTokens: 0,
          outputTokens: 8,
          reasoningTokens: 0,
          totalTokens: command.text.length + 8,
          contextWindow: 128_000,
        });
        this.emit({
          type: "operation.completed",
          timestamp: now(),
          operationId,
        });
        return { requestId: command.requestId, operationId };
      }
      case "marketplace.install":
        this.emit({
          type: "marketplace.installed",
          timestamp: now(),
          miniAppId: command.miniAppId,
          version: "1.0.0",
        });
        return { requestId: command.requestId };
      case "miniapp.open":
        this.emit({
          type: "miniapp.opened",
          timestamp: now(),
          miniAppId: command.miniAppId,
          html: `<!doctype html><html lang="zh-CN"><body style="font:15px system-ui;background:#101722;color:#edf3ff;padding:20px"><h1>测试 MiniApp</h1><p>${command.miniAppId} 已在隔离容器中打开。</p></body></html>`,
        });
        return { requestId: command.requestId };
      case "capability.request": {
        const approvalId = this.nextId("approval");
        this.approvals.add(approvalId);
        this.emit({
          type: "approval.requested",
          timestamp: now(),
          approvalId,
          miniAppId: command.miniAppId,
          capability: command.capability,
          reason: command.reason,
          kind: "capability",
        });
        return { requestId: command.requestId };
      }
      case "connector.list":
        this.emit({
          type: "connector.listed",
          timestamp: now(),
          connectors: [...this.connectors.values()],
        });
        return { requestId: command.requestId };
      case "connector.connect": {
        const connector = this.connectors.get(command.connectorId);
        if (!connector) throw new Error(`Unknown connector: ${command.connectorId}`);
        if (!connector.canAddAccount) throw new Error(`${connector.displayName} cannot add accounts`);
        this.emit({
          type: "connector.oauthRequested",
          timestamp: now(),
          connectorId: connector.id,
          authorizationUrl: `about:blank#fabushi-test-connector-${connector.id}`,
        });
        const accountLabel = command.accountLabel?.trim() || "Personal";
        const next: ConnectorSummary = {
          ...connector,
          status: "connected",
          accounts: [
            ...connector.accounts,
            {
              id: this.nextId("account"),
              label: accountLabel,
              status: "connected",
              teamManaged: false,
            },
          ],
        };
        this.connectors.set(next.id, next);
        this.emit({ type: "connector.changed", timestamp: now(), action: "connected", connector: next });
        const platform = next.id as ListenerPlatform;
        if (this.listenerIntegrations.has(platform)) {
          const integration = {
            ...this.listenerIntegrations.get(platform)!,
            isConnected: true,
            accountLabel,
          };
          this.listenerIntegrations.set(platform, integration);
          this.emit({ type: "listener.changed", timestamp: now(), integration });
        }
        return { requestId: command.requestId };
      }
      case "connector.renameAccount": {
        const connector = this.connectors.get(command.connectorId);
        if (!connector) throw new Error(`Unknown connector: ${command.connectorId}`);
        const account = connector.accounts.find((item) => item.id === command.accountId);
        if (!account) throw new Error(`Unknown connector account: ${command.accountId}`);
        if (account.teamManaged) throw new Error("Team-managed accounts cannot be renamed");
        if (!command.label.trim()) throw new Error("Account label must not be empty");
        const next = {
          ...connector,
          accounts: connector.accounts.map((item) =>
            item.id === command.accountId ? { ...item, label: command.label.trim() } : item,
          ),
        };
        this.connectors.set(next.id, next);
        this.emit({ type: "connector.changed", timestamp: now(), action: "updated", connector: next });
        return { requestId: command.requestId };
      }
      case "connector.removeAccount": {
        const connector = this.connectors.get(command.connectorId);
        if (!connector) throw new Error(`Unknown connector: ${command.connectorId}`);
        const account = connector.accounts.find((item) => item.id === command.accountId);
        if (!account) throw new Error(`Unknown connector account: ${command.accountId}`);
        if (account.teamManaged) throw new Error("Team-managed accounts cannot be removed");
        const accounts = connector.accounts.filter((item) => item.id !== command.accountId);
        const next: ConnectorSummary = {
          ...connector,
          accounts,
          status: accounts.length ? connector.status : "disconnected",
        };
        this.connectors.set(next.id, next);
        this.emit({ type: "connector.changed", timestamp: now(), action: "removed", connector: next });
        return { requestId: command.requestId };
      }
      case "connector.setToolEnabled": {
        const connector = this.connectors.get(command.connectorId);
        if (!connector) throw new Error(`Unknown connector: ${command.connectorId}`);
        if (!connector.tools.some((tool) => tool.id === command.toolId)) {
          throw new Error(`Unknown connector tool: ${command.toolId}`);
        }
        const next = {
          ...connector,
          tools: connector.tools.map((tool) =>
            tool.id === command.toolId ? { ...tool, enabled: command.enabled } : tool,
          ),
        };
        this.connectors.set(next.id, next);
        this.emit({ type: "connector.changed", timestamp: now(), action: "toolChanged", connector: next });
        return { requestId: command.requestId };
      }
      case "skill.list":
        this.emit({
          type: "skill.listed",
          timestamp: now(),
          skills: [...this.skills.values()].filter(
            (skill) => !command.agentId || skill.ownerAgentId === command.agentId,
          ),
          teams: defaultTeams(),
        });
        return { requestId: command.requestId };
      case "skill.upsert": {
        if (!command.name.trim()) throw new Error("Skill name must not be empty");
        if (!command.useWhen.trim()) throw new Error("Skill useWhen must not be empty");
        if (!command.instructions.trim()) throw new Error("Skill instructions must not be empty");
        const id = command.id ?? this.nextId("skill");
        const previous = this.skills.get(id);
        if (previous?.readOnly) throw new Error("Managed skills cannot be edited");
        const skill: SkillSummary = {
          id,
          name: command.name.trim(),
          description: command.description.trim(),
          useWhen: command.useWhen.trim(),
          instructions: command.instructions,
          source: previous?.source ?? "private",
          publishState: previous?.publishState ?? "local",
          ownerAgentId: command.ownerAgentId,
          teamId: previous?.teamId,
          teamName: previous?.teamName,
          readOnly: previous?.readOnly ?? false,
          updatedAtMs: Date.now(),
        };
        this.skills.set(id, skill);
        this.emit({
          type: "skill.changed",
          timestamp: now(),
          action: previous ? "updated" : "created",
          skill,
        });
        return { requestId: command.requestId };
      }
      case "skill.delete": {
        const skill = this.skills.get(command.id);
        if (!skill) throw new Error(`Unknown skill: ${command.id}`);
        if (skill.readOnly || skill.source === "team") throw new Error("Team-managed skills cannot be deleted");
        this.skills.delete(command.id);
        this.emit({ type: "skill.changed", timestamp: now(), action: "deleted", skill });
        return { requestId: command.requestId };
      }
      case "skill.publish": {
        const skill = this.skills.get(command.id);
        if (!skill) throw new Error(`Unknown skill: ${command.id}`);
        const team = defaultTeams().find((item) => item.id === command.teamId);
        if (!team) throw new Error(`Unknown skill team: ${command.teamId}`);
        if (!skill.description.trim()) throw new Error("Add a description first");
        const next: SkillSummary = {
          ...skill,
          source: "team",
          publishState: "published",
          teamId: team.id,
          teamName: team.name,
          updatedAtMs: Date.now(),
        };
        this.skills.set(next.id, next);
        this.emit({ type: "skill.changed", timestamp: now(), action: "published", skill: next });
        return { requestId: command.requestId };
      }
      case "skill.unpublish": {
        const skill = this.skills.get(command.id);
        if (!skill) throw new Error(`Unknown skill: ${command.id}`);
        if (skill.publishState === "managed") throw new Error("Managed skills cannot be unpublished");
        const next: SkillSummary = {
          ...skill,
          source: "private",
          publishState: "local",
          teamId: undefined,
          teamName: undefined,
          updatedAtMs: Date.now(),
        };
        this.skills.set(next.id, next);
        this.emit({ type: "skill.changed", timestamp: now(), action: "unpublished", skill: next });
        return { requestId: command.requestId };
      }
      case "skill.sync": {
        const skill = this.skills.get(command.id);
        if (!skill) throw new Error(`Unknown skill: ${command.id}`);
        if (!skill.teamId) throw new Error("Only published skills can be synced");
        const next: SkillSummary = { ...skill, publishState: "synced", updatedAtMs: Date.now() };
        this.skills.set(next.id, next);
        this.emit({ type: "skill.changed", timestamp: now(), action: "synced", skill: next });
        return { requestId: command.requestId };
      }
      case "bot.list":
        this.emit({ type: "bot.listed", timestamp: now(), bots: [...this.bots.values()] });
        return { requestId: command.requestId };
      case "bot.create": {
        const name = command.name.replace(/\s+/g, " ").trim().slice(0, 72);
        if (!name) throw new Error("Bot name must not be empty");
        const id = `agent-${++this.sequence}`;
        const bot: BotSummary = {
          id,
          name,
          description: (command.description ?? "").trim().slice(0, 2000),
          title: (command.title ?? "").trim(),
          hidden: false,
          avatarShape: command.avatarShape?.trim() || undefined,
          avatarColor: command.avatarColor?.trim() || undefined,
          notificationsEnabled: true,
          notifyOnUpdates: true,
          unread: false,
          conversationId: `codex:agent:${id}`,
        };
        this.bots.set(bot.id, bot);
        this.emit({ type: "bot.changed", timestamp: now(), action: "created", bot });
        return { requestId: command.requestId };
      }
      case "bot.update": {
        const current = this.bots.get(command.id);
        if (!current) throw new Error(`Unknown bot: ${command.id}`);
        const next: BotSummary = {
          ...current,
          ...(command.name === undefined ? {} : { name: command.name.replace(/\s+/g, " ").trim().slice(0, 72) }),
          ...(command.description === undefined ? {} : { description: command.description.trim().slice(0, 2000) }),
          ...(command.title === undefined ? {} : { title: command.title.trim() }),
          ...(command.avatar === undefined ? {} : { avatar: command.avatar.trim() || undefined }),
          ...(command.avatarShape === undefined ? {} : { avatarShape: command.avatarShape.trim() || undefined }),
          ...(command.avatarColor === undefined ? {} : { avatarColor: command.avatarColor.trim() || undefined }),
          ...(command.notificationsEnabled === undefined ? {} : { notificationsEnabled: command.notificationsEnabled }),
          ...(command.notifyOnUpdates === undefined ? {} : { notifyOnUpdates: command.notifyOnUpdates }),
          ...(command.unread === undefined ? {} : { unread: command.unread }),
        };
        if (!next.name) throw new Error("Bot name must not be empty");
        this.bots.set(next.id, next);
        this.emit({ type: "bot.changed", timestamp: now(), action: "updated", bot: next });
        return { requestId: command.requestId };
      }
      case "bot.clone": {
        const source = this.bots.get(command.id);
        if (!source) throw new Error(`Unknown bot: ${command.id}`);
        const id = `agent-${++this.sequence}`;
        const trimmed = source.name.trim();
        const bot: BotSummary = {
          ...source,
          id,
          name: trimmed ? `${trimmed} copy` : "copy",
          hidden: false,
          unread: false,
          conversationId: `codex:agent:${id}`,
        };
        this.bots.set(bot.id, bot);
        this.emit({ type: "bot.changed", timestamp: now(), action: "cloned", bot });
        return { requestId: command.requestId };
      }
      case "bot.delete": {
        if (command.id === "mahayana-assistant") throw new Error("The primary assistant cannot be deleted");
        const bot = this.bots.get(command.id);
        if (!bot) throw new Error(`Unknown bot: ${command.id}`);
        this.bots.delete(command.id);
        this.emit({ type: "bot.changed", timestamp: now(), action: "deleted", bot });
        return { requestId: command.requestId };
      }
      case "bot.setHidden": {
        const bot = this.bots.get(command.id);
        if (!bot) throw new Error(`Unknown bot: ${command.id}`);
        const next = { ...bot, hidden: command.hidden };
        this.bots.set(next.id, next);
        this.emit({ type: "bot.changed", timestamp: now(), action: "updated", bot: next });
        return { requestId: command.requestId };
      }
      case "group.list":
        this.emit({ type: "group.listed", timestamp: now(), groups: [...this.groups.values()] });
        return { requestId: command.requestId };
      case "group.create": {
        const name = command.name.replace(/\s+/g, " ").trim().slice(0, 72);
        const memberIds = [...new Set(command.memberIds)].filter((id) => this.bots.has(id));
        if (!name) throw new Error("Group name must not be empty");
        if (!memberIds.length) throw new Error("Group must contain at least one Bot");
        const id = `group-${++this.sequence}`;
        const createdAtMs = Date.now();
        const group: GroupSummary = {
          id,
          name,
          description: (command.description ?? "").trim().slice(0, 2000),
          memberIds,
          messages: [],
          createdAtMs,
          updatedAtMs: createdAtMs,
        };
        this.groups.set(id, group);
        this.emit({ type: "group.changed", timestamp: now(), action: "created", group });
        return { requestId: command.requestId };
      }
      case "group.update": {
        const current = this.groups.get(command.id);
        if (!current) throw new Error(`Unknown group: ${command.id}`);
        const memberIds = command.memberIds === undefined
          ? current.memberIds
          : [...new Set(command.memberIds)].filter((id) => this.bots.has(id));
        if (!memberIds.length) throw new Error("Group must contain at least one Bot");
        const group: GroupSummary = {
          ...current,
          ...(command.name === undefined ? {} : { name: command.name.replace(/\s+/g, " ").trim().slice(0, 72) }),
          ...(command.description === undefined ? {} : { description: command.description.trim().slice(0, 2000) }),
          memberIds,
          updatedAtMs: Date.now(),
        };
        if (!group.name) throw new Error("Group name must not be empty");
        this.groups.set(group.id, group);
        this.emit({ type: "group.changed", timestamp: now(), action: "updated", group });
        return { requestId: command.requestId };
      }
      case "group.delete": {
        const group = this.groups.get(command.id);
        if (!group) throw new Error(`Unknown group: ${command.id}`);
        this.groups.delete(command.id);
        this.emit({ type: "group.changed", timestamp: now(), action: "deleted", group });
        return { requestId: command.requestId };
      }
      case "group.send": {
        const current = this.groups.get(command.id);
        if (!current) throw new Error(`Unknown group: ${command.id}`);
        const text = command.text.trim().slice(0, 8000);
        if (!text) throw new Error("Group message must not be empty");
        const group: GroupSummary = {
          ...current,
          messages: [...current.messages, {
            id: `group-message-${++this.sequence}`,
            speaker: { kind: "user" as const },
            content: text,
            createdAtMs: Date.now(),
          }].slice(-500),
          updatedAtMs: Date.now(),
        };
        this.groups.set(group.id, group);
        this.emit({ type: "group.changed", timestamp: now(), action: "message", group });
        return { requestId: command.requestId };
      }
      case "agent.send": {
        const sender = this.bots.get(command.fromAgentId);
        if (!sender) throw new Error(`Unknown sender bot: ${command.fromAgentId}`);
        if (command.fromAgentId === command.targetId) throw new Error("An agent cannot message itself");
        const text = command.text.trim().slice(0, 8000);
        if (!text) throw new Error("Agent message must not be empty");
        const target = this.bots.get(command.targetId);
        if (target) {
          const message: AgentPeerMessage = {
            id: `agent-message-${++this.sequence}`,
            fromAgentId: sender.id,
            fromAgentName: sender.name,
            targetId: target.id,
            targetName: target.name,
            text,
            priority: command.priority === true,
            createdAtMs: Date.now(),
          };
          this.peerMessages = [...this.peerMessages, message].slice(-5000);
          this.emit({ type: "agent.peerMessage", timestamp: now(), message });
          const operationId = `agent-background-${++this.sequence}`;
          this.emit({ type: "agent.backgroundStarted", timestamp: now(), agentId: target.id, agentName: target.name, operationId, source: command.priority ? "agent-priority" : "agent-message" });
          this.emit({ type: "agent.backgroundMessage", timestamp: now(), agentId: target.id, agentName: target.name, operationId, source: command.priority ? "agent-priority" : "agent-message", text: `${target.name} received the message from ${sender.name}.` });
          this.emit({ type: "agent.backgroundFinished", timestamp: now(), agentId: target.id, agentName: target.name, operationId, source: command.priority ? "agent-priority" : "agent-message" });
          return { requestId: command.requestId };
        }
        const current = this.groups.get(command.targetId);
        if (!current) throw new Error(`Unknown agent or group: ${command.targetId}`);
        if (!current.memberIds.includes(sender.id)) throw new Error(`Agent is not a member of group: ${command.targetId}`);
        const group: GroupSummary = {
          ...current,
          messages: [...current.messages, { id: `group-message-${++this.sequence}`, speaker: { kind: "member" as const, id: sender.id, name: sender.name }, content: text, createdAtMs: Date.now() }].slice(-500),
          updatedAtMs: Date.now(),
        };
        this.groups.set(group.id, group);
        this.emit({ type: "group.changed", timestamp: now(), action: "message", group });
        return { requestId: command.requestId };
      }
      case "agent.broadcast": {
        const targets = command.targetIds?.length
          ? [...new Set(command.targetIds)].map((id) => this.bots.get(id)).filter((bot): bot is BotSummary => Boolean(bot))
          : [...this.bots.values()];
        const message = command.message.trim().slice(0, 8000);
        if (!message) throw new Error("Broadcast message must not be empty");
        for (const target of targets) {
          const operationId = `broadcast-${++this.sequence}`;
          this.emit({ type: "agent.backgroundStarted", timestamp: now(), agentId: target.id, agentName: target.name, operationId, source: "broadcast" });
          this.emit({ type: "agent.backgroundMessage", timestamp: now(), agentId: target.id, agentName: target.name, operationId, source: "broadcast", text: `${target.name} received the broadcast.` });
          this.emit({ type: "agent.backgroundFinished", timestamp: now(), agentId: target.id, agentName: target.name, operationId, source: "broadcast" });
        }
        this.emit({ type: "agent.broadcasted", timestamp: now(), result: { total: targets.length, scheduled: targets.length } });
        return { requestId: command.requestId };
      }
      case "agent.peerHistory":
        this.emit({ type: "agent.peerHistory", timestamp: now(), agentId: command.agentId, messages: this.peerMessages.filter((message) => message.fromAgentId === command.agentId || message.targetId === command.agentId).slice(-(command.limit ?? 200)) });
        return { requestId: command.requestId };
      case "subagent.list":
        this.emit({ type: "subagent.listed", timestamp: now(), agentId: command.agentId, subagents: [] });
        return { requestId: command.requestId };
      case "asyncTask.list":
        this.emit({ type: "asyncTask.listed", timestamp: now(), agentId: command.agentId, tasks: [] });
        return { requestId: command.requestId };
      case "teach.status":
        this.emit({
          type: "teach.changed",
          timestamp: now(),
          status: this.teachRecording
            ? { state: "recording", agentId: this.teachRecording.agentId, startedAtMs: this.teachRecording.startedAtMs, maxDurationMs: 600_000 }
            : { state: "idle", maxDurationMs: 600_000 },
        });
        return { requestId: command.requestId };
      case "teach.start":
        if (this.teachRecording && this.teachRecording.agentId !== command.agentId) throw new Error(`Teach recording is active for ${this.teachRecording.agentId}`);
        this.teachRecording ??= { agentId: command.agentId, startedAtMs: Date.now() };
        this.emit({ type: "teach.changed", timestamp: now(), status: { state: "recording", agentId: command.agentId, startedAtMs: this.teachRecording.startedAtMs, maxDurationMs: 600_000 } });
        return { requestId: command.requestId };
      case "teach.stop": {
        const active = this.teachRecording;
        if (active && active.agentId !== command.agentId) throw new Error(`Teach recording belongs to ${active.agentId}`);
        this.teachRecording = null;
        this.emit({
          type: "teach.changed",
          timestamp: now(),
          status: { state: "idle", maxDurationMs: 600_000 },
          result: active ? { agentId: command.agentId, videoPath: `/mock/agents/${command.agentId}/teach-sessions/demo.mp4`, startedAtMs: active.startedAtMs, endedAtMs: Date.now(), durationMs: Date.now() - active.startedAtMs, saved: command.save } : undefined,
        });
        return { requestId: command.requestId };
      }
      case "computer.status":
        this.emit({
          type: "computer.status",
          timestamp: now(),
          requestId: command.requestId,
          status: {
            platform: "mock",
            available: true,
            captureSupported: true,
            inputSupported: true,
            accessibilityGranted: true,
            screenRecordingGranted: true,
            localExecutionEnabled: this.hostSettings.localExecution,
            routeEgressLocally: this.hostSettings.routeEgressLocally,
            remoteControlEnabled: this.hostSettings.remoteControlEnabled,
            aiControlEnabled: this.hostSettings.aiComputerControlEnabled,
          },
        });
        return { requestId: command.requestId };
      case "computer.screenshot": {
        const snapshot = mockComputerSnapshot();
        this.emit({ type: "computer.snapshot", timestamp: now(), requestId: command.requestId, origin: command.origin ?? "local-ui", snapshot });
        return { requestId: command.requestId };
      }
      case "computer.action": {
        const origin = command.origin ?? "local-ui";
        const actions = [command.action, ...(command.then ?? [])];
        if (actions.length > 10) throw new Error("At most 10 computer actions can be batched");
        const snapshot = mockComputerSnapshot();
        this.emit({ type: "computer.result", timestamp: now(), requestId: command.requestId, result: { origin, actionsExecuted: actions.length, snapshot } });
        return { requestId: command.requestId };
      }
      case "remoteComputer.register":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "registered", data: { deviceId: command.deviceId, label: command.label, provider: command.provider, platform: command.platform, appVersion: command.appVersion, capabilities: command.capabilities, pairingCode: "AB12CD34", pairingExpiresAt: Math.floor(Date.now() / 1000) + 600 } });
        return { requestId: command.requestId };
      case "remoteComputer.heartbeat":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "heartbeat", data: { ok: true, lastSeenAt: Math.floor(Date.now() / 1000) } });
        return { requestId: command.requestId };
      case "remoteComputer.clients":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "clients", data: { deviceId: command.deviceId, clients: [] } });
        return { requestId: command.requestId };
      case "remoteComputer.clientRevoke":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "clientRevoked", data: { revoked: true, clientId: command.clientId } });
        return { requestId: command.requestId };
      case "remoteComputer.sessions":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "sessions", data: { deviceId: command.deviceId, sessions: [] } });
        return { requestId: command.requestId };
      case "remoteComputer.sessionActivate":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "sessionActivated", data: { sessionId: command.sessionId, clientId: "remote-client-test", expiresAt: Math.floor(Date.now() / 1000) + 7200, state: "active" } });
        return { requestId: command.requestId };
      case "remoteComputer.sessionClose":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "sessionClosed", data: { sessionId: command.sessionId, state: "closed" } });
        return { requestId: command.requestId };
      case "remoteComputer.signal":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "signal", data: { accepted: true, expiresAt: Math.floor(Date.now() / 1000) + 300 } });
        return { requestId: command.requestId };
      case "remoteComputer.signalDrain":
        this.emit({ type: "remoteComputer.changed", timestamp: now(), requestId: command.requestId, action: "signals", data: { sessionId: command.sessionId, signals: [], lastSignalId: command.afterSignalId ?? 0 } });
        return { requestId: command.requestId };
      case "memory.list": {
        const memories = this.memories.get(command.agentId) ?? [];
        const limit = Math.min(command.limit ?? 1000, 1000);
        this.emit({ type: "memory.listed", timestamp: now(), agentId: command.agentId, memories: memories.slice(0, limit), count: memories.length, location: `/mock/agents/${command.agentId}/memory` });
        return { requestId: command.requestId };
      }
      case "memory.add": {
        const content = normalizeMemoryContent(command.content);
        const current = this.memories.get(command.agentId) ?? [];
        const duplicate = current.some((memory) => memoryDedupeKey(memory.content) === memoryDedupeKey(content));
        const memory = !content || duplicate ? undefined : {
          id: makeMemoryId(content),
          content,
          createdAt: Date.now(),
          kind: command.kind,
        } satisfies MemoryRecord;
        if (memory) this.memories.set(command.agentId, [memory, ...current]);
        this.emit({ type: "memory.changed", timestamp: now(), agentId: command.agentId, action: memory ? "added" : "duplicate", memory });
        return { requestId: command.requestId };
      }
      case "memory.remove": {
        const current = this.memories.get(command.agentId) ?? [];
        const next = current.filter((memory) => memory.id !== command.id);
        this.memories.set(command.agentId, next);
        this.emit({ type: "memory.changed", timestamp: now(), agentId: command.agentId, action: next.length === current.length ? "notFound" : "removed" });
        return { requestId: command.requestId };
      }
      case "memory.clear":
        this.memories.delete(command.agentId);
        this.emit({ type: "memory.changed", timestamp: now(), agentId: command.agentId, action: "cleared" });
        return { requestId: command.requestId };
      case "tray.list":
        this.emit({ type: "tray.listed", timestamp: now(), trays: [...this.trays.getTrays()] });
        return { requestId: command.requestId };
      case "tray.dismiss":
        if (this.trays.dismiss(command.id)) {
          this.emit({ type: "tray.changed", timestamp: now(), action: "dismissed", id: command.id });
        }
        return { requestId: command.requestId };
      case "tray.clear":
        if (this.trays.getTrays().length) {
          this.trays.clearAll();
          this.emit({ type: "tray.changed", timestamp: now(), action: "cleared" });
        }
        return { requestId: command.requestId };
      case "tray.clearForAgent": {
        const ids = this.trays.getTrays().filter((tray) => tray.agentId === command.agentId).map((tray) => tray.id);
        this.trays.clearForAgent(command.agentId);
        for (const id of ids) this.emit({ type: "tray.changed", timestamp: now(), action: "dismissed", id });
        return { requestId: command.requestId };
      }
      case "workflow.list": {
        const disabled = this.disabledWorkflows.get(command.agentId) ?? new Set<string>();
        const workflows = [...this.workflows.values()].map((workflow) => ({
          ...workflow,
          isEnabledForAgent: !disabled.has(workflow.id),
        }));
        this.emit({ type: "workflow.listed", timestamp: now(), agentId: command.agentId, workflows });
        return { requestId: command.requestId };
      }
      case "workflow.upsert": {
        const name = command.name.replace(/\s+/g, " ").trim().slice(0, 80);
        const body = command.body.trim().slice(0, 100_000);
        if (!name || !body) throw new Error("Workflow name and body are required");
        const id = command.id ?? (name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 48) || `workflow-${++this.sequence}`);
        const previous = this.workflows.get(id);
        const workflow: WorkflowSummary = {
          id,
          name,
          description: (command.description ?? "").replace(/\s+/g, " ").trim().slice(0, 1536),
          body,
          trigger: command.trigger,
          sourceRef: command.sourceRef,
          source: command.trigger ? "automation" : "workflow",
          publishedByCurrentUser: false,
          isEnabledForAgent: !(this.disabledWorkflows.get(command.agentId)?.has(id) ?? false),
          createdAt: previous?.createdAt ?? Date.now(),
          lastRunAt: previous?.lastRunAt,
          nextRunAt: undefined,
          helperScripts: previous?.helperScripts ?? [],
          filePath: command.trigger ? "" : `/mock/workflows/${id}/SKILL.md`,
        };
        this.workflows.set(id, workflow);
        this.emit({ type: "workflow.changed", timestamp: now(), agentId: command.agentId, action: "saved", workflow });
        return { requestId: command.requestId };
      }
      case "workflow.setEnabled": {
        const workflow = this.workflows.get(command.id);
        if (!workflow) throw new Error(`Unknown workflow: ${command.id}`);
        const disabled = new Set(this.disabledWorkflows.get(command.agentId) ?? []);
        if (command.enabled) disabled.delete(command.id);
        else disabled.add(command.id);
        this.disabledWorkflows.set(command.agentId, disabled);
        const next = { ...workflow, isEnabledForAgent: command.enabled };
        this.workflows.set(command.id, next);
        this.emit({ type: "workflow.changed", timestamp: now(), agentId: command.agentId, action: "enabled", workflow: next, id: command.id });
        return { requestId: command.requestId };
      }
      case "workflow.delete": {
        this.workflows.delete(command.id);
        for (const disabled of this.disabledWorkflows.values()) disabled.delete(command.id);
        this.emit({ type: "workflow.changed", timestamp: now(), agentId: command.agentId, action: "deleted", id: command.id });
        return { requestId: command.requestId };
      }
      case "workflow.run": {
        const workflow = this.workflows.get(command.id);
        if (!workflow) throw new Error(`Unknown workflow: ${command.id}`);
        if (this.disabledWorkflows.get(command.agentId)?.has(command.id)) throw new Error(`Workflow is disabled: ${command.id}`);
        this.emit({ type: "chat.message", timestamp: now(), role: "user", text: `@${workflow.name}` });
        this.emit({ type: "chat.message", timestamp: now(), role: "assistant", text: `Running workflow: ${workflow.name}` });
        return { requestId: command.requestId };
      }
      case "workflow.importMarkdown": {
        const raw = command.markdown.trim();
        if (!raw) throw new Error("Workflow markdown is empty");
        const frontmatter = raw.startsWith("---\n") ? raw.slice(4, raw.indexOf("\n---", 4)) : "";
        const body = frontmatter ? raw.slice(raw.indexOf("\n---", 4) + 4).replace(/^\n/, "").trim() : raw;
        const frontmatterName = frontmatter.match(/^name:\s*["']?([^\n"']+)/m)?.[1]?.trim();
        const headingName = body.match(/^#{1,6}\s+(.+)$/m)?.[1]?.replace(/[*_`#>]/g, "").trim();
        const name = frontmatterName || command.fallbackName || headingName || "Imported workflow";
        const id = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 48) || `workflow-${++this.sequence}`;
        const description = frontmatter.match(/^description:\s*["']?([^\n"']+)/m)?.[1]?.trim() ?? "";
        const schedule = frontmatter.match(/^\s*schedule:\s*["']?([^\n"']+)/m)?.[1]?.trim();
        const enabledRaw = frontmatter.match(/^\s*enabled:\s*(true|false)/m)?.[1];
        const workflow: WorkflowSummary = {
          id,
          name: name.slice(0, 80),
          description: description.slice(0, 1536),
          body: body.slice(0, 100_000),
          trigger: schedule ? { schedule, isEnabled: enabledRaw !== "false" } : undefined,
          source: schedule ? "automation" : "workflow",
          publishedByCurrentUser: false,
          isEnabledForAgent: true,
          createdAt: Date.now(),
          helperScripts: [],
          filePath: schedule ? "" : `/mock/workflows/${id}/SKILL.md`,
        };
        this.workflows.set(id, workflow);
        this.emit({ type: "workflow.changed", timestamp: now(), agentId: command.agentId, action: "imported", workflow });
        return { requestId: command.requestId };
      }
      case "workflow.importLiveSource": {
        const source = command.source.trim();
        if (!source) throw new Error("Live source is required");
        const leaf = source.split("/").filter(Boolean).at(-1)?.replace(/\.(markdown|mdc|md|txt)$/i, "").replace(/[-_]/g, " ") || "Imported skill";
        const name = (command.fallbackName || leaf).trim().slice(0, 80);
        const id = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 48) || `workflow-${++this.sequence}`;
        const workflow: WorkflowSummary = {
          id,
          name,
          description: `Use when the \"${name}\" skill applies; it is a live reference to ${source}.`.slice(0, 1536),
          body: `This workflow is a live reference to the skill at \`${source}\`.\nRead that source now with your file or fetch tools and follow it as written. Do not assume its contents from this note; the source is the source of truth and may have changed since this workflow was created.`,
          sourceRef: source,
          source: "workflow",
          publishedByCurrentUser: false,
          isEnabledForAgent: true,
          createdAt: Date.now(),
          helperScripts: [],
          filePath: `/mock/workflows/${id}/SKILL.md`,
        };
        this.workflows.set(id, workflow);
        this.emit({ type: "workflow.changed", timestamp: now(), agentId: command.agentId, action: "imported", workflow });
        return { requestId: command.requestId };
      }
      case "attachment.upload": {
        const bytes = command.bytesBase64.trim();
        if (!bytes) throw new Error("Attachment is empty");
        const safeName = command.filename.replace(/[^a-zA-Z0-9._-]+/g, "-") || `file-${++this.sequence}`;
        const path = `/mock/agents/${command.agentId}/attachments/${safeName}`;
        this.attachmentData.set(path, {
          agentId: command.agentId,
          name: command.filename,
          mimeType: command.mimeType,
          bytesBase64: bytes,
        });
        const sizeBytes = Math.floor(bytes.length * 3 / 4);
        this.emit({
          type: "attachment.stored",
          timestamp: now(),
          attachment: {
            id: `mock-attachment-${++this.sequence}`,
            agentId: command.agentId,
            name: command.filename,
            path,
            mimeType: command.mimeType,
            sizeBytes,
            hash: `mock-${this.sequence}`,
          },
        });
        return { requestId: command.requestId };
      }
      case "attachment.readText": {
        const stored = this.attachmentData.get(command.path);
        if (!stored || stored.agentId !== command.agentId) throw new Error("Unknown attachment");
        const raw = Uint8Array.from(atob(stored.bytesBase64), (character) => character.charCodeAt(0));
        const text = new TextDecoder().decode(raw);
        this.emit({
          type: "attachment.text",
          timestamp: now(),
          result: {
            path: command.path,
            kind: "text",
            text: text.slice(0, 64 * 1024),
            truncated: text.length > 64 * 1024,
            bytes: raw.byteLength,
          },
        });
        return { requestId: command.requestId };
      }
      case "attachment.readChunk": {
        const stored = this.attachmentData.get(command.path);
        if (!stored || stored.agentId !== command.agentId) throw new Error("Unknown attachment");
        this.emit({
          type: "attachment.chunk",
          timestamp: now(),
          result: {
            path: command.path,
            bytesBase64: stored.bytesBase64,
            totalSize: Math.floor(stored.bytesBase64.length * 3 / 4),
            mime: stored.mimeType,
          },
        });
        return { requestId: command.requestId };
      }
      case "attachment.readImage": {
        const stored = this.attachmentData.get(command.path);
        if (!stored || stored.agentId !== command.agentId) throw new Error("Unknown attachment");
        this.emit({
          type: "attachment.image",
          timestamp: now(),
          result: {
            path: command.path,
            dataUrl: `data:${stored.mimeType ?? "application/octet-stream"};base64,${stored.bytesBase64}`,
          },
        });
        return { requestId: command.requestId };
      }
      case "search.messages": {
        const query = command.query.trim().toLowerCase();
        const limit = Math.max(1, Math.min(command.limit ?? 50, 50));
        const matches = query
          ? richResultsMessages()
              .filter((message) => message.text.toLowerCase().includes(query))
              .slice(-5)
              .reverse()
              .map((message) => {
                const flat = message.text.replace(/\s+/g, " ").trim();
                const at = flat.toLowerCase().indexOf(query);
                const start = Math.max(0, at - 30);
                const end = Math.min(flat.length, at + query.length + 60);
                return {
                  agentId: "mahayana-assistant",
                  agentName: "大乘助手",
                  conversationId: RICH_RESULTS_CONVERSATION_ID,
                  entryId: message.id,
                  role: message.role,
                  timestampMs: message.createdAtMs,
                  snippet: `${start ? "…" : ""}${flat.slice(start, end)}${end < flat.length ? "…" : ""}`,
                };
              })
              .slice(0, limit)
          : [];
        this.emit({ type: "search.messages", timestamp: now(), query, matches });
        return { requestId: command.requestId };
      }
      case "search.media": {
        const query = command.query?.trim().toLowerCase() ?? "";
        const matches = [...this.attachmentData.entries()]
          .filter(([, attachment]) => !query || attachment.name.toLowerCase().includes(query))
          .slice(0, Math.max(1, Math.min(command.limit ?? 50, 50)))
          .map(([path, attachment]) => ({
            agentId: attachment.agentId,
            agentName: this.bots.get(attachment.agentId)?.name ?? attachment.agentId,
            path,
            name: attachment.name,
            mimeType: attachment.mimeType,
            sizeBytes: Math.floor(attachment.bytesBase64.length * 3 / 4),
            timestampMs: Date.now(),
          }));
        this.emit({ type: "search.media", timestamp: now(), query, matches });
        return { requestId: command.requestId };
      }
      case "mcp.list":
        this.emit({
          type: "mcp.listed",
          timestamp: now(),
          servers: [
            { name: "github", status: "connected", transport: "streamable_http", tools: [{ name: "search_repositories" }, { name: "get_pull_request" }] },
            { name: "notion", status: "auth_required", transport: "streamable_http", tools: [] },
          ],
        });
        return { requestId: command.requestId };
      case "mcp.apps":
        this.emit({ type: "mcp.apps", timestamp: now(), apps: [{ id: "github", name: "GitHub" }, { id: "notion", name: "Notion" }] });
        return { requestId: command.requestId };
      case "mcp.oauthLogin":
        this.emit({ type: "mcp.oauth", timestamp: now(), server: command.server, authorizationUrl: `https://example.test/oauth/${encodeURIComponent(command.server)}`, removed: false });
        return { requestId: command.requestId };
      case "mcp.oauthLogout":
        this.emit({ type: "mcp.oauth", timestamp: now(), server: command.server, removed: true });
        return { requestId: command.requestId };
      case "mcp.remove":
        this.emit({ type: "mcp.refreshed", timestamp: now() });
        return { requestId: command.requestId };
      case "mcp.setCustomInstructions":
      case "mcp.setToolDisabled":
        this.emit({ type: "mcp.refreshed", timestamp: now() });
        return { requestId: command.requestId };
      case "mcp.refresh":
        this.emit({ type: "mcp.refreshed", timestamp: now() });
        return { requestId: command.requestId };
      case "mcp.toolCall":
        this.emit({ type: "mcp.toolResult", timestamp: now(), server: command.server, tool: command.tool, result: { ok: true, arguments: command.arguments ?? null } });
        return { requestId: command.requestId };
      case "settings.get":
        this.emit({ type: "settings.changed", timestamp: now(), settings: this.hostSettings });
        return { requestId: command.requestId };
      case "settings.update":
        this.hostSettings = { ...command.settings, autoReviewRules: command.settings.autoReviewRules.slice(0, 200) };
        this.emit({ type: "settings.changed", timestamp: now(), settings: this.hostSettings });
        return { requestId: command.requestId };
      case "audit.list":
        this.emit({ type: "audit.listed", timestamp: now(), agentId: command.agentId, records: [] });
        return { requestId: command.requestId };
      case "draft.resolve": {
        if (command.action === "send") validateDraft(command.draft);
        if (command.action === "send") {
          this.emit({
            type: "draft.changed",
            timestamp: now(),
            draftId: command.draft.id,
            status: "sending",
          });
        }
        this.emit({
          type: "draft.changed",
          timestamp: now(),
          draftId: command.draft.id,
          status: command.action === "send" ? "sent" : "discarded",
        });
        return { requestId: command.requestId };
      }
      case "widget.respond":
        this.emit({
          type: "widget.changed",
          timestamp: now(),
          interaction: {
            widgetId: command.widgetId,
            agentId: command.agentId,
            conversationId: command.conversationId,
            actionId: command.actionId,
            value: command.value,
            status: "responded",
            createdAtMs: Date.now(),
          },
        });
        return { requestId: command.requestId };
      case "widget.dismiss":
        this.emit({
          type: "widget.changed",
          timestamp: now(),
          interaction: {
            widgetId: command.widgetId,
            agentId: command.agentId,
            conversationId: command.conversationId,
            status: "dismissed",
            reason: command.reason,
            createdAtMs: Date.now(),
          },
        });
        return { requestId: command.requestId };
      case "secret.provide":
        if (!command.value.trim()) throw new Error("Secret value must not be empty");
        this.providedSecrets.add(command.secretRequestId);
        this.emit({
          type: "secret.provided",
          timestamp: now(),
          secretRequestId: command.secretRequestId,
        });
        return { requestId: command.requestId };
      case "listener.list":
        this.emit({
          type: "listener.listed",
          timestamp: now(),
          integrations: [...this.listenerIntegrations.values()],
        });
        return { requestId: command.requestId };
      case "listener.connect": {
        const integration = this.listenerIntegrations.get(command.platform);
        if (!integration) throw new Error(`Unsupported listener: ${command.platform}`);
        const next = { ...integration, isConnected: true, error: undefined };
        this.listenerIntegrations.set(command.platform, next);
        this.emit({ type: "listener.changed", timestamp: now(), integration: next });
        const connector = this.connectors.get(connectorForPlatform(command.platform));
        if (connector) {
          const connected: ConnectorSummary = { ...connector, status: "connected" };
          this.connectors.set(connected.id, connected);
          this.emit({ type: "connector.changed", timestamp: now(), action: "connected", connector: connected });
        }
        return { requestId: command.requestId };
      }
      case "listener.disconnect": {
        const integration = this.listenerIntegrations.get(command.platform);
        if (!integration) throw new Error(`Unsupported listener: ${command.platform}`);
        const next = { ...integration, isConnected: false, accountLabel: undefined, error: undefined };
        this.listenerIntegrations.set(command.platform, next);
        this.emit({ type: "listener.changed", timestamp: now(), integration: next });
        const connector = this.connectors.get(connectorForPlatform(command.platform));
        if (connector) {
          const disconnected: ConnectorSummary = { ...connector, status: "disconnected", accounts: [] };
          this.connectors.set(disconnected.id, disconnected);
          this.emit({ type: "connector.changed", timestamp: now(), action: "disconnected", connector: disconnected });
        }
        return { requestId: command.requestId };
      }
      case "update.status":
        this.emit({ type: "update.changed", timestamp: now(), state: this.updateState });
        return { requestId: command.requestId };
      case "update.check":
        this.updateState = { type: "checking" };
        this.emit({ type: "update.changed", timestamp: now(), state: this.updateState });
        this.updateState = { type: "upToDate", version: "0.1.0" };
        this.emit({ type: "update.changed", timestamp: now(), state: this.updateState });
        return { requestId: command.requestId };
      case "update.install": {
        const version =
          this.updateState.type === "available" ||
          this.updateState.type === "downloading" ||
          this.updateState.type === "staging" ||
          this.updateState.type === "ready"
            ? this.updateState.version
            : null;
        if (!version) throw new Error("No update is available to install");
        this.updateState = { type: "downloading", version, progress: 100 };
        this.emit({ type: "update.changed", timestamp: now(), state: this.updateState });
        this.updateState = { type: "ready", version };
        this.emit({ type: "update.changed", timestamp: now(), state: this.updateState });
        return { requestId: command.requestId };
      }
      case "runtime.longTask": {
        const operationId = this.nextId("operation");
        this.emit({
          type: "operation.started",
          timestamp: now(),
          operationId,
          label: command.label,
          interruptible: true,
        });
        return { requestId: command.requestId, operationId };
      }
      case "session.clear":
        this.emit({ type: "session.cleared", timestamp: now() });
        return { requestId: command.requestId };
    }
  }

  async authStatus(): Promise<AuthState> {
    if (this.native) return this.native.authStatus();
    return this.auth;
  }

  async browserLoginStart(): Promise<BrowserLoginAttempt> {
    if (this.native) return this.native.browserLoginStart();
    this.assertReady();
    this.browserLoginAttempt = {
      attemptId: `browser-${this.nextId("attempt")}`,
      loginUrl: "about:blank#fabushi-test-browser-login",
      expiresAt: Math.floor(Date.now() / 1000) + 600,
      pollAfterMs: 120,
    };
    return this.browserLoginAttempt;
  }

  async browserLoginPoll(attemptId: string): Promise<BrowserLoginPollResult> {
    if (this.native) return this.native.browserLoginPoll(attemptId);
    if (this.browserLoginAttempt?.attemptId !== attemptId) return { status: "expired" };
    this.auth = {
      loggedIn: true,
      provider: "browser",
      user: {
        id: "fast-e2e-browser-user",
        email: "browser@example.test",
        nickname: "Browser 测试用户",
      },
    };
    this.browserLoginAttempt = null;
    return { status: "completed", provider: "browser", auth: this.auth };
  }

  async browserLoginReopen(attemptId: string): Promise<BrowserLoginReopenResult> {
    if (this.native) return this.native.browserLoginReopen(attemptId);
    if (this.browserLoginAttempt?.attemptId !== attemptId) throw new Error("Browser login attempt expired");
    this.browserLoginAttempt = {
      ...this.browserLoginAttempt,
      loginUrl: "about:blank#fabushi-test-browser-login",
      pollAfterMs: 120,
    };
    return { ...this.browserLoginAttempt, status: "pending" };
  }

  async browserLoginCancel(attemptId: string): Promise<BrowserLoginPollResult> {
    if (this.native) return this.native.browserLoginCancel(attemptId);
    if (this.browserLoginAttempt?.attemptId !== attemptId) return { status: "expired" };
    this.browserLoginAttempt = null;
    return { status: "cancelled" };
  }

  async authProviders(): Promise<AuthProvider[]> {
    if (this.native) return this.native.authProviders();
    return [
      { id: "google", displayName: "Google", enabled: true },
      { id: "apple", displayName: "Apple", enabled: true },
      { id: "microsoft", displayName: "Microsoft", enabled: true },
      { id: "github", displayName: "GitHub", enabled: true },
    ];
  }

  async oauthStart(provider: AuthProviderId): Promise<OAuthAttempt> {
    if (this.native) return this.native.oauthStart(provider);
    this.assertReady();
    this.oauthAttempt = {
      attemptId: `oauth-${provider}-${this.nextId("attempt")}`,
      provider,
      authorizationUrl: `about:blank#fabushi-test-oauth-${provider}`,
    };
    return this.oauthAttempt;
  }

  async oauthPoll(attemptId: string): Promise<OAuthPollResult> {
    if (this.native) return this.native.oauthPoll(attemptId);
    if (this.oauthAttempt?.attemptId !== attemptId) {
      return { status: "expired" };
    }
    const provider = this.oauthAttempt.provider;
    this.auth = {
      loggedIn: true,
      provider,
      user: {
        id: `fast-e2e-${provider}`,
        email: `test@${provider}.example`,
        nickname: `${provider} 测试用户`,
      },
    };
    this.oauthAttempt = null;
    return { status: "completed", auth: this.auth };
  }

  async openExternal(url: string): Promise<void> {
    if (this.native) return this.native.openExternal(url);
    if (
      url.startsWith("about:blank#fabushi-test-oauth") ||
      url.startsWith("about:blank#fabushi-test-connector")
    ) return;
    const popup = window.open(url, "fabushi-oauth", "popup,width=520,height=720");
    if (!popup) throw new Error("浏览器窗口被拦截，请允许弹出窗口后重试");
  }

  async openSystemSettings(pane: "screen-recording" | "accessibility"): Promise<void> {
    if (this.native) return this.native.openSystemSettings(pane);
  }

  async windowFocused(): Promise<boolean> {
    if (this.native) return this.native.windowFocused();
    return typeof document === "undefined" ? true : document.hasFocus();
  }

  async showNotification(title: string, body: string): Promise<void> {
    if (this.native) return this.native.showNotification(title, body);
    if (typeof Notification === "undefined") return;
    if (Notification.permission === "default") await Notification.requestPermission();
    if (Notification.permission === "granted") new Notification(title, { body });
  }

  async passwordLogin(username: string, password: string): Promise<AuthState> {
    if (this.native) return this.native.passwordLogin(username, password);
    this.assertReady();
    if (!username.trim() || !password) throw new Error("请输入账号和密码");
    this.auth = {
      loggedIn: true,
      provider: "test",
      user: { id: "fast-e2e-user", username, nickname: "本地测试用户" },
    };
    return this.auth;
  }

  async logout(): Promise<AuthState> {
    if (this.native) return this.native.logout();
    this.auth = { loggedIn: false, provider: "test" };
    return this.auth;
  }

  async interrupt(operationId: string): Promise<void> {
    if (this.native) return this.native.interrupt(operationId);
    this.assertReady();
    this.emit({
      type: "operation.interrupted",
      timestamp: now(),
      operationId,
    });
  }

  async resolveApproval(resolution: ApprovalResolution): Promise<void> {
    if (this.native) return this.native.resolveApproval(resolution);
    this.assertReady();
    if (!this.approvals.delete(resolution.approvalId)) {
      throw new Error(`Unknown approval: ${resolution.approvalId}`);
    }
    this.emit({
      type: "approval.resolved",
      timestamp: now(),
      approvalId: resolution.approvalId,
      decision: resolution.decision,
    });
  }

  async close(): Promise<void> {
    if (this.native) return this.native.close();
    if (this.status === "closed") return;
    this.status = "closed";
    this.emit({ type: "host.closed", timestamp: now() });
  }

  private assertReady(): void {
    if (this.status !== "ready") {
      throw new Error(`Mahayana host is not ready: ${this.status}`);
    }
  }

  private nextId(prefix: string): string {
    this.sequence += 1;
    return `${prefix}-${this.sequence}`;
  }

  private emit(event: RuntimeEvent): void {
    for (const listener of this.listeners) listener(event);
  }
}
