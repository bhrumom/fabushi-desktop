import React, { useEffect, useState } from 'react';
import { invokeNativeDesktop } from '../../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type {
  InferenceProvider,
  ProductHostSettings,
} from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import {
  FabButton,
  FabInput,
  FabSelect,
  FabSurface,
  FabSwitch,
} from '../../ui/primitives/fab-primitives';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import dataStyles from '../compatibility/compatibility-data.module.css';
import frameStyles from '../compatibility/compatibility-feature-frame.module.css';

const defaults: ProductHostSettings = {
  notifications: true,
  autoUpdateWhenIdle: true,
  localExecution: true,
  routeEgressLocally: false,
  securityKeys: false,
  webauthnProxyEnabled: false,
  localToolPermission: 'ask',
  remoteControlEnabled: false,
  aiComputerControlEnabled: true,
  autoReviewRules: [],
  inferenceProvider: 'fabushi',
  sandboxRuntime: 'host',
};

function requestId(prefix: string): string {
  return `${prefix}:${Date.now().toString(36)}:${crypto.randomUUID()}`;
}

export default function SettingsCompatibilityAdapter({
  transport,
  onClose,
  onLogout,
}: {
  readonly transport: MahayanaHostTransport;
  readonly onClose: () => void;
  readonly onLogout: () => Promise<void>;
}) {
  const [settings, setSettings] = useState<ProductHostSettings>(defaults);
  const [theme, setTheme] = useState<'system' | 'light' | 'dark'>('system');
  const [timeZone, setTimeZone] = useState('');
  const [localToolPermission, setLocalToolPermission] = useState<'always' | 'ask' | 'never'>('ask');
  const [error, setError] = useState<string | null>(null);
  const [logoutBusy, setLogoutBusy] = useState(false);

  useEffect(() => {
    const unsubscribe = transport.subscribe((event) => {
      if (event.type === 'settings.changed') setSettings(event.settings);
    });
    void transport.execute({
      type: 'settings.get',
      requestId: requestId('compat-settings-get'),
    } as Parameters<MahayanaHostTransport['execute']>[0]).catch((cause: unknown) => {
      setError(cause instanceof Error ? cause.message : String(cause));
    });
    void Promise.all([
      invokeNativeDesktop<{ preference?: 'system' | 'light' | 'dark' }>('getThemeState'),
      invokeNativeDesktop<string>('getTimeZone'),
      invokeNativeDesktop<'always' | 'ask' | 'never'>('getLocalToolPermission'),
    ]).then(([themeState, zone, permission]) => {
      setTheme(themeState.preference ?? 'system');
      setTimeZone(zone);
      setLocalToolPermission(permission);
    }).catch(() => {});
    return unsubscribe;
  }, [transport]);

  async function updateSetting<K extends keyof ProductHostSettings>(key: K, value: ProductHostSettings[K]) {
    const next = { ...settings, [key]: value };
    setSettings(next);
    setError(null);
    try {
      await transport.execute({
        type: 'settings.update',
        requestId: requestId('compat-settings-update'),
        settings: next,
      } as Parameters<MahayanaHostTransport['execute']>[0]);
    } catch (cause) {
      setSettings(settings);
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  return <CompatibilityFeatureFrame
    title="Settings"
    description="Account and product preferences are a compatibility/product feature; per-Agent profile and runtime settings remain Agent-owned."
    onClose={onClose}
  >
    <div className={frameStyles.stack}>
      {error ? <div className={dataStyles.error} role="alert">{error}</div> : null}

      <FabSurface elevated className={frameStyles.stack}>
        <strong>Appearance & locale</strong>
        <label>
          <span className={dataStyles.status}>Theme</span>
          <FabSelect value={theme} onChange={(event) => {
            const next = event.target.value as 'system' | 'light' | 'dark';
            setTheme(next);
            void invokeNativeDesktop('setThemePreference', { preference: next }).catch((cause: unknown) => {
              setError(cause instanceof Error ? cause.message : String(cause));
            });
          }}>
            <option value="system">Follow system</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </FabSelect>
        </label>
        <form className={dataStyles.composer} onSubmit={(event) => {
          event.preventDefault();
          void invokeNativeDesktop('setTimeZoneOverride', { timeZone: timeZone.trim() || null }).catch((cause: unknown) => {
            setError(cause instanceof Error ? cause.message : String(cause));
          });
        }}>
          <FabInput value={timeZone} onChange={(event) => setTimeZone(event.target.value)} placeholder="Timezone, e.g. America/Los_Angeles" />
          <FabButton type="submit">Save timezone</FabButton>
        </form>
      </FabSurface>

      <FabSurface elevated className={frameStyles.stack}>
        <strong>Execution & permissions</strong>
        <label>
          <span className={dataStyles.status}>Local tool permission</span>
          <FabSelect value={localToolPermission} onChange={(event) => {
            const next = event.target.value as 'always' | 'ask' | 'never';
            void invokeNativeDesktop<'always' | 'ask' | 'never'>('setLocalToolPermission', { permission: next })
              .then(setLocalToolPermission)
              .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)));
          }}>
            <option value="always">Always allow</option>
            <option value="ask">Ask every time</option>
            <option value="never">Never allow</option>
          </FabSelect>
        </label>
        <label>
          <span className={dataStyles.status}>Inference provider</span>
          <FabSelect value={settings.inferenceProvider} onChange={(event) => void updateSetting('inferenceProvider', event.target.value as InferenceProvider)}>
            <option value="fabushi">Fabushi</option>
            <option value="codex">Codex</option>
            <option value="claude-code">Claude</option>
            <option value="openrouter">OpenRouter</option>
          </FabSelect>
        </label>
        <label className={dataStyles.row}>
          <span className={dataStyles.copy}><strong>AI computer control</strong><small>Allows capability-gated local computer execution.</small></span>
          <span />
          <FabSwitch aria-label="AI computer control" checked={settings.aiComputerControlEnabled} onChange={(event) => void updateSetting('aiComputerControlEnabled', event.target.checked)} />
        </label>
        <label className={dataStyles.row}>
          <span className={dataStyles.copy}><strong>Remote computer control</strong><small>Enables the background remote-control service without moving it into React.</small></span>
          <span />
          <FabSwitch aria-label="Remote computer control" checked={settings.remoteControlEnabled} onChange={(event) => void updateSetting('remoteControlEnabled', event.target.checked)} />
        </label>
      </FabSurface>

      <FabSurface elevated className={frameStyles.stack}>
        <strong>Account</strong>
        <p>Signing out revokes the authenticated desktop session. Agent runtime ownership remains in Mahayana and is not stored in this adapter.</p>
        <div className={frameStyles.actions}>
          <FabButton
            type="button"
            variant="danger"
            disabled={logoutBusy}
            data-testid="settings-logout"
            data-agent-id="settings-logout"
            onClick={() => {
              if (logoutBusy) return;
              setLogoutBusy(true);
              void onLogout()
                .catch((cause: unknown) => setError(cause instanceof Error ? cause.message : String(cause)))
                .finally(() => setLogoutBusy(false));
            }}
          >
            {logoutBusy ? 'Signing out…' : 'Sign out'}
          </FabButton>
        </div>
      </FabSurface>
    </div>
  </CompatibilityFeatureFrame>;
}
