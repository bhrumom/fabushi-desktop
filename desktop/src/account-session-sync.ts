import type { AuthState } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  isElectronMahayanaHostAvailable,
  MAHAYANA_ACCOUNT_SESSION_RESET_EVENT,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';

const REAUTH_POLL_MS = 350;
const MESSENGER_WORKSPACE_SELECTOR = '[data-testid="messenger-workspace"]';

let installed = false;

/**
 * Keep the top-level DesktopShell auth gate synchronized with browser login
 * completions that happen after an explicit logout or terminal/revoked session.
 *
 * The same account-reset event is also emitted during the normal first-login
 * bootstrap when no Messenger workspace has existed yet. That path is already
 * handled by DesktopShell's own auth probe and must never trigger a renderer
 * reload. We therefore arm this synchronizer only when the reset is observed
 * while a real Messenger workspace is mounted.
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
        // If DesktopShell has already restored the workspace itself, there is
        // nothing to rebuild. Only the post-logout HostClient path needs one
        // reload to re-enter the authenticated top-level shell.
        if (!document.querySelector(MESSENGER_WORKSPACE_SELECTOR)) {
          window.location.reload();
        }
        return;
      }
    } catch {
      // Browser login and Host reconnect can briefly overlap. Keep the login
      // gate mounted and retry instead of surfacing a transient transport error.
    }
    schedule();
  };

  const onAccountSessionReset = () => {
    // Initial unauthenticated bootstrap emits the same cache-reset event. Do not
    // arm in that state; otherwise the first successful login races DesktopShell
    // and causes an unnecessary reload that detaches active controls.
    if (!document.querySelector(MESSENGER_WORKSPACE_SELECTOR)) {
      waitingForLogin = false;
      clearTimer();
      return;
    }
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
