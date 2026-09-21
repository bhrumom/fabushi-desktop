import { useCallback, useEffect, useRef, useState } from 'react';
import type {
  ComputerControlLeaseState,
  ComputerStatus,
  RuntimeEvent,
} from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { invokeNativeDesktop, subscribeNativeDesktopEvents } from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { RemoteComputerDesktopState } from '../../../frontend/apps/web/src/lib/remote-computer/desktop-peer';
import { AgentCoordinatorClient } from './coordinator-client';

export interface UseAgentComputerControllerOptions {
  readonly hostReady: boolean;
  readonly hydrated: boolean;
  readonly accountScope: string | null;
  readonly activePeerKey: string | null;
  readonly transport: MahayanaHostTransport;
  readonly coordinatorClient: AgentCoordinatorClient;
  readonly label: string;
  readonly remoteControlEnabled: boolean;
  resolveAgentId(requestedAgentId: string): string | null;
  onError(message: string): void;
}

export interface AgentComputerControlProjection {
  readonly agentId: string;
  readonly leaseId: string;
  readonly lease: ComputerControlLeaseState;
}

export interface AgentComputerController {
  readonly open: boolean;
  readonly state: RemoteComputerDesktopState | null;
  readonly capabilityStatus: ComputerStatus | null;
  readonly control: AgentComputerControlProjection | null;
  readonly online: boolean;
  readonly status: string;
  readonly handleCapabilityStatus: (status: ComputerStatus) => void;
  handleRuntimeEvent(event: RuntimeEvent): boolean;
  refreshCapability(): void;
  openForAgent(agentId: string, source?: string): void;
  toggleForAgent(agentId: string, source?: string): void;
  close(): void;
  refreshPairingCode(): void;
  approveSession(sessionId: string): void;
  denySession(sessionId: string): void;
  disconnect(): void;
  takeControl(agentId: string, operationId?: string | null): void;
  releaseControl(): void;
  openControlPage(agentId: string): void;
}

let computerRequestSequence = 0;

function nextComputerRequestId(): string {
  computerRequestSequence += 1;
  return `computer-status:${Date.now().toString(36)}:${computerRequestSequence.toString(36)}`;
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

function acceptBackgroundState(
  payload: unknown,
  controlEnabled: boolean,
  update: (state: RemoteComputerDesktopState) => void,
): void {
  const state = payload && typeof payload === 'object' && !Array.isArray(payload)
    ? payload as Record<string, unknown>
    : {};
  const running = state.running === true;
  const deviceId = typeof state.deviceId === 'string' ? state.deviceId : '';
  const error = typeof state.error === 'string' && state.error.trim() ? state.error : undefined;
  update({
    running,
    controlEnabled,
    deviceId,
    clients: [],
    sessions: [],
    connectionState: running ? 'connected' : 'idle',
    channelOpen: false,
    ...(error ? { error } : {}),
  });
}

/**
 * Owns the installed-machine Computer lifecycle for Agent workspaces.
 *
 * The renderer may decide when the Computer surface is visible, but it must not
 * own registration, pairing, capability refresh, remote-session authorization
 * or any renderer-owned remote-session controller lifetime. Every Agent resolves through the
 * same installed Fabushi machine while retaining its own agentId surface scope.
 */
export function useAgentComputerController(
  options: UseAgentComputerControllerOptions,
): AgentComputerController {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const [open, setOpen] = useState(false);
  const [state, setState] = useState<RemoteComputerDesktopState | null>(null);
  const [capabilityStatus, setCapabilityStatus] = useState<ComputerStatus | null>(null);
  const [control, setControl] = useState<AgentComputerControlProjection | null>(null);
  const stateRef = useRef<RemoteComputerDesktopState | null>(null);
  stateRef.current = state;

  const reportError = useCallback((cause: unknown) => {
    optionsRef.current.onError(errorMessage(cause));
  }, []);

  const handleCapabilityStatus = useCallback((status: ComputerStatus) => {
    setCapabilityStatus(status);
  }, []);

  const handleRuntimeEvent = useCallback((event: RuntimeEvent): boolean => {
    if (event.type !== 'computer.controlChanged') return false;
    setControl(event.active && event.lease
      ? { agentId: event.agentId, leaseId: event.leaseId, lease: event.lease }
      : null);
    return true;
  }, []);

  const refreshCapability = useCallback(() => {
    if (!optionsRef.current.hostReady) return;
    void optionsRef.current.coordinatorClient
      .refreshComputerStatus(nextComputerRequestId())
      .catch(() => {});
  }, []);

  const reportOpened = useCallback((agentId: string, source: string) => {
    void invokeNativeDesktop('reportOpenComputer', {
      source,
      agentId,
      connected: stateRef.current?.channelOpen === true,
    }).catch(() => {});
  }, []);

  const openForAgent = useCallback((agentId: string, source = 'agent-workspace') => {
    setOpen(true);
    refreshCapability();
    reportOpened(agentId, source);
  }, [refreshCapability, reportOpened]);

  const toggleForAgent = useCallback((agentId: string, source = 'agent-workspace') => {
    setOpen((current) => {
      const next = !current;
      if (next) {
        refreshCapability();
        reportOpened(agentId, source);
      }
      return next;
    });
  }, [refreshCapability, reportOpened]);

  const close = useCallback(() => setOpen(false), []);

  const refreshPairingCode = useCallback(() => {
    void invokeNativeDesktop('refreshRemoteComputerBackground')
      .then((payload) => acceptBackgroundState(payload, optionsRef.current.remoteControlEnabled, setState))
      .catch(reportError);
  }, [reportError]);

  // Session authorization moved out of React/WebRTC. App-owned remote-device
  // agents perform the gateway lifecycle in Main; no renderer session should
  // ever reach these compatibility callbacks.
  const approveSession = useCallback((_sessionId: string) => {}, []);
  const denySession = useCallback((_sessionId: string) => {}, []);
  const disconnect = useCallback(() => {}, []);

  const takeControl = useCallback((agentId: string, operationId?: string | null) => {
    const resolved = optionsRef.current.resolveAgentId(agentId);
    if (!resolved) {
      reportError(new Error(`Unknown Agent: ${agentId}`));
      return;
    }
    const leaseId = operationId?.trim() || `human-takeover:${crypto.randomUUID()}`;
    void optionsRef.current.coordinatorClient
      .takeComputerControl(nextComputerRequestId(), resolved, leaseId)
      .catch(reportError);
  }, [reportError]);

  const releaseControl = useCallback(() => {
    const current = control;
    if (!current) return;
    void optionsRef.current.coordinatorClient
      .releaseComputerControl(nextComputerRequestId(), current.agentId, current.leaseId)
      .catch(reportError);
  }, [control, reportError]);

  const openControlPage = useCallback((agentId: string) => {
    void invokeNativeDesktop('openExternal', {
      url: `https://fabushi.ombhrum.com/remote-computer?agentId=${encodeURIComponent(agentId)}`,
    }).catch(reportError);
  }, [reportError]);

  useEffect(() => {
    if (!options.hostReady || !options.accountScope || !options.hydrated) return;
    let disposed = false;
    const accept = (payload: unknown) => {
      if (disposed) return;
      acceptBackgroundState(payload, optionsRef.current.remoteControlEnabled, setState);
    };
    const unsubscribe = subscribeNativeDesktopEvents({
      'remote-computer-background-state': accept,
    });
    void invokeNativeDesktop('getRemoteComputerBackgroundState').then(accept).catch(reportError);
    return () => {
      disposed = true;
      unsubscribe();
    };
  }, [options.hostReady, options.hydrated, options.accountScope, reportError]);

  useEffect(() => {
    if (options.hostReady) refreshCapability();
  }, [options.hostReady, refreshCapability]);

  useEffect(() => {
    setOpen(false);
  }, [options.activePeerKey]);

  const online = Boolean(state?.running);
  const status = capabilityStatus && !capabilityStatus.available
    ? '本机控制不可用'
    : capabilityStatus && (!capabilityStatus.accessibilityGranted || !capabilityStatus.screenRecordingGranted)
      ? '需要系统权限'
      : state?.channelOpen
        ? '正在远程控制'
        : online
          ? options.remoteControlEnabled ? '在线，等待连接' : '在线，仅可发现'
          : state?.running ? '正在注册' : capabilityStatus?.available ? '本机可用' : '离线';

  return {
    open,
    state,
    capabilityStatus,
    control,
    online,
    status,
    handleCapabilityStatus,
    handleRuntimeEvent,
    refreshCapability,
    openForAgent,
    toggleForAgent,
    close,
    refreshPairingCode,
    approveSession,
    denySession,
    disconnect,
    takeControl,
    releaseControl,
    openControlPage,
  };
}
