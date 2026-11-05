import React, { useState, useEffect, useCallback, useMemo } from "react";
import { Plus, FolderPlus, Bell, User, Users, Bot, MessageSquare, Radio, X } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { TdrpChatFilterEditModal, type ChatFilterModel } from "./TdrpChatFilterEditModal";

export interface FabushiFilterFolder {
  readonly id: string;
  readonly name: string;
  readonly badge?: number;
  readonly isCustom?: boolean;
}

export interface FabushiCollectionFilterBarProps {
  readonly activeFolderId: string;
  readonly onSelectFolder: (folderId: string) => void;
  readonly onOpenNewChat: () => void;
  readonly onOpenSearch?: () => void;
  readonly unreadCount?: number;
  readonly directCount?: number;
  readonly groupCount?: number;
  readonly channelCount?: number;
  readonly agentCount?: number;
  readonly client?: ProductionCoordinatorClient | null;
}

const STORAGE_KEY = "fabushi_collection_custom_folders_v1";

export const FabushiCollectionFilterBar: React.FC<FabushiCollectionFilterBarProps> = ({
  activeFolderId,
  onSelectFolder,
  onOpenNewChat,
  unreadCount = 0,
  directCount = 0,
  groupCount = 0,
  channelCount = 0,
  agentCount = 0,
  client,
}) => {
  const [customFolders, setCustomFolders] = useState<FabushiFilterFolder[]>(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      return saved ? JSON.parse(saved) : [];
    } catch {
      return [];
    }
  });
  const [showAddFolderModal, setShowAddFolderModal] = useState(false);

  useEffect(() => {
    if (client == null) return;
    client
      .call("folders.list", {})
      .then((res: unknown) => {
        if (res && typeof res === "object" && "folders" in res && Array.isArray((res as { folders: unknown[] }).folders)) {
          const list = (res as { folders: Array<{ id: string; name: string }> }).folders.map((f) => ({
            id: f.id,
            name: f.name,
            isCustom: true,
          }));
          if (list.length > 0) {
            setCustomFolders(list);
            try {
              localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
            } catch {}
          }
        }
      })
      .catch(() => {});
  }, [client]);

  const handleSaveFilter = useCallback(
    (filter: ChatFilterModel) => {
      const newFolder: FabushiFilterFolder = { id: filter.id, name: filter.title, isCustom: true };
      const updated = [...customFolders, newFolder];
      setCustomFolders(updated);
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
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
      const updated = customFolders.filter((f) => f.id !== folderId);
      setCustomFolders(updated);
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
      } catch {}

      if (client != null) {
        client.call("folders.delete", { folderId }).catch(() => {});
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
        alignItems: "center",
        justifyContent: "space-between",
        gap: "4px",
        padding: "6px 8px",
        background: "var(--sand-surface-backdrop, var(--cursor-bg-chrome, #18181c))",
        borderBottom: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))",
        userSelect: "none",
        overflow: "hidden",
      }}
    >
      {/* Horizontal scrollable tab pills */}
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
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                padding: "3px 8px",
                fontSize: "11px",
                fontWeight: isActive ? 700 : 500,
                color: isActive ? "#ffffff" : "var(--sand-text-primary, #18181b)",
                background: isActive
                  ? "#1084fe"
                  : "var(--sand-surface-muted, rgba(120, 120, 128, 0.14))",
                border: 0,
                borderRadius: "10px",
                cursor: "pointer",
                transition: "all 0.12s ease",
                flexShrink: 0,
              }}
            >
              <TabIcon size={12} strokeWidth={isActive ? 2.2 : 1.8} />
              <span>{tab.name}</span>
              {tab.badge > 0 && (
                <span
                  style={{
                    display: "inline-block",
                    padding: "0 4px",
                    fontSize: "9px",
                    lineHeight: "12px",
                    fontWeight: 700,
                    borderRadius: "6px",
                    background: isActive ? "#ffffff" : "#1084fe",
                    color: isActive ? "#1084fe" : "#ffffff",
                  }}
                >
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
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                padding: "3px 8px",
                fontSize: "11px",
                fontWeight: isActive ? 700 : 500,
                color: isActive ? "#ffffff" : "var(--sand-text-primary, #18181b)",
                background: isActive
                  ? "#1084fe"
                  : "var(--sand-surface-muted, rgba(120, 120, 128, 0.14))",
                border: 0,
                borderRadius: "10px",
                cursor: "pointer",
                transition: "all 0.12s ease",
                flexShrink: 0,
              }}
            >
              <span>{folder.name}</span>
              <span
                data-testid={`fabushi-del-folder-${folder.id}`}
                onClick={(e) => handleDeleteFolder(folder.id, e)}
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  marginLeft: "2px",
                  opacity: 0.6,
                }}
              >
                <X size={10} />
              </span>
            </button>
          );
        })}
      </div>

      {/* Action buttons: + folder, + new chat */}
      <div style={{ display: "flex", alignItems: "center", gap: "3px", flexShrink: 0 }}>
        <button
          type="button"
          data-testid="fabushi-add-folder-btn"
          title="新建对话分组"
          aria-label="新建分组"
          onClick={() => setShowAddFolderModal(true)}
          style={{
            width: "24px",
            height: "24px",
            borderRadius: "6px",
            border: 0,
            background: "transparent",
            color: "var(--sand-text-secondary, #52525b)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
          }}
        >
          <FolderPlus size={14} />
        </button>

        <button
          type="button"
          data-testid="fabushi-collection-new-btn"
          title="新建会话"
          aria-label="新建会话"
          onClick={onOpenNewChat}
          style={{
            width: "24px",
            height: "24px",
            borderRadius: "6px",
            border: 0,
            background: "#1084fe",
            color: "#ffffff",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
          }}
        >
          <Plus size={14} strokeWidth={2.4} />
        </button>
      </div>

      <TdrpChatFilterEditModal
        isOpen={showAddFolderModal}
        onClose={() => setShowAddFolderModal(false)}
        onSave={handleSaveFilter}
      />
    </div>
  );
};
