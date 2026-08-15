import { StrictMode, useCallback, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import HostClient from '../../frontend/apps/web/src/app/host/host-client';
import './styles.css';

type MarketplacePlugin = {
  pluginId: string;
  displayName?: string;
  description?: string;
  latestVersion?: string;
  platforms?: string[];
};

type MarketplaceResponse = { plugins: MarketplacePlugin[] };

type InstalledPluginPointer = {
  pluginId: string;
  version: string;
  artifactId: string;
  artifactSha256: string;
  runtime: string;
  entry?: string;
  requestedPermissions: string[];
  installedPath: string;
};

type PluginPermissionStatus = {
  requested: string[];
  granted: string[];
  missing: string[];
};

type CompatibilityReport = {
  portableCompatible: boolean;
  mobileCompatible: boolean;
  supportedModules: string[];
  capabilityRequiredModules: string[];
  desktopOnlyModules: string[];
  unsupportedModules: string[];
  nativeAddons: string[];
  commonjsRequireFiles: string[];
};

function compatibilityBlockers(report: CompatibilityReport): string[] {
  return [
    ...report.capabilityRequiredModules.map((item) => `需 Host capability: ${item}`),
    ...report.desktopOnlyModules.map((item) => `仅桌面: ${item}`),
    ...report.unsupportedModules.map((item) => `当前未兼容: ${item}`),
    ...report.nativeAddons.map((item) => `Node native addon: ${item}`),
    ...report.commonjsRequireFiles.map((item) => `动态 CommonJS require: ${item}`),
  ];
}

function PluginRuntimeApp() {
  const [plugins, setPlugins] = useState<MarketplacePlugin[]>([]);
  const [query, setQuery] = useState('');
  const [message, setMessage] = useState('Mahayana Rust Host 正在启动');
  const [busy, setBusy] = useState<string | null>(null);
  const [runtimeTools, setRuntimeTools] = useState<string[]>([]);
  const [surface, setSurface] = useState<{ pluginId: string; html: string } | null>(null);

  const refresh = useCallback(async (search: string) => {
    try {
      const result = await window.fabushi.invoke<MarketplaceResponse>('marketplace.browse', {
        query: search.trim() || null,
        platform: 'desktop',
      });
      setPlugins(result.plugins ?? []);
      setMessage('Electron + Mahayana Rust Host 已连接');
    } catch (error) {
      setMessage(`市场加载失败：${String(error)}`);
    }
  }, []);

  const refreshTools = useCallback(async () => {
    try {
      setRuntimeTools(await window.fabushi.invoke<string[]>('runtime.tools'));
    } catch {
      setRuntimeTools([]);
    }
  }, []);

  useEffect(() => {
    void refresh('');
    void refreshTools();
  }, [refresh, refreshTools]);

  const grantRequestedPermissions = useCallback(async (installed: InstalledPluginPointer) => {
    if (!installed.requestedPermissions?.length) return true;
    let status = await window.fabushi.invoke<PluginPermissionStatus>('plugin.permissions', {
      pluginId: installed.pluginId,
    });
    if (!status.missing.length) return true;
    const approved = window.confirm(
      `${installed.pluginId} 请求以下权限：\n\n${status.missing.join('\n')}\n\n是否授权？`,
    );
    if (!approved) {
      setMessage(`已安装，但未授权：${status.missing.join('、')}`);
      return false;
    }
    for (const permission of status.missing) {
      status = await window.fabushi.invoke<PluginPermissionStatus>('plugin.permission.grant', {
        pluginId: installed.pluginId,
        permission,
      });
    }
    return status.missing.length === 0;
  }, []);

  const installPlugin = useCallback(async (plugin: MarketplacePlugin) => {
    if (!plugin.latestVersion) {
      setMessage(`${plugin.pluginId} 没有可安装版本`);
      return;
    }
    setBusy(plugin.pluginId);
    try {
      setMessage(`正在解析 ${plugin.pluginId}@${plugin.latestVersion}…`);
      const metadata = await window.fabushi.invoke<Record<string, unknown>>('marketplace.release', {
        pluginId: plugin.pluginId,
        version: plugin.latestVersion,
      });
      const release = metadata.releaseManifest;
      if (!release || typeof release !== 'object') throw new Error('marketplace release has no releaseManifest');
      const installed = await window.fabushi.invoke<InstalledPluginPointer>('plugin.install', {
        release,
        platform: 'desktop',
      });
      if (!(await grantRequestedPermissions(installed))) return;

      if (['deepseek-js', 'javascript', 'cordis-js'].includes(installed.runtime)) {
        const compatibility = await window.fabushi.invoke<CompatibilityReport>('plugin.compatibility', {
          pluginId: installed.pluginId,
        });
        if (!compatibility.portableCompatible) {
          setMessage(`已安装但未启动：${compatibilityBlockers(compatibility).join('；')}`);
          return;
        }
        await window.fabushi.invoke('runtime.start', { pluginId: installed.pluginId, config: {} });
        await refreshTools();
      } else if (installed.runtime === 'local-web') {
        const document = await window.fabushi.invoke<{ pluginId: string; html: string }>('plugin.uiDocument', {
          pluginId: installed.pluginId,
        });
        const policy = `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; script-src 'unsafe-inline'; font-src data:">`;
        setSurface({ pluginId: document.pluginId, html: `${policy}${document.html}` });
      }
      setMessage(`已安装 ${installed.pluginId}@${installed.version} · ${installed.runtime}`);
    } catch (error) {
      setMessage(`安装失败：${String(error)}`);
    } finally {
      setBusy(null);
    }
  }, [grantRequestedPermissions, refreshTools]);

  return (
    <main className="app-shell" data-testid="app-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">MAHAYANA RUST HOST</p>
          <h1>全球法布施</h1>
        </div>
        <span className="runtime-badge" data-testid="runtime-badge">Electron 43 · Rust</span>
      </header>

      <section className="hero-card">
        <h2>本地插件市场</h2>
        <p>Electron 只负责桌面 UI；插件安装、权限、兼容性与运行时继续由共享 Rust Host 管理。</p>
        <form className="search-row" onSubmit={(event) => { event.preventDefault(); void refresh(query); }}>
          <input
            aria-label="搜索插件"
            data-testid="marketplace-search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索插件"
          />
          <button type="submit" data-testid="marketplace-search-submit">搜索</button>
        </form>
      </section>

      <section className="status-card" aria-live="polite">
        <strong>Host 状态</strong>
        <span data-testid="host-status">{message}</span>
        {runtimeTools.length > 0 && <small>Runtime tools: {runtimeTools.join(' · ')}</small>}
      </section>

      <section className="plugin-grid" aria-label="插件列表">
        {plugins.map((plugin) => (
          <article className="plugin-card" key={plugin.pluginId} data-testid={`plugin-${plugin.pluginId}`}>
            <div>
              <p className="plugin-id">{plugin.pluginId}</p>
              <h3>{plugin.displayName || plugin.pluginId}</h3>
              <p>{plugin.description || '无描述'}</p>
              <small>{plugin.latestVersion || '无版本'}</small>
            </div>
            <div className="plugin-actions">
              <button
                disabled={busy === plugin.pluginId}
                onClick={() => void installPlugin(plugin)}
                data-testid={`install-${plugin.pluginId}`}
              >
                {busy === plugin.pluginId ? '处理中…' : '安装 / 更新'}
              </button>
            </div>
          </article>
        ))}
        {plugins.length === 0 && <div className="empty-state">没有匹配的桌面插件。</div>}
      </section>

      {surface && (
        <section className="surface-layer" role="dialog" aria-modal="true" aria-label={`${surface.pluginId} 插件界面`}>
          <header className="surface-toolbar">
            <strong>{surface.pluginId}</strong>
            <button type="button" onClick={() => setSurface(null)} data-testid="close-plugin-surface">关闭</button>
          </header>
          <iframe
            title={`${surface.pluginId} plugin surface`}
            data-testid="plugin-surface"
            sandbox="allow-scripts"
            srcDoc={surface.html}
          />
        </section>
      )}
    </main>
  );
}

function App() {
  const [surface, setSurface] = useState<'host' | 'plugins'>('host');
  return (
    <div className="desktop-root">
      <nav className="desktop-mode-switch" aria-label="桌面功能区">
        <button
          type="button"
          className={surface === 'host' ? 'active' : ''}
          onClick={() => setSurface('host')}
          data-testid="open-agent-host"
        >
          Agent Host
        </button>
        <button
          type="button"
          className={surface === 'plugins' ? 'active' : ''}
          onClick={() => setSurface('plugins')}
          data-testid="open-plugin-runtime"
        >
          插件 Runtime
        </button>
      </nav>
      <section className="desktop-surface" data-surface={surface}>
        {surface === 'host' ? <HostClient /> : <PluginRuntimeApp />}
      </section>
    </div>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
