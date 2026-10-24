import { useEffect, useRef, useState } from "react";
import type { DesktopBridge } from "../../../contracts/desktop-bridge";

export function MiniAppPublisher({ bridge }: { bridge: DesktopBridge }) {
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const alive = useRef(true);
  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
  const run = async (action: () => Promise<string>) => {
    if (busy) return;
    setBusy(true);
    try { const result = await action(); if (alive.current) setStatus(result); }
    catch (error) { if (alive.current) setStatus(error instanceof Error ? error.message : "发布操作失败"); }
    finally { if (alive.current) setBusy(false); }
  };
  const publisher = bridge.marketplacePublisher;
  return <details className="sand-miniapp-publisher">
    <summary>发布离线小程序</summary>
    <p>Global Dandelion 0.1.0 专有预览版：离线分包、校验与报告导出。尚不包含 LAN/WAN、热点和系统扫描；提交审核不等于上架或安装完成。</p>
    {publisher == null ? <p role="status">当前客户端尚不支持小程序发布，请更新客户端。</p> : <div>
      <button disabled={busy} type="button" onClick={() => void run(async () => {
        const result = await publisher.ownedNamespaces(); return `当前账号拥有：${result.namespaces.join("、") || "尚无发布命名空间"}`;
      })}>查看发布者登记</button>
      <button disabled={busy} type="button" onClick={() => void run(async () => {
        await publisher.claimDandelionNamespace(); return "bhrumom 命名空间已登记到当前真实账号";
      })}>登记 bhrumom 发布者</button>
      <button disabled={busy} type="button" onClick={() => void run(async () => {
        const result = await publisher.submitDandelion();
        return result.state === "cancelled" ? "已取消，未提交" : `市场状态：${result.state}；包 SHA256：${result.digest ?? ""}`;
      })}>选择签名包并提交市场审核</button>
    </div>}
    <p aria-live="polite" role="status">{busy ? "正在处理，请勿切换账号…" : status}</p>
  </details>;
}
