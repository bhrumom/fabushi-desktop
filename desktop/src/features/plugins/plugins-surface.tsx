import { ExternalLink, Plug, RefreshCcw, Trash2, X } from 'lucide-react';
import React, { useEffect, useMemo, useState } from 'react';
import type {
  AgentMcpController,
  AgentMcpReference,
} from '../../agent-workspace/use-agent-mcp-controller';
import {
  FabButton,
  FabIconButton,
  FabInput,
  FabSpinner,
  FabSurface,
} from '../../ui/primitives/fab-primitives';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import styles from '../compatibility/compatibility-data.module.css';

function connected(server: AgentMcpReference): boolean {
  const status = (server.status || '').toLowerCase();
  return ['connected', 'ready', 'authenticated', 'running'].some((value) => status.includes(value));
}

export default function PluginsSurface({
  mcp,
  onClose,
}: {
  readonly mcp: AgentMcpController;
  readonly onClose: () => void;
}) {
  const [query, setQuery] = useState('');
  const [busy, setBusy] = useState<Set<string>>(() => new Set());
  const [instructions, setInstructions] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void mcp.list().catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)));
  }, [mcp.list]);

  useEffect(() => {
    setInstructions((current) => {
      const next = { ...current };
      for (const server of mcp.references) {
        if (!(server.server in next)) next[server.server] = server.customInstructions || '';
      }
      return next;
    });
  }, [mcp.references]);

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return mcp.references;
    return mcp.references.filter((server) =>
      [server.name, server.server, server.description, server.status, server.transport]
        .filter(Boolean)
        .some((value) => String(value).toLowerCase().includes(needle)),
    );
  }, [mcp.references, query]);

  function setBusyState(key: string, value: boolean) {
    setBusy((current) => {
      const next = new Set(current);
      if (value) next.add(key);
      else next.delete(key);
      return next;
    });
  }

  async function run(key: string, action: () => Promise<void>) {
    setBusyState(key, true);
    setError(null);
    try {
      await action();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusyState(key, false);
    }
  }

  return <CompatibilityFeatureFrame
    title="Plugins"
    description="MCP servers, accounts, OAuth, tools and per-server instructions."
    onClose={onClose}
  >
    <div className={styles.toolbar}>
      <FabInput
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Search Plugins and MCP servers"
        aria-label="Search Plugins and MCP servers"
      />
      <FabButton type="button" variant="ghost" onClick={() => void run('refresh', mcp.refresh)}>
        <RefreshCcw size={15} /> Refresh
      </FabButton>
      <FabIconButton label="Close Plugins" onClick={onClose}><X size={16} /></FabIconButton>
    </div>

    {busy.has('refresh') ? <div className={styles.status}><FabSpinner label="Refreshing Plugins" /> Refreshing MCP catalog…</div> : null}
    {error ? <div className={styles.error} role="alert">{error}</div> : null}

    <div className={styles.cardGrid}>
      {visible.map((server) => {
        const isConnected = connected(server);
        const authorizationUrl = mcp.authorizationUrls[server.server];
        return <FabSurface key={server.id} className={styles.card} elevated data-testid={`plugin-server-${server.server}`}>
          <div className={styles.toolbar}>
            <Plug size={18} />
            <strong>{server.name}</strong>
          </div>
          <small>{server.server}</small>
          <span className={styles.meta}>
            {[server.status || 'unknown', server.transport, `${server.toolCount} tools`].filter(Boolean).join(' · ')}
          </span>

          <div className={styles.cardActions}>
            <FabButton
              type="button"
              variant={isConnected ? 'ghost' : 'primary'}
              disabled={busy.has(server.server)}
              onClick={() => void run(
                server.server,
                () => isConnected ? mcp.oauthLogout(server.server) : mcp.oauthLogin(server.server),
              )}
            >
              {isConnected ? 'Sign out' : 'Connect'}
            </FabButton>
            {authorizationUrl ? <FabButton
              type="button"
              variant="ghost"
              onClick={() => void window.fabushi.openExternal(authorizationUrl)}
            >
              <ExternalLink size={14} /> Open authorization
            </FabButton> : null}
            <FabButton
              type="button"
              variant="danger"
              disabled={busy.has(server.server)}
              onClick={() => void run(server.server, () => mcp.remove(server.server))}
            >
              <Trash2 size={14} /> Remove
            </FabButton>
          </div>

          <label>
            <small>Custom instructions</small>
            <textarea
              value={instructions[server.server] || ''}
              onChange={(event) => setInstructions((current) => ({
                ...current,
                [server.server]: event.target.value,
              }))}
              rows={3}
              aria-label={`Custom instructions for ${server.name}`}
            />
          </label>
          <FabButton
            type="button"
            variant="ghost"
            disabled={busy.has(`${server.server}:instructions`)}
            onClick={() => void run(
              `${server.server}:instructions`,
              () => mcp.setCustomInstructions(server.server, instructions[server.server] || ''),
            )}
          >
            Save instructions
          </FabButton>

          {server.tools.length ? <div>
            <strong>Tools</strong>
            {server.tools.map((tool) => <div key={tool.id} className={styles.toolbar}>
              <span title={tool.description}>{tool.name}</span>
              <FabButton
                type="button"
                variant="ghost"
                disabled={busy.has(`${server.server}:${tool.id}`)}
                onClick={() => void run(
                  `${server.server}:${tool.id}`,
                  () => mcp.setToolEnabled(server.server, tool.id, tool.disabled),
                )}
              >
                {tool.disabled ? 'Enable' : 'Disable'}
              </FabButton>
            </div>)}
          </div> : <span className={styles.meta}>No tools reported by this server.</span>}
        </FabSurface>;
      })}
    </div>

    {!visible.length && !busy.has('refresh')
      ? <div className={styles.status}>No MCP servers match this search.</div>
      : null}
  </CompatibilityFeatureFrame>;
}
