import React from "react";
import { Download, X } from "lucide-react";

export interface TdrpExportModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly exportFormat: "html" | "json" | "text";
  readonly onExportFormatChange: (fmt: "html" | "json" | "text") => void;
  readonly includeMedia: boolean;
  readonly onIncludeMediaChange: (include: boolean) => void;
  readonly exportStatus: string | null;
  readonly onExportConfirm: () => void;
}

export const TdrpExportModal: React.FC<TdrpExportModalProps> = ({
  isOpen,
  onClose,
  exportFormat,
  onExportFormatChange,
  includeMedia,
  onIncludeMediaChange,
  exportStatus,
  onExportConfirm,
}) => {
  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      data-testid="tdrp-export-modal"
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
          borderRadius: "14px",
          width: "340px",
          color: "var(--fabushi-text-primary, #ffffff)",
          boxShadow: "0 10px 30px rgba(0,0,0,0.5)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "14px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Download size={18} color="var(--fabushi-accent, #1084fe)" />
            <h3 style={{ margin: 0, fontSize: "15px", fontWeight: 600 }}>导出聊天记录</h3>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            style={{ background: "none", border: 0, color: "var(--fabushi-text-secondary, #888)", cursor: "pointer" }}
          >
            <X size={16} />
          </button>
        </div>
        <div style={{ fontSize: "12px", marginBottom: "8px" }}>导出格式</div>
        <div style={{ display: "flex", gap: "8px", marginBottom: "14px" }}>
          {(["html", "json", "text"] as const).map((fmt) => (
            <button
              key={fmt}
              type="button"
              data-testid={`tdrp-export-fmt-${fmt}`}
              onClick={() => onExportFormatChange(fmt)}
              style={{
                flex: 1,
                padding: "6px",
                borderRadius: "6px",
                border: exportFormat === fmt ? "1px solid var(--sand-fill-accent, #3390ec)" : "1px solid #444",
                background: exportFormat === fmt ? "rgba(51, 144, 236, 0.2)" : "transparent",
                color: exportFormat === fmt ? "#3390ec" : "#aaa",
                fontSize: "12px",
                fontWeight: exportFormat === fmt ? 600 : 400,
                cursor: "pointer",
              }}
            >
              {fmt.toUpperCase()}
            </button>
          ))}
        </div>
        <label style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "12px", marginBottom: "16px", cursor: "pointer" }}>
          <input type="checkbox" checked={includeMedia} onChange={(e) => onIncludeMediaChange(e.target.checked)} />
          包含图片与附件媒体
        </label>
        {exportStatus && <div style={{ fontSize: "11px", color: "#69f0ae", marginBottom: "12px" }}>{exportStatus}</div>}
        <button
          type="button"
          data-testid="tdrp-export-confirm-button"
          onClick={onExportConfirm}
          style={{
            width: "100%",
            padding: "8px",
            borderRadius: "6px",
            border: 0,
            background: "var(--sand-fill-accent, #3390ec)",
            color: "#fff",
            fontWeight: 600,
            fontSize: "13px",
            cursor: "pointer",
          }}
        >
          开始导出
        </button>
      </div>
    </div>
  );
};
