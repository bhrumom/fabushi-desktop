import React, { useState, useEffect, useCallback, useMemo } from "react";
import type { ProductionCoordinatorClient } from "../coordinator-client";

export interface TdrpFolder {
  readonly id: string;
  readonly name: string;
  readonly icon?: string;
  readonly badge?: number;
  readonly isCustom?: boolean;
}

export interface TdrpFolderFilterBarProps {
  readonly activeFolderId: string;
  readonly onSelectFolder: (folderId: string) => void;
  readonly unreadCount?: number;
  readonly humanCount?: number;
  readonly groupCount?: number;
  readonly agentCount?: number;
  readonly client?: ProductionCoordinatorClient | null;
}

const STORAGE_KEY = "fabushi_tdrp_custom_folders_v1";

const DEFAULT_FOLDERS: readonly TdrpFolder[] = [
  { id: "all", name: "全部", icon: "💬" },
  { id: "unread", name: "未读", icon: "🔔" },
  { id: "human", name: "人类", icon: "👤" },
  { id: "groups", name: "群组/频道", icon: "👥" },
  { id: "agents", name: "智能体", icon: "🤖" },
];

export { filterAgentsByFolder, type FilterableAgent } from "./tdrp-folder-filter";

export const TdrpFolderFilterBar: React.FC<TdrpFolderFilterBarProps> = ({
  activeFolderId,
  onSelectFolder,
  unreadCount = 0,
  humanCount = 0,
  groupCount = 0,
  agentCount = 0,
  client,
}) => {
  const [customFolders, setCustomFolders] = useState<TdrpFolder[]>(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      return saved ? JSON.parse(saved) : [];
    } catch {
      return [];
    }
  });
  const [showAddModal, setShowAddModal] = useState(false);
  const [newFolderName, setNewFolderName] = useState("");
  const [newFolderIcon, setNewFolderIcon] = useState("📁");

  // Sync with coordinator if available
  useEffect(() => {
    if (client == null) return;
    client
      .call("folders.list", {})
      .then((res: unknown) => {
        if (res && typeof res === "object" && "folders" in res && Array.isArray((res as { folders: unknown[] }).folders)) {
          const list = (res as { folders: Array<{ id: string; name: string; icon?: string }> }).folders.map((f) => ({
            id: f.id,
            name: f.name,
            icon: f.icon || "📁",
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
      .catch(() => {
        // Fallback to local storage
      });
  }, [client]);

  const handleCreateFolder = useCallback(() => {
    const trimmed = newFolderName.trim();
    if (!trimmed) return;
    const folderId = `folder_${Date.now()}`;
    const newFolder: TdrpFolder = {
      id: folderId,
      name: trimmed,
      icon: newFolderIcon,
      isCustom: true,
    };
    const updated = [...customFolders, newFolder];
    setCustomFolders(updated);
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
    } catch {}

    if (client != null) {
      client.call("folders.create", { folder: { id: folderId, title: trimmed, icon: newFolderIcon } }).catch(() => {});
    }

    setNewFolderName("");
    setShowAddModal(false);
    onSelectFolder(folderId);
  }, [client, customFolders, newFolderIcon, newFolderName, onSelectFolder]);

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

  const allFolders = useMemo(() => {
    return [
      ...DEFAULT_FOLDERS.map((f) => {
        if (f.id === "unread" && unreadCount > 0) return { ...f, badge: unreadCount };
        if (f.id === "human" && humanCount > 0) return { ...f, badge: humanCount };
        if (f.id === "groups" && groupCount > 0) return { ...f, badge: groupCount };
        if (f.id === "agents" && agentCount > 0) return { ...f, badge: agentCount };
        return f;
      }),
      ...customFolders,
    ];
  }, [agentCount, customFolders, groupCount, humanCount, unreadCount]);

  return (
    <div
      className="tdrp-folder-filter-bar"
      data-testid="tdrp-folder-filter-bar"
      style={{
        display: "flex",
        alignItems: "center",
        gap: "4px",
        padding: "6px 10px",
        background: "var(--cursor-bg-chrome, #1e1e24)",
        borderBottom: "1px solid var(--sand-border-weak, rgba(255, 255, 255, 0.08))",
        overflowX: "auto",
        scrollbarWidth: "none",
        whiteSpace: "nowrap",
        userSelect: "none",
      }}
    >
      {allFolders.map((folder) => {
        const isActive = activeFolderId === folder.id;
        return (
          <button
            key={folder.id}
            type="button"
            data-testid={`tdrp-folder-tab-${folder.id}`}
            onClick={() => onSelectFolder(folder.id)}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "5px",
              padding: "4px 9px",
              fontSize: "12px",
              fontWeight: isActive ? 600 : 400,
              color: isActive ? "#ffffff" : "var(--cursor-text-secondary, rgba(255, 255, 255, 0.65))",
              background: isActive
                ? "var(--sand-fill-accent, #3390ec)"
                : "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.06))",
              border: 0,
              borderRadius: "14px",
              cursor: "pointer",
              transition: "all 0.15s ease",
            }}
          >
            <span>{folder.icon}</span>
            <span>{folder.name}</span>
            {folder.badge != null && folder.badge > 0 && (
              <span
                style={{
                  display: "inline-block",
                  padding: "1px 5px",
                  fontSize: "10px",
                  lineHeight: "12px",
                  fontWeight: 700,
                  color: isActive ? "var(--sand-fill-accent, #3390ec)" : "#ffffff",
                  background: isActive ? "#ffffff" : "var(--sand-fill-accent, #3390ec)",
                  borderRadius: "10px",
                  minWidth: "14px",
                  textAlign: "center",
                }}
              >
                {folder.badge > 99 ? "99+" : folder.badge}
              </span>
            )}
            {folder.isCustom && (
              <span
                role="button"
                tabIndex={0}
                aria-label="Delete folder"
                onClick={(e) => handleDeleteFolder(folder.id, e)}
                style={{
                  marginLeft: "2px",
                  fontSize: "11px",
                  opacity: 0.6,
                  cursor: "pointer",
                }}
              >
                ×
              </span>
            )}
          </button>
        );
      })}

      <button
        type="button"
        data-testid="tdrp-add-folder-button"
        title="新建文件夹"
        onClick={() => setShowAddModal(true)}
        style={{
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "4px 8px",
          fontSize: "12px",
          color: "var(--cursor-text-secondary, rgba(255, 255, 255, 0.65))",
          background: "transparent",
          border: "1px dashed var(--sand-border-weak, rgba(255, 255, 255, 0.2))",
          borderRadius: "14px",
          cursor: "pointer",
        }}
      >
        + 文件夹
      </button>

      {showAddModal && (
        <div
          role="dialog"
          aria-modal="true"
          data-testid="tdrp-add-folder-modal"
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
          onClick={() => setShowAddModal(false)}
        >
          <div
            style={{
              background: "var(--cursor-bg-editor, #232328)",
              padding: "18px 20px",
              borderRadius: "12px",
              width: "280px",
              boxShadow: "0 8px 24px rgba(0,0,0,0.4)",
              color: "var(--cursor-text-primary, #ffffff)",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h4 style={{ margin: "0 0 12px 0", fontSize: "14px", fontWeight: 600 }}>新建分组文件夹</h4>
            <div style={{ display: "flex", gap: "8px", marginBottom: "12px" }}>
              <select
                value={newFolderIcon}
                onChange={(e) => setNewFolderIcon(e.target.value)}
                style={{
                  padding: "6px",
                  borderRadius: "6px",
                  border: "1px solid var(--sand-border-weak, #444)",
                  background: "var(--cursor-bg-chrome, #18181c)",
                  color: "#fff",
                }}
              >
                <option value="📁">📁 默认</option>
                <option value="⭐">⭐ 重要</option>
                <option value="💼">💼 工作</option>
                <option value="🎮">🎮 娱乐</option>
                <option value="🚀">🚀 项目</option>
              </select>
              <input
                type="text"
                data-testid="tdrp-new-folder-input"
                placeholder="文件夹名称"
                value={newFolderName}
                onChange={(e) => setNewFolderName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleCreateFolder();
                }}
                autoFocus
                style={{
                  flex: 1,
                  padding: "6px 8px",
                  borderRadius: "6px",
                  border: "1px solid var(--sand-border-weak, #444)",
                  background: "var(--cursor-bg-chrome, #18181c)",
                  color: "#fff",
                  fontSize: "12px",
                }}
              />
            </div>
            <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
              <button
                type="button"
                onClick={() => setShowAddModal(false)}
                style={{
                  padding: "5px 12px",
                  borderRadius: "6px",
                  border: "1px solid var(--sand-border-weak, #444)",
                  background: "transparent",
                  color: "#bbb",
                  fontSize: "12px",
                  cursor: "pointer",
                }}
              >
                取消
              </button>
              <button
                type="button"
                data-testid="tdrp-create-folder-confirm"
                onClick={handleCreateFolder}
                style={{
                  padding: "5px 14px",
                  borderRadius: "6px",
                  border: 0,
                  background: "var(--sand-fill-accent, #3390ec)",
                  color: "#fff",
                  fontSize: "12px",
                  fontWeight: 600,
                  cursor: "pointer",
                }}
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
