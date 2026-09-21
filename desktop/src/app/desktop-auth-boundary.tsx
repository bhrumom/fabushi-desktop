import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import HostClient from '../../../frontend/apps/web/src/app/host/host-client';
import type { AuthState } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { isTerminalAuthSessionFailure } from '../auth-session';
import { createDesktopAgentTransport } from '../bridge/agent-host';
import FabAvatar from '../ui/avatar/fab-avatar';
import { FabSpinner, FabSurface } from '../ui/primitives/fab-primitives';
import styles from './desktop-auth-boundary.module.css';

export interface DesktopAuthSession {
  readonly transport: MahayanaHostTransport;
  readonly onLogout: () => Promise<void>;
}

export default function DesktopAuthBoundary({
  children,
}: {
  readonly children: (session: DesktopAuthSession) => React.ReactNode;
}) {
  const transport = useMemo(() => createDesktopAgentTransport(), []);
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);
  const transitionEpoch = useRef(0);

  const resetToLogin = useCallback(async (revoke = true) => {
    const epoch = ++transitionEpoch.current;
    try {
      if (revoke) await transport.logout();
    } catch {
      // Local sign-out remains authoritative when the remote edge is unavailable.
    } finally {
      if (epoch === transitionEpoch.current) setAuthenticated(false);
    }
  }, [transport]);

  const acceptHostAuth = useCallback((state: AuthState) => {
    if (!state.loggedIn) return;
    transitionEpoch.current += 1;
    setAuthenticated(true);
  }, []);

  useEffect(() => {
    let closed = false;
    let retry: number | undefined;
    const check = async () => {
      const epoch = transitionEpoch.current;
      try {
        const state = await transport.authStatus();
        if (closed || epoch !== transitionEpoch.current) return;
        setAuthenticated(state.loggedIn);
        if (!state.loggedIn) retry = window.setTimeout(() => void check(), 900);
      } catch (cause) {
        if (closed) return;
        if (isTerminalAuthSessionFailure(cause)) {
          await resetToLogin(true);
          return;
        }
        retry = window.setTimeout(() => void check(), 1_800);
      }
    };
    void check();
    return () => {
      closed = true;
      if (retry) window.clearTimeout(retry);
      void transport.close();
    };
  }, [resetToLogin, transport]);

  if (authenticated === false) {
    return <div className={styles.root} data-testid="desktop-shell">
      <HostClient onAuthStateChange={acceptHostAuth} />
    </div>;
  }

  if (authenticated !== true) {
    return <div className={styles.root} data-testid="desktop-shell">
      <FabSurface className={styles.bootstrap} elevated>
        <FabAvatar identity="fabushi:bootstrap" state="thinking" size={64} label="Fabushi" active />
        <strong>Mahayana</strong>
        <p>Connecting to the local Agent runtime…</p>
        <FabSpinner label="Connecting" />
      </FabSurface>
    </div>;
  }

  return <div className={styles.root} data-testid="desktop-shell">
    {children({ transport, onLogout: () => resetToLogin(true) })}
  </div>;
}
