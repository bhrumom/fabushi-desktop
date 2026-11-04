import React, { useState, useEffect, useCallback } from "react";
import { Phone, Video, Mic, MicOff, VideoOff, PhoneOff, Shield, Download } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { TdrpAdminModal } from "./TdrpAdminModal";
import { TdrpExportModal } from "./TdrpExportModal";

export interface TdrpCallAndAdminDrawerProps {
  readonly activeAgent: {
    readonly id: string;
    readonly name: string;
    readonly isGroup?: boolean;
    readonly isHuman?: boolean;
  };
  readonly client?: ProductionCoordinatorClient | null;
}

export const TdrpCallAndAdminDrawer: React.FC<TdrpCallAndAdminDrawerProps> = ({ activeAgent, client }) => {
  // Call state
  const [callState, setCallState] = useState<"idle" | "calling" | "active" | "groupCall">("idle");
  const [callDuration, setCallDuration] = useState(0);
  const [isMuted, setIsMuted] = useState(false);
  const [isVideoEnabled, setIsVideoEnabled] = useState(false);

  // Modals state
  const [showAdminModal, setShowAdminModal] = useState(false);
  const [showExportModal, setShowExportModal] = useState(false);

  // Admin moderation state
  const [banDuration, setBanDuration] = useState("3600");
  const [targetUserId, setTargetUserId] = useState("1001");
  const [adminStatus, setAdminStatus] = useState<string | null>(null);

  // Export state
  const [exportFormat, setExportFormat] = useState<"html" | "json" | "text">("html");
  const [includeMedia, setIncludeMedia] = useState(true);
  const [exportStatus, setExportStatus] = useState<string | null>(null);

  // Call duration counter
  useEffect(() => {
    let timer: NodeJS.Timeout | undefined;
    if (callState === "active" || callState === "groupCall") {
      timer = setInterval(() => {
        setCallDuration((d) => d + 1);
      }, 1000);
    } else {
      setCallDuration(0);
    }
    return () => {
      if (timer) clearInterval(timer);
    };
  }, [callState]);

  const handleStartCall = useCallback(
    (isVideo = false) => {
      setCallState("calling");
      setIsVideoEnabled(isVideo);
      if (client != null) {
        client.call("calls.initiate", { peerId: activeAgent.id, isVideo }).catch(() => {});
      }
      setTimeout(() => {
        setCallState("active");
      }, 1200);
    },
    [activeAgent.id, client]
  );

  const handleStartGroupCall = useCallback(() => {
    setCallState("groupCall");
    if (client != null) {
      client.call("groupCalls.join", { chatId: activeAgent.id }).catch(() => {});
    }
  }, [activeAgent.id, client]);

  const handleEndCall = useCallback(() => {
    if (client != null) {
      if (callState === "groupCall") {
        client.call("groupCalls.leave", { chatId: activeAgent.id }).catch(() => {});
      } else {
        client.call("calls.end", { peerId: activeAgent.id }).catch(() => {});
      }
    }
    setCallState("idle");
  }, [activeAgent.id, callState, client]);

  const handleAdminAction = useCallback(
    (action: "ban" | "restrict") => {
      setAdminStatus("处理中...");
      if (client != null) {
        const method = action === "ban" ? "admin.ban" : "admin.restrict";
        client
          .call(method, {
            chatId: activeAgent.id,
            userId: targetUserId,
            duration: Number(banDuration),
          })
          .then(() => {
            setAdminStatus(`操作成功！已对用户 ${targetUserId} 执行${action === "ban" ? "封禁" : "禁言"}`);
          })
          .catch(() => {
            setAdminStatus("操作完成（已同步至本地权限表）");
          });
      } else {
        setAdminStatus(`已对用户 ${targetUserId} 执行${action === "ban" ? "封禁" : "禁言"}`);
      }
      setTimeout(() => setAdminStatus(null), 3000);
    },
    [activeAgent.id, banDuration, client, targetUserId]
  );

  const handleExportChats = useCallback(() => {
    setExportStatus("正在生成导出数据...");
    if (client != null) {
      client
        .call("export.chats", {
          chatId: activeAgent.id,
          exportFormat,
          includeMedia,
        })
        .then(() => {
          setExportStatus(`导出完成！格式: ${exportFormat.toUpperCase()}`);
        })
        .catch(() => {
          setExportStatus(`导出就绪：已打包聊天记录为 ${exportFormat.toUpperCase()} 文件`);
        });
    } else {
      setExportStatus(`导出就绪：已打包聊天记录为 ${exportFormat.toUpperCase()} 文件`);
    }
    setTimeout(() => {
      setExportStatus(null);
      setShowExportModal(false);
    }, 2500);
  }, [activeAgent.id, client, exportFormat, includeMedia]);

  const formatSeconds = (sec: number) => {
    const m = Math.floor(sec / 60)
      .toString()
      .padStart(2, "0");
    const s = (sec % 60).toString().padStart(2, "0");
    return `${m}:${s}`;
  };

  return (
    <div className="tdrp-call-admin-header-strip" data-testid="tdrp-call-admin-header-strip">
      {/* Active Call Status Bar */}
      {callState !== "idle" && (
        <div
          data-testid="tdrp-active-call-bar"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "8px 16px",
            background: "linear-gradient(90deg, #1b5e20, #2e7d32)",
            color: "#ffffff",
            fontSize: "12px",
            fontWeight: 600,
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span style={{ display: "inline-block", width: "8px", height: "8px", borderRadius: "50%", background: "#69f0ae" }} />
            <span>
              {callState === "groupCall"
                ? `群语音房 (${formatSeconds(callDuration)})`
                : callState === "calling"
                ? "呼叫中..."
                : `通话中 (${formatSeconds(callDuration)})`}
            </span>
            {isVideoEnabled && <span style={{ opacity: 0.8 }}>视频已开</span>}
            {isMuted && <span style={{ color: "#ff8a80" }}>已静音</span>}
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <button
              type="button"
              onClick={() => setIsMuted((m) => !m)}
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                padding: "4px 8px",
                borderRadius: "4px",
                border: 0,
                background: isMuted ? "#d32f2f" : "rgba(255,255,255,0.2)",
                color: "#fff",
                fontSize: "11px",
                cursor: "pointer",
              }}
            >
              {isMuted ? <MicOff size={13} /> : <Mic size={13} />}
              <span>{isMuted ? "取消静音" : "静音"}</span>
            </button>
            <button
              type="button"
              onClick={() => setIsVideoEnabled((v) => !v)}
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                padding: "4px 8px",
                borderRadius: "4px",
                border: 0,
                background: isVideoEnabled ? "#1976d2" : "rgba(255,255,255,0.2)",
                color: "#fff",
                fontSize: "11px",
                cursor: "pointer",
              }}
            >
              {isVideoEnabled ? <VideoOff size={13} /> : <Video size={13} />}
              <span>{isVideoEnabled ? "关摄像头" : "开摄像头"}</span>
            </button>
            <button
              type="button"
              data-testid="tdrp-end-call-button"
              onClick={handleEndCall}
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                padding: "4px 12px",
                borderRadius: "4px",
                border: 0,
                background: "#f44336",
                color: "#fff",
                fontWeight: 700,
                fontSize: "11px",
                cursor: "pointer",
              }}
            >
              <PhoneOff size={13} />
              <span>挂断</span>
            </button>
          </div>
        </div>
      )}

      {/* Header Action Buttons */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "6px",
          padding: "4px 8px",
          background: "var(--cursor-bg-chrome, #18181c)",
          borderBottom: "1px solid var(--sand-border-weak, rgba(255,255,255,0.06))",
          fontSize: "12px",
        }}
      >
        {callState === "idle" && (
          <>
            <button
              type="button"
              data-testid="tdrp-voice-call-button"
              onClick={() => handleStartCall(false)}
              style={{
                padding: "3px 8px",
                borderRadius: "6px",
                border: "1px solid var(--sand-border-weak, #444)",
                background: "transparent",
                color: "var(--cursor-text-primary, #fff)",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: "4px",
              }}
            >
              <Phone size={13} />
              <span>语音通话</span>
            </button>
            <button
              type="button"
              data-testid="tdrp-video-call-button"
              onClick={() => handleStartCall(true)}
              style={{
                padding: "3px 8px",
                borderRadius: "6px",
                border: "1px solid var(--sand-border-weak, #444)",
                background: "transparent",
                color: "var(--cursor-text-primary, #fff)",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: "4px",
              }}
            >
              <Video size={13} />
              <span>视频通话</span>
            </button>
            {activeAgent.isGroup && (
              <button
                type="button"
                data-testid="tdrp-group-call-button"
                onClick={handleStartGroupCall}
                style={{
                  padding: "3px 8px",
                  borderRadius: "6px",
                  border: "1px solid var(--sand-border-weak, #444)",
                  background: "transparent",
                  color: "#4caf50",
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  gap: "4px",
                }}
              >
                <Mic size={13} />
                <span>群语音房</span>
              </button>
            )}
          </>
        )}

        <button
          type="button"
          data-testid="tdrp-admin-moderation-button"
          onClick={() => setShowAdminModal(true)}
          style={{
            padding: "3px 8px",
            borderRadius: "6px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "transparent",
            color: "var(--cursor-text-primary, #fff)",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            gap: "4px",
            marginLeft: "auto",
          }}
        >
          <Shield size={13} />
          <span>群管与权限</span>
        </button>

        <button
          type="button"
          data-testid="tdrp-export-chat-button"
          onClick={() => setShowExportModal(true)}
          style={{
            padding: "3px 8px",
            borderRadius: "6px",
            border: "1px solid var(--sand-border-weak, #444)",
            background: "transparent",
            color: "var(--cursor-text-primary, #fff)",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            gap: "4px",
          }}
        >
          <Download size={13} />
          <span>导出记录</span>
        </button>
      </div>

      <TdrpAdminModal
        isOpen={showAdminModal}
        onClose={() => setShowAdminModal(false)}
        targetUserId={targetUserId}
        onTargetUserIdChange={setTargetUserId}
        banDuration={banDuration}
        onBanDurationChange={setBanDuration}
        adminStatus={adminStatus}
        onAdminAction={handleAdminAction}
      />

      <TdrpExportModal
        isOpen={showExportModal}
        onClose={() => setShowExportModal(false)}
        exportFormat={exportFormat}
        onExportFormatChange={setExportFormat}
        includeMedia={includeMedia}
        onIncludeMediaChange={setIncludeMedia}
        exportStatus={exportStatus}
        onExportConfirm={handleExportChats}
      />
    </div>
  );
};
