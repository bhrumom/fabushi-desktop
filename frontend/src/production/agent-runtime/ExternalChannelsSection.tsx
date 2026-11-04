import { useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { ChannelId, ChannelItem, DmPolicy, GroupPolicy } from "./types";

interface ExternalChannelsSectionProps {
  readonly agentId: string;
}

const INITIAL_CHANNELS: ChannelItem[] = [
  { id: "telegram", name: "Telegram Bot", protocol: "Bot API / MTProto", status: "connected", dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 3 },
  { id: "discord", name: "Discord", protocol: "Gateway WebSocket", status: "connected", dmPolicy: "open", groupPolicy: "open", activeChatsCount: 8 },
  { id: "slack", name: "Slack", protocol: "Events API / Socket", status: "connected", dmPolicy: "allowlist", groupPolicy: "allowlist", activeChatsCount: 2 },
  { id: "whatsapp", name: "WhatsApp", protocol: "Cloud API / Baileys", status: "disconnected", dmPolicy: "pairing", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "feishu", name: "飞书 (Feishu)", protocol: "Open API", status: "connected", dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 5 },
  { id: "matrix", name: "Matrix", protocol: "Client-Server v3", status: "disconnected", dmPolicy: "pairing", groupPolicy: "allowlist", activeChatsCount: 0 },
  { id: "signal", name: "Signal", protocol: "libsignal Daemon", status: "disconnected", dmPolicy: "pairing", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "line", name: "LINE", protocol: "Messaging API", status: "disconnected", dmPolicy: "open", groupPolicy: "open", activeChatsCount: 0 },
  { id: "msteams", name: "Microsoft Teams", protocol: "Graph / Bot Framework", status: "disconnected", dmPolicy: "allowlist", groupPolicy: "allowlist", activeChatsCount: 0 },
  { id: "googlechat", name: "Google Chat", protocol: "Workspace API", status: "disconnected", dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 0 },
  { id: "mattermost", name: "Mattermost", protocol: "REST / WebSocket", status: "disconnected", dmPolicy: "open", groupPolicy: "open", activeChatsCount: 0 },
  { id: "irc", name: "IRC", protocol: "IRCv3 RFC 1459", status: "disconnected", dmPolicy: "pairing", groupPolicy: "open", activeChatsCount: 0 },
  { id: "twitch", name: "Twitch", protocol: "Twitch IRC", status: "disconnected", dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 0 },
  { id: "nostr", name: "Nostr", protocol: "NIP-01 Relay Pool", status: "disconnected", dmPolicy: "pairing", groupPolicy: "open", activeChatsCount: 0 },
  { id: "zalo", name: "Zalo", protocol: "Zalo Official Account", status: "disconnected", dmPolicy: "allowlist", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "bluebubbles", name: "BlueBubbles (iMessage)", protocol: "Bridge API", status: "disconnected", dmPolicy: "pairing", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "nextcloudtalk", name: "Nextcloud Talk", protocol: "Nextcloud Talk API", status: "disconnected", dmPolicy: "open", groupPolicy: "open", activeChatsCount: 0 },
  { id: "synologychat", name: "Synology Chat", protocol: "Bot Webhook", status: "disconnected", dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 0 },
];

export function ExternalChannelsSection({ agentId: _agentId }: ExternalChannelsSectionProps) {
  const [channels, setChannels] = useState<ChannelItem[]>(() => {
    try {
      const saved = localStorage.getItem("sand_agent_channels");
      if (saved) return JSON.parse(saved);
    } catch {
      // fallback
    }
    return INITIAL_CHANNELS;
  });

  const [search, setSearch] = useState("");
  const [editingId, setEditingId] = useState<ChannelId | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  const persist = (updated: ChannelItem[]) => {
    setChannels(updated);
    try {
      localStorage.setItem("sand_agent_channels", JSON.stringify(updated));
    } catch {
      // ignore
    }
  };

  const toggleStatus = (id: ChannelId) => {
    const updated = channels.map((c) => {
      if (c.id === id) {
        const next = c.status === "connected" ? "disconnected" : "connected";
        return { ...c, status: next as any, activeChatsCount: next === "connected" ? 1 : 0 };
      }
      return c;
    });
    persist(updated);
    showToast(`渠道状态已切换`);
  };

  const updateDmPolicy = (id: ChannelId, dmPolicy: DmPolicy) => {
    const updated = channels.map((c) => (c.id === id ? { ...c, dmPolicy } : c));
    persist(updated);
  };

  const updateGroupPolicy = (id: ChannelId, groupPolicy: GroupPolicy) => {
    const updated = channels.map((c) => (c.id === id ? { ...c, groupPolicy } : c));
    persist(updated);
  };

  const filtered = channels.filter((c) => {
    if (!search) return true;
    return c.name.toLowerCase().includes(search.toLowerCase()) || c.protocol.toLowerCase().includes(search.toLowerCase());
  });

  const connectedCount = channels.filter((c) => c.status === "connected").length;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {toast && (
        <div className="sand-agent-action-toast">
          <SandIcon name="chat-bubbles" size="sm" />
          <span>{toast}</span>
        </div>
      )}

      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>外部消息渠道适配器 ({connectedCount}/{channels.length} 活跃)</span>
        </div>
        <p style={{ margin: "0 0 10px", fontSize: 11, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
          作为 Fabushi Agent 渠道/插件适配器运行。支持独立 DM 配对门禁与群组权限策略，数据严格在本地隔离。
        </p>

        <input
          type="text"
          placeholder="搜索渠道平台或协议..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{
            width: "100%",
            padding: "5px 10px",
            fontSize: 11,
            background: "rgba(0,0,0,0.3)",
            border: "1px solid var(--cursor-border, rgba(255,255,255,0.12))",
            borderRadius: 4,
            color: "#fff",
            marginBottom: 10,
            outline: "none",
          }}
        />

        <div className="sand-runtime-channel-list">
          {filtered.map((c) => (
            <div key={c.id} className="sand-runtime-channel-row">
              <div style={{ display: "flex", flexDirection: "column", gap: 2, flex: 1 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ fontWeight: 600 }}>{c.name}</span>
                  <span className={`sand-runtime-pill sand-runtime-pill--${c.status === "connected" ? "green" : "gray"}`}>
                    {c.status === "connected" ? "已连接" : "已断开"}
                  </span>
                </div>
                <div style={{ fontSize: 10, color: "var(--cursor-text-secondary, #a1a1aa)" }}>
                  协议: {c.protocol} {c.status === "connected" && `• 会话: ${c.activeChatsCount}`}
                </div>

                <div style={{ display: "flex", gap: 8, marginTop: 4, fontSize: 10 }}>
                  <div>
                    <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>私聊: </span>
                    <select
                      value={c.dmPolicy}
                      onChange={(e) => updateDmPolicy(c.id, e.target.value as DmPolicy)}
                      style={{ background: "rgba(0,0,0,0.4)", color: "#fff", border: "1px solid rgba(255,255,255,0.1)", borderRadius: 3, fontSize: 10 }}
                    >
                      <option value="open">开放</option>
                      <option value="allowlist">白名单</option>
                      <option value="pairing">需配对</option>
                    </select>
                  </div>
                  <div>
                    <span style={{ color: "var(--cursor-text-secondary, #a1a1aa)" }}>群组: </span>
                    <select
                      value={c.groupPolicy}
                      onChange={(e) => updateGroupPolicy(c.id, e.target.value as GroupPolicy)}
                      style={{ background: "rgba(0,0,0,0.4)", color: "#fff", border: "1px solid rgba(255,255,255,0.1)", borderRadius: 3, fontSize: 10 }}
                    >
                      <option value="open">开放</option>
                      <option value="allowlist">白名单</option>
                      <option value="disabled">禁止</option>
                    </select>
                  </div>
                </div>

                {editingId === c.id && (
                  <div style={{ marginTop: 6, padding: 6, background: "rgba(0,0,0,0.25)", borderRadius: 4 }}>
                    <div style={{ fontSize: 10, color: "var(--cursor-text-secondary, #a1a1aa)", marginBottom: 2 }}>凭证/Token:</div>
                    <input
                      type="password"
                      defaultValue="••••••••••••••••"
                      style={{ width: "100%", padding: "3px 6px", fontSize: 10, background: "rgba(0,0,0,0.3)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: 3, color: "#fff" }}
                    />
                  </div>
                )}
              </div>

              <div style={{ display: "flex", flexDirection: "column", gap: 4, marginLeft: 8 }}>
                <SandButton onClick={() => toggleStatus(c.id)} size="sm" variant={c.status === "connected" ? "secondary" : "primary"}>
                  {c.status === "connected" ? "断开" : "连接"}
                </SandButton>
                <button
                  onClick={() => setEditingId(editingId === c.id ? null : c.id)}
                  style={{ background: "transparent", border: "none", color: "var(--cursor-text-secondary, #a1a1aa)", fontSize: 10, cursor: "pointer" }}
                >
                  {editingId === c.id ? "收起" : "配置"}
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
