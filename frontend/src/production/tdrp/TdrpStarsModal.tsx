import React from "react";
import { Sparkles, Gift, X } from "lucide-react";

export interface GiftItem {
  readonly name: string;
  readonly cost: number;
}

export interface TdrpStarsModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly starsBalance: number;
  readonly selectedGift: GiftItem;
  readonly onSelectGift: (gift: GiftItem) => void;
  readonly onRechargeStars: (amount: number) => void;
  readonly onSendGift: () => void;
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
  { name: "奖杯", cost: 500 },
  { name: "火箭", cost: 1000 },
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
          background: "var(--cursor-bg-editor, #242429)",
          padding: "20px 24px",
          borderRadius: "16px",
          width: "360px",
          color: "#fff",
          boxShadow: "0 10px 30px rgba(0,0,0,0.5)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "16px" }}>
          <h3 style={{ margin: 0, fontSize: "16px", fontWeight: 600, display: "flex", alignItems: "center", gap: "6px" }}>
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
            background: "rgba(255, 193, 7, 0.1)",
            border: "1px solid rgba(255, 193, 7, 0.3)",
            borderRadius: "10px",
            padding: "12px",
            textAlign: "center",
            marginBottom: "16px",
          }}
        >
          <div style={{ fontSize: "11px", color: "rgba(255,255,255,0.7)" }}>当前余额</div>
          <div style={{ fontSize: "22px", fontWeight: 700, color: "#ffc107", marginTop: "2px" }}>
            {starsBalance.toLocaleString()} Stars
          </div>
        </div>

        <div style={{ fontSize: "12px", fontWeight: 600, marginBottom: "8px" }}>充值 Stars</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px", marginBottom: "16px" }}>
          {RECHARGE_TIERS.map((tier) => (
            <button
              key={tier.amount}
              type="button"
              onClick={() => onRechargeStars(tier.amount)}
              style={{
                padding: "8px",
                borderRadius: "8px",
                border: "1px solid var(--sand-border-weak, #444)",
                background: "var(--cursor-bg-chrome, #1c1c21)",
                color: "#fff",
                cursor: "pointer",
                fontSize: "12px",
                display: "flex",
                justifyContent: "space-between",
              }}
            >
              <span style={{ color: "#ffc107", fontWeight: 600 }}>{tier.amount} Stars</span>
              <span style={{ color: "#aaa" }}>{tier.price}</span>
            </button>
          ))}
        </div>

        <div style={{ fontSize: "12px", fontWeight: 600, marginBottom: "8px" }}>赠礼中心</div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "6px", marginBottom: "16px" }}>
          {GIFT_OPTIONS.map((gift) => (
            <div
              key={gift.name}
              onClick={() => onSelectGift(gift)}
              style={{
                padding: "8px 4px",
                borderRadius: "8px",
                border: selectedGift.name === gift.name ? "1px solid #ffc107" : "1px solid #444",
                background: selectedGift.name === gift.name ? "rgba(255, 193, 7, 0.15)" : "transparent",
                textAlign: "center",
                cursor: "pointer",
              }}
            >
              <Gift size={18} style={{ margin: "0 auto", color: selectedGift.name === gift.name ? "#ffc107" : "#aaa" }} />
              <div style={{ fontSize: "11px", marginTop: "4px" }}>{gift.name}</div>
              <div style={{ fontSize: "9px", color: "#ffc107" }}>{gift.cost} Stars</div>
            </div>
          ))}
        </div>

        {transferStatus && (
          <div style={{ fontSize: "11px", color: "#00c972", textAlign: "center", marginBottom: "10px" }}>
            {transferStatus}
          </div>
        )}

        <button
          type="button"
          data-testid="tdrp-send-gift-button"
          onClick={onSendGift}
          style={{
            width: "100%",
            padding: "8px",
            borderRadius: "8px",
            border: 0,
            background: "var(--sand-fill-accent, #1084fe)",
            color: "#fff",
            fontWeight: 600,
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
