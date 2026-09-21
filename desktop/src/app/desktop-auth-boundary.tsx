import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type {
  AuthState,
  BrowserLoginAttempt,
} from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { isTerminalAuthSessionFailure } from '../auth-session';
import { createDesktopAgentTransport } from '../bridge/agent-host';
import {
  nativeOnboardingSeen,
  rememberNativeOnboarding,
  subscribeNativeDesktopEvents,
} from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import FabAvatar from '../ui/avatar/fab-avatar';
import {
  FabBadge,
  FabButton,
  FabSpinner,
  FabSurface,
} from '../ui/primitives/fab-primitives';
import styles from './desktop-auth-boundary.module.css';

export interface DesktopAuthSession {
  readonly transport: MahayanaHostTransport;
  readonly onLogout: () => Promise<void>;
}

type AuthBootstrapState = 'connecting' | 'ready' | 'error';

const onboardingCopy = [
  {
    eyebrow: 'YOUR AGENT WORKSPACE',
    title: '不是聊天窗口。是一支会继续工作的团队。',
    detail: '把目标交给 Fabushi。Agent 会拆解任务、持续执行，并在真正需要你时回来。',
  },
  {
    eyebrow: 'SPECIALIZED AGENTS',
    title: '每个 Agent 都有自己的工作、记忆和节奏。',
    detail: '让不同 Agent 长期负责不同领域；身份、会话和运行状态都由 Mahayana Runtime 独立维护。',
  },
  {
    eyebrow: 'LOCAL-FIRST CONTROL',
    title: '能力留在明确的权限边界里。',
    detail: '电脑、文件、浏览器和协作能力由 Runtime 管理；登录凭据不会进入桌面聊天界面。',
  },
] as const;

export default function DesktopAuthBoundary({
  children,
}: {
  readonly children: (session: DesktopAuthSession) => React.ReactNode;
}) {
  const transport = useMemo(() => createDesktopAgentTransport(), []);
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);
  const [bootstrapState, setBootstrapState] = useState<AuthBootstrapState>('connecting');
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);
  const [onboardingSeen, setOnboardingSeen] = useState<boolean | null>(null);
  const [onboardingStep, setOnboardingStep] = useState(0);
  const [loginAttempt, setLoginAttempt] = useState<BrowserLoginAttempt | null>(null);
  const [loginBusy, setLoginBusy] = useState(false);
  const [loginError, setLoginError] = useState<string | null>(null);
  const transitionEpoch = useRef(0);

  const applyAuth = useCallback((state: AuthState) => {
    transitionEpoch.current += 1;
    setAuthenticated(state.loggedIn);
    setBootstrapState('ready');
    setBootstrapError(null);
    if (state.loggedIn) {
      setLoginAttempt(null);
      setLoginBusy(false);
      setLoginError(null);
    }
  }, []);

  const refreshAuth = useCallback(async () => {
    const epoch = transitionEpoch.current;
    try {
      const state = await transport.authStatus();
      if (epoch !== transitionEpoch.current) return;
      applyAuth(state);
    } catch (cause) {
      if (epoch !== transitionEpoch.current) return;
      if (isTerminalAuthSessionFailure(cause)) {
        transitionEpoch.current += 1;
        setAuthenticated(false);
        setBootstrapState('ready');
        setBootstrapError(null);
        return;
      }
      setBootstrapState('error');
      setBootstrapError(cause instanceof Error ? cause.message : String(cause));
    }
  }, [applyAuth, transport]);

  const resetToLogin = useCallback(async (revoke = true) => {
    const epoch = ++transitionEpoch.current;
    setLoginAttempt(null);
    setLoginBusy(false);
    setLoginError(null);
    try {
      if (revoke) await transport.logout();
    } catch {
      // Local sign-out remains authoritative when the remote edge is unavailable.
    } finally {
      if (epoch === transitionEpoch.current) {
        setAuthenticated(false);
        setBootstrapState('ready');
      }
    }
  }, [transport]);

  useEffect(() => {
    let closed = false;
    let retry: number | undefined;
    const bootstrap = async () => {
      const epoch = transitionEpoch.current;
      setBootstrapState('connecting');
      try {
        await transport.initialize({ profileId: 'default', mode: 'production' });
        if (closed || epoch !== transitionEpoch.current) return;
        const state = await transport.authStatus();
        if (closed || epoch !== transitionEpoch.current) return;
        applyAuth(state);
      } catch (cause) {
        if (closed || epoch !== transitionEpoch.current) return;
        if (isTerminalAuthSessionFailure(cause)) {
          setAuthenticated(false);
          setBootstrapState('ready');
          setBootstrapError(null);
          return;
        }
        setBootstrapState('error');
        setBootstrapError(cause instanceof Error ? cause.message : String(cause));
        retry = window.setTimeout(() => void bootstrap(), 1_800);
      }
    };
    void bootstrap();
    const unsubscribe = subscribeNativeDesktopEvents({
      'account-auth-changed': () => void refreshAuth(),
    });
    return () => {
      closed = true;
      if (retry) window.clearTimeout(retry);
      unsubscribe();
      void transport.close();
    };
  }, [applyAuth, refreshAuth, transport]);

  useEffect(() => {
    let closed = false;
    void nativeOnboardingSeen().then((seen) => {
      if (!closed) setOnboardingSeen(seen === true);
    });
    return () => { closed = true; };
  }, []);

  useEffect(() => {
    const attempt = loginAttempt;
    if (!attempt) return undefined;
    let closed = false;
    let timer: number | undefined;
    const baseDelay = Math.max(250, Math.min(2_000, attempt.pollAfterMs ?? 750));
    const schedule = (delay = baseDelay) => {
      if (!closed) timer = window.setTimeout(() => void poll(), delay);
    };
    const poll = async () => {
      if (closed) return;
      if (attempt.expiresAt && Date.now() / 1000 >= attempt.expiresAt) {
        setLoginAttempt(null);
        setLoginBusy(false);
        setLoginError('登录链接已过期，请重新开始。');
        return;
      }
      try {
        const result = await transport.browserLoginPoll(attempt.attemptId);
        if (closed) return;
        if (result.status === 'completed' && result.auth?.loggedIn) {
          applyAuth({ ...result.auth, loggedIn: true });
          return;
        }
        if (result.status === 'expired' || result.status === 'cancelled' || result.status === 'failed') {
          setLoginAttempt(null);
          setLoginBusy(false);
          setLoginError(
            result.status === 'cancelled'
              ? '登录已取消。'
              : result.status === 'expired'
                ? '登录链接已过期，请重新开始。'
                : '登录流程未完成，请重新开始。',
          );
          return;
        }
        schedule();
      } catch (cause) {
        if (closed) return;
        setLoginError(cause instanceof Error ? cause.message : String(cause));
        schedule(Math.min(2_000, baseDelay * 2));
      }
    };
    schedule(0);
    return () => {
      closed = true;
      if (timer) window.clearTimeout(timer);
    };
  }, [applyAuth, loginAttempt, transport]);

  const finishOnboarding = useCallback(() => {
    setOnboardingSeen(true);
    rememberNativeOnboarding();
  }, []);

  const beginBrowserLogin = useCallback(async () => {
    if (loginBusy) return;
    setLoginBusy(true);
    setLoginError(null);
    try {
      const attempt = await transport.browserLoginStart();
      setLoginAttempt(attempt);
      await transport.openExternal(attempt.loginUrl);
    } catch (cause) {
      setLoginAttempt(null);
      setLoginBusy(false);
      setLoginError(cause instanceof Error ? cause.message : String(cause));
    }
  }, [loginBusy, transport]);

  const reopenBrowserLogin = useCallback(async () => {
    const attempt = loginAttempt;
    if (!attempt) return;
    setLoginError(null);
    try {
      const reopened = await transport.browserLoginReopen(attempt.attemptId);
      if (reopened.status !== 'pending' || !reopened.loginUrl?.trim()) {
        throw new Error('登录会话已不可用，请重新开始。');
      }
      setLoginAttempt({
        ...attempt,
        loginUrl: reopened.loginUrl,
        pollAfterMs: reopened.pollAfterMs ?? attempt.pollAfterMs,
      });
      await transport.openExternal(reopened.loginUrl);
    } catch (cause) {
      setLoginError(cause instanceof Error ? cause.message : String(cause));
    }
  }, [loginAttempt, transport]);

  const cancelBrowserLogin = useCallback(async () => {
    const attempt = loginAttempt;
    if (!attempt) return;
    try {
      await transport.browserLoginCancel(attempt.attemptId);
    } catch {
      // A local cancel still ends this UI attempt.
    } finally {
      setLoginAttempt(null);
      setLoginBusy(false);
    }
  }, [loginAttempt, transport]);

  if (authenticated === true) {
    return <div className={styles.root} data-testid="desktop-shell">
      {children({ transport, onLogout: () => resetToLogin(true) })}
    </div>;
  }

  if (onboardingSeen === null || authenticated === null || bootstrapState === 'connecting') {
    return <div className={styles.root} data-testid="desktop-shell">
      <FabSurface className={styles.bootstrap} elevated data-testid="desktop-auth-bootstrap">
        <FabAvatar identity="fabushi:bootstrap" state="thinking" size={64} label="Fabushi" active />
        <FabBadge tone="accent">MAHAYANA RUNTIME</FabBadge>
        <strong>正在连接本机 Agent Runtime</strong>
        <p>认证、Agent 生命周期和能力权限都从本机 Runtime 建立，不由 React 猜测。</p>
        <FabSpinner label="Connecting" />
      </FabSurface>
    </div>;
  }

  if (bootstrapState === 'error') {
    return <div className={styles.root} data-testid="desktop-shell">
      <FabSurface className={styles.gate} elevated data-testid="desktop-auth-error">
        <FabAvatar identity="fabushi:auth-error" state="error" size={68} label="Fabushi" />
        <FabBadge tone="danger">RUNTIME UNAVAILABLE</FabBadge>
        <h1>无法连接本机 Runtime</h1>
        <p>{bootstrapError || '未知启动错误。'}</p>
        <FabButton variant="primary" onClick={() => void refreshAuth()}>重试</FabButton>
      </FabSurface>
    </div>;
  }

  if (!onboardingSeen) {
    const copy = onboardingCopy[onboardingStep];
    return <div className={styles.root} data-testid="desktop-shell">
      <FabSurface className={styles.gate} elevated data-testid="onboarding-gate">
        <div className={styles.brand}>
          <FabAvatar identity="fabushi:onboarding" state={onboardingStep === 1 ? 'working' : 'idle'} size={76} label="Fabushi" active={onboardingStep === 1} />
          <FabBadge tone="accent">{copy.eyebrow}</FabBadge>
        </div>
        <h1>{copy.title}</h1>
        <p>{copy.detail}</p>
        <div className={styles.progress} aria-label={`引导第 ${onboardingStep + 1} 步，共 3 步`}>
          {onboardingCopy.map((_, index) => <i key={index} data-active={index <= onboardingStep || undefined} />)}
        </div>
        <div className={styles.actions}>
          {onboardingStep > 0
            ? <FabButton data-testid="onboarding-back" onClick={() => setOnboardingStep((step) => Math.max(0, step - 1))}>返回</FabButton>
            : <span />}
          <FabButton
            variant="primary"
            data-testid="onboarding-next"
            onClick={() => {
              if (onboardingStep >= onboardingCopy.length - 1) finishOnboarding();
              else setOnboardingStep((step) => step + 1);
            }}
          >
            {onboardingStep >= onboardingCopy.length - 1 ? '开始使用 Fabushi' : '继续'}
          </FabButton>
        </div>
      </FabSurface>
    </div>;
  }

  return <div className={styles.root} data-testid="desktop-shell">
    <FabSurface className={styles.gate} elevated data-testid="login-gate">
      <div className={styles.brand}>
        <FabAvatar
          identity="fabushi:account"
          state={loginAttempt ? 'waiting' : 'idle'}
          size={82}
          label="Fabushi Account"
          active={Boolean(loginAttempt)}
        />
        <FabBadge tone={loginAttempt ? 'warning' : 'accent'}>
          {loginAttempt ? 'BROWSER AUTHORIZATION' : 'FABUSHI ACCOUNT'}
        </FabBadge>
      </div>
      {loginAttempt ? <>
        <h1 data-testid="browser-login-waiting">在浏览器完成登录</h1>
        <p>密码和第三方登录表单只留在 Fabushi Account Portal。完成授权后，本机 Runtime 会领取会话。</p>
        {loginError ? <output className={styles.error} role="status">{loginError}</output> : null}
        <div className={styles.actions}>
          <FabButton onClick={() => void cancelBrowserLogin()}>取消</FabButton>
          <FabButton variant="primary" data-testid="browser-login-reopen" onClick={() => void reopenBrowserLogin()}>重新打开浏览器</FabButton>
        </div>
      </> : <>
        <h1>登录 Fabushi</h1>
        <p>桌面端只发起一次性浏览器会话，不显示、接收或保存第三方登录密码。</p>
        {loginError ? <output className={styles.error} role="alert">{loginError}</output> : null}
        <FabButton
          variant="primary"
          data-testid="browser-login-start"
          disabled={loginBusy}
          onClick={() => void beginBrowserLogin()}
        >
          {loginBusy ? '正在打开浏览器…' : '在浏览器中登录'}
        </FabButton>
      </>}
    </FabSurface>
  </div>;
}
