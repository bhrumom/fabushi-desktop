import React, { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { Plus, FolderPlus, Bell, User, Users, Bot, MessageSquare, Radio, X } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { TdrpChatFilterEditModal, type ChatFilterModel } from "./TdrpChatFilterEditModal";
import { TdrpStoryPostModal, type StoryPublishPayload } from "./TdrpStoryPostModal";

/**
 * Canonical Collection Filter Bar & Stories Strip (Blueprint 01 §4.1, §5.1, §5.3).
 *
 * Upstream Source Alignment:
 * - Telegram Desktop: `Telegram/SourceFiles/data/data_chat_filters.cpp` (`Data::ChatFilter::FromTL`),
 *   `Telegram/SourceFiles/dialogs/ui/dialogs_stories_content.cpp` (`Dialogs::Stories::ContentForSession`).
 * - Invariant: Custom folders storage key is scoped by `accountId` and all async RPC responses / mutations
 *   are fenced by `accountScopeGeneration` to prevent cross-account folder leakage.
 */

export interface FabushiFilterFolder {
  readonly id: string;
  readonly name: string;
  readonly icon?: string;
  readonly badge?: number;
  readonly isCustom?: boolean;
}

interface StoryItem {
  readonly id: string;
  readonly authorName: string;
  readonly avatarText: string;
  readonly content: string;
  readonly timestamp: string;
}

const INITIAL_STORIES: readonly StoryItem[] = [
  { id: "s1", authorName: "智能体", avatarText: "智", content: "原生架构升级完成，自动化无缝对接！", timestamp: "10分钟前" },
  { id: "s2", authorName: "热点", avatarText: "热", content: "去中心化网络技术融合进入新阶段。", timestamp: "1小时前" },
  { id: "s3", authorName: "法布施", avatarText: "法", content: "全球法布施 1.2.78 正式就绪！", timestamp: "2小时前" },
];

export interface FabushiCollectionFilterBarProps {
  readonly activeFolderId: string;
  readonly onSelectFolder: (folderId: string) => void;
  readonly onOpenNewChat?: () => void;
  readonly onOpenSearch?: () => void;
  readonly unreadCount?: number;
  readonly directCount?: number;
  readonly groupCount?: number;
  readonly channelCount?: number;
  readonly agentCount?: number;
  readonly accountId?: string | null;
  readonly accountScopeGeneration?: number;
  readonly client?: ProductionCoordinatorClient | null;
  readonly style?: React.CSSProperties;
}

const STORAGE_KEY_PREFIX = "fabushi_collection_custom_folders_v1";

export function resolveCollectionCustomFoldersStorageKey(accountId?: string | null): string {
  const normalized = typeof accountId === "string" ? accountId.trim() : "";
  return `${STORAGE_KEY_PREFIX}_${normalized.length > 0 ? normalized : "default"}`;
}

function loadCustomFoldersFromStorage(storageKey: string): FabushiFilterFolder[] {
  try {
    const saved = localStorage.getItem(storageKey);
    if (saved) {
      const parsed: unknown = JSON.parse(saved);
      if (Array.isArray(parsed)) {
        return parsed.filter(
          (item): item is FabushiFilterFolder =>
            typeof item === "object" &&
            item !== null &&
            typeof (item as { id?: unknown }).id === "string" &&
            typeof (item as { name?: unknown }).name === "string",
        );
      }
    }
  } catch {
    // Ignore storage read errors
  }
  return [];
}

const pillBase: React.CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: "4px",
  padding: "3px 8px",
  fontSize: "11px",
  border: 0,
  borderRadius: "10px",
  cursor: "pointer",
  transition: "all 0.12s ease",
  flexShrink: 0,
};

const iconBtnBase: React.CSSProperties = {
  width: "24px",
  height: "24px",
  borderRadius: "6px",
  border: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  cursor: "pointer",
};

export const FabushiCollectionFilterBar: React.FC<FabushiCollectionFilterBarProps> = ({
  activeFolderId,
  onSelectFolder,
  onOpenNewChat,
  unreadCount = 0,
  directCount = 0,
  groupCount = 0,
  channelCount = 0,
  agentCount = 0,
  accountId = null,
  accountScopeGeneration = 0,
  client,
  style,
}) => {
  const storageKey = useMemo(
    () => resolveCollectionCustomFoldersStorageKey(accountId),
    [accountId],
  );
  const currentGenerationRef = useRef<number>(accountScopeGeneration);
  const currentStorageKeyRef = useRef<string>(storageKey);
  currentGenerationRef.current = accountScopeGeneration;
  currentStorageKeyRef.current = storageKey;

  const [customFolders, setCustomFolders] = useState<FabushiFilterFolder[]>(() =>
    loadCustomFoldersFromStorage(storageKey),
  );
  const [showAddFolderModal, setShowAddFolderModal] = useState(false);
  const [stories, setStories] = useState<StoryItem[]>(() => [...INITIAL_STORIES]);
  const [activeStory, setActiveStory] = useState<StoryItem | null>(null);
  const [showPostModal, setShowPostModal] = useState(false);

  // Reload account-scoped folders immediately when accountId or accountScopeGeneration changes
  useEffect(() => {
    setCustomFolders(loadCustomFoldersFromStorage(storageKey));
    setStories([...INITIAL_STORIES]);
  }, [storageKey, accountScopeGeneration]);

  useEffect(() => {
    if (client == null) return;
    const requestGeneration = accountScopeGeneration;
    const requestStorageKey = storageKey;
    const isCurrentScope = () =>
      currentGenerationRef.current === requestGeneration &&
      currentStorageKeyRef.current === requestStorageKey;

    client
      .call("folders.list", {})
      .then((res: unknown) => {
        if (!isCurrentScope()) return;
        if (res && typeof res === "object" && "folders" in res && Array.isArray((res as { folders: unknown[] }).folders)) {
          const list = (res as { folders: Array<{ id: string; name: string }> }).folders.map((f) => ({
            id: f.id,
            name: f.name,
            isCustom: true,
          }));
          if (list.length > 0) {
            setCustomFolders(list);
            try {
              localStorage.setItem(requestStorageKey, JSON.stringify(list));
            } catch {}
          }
        }
      })
      .catch(() => {});

    client
      .call("stories.list", {})
      .then((res: unknown) => {
        if (!isCurrentScope()) return;
        if (res && typeof res === "object" && "stories" in res && Array.isArray((res as { stories: unknown[] }).stories)) {
          const fetched = (res as { stories: Array<{ id: string; authorName?: string; content?: string }> }).stories.map(
            (s, idx) => ({
              id: s.id || `story-${idx}`,
              authorName: s.authorName || "用户",
              avatarText: (s.authorName || "用")[0],
              content: s.content || "精彩瞬间",
              timestamp: "最近",
            })
          );
          if (fetched.length > 0) {
            setStories(fetched);
          }
        }
      })
      .catch(() => {});
  }, [accountScopeGeneration, client, storageKey]);

  const handlePublishStory = useCallback(
    (payload: StoryPublishPayload) => {
      const mutationGeneration = currentGenerationRef.current;
      const newStory: StoryItem = {
        id: `story-${Date.now()}`,
        authorName: "我",
        avatarText: "我",
        content: payload.content,
        timestamp: "刚刚",
      };
      if (currentGenerationRef.current !== mutationGeneration) return;
      setStories((prev) => [newStory, ...prev]);
      if (client != null) {
        client.call("stories.publish", payload).catch(() => {});
      }
    },
    [client]
  );

  const handleSaveFilter = useCallback(
    (filter: ChatFilterModel) => {
      const mutationGeneration = currentGenerationRef.current;
      const mutationStorageKey = currentStorageKeyRef.current;
      if (
        currentGenerationRef.current !== mutationGeneration ||
        currentStorageKeyRef.current !== mutationStorageKey
      ) {
        return;
      }
      const newFolder: FabushiFilterFolder = { id: filter.id, name: filter.title, isCustom: true };
      const updated = [...customFolders, newFolder];
      setCustomFolders(updated);
      try {
        localStorage.setItem(mutationStorageKey, JSON.stringify(updated));
      } catch {}

      if (client != null) {
        client
          .call("folders.create", {
            folder: {
              id: filter.id,
              title: filter.title,
              colorIndex: filter.colorIndex,
              flags: filter.flags,
              alwaysPeers: filter.alwaysPeers,
              neverPeers: filter.neverPeers,
              accountScopeGeneration: mutationGeneration,
            },
          })
          .catch(() => {});
      }

      onSelectFolder(filter.id);
    },
    [client, customFolders, onSelectFolder]
  );

  const handleDeleteFolder = useCallback(
    (folderId: string, event: React.MouseEvent) => {
      event.stopPropagation();
      const mutationGeneration = currentGenerationRef.current;
      const mutationStorageKey = currentStorageKeyRef.current;
      if (
        currentGenerationRef.current !== mutationGeneration ||
        currentStorageKeyRef.current !== mutationStorageKey
      ) {
        return;
      }
      const updated = customFolders.filter((f) => f.id !== folderId);
      setCustomFolders(updated);
      try {
        localStorage.setItem(mutationStorageKey, JSON.stringify(updated));
      } catch {}

      if (client != null) {
        client.call("folders.delete", { folderId, accountScopeGeneration: mutationGeneration }).catch(() => {});
      }

      if (activeFolderId === folderId) {
        onSelectFolder("all");
      }
    },
    [activeFolderId, client, customFolders, onSelectFolder]
  );

  const defaultTabs = useMemo(
    () => [
      { id: "all", name: "全部", icon: MessageSquare, badge: 0 },
      { id: "unread", name: "未读", icon: Bell, badge: unreadCount },
      { id: "direct", name: "私聊", icon: User, badge: directCount },
      { id: "groups", name: "群组", icon: Users, badge: groupCount },
      { id: "channels", name: "频道", icon: Radio, badge: channelCount },
      { id: "agents", name: "Agent", icon: Bot, badge: agentCount },
    ],
    [agentCount, channelCount, directCount, groupCount, unreadCount]
  );

  return (
    <div
      className="fabushi-collection-filter-bar"
      data-testid="fabushi-collection-filter-bar"
      style={{
        display: "flex",
        flexDirection: "column",
        background: "var(--sand-surface-backdrop, var(--cursor-bg-chrome, #18181c))",
        borderBottom: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))",
        userSelect: "none",
        overflow: "hidden",
        ...style,
      }}
    >
      {/* Row 1: Folder tabs & action buttons */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "4px",
          padding: "6px 8px 4px 8px",
          overflow: "hidden",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "4px",
            overflowX: "auto",
            scrollbarWidth: "none",
            whiteSpace: "nowrap",
            flex: 1,
            minWidth: 0,
          }}
        >
          {defaultTabs.map((tab) => {
            const isActive = activeFolderId === tab.id;
            const TabIcon = tab.icon;
            return (
              <button
                key={tab.id}
                type="button"
                data-testid={`fabushi-filter-tab-${tab.id}`}
                onClick={() => onSelectFolder(tab.id)}
                style={{
                  ...pillBase,
                  fontWeight: isActive ? 700 : 500,
                  color: isActive ? "#ffffff" : "var(--sand-text-primary, #18181b)",
                  background: isActive ? "#1084fe" : "var(--sand-surface-muted, rgba(120, 120, 128, 0.14))",
                }}
              >
                <TabIcon size={12} strokeWidth={isActive ? 2.2 : 1.8} />
                <span>{tab.name}</span>
                {tab.badge > 0 && (
                  <span style={{ display: "inline-block", padding: "0 4px", fontSize: "9px", lineHeight: "12px", fontWeight: 700, borderRadius: "6px", background: isActive ? "#ffffff" : "#1084fe", color: isActive ? "#1084fe" : "#ffffff" }}>
                    {tab.badge > 99 ? "99+" : tab.badge}
                  </span>
                )}
              </button>
            );
          })}

          {customFolders.map((folder) => {
            const isActive = activeFolderId === folder.id;
            return (
              <button
                key={folder.id}
                type="button"
                data-testid={`fabushi-filter-tab-${folder.id}`}
                onClick={() => onSelectFolder(folder.id)}
                style={{
                  ...pillBase,
                  fontWeight: isActive ? 700 : 500,
                  color: isActive ? "#ffffff" : "var(--sand-text-primary, #18181b)",
                  background: isActive ? "#1084fe" : "var(--sand-surface-muted, rgba(120, 120, 128, 0.14))",
                }}
              >
                <span>{folder.name}</span>
                <span data-testid={`fabushi-del-folder-${folder.id}`} onClick={(e) => handleDeleteFolder(folder.id, e)} style={{ display: "inline-flex", alignItems: "center", marginLeft: "2px", opacity: 0.6 }}>
                  <X size={10} />
                </span>
              </button>
            );
          })}
        </div>

        {/* Action buttons: + folder, + new chat */}
        <div style={{ display: "flex", alignItems: "center", gap: "3px", flexShrink: 0 }}>
          <button type="button" data-testid="fabushi-add-folder-btn" title="新建对话分组" aria-label="新建分组" onClick={() => setShowAddFolderModal(true)} style={{ ...iconBtnBase, background: "transparent", color: "var(--sand-text-secondary, #52525b)" }}>
            <FolderPlus size={14} />
          </button>
          <button type="button" data-testid="fabushi-collection-new-btn" title="新建会话" aria-label="新建会话" onClick={onOpenNewChat} style={{ ...iconBtnBase, background: "#1084fe", color: "#ffffff" }}>
            <Plus size={14} strokeWidth={2.4} />
          </button>
        </div>
      </div>

      {/* Row 2: Telegram Stories Avatar Ring Strip (dialogs_stories_list.cpp) */}
      <div className="tdrp-sidebar-stories-strip" data-testid="tdrp-sidebar-stories-strip" style={{ display: "flex", alignItems: "center", gap: "8px", padding: "2px 8px 6px 8px", overflowX: "auto", scrollbarWidth: "none", borderTop: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.08))" }}>
        <button type="button" data-testid="tdrp-post-story-trigger" onClick={() => setShowPostModal(true)} style={{ display: "flex", alignItems: "center", gap: "4px", padding: "2px 6px", borderRadius: "12px", border: "1px dashed #1084fe", background: "rgba(16, 132, 254, 0.06)", color: "#1084fe", fontSize: "11px", fontWeight: 600, cursor: "pointer", flexShrink: 0 }}>
          <Plus size={12} strokeWidth={2.4} />
          <span>动态</span>
        </button>

        {stories.map((story) => (
          <button key={story.id} type="button" data-testid={`tdrp-story-ring-${story.id}`} onClick={() => setActiveStory(story)} style={{ display: "inline-flex", alignItems: "center", gap: "4px", padding: "2px 6px", borderRadius: "12px", border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.2))", background: "transparent", color: "inherit", fontSize: "11px", cursor: "pointer", flexShrink: 0 }}>
            <div style={{ width: "16px", height: "16px", borderRadius: "50%", background: "linear-gradient(135deg, #1084fe, #ff5c5c)", display: "flex", alignItems: "center", justifyContent: "center", color: "#fff", fontSize: "9px", fontWeight: 700 }}>
              {story.avatarText}
            </div>
            <span>{story.authorName}</span>
          </button>
        ))}
      </div>

      {activeStory && (
        <div data-testid="tdrp-story-viewer-modal" style={{ position: "fixed", top: 0, left: 0, right: 0, bottom: 0, background: "rgba(0, 0, 0, 0.5)", zIndex: 9999, display: "flex", alignItems: "center", justifyContent: "center" }} onClick={() => setActiveStory(null)}>
          <div style={{ background: "var(--bg-elevated, #ffffff)", border: "1px solid var(--border, #e8e4dc)", borderRadius: "14px", padding: "16px", maxWidth: "320px", width: "90%", boxShadow: "0 12px 32px rgba(0, 0, 0, 0.2)", color: "var(--text-strong, #211e1a)" }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "6px", fontWeight: 600 }}>
                <div style={{ width: "24px", height: "24px", borderRadius: "50%", background: "#1084fe", color: "#fff", display: "flex", alignItems: "center", justifyContent: "center", fontSize: "11px" }}>{activeStory.avatarText}</div>
                <span>{activeStory.authorName}</span>
              </div>
              <button type="button" onClick={() => setActiveStory(null)} style={{ background: "none", border: 0, cursor: "pointer", color: "inherit" }}><X size={16} /></button>
            </div>
            <p style={{ fontSize: "13px", lineHeight: "1.5", margin: "8px 0" }}>{activeStory.content}</p>
            <span style={{ fontSize: "11px", color: "var(--sand-text-secondary, #52525b)" }}>{activeStory.timestamp}</span>
          </div>
        </div>
      )}

      <TdrpChatFilterEditModal
        isOpen={showAddFolderModal}
        onClose={() => setShowAddFolderModal(false)}
        onSave={handleSaveFilter}
      />

      <TdrpStoryPostModal
        isOpen={showPostModal}
        onClose={() => setShowPostModal(false)}
        onPublish={handlePublishStory}
      />
    </div>
  );
};
