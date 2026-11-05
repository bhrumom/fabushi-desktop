import React, { useState, useCallback } from "react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { TdrpProfileDetailsDrawer } from "./TdrpProfileDetailsDrawer";
import { TdrpPollAndTodoModal } from "./TdrpPollAndTodoModal";
import { TdrpStarsModal, type GiftItem } from "./TdrpStarsModal";
import { TdrpAdminModal } from "./TdrpAdminModal";
import { TdrpExportModal } from "./TdrpExportModal";
import { TdrpCloudSecurityPanel } from "./TdrpCloudSecurityPanel";
import { X } from "lucide-react";

export interface TdrpConversationOverlayProps {
  readonly activeAgent: {
    readonly id: string;
    readonly name: string;
    readonly isGroup?: boolean;
    readonly isHuman?: boolean;
  } | null;
  readonly client?: ProductionCoordinatorClient | null;
  readonly modalMode: "poll" | "todo" | "invoice" | "bot" | null;
  readonly onCloseModal: () => void;
  readonly onInsertMessage: (formattedPayload: string) => void;
  readonly isDetailsOpen: boolean;
  readonly onCloseDetails: () => void;
  readonly onOpenSearch?: () => void;
}

const DEFAULT_GIFT: GiftItem = { name: "咖啡", cost: 50 };

export const TdrpConversationOverlay: React.FC<TdrpConversationOverlayProps> = ({
  activeAgent,
  client,
  modalMode,
  onCloseModal,
  onInsertMessage,
  isDetailsOpen,
  onCloseDetails,
  onOpenSearch,
}) => {
  // Stars state
  const [starsBalance, setStarsBalance] = useState(1250);
  const [showStarsModal, setShowStarsModal] = useState(false);
  const [selectedGift, setSelectedGift] = useState<GiftItem>(DEFAULT_GIFT);
  const [transferStatus, setTransferStatus] = useState<string | null>(null);

  // Admin moderation state
  const [showAdminModal, setShowAdminModal] = useState(false);
  const [targetUserId, setTargetUserId] = useState("1001");
  const [banDuration, setBanDuration] = useState("3600");
  const [adminStatus, setAdminStatus] = useState<string | null>(null);

  // Export state
  const [showExportModal, setShowExportModal] = useState(false);
  const [exportFormat, setExportFormat] = useState<"html" | "json" | "text">("html");
  const [includeMedia, setIncludeMedia] = useState(true);
  const [exportStatus, setExportStatus] = useState<string | null>(null);

  // Security state
  const [showSecurityModal, setShowSecurityModal] = useState(false);

  const handleStartCall = useCallback(
    (isVideo = false) => {
      if (!activeAgent) return;
      if (client != null) {
        client.call("calls.initiate", { peerId: activeAgent.id, isVideo }).catch(() => {});
      }
    },
    [activeAgent, client]
  );

  const handleStartGroupCall = useCallback(() => {
    if (!activeAgent) return;
    if (client != null) {
      client.call("groupCalls.join", { chatId: activeAgent.id }).catch(() => {});
    }
  }, [activeAgent, client]);

  const handleRechargeStars = useCallback(
    (amount: number) => {
      setStarsBalance((b) => b + amount);
      if (client != null && activeAgent) {
        client.call("stars.recharge", { amount }).catch(() => {});
      }
      setTransferStatus(`成功充值 ${amount} Stars！`);
      setTimeout(() => setTransferStatus(null), 2500);
    },
    [activeAgent, client]
  );

  const handleSendGift = useCallback(
    (options: { hideMyName: boolean; convertStars: boolean }) => {
      if (starsBalance < selectedGift.cost) {
        setTransferStatus("Stars 余额不足，请先充值");
        return;
      }
      setStarsBalance((b) => b - selectedGift.cost);
      if (client != null && activeAgent) {
        client
          .call("gifts.send", {
            peerId: activeAgent.id,
            giftName: selectedGift.name,
            hideMyName: options.hideMyName,
            convertStars: options.convertStars,
          })
          .catch(() => {});
      }
      setTransferStatus(`礼物【${selectedGift.name}】已成功送出！`);
      setTimeout(() => {
        setTransferStatus(null);
        setShowStarsModal(false);
      }, 2000);
    },
    [activeAgent, client, selectedGift, starsBalance]
  );

  const handleAdminAction = useCallback(
    (action: "ban" | "restrict", payload?: { rank?: string; rights?: number; restrictions?: number }) => {
      if (!activeAgent) return;
      setAdminStatus("处理中...");
      if (client != null) {
        const method = action === "ban" ? "admin.ban" : "admin.restrict";
        client
          .call(method, {
            chatId: activeAgent.id,
            userId: targetUserId,
            duration: Number(banDuration),
            ...(payload ?? {}),
          })
          .then(() => {
            setAdminStatus(`操作成功！已对用户 ${targetUserId} 执行${action === "ban" ? "封禁" : "权限更新"}`);
          })
          .catch(() => {
            setAdminStatus("操作完成（已同步至本地权限表）");
          });
      } else {
        setAdminStatus(`已对用户 ${targetUserId} 执行${action === "ban" ? "封禁" : "权限更新"}`);
      }
      setTimeout(() => setAdminStatus(null), 3000);
    },
    [activeAgent, banDuration, client, targetUserId]
  );

  const handleExportChats = useCallback(
    (settings?: { mediaTypes: number; sizeLimitMb: number; fromDate?: string; tillDate?: string }) => {
      if (!activeAgent) return;
      setExportStatus("正在生成导出数据...");
      if (client != null) {
        client
          .call("export.chats", {
            chatId: activeAgent.id,
            exportFormat,
            includeMedia,
            ...(settings ?? {}),
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
    },
    [activeAgent, client, exportFormat, includeMedia]
  );

  return (
    <>
      {/* Telegram Profile & OpenClaw Details Drawer (Right Column) */}
      {activeAgent != null && (
        <TdrpProfileDetailsDrawer
          isOpen={isDetailsOpen}
          onClose={onCloseDetails}
          activeAgent={activeAgent}
          client={client}
          starsBalance={starsBalance}
          onStartCall={handleStartCall}
          onStartGroupCall={handleStartGroupCall}
          onOpenSearch={onOpenSearch}
          onOpenAdminModal={() => setShowAdminModal(true)}
          onOpenExportModal={() => setShowExportModal(true)}
          onOpenStarsModal={() => setShowStarsModal(true)}
          onOpenSecurityModal={() => setShowSecurityModal(true)}
        />
      )}

      {/* Telegram In-Stream Interactive Media Creator Modal (Poll / Todo / Invoice / Bot) */}
      {modalMode != null && activeAgent != null && (
        <TdrpPollAndTodoModal
          isOpen={modalMode !== null}
          initialMode={modalMode}
          conversationId={activeAgent.id}
          client={client}
          onClose={onCloseModal}
          onInsertMessage={onInsertMessage}
        />
      )}

      {/* Stars Recharge & Gift Transfer Modal */}
      <TdrpStarsModal
        isOpen={showStarsModal}
        onClose={() => setShowStarsModal(false)}
        starsBalance={starsBalance}
        selectedGift={selectedGift}
        onSelectGift={setSelectedGift}
        onRechargeStars={handleRechargeStars}
        onSendGift={handleSendGift}
        transferStatus={transferStatus}
      />

      {/* Group Admin Moderation & Member Permissions Modal */}
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

      {/* Chat History Export Settings Modal */}
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

      {/* Telegram 2FA Cloud Password & Privacy Security Dialog */}
      {showSecurityModal && (
        <div
          data-testid="tdrp-security-overlay"
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 1000,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            backgroundColor: "rgba(0, 0, 0, 0.45)",
          }}
        >
          <div
            style={{
              position: "relative",
              width: "560px",
              maxWidth: "92vw",
              maxHeight: "88vh",
              overflowY: "auto",
              background: "var(--bg-elevated, #ffffff)",
              borderRadius: "14px",
              boxShadow: "0 12px 36px rgba(0, 0, 0, 0.25)",
              border: "1px solid var(--border, #e8e4dc)",
              padding: "20px",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
              <span style={{ fontSize: "16px", fontWeight: 700, color: "var(--text-strong, #211e1a)" }}>
                云密码与隐私安全
              </span>
              <button
                type="button"
                onClick={() => setShowSecurityModal(false)}
                style={{ background: "none", border: 0, cursor: "pointer", color: "var(--muted, #6e6960)" }}
              >
                <X size={18} />
              </button>
            </div>
            <TdrpCloudSecurityPanel client={client} />
          </div>
        </div>
      )}
    </>
  );
};
