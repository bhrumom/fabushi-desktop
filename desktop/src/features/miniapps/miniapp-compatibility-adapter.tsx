import { AppWindow, X } from 'lucide-react';
import React, { useEffect, useMemo, useRef, useState } from 'react';
import { invokeNativeDesktop } from '../../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import { marketplaceInstallAction, marketplaceInstallActionLabel } from '../../../../frontend/apps/web/src/lib/marketplace-install-contract';
import type {
  InstalledPluginPointer,
  MahayanaHostTransport,
  MarketplacePluginSummary,
} from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import {
  deleteMiniAppCloudStorage,
  readMiniAppCloudStorage,
  reconcileAccountMiniApps,
  writeMiniAppCloudStorage,
} from '../../account-sync-client';
import { prepareDesktopMiniAppWebMcpDocument } from '../../miniapp-webmcp-host';
import FabAvatar from '../../ui/avatar/fab-avatar';
import {
  FabButton,
  FabDialog,
  FabIconButton,
  FabInput,
  FabSpinner,
  FabSurface,
} from '../../ui/primitives/fab-primitives';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import styles from '../compatibility/compatibility-data.module.css';

function cloudBridgeDocument(html: string): string {
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

function MiniAppDialog({
  app,
  onClose,
}: {
  readonly app: { readonly id: string; readonly title: string; readonly url: string };
  readonly onClose: () => void;
}) {
  const frameRef = useRef<HTMLIFrameElement>(null);
  useEffect(() => {
    const onMessage = (event: MessageEvent) => {
      if (event.source !== frameRef.current?.contentWindow) return;
      const data = event.data as {
        protocol?: string;
        requestId?: string;
        action?: string;
        key?: string;
        values?: Record<string, string>;
      } | null;
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

  return <FabDialog label={app.title} onClose={onClose} surfaceClassName={styles.dialogBody}>
    <div className={styles.toolbar}>
      <strong>{app.title}</strong>
      <FabIconButton label="Close Mini App" onClick={onClose}><X size={16} /></FabIconButton>
    </div>
    <iframe
      ref={frameRef}
      className={styles.miniAppFrame}
      title={app.id}
      sandbox="allow-scripts allow-forms"
      src={app.url}
    />
  </FabDialog>;
}

export default function MiniAppCompatibilityAdapter({
  transport,
  onClose,
}: {
  readonly transport: MahayanaHostTransport;
  readonly onClose: () => void;
}) {
  const [query, setQuery] = useState('');
  const [apps, setApps] = useState<MarketplacePluginSummary[]>([]);
  const [installed, setInstalled] = useState<Record<string, InstalledPluginPointer>>({});
  const [busy, setBusy] = useState<Set<string>>(() => new Set());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [openApp, setOpenApp] = useState<{ id: string; title: string; url: string } | null>(null);

  function setBusyState(id: string, value: boolean) {
    setBusy((current) => {
      const next = new Set(current);
      if (value) next.add(id);
      else next.delete(id);
      return next;
    });
  }

  async function refresh(search = query) {
    setLoading(true);
    setError(null);
    try {
      await reconcileAccountMiniApps().catch(() => undefined);
      const [marketplace, local] = await Promise.all([
        transport.marketplaceBrowse(search),
        transport.pluginListInstalled(),
      ]);
      setApps(marketplace.plugins);
      setInstalled(Object.fromEntries(local.plugins.map((pointer) => [pointer.pluginId, pointer])));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    const timer = window.setTimeout(() => void refresh(query), query.trim() ? 240 : 0);
    return () => window.clearTimeout(timer);
  }, [query, transport]);

  async function install(app: MarketplacePluginSummary) {
    setBusyState(app.pluginId, true);
    setError(null);
    try {
      const release = await transport.marketplaceRelease(app.pluginId, app.latestVersion);
      if (release.releaseStatus && release.releaseStatus !== 'approved') {
        throw new Error(`Mini App release is not approved: ${release.releaseStatus}`);
      }
      if (release.releaseManifest?.protocol !== 'mahayana.external-release.v1') {
        throw new Error('Mini App release is missing a verified external release manifest');
      }
      const contract = release.install ?? (release.releaseManifest.install as Record<string, unknown> | undefined);
      const source = contract?.source as Record<string, unknown> | undefined;
      if (contract?.protocol !== 'fabushi.marketplace.install.v1'
        || contract.strategy !== 'github-immutable'
        || source?.marketplaceHostsPackage === true
        || typeof source?.repository !== 'string'
        || typeof source?.sourceRef !== 'string'
        || !source.sourceRef.trim()) {
        throw new Error('Mini App release is not a GitHub immutable package');
      }
      const pointer = await transport.pluginInstall(release.releaseManifest, 'desktop');
      try {
        await invokeNativeDesktop('addMiniAppToAccount', { pluginId: app.pluginId });
      } catch (cause) {
        await transport.pluginUninstall(app.pluginId).catch(() => undefined);
        throw cause;
      }
      setInstalled((current) => ({ ...current, [app.pluginId]: pointer }));
      await refresh(query);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusyState(app.pluginId, false);
    }
  }

  async function uninstall(pluginId: string) {
    setBusyState(pluginId, true);
    setError(null);
    try {
      await transport.pluginUninstall(pluginId);
      await invokeNativeDesktop('removeMiniAppFromAccount', { pluginId });
      if (openApp?.id === pluginId) setOpenApp(null);
      await refresh(query);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusyState(pluginId, false);
    }
  }

  async function open(app: MarketplacePluginSummary) {
    setBusyState(app.pluginId, true);
    setError(null);
    try {
      const active = installed[app.pluginId] ?? await transport.pluginActive(app.pluginId);
      if (!active) throw new Error('Install this Mini App before opening it.');
      const document = await transport.pluginUiDocument(app.pluginId);
      const html = cloudBridgeDocument(prepareDesktopMiniAppWebMcpDocument(app.pluginId, document.html));
      const url = await window.fabushi.registerMiniAppDocument(app.pluginId, html);
      setOpenApp({ id: app.pluginId, title: app.displayName, url });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusyState(app.pluginId, false);
    }
  }

  const visible = useMemo(() => apps, [apps]);

  return <CompatibilityFeatureFrame
    title="Mini Apps"
    description="Marketplace, immutable GitHub package install, WebMCP UI and account CloudStorage live outside the Agent shell."
    onClose={onClose}
  >
    <div className={styles.toolbar}>
      <FabInput value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search Mini Apps" aria-label="Search Mini Apps" />
      <FabButton type="button" variant="ghost" onClick={() => void refresh()}>Refresh</FabButton>
    </div>
    {loading ? <div className={styles.status}><FabSpinner label="Searching Mini Apps" /> Searching marketplace…</div> : null}
    {error ? <div className={styles.error} role="alert">{error}</div> : null}
    <div className={styles.cardGrid}>
      {visible.map((app) => {
        const pointer = installed[app.pluginId];
        const action = marketplaceInstallAction({
          id: app.pluginId,
          latestVersion: app.latestVersion,
          install: app.install,
          releaseManifest: app.releaseManifest,
        }, pointer ? { version: pointer.version, artifactSha256: pointer.artifactSha256 } : undefined);
        const needsInstall = action === 'install' || action === 'update' || action === 'reinstall';
        return <FabSurface key={app.pluginId} className={styles.card} elevated data-testid={`miniapp-market-${app.pluginId}`}>
          <FabAvatar identity={`miniapp:${app.pluginId}`} state={pointer ? 'idle' : 'offline'} size={44} label={app.displayName} />
          <strong>{app.displayName}</strong>
          <small>{app.description}</small>
          <span className={styles.meta}>{pointer ? `Installed ${pointer.version}` : `Online ${app.latestVersion}`}</span>
          <div className={styles.cardActions}>
            {pointer ? <FabButton type="button" variant="primary" disabled={busy.has(app.pluginId)} onClick={() => void open(app)}>Open</FabButton> : null}
            {needsInstall ? <FabButton type="button" disabled={busy.has(app.pluginId)} onClick={() => void install(app)}>
              {busy.has(app.pluginId) ? 'Working…' : marketplaceInstallActionLabel(action)}
            </FabButton> : null}
            {pointer ? <FabButton type="button" variant="danger" disabled={busy.has(app.pluginId)} onClick={() => void uninstall(app.pluginId)}>Uninstall</FabButton> : null}
          </div>
        </FabSurface>;
      })}
    </div>
    {!loading && !visible.length ? <div className={styles.status}>No matching Mini Apps.</div> : null}
    {openApp ? <MiniAppDialog app={openApp} onClose={() => setOpenApp(null)} /> : null}
  </CompatibilityFeatureFrame>;
}
