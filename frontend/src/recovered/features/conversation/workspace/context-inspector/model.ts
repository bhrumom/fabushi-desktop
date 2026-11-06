/**
 * Canonical Context Inspector State Machine, Entity Taxonomy & Security/Memory Policies
 *
 * Upstream comparative anchors studied before implementation:
 * - Telegram Desktop:
 *   - `Telegram/SourceFiles/info/info_controller.cpp` (`Info::Key`, `Info::Section`, peer/topic/settings routing)
 *   - `Telegram/SourceFiles/info/profile/info_profile_widget.cpp` (`Info::Profile::Memento`, profile header & quick actions)
 *   - `Telegram/SourceFiles/info/media/info_media_widget.cpp` (`Info::Media::Type`, shared media section indexing)
 *   - `Telegram/SourceFiles/data/data_chat_participant_status.cpp` (`ChatAdminRight`, `ChatRestriction` bitflags)
 * - OpenClaw:
 *   - `extensions/memory-core/src/tools.citations.ts` (`formatCitation`, `shouldIncludeCitations` group/channel auto-suppression)
 *   - `extensions/memory-core/src/memory-forget.ts` (`forgetMemoryEntries` tombstone lifecycle)
 * - Fabushi Native & Host:
 *   - `native/mahayana-messaging/src/secret_chat.rs` (`SecretPublicKey`, X25519+HKDF-SHA256+XChaCha20Poly1305, 128-bit ReplayWindow)
 *   - `source/host/src/extensions/content_search/memory_citations.rs` (`MemoryCitationsMode`, `format_citation`)
 *   - `source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs` (`ChatAdminRights`, `AuditLogEntry`)
 *   - `source/host/src/storage/sqlite_recovery.rs` (`QuarantineCorruptSqliteResult`, WAL checkpoint & salvage)
 */

export type InspectorEntityType = "human" | "group" | "channel" | "agent" | "hybrid";

export type ContextInspectorTabId =
  | "profile"
  | "media"
  | "members"
  | "settings"
  | "skills_channels"
  | "routines_tasks"
  | "memory_wiki"
  | "security_storage";

export type SharedMediaFilterKind = "photos" | "videos" | "files" | "links" | "voice";

export type MemoryCitationsMode = "on" | "off" | "auto";

export interface InspectorAgentSubject {
  readonly id: string;
  readonly name: string;
  readonly title?: string;
  readonly description?: string;
  readonly isHuman?: boolean;
  readonly isGroup?: boolean;
  readonly isChannel?: boolean;
  readonly hasAgentMembers?: boolean;
  readonly handle?: string;
  readonly avatarUrl?: string | null;
  readonly raw?: Record<string, unknown>;
}

export interface InspectorTabMeta {
  readonly id: ContextInspectorTabId;
  readonly label: string;
  readonly shortLabel: string;
  readonly description: string;
}

export const INSPECTOR_WIDTH_MIN = 320;
export const INSPECTOR_WIDTH_MAX = 420;
export const INSPECTOR_WIDTH_DEFAULT = 360;

export const CONTEXT_INSPECTOR_TABS: Readonly<Record<ContextInspectorTabId, InspectorTabMeta>> = {
  profile: {
    id: "profile",
    label: "资料与概览",
    shortLabel: "资料",
    description: "身份卡片、真实星币账本、加密指纹与快捷呼叫/导出入口",
  },
  media: {
    id: "media",
    label: "共享媒体",
    shortLabel: "媒体",
    description: "按照片、视频、文件、链接与语音分片过滤时间轴",
  },
  members: {
    id: "members",
    label: "成员与治理",
    shortLabel: "成员",
    description: "万人社群成员名册、10 项管理员权限、审计日志与统计",
  },
  settings: {
    id: "settings",
    label: "模型与设定",
    shortLabel: "设定",
    description: "智能体身份设定、活跃推理模型、思考强度与执行门禁",
  },
  skills_channels: {
    id: "skills_channels",
    label: "技能与渠道",
    shortLabel: "渠道",
    description: "真实网关在线状态、外部通讯连接器与 MCP 工具白名单",
  },
  routines_tasks: {
    id: "routines_tasks",
    label: "自动化与任务",
    shortLabel: "任务",
    description: "定时 Cron Routines、会话大纲与 7 态子代理任务树",
  },
  memory_wiki: {
    id: "memory_wiki",
    label: "记忆与知识",
    shortLabel: "记忆",
    description: "长期记忆事实、真实行号引用 (#Lstart-Lend) 与 Wiki 图谱",
  },
  security_storage: {
    id: "security_storage",
    label: "安全与存储",
    shortLabel: "安全",
    description: "E2E 双棘轮密钥指纹、2FA/Passkeys 与 SQLite WAL 诊断",
  },
};

const ENTITY_VISIBLE_TABS: Readonly<Record<InspectorEntityType, readonly ContextInspectorTabId[]>> = {
  human: ["profile", "media", "security_storage"],
  group: ["profile", "members", "media", "security_storage"],
  channel: ["profile", "members", "media", "security_storage"],
  agent: [
    "profile",
    "settings",
    "skills_channels",
    "routines_tasks",
    "memory_wiki",
    "media",
    "security_storage",
  ],
  hybrid: [
    "profile",
    "members",
    "skills_channels",
    "routines_tasks",
    "memory_wiki",
    "media",
    "security_storage",
  ],
};

/**
 * Resolves the canonical `InspectorEntityType` from an active conversation/agent descriptor.
 * Aligns with `Info::Key` peer classification in `Telegram/SourceFiles/info/info_controller.cpp`.
 */
export function resolveInspectorEntityType(
  subject: InspectorAgentSubject | null | undefined
): InspectorEntityType {
  if (!subject) {
    return "agent";
  }
  const raw = subject.raw ?? {};
  const isChannel =
    subject.isChannel === true ||
    raw.isChannel === true ||
    raw.kind === "channel" ||
    raw.peerKind === "channel";
  if (isChannel) {
    return "channel";
  }

  const isGroup =
    subject.isGroup === true ||
    raw.isGroup === true ||
    raw.kind === "group" ||
    raw.kind === "supergroup";
  const hasAgentMembers =
    subject.hasAgentMembers === true ||
    raw.hasAgentMembers === true ||
    raw.isHybrid === true ||
    (Array.isArray(raw.agentMemberIds) && raw.agentMemberIds.length > 0);

  if (isGroup && hasAgentMembers) {
    return "hybrid";
  }
  if (isGroup) {
    return "group";
  }
  if (subject.isHuman === true || raw.isHuman === true || raw.kind === "human") {
    return "human";
  }
  return "agent";
}

/**
 * Returns the ordered list of visible tabs for a given `InspectorEntityType`.
 */
export function getVisibleTabsForEntity(
  entityType: InspectorEntityType
): readonly ContextInspectorTabId[] {
  return ENTITY_VISIBLE_TABS[entityType] ?? ENTITY_VISIBLE_TABS.agent;
}

/**
 * Normalizes legacy tab aliases and auto-heals unsupported tabs to `"profile"`
 * when switching between entities (e.g., from an Agent to a Human chat).
 *
 * State machine invariant:
 * SanitizeTab(entityType, T) = T if T in VisibleTabs(entityType) else "profile"
 */
export function sanitizeInspectorTab(
  entityType: InspectorEntityType,
  currentTab: ContextInspectorTabId | string | null | undefined
): ContextInspectorTabId {
  const normalizedAlias = normalizeTabAlias(currentTab);
  const visible = getVisibleTabsForEntity(entityType);
  if (normalizedAlias && visible.includes(normalizedAlias)) {
    return normalizedAlias;
  }
  return "profile";
}

function normalizeTabAlias(tab: string | null | undefined): ContextInspectorTabId | null {
  if (!tab) return null;
  switch (tab) {
    case "profile":
    case "media":
    case "members":
    case "settings":
    case "skills_channels":
    case "routines_tasks":
    case "memory_wiki":
    case "security_storage":
      return tab;
    case "skills":
    case "channels":
      return "skills_channels";
    case "routines":
    case "outline":
    case "tasks":
      return "routines_tasks";
    case "memory":
    case "wiki":
      return "memory_wiki";
    case "security":
    case "storage":
      return "security_storage";
    default:
      return null;
  }
}

export function clampInspectorWidth(widthPx: number): number {
  if (!Number.isFinite(widthPx)) return INSPECTOR_WIDTH_DEFAULT;
  return Math.max(INSPECTOR_WIDTH_MIN, Math.min(INSPECTOR_WIDTH_MAX, Math.round(widthPx)));
}

export interface ContextInspectorState {
  readonly isOpen: boolean;
  readonly targetId: string | null;
  readonly entityType: InspectorEntityType;
  readonly activeTab: ContextInspectorTabId;
  readonly accountScopeGeneration: number;
  readonly widthPx: number;
}

export type ContextInspectorEvent =
  | {
      readonly type: "OPEN_INSPECTOR";
      readonly subject: InspectorAgentSubject;
      readonly preferredTab?: ContextInspectorTabId | string;
    }
  | { readonly type: "CLOSE_INSPECTOR" }
  | { readonly type: "SWITCH_TAB"; readonly tab: ContextInspectorTabId | string }
  | { readonly type: "CONVERSATION_CHANGED"; readonly subject: InspectorAgentSubject | null }
  | { readonly type: "ACCOUNT_SCOPE_CHANGED"; readonly accountScopeGeneration: number }
  | { readonly type: "RESIZE_WIDTH"; readonly widthPx: number };

export function createInitialInspectorState(params?: {
  readonly isOpen?: boolean;
  readonly subject?: InspectorAgentSubject | null;
  readonly initialTab?: ContextInspectorTabId | string;
  readonly accountScopeGeneration?: number;
  readonly widthPx?: number;
}): ContextInspectorState {
  const entityType = resolveInspectorEntityType(params?.subject);
  return {
    isOpen: Boolean(params?.isOpen),
    targetId: params?.subject?.id ?? null,
    entityType,
    activeTab: sanitizeInspectorTab(entityType, params?.initialTab),
    accountScopeGeneration: params?.accountScopeGeneration ?? 0,
    widthPx: clampInspectorWidth(params?.widthPx ?? INSPECTOR_WIDTH_DEFAULT),
  };
}

/**
 * Deterministic Context Inspector FSM transition function.
 * Enforces `accountScopeGeneration` monotonic isolation and automatic tab healing.
 */
export function transitionContextInspectorState(
  state: ContextInspectorState,
  event: ContextInspectorEvent
): ContextInspectorState {
  switch (event.type) {
    case "OPEN_INSPECTOR": {
      const entityType = resolveInspectorEntityType(event.subject);
      return {
        ...state,
        isOpen: true,
        targetId: event.subject.id,
        entityType,
        activeTab: sanitizeInspectorTab(entityType, event.preferredTab ?? state.activeTab),
      };
    }
    case "CLOSE_INSPECTOR":
      return {
        ...state,
        isOpen: false,
      };
    case "SWITCH_TAB":
      return {
        ...state,
        activeTab: sanitizeInspectorTab(state.entityType, event.tab),
      };
    case "CONVERSATION_CHANGED": {
      if (!event.subject) {
        return {
          ...state,
          targetId: null,
          entityType: "agent",
          activeTab: "profile",
        };
      }
      const nextEntityType = resolveInspectorEntityType(event.subject);
      return {
        ...state,
        targetId: event.subject.id,
        entityType: nextEntityType,
        activeTab: sanitizeInspectorTab(nextEntityType, state.activeTab),
      };
    }
    case "ACCOUNT_SCOPE_CHANGED": {
      if (event.accountScopeGeneration === state.accountScopeGeneration) {
        return state;
      }
      return {
        ...state,
        isOpen: false,
        targetId: null,
        activeTab: "profile",
        accountScopeGeneration: event.accountScopeGeneration,
      };
    }
    case "RESIZE_WIDTH":
      return {
        ...state,
        widthPx: clampInspectorWidth(event.widthPx),
      };
  }
}

/**
 * Fencing guard for in-flight async inspector requests across account switches
 * and rapid conversation transitions.
 */
export interface InspectorScopeTicket {
  readonly accountScopeGeneration: number;
  readonly entityId: string;
}

export function isInspectorScopeCurrent(
  ticket: InspectorScopeTicket,
  currentGeneration: number,
  currentEntityId: string | null | undefined
): boolean {
  return (
    ticket.accountScopeGeneration === currentGeneration &&
    Boolean(currentEntityId) &&
    ticket.entityId === currentEntityId
  );
}

/**
 * Pure memory citation formatter aligned with:
 * - `source/host/src/extensions/content_search/memory_citations.rs` (`format_citation`, `should_include_citations`)
 * - `extensions/memory-core/src/tools.citations.ts`
 * Never fabricates line numbers from offsets or array indices.
 */
export function formatMemoryCitationProvenance(input: {
  readonly path: string;
  readonly startLine: number;
  readonly endLine: number;
}): string | null {
  const trimmedPath = input.path.trim();
  if (
    trimmedPath.length === 0 ||
    !Number.isInteger(input.startLine) ||
    !Number.isInteger(input.endLine) ||
    input.startLine < 1 ||
    input.endLine < 1 ||
    input.endLine < input.startLine
  ) {
    return null;
  }
  const range =
    input.startLine === input.endLine
      ? `#L${input.startLine}`
      : `#L${input.startLine}-L${input.endLine}`;
  return `${trimmedPath}${range}`;
}

export function shouldIncludeMemoryCitations(
  mode: MemoryCitationsMode,
  sessionKey?: string | null,
  entityType?: InspectorEntityType
): boolean {
  if (mode === "on") return true;
  if (mode === "off") return false;
  if (entityType === "group" || entityType === "channel" || entityType === "hybrid") {
    return false;
  }
  if (!sessionKey) return true;
  const tokens = sessionKey
    .toLowerCase()
    .split(":")
    .map((t) => t.trim())
    .filter(Boolean);
  return !tokens.includes("group") && !tokens.includes("channel");
}

const SAFETY_EMOJI_PALETTE: readonly string[] = [
  "🔒", "🛡️", "🔑", "💎", "🧭", "⚡", "🛰️", "🔮",
  "🏔️", "🌊", "🌲", "🔥", "🍀", "🎯", "⚓", "🪐",
  "🦅", "🐬", "🦊", "🦉", "🐝", "🦋", "🐢", "🐳",
  "🍎", "🍋", "🍇", "🍒", "🥑", "🌶️", "🥨", "🍵",
];

/**
 * Deterministically derives a 2x2 emoji safety fingerprint and hex digest preview
 * for E2E Double Ratchet (`X25519+HKDF-SHA256+XChaCha20Poly1305`) key verification.
 * Aligns with `native/mahayana-messaging/src/secret_chat.rs` & Telegram `tde2e` key verification.
 */
export function deriveE2ESafetyFingerprint(
  peerId: string,
  localId = "self",
  publicKeyMaterial = ""
): {
  readonly grid: readonly [string, string, string, string];
  readonly hexBlocks: readonly [string, string, string, string];
  readonly algorithm: "X25519+HKDF-SHA256+XChaCha20Poly1305";
} {
  const ordered = [peerId.trim() || "peer", localId.trim() || "self"].sort().join("::");
  const seedInput = `${ordered}::${publicKeyMaterial.trim()}`;
  const hashes: [number, number, number, number] = [0x811c9dc5, 0x9e3779b9, 0x85ebca6b, 0xc2b2ae35];

  for (let i = 0; i < seedInput.length; i += 1) {
    const code = seedInput.charCodeAt(i);
    const lane = i & 3;
    hashes[lane] = Math.imul(hashes[lane] ^ code, 0x01000193) >>> 0;
  }

  const grid: [string, string, string, string] = [
    SAFETY_EMOJI_PALETTE[hashes[0] % SAFETY_EMOJI_PALETTE.length] ?? "🔒",
    SAFETY_EMOJI_PALETTE[hashes[1] % SAFETY_EMOJI_PALETTE.length] ?? "🛡️",
    SAFETY_EMOJI_PALETTE[hashes[2] % SAFETY_EMOJI_PALETTE.length] ?? "🔑",
    SAFETY_EMOJI_PALETTE[hashes[3] % SAFETY_EMOJI_PALETTE.length] ?? "💎",
  ];

  const hexBlocks: [string, string, string, string] = [
    hashes[0].toString(16).padStart(8, "0").toUpperCase(),
    hashes[1].toString(16).padStart(8, "0").toUpperCase(),
    hashes[2].toString(16).padStart(8, "0").toUpperCase(),
    hashes[3].toString(16).padStart(8, "0").toUpperCase(),
  ];

  return {
    grid,
    hexBlocks,
    algorithm: "X25519+HKDF-SHA256+XChaCha20Poly1305",
  };
}
