import React, { useState } from "react";
import { Download, X } from "lucide-react";

export interface TdrpExportModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly exportFormat: "html" | "json" | "text";
  readonly onExportFormatChange: (fmt: "html" | "json" | "text") => void;
  readonly includeMedia: boolean;
  readonly onIncludeMediaChange: (include: boolean) => void;
  readonly exportStatus: string | null;
  readonly onExportConfirm: (settings?: { mediaTypes: number; sizeLimitMb: number; fromDate?: string; tillDate?: string }) => void;
}

// Upstream export_settings.h MediaSettings::Type
export const EXPORT_MEDIA_TYPES = {
  Photo: 0x01,
  Video: 0x02,
  VoiceMessage: 0x04,
  VideoMessage: 0x08,
  Sticker: 0x10,
  GIF: 0x20,
  File: 0x40,
} as const;

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
  const [mediaTypes, setMediaTypes] = useState<number>(
    EXPORT_MEDIA_TYPES.Photo | EXPORT_MEDIA_TYPES.VoiceMessage | EXPORT_MEDIA_TYPES.File
  );
  const [sizeLimitMb, setSizeLimitMb] = useState<number>(50);
  const [fromDate, setFromDate] = useState("");
  const [tillDate, setTillDate] = useState("");

  if (!isOpen) return null;

  const toggleMediaType = (val: number) => {
    setMediaTypes((prev) => (prev & val ? prev & ~val : prev | val));
  };

  const mediaOptions = [
    { bit: EXPORT_MEDIA_TYPES.Photo, label: "照片 (Photos)" },
    { bit: EXPORT_MEDIA_TYPES.Video, label: "视频 (Videos)" },
    { bit: EXPORT_MEDIA_TYPES.VoiceMessage, label: "语音 (Voice)" },
    { bit: EXPORT_MEDIA_TYPES.VideoMessage, label: "视频留言 (Video Msgs)" },
    { bit: EXPORT_MEDIA_TYPES.Sticker, label: "贴纸 (Stickers)" },
    { bit: EXPORT_MEDIA_TYPES.GIF, label: "动图 (GIFs)" },
    { bit: EXPORT_MEDIA_TYPES.File, label: "文件与附件 (Files)" },
  ];

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
          background: "var(--sand-surface-backdrop, #242429)",
          padding: "20px 24px",
          borderRadius: "16px",
          width: "360px",
          maxHeight: "88vh",
          overflowY: "auto",
          color: "var(--sand-text-primary, #ffffff)",
          boxShadow: "0 12px 36px rgba(0,0,0,0.5)",
          border: "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "14px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Download size={18} color="#1084fe" />
            <h3 style={{ margin: 0, fontSize: "15px", fontWeight: 700 }}>导出聊天记录 (Export Settings)</h3>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="关闭"
            style={{ background: "none", border: 0, color: "var(--sand-text-secondary, #888)", cursor: "pointer" }}
          >
            <X size={16} />
          </button>
        </div>

        <div style={{ fontSize: "12px", marginBottom: "6px", fontWeight: 600 }}>导出格式 (Format)</div>
        <div style={{ display: "flex", gap: "6px", marginBottom: "12px" }}>
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
                border: exportFormat === fmt ? "1px solid #1084fe" : "1px solid var(--sand-border-weak, #444)",
                background: exportFormat === fmt ? "rgba(16, 132, 254, 0.15)" : "transparent",
                color: exportFormat === fmt ? "#1084fe" : "inherit",
                fontSize: "12px",
                fontWeight: exportFormat === fmt ? 700 : 400,
                cursor: "pointer",
              }}
            >
              {fmt.toUpperCase()}
            </button>
          ))}
        </div>

        <label style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "12px", marginBottom: "10px", cursor: "pointer" }}>
          <input type="checkbox" checked={includeMedia} onChange={(e) => onIncludeMediaChange(e.target.checked)} />
          <span>包含媒体与附件 (MediaSettings)</span>
        </label>

        {includeMedia && (
          <div style={{ background: "var(--sand-surface-muted, rgba(128,128,128,0.08))", padding: "10px", borderRadius: "8px", marginBottom: "12px" }}>
            <div style={{ fontSize: "11px", fontWeight: 600, marginBottom: "6px" }}>媒体类型过滤</div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "5px", marginBottom: "10px" }}>
              {mediaOptions.map((opt) => (
                <label key={opt.bit} style={{ display: "flex", alignItems: "center", gap: "6px", fontSize: "11px", cursor: "pointer" }}>
                  <input type="checkbox" checked={Boolean(mediaTypes & opt.bit)} onChange={() => toggleMediaType(opt.bit)} />
                  <span>{opt.label}</span>
                </label>
              ))}
            </div>

            <div style={{ display: "flex", justifyContent: "space-between", fontSize: "11px", marginBottom: "4px" }}>
              <span>单文件大小上限 (sizeLimit)</span>
              <span style={{ color: "#1084fe", fontWeight: 600 }}>{sizeLimitMb} MB</span>
            </div>
            <input
              type="range"
              min={1}
              max={2000}
              value={sizeLimitMb}
              onChange={(e) => setSizeLimitMb(Number(e.target.value))}
              style={{ width: "100%", cursor: "pointer" }}
            />
          </div>
        )}

        <div style={{ fontSize: "12px", marginBottom: "4px", fontWeight: 600 }}>导出时间范围 (Date Range)</div>
        <div style={{ display: "flex", gap: "6px", marginBottom: "16px" }}>
          <input
            type="date"
            value={fromDate}
            onChange={(e) => setFromDate(e.target.value)}
            style={{
              flex: 1,
              padding: "5px",
              borderRadius: "6px",
              border: "1px solid var(--sand-border-weak, #444)",
              background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
              color: "inherit",
              fontSize: "11px",
            }}
          />
          <span style={{ alignSelf: "center", fontSize: "12px" }}>至</span>
          <input
            type="date"
            value={tillDate}
            onChange={(e) => setTillDate(e.target.value)}
            style={{
              flex: 1,
              padding: "5px",
              borderRadius: "6px",
              border: "1px solid var(--sand-border-weak, #444)",
              background: "var(--sand-surface-muted, rgba(128,128,128,0.1))",
              color: "inherit",
              fontSize: "11px",
            }}
          />
        </div>

        {exportStatus && <div style={{ fontSize: "11px", color: "#69f0ae", marginBottom: "12px" }}>{exportStatus}</div>}
        <button
          type="button"
          data-testid="tdrp-export-confirm-button"
          onClick={() => onExportConfirm({ mediaTypes, sizeLimitMb, fromDate, tillDate })}
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
          开始导出
        </button>
      </div>
    </div>
  );
};
