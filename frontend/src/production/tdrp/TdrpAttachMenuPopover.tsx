/**
 * Composer "+" attach menu popover for files and non-destructive typed actions
 * (Poll, Todo List, Invoice, Bot Inline, and Star Gift).
 *
 * Upstream & Open PR Alignment:
 * - Telegram Desktop `Telegram/SourceFiles/api/api_sending.cpp` (`SendSimpleMedia`:
 *   `action.clearDraft = false` ensuring typed actions open modals or dispatch
 *   directly without overwriting the active composer text draft).
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`) &
 *   Blueprint 13 (`docs/specs/fusion-blueprints/13-checkout-stars-ledger-and-economy.md`).
 */

import React, { useEffect, useRef } from "react";
import { Image, BarChart2, CheckSquare, CreditCard, Terminal, Gift } from "lucide-react";
import type { DirectTypedActionPayload } from "../../recovered/features/conversation/workspace/submission";

export interface TdrpAttachMenuPopoverProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSelectPhotoOrFile: () => void;
  readonly onOpenModal: (mode: "poll" | "todo" | "invoice" | "bot" | "gift") => void;
  readonly onSelectGift?: () => void;
  readonly onSubmitDirectMessage?: (payload: string) => Promise<void> | void;
  readonly onDirectSendTypedAction?: (payload: DirectTypedActionPayload) => Promise<void> | void;
  readonly anchorPosition?: { top: number; left: number };
}

export const TdrpAttachMenuPopover: React.FC<TdrpAttachMenuPopoverProps> = ({
  isOpen,
  onClose,
  onSelectPhotoOrFile,
  onOpenModal,
  onSelectGift,
  anchorPosition,
}) => {
  const popoverRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    const handleOutsideClick = (e: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    document.addEventListener("mousedown", handleOutsideClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleOutsideClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const itemStyle: React.CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: "10px",
    width: "100%",
    padding: "8px 12px",
    borderRadius: "8px",
    border: 0,
    background: "transparent",
    color: "var(--text-strong, #211e1a)",
    fontSize: "13px",
    fontWeight: 500,
    textAlign: "left",
    cursor: "pointer",
    transition: "background 0.12s ease",
    userSelect: "none",
  };

  const popoverStyle: React.CSSProperties = {
    position: "absolute",
    bottom: anchorPosition ? undefined : "100%",
    left: anchorPosition ? undefined : 0,
    top: anchorPosition ? `${anchorPosition.top}px` : undefined,
    marginBottom: "8px",
    minWidth: "170px",
    background: "var(--bg-elevated, #ffffff)",
    border: "1px solid var(--border, #e8e4dc)",
    borderRadius: "12px",
    boxShadow: "0 12px 28px -4px rgba(0, 0, 0, 0.14), 0 6px 12px -2px rgba(0, 0, 0, 0.08)",
    padding: "5px",
    zIndex: 1000,
  };

  return (
    <div
      ref={popoverRef}
      className="tdrp-attach-menu-popover"
      data-testid="tdrp-attach-menu-popover"
      style={popoverStyle}
    >
      <button
        type="button"
        style={itemStyle}
        data-testid="tdrp-attach-file"
        onClick={() => {
          onClose();
          onSelectPhotoOrFile();
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "var(--bg-hover, #f4f1ec)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <Image size={16} strokeWidth={2} style={{ color: "#1084fe" }} />
        <span>照片与文件</span>
      </button>

      <button
        type="button"
        style={itemStyle}
        data-testid="tdrp-attach-poll"
        onClick={() => {
          onClose();
          onOpenModal("poll");
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "var(--bg-hover, #f4f1ec)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <BarChart2 size={16} strokeWidth={2} style={{ color: "#d9534f" }} />
        <span>发起投票</span>
      </button>

      <button
        type="button"
        style={itemStyle}
        data-testid="tdrp-attach-todo"
        onClick={() => {
          onClose();
          onOpenModal("todo");
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "var(--bg-hover, #f4f1ec)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <CheckSquare size={16} strokeWidth={2} style={{ color: "#2e7d32" }} />
        <span>待办清单</span>
      </button>

      <button
        type="button"
        style={itemStyle}
        data-testid="tdrp-attach-invoice"
        onClick={() => {
          onClose();
          onOpenModal("invoice");
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "var(--bg-hover, #f4f1ec)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <CreditCard size={16} strokeWidth={2} style={{ color: "#e65100" }} />
        <span>收款账单</span>
      </button>

      <button
        type="button"
        style={itemStyle}
        data-testid="tdrp-attach-bot"
        onClick={() => {
          onClose();
          onOpenModal("bot");
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "var(--bg-hover, #f4f1ec)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        <Terminal size={16} strokeWidth={2} style={{ color: "#6a1b9a" }} />
        <span>快捷指令</span>
      </button>

      {onSelectGift != null && (
        <button
          type="button"
          style={itemStyle}
          data-testid="tdrp-attach-gift"
          onClick={() => {
            onClose();
            onSelectGift();
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = "var(--bg-hover, #f4f1ec)";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = "transparent";
          }}
        >
          <Gift size={16} strokeWidth={2} style={{ color: "#d97706" }} />
          <span>赠送礼物</span>
        </button>
      )}
    </div>
  );
};
