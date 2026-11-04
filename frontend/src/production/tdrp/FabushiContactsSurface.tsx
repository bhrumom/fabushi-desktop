import React, { useState, useMemo } from "react";
import { UserPlus, Search, MessageSquare, Phone, Video, Shield, User, Users, Bot } from "lucide-react";

export interface ContactItem {
  readonly id: string;
  readonly name: string;
  readonly role?: string;
  readonly status?: string;
  readonly avatarText?: string;
  readonly kind: "human" | "agent" | "group";
  readonly bio?: string;
  readonly sharedGroupsCount?: number;
}

export interface FabushiContactsSurfaceProps {
  readonly contacts?: readonly ContactItem[];
  readonly onOpenConversation: (id: string) => void;
  readonly onInitiateCall?: (id: string, isVideo: boolean) => void;
}

const DEFAULT_CONTACTS: readonly ContactItem[] = [
  { id: "c1", name: "张明 (Zhang Ming)", role: "技术总监", status: "在线", avatarText: "张", kind: "human", bio: "全球法布施技术研发负责人，专注去中心化与智能体通信架构。", sharedGroupsCount: 4 },
  { id: "c2", name: "李华 (Li Hua)", role: "产品运营", status: "离线 (2小时前)", avatarText: "李", kind: "human", bio: "负责全球社群建设与产品体验迭代。", sharedGroupsCount: 2 },
  { id: "c3", name: "法布施智能客服", role: "Official Agent", status: "全天候运行", avatarText: "法", kind: "agent", bio: "全天候解答平台使用问题，支持多语言实时答疑与操作指引。", sharedGroupsCount: 1 },
  { id: "c4", name: "自动化发布流水线", role: "DevOps Agent", status: "空闲", avatarText: "流", kind: "agent", bio: "自动化交付与正式版本封包流水线调度器。", sharedGroupsCount: 3 },
  { id: "c5", name: "全球核心开发者群", role: "群组 (28人)", status: "活跃中", avatarText: "核", kind: "group", bio: "全球法布施官方核心开发与贡献者协同群组。", sharedGroupsCount: 1 },
];

export const FabushiContactsSurface: React.FC<FabushiContactsSurfaceProps> = ({
  contacts = DEFAULT_CONTACTS,
  onOpenConversation,
  onInitiateCall,
}) => {
  const [selectedContactId, setSelectedContactId] = useState<string>(contacts[0]?.id ?? "c1");
  const [searchQuery, setSearchQuery] = useState("");
  const [showAddContactModal, setShowAddContactModal] = useState(false);
  const [newContactName, setNewContactName] = useState("");
  const [contactList, setContactList] = useState<ContactItem[]>(() => [...contacts]);

  const filteredContacts = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return contactList;
    return contactList.filter((c) => c.name.toLowerCase().includes(q) || (c.role && c.role.toLowerCase().includes(q)));
  }, [contactList, searchQuery]);

  const selectedContact = useMemo(() => {
    return contactList.find((c) => c.id === selectedContactId) || contactList[0];
  }, [contactList, selectedContactId]);

  const handleAddContact = () => {
    const trimmed = newContactName.trim();
    if (!trimmed) return;
    const newContact: ContactItem = {
      id: `contact_${Date.now()}`,
      name: trimmed,
      role: "外部联系人",
      status: "离线",
      avatarText: trimmed[0],
      kind: "human",
      bio: "新添加的联系人",
    };
    setContactList((prev) => [newContact, ...prev]);
    setSelectedContactId(newContact.id);
    setNewContactName("");
    setShowAddContactModal(false);
  };

  return (
    <div
      className="fabushi-contacts-surface"
      data-testid="fabushi-contacts-surface"
      style={{
        display: "grid",
        gridTemplateColumns: "288px minmax(0, 1fr)",
        height: "100%",
        width: "100%",
        background: "var(--cursor-bg-editor, #202024)",
        color: "var(--fabushi-text-primary, #ffffff)",
      }}
    >
      {/* Middle collection pane */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          borderRight: "1px solid var(--fabushi-border-weak, var(--sand-border-weak, rgba(255, 255, 255, 0.08)))",
          background: "var(--cursor-bg-chrome, #18181c)",
          height: "100%",
          overflow: "hidden",
        }}
      >
        {/* Header */}
        <div style={{ padding: "12px 14px", borderBottom: "1px solid var(--fabushi-border-weak, rgba(255, 255, 255, 0.08))" }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "8px" }}>
            <h2 style={{ margin: 0, fontSize: "16px", fontWeight: 700 }}>联系人</h2>
            <button
              type="button"
              data-testid="fabushi-add-contact-btn"
              title="添加联系人"
              aria-label="添加联系人"
              onClick={() => setShowAddContactModal(true)}
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
              <UserPlus size={14} />
              <span>添加</span>
            </button>
          </div>
          {/* Search bar */}
          <div style={{ position: "relative" }}>
            <Search size={14} style={{ position: "absolute", left: "8px", top: "50%", transform: "translateY(-50%)", color: "#888" }} />
            <input
              type="text"
              data-testid="fabushi-contact-search-input"
              placeholder="搜索联系人..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{
                width: "100%",
                padding: "6px 8px 6px 28px",
                borderRadius: "6px",
                border: "1px solid var(--fabushi-border-weak, #444)",
                background: "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.06))",
                color: "#fff",
                fontSize: "12px",
              }}
            />
          </div>
        </div>

        {/* Contact List */}
        <div style={{ flex: 1, overflowY: "auto", padding: "6px 0" }}>
          {filteredContacts.map((contact) => {
            const isSelected = selectedContactId === contact.id;
            return (
              <div
                key={contact.id}
                data-testid={`fabushi-contact-row-${contact.id}`}
                onClick={() => setSelectedContactId(contact.id)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "10px",
                  padding: "8px 14px",
                  cursor: "pointer",
                  background: isSelected ? "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.08))" : "transparent",
                  borderLeft: isSelected ? "3px solid var(--fabushi-accent, #1084fe)" : "3px solid transparent",
                  transition: "background 0.1s ease",
                }}
              >
                <div
                  style={{
                    width: "36px",
                    height: "36px",
                    borderRadius: "50%",
                    background: contact.kind === "agent" ? "#673ab7" : contact.kind === "group" ? "#009688" : "#1084fe",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: "14px",
                    fontWeight: 700,
                    color: "#ffffff",
                    flexShrink: 0,
                  }}
                >
                  {contact.avatarText || contact.name[0]}
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: "13px", fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {contact.name}
                  </div>
                  <div style={{ fontSize: "11px", color: "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.6))", marginTop: "2px" }}>
                    {contact.role || contact.status}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Main workspace (Participant Profile View) */}
      {selectedContact ? (
        <div
          data-testid="fabushi-participant-profile-workspace"
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            padding: "48px 32px",
            overflowY: "auto",
          }}
        >
          {/* Avatar & Title */}
          <div
            style={{
              width: "72px",
              height: "72px",
              borderRadius: "50%",
              background: selectedContact.kind === "agent" ? "#673ab7" : selectedContact.kind === "group" ? "#009688" : "#1084fe",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: "28px",
              fontWeight: 700,
              color: "#ffffff",
              marginBottom: "16px",
              boxShadow: "0 4px 16px rgba(0,0,0,0.3)",
            }}
          >
            {selectedContact.avatarText || selectedContact.name[0]}
          </div>

          <h1 style={{ margin: "0 0 4px 0", fontSize: "20px", fontWeight: 700 }}>{selectedContact.name}</h1>
          <div style={{ fontSize: "13px", color: "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.6))", marginBottom: "8px" }}>
            {selectedContact.role} · {selectedContact.status}
          </div>

          {/* Action buttons */}
          <div style={{ display: "flex", gap: "10px", marginTop: "16px", marginBottom: "32px" }}>
            <button
              type="button"
              data-testid="fabushi-contact-send-msg-btn"
              onClick={() => onOpenConversation(selectedContact.id)}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "6px",
                padding: "8px 18px",
                borderRadius: "8px",
                border: 0,
                background: "var(--fabushi-accent, #1084fe)",
                color: "#ffffff",
                fontSize: "13px",
                fontWeight: 600,
                cursor: "pointer",
              }}
            >
              <MessageSquare size={16} />
              <span>发消息</span>
            </button>

            <button
              type="button"
              data-testid="fabushi-contact-call-btn"
              onClick={() => onInitiateCall && onInitiateCall(selectedContact.id, false)}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "6px",
                padding: "8px 16px",
                borderRadius: "8px",
                border: "1px solid var(--fabushi-border-weak, #444)",
                background: "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.08))",
                color: "#ffffff",
                fontSize: "13px",
                fontWeight: 500,
                cursor: "pointer",
              }}
            >
              <Phone size={15} />
              <span>语音</span>
            </button>

            <button
              type="button"
              data-testid="fabushi-contact-video-btn"
              onClick={() => onInitiateCall && onInitiateCall(selectedContact.id, true)}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "6px",
                padding: "8px 16px",
                borderRadius: "8px",
                border: "1px solid var(--fabushi-border-weak, #444)",
                background: "var(--cursor-bg-secondary, rgba(255, 255, 255, 0.08))",
                color: "#ffffff",
                fontSize: "13px",
                fontWeight: 500,
                cursor: "pointer",
              }}
            >
              <Video size={15} />
              <span>视频</span>
            </button>
          </div>

          {/* Profile details card */}
          <div
            style={{
              width: "100%",
              maxWidth: "520px",
              background: "var(--cursor-bg-chrome, #18181c)",
              borderRadius: "12px",
              border: "1px solid var(--fabushi-border-weak, rgba(255, 255, 255, 0.08))",
              padding: "16px 20px",
            }}
          >
            <div style={{ marginBottom: "14px" }}>
              <div style={{ fontSize: "11px", color: "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.5))", marginBottom: "4px" }}>个人简介</div>
              <div style={{ fontSize: "13px", lineHeight: "1.5" }}>{selectedContact.bio || "暂无简介"}</div>
            </div>

            <div style={{ borderTop: "1px solid var(--fabushi-border-weak, rgba(255, 255, 255, 0.08))", paddingTop: "12px", display: "flex", justifyContent: "space-between" }}>
              <span style={{ fontSize: "12px", color: "var(--fabushi-text-secondary, rgba(255, 255, 255, 0.6))" }}>共享群组</span>
              <span style={{ fontSize: "12px", fontWeight: 600 }}>{selectedContact.sharedGroupsCount ?? 0} 个群组</span>
            </div>
          </div>
        </div>
      ) : (
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "#888" }}>
          请选择一个联系人查看资料
        </div>
      )}

      {/* Add Contact Modal */}
      {showAddContactModal && (
        <div
          role="dialog"
          aria-modal="true"
          data-testid="fabushi-add-contact-modal"
          onClick={() => setShowAddContactModal(false)}
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: "rgba(0, 0, 0, 0.6)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 9999,
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "var(--cursor-bg-editor, #242429)",
              padding: "20px 24px",
              borderRadius: "14px",
              width: "320px",
              color: "#fff",
            }}
          >
            <h3 style={{ margin: "0 0 12px 0", fontSize: "15px", fontWeight: 600 }}>添加联系人</h3>
            <input
              type="text"
              data-testid="fabushi-add-contact-name-input"
              placeholder="输入联系人姓名或用户名"
              value={newContactName}
              onChange={(e) => setNewContactName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAddContact();
              }}
              autoFocus
              style={{
                width: "100%",
                padding: "8px",
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
                onClick={() => setShowAddContactModal(false)}
                style={{ padding: "5px 12px", borderRadius: "6px", border: "1px solid #444", background: "transparent", color: "#bbb", fontSize: "12px", cursor: "pointer" }}
              >
                取消
              </button>
              <button
                type="button"
                data-testid="fabushi-add-contact-confirm-btn"
                onClick={handleAddContact}
                style={{ padding: "5px 14px", borderRadius: "6px", border: 0, background: "var(--fabushi-accent, #1084fe)", color: "#fff", fontSize: "12px", fontWeight: 600, cursor: "pointer" }}
              >
                添加
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
