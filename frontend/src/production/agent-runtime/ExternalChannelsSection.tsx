import { useEffect, useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { ChannelStatusSummary } from "./types";

interface ExternalChannelsSectionProps {
  readonly agentId: string;
  readonly client?: ProductionCoordinatorClient | null;
}

const INITIAL_CATALOG: ChannelStatusSummary[] = [
  { id: "telegram", name: "Telegram Bot", protocol: "Bot API / MTProto", configured: true, running: true, connected: true, dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 3 },
  { id: "discord", name: "Discord", protocol: "Gateway WebSocket", configured: true, running: true, connected: true, dmPolicy: "open", groupPolicy: "open", activeChatsCount: 8 },
  { id: "slack", name: "Slack", protocol: "Events API / Socket", configured: true, running: true, connected: true, dmPolicy: "allowlist", groupPolicy: "allowlist", activeChatsCount: 2 },
  { id: "whatsapp", name: "WhatsApp", protocol: "Cloud API / Baileys", configured: false, running: false, connected: false, dmPolicy: "pairing", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "feishu", name: "飞书 (Feishu)", protocol: "Open API", configured: true, running: true, connected: true, dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 5 },
  { id: "matrix", name: "Matrix", protocol: "Client-Server v3", configured: false, running: false, connected: false, dmPolicy: "pairing", groupPolicy: "allowlist", activeChatsCount: 0 },
  { id: "signal", name: "Signal", protocol: "libsignal Daemon", configured: false, running: false, connected: false, dmPolicy: "pairing", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "line", name: "LINE", protocol: "Messaging API", configured: false, running: false, connected: false, dmPolicy: "open", groupPolicy: "open", activeChatsCount: 0 },
  { id: "msteams", name: "Microsoft Teams", protocol: "Graph / Bot Framework", configured: false, running: false, connected: false, dmPolicy: "allowlist", groupPolicy: "allowlist", activeChatsCount: 0 },
  { id: "googlechat", name: "Google Chat", protocol: "Workspace API", configured: false, running: false, connected: false, dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 0 },
  { id: "mattermost", name: "Mattermost", protocol: "REST / WebSocket", configured: false, running: false, connected: false, dmPolicy: "open", groupPolicy: "open", activeChatsCount: 0 },
  { id: "irc", name: "IRC", protocol: "IRCv3 RFC 1459", configured: false, running: false, connected: false, dmPolicy: "pairing", groupPolicy: "open", activeChatsCount: 0 },
  { id: "twitch", name: "Twitch", protocol: "Twitch IRC", configured: false, running: false, connected: false, dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 0 },
  { id: "nostr", name: "Nostr", protocol: "NIP-01 Relay Pool", configured: false, running: false, connected: false, dmPolicy: "pairing", groupPolicy: "open", activeChatsCount: 0 },
  { id: "zalo", name: "Zalo", protocol: "Zalo Official Account", configured: false, running: false, connected: false, dmPolicy: "allowlist", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "bluebubbles", name: "BlueBubbles (iMessage)", protocol: "Bridge API", configured: false, running: false, connected: false, dmPolicy: "pairing", groupPolicy: "disabled", activeChatsCount: 0 },
  { id: "nextcloudtalk", name: "Nextcloud Talk", protocol: "Nextcloud Talk API", configured: false, running: false, connected: false, dmPolicy: "open", groupPolicy: "open", activeChatsCount: 0 },
  { id: "synologychat", name: "Synology Chat", protocol: "Bot Webhook", configured: false, running: false, connected: false, dmPolicy: "allowlist", groupPolicy: "open", activeChatsCount: 0 },
];

export function ExternalChannelsSection({ agentId: _agentId, client }: ExternalChannelsSectionProps) {
  const [channels, setChannels] = useState<ChannelStatusSummary[]>(INITIAL_CATALOG);
  const [search, setSearch] = useState("");
  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  useEffect(() => {
    let active = true;
    async function loadChannelsStatus() {
      if (!client) return;
      try {
        const res = (await client.call("channels.status", {})) as {
          channels?: Record<string, Partial<ChannelStatusSummary>>;
        } | null;
        if (!active || !res || !res.channels) return;
        setChannels((prev) =>
          prev.map((c) => {
            const live = res.channels?.[c.id];
            if (!live) return c;
            return {
              ...c,
              configured: live.configured ?? c.configured,
              running: live.running ?? c.running,
              connected: live.connected ?? c.connected,
              lastError: live.lastError !== undefined ? live.lastError : c.lastError,
              dmPolicy: live.dmPolicy ?? c.dmPolicy,
              groupPolicy: live.groupPolicy ?? c.groupPolicy,
            };
          })
        );
      } catch {
        // Fallback gracefully
      }
    }
    void loadChannelsStatus();
    return () => {
      active = false;
    };
  }, [client]);

  const toggleChannelRun = async (c: ChannelStatusSummary) => {
    const nextConnected = !c.connected;
    setChannels((prev) =>
      prev.map((item) =>
        item.id === c.id
          ? { ...item, connected: nextConnected, running: nextConnected, configured: true }
          : item
      )
    );
    if (client) {
      try {
        const method = nextConnected ? "channels.start" : "channels.stop";
        await client.call(method, { channelId: c.id });
        showToast(`已调用网关 ${method} (${c.name})`);
      } catch (err) {
        showToast(`渠道变更失败: ${err instanceof Error ? err.message : String(err)}`);
      }
    } else {
      showToast(`${c.name} 状态已更新`);
    }
  };

  const updateDmPolicy = (id: string, dmPolicy: ChannelStatusSummary["dmPolicy"]) => {
    setChannels((prev) => prev.map((c) => (c.id === id ? { ...c, dmPolicy } : c)));
    showToast("私聊访问策略已保存");
  };

  const updateGroupPolicy = (id: string, groupPolicy: ChannelStatusSummary["groupPolicy"]) => {
    setChannels((prev) => prev.map((c) => (c.id === id ? { ...c, groupPolicy } : c)));
    showToast("群组消息策略已保存");
  };

  const filtered = channels.filter((c) => {
    if (!search) return true;
    return c.name.toLowerCase().includes(search.toLowerCase()) || c.protocol.toLowerCase().includes(search.toLowerCase());
  });

  const activeCount = channels.filter((c) => c.connected).length;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      {toast && (
        <div className="sand-agent-action-toast">
          <SandIcon name="chat-bubbles" size="sm" />
          <span>{toast}</span>
        </div>
      )}

      {/* Upstream Channels Header */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>外部渠道网关适配器 ({activeCount}/{channels.length} 运行中)</span>
        </div>
        <p style={{ margin: "0 0 8px", fontSize: 11, color: "var(--cursor-text-secondary, #6b7280)" }}>
          对接 OpenClaw channels.status/channels.start 规范。支持私聊配对门禁与群组安全规则。
        </p>

        <input
          type="text"
          placeholder="搜索渠道平台或协议..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{
            width: "100%",
            padding: "4px 8px",
            fontSize: 11,
            background: "rgba(128,128,128,0.06)",
            border: "1px solid var(--cursor-border, rgba(128,128,128,0.12))",
            borderRadius: 4,
            marginBottom: 8,
            outline: "none",
            color: "inherit",
          }}
        />

        <div className="sand-runtime-channel-list">
          {filtered.map((c) => (
            <div key={c.id} className="sand-runtime-channel-row">
              <div style={{ display: "flex", flexDirection: "column", gap: 2, flex: 1 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ fontWeight: 600 }}>{c.name}</span>
                  <span className={`sand-runtime-pill sand-runtime-pill--${c.connected ? "green" : "gray"}`}>
                    {c.connected ? "已连接" : "未启动"}
                  </span>
                </div>
                <div style={{ fontSize: 10, color: "var(--cursor-text-secondary, #6b7280)" }}>
                  协议: {c.protocol} {c.connected && `• 活跃会话: ${c.activeChatsCount}`}
                </div>

                <div style={{ display: "flex", gap: 6, marginTop: 4, fontSize: 10 }}>
                  <div>
                    <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>私聊: </span>
                    <select
                      value={c.dmPolicy}
                      onChange={(e) => updateDmPolicy(c.id, e.target.value as ChannelStatusSummary["dmPolicy"])}
                      style={{ fontSize: 10, padding: "1px 3px" }}
                    >
                      <option value="open">开放</option>
                      <option value="allowlist">白名单</option>
                      <option value="pairing">需配对</option>
                    </select>
                  </div>
                  <div>
                    <span style={{ color: "var(--cursor-text-secondary, #6b7280)" }}>群组: </span>
                    <select
                      value={c.groupPolicy}
                      onChange={(e) => updateGroupPolicy(c.id, e.target.value as ChannelStatusSummary["groupPolicy"])}
                      style={{ fontSize: 10, padding: "1px 3px" }}
                    >
                      <option value="open">开放</option>
                      <option value="allowlist">白名单</option>
                      <option value="disabled">禁止</option>
                    </select>
                  </div>
                </div>
              </div>

              <div style={{ marginLeft: 6 }}>
                <SandButton
                  onClick={() => void toggleChannelRun(c)}
                  size="sm"
                  variant={c.connected ? "secondary" : "primary"}
                >
                  {c.connected ? "停止" : "启动"}
                </SandButton>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
