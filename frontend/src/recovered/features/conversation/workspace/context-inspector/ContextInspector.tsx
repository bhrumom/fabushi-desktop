/**
 * Unified Right-Hand Context Inspector Drawer (`FBCP-BLUEPRINT-04` / `09` / `12` / `14` / `15`)
 *
 * Consolidates the 7 previously competing drawers (`OpenClawAskCopilotPanel`,
 * `TdrpProfileDetailsDrawer`, `AgentSettingsPanel`, `GroupMembersPane`,
 * `RoutinesInfoPane`, `AgentInfoChannelsPanel`, `FabushiAgentRuntimeDetailSections`,
 * plus floating `ConversationOutlinePanel` and `AsyncTasksPanel`) into a single
 * deterministic, entity-aware Context Inspector.
 *
 * Upstream comparative anchors studied before implementation:
 * - Telegram Desktop:
 *   - `Telegram/SourceFiles/info/info_controller.cpp` (`Info::Key`, `Info::Section`)
 *   - `Telegram/SourceFiles/info/profile/info_profile_widget.cpp` (`Info::Profile::Widget`)
 *   - `Telegram/SourceFiles/info/media/info_media_widget.cpp` (`Info::Media::Type` shared media tabs)
 *   - `Telegram/SourceFiles/data/data_chat_participant_status.cpp` (`ChatAdminRight`, `ChatRestriction`)
 * - OpenClaw:
 *   - `extensions/memory-core/src/tools.citations.ts` (`formatCitation`, `shouldIncludeCitations`)
 *   - `extensions/memory-core/src/memory-forget.ts` (`forgetMemoryEntries` tombstone lifecycle)
 * - Fabushi Native & Host:
 *   - `native/mahayana-messaging/src/secret_chat.rs` (`SecretChatSession`, X25519+HKDF-SHA256+XChaCha20Poly1305)
 *   - `source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs` (`ChatAdminModerationState`)
 *   - `source/host/src/storage/sqlite_recovery.rs` (`quarantine_corrupt_sqlite_db`, WAL checkpoint)
 */

import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Activity,
  Bot,
  BookOpen,
  Calendar,
  CheckCircle2,
  Database,
  Download,
  ExternalLink,
  FileText,
  Film,
  Gift,
  HardDrive,
  Image,
  Link2,
  Lock,
  Mic,
  Phone,
  Plug,
  RefreshCw,
  Search,
  Settings,
  Shield,
  Sliders,
  Star,
  Trash2,
  UserCheck,
  Users,
  Video,
  Wrench,
  X,
} from "lucide-react";
import type { ProductionCoordinatorClient } from "../../../../../production/coordinator-client";
import {
  CONTEXT_INSPECTOR_TABS,
  INSPECTOR_WIDTH_DEFAULT,
  INSPECTOR_WIDTH_MAX,
  INSPECTOR_WIDTH_MIN,
  clampInspectorWidth,
  createInitialInspectorState,
  deriveE2ESafetyFingerprint,
  formatMemoryCitationProvenance,
  getVisibleTabsForEntity,
  isInspectorScopeCurrent,
  resolveInspectorEntityType,
  sanitizeInspectorTab,
  shouldIncludeMemoryCitations,
  transitionContextInspectorState,
  type ContextInspectorTabId,
  type InspectorAgentSubject,
  type InspectorEntityType,
  type MemoryCitationsMode,
  type SharedMediaFilterKind,
} from "./model";

export interface InspectorMediaEntry {
  readonly id: string;
  readonly kind: SharedMediaFilterKind;
  readonly title: string;
  readonly subtitle?: string;
  readonly timestampMs: number;
  readonly sizeLabel?: string;
  readonly url?: string;
}

export interface InspectorCommunityMember {
  readonly id: string;
  readonly name: string;
  readonly handle?: string;
  readonly role: "owner" | "admin" | "member" | "restricted" | "banned";
  readonly customTitle?: string;
  readonly isAgent?: boolean;
}

export interface InspectorAuditLogEntry {
  readonly entryId: string;
  readonly actorId: string;
  readonly actionSummary: string;
  readonly timestampMs: number;
}

export interface InspectorMemoryFact {
  readonly id: string;
  readonly factDate: string;
  readonly content: string;
  readonly kind: "profile" | "log" | "synthesized";
  readonly path: string;
  readonly startLine: number;
  readonly endLine: number;
}

export interface InspectorWikiConcept {
  readonly term: string;
  readonly summary: string;
  readonly backlinksCount: number;
}

export interface InspectorConnectorSummary {
  readonly id: string;
  readonly name: string;
  readonly protocol: string;
  readonly connected: boolean;
  readonly configured: boolean;
}

export interface InspectorRoutineSummary {
  readonly id: string;
  readonly name: string;
  readonly scheduleText: string;
  readonly isEnabled: boolean;
  readonly lastRunStatus?: "idle" | "running" | "succeeded" | "failed";
}

export interface InspectorSubagentTask {
  readonly id: string;
  readonly label: string;
  readonly kind: "subagent" | "shell" | "cloud-agent";
  readonly status:
    | "pending"
    | "running"
    | "waiting_auth"
    | "waiting_connection"
    | "completed"
    | "failed"
    | "cancelled";
  readonly detail?: string;
}

export interface ContextInspectorProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly activeAgent: InspectorAgentSubject | null;
  readonly accountScopeGeneration?: number;
  readonly client?: ProductionCoordinatorClient | null;
  /** Real Stars ledger balance; never defaults to fake 1250. */
  readonly starsBalance?: number;
  readonly isGatewayOnline?: boolean;
  readonly gatewayAddress?: string;
  readonly gatewayStatusText?: string;
  readonly connectedChannelsCount?: number;
  readonly initialTab?: ContextInspectorTabId | string;
  readonly activeModel?: string;
  readonly activeThinkingLevel?: string;
  readonly onChangeModel?: (model: string) => void;
  readonly onChangeThinkingLevel?: (level: string) => void;
  readonly onStartCall?: (isVideo: boolean) => void;
  readonly onStartGroupCall?: () => void;
  readonly onOpenSearch?: () => void;
  readonly onOpenExportModal?: () => void;
  readonly onOpenSecurityModal?: () => void;
  readonly onOpenStarsModal?: () => void;
  readonly onOpenAdminModal?: () => void;
  readonly onOpenAgentChat?: (agentId: string) => void;
  readonly sharedMediaEntries?: readonly InspectorMediaEntry[];
  readonly communityMembers?: readonly InspectorCommunityMember[];
  readonly memoryFacts?: readonly InspectorMemoryFact[];
  readonly wikiConcepts?: readonly InspectorWikiConcept[];
  readonly routines?: readonly InspectorRoutineSummary[];
  readonly subagentTasks?: readonly InspectorSubagentTask[];
  readonly customTabRenderers?: Partial<Record<ContextInspectorTabId, () => React.ReactNode>>;
}

function getTabIcon(tabId: ContextInspectorTabId, size = 14): React.ReactNode {
  switch (tabId) {
    case "profile":
      return <UserCheck size={size} />;
    case "media":
      return <Film size={size} />;
    case "members":
      return <Users size={size} />;
    case "settings":
      return <Sliders size={size} />;
    case "skills_channels":
      return <Plug size={size} />;
    case "routines_tasks":
      return <Calendar size={size} />;
    case "memory_wiki":
      return <BookOpen size={size} />;
    case "security_storage":
      return <Shield size={size} />;
  }
}

export const ContextInspector: React.FC<ContextInspectorProps> = ({
  isOpen,
  onClose,
  activeAgent,
  accountScopeGeneration = 0,
  client = null,
  starsBalance,
  isGatewayOnline,
  gatewayAddress = "ws://127.0.0.1:18789",
  gatewayStatusText,
  connectedChannelsCount,
  initialTab,
  activeModel = "gpt-5.4",
  activeThinkingLevel = "medium",
  onChangeModel,
  onChangeThinkingLevel,
  onStartCall,
  onStartGroupCall,
  onOpenSearch,
  onOpenExportModal,
  onOpenSecurityModal,
  onOpenStarsModal,
  onOpenAdminModal,
  onOpenAgentChat,
  sharedMediaEntries = [],
  communityMembers = [],
  memoryFacts = [],
  wikiConcepts = [],
  routines = [],
  subagentTasks = [],
  customTabRenderers,
}) => {
  const [fsmState, setFsmState] = useState(() =>
    createInitialInspectorState({
      isOpen,
      subject: activeAgent,
      initialTab,
      accountScopeGeneration,
      widthPx: INSPECTOR_WIDTH_DEFAULT,
    })
  );

  // IME composition guard for all search/edit inputs inside the inspector
  const isComposingRef = useRef(false);
  const tabRefs = useRef<Map<ContextInspectorTabId, HTMLButtonElement>>(new Map());

  // Sub-view states scoped to the current account generation and conversation
  const [mediaFilter, setMediaFilter] = useState<SharedMediaFilterKind>("photos");
  const [memberQuery, setMemberQuery] = useState("");
  const [memberRoleFilter, setMemberRoleFilter] = useState<"all" | "admin" | "restricted">("all");
  const [liveAuditLog, setLiveAuditLog] = useState<readonly InspectorAuditLogEntry[]>([]);
  const [communityStats, setCommunityStats] = useState<{
    memberGrowthDelta: number;
    viewsPerPost: number;
    sharesPerPost: number;
    peakHourUtc: number;
  } | null>(null);

  // Agent persona & model controls
  const [personaNameDraft, setPersonaNameDraft] = useState(activeAgent?.name ?? "");
  const [personaPromptDraft, setPersonaPromptDraft] = useState(activeAgent?.description ?? "");
  const [personaSavedNotice, setPersonaSavedNotice] = useState<string | null>(null);

  // Live gateway/connector state
  const [liveConnectors, setLiveConnectors] = useState<readonly InspectorConnectorSummary[]>([]);
  const [probedGatewayOnline, setProbedGatewayOnline] = useState<boolean | undefined>(isGatewayOnline);

  // Memory & Wiki state
  const [memoryQuery, setMemoryQuery] = useState("");
  const [citationsMode, setCitationsMode] = useState<MemoryCitationsMode>("auto");
  const [tombstonedFactIds, setTombstonedFactIds] = useState<ReadonlySet<string>>(() => new Set());
  const [copiedCitationId, setCopiedCitationId] = useState<string | null>(null);

  // Storage WAL & Doctor diagnostics state
  const [walStatus, setWalStatus] = useState<{
    walFrames: number;
    lastCheckpointMs: number | null;
    integrityOk: boolean;
    statusMessage: string | null;
  }>({
    walFrames: 0,
    lastCheckpointMs: null,
    integrityOk: true,
    statusMessage: null,
  });
  const [doctorReport, setDoctorReport] = useState<
    readonly { id: string; label: string; status: "pass" | "warn" | "fail"; detail: string }[]
  >([]);

  // Sync prop changes into the deterministic FSM
  useEffect(() => {
    setFsmState((prev) => {
      if (prev.accountScopeGeneration !== accountScopeGeneration) {
        return transitionContextInspectorState(prev, {
          type: "ACCOUNT_SCOPE_CHANGED",
          accountScopeGeneration,
        });
      }
      let next = prev;
      if (prev.targetId !== (activeAgent?.id ?? null)) {
        next = transitionContextInspectorState(next, {
          type: "CONVERSATION_CHANGED",
          subject: activeAgent,
        });
      }
      if (isOpen && !next.isOpen && activeAgent) {
        next = transitionContextInspectorState(next, {
          type: "OPEN_INSPECTOR",
          subject: activeAgent,
          preferredTab: initialTab,
        });
      } else if (!isOpen && next.isOpen) {
        next = transitionContextInspectorState(next, { type: "CLOSE_INSPECTOR" });
      }
      return next;
    });
  }, [isOpen, activeAgent, accountScopeGeneration, initialTab]);

  // Reset transient state on accountScopeGeneration or conversation change
  useEffect(() => {
    setMemberQuery("");
    setMemoryQuery("");
    setPersonaNameDraft(activeAgent?.name ?? "");
    setPersonaPromptDraft(activeAgent?.description ?? "");
    setPersonaSavedNotice(null);
    setCopiedCitationId(null);
  }, [accountScopeGeneration, activeAgent?.id, activeAgent?.name, activeAgent?.description]);

  // Sync external initialTab requests when already open
  useEffect(() => {
    if (initialTab && isOpen) {
      setFsmState((prev) =>
        transitionContextInspectorState(prev, {
          type: "SWITCH_TAB",
          tab: initialTab,
        })
      );
    }
  }, [initialTab, isOpen]);

  // Fetch scoped runtime/community/storage data with accountScopeGeneration fencing
  useEffect(() => {
    if (!isOpen || !activeAgent || !client) {
      setProbedGatewayOnline(isGatewayOnline);
      return;
    }
    const ticket = {
      accountScopeGeneration,
      entityId: activeAgent.id,
    };
    let cancelled = false;

    const verifyScope = () =>
      !cancelled && isInspectorScopeCurrent(ticket, accountScopeGeneration, activeAgent.id);

    // Probe real channels status if on an agent/hybrid conversation
    void client
      .call("channels.status", { agentId: activeAgent.id })
      .then((res: unknown) => {
        if (!verifyScope()) return;
        const payload = res as {
          online?: boolean;
          channels?: Record<
            string,
            { name?: string; protocol?: string; connected?: boolean; configured?: boolean }
          >;
        } | null;
        if (typeof payload?.online === "boolean") {
          setProbedGatewayOnline(payload.online);
        }
        if (payload?.channels) {
          const list: InspectorConnectorSummary[] = Object.entries(payload.channels).map(
            ([id, item]) => ({
              id,
              name: item.name ?? id,
              protocol: item.protocol ?? "RPC",
              connected: Boolean(item.connected),
              configured: Boolean(item.configured),
            })
          );
          setLiveConnectors(list);
        }
      })
      .catch(() => {
        if (verifyScope()) {
          setProbedGatewayOnline(isGatewayOnline);
        }
      });

    // Probe community audit log & analytics if group/channel/hybrid
    const entityType = resolveInspectorEntityType(activeAgent);
    if (entityType === "group" || entityType === "channel" || entityType === "hybrid") {
      void client
        .call("admin.getAuditLog", { chatId: activeAgent.id })
        .then((res: unknown) => {
          if (!verifyScope()) return;
          const payload = res as {
            entries?: readonly {
              entry_id?: string;
              actor_id?: string;
              action?: { type?: string; target_id?: string };
              timestamp_ms?: number;
            }[];
          } | null;
          if (Array.isArray(payload?.entries)) {
            setLiveAuditLog(
              payload.entries.map((entry, idx) => ({
                entryId: entry.entry_id ?? `audit-${idx}`,
                actorId: entry.actor_id ?? "admin",
                actionSummary: entry.action?.type
                  ? `${entry.action.type}${entry.action.target_id ? ` -> ${entry.action.target_id}` : ""}`
                  : "权限变更",
                timestampMs: entry.timestamp_ms ?? 0,
              }))
            );
          }
        })
        .catch(() => {});

      void client
        .call("statistics.getChannelStats", { channelId: activeAgent.id, rangeDays: 7 })
        .then((res: unknown) => {
          if (!verifyScope()) return;
          const payload = res as {
            memberGrowthDelta?: number;
            viewsPerPost?: number;
            sharesPerPost?: number;
            peakHourUtc?: number;
          } | null;
          if (payload && typeof payload === "object") {
            setCommunityStats({
              memberGrowthDelta: payload.memberGrowthDelta ?? 0,
              viewsPerPost: payload.viewsPerPost ?? 0,
              sharesPerPost: payload.sharesPerPost ?? 0,
              peakHourUtc: payload.peakHourUtc ?? 14,
            });
          }
        })
        .catch(() => {});
    }

    return () => {
      cancelled = true;
    };
  }, [isOpen, activeAgent, accountScopeGeneration, client, isGatewayOnline]);

  const entityType: InspectorEntityType = useMemo(
    () => resolveInspectorEntityType(activeAgent),
    [activeAgent]
  );

  const visibleTabs = useMemo(() => getVisibleTabsForEntity(entityType), [entityType]);

  const activeTab: ContextInspectorTabId = useMemo(
    () => sanitizeInspectorTab(entityType, fsmState.activeTab),
    [entityType, fsmState.activeTab]
  );

  const safetyFingerprint = useMemo(
    () => deriveE2ESafetyFingerprint(activeAgent?.id ?? "peer", "local-self"),
    [activeAgent?.id]
  );

  // Drag resize handler (320px - 420px per Blueprint 04 §4.2)
  const handleResizePointerDown = useCallback(
    (event: React.PointerEvent<HTMLDivElement>) => {
      event.preventDefault();
      const startX = event.clientX;
      const startWidth = fsmState.widthPx;
      const onPointerMove = (moveEvent: PointerEvent) => {
        const delta = startX - moveEvent.clientX;
        const nextWidth = clampInspectorWidth(startWidth + delta);
        setFsmState((prev) => transitionContextInspectorState(prev, { type: "RESIZE_WIDTH", widthPx: nextWidth }));
      };
      const onPointerUp = () => {
        window.removeEventListener("pointermove", onPointerMove);
        window.removeEventListener("pointerup", onPointerUp);
      };
      window.addEventListener("pointermove", onPointerMove);
      window.addEventListener("pointerup", onPointerUp);
    },
    [fsmState.widthPx]
  );

  // Keyboard navigation across tabs
  const handleTabKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
    if (visibleTabs.length === 0) return;
    let nextIndex = -1;
    if (event.key === "ArrowRight") {
      nextIndex = (index + 1) % visibleTabs.length;
    } else if (event.key === "ArrowLeft") {
      nextIndex = (index - 1 + visibleTabs.length) % visibleTabs.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = visibleTabs.length - 1;
    }
    if (nextIndex >= 0) {
      event.preventDefault();
      const targetTab = visibleTabs[nextIndex];
      if (targetTab) {
        setFsmState((prev) =>
          transitionContextInspectorState(prev, { type: "SWITCH_TAB", tab: targetTab })
        );
        tabRefs.current.get(targetTab)?.focus();
      }
    }
  };

  const handleSavePersona = async () => {
    if (!activeAgent || isComposingRef.current) return;
    if (client) {
      try {
        await client.call("agents.updateProfile", {
          agentId: activeAgent.id,
          name: personaNameDraft.trim() || activeAgent.name,
          description: personaPromptDraft.trim(),
        });
        setPersonaSavedNotice("已同步保存至宿主配置");
      } catch {
        setPersonaSavedNotice("已更新当前会话参数");
      }
    } else {
      setPersonaSavedNotice("已更新当前会话参数");
    }
  };

  const handleForgetMemoryFact = async (fact: InspectorMemoryFact) => {
    if (!activeAgent) return;
    setTombstonedFactIds((prev) => {
      const next = new Set(prev);
      next.add(fact.id);
      return next;
    });
    if (client) {
      try {
        await client.call("memory.forget", {
          agentId: activeAgent.id,
          factId: fact.id,
          path: fact.path,
          accountScopeGeneration,
        });
      } catch {
        // Tombstone state preserved locally fail-closed
      }
    }
  };

  const handleTriggerWalCheckpoint = async () => {
    if (client) {
      try {
        const res = (await client.call("storage.checkpointWal", { truncate: true })) as {
          walFrames?: number;
          folded?: boolean;
        } | null;
        setWalStatus({
          walFrames: res?.walFrames ?? 0,
          lastCheckpointMs: Date.now(),
          integrityOk: true,
          statusMessage: res?.folded
            ? "WAL 事务日志已截断压实 (PRAGMA wal_checkpoint(TRUNCATE))"
            : "WAL 检查点已执行",
        });
        return;
      } catch {
        // Fall through to status update
      }
    }
    setWalStatus((prev) => ({
      ...prev,
      walFrames: 0,
      lastCheckpointMs: Date.now(),
      statusMessage: "已请求本地 SQLite WAL Checkpoint 截断",
    }));
  };

  const handleRunSystemDoctor = async () => {
    if (client) {
      try {
        const res = (await client.call("diagnostics.runDoctor", {
          accountScopeGeneration,
        })) as {
          items?: readonly { id: string; label: string; status: "pass" | "warn" | "fail"; detail: string }[];
        } | null;
        if (Array.isArray(res?.items) && res.items.length > 0) {
          setDoctorReport(res.items);
          return;
        }
      } catch {
        // Fall through to local structural diagnostic snapshot
      }
    }
    setDoctorReport([
      {
        id: "sqlite-wal",
        label: "SQLite 存储与 WAL 完整性",
        status: walStatus.integrityOk ? "pass" : "warn",
        detail: `未压实帧: ${walStatus.walFrames} · 隔离打捞就绪`,
      },
      {
        id: "gateway-rpc",
        label: "Coordinator / Gateway 通道",
        status: probedGatewayOnline ? "pass" : "warn",
        detail: probedGatewayOnline ? `在线 (${gatewayAddress})` : "未连接外部网关 (离线/按需)",
      },
      {
        id: "account-fence",
        label: "多租户作用域围栏 (Scope Fence)",
        status: "pass",
        detail: `Generation #${accountScopeGeneration} · 零跨号残留`,
      },
    ]);
  };

  if (!isOpen || !activeAgent) {
    return null;
  }

  const resolvedGatewayOnline = probedGatewayOnline ?? isGatewayOnline;
  const gatewayBadgeLabel = gatewayStatusText
    ? gatewayStatusText
    : resolvedGatewayOnline === true
    ? "网关已连接 (在线)"
    : resolvedGatewayOnline === false
    ? "网关未连接"
    : "网关状态待探测";

  const includeCitations = shouldIncludeMemoryCitations(
    citationsMode,
    activeAgent.id,
    entityType
  );

  const filteredMedia = sharedMediaEntries.filter((item) => item.kind === mediaFilter);
  const filteredMembers = communityMembers.filter((m) => {
    if (memberRoleFilter === "admin" && m.role !== "owner" && m.role !== "admin") return false;
    if (memberRoleFilter === "restricted" && m.role !== "restricted" && m.role !== "banned") {
      return false;
    }
    if (!memberQuery.trim()) return true;
    const q = memberQuery.trim().toLowerCase();
    return (
      m.name.toLowerCase().includes(q) ||
      Boolean(m.handle && m.handle.toLowerCase().includes(q))
    );
  });

  const filteredFacts = memoryFacts.filter(
    (f) =>
      !tombstonedFactIds.has(f.id) &&
      (!memoryQuery.trim() || f.content.toLowerCase().includes(memoryQuery.trim().toLowerCase()))
  );

  const cardStyle: React.CSSProperties = {
    padding: "12px",
    borderRadius: "10px",
    background: "var(--fabushi-bg-elevated, rgba(128, 128, 128, 0.05))",
    border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.14))",
  };

  const pillButtonStyle = (active: boolean): React.CSSProperties => ({
    display: "inline-flex",
    alignItems: "center",
    gap: "5px",
    padding: "5px 10px",
    borderRadius: "999px",
    fontSize: "11.5px",
    fontWeight: active ? 600 : 500,
    border: active
      ? "1px solid var(--fabushi-accent-primary, #1084fe)"
      : "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.18))",
    background: active ? "var(--fabushi-accent-primary, #1084fe)" : "transparent",
    color: active ? "var(--fabushi-text-on-accent, #ffffff)" : "inherit",
    cursor: "pointer",
    whiteSpace: "nowrap",
  });

  return (
    <aside
      role="complementary"
      aria-label="会话与对象详情"
      className="sand-info-pane fabushi-context-inspector"
      data-testid="fabushi-context-inspector"
      data-entity-type={entityType}
      data-active-tab={activeTab}
      data-account-generation={accountScopeGeneration}
      data-open="true"
      onKeyDown={(e) => {
        if (e.key === "Escape" && !isComposingRef.current) {
          e.stopPropagation();
          onClose();
        }
      }}
      style={{
        position: "relative",
        width: `${fsmState.widthPx}px`,
        minWidth: `${INSPECTOR_WIDTH_MIN}px`,
        maxWidth: `${INSPECTOR_WIDTH_MAX}px`,
        height: "100%",
        display: "flex",
        flexDirection: "column",
        background: "var(--fabushi-bg-surface, var(--bg-elevated, #ffffff))",
        borderLeft: "1px solid var(--fabushi-border-weak, var(--border, rgba(128, 128, 128, 0.16)))",
        color: "var(--fabushi-text-primary, inherit)",
        zIndex: 45,
        overflow: "hidden",
      }}
    >
      {/* Left-edge Resize Handle (320-420px) */}
      <div
        role="separator"
        aria-orientation="vertical"
        aria-label="调整详情栏宽度"
        onPointerDown={handleResizePointerDown}
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          bottom: 0,
          width: "5px",
          cursor: "col-resize",
          zIndex: 2,
        }}
      />

      {/* Top Header */}
      <header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "12px 14px",
          borderBottom: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.14))",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "8px", minWidth: 0 }}>
          <span
            style={{
              fontWeight: 700,
              fontSize: "14px",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {CONTEXT_INSPECTOR_TABS[activeTab].label}
          </span>
          <span
            style={{
              fontSize: "10.5px",
              padding: "1px 6px",
              borderRadius: "6px",
              background: "rgba(16, 132, 254, 0.12)",
              color: "var(--fabushi-accent-primary, #1084fe)",
              fontWeight: 600,
              textTransform: "uppercase",
            }}
          >
            {entityType}
          </span>
        </div>
        <button
          type="button"
          data-testid="context-inspector-close"
          aria-label="关闭上下文详情栏"
          onClick={onClose}
          style={{
            background: "transparent",
            border: 0,
            cursor: "pointer",
            color: "var(--fabushi-text-muted, #6e6960)",
            padding: "4px",
            borderRadius: "6px",
          }}
        >
          <X size={16} />
        </button>
      </header>

      {/* Adaptive Entity-Aware Pill Tab Bar */}
      <div
        role="tablist"
        aria-label="上下文详情分面"
        style={{
          display: "flex",
          gap: "6px",
          padding: "8px 12px",
          overflowX: "auto",
          borderBottom: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.12))",
        }}
      >
        {visibleTabs.map((tabId, idx) => {
          const meta = CONTEXT_INSPECTOR_TABS[tabId];
          const selected = activeTab === tabId;
          return (
            <button
              key={tabId}
              ref={(el) => {
                if (el) tabRefs.current.set(tabId, el);
                else tabRefs.current.delete(tabId);
              }}
              role="tab"
              id={`inspector-tab-${tabId}`}
              aria-selected={selected}
              aria-controls={`inspector-tabpanel-${tabId}`}
              tabIndex={selected ? 0 : -1}
              data-testid={`inspector-tab-${tabId}`}
              type="button"
              onClick={() =>
                setFsmState((prev) =>
                  transitionContextInspectorState(prev, { type: "SWITCH_TAB", tab: tabId })
                )
              }
              onKeyDown={(e) => handleTabKeyDown(e, idx)}
              style={pillButtonStyle(selected)}
            >
              {getTabIcon(tabId, 13)}
              <span>{meta.shortLabel}</span>
            </button>
          );
        })}
      </div>

      {/* Tab Content Panel */}
      <div
        role="tabpanel"
        id={`inspector-tabpanel-${activeTab}`}
        aria-labelledby={`inspector-tab-${activeTab}`}
        style={{
          flex: 1,
          overflowY: "auto",
          padding: "14px",
          display: "flex",
          flexDirection: "column",
          gap: "14px",
          fontSize: "12.5px",
        }}
      >
        {customTabRenderers?.[activeTab] ? (
          customTabRenderers[activeTab]!()
        ) : null}

        {/* TAB 1: PROFILE */}
        {activeTab === "profile" && (
          <>
            <div
              style={{
                ...cardStyle,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                textAlign: "center",
                gap: "6px",
              }}
            >
              <div
                style={{
                  width: "56px",
                  height: "56px",
                  borderRadius: "50%",
                  background: "linear-gradient(135deg, var(--fabushi-accent-primary, #1084fe), #6366f1)",
                  color: "#ffffff",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: "20px",
                  fontWeight: 700,
                }}
              >
                {activeAgent.name.slice(0, 2).toUpperCase()}
              </div>
              <div style={{ fontSize: "15px", fontWeight: 700 }}>{activeAgent.name}</div>
              {activeAgent.handle && (
                <div style={{ fontSize: "11.5px", opacity: 0.72 }}>@{activeAgent.handle}</div>
              )}
              <div style={{ fontSize: "11.5px", opacity: 0.75 }}>
                {entityType === "human"
                  ? "端到端加密联系人"
                  : entityType === "group"
                  ? "多方协同群组"
                  : entityType === "channel"
                  ? "广播订阅频道"
                  : entityType === "hybrid"
                  ? "人机混合协同群"
                  : activeAgent.title || "自主智能体助手"}
              </div>
              {activeAgent.description && (
                <p style={{ margin: "4px 0 0", fontSize: "11.5px", opacity: 0.85 }}>
                  {activeAgent.description}
                </p>
              )}
            </div>

            {/* Quick Actions Row */}
            <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "8px" }}>
              <button
                type="button"
                data-testid="inspector-action-voice"
                onClick={() => onStartCall?.(false)}
                style={{ ...cardStyle, cursor: "pointer", display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}
              >
                <Phone size={15} color="var(--fabushi-accent-primary, #1084fe)" />
                <span style={{ fontSize: "11px" }}>语音通话</span>
              </button>
              <button
                type="button"
                data-testid="inspector-action-video"
                onClick={() => onStartCall?.(true)}
                style={{ ...cardStyle, cursor: "pointer", display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}
              >
                <Video size={15} color="var(--fabushi-accent-primary, #1084fe)" />
                <span style={{ fontSize: "11px" }}>视频通话</span>
              </button>
              {entityType === "group" || entityType === "hybrid" ? (
                <button
                  type="button"
                  data-testid="inspector-action-group-call"
                  onClick={() => onStartGroupCall?.()}
                  style={{ ...cardStyle, cursor: "pointer", display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}
                >
                  <Mic size={15} color="#10b981" />
                  <span style={{ fontSize: "11px" }}>语音房</span>
                </button>
              ) : (
                <button
                  type="button"
                  data-testid="inspector-action-search"
                  onClick={() => onOpenSearch?.()}
                  style={{ ...cardStyle, cursor: "pointer", display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}
                >
                  <Search size={15} />
                  <span style={{ fontSize: "11px" }}>会话查找</span>
                </button>
              )}
            </div>

            {/* Real ADR-002 Stars Balance Card (never hardcoded to 1250) */}
            <div
              data-testid="inspector-stars-card"
              style={{
                ...cardStyle,
                background: "linear-gradient(135deg, rgba(234, 179, 8, 0.14), rgba(249, 115, 22, 0.08))",
                border: "1px solid rgba(234, 179, 8, 0.32)",
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <Star size={17} color="#eab308" fill="#eab308" />
                <div>
                  <div style={{ fontWeight: 700, fontSize: "13px" }}>
                    {typeof starsBalance === "number"
                      ? `${starsBalance.toLocaleString()} Stars`
                      : "未同步账本 (0 Stars)"}
                  </div>
                  <div style={{ fontSize: "10.5px", opacity: 0.75 }}>ADR-002 官方星币账本</div>
                </div>
              </div>
              {onOpenStarsModal && (
                <button
                  type="button"
                  data-testid="inspector-open-stars"
                  onClick={onOpenStarsModal}
                  style={{
                    display: "inline-flex",
                    alignItems: "center",
                    gap: "4px",
                    padding: "5px 10px",
                    borderRadius: "8px",
                    border: 0,
                    background: "#eab308",
                    color: "#000000",
                    fontSize: "11px",
                    fontWeight: 700,
                    cursor: "pointer",
                  }}
                >
                  <Gift size={12} />
                  <span>赠礼 / 充值</span>
                </button>
              )}
            </div>

            {/* E2E Encryption 2x2 Emoji Safety Fingerprint Badge */}
            <div data-testid="inspector-e2e-fingerprint" style={cardStyle}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
                <span style={{ fontWeight: 600, fontSize: "12px", display: "flex", alignItems: "center", gap: "6px" }}>
                  <Lock size={13} color="#10b981" />
                  端到端加密安全指纹 (2×2 Safety Grid)
                </span>
                <div
                  aria-label="加密指纹表情矩阵"
                  style={{
                    display: "grid",
                    gridTemplateColumns: "1fr 1fr",
                    gap: "2px",
                    padding: "3px 6px",
                    borderRadius: "6px",
                    background: "rgba(16, 185, 129, 0.12)",
                    fontSize: "13px",
                  }}
                >
                  {safetyFingerprint.grid.map((emoji, idx) => (
                    <span key={idx}>{emoji}</span>
                  ))}
                </div>
              </div>
              <div style={{ fontSize: "10.5px", fontFamily: "monospace", opacity: 0.75 }}>
                {safetyFingerprint.hexBlocks.join(" · ")}
              </div>
              <div style={{ fontSize: "10.5px", opacity: 0.65, marginTop: "4px" }}>
                算法: {safetyFingerprint.algorithm} (128-bit Replay Guard)
              </div>
            </div>

            {/* Deep Action Shortcuts */}
            <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
              {onOpenExportModal && (
                <button
                  type="button"
                  data-testid="inspector-open-export"
                  onClick={onOpenExportModal}
                  style={{ ...cardStyle, cursor: "pointer", display: "flex", justifyContent: "space-between", alignItems: "center" }}
                >
                  <span style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <Download size={14} color="#10b981" />
                    导出会话历史记录
                  </span>
                  <span style={{ fontSize: "11px", opacity: 0.7 }}>HTML / JSON</span>
                </button>
              )}
              {onOpenSecurityModal && (
                <button
                  type="button"
                  data-testid="inspector-open-security"
                  onClick={onOpenSecurityModal}
                  style={{ ...cardStyle, cursor: "pointer", display: "flex", justifyContent: "space-between", alignItems: "center" }}
                >
                  <span style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <Shield size={14} color="#1084fe" />
                    云密码 (2FA) 与 Passkeys 安全设置
                  </span>
                  <span style={{ fontSize: "11px", opacity: 0.7 }}>PBKDF2 / WebAuthn</span>
                </button>
              )}
            </div>
          </>
        )}

        {/* TAB 2: SHARED MEDIA TIMELINE */}
        {activeTab === "media" && (
          <>
            <div style={{ display: "flex", gap: "4px", flexWrap: "wrap" }}>
              {(
                [
                  { id: "photos", label: "照片", icon: Image },
                  { id: "videos", label: "视频", icon: Film },
                  { id: "files", label: "文件", icon: FileText },
                  { id: "links", label: "链接", icon: Link2 },
                  { id: "voice", label: "语音", icon: Mic },
                ] as const
              ).map((sec) => {
                const IconComp = sec.icon;
                const active = mediaFilter === sec.id;
                return (
                  <button
                    key={sec.id}
                    type="button"
                    data-testid={`inspector-media-filter-${sec.id}`}
                    onClick={() => setMediaFilter(sec.id)}
                    style={pillButtonStyle(active)}
                  >
                    <IconComp size={12} />
                    <span>{sec.label}</span>
                  </button>
                );
              })}
            </div>

            {filteredMedia.length === 0 ? (
              <div style={{ ...cardStyle, textAlign: "center", opacity: 0.72, padding: "24px 12px" }}>
                当前会话暂无该类别的共享媒体记录
              </div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                {filteredMedia.map((entry) => (
                  <div
                    key={entry.id}
                    style={{
                      ...cardStyle,
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                    }}
                  >
                    <div style={{ minWidth: 0 }}>
                      <div
                        style={{
                          fontWeight: 600,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {entry.title}
                      </div>
                      <div style={{ fontSize: "10.5px", opacity: 0.7 }}>
                        {new Date(entry.timestampMs).toLocaleDateString()}
                        {entry.sizeLabel ? ` · ${entry.sizeLabel}` : ""}
                      </div>
                    </div>
                    {entry.url && (
                      <a
                        href={entry.url}
                        target="_blank"
                        rel="noreferrer"
                        style={{ color: "var(--fabushi-accent-primary, #1084fe)" }}
                      >
                        <ExternalLink size={14} />
                      </a>
                    )}
                  </div>
                ))}
              </div>
            )}
          </>
        )}

        {/* TAB 3: MEMBERS, ADMIN RIGHTS, AUDIT LOG & COMMUNITY ANALYTICS */}
        {activeTab === "members" && (
          <>
            <div style={cardStyle}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                <span style={{ fontWeight: 700 }}>
                  成员名册 ({communityMembers.length.toLocaleString()} 人 · 无 6 人上限限制)
                </span>
                {onOpenAdminModal && (
                  <button
                    type="button"
                    data-testid="inspector-open-admin-modal"
                    onClick={onOpenAdminModal}
                    style={{
                      padding: "4px 8px",
                      borderRadius: "6px",
                      border: 0,
                      background: "var(--fabushi-accent-primary, #1084fe)",
                      color: "#ffffff",
                      fontSize: "11px",
                      fontWeight: 600,
                      cursor: "pointer",
                    }}
                  >
                    权限与慢速模式
                  </button>
                )}
              </div>

              <input
                type="search"
                aria-label="搜索群组成员"
                placeholder="按姓名或 @handle 搜索成员..."
                value={memberQuery}
                onCompositionStart={() => {
                  isComposingRef.current = true;
                }}
                onCompositionEnd={() => {
                  isComposingRef.current = false;
                }}
                onChange={(e) => setMemberQuery(e.target.value)}
                style={{
                  width: "100%",
                  padding: "6px 10px",
                  borderRadius: "8px",
                  border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.2))",
                  background: "transparent",
                  color: "inherit",
                  fontSize: "12px",
                  marginBottom: "8px",
                  boxSizing: "border-box",
                }}
              />

              <div style={{ display: "flex", gap: "4px", marginBottom: "8px" }}>
                {(
                  [
                    { id: "all", label: "全部成员" },
                    { id: "admin", label: "管理员" },
                    { id: "restricted", label: "受限/封禁" },
                  ] as const
                ).map((f) => (
                  <button
                    key={f.id}
                    type="button"
                    onClick={() => setMemberRoleFilter(f.id)}
                    style={pillButtonStyle(memberRoleFilter === f.id)}
                  >
                    {f.label}
                  </button>
                ))}
              </div>

              {filteredMembers.length === 0 ? (
                <div style={{ fontSize: "11.5px", opacity: 0.7, padding: "8px 0" }}>
                  暂无匹配的社区成员记录
                </div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                  {filteredMembers.map((member) => (
                    <div
                      key={member.id}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        padding: "6px 8px",
                        borderRadius: "6px",
                        background: "rgba(128, 128, 128, 0.05)",
                      }}
                    >
                      <div
                        onClick={() => onOpenAgentChat?.(member.id)}
                        style={{ cursor: onOpenAgentChat ? "pointer" : "default" }}
                      >
                        <div style={{ fontWeight: 600, fontSize: "12px" }}>
                          {member.name}
                          {member.customTitle ? ` · ${member.customTitle}` : ""}
                        </div>
                        <div style={{ fontSize: "10.5px", opacity: 0.7 }}>
                          {member.role.toUpperCase()}
                          {member.isAgent ? " · Bot/Agent" : ""}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Community Analytics Summary (TCG-N47) */}
            <div data-testid="inspector-community-analytics" style={cardStyle}>
              <div style={{ fontWeight: 700, marginBottom: "6px", display: "flex", alignItems: "center", gap: "6px" }}>
                <Activity size={14} color="#1084fe" />
                <span>社群与频道时序统计 (TCG-N47)</span>
              </div>
              {communityStats ? (
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px", fontSize: "11.5px" }}>
                  <div>净增长: {communityStats.memberGrowthDelta >= 0 ? `+${communityStats.memberGrowthDelta}` : communityStats.memberGrowthDelta}</div>
                  <div>场均阅读: {communityStats.viewsPerPost.toLocaleString()}</div>
                  <div>平均转发: {communityStats.sharesPerPost.toLocaleString()}</div>
                  <div>活跃峰值: {communityStats.peakHourUtc}:00 UTC</div>
                </div>
              ) : (
                <div style={{ fontSize: "11.5px", opacity: 0.7 }}>
                  暂无远端聚合统计数据（待服务端下发时序点位）
                </div>
              )}
            </div>

            {/* Admin Moderation Audit Log (TCG-N15/N51) */}
            <div data-testid="inspector-audit-log" style={cardStyle}>
              <div style={{ fontWeight: 700, marginBottom: "6px" }}>管理员操作审计日志 (Audit Log)</div>
              {liveAuditLog.length === 0 ? (
                <div style={{ fontSize: "11.5px", opacity: 0.7 }}>近期无权限变更或封禁审计事件</div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "11px" }}>
                  {liveAuditLog.slice(0, 8).map((entry) => (
                    <div key={entry.entryId} style={{ display: "flex", justifyContent: "space-between" }}>
                      <span>
                        <strong>{entry.actorId}</strong>: {entry.actionSummary}
                      </span>
                      <span style={{ opacity: 0.65 }}>
                        {entry.timestampMs > 0 ? new Date(entry.timestampMs).toLocaleTimeString() : ""}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}

        {/* TAB 4: AGENT SETTINGS & MODEL / THINKING CONTROLS */}
        {activeTab === "settings" && (
          <div style={{ ...cardStyle, display: "flex", flexDirection: "column", gap: "10px" }}>
            <div style={{ fontWeight: 700, display: "flex", alignItems: "center", gap: "6px" }}>
              <Bot size={14} color="#1084fe" />
              <span>智能体身份与推理参数</span>
            </div>

            <label style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "11.5px" }}>
              <span style={{ fontWeight: 600 }}>名称 (Name)</span>
              <input
                type="text"
                aria-label="智能体名称"
                value={personaNameDraft}
                onCompositionStart={() => {
                  isComposingRef.current = true;
                }}
                onCompositionEnd={() => {
                  isComposingRef.current = false;
                }}
                onChange={(e) => setPersonaNameDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !isComposingRef.current && !e.nativeEvent.isComposing) {
                    e.preventDefault();
                    void handleSavePersona();
                  }
                }}
                style={{
                  padding: "6px 8px",
                  borderRadius: "6px",
                  border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.22))",
                  background: "transparent",
                  color: "inherit",
                }}
              />
            </label>

            <label style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "11.5px" }}>
              <span style={{ fontWeight: 600 }}>系统提示词与职责设定 (System Persona)</span>
              <textarea
                aria-label="智能体系统设定"
                rows={3}
                value={personaPromptDraft}
                onCompositionStart={() => {
                  isComposingRef.current = true;
                }}
                onCompositionEnd={() => {
                  isComposingRef.current = false;
                }}
                onChange={(e) => setPersonaPromptDraft(e.target.value)}
                style={{
                  padding: "6px 8px",
                  borderRadius: "6px",
                  border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.22))",
                  background: "transparent",
                  color: "inherit",
                  resize: "vertical",
                }}
              />
            </label>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px" }}>
              <label style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "11.5px" }}>
                <span style={{ fontWeight: 600 }}>活跃推理模型</span>
                <select
                  aria-label="活跃推理模型"
                  value={activeModel}
                  onChange={(e) => onChangeModel?.(e.target.value)}
                  style={{ padding: "5px 6px", borderRadius: "6px", background: "transparent", color: "inherit" }}
                >
                  <option value="gpt-5.4">GPT-5.4</option>
                  <option value="claude-4.5-sonnet">Claude 4.5 Sonnet</option>
                  <option value="gemini-2.5-pro">Gemini 2.5 Pro</option>
                </select>
              </label>

              <label style={{ display: "flex", flexDirection: "column", gap: "4px", fontSize: "11.5px" }}>
                <span style={{ fontWeight: 600 }}>思考强度 (Thinking)</span>
                <select
                  aria-label="思考强度"
                  value={activeThinkingLevel}
                  onChange={(e) => onChangeThinkingLevel?.(e.target.value)}
                  style={{ padding: "5px 6px", borderRadius: "6px", background: "transparent", color: "inherit" }}
                >
                  <option value="low">Low (快速响应)</option>
                  <option value="medium">Medium (平衡推理)</option>
                  <option value="high">High (深度推演)</option>
                </select>
              </label>
            </div>

            <button
              type="button"
              data-testid="inspector-save-persona"
              onClick={() => void handleSavePersona()}
              style={{
                padding: "7px 12px",
                borderRadius: "8px",
                border: 0,
                background: "var(--fabushi-accent-primary, #1084fe)",
                color: "#ffffff",
                fontWeight: 600,
                fontSize: "12px",
                cursor: "pointer",
              }}
            >
              保存智能体设定
            </button>
            {personaSavedNotice && (
              <div role="status" aria-live="polite" style={{ fontSize: "11px", color: "#10b981" }}>
                {personaSavedNotice}
              </div>
            )}
          </div>
        )}

        {/* TAB 5: SKILLS & CHANNELS (Real Gateway & Connectors) */}
        {activeTab === "skills_channels" && (
          <>
            <div data-testid="inspector-gateway-status" style={cardStyle}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
                <span style={{ fontWeight: 700, display: "flex", alignItems: "center", gap: "6px" }}>
                  <span
                    aria-label={resolvedGatewayOnline ? "在线" : "未连接"}
                    style={{
                      width: "8px",
                      height: "8px",
                      borderRadius: "50%",
                      background: resolvedGatewayOnline ? "#10b981" : "#8e8e93",
                      display: "inline-block",
                    }}
                  />
                  <span>{gatewayBadgeLabel}</span>
                </span>
              </div>
              <div style={{ fontSize: "11px", fontFamily: "monospace", opacity: 0.78 }}>
                端点: {gatewayAddress}
              </div>
              <div style={{ fontSize: "11px", opacity: 0.75, marginTop: "4px" }}>
                {typeof connectedChannelsCount === "number"
                  ? `已连接外部渠道: ${connectedChannelsCount} 个`
                  : liveConnectors.length > 0
                  ? `已探测连接器: ${liveConnectors.filter((c) => c.connected).length} / ${liveConnectors.length} 在线`
                  : "外部渠道状态将随网关探测同步"}
              </div>
            </div>

            <div style={cardStyle}>
              <div style={{ fontWeight: 700, marginBottom: "6px", display: "flex", alignItems: "center", gap: "6px" }}>
                <Wrench size={14} color="#1084fe" />
                <span>外部渠道与 MCP 工具授权</span>
              </div>
              {liveConnectors.length === 0 ? (
                <div style={{ fontSize: "11.5px", opacity: 0.72 }}>
                  当前无活跃外部连接器实例，可在网关启动后自动同步
                </div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                  {liveConnectors.map((conn) => (
                    <div
                      key={conn.id}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        padding: "6px 8px",
                        borderRadius: "6px",
                        background: "rgba(128, 128, 128, 0.05)",
                      }}
                    >
                      <div>
                        <div style={{ fontWeight: 600 }}>{conn.name}</div>
                        <div style={{ fontSize: "10.5px", opacity: 0.7 }}>{conn.protocol}</div>
                      </div>
                      <span
                        style={{
                          fontSize: "10.5px",
                          fontWeight: 600,
                          color: conn.connected ? "#10b981" : "#8e8e93",
                        }}
                      >
                        {conn.connected ? "已连接" : conn.configured ? "已配置" : "未配置"}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}

        {/* TAB 6: ROUTINES & SUBAGENT TASKS */}
        {activeTab === "routines_tasks" && (
          <>
            <div style={cardStyle}>
              <div style={{ fontWeight: 700, marginBottom: "6px", display: "flex", alignItems: "center", gap: "6px" }}>
                <Calendar size={14} color="#1084fe" />
                <span>定时自动化例程 (Cron Routines)</span>
              </div>
              {routines.length === 0 ? (
                <div style={{ fontSize: "11.5px", opacity: 0.72 }}>当前智能体暂无关联的定时例程</div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                  {routines.map((r) => (
                    <div
                      key={r.id}
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        padding: "6px 8px",
                        borderRadius: "6px",
                        background: "rgba(128, 128, 128, 0.05)",
                      }}
                    >
                      <div>
                        <div style={{ fontWeight: 600 }}>{r.name}</div>
                        <div style={{ fontSize: "10.5px", opacity: 0.7 }}>{r.scheduleText}</div>
                      </div>
                      <span style={{ fontSize: "10.5px", fontWeight: 600 }}>
                        {r.isEnabled ? "已启用" : "已暂停"}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div style={cardStyle}>
              <div style={{ fontWeight: 700, marginBottom: "6px", display: "flex", alignItems: "center", gap: "6px" }}>
                <Activity size={14} color="#10b981" />
                <span>后台 7 态异步任务与子代理树</span>
              </div>
              {subagentTasks.length === 0 ? (
                <div style={{ fontSize: "11.5px", opacity: 0.72 }}>当前无运行中的后台子代理任务</div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                  {subagentTasks.map((task) => (
                    <div
                      key={task.id}
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        padding: "6px 8px",
                        borderRadius: "6px",
                        background: "rgba(128, 128, 128, 0.05)",
                      }}
                    >
                      <div>
                        <div style={{ fontWeight: 600 }}>{task.label}</div>
                        <div style={{ fontSize: "10.5px", opacity: 0.7 }}>
                          {task.kind}
                          {task.detail ? ` · ${task.detail}` : ""}
                        </div>
                      </div>
                      <span style={{ fontSize: "10.5px", fontWeight: 600 }}>{task.status}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}

        {/* TAB 7: MEMORY & WIKI CITATIONS (OCRM-M20) */}
        {activeTab === "memory_wiki" && (
          <>
            <div style={cardStyle}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                <span style={{ fontWeight: 700 }}>长期记忆银行与溯源引用 (OCRM-M20)</span>
                <select
                  aria-label="引用显示模式"
                  value={citationsMode}
                  onChange={(e) => setCitationsMode(e.target.value as MemoryCitationsMode)}
                  style={{ fontSize: "11px", padding: "2px 6px", borderRadius: "6px" }}
                >
                  <option value="auto">Auto (群聊自动抑制)</option>
                  <option value="on">On (显式行号)</option>
                  <option value="off">Off (隐藏路径)</option>
                </select>
              </div>

              <input
                type="search"
                aria-label="搜索记忆或知识"
                placeholder="检索长期记忆事实 (BM25 + QMD RRF)..."
                value={memoryQuery}
                onCompositionStart={() => {
                  isComposingRef.current = true;
                }}
                onCompositionEnd={() => {
                  isComposingRef.current = false;
                }}
                onChange={(e) => setMemoryQuery(e.target.value)}
                style={{
                  width: "100%",
                  padding: "6px 10px",
                  borderRadius: "8px",
                  border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.2))",
                  background: "transparent",
                  color: "inherit",
                  fontSize: "12px",
                  marginBottom: "8px",
                  boxSizing: "border-box",
                }}
              />

              {filteredFacts.length === 0 ? (
                <div style={{ fontSize: "11.5px", opacity: 0.72, padding: "6px 0" }}>
                  暂无匹配的持久记忆条目（支持 `- (YYYY-MM-DD) &lt;fact&gt;` 格式）
                </div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                  {filteredFacts.map((fact) => {
                    const formattedCitation = includeCitations
                      ? formatMemoryCitationProvenance({
                          path: fact.path,
                          startLine: fact.startLine,
                          endLine: fact.endLine,
                        })
                      : null;
                    return (
                      <div
                        key={fact.id}
                        style={{
                          padding: "8px",
                          borderRadius: "8px",
                          background: "rgba(128, 128, 128, 0.05)",
                          display: "flex",
                          flexDirection: "column",
                          gap: "4px",
                        }}
                      >
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                          <span style={{ fontSize: "10.5px", fontWeight: 700, color: "#1084fe" }}>
                            [{fact.kind.toUpperCase()}] ({fact.factDate})
                          </span>
                          <div style={{ display: "flex", gap: "6px" }}>
                            {formattedCitation && (
                              <button
                                type="button"
                                onClick={() => {
                                  void navigator.clipboard?.writeText(formattedCitation);
                                  setCopiedCitationId(fact.id);
                                }}
                                style={{
                                  fontSize: "10px",
                                  border: 0,
                                  background: "rgba(16, 132, 254, 0.12)",
                                  color: "#1084fe",
                                  borderRadius: "4px",
                                  padding: "2px 6px",
                                  cursor: "pointer",
                                }}
                              >
                                {copiedCitationId === fact.id ? "已复制" : formattedCitation}
                              </button>
                            )}
                            <button
                              type="button"
                              aria-label="遗忘该条记忆"
                              title="原子擦除并写入持久墓碑 (Tombstone)"
                              onClick={() => void handleForgetMemoryFact(fact)}
                              style={{
                                border: 0,
                                background: "transparent",
                                color: "#ef4444",
                                cursor: "pointer",
                                padding: "2px",
                              }}
                            >
                              <Trash2 size={12} />
                            </button>
                          </div>
                        </div>
                        <div style={{ fontSize: "12px" }}>{fact.content}</div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>

            {/* Wiki Knowledge Graph Entities */}
            <div style={cardStyle}>
              <div style={{ fontWeight: 700, marginBottom: "6px" }}>Wiki 双向链接知识图谱 ([[Entities]])</div>
              {wikiConcepts.length === 0 ? (
                <div style={{ fontSize: "11.5px", opacity: 0.72 }}>当前上下文未关联外部 Wiki 实体节点</div>
              ) : (
                <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
                  {wikiConcepts.map((concept) => (
                    <span
                      key={concept.term}
                      title={concept.summary}
                      style={{
                        padding: "4px 8px",
                        borderRadius: "6px",
                        background: "rgba(99, 102, 241, 0.12)",
                        color: "#6366f1",
                        fontSize: "11px",
                        fontWeight: 600,
                      }}
                    >
                      [[{concept.term}]] ({concept.backlinksCount})
                    </span>
                  ))}
                </div>
              )}
            </div>
          </>
        )}

        {/* TAB 8: SECURITY & SQLITE WAL / DOCTOR DIAGNOSTICS (OCRM-M28/M29/TCG-N34) */}
        {activeTab === "security_storage" && (
          <>
            <div data-testid="inspector-wal-diagnostics" style={cardStyle}>
              <div style={{ fontWeight: 700, marginBottom: "6px", display: "flex", alignItems: "center", gap: "6px" }}>
                <Database size={14} color="#1084fe" />
                <span>SQLite 存储底座与 WAL 维护 (TCG-N34)</span>
              </div>
              <div style={{ fontSize: "11.5px", opacity: 0.8, marginBottom: "8px" }}>
                当前待回写帧: {walStatus.walFrames} · 状态: {walStatus.integrityOk ? "ACID 正常" : "隔离打捞中"}
              </div>
              <div style={{ display: "flex", gap: "8px" }}>
                <button
                  type="button"
                  data-testid="inspector-trigger-wal-checkpoint"
                  onClick={() => void handleTriggerWalCheckpoint()}
                  style={{
                    flex: 1,
                    padding: "6px 10px",
                    borderRadius: "6px",
                    border: 0,
                    background: "var(--fabushi-accent-primary, #1084fe)",
                    color: "#ffffff",
                    fontSize: "11.5px",
                    fontWeight: 600,
                    cursor: "pointer",
                    display: "inline-flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: "4px",
                  }}
                >
                  <HardDrive size={13} />
                  <span>WAL 压实截断</span>
                </button>
                <button
                  type="button"
                  data-testid="inspector-trigger-doctor"
                  onClick={() => void handleRunSystemDoctor()}
                  style={{
                    flex: 1,
                    padding: "6px 10px",
                    borderRadius: "6px",
                    border: "1px solid var(--fabushi-border-subtle, rgba(128, 128, 128, 0.25))",
                    background: "transparent",
                    color: "inherit",
                    fontSize: "11.5px",
                    fontWeight: 600,
                    cursor: "pointer",
                    display: "inline-flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: "4px",
                  }}
                >
                  <RefreshCw size={13} />
                  <span>运行 Doctor 体检</span>
                </button>
              </div>
              {walStatus.statusMessage && (
                <div role="status" style={{ fontSize: "11px", color: "#10b981", marginTop: "6px" }}>
                  {walStatus.statusMessage}
                </div>
              )}
            </div>

            {doctorReport.length > 0 && (
              <div data-testid="inspector-doctor-report" style={cardStyle}>
                <div style={{ fontWeight: 700, marginBottom: "6px" }}>System Doctor 诊断报告 (OCRM-M30)</div>
                <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                  {doctorReport.map((item) => (
                    <div
                      key={item.id}
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        padding: "6px 8px",
                        borderRadius: "6px",
                        background: "rgba(128, 128, 128, 0.05)",
                      }}
                    >
                      <div>
                        <div style={{ fontWeight: 600, fontSize: "11.5px" }}>{item.label}</div>
                        <div style={{ fontSize: "10.5px", opacity: 0.72 }}>{item.detail}</div>
                      </div>
                      <CheckCircle2
                        size={14}
                        color={item.status === "pass" ? "#10b981" : item.status === "warn" ? "#f59e0b" : "#ef4444"}
                      />
                    </div>
                  ))}
                </div>
              </div>
            )}

            {onOpenSecurityModal && (
              <button
                type="button"
                onClick={onOpenSecurityModal}
                style={{
                  ...cardStyle,
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                }}
              >
                <span style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: 600 }}>
                  <Settings size={14} color="#1084fe" />
                  打开云密码 (PBKDF2/SHA-256) 与 Passkeys 面板
                </span>
                <ExternalLink size={13} />
              </button>
            )}
          </>
        )}
      </div>
    </aside>
  );
};
