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
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        padding: "4px 8px",
        borderTop: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.08))",
        fontSize: 12,
      }}
    >
      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("poll")}
        sentiment="neutral"
        size="sm"
        style={{ display: "inline-flex", alignItems: "center", gap: 5, padding: "2px 8px", height: 26 }}
        type="button"
      >
        <SandIcon name="chart-bars" size="sm" />
        <span>Poll</span>
      </SandButton>

      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("todo")}
        sentiment="neutral"
        size="sm"
        style={{ display: "inline-flex", alignItems: "center", gap: 5, padding: "2px 8px", height: 26 }}
        type="button"
      >
        <SandIcon name="check-square" size="sm" />
        <span>Todo</span>
      </SandButton>

      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("invoice")}
        sentiment="neutral"
        size="sm"
        style={{ display: "inline-flex", alignItems: "center", gap: 5, padding: "2px 8px", height: 26 }}
        type="button"
      >
        <SandIcon name="credit-card" size="sm" />
        <span>Invoice</span>
      </SandButton>

      <SandButton
        disabled={disabled}
        onClick={() => onOpenModal("bot")}
        sentiment="neutral"
        size="sm"
        style={{ display: "inline-flex", alignItems: "center", gap: 5, padding: "2px 8px", height: 26 }}
        type="button"
      >
        <SandIcon name="sparkle" size="sm" />
        <span>Inline Bot</span>
      </SandButton>
    </div>
  );
}
