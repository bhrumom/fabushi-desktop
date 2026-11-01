import { useEffect, useRef, useState } from "react";
import type { DesktopBridge, DesktopMiniAppEntry } from "../../../contracts/desktop-bridge";
import { SandButton } from "../../../ui/sand-kit-primitives";

export function MiniAppManagement({ bridge }: { bridge: DesktopBridge }) {
  const api = bridge.marketplaceMiniApps;
  const [entries, setEntries] = useState<DesktopMiniAppEntry[]>([]);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const generation = useRef(0);
  useEffect(() => { generation.current += 1; return () => { generation.current += 1; }; }, []);
  const run = async (action: () => Promise<void>) => {
    if (busy || !api) return;
    const ticket = generation.current;
    setBusy(true);
    try {
      await action();
      const snapshot = await api.list();
      if (generation.current === ticket) { setEntries(snapshot.entries); setStatus("状态已更新"); }
    } catch (error) {
      if (generation.current === ticket) setStatus(error instanceof Error ? error.message : "小程序操作失败");
    } finally { if (generation.current === ticket) setBusy(false); }
  };
  return <section aria-label="离线小程序" className="sand-miniapp-management">
    <h3>离线小程序</h3>
    <p>完整验证并安装后可离线启动。业务功能由小程序安装包提供；原生能力仍需单独授权。</p>
    {!api ? <p role="status">当前客户端尚不支持离线小程序安装。</p> : <>
      <SandButton disabled={busy} onClick={() => void run(async () => {})}>刷新市场与安装状态</SandButton>
      <ul>{entries.map((entry) => <li key={entry.appId}>
        <h4>{entry.displayName}</h4>
        <p>{entry.publisherId} · {entry.version} · {Math.ceil(entry.sizeBytes / 1024)} KiB · {entry.offlineCore ? "离线包" : "非离线包"}</p>
        <p>{!entry.supported ? "宿主尚不支持" : entry.installed ? "已安装" : "未安装"}</p>
        {entry.availableUpdate && <p>可更新至 {entry.availableUpdate.version}</p>}
        <details><summary>包摘要</summary><code>{entry.packageDigest}</code></details>
        <SandButton disabled={busy || !entry.supported} onClick={() => void run(async () => {
          if (entry.installed) await api.launch(entry.appId); else await api.install(entry.appId);
        })}>{entry.installed ? "打开" : "安装并验证"}</SandButton>
        {entry.availableUpdate && <SandButton disabled={busy} onClick={() => void run(async () => {
          await api.install(entry.appId);
        })}>更新并验证</SandButton>}
        {entry.installed && <SandButton disabled={busy} onClick={() => void run(async () => {
          await api.removeCode(entry.appId);
        })}>移除代码，保留数据</SandButton>}
      </li>)}</ul>
      {entries.length === 0 && !busy && <p>请刷新查看已批准的小程序；未上架的包不会显示为可安装。</p>}
    </>}
    <p role="status" aria-live="polite">{busy ? "正在处理…" : status}</p>
  </section>;
}
