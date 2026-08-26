import type { AuthState } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  isElectronMahayanaHostAvailable,
  MAHAYANA_ACCOUNT_SESSION_RESET_EVENT,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';

const REAUTH_POLL_MS = 350;

let installed = false;

/**
 * Keep the top-level DesktopShell auth gate synchronized with browser login
 * completions that happen inside HostClient after an explicit logout or a
 * terminal/revoked session.
 *
 * DesktopShell deliberately stops polling while a valid session is active.
 * When account-scoped state is reset it emits MAHAYANA_ACCOUNT_SESSION_RESET_EVENT;
 * from that point we poll only until the Rust Host reports a new authenticated
 * session, then reload once so every account-scoped transport/cache is rebuilt
 * from the new identity.
 */
export function installDesktopAccountSessionSync(): () => void {
  if (installed || typeof window === 'undefined' || !isElectronMahayanaHostAvailable()) {
    return () => {};
  }

  installed = true;
  let disposed = false;
  let waitingForLogin = false;
  let timer: number | null = null;

  const clearTimer = () => {
    if (timer !== null) {
      window.clearTimeout(timer);
      timer = null;
    }
  };

  const schedule = (delayMs = REAUTH_POLL_MS) => {
    if (disposed || !waitingForLogin || timer !== null) return;
    timer = window.setTimeout(() => {
      timer = null;
      void poll();
    }, delayMs);
  };

  const poll = async () => {
    if (disposed || !waitingForLogin || !window.mahayana) return;
    try {
      const state = await window.mahayana.invoke<AuthState>('feature.auth.status');
      if (disposed || !waitingForLogin) return;
      if (state.loggedIn) {
        waitingForLogin = false;
        clearTimer();
        window.location.reload();
        return;
      }
    } catch {
      // Browser login and Host reconnect can briefly overlap. Keep the login
      // gate mounted and retry instead of surfacing a transient transport error.
    }
    schedule();
  };

  const onAccountSessionReset = () => {
    waitingForLogin = true;
    clearTimer();
    schedule(0);
  };

  window.addEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, onAccountSessionReset);

  return () => {
    disposed = true;
    installed = false;
    waitingForLogin = false;
    clearTimer();
    window.removeEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, onAccountSessionReset);
  };
}
