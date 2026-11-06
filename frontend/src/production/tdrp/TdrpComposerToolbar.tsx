import React from "react";
import { SandIcon, SandButton } from "../../recovered/ui/sand-kit-primitives";

export interface TdrpComposerToolbarProps {
  readonly disabled?: boolean;
  readonly onOpenModal: (mode: "poll" | "todo" | "invoice" | "bot") => void;
}

export function TdrpComposerToolbar({
  disabled = false,
  onOpenModal,
}: TdrpComposerToolbarProps): React.JSX.Element {
  const pillStyle: React.CSSProperties = {
    display: "inline-flex",
    alignItems: "center",
    gap: 5,
    padding: "0 10px",
    height: 26,
    borderRadius: 13,
    fontSize: 12,
    fontWeight: 500,
    background: "transparent",
    color: "inherit",
    border: "1px solid rgba(128, 128, 128, 0.22)",
    boxShadow: "none",
    cursor: disabled ? "not-allowed" : "pointer",
    opacity: disabled ? 0.5 : 1,
    transition: "background 0.15s ease, border-color 0.15s ease",
  };

  return (
    <div
      className="tdrp-composer-toolbar"
      data-testid="tdrp-composer-toolbar"
      style={{
        display: "flex",
        alignItems: "center",
        flexWrap: "wrap",
        gap: 6,
        padding: "6px 8px 4px 8px",
        borderTop: "1px solid rgba(128, 128, 128, 0.12)",
        background: "transparent",
        color: "inherit",
      }}
    >
      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("poll")}
        sentiment="neutral"
        size="sm"
        style={pillStyle}
        type="button"
      >
        <SandIcon name="chart-bars" size="sm" />
        <span>投票</span>
      </SandButton>

      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("todo")}
        sentiment="neutral"
        size="sm"
        style={pillStyle}
        type="button"
      >
        <SandIcon name="check-square" size="sm" />
        <span>待办清单</span>
      </SandButton>

      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("invoice")}
        sentiment="neutral"
        size="sm"
        style={pillStyle}
        type="button"
      >
        <SandIcon name="credit-card" size="sm" />
        <span>收款账单</span>
      </SandButton>

      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("bot")}
        sentiment="neutral"
        size="sm"
        style={pillStyle}
        type="button"
      >
        <SandIcon name="sparkle" size="sm" />
        <span>快捷指令</span>
      </SandButton>
    </div>
  );
}
