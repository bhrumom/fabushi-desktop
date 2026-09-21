import React, { useEffect, useRef } from 'react';
import { AppWindow, Search, Trash2, X } from 'lucide-react';
import type { InstalledPluginPointer, MarketplacePluginSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { marketplaceInstallAction, marketplaceInstallActionLabel } from '../../../../frontend/apps/web/src/lib/marketplace-install-contract';
import { deleteMiniAppCloudStorage, readMiniAppCloudStorage, writeMiniAppCloudStorage } from '../../account-sync-client';
import FabAvatar from '../../ui/avatar/fab-avatar';
import styles from '../../messaging-shell.module.css';

function miniAppMarketplaceAction(
  app: MarketplacePluginSummary,
  installed: InstalledPluginPointer | undefined,
) {
  return marketplaceInstallAction(
    {
      id: app.pluginId,
      latestVersion: app.latestVersion,
      install: app.install,
      releaseManifest: app.releaseManifest,
    },
    installed
      ? { version: installed.version, artifactSha256: installed.artifactSha256 }
      : undefined,
  );
}

function miniAppReleaseLabel(app: MarketplacePluginSummary): string {
  const source = app.install?.source as { sourceRef?: unknown } | undefined;
  const sourceRef = typeof source?.sourceRef === 'string' ? source.sourceRef.slice(0, 9) : '';
  return sourceRef ? `GitHub · ${sourceRef}` : 'GitHub 来源待确认';
}

export function miniAppCloudBridgeDocument(html: string): string {
  const bootstrap = `<script>(function(){
    const protocol='fabushi.miniapp.storage.v1';
    let sequence=0; const pending=new Map();
    function request(action,payload){return new Promise((resolve,reject)=>{const requestId='storage-'+Date.now()+'-'+(++sequence);pending.set(requestId,{resolve,reject});window.parent.postMessage({protocol,requestId,action,...(payload||{})},'*');});}
    window.addEventListener('message',(event)=>{const data=event.data||{};if(data.protocol!==protocol||!data.requestId||!pending.has(data.requestId))return;const task=pending.get(data.requestId);pending.delete(data.requestId);if(data.ok)task.resolve(data.data);else task.reject(new Error(data.error||'CloudStorage request failed'));});
    const api={
      getItem:async(key,callback)=>{const data=await request('get',{key});const value=data&&data.item?String(data.item.value??''):'';if(typeof callback==='function')callback(null,value);return value;},
      setItem:async(key,value,callback)=>{await request('set',{values:{[key]:String(value)}});if(typeof callback==='function')callback(null,true);return true;},
      getItems:async(keys,callback)=>{const data=await request('list');const wanted=new Set(Array.isArray(keys)?keys:[]);const values=Object.fromEntries((data.items||[]).filter(item=>wanted.size===0||wanted.has(item.key)).map(item=>[item.key,item.value]));if(typeof callback==='function')callback(null,values);return values;},
      setItems:async(values,callback)=>{await request('set',{values:values||{}});if(typeof callback==='function')callback(null,true);return true;},
      removeItem:async(key,callback)=>{await request('delete',{key});if(typeof callback==='function')callback(null,true);return true;},
      getKeys:async(callback)=>{const data=await request('list');const keys=(data.items||[]).map(item=>item.key);if(typeof callback==='function')callback(null,keys);return keys;}
    };
    window.FabushiMiniApp=Object.assign({},window.FabushiMiniApp||{},{CloudStorage:api});
  })();</script>`;
  return html.includes('</head>') ? html.replace('</head>', `${bootstrap}</head>`) : `${bootstrap}${html}`;
}

export function MiniAppCompatibilityDialog({ app, onClose }: { app: { id: string; title: string; url: string }; onClose: () => void }) {
  const frameRef = useRef<HTMLIFrameElement>(null);
  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (event.source !== frameRef.current?.contentWindow) return;
      const data = event.data as { protocol?: string; requestId?: string; action?: string; key?: string; values?: Record<string, string> } | null;
      if (!data || data.protocol !== 'fabushi.miniapp.storage.v1' || !data.requestId) return;
      const respond = (ok: boolean, payload: unknown) => frameRef.current?.contentWindow?.postMessage({
        protocol: 'fabushi.miniapp.storage.v1', requestId: data.requestId, ok,
        ...(ok ? { data: payload } : { error: payload instanceof Error ? payload.message : String(payload) }),
      }, '*');
      void (async () => {
        try {
          if (data.action === 'get') respond(true, await readMiniAppCloudStorage(app.id, data.key));
          else if (data.action === 'list') respond(true, await readMiniAppCloudStorage(app.id));
          else if (data.action === 'set') respond(true, await writeMiniAppCloudStorage(app.id, data.values ?? {}));
          else if (data.action === 'delete' && data.key) respond(true, await deleteMiniAppCloudStorage(app.id, data.key));
          else throw new Error('Unsupported Mini App CloudStorage operation');
        } catch (cause) { respond(false, cause); }
      })();
    };
    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  }, [app.id]);
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.miniAppDialog} onMouseDown={(event) => event.stopPropagation()}><header><div><strong>{app.title}</strong><small>Mini App · 已安装线上包 · 账号云同步</small></div><button type="button" data-testid="miniapp-close" aria-label="关闭小程序" onClick={onClose}><X size={17} /></button></header><iframe ref={frameRef} title={app.id} sandbox="allow-scripts allow-forms" src={app.url} /></section></div>;
}

export type MiniAppMarketplaceProps = {
  miniApps: MarketplacePluginSummary[];
  installedMiniApps: Record<string, InstalledPluginPointer>;
  miniAppQuery: string;
  onMiniAppQuery: (query: string) => void;
  miniAppLoading: boolean;
  miniAppBusy: Set<string>;
  onOpenMiniApp: (id: string) => Promise<void>;
  onInstallMiniApp: (app: MarketplacePluginSummary) => Promise<void>;
  onUninstallMiniApp: (id: string) => Promise<void>;
};

function MiniAppMarketplaceSearch({ query, onChange }: { query: string; onChange: (query: string) => void }) {
  return <label className={styles.marketplaceSearch}><Search size={15} /><input value={query} onChange={(event) => onChange(event.target.value)} placeholder="搜索线上 Mini App" />{query ? <button type="button" onClick={() => onChange('')}><X size={13} /></button> : null}</label>;
}

function MiniAppMarketplaceList(props: MiniAppMarketplaceProps) {
  return <div className={styles.sectionList}>
    <MiniAppMarketplaceSearch query={props.miniAppQuery} onChange={props.onMiniAppQuery} />
    {props.miniAppLoading ? <div className={styles.marketplaceStatus}>正在搜索在线市场…</div> : null}
    {!props.miniAppLoading && !props.miniApps.length ? <div className={styles.marketplaceStatus}>没有找到可安装的 Mini App</div> : null}
    {props.miniApps.map((app) => {
      const installed = props.installedMiniApps[app.pluginId];
      const busy = props.miniAppBusy.has(app.pluginId);
      const action = miniAppMarketplaceAction(app, installed);
      const needsInstall = action === 'install' || action === 'update' || action === 'reinstall';
      return <div className={styles.marketplaceRow} key={app.pluginId} data-testid={`miniapp-market-${app.pluginId}`}>
        <span className={styles.appIcon}><FabAvatar identity={`miniapp:${app.pluginId}`} state={installed ? "idle" : "sleeping"} size={34} label={app.displayName} /></span>
        <div className={styles.marketplaceCopy}><strong>{app.displayName}</strong><small>{app.description}</small><em>{installed ? `已安装 ${installed.version}` : `在线 · ${app.latestVersion}`} · {miniAppReleaseLabel(app)}</em></div>
        <div className={styles.marketplaceActions}>
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}
          {needsInstall ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中' : marketplaceInstallActionLabel(action)}</button> : action === 'blocked' ? <button type="button" disabled>阻止降级</button> : null}
          {installed ? <button type="button" disabled={busy} title="卸载" onClick={() => void props.onUninstallMiniApp(app.pluginId)}><Trash2 size={13} /></button> : null}
        </div>
      </div>;
    })}
  </div>;
}

export function MiniAppMarketplaceCompatibilityAdapter(props: MiniAppMarketplaceProps) {
  return <div className={styles.featureWorkspace}>
    <AppWindow size={54} />
    <h2>Mini Apps</h2>
    <p>所有官方与第三方 Mini App 都从线上市场搜索、验证并安装；Fabushi 主程序不预装应用。</p>
    <MiniAppMarketplaceSearch query={props.miniAppQuery} onChange={props.onMiniAppQuery} />
    {props.miniAppLoading ? <div className={styles.marketplaceStatus}>正在搜索在线市场…</div> : null}
    <div className={styles.featureGrid}>{props.miniApps.map((app) => {
      const installed = props.installedMiniApps[app.pluginId];
      const busy = props.miniAppBusy.has(app.pluginId);
      const action = miniAppMarketplaceAction(app, installed);
      const needsInstall = action === 'install' || action === 'update' || action === 'reinstall';
      return <article className={styles.marketplaceCard} key={app.pluginId}>
        <FabAvatar identity={`miniapp:${app.pluginId}`} state={installed ? "idle" : "sleeping"} size={48} label={app.displayName} />
        <strong>{app.displayName}</strong>
        <small>{app.description}</small>
        <em>{installed ? `已安装 ${installed.version}` : `线上版本 ${app.latestVersion}`} · {miniAppReleaseLabel(app)}</em>
        <div>
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</button> : null}
          {needsInstall ? <button type="button" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中…' : marketplaceInstallActionLabel(action)}</button> : action === 'blocked' ? <button type="button" disabled>阻止降级</button> : null}
          {installed ? <button type="button" disabled={busy} onClick={() => void props.onUninstallMiniApp(app.pluginId)}>卸载</button> : null}
        </div>
      </article>;
    })}</div>
    {!props.miniAppLoading && !props.miniApps.length ? <div className={styles.marketplaceStatus}>没有找到可安装的 Mini App</div> : null}
  </div>;
}


export type GeneratedMiniAppPreview = {
  id: string;
  title: string;
  html: string;
  complete: boolean;
};

export function generatedMiniAppPreview(text: string, stableId: string): GeneratedMiniAppPreview | null {
  const fence = text.match(/```(?:html)?\s*\n([\s\S]*)/i);
  const raw = fence?.[1] ?? text;
  const start = raw.search(/<!doctype\s+html|<html\b/i);
  if (start < 0) return null;
  const candidate = raw.slice(start);
  if (!/<body\b/i.test(candidate) || candidate.length < 160) return null;
  const close = candidate.search(/<\/html>/i);
  const html = close >= 0 ? candidate.slice(0, close + candidate.slice(close).match(/^<\/html>/i)![0].length) : candidate;
  const titleMatch = html.match(/<title[^>]*>([\s\S]*?)<\/title>/i);
  const title = (titleMatch?.[1] ?? 'AI 生成小程序').replace(/<[^>]+>/g, '').trim().slice(0, 80) || 'AI 生成小程序';
  const suffix = String(stableId || Date.now()).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 44) || 'preview';
  return { id: `generated-${suffix}`.slice(0, 64), title, html, complete: close >= 0 };
}

