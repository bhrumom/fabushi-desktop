import React, { useEffect, useRef } from 'react';
import { AppWindow, Search, Trash2, X } from 'lucide-react';
import type { InstalledPluginPointer, MarketplacePluginSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { marketplaceInstallAction, marketplaceInstallActionLabel } from '../../../../frontend/apps/web/src/lib/marketplace-install-contract';
import { deleteMiniAppCloudStorage, readMiniAppCloudStorage, writeMiniAppCloudStorage } from '../../account-sync-client';
import { FabAvatar, FabButton, FabIconButton, FabInput, FabSpinner, FabSurface } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';

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

function releaseLabel(app: MarketplacePluginSummary): string {
  const source = app.install?.source as { sourceRef?: unknown } | undefined;
  const sourceRef = typeof source?.sourceRef === 'string' ? source.sourceRef.slice(0, 9) : '';
  return sourceRef ? `GitHub · ${sourceRef}` : 'GitHub 来源待确认';
}

function MarketplaceSearch({ query, onChange }: { query: string; onChange(query: string): void }) {
  return <label className={styles.marketplaceSearch}>
    <Search size={15} />
    <FabInput value={query} onChange={(event) => onChange(event.target.value)} placeholder="搜索线上 Mini App" />
    {query ? <FabIconButton label="清空搜索" onClick={() => onChange('')}><X size={13} /></FabIconButton> : null}
  </label>;
}

export function MiniAppsCompatibilityWorkspace(props: MiniAppMarketplaceProps) {
  return <FabSurface className={styles.featureWorkspace} elevated data-compatibility-feature="miniapps">
    <AppWindow size={54} />
    <h2>Mini Apps</h2>
    <p>官方与第三方 Mini App 从线上市场验证、安装；Agent 主壳不持有 Mini App 生命周期。</p>
    <MarketplaceSearch query={props.miniAppQuery} onChange={props.onMiniAppQuery} />
    {props.miniAppLoading ? <div className={styles.marketplaceStatus}><FabSpinner label="正在搜索在线市场" />正在搜索在线市场…</div> : null}
    {!props.miniAppLoading && !props.miniApps.length ? <div className={styles.marketplaceStatus}>没有找到可安装的 Mini App</div> : null}
    <div className={styles.featureGrid}>{props.miniApps.map((app) => {
      const installed = props.installedMiniApps[app.pluginId];
      const busy = props.miniAppBusy.has(app.pluginId);
      const action = marketplaceInstallAction({
        id: app.pluginId,
        latestVersion: app.latestVersion,
        install: app.install,
        releaseManifest: app.releaseManifest,
      }, installed ? { version: installed.version, artifactSha256: installed.artifactSha256 } : undefined);
      const needsInstall = action === 'install' || action === 'update' || action === 'reinstall';
      return <FabSurface elevated className={styles.marketplaceCard} key={app.pluginId}>
        <FabAvatar identity={`miniapp:${app.pluginId}`} state={installed ? 'idle' : 'sleeping'} size={48} label={app.displayName} />
        <strong>{app.displayName}</strong><small>{app.description}</small>
        <em>{installed ? `已安装 ${installed.version}` : `线上版本 ${app.latestVersion}`} · {releaseLabel(app)}</em>
        <div>
          {installed ? <FabButton disabled={busy} onClick={() => void props.onOpenMiniApp(app.pluginId)}>打开</FabButton> : null}
          {needsInstall ? <FabButton variant="primary" disabled={busy} onClick={() => void props.onInstallMiniApp(app)}>{busy ? '处理中…' : marketplaceInstallActionLabel(action)}</FabButton> : action === 'blocked' ? <FabButton disabled>阻止降级</FabButton> : null}
          {installed ? <FabIconButton label="卸载" disabled={busy} onClick={() => void props.onUninstallMiniApp(app.pluginId)}><Trash2 size={13} /></FabIconButton> : null}
        </div>
      </FabSurface>;
    })}</div>
  </FabSurface>;
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

export function MiniAppCompatibilityDialog({ app, onClose }: { app: { id: string; title: string; url: string }; onClose(): void }) {
  const frameRef = useRef<HTMLIFrameElement>(null);
  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (event.source !== frameRef.current?.contentWindow) return;
      const data = event.data as { protocol?: string; requestId?: string; action?: string; key?: string; values?: Record<string, string> } | null;
      if (!data || data.protocol !== 'fabushi.miniapp.storage.v1' || !data.requestId) return;
      const respond = (ok: boolean, payload: unknown) => frameRef.current?.contentWindow?.postMessage({
        protocol: 'fabushi.miniapp.storage.v1',
        requestId: data.requestId,
        ok,
        ...(ok ? { data: payload } : { error: payload instanceof Error ? payload.message : String(payload) }),
      }, '*');
      void (async () => {
        try {
          if (data.action === 'get') respond(true, await readMiniAppCloudStorage(app.id, data.key));
          else if (data.action === 'list') respond(true, await readMiniAppCloudStorage(app.id));
          else if (data.action === 'set') respond(true, await writeMiniAppCloudStorage(app.id, data.values ?? {}));
          else if (data.action === 'delete' && data.key) respond(true, await deleteMiniAppCloudStorage(app.id, data.key));
          else throw new Error('Unsupported Mini App CloudStorage operation');
        } catch (cause) {
          respond(false, cause);
        }
      })();
    };
    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  }, [app.id]);
  return <div className={styles.backdrop} data-compatibility-adapter="miniapps" onMouseDown={onClose}>
    <FabSurface className={styles.miniAppDialog} elevated onMouseDown={(event) => event.stopPropagation()}>
      <header><div><strong>{app.title}</strong><small>Mini App · 已安装线上包 · 账号云同步</small></div><FabIconButton label="关闭小程序" data-testid="miniapp-close" onClick={onClose}><X size={17} /></FabIconButton></header>
      <iframe ref={frameRef} title={app.id} sandbox="allow-scripts allow-forms" src={app.url} />
    </FabSurface>
  </div>;
}
