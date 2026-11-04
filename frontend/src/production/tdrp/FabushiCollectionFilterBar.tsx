import React, { useState, useEffect, useCallback, useMemo } from "react";
import { Plus, Search, FolderPlus, Bell, User, Users, Bot, MessageSquare, Radio } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";

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
  onOpenSearch,
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
  const [newFolderName, setNewFolderName] = useState("");

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

  const handleCreateFolder = useCallback(() => {
    const trimmed = newFolderName.trim();
    if (!trimmed) return;
    const folderId = `folder_${Date.now()}`;
    const newFolder: FabushiFilterFolder = { id: folderId, name: trimmed, isCustom: true };
    const updated = [...customFolders, newFolder];
    setCustomFolders(updated);
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
    } catch {}

    if (client != null) {
      client.call("folders.create", { folder: { id: folderId, title: trimmed } }).catch(() => {});
    }

    setNewFolderName("");
    setShowAddFolderModal(false);
    onSelectFolder(folderId);
  }, [client, customFolders, newFolderName, onSelectFolder]);

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

  const defaultTabs = useMemo(() => [
    { id: "all", name: "全部", icon: MessageSquare, badge: 0 },
    { id: "unread", name: "未读", icon: Bell, badge: unreadCount },
    { id: "direct", name: "私聊", icon: User, badge: directCount },
    { id: "groups", name: "群组", icon: Users, badge: groupCount },
    { id: "channels", name: "频道", icon: Radio, badge: channelCount },
    { id: "agents", name: "Agent", icon: Bot, badge: agentCount },
  ], [agentCount, channelCount, directCount, groupCount, unreadCount]);

  return (
    <div
      className="fabushi-collection-filter-bar"
      data-testid="fabushi-collection-filter-bar"
      style={{
        display: "flex",
        flexDirection: "column",
        background: "var(--fabushi-surface-chrome, var(--cursor-bg-chrome, #1a1a1f))",
        borderBottom: "1px solid var(--fabushi-border-weak, var(--sand-border-weak, rgba(255, 255, 255, 0.08)))",
        padding: "10px 12px 6px 12px",
        userSelect: "none",
      }}
    >
      {/* Messages collection header */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "8px" }}>
        <h2 style={{ margin: 0, fontSize: "16px", fontWeight: 700, color: "var(--fabushi-text-primary, #ffffff)" }}>
          消息
        </h2>
        <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
          {onOpenSearch && (
            <button
              type="button"
              data-testid="fabushi-collection-search-btn"
              title="搜索 (Cmd+K)"
              aria-label="搜索会话"
              onClick={onOpenSearch}
              style={{
                width: "28px",
                height: "28px",
                borderRadius: "6px",
                border: 0,
                background: "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.08))",
                color: "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.7))",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                cursor: "pointer",
              }}
            >
              <Search size={15} strokeWidth={2} />
            </button>
          )}
          <button
            type="button"
            data-testid="fabushi-collection-new-btn"
            title="新建会话"
            aria-label="新建会话"
            onClick={onOpenNewChat}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "4px",
              padding: "4px 8px",
              borderRadius: "6px",
              border: 0,
              background: "var(--fabushi-accent, #1084fe)",
              color: "#ffffff",
              fontSize: "12px",
              fontWeight: 600,
              cursor: "pointer",
            }}
          >
            <Plus size={14} strokeWidth={2.4} />
            <span>新建</span>
          </button>
        </div>
      </div>

      {/* Filter views tabs */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "4px",
          overflowX: "auto",
          scrollbarWidth: "none",
          whiteSpace: "nowrap",
          paddingBottom: "2px",
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
                padding: "4px 8px",
                fontSize: "11px",
                fontWeight: isActive ? 600 : 400,
                color: isActive ? "#ffffff" : "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.65))",
                background: isActive
                  ? "var(--fabushi-accent, #1084fe)"
                  : "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.06))",
                border: 0,
                borderRadius: "12px",
                cursor: "pointer",
                transition: "all 0.12s ease",
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
                    background: isActive ? "#ffffff" : "var(--fabushi-accent, #1084fe)",
                    color: isActive ? "var(--fabushi-accent, #1084fe)" : "#ffffff",
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
                padding: "4px 8px",
                fontSize: "11px",
                fontWeight: isActive ? 600 : 400,
                color: isActive ? "#ffffff" : "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.65))",
                background: isActive
                  ? "var(--fabushi-accent, #1084fe)"
                  : "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.06))",
                border: 0,
                borderRadius: "12px",
                cursor: "pointer",
              }}
            >
              <span>{folder.name}</span>
              <span
                role="button"
                tabIndex={0}
                aria-label="删除文件夹"
                onClick={(e) => handleDeleteFolder(folder.id, e)}
                style={{ fontSize: "10px", opacity: 0.7, cursor: "pointer", marginLeft: "2px" }}
              >
                ×
              </span>
            </button>
          );
        })}

        <button
          type="button"
          data-testid="fabushi-add-folder-btn"
          title="新建文件夹"
          onClick={() => setShowAddFolderModal(true)}
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "2px",
            padding: "4px 6px",
            fontSize: "11px",
            color: "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.5))",
            background: "transparent",
            border: "1px dashed var(--fabushi-border-weak, rgba(255, 255, 255, 0.15))",
            borderRadius: "12px",
            cursor: "pointer",
          }}
        >
          <FolderPlus size={12} strokeWidth={1.8} />
        </button>
      </div>

      {/* New Folder Modal */}
      {showAddFolderModal && (
        <div
          role="dialog"
          aria-modal="true"
          data-testid="fabushi-create-folder-modal"
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: "rgba(0, 0, 0, 0.5)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 9999,
          }}
          onClick={() => setShowAddFolderModal(false)}
        >
          <div
            style={{
              background: "var(--cursor-bg-editor, #232328)",
              padding: "18px 20px",
              borderRadius: "12px",
              width: "280px",
              boxShadow: "0 8px 24px rgba(0,0,0,0.4)",
              color: "#ffffff",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h4 style={{ margin: "0 0 12px 0", fontSize: "14px", fontWeight: 600 }}>新建分组文件夹</h4>
            <input
              type="text"
              data-testid="fabushi-folder-name-input"
              placeholder="文件夹名称"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreateFolder();
              }}
              autoFocus
              style={{
                width: "100%",
                padding: "6px 8px",
                borderRadius: "6px",
                border: "1px solid var(--fabushi-border-weak, #444)",
                background: "var(--cursor-bg-chrome, #18181c)",
                color: "#fff",
                fontSize: "12px",
                marginBottom: "14px",
              }}
            />
            <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
              <button
                type="button"
                onClick={() => setShowAddFolderModal(false)}
                style={{ padding: "5px 12px", borderRadius: "6px", border: "1px solid #444", background: "transparent", color: "#bbb", fontSize: "12px", cursor: "pointer" }}
              >
                取消
              </button>
              <button
                type="button"
                data-testid="fabushi-folder-confirm-btn"
                onClick={handleCreateFolder}
                style={{ padding: "5px 14px", borderRadius: "6px", border: 0, background: "var(--fabushi-accent, #1084fe)", color: "#fff", fontSize: "12px", fontWeight: 600, cursor: "pointer" }}
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
