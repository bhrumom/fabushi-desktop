import React, { useState } from "react";
import { Sparkles, Gift, X } from "lucide-react";

export interface GiftItem {
  readonly name: string;
  readonly cost: number;
  readonly limitedCount?: number;
  readonly convertStars?: boolean;
}

export interface TdrpStarsModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly starsBalance: number;
  readonly selectedGift: GiftItem;
  readonly onSelectGift: (gift: GiftItem) => void;
  readonly onRechargeStars: (amount: number) => void;
  readonly onSendGift: (options: { hideMyName: boolean; convertStars: boolean }) => void;
  readonly transferStatus: string | null;
}

const RECHARGE_TIERS = [
  { amount: 100, price: "¥12" },
  { amount: 500, price: "¥58" },
  { amount: 1000, price: "¥108" },
  { amount: 2500, price: "¥258" },
];

const GIFT_OPTIONS: readonly GiftItem[] = [
  { name: "咖啡", cost: 50 },
  { name: "赞赏", cost: 100 },
  { name: "限量奖杯", cost: 500, limitedCount: 500, convertStars: true },
  { name: "限量火箭", cost: 1000, limitedCount: 100, convertStars: true },
];

export const TdrpStarsModal: React.FC<TdrpStarsModalProps> = ({
  isOpen,
  onClose,
  starsBalance,
  selectedGift,
  onSelectGift,
  onRechargeStars,
  onSendGift,
  transferStatus,
}) => {
  const [hideMyName, setHideMyName] = useState(false);
  const [convertStars, setConvertStars] = useState(true);

  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      data-testid="tdrp-stars-modal"
      onClick={onClose}
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: "rgba(0, 0, 0, 0.65)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 9999,
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: "var(--sand-surface-backdrop, #242429)",
          padding: "20px 24px",
          borderRadius: "16px",
          width: "360px",
          color: "var(--sand-text-primary, #ffffff)",
          boxShadow: "0 10px 30px rgba(0,0,0,0.5)",
          border: "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
          <h3 style={{ margin: 0, fontSize: "16px", fontWeight: 700, display: "flex", alignItems: "center", gap: "6px" }}>
            <Sparkles size={18} color="#ffc107" /> 法布施 Stars & 赠礼
          </h3>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            style={{ background: "none", border: 0, color: "#888", cursor: "pointer" }}
          >
            <X size={18} />
          </button>
        </div>

        <div
          style={{
            background: "rgba(255, 193, 7, 0.12)",
            border: "1px solid rgba(255, 193, 7, 0.3)",
            borderRadius: "10px",
            padding: "10px",
            textAlign: "center",
            marginBottom: "14px",
          }}
        >
          <div style={{ fontSize: "11px", opacity: 0.8 }}>当前余额 (Balance)</div>
          <div style={{ fontSize: "22px", fontWeight: 700, color: "#ffc107", marginTop: "2px" }}>
            {starsBalance.toLocaleString()} Stars
          </div>
        </div>

        <div style={{ fontSize: "12px", fontWeight: 600, marginBottom: "6px" }}>充值 Stars</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "6px", marginBottom: "14px" }}>
          {RECHARGE_TIERS.map((tier) => (
            <button
              key={tier.amount}
              type="button"
              onClick={() => onRechargeStars(tier.amount)}
              style={{
                padding: "6px 8px",
                borderRadius: "8px",
                border: "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))",
                background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
                color: "inherit",
                cursor: "pointer",
                fontSize: "11px",
                display: "flex",
                justifyContent: "space-between",
              }}
            >
              <span style={{ color: "#ffc107", fontWeight: 600 }}>{tier.amount} Stars</span>
              <span style={{ opacity: 0.7 }}>{tier.price}</span>
            </button>
          ))}
        </div>

        <div style={{ fontSize: "12px", fontWeight: 600, marginBottom: "6px" }}>赠礼中心 (Star Gifts)</div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "6px", marginBottom: "12px" }}>
          {GIFT_OPTIONS.map((gift) => (
            <div
              key={gift.name}
              onClick={() => onSelectGift(gift)}
              style={{
                padding: "6px 4px",
                borderRadius: "8px",
                border: selectedGift.name === gift.name ? "1px solid #ffc107" : "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))",
                background: selectedGift.name === gift.name ? "rgba(255, 193, 7, 0.15)" : "transparent",
                textAlign: "center",
                cursor: "pointer",
              }}
            >
              <Gift size={16} style={{ margin: "0 auto", color: selectedGift.name === gift.name ? "#ffc107" : "inherit" }} />
              <div style={{ fontSize: "11px", marginTop: "3px" }}>{gift.name}</div>
              <div style={{ fontSize: "9px", color: "#ffc107" }}>{gift.cost} Stars</div>
            </div>
          ))}
        </div>

        {/* Upstream star_gift_box.cpp options */}
        <div style={{ display: "flex", flexDirection: "column", gap: "6px", marginBottom: "12px", fontSize: "11px" }}>
          <label style={{ display: "flex", alignItems: "center", gap: "6px", cursor: "pointer" }}>
            <input type="checkbox" checked={hideMyName} onChange={(e) => setHideMyName(e.target.checked)} />
            <span>匿名赠送 (hideMyName)</span>
          </label>
          <label style={{ display: "flex", alignItems: "center", gap: "6px", cursor: "pointer" }}>
            <input type="checkbox" checked={convertStars} onChange={(e) => setConvertStars(e.target.checked)} />
            <span>支持接收者折现为 Stars (convertStars)</span>
          </label>
        </div>

        {transferStatus && (
          <div style={{ fontSize: "11px", color: "#00c972", textAlign: "center", marginBottom: "10px" }}>
            {transferStatus}
          </div>
        )}

        <button
          type="button"
          data-testid="tdrp-send-gift-button"
          onClick={() => onSendGift({ hideMyName, convertStars })}
          style={{
            width: "100%",
            padding: "8px",
            borderRadius: "8px",
            border: 0,
            background: "#1084fe",
            color: "#fff",
            fontWeight: 700,
            fontSize: "13px",
            cursor: "pointer",
          }}
        >
          赠送 {selectedGift.name} ({selectedGift.cost} Stars)
        </button>
      </div>
    </div>
  );
};
