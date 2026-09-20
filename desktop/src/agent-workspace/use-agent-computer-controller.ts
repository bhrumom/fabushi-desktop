import { useCallback, useEffect, useRef, useState } from 'react';
import type { ComputerStatus } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { invokeNativeDesktop } from '../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import {
  RemoteComputerDesktopController,
  type RemoteComputerDesktopState,
} from '../../../frontend/apps/web/src/lib/remote-computer/desktop-peer';
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

export interface AgentComputerController {
  readonly open: boolean;
  readonly state: RemoteComputerDesktopState | null;
  readonly capabilityStatus: ComputerStatus | null;
  readonly online: boolean;
  readonly status: string;
  readonly handleCapabilityStatus: (status: ComputerStatus) => void;
  refreshCapability(): void;
  openForAgent(agentId: string, source?: string): void;
  toggleForAgent(agentId: string, source?: string): void;
  close(): void;
  refreshPairingCode(): void;
  approveSession(sessionId: string): void;
  denySession(sessionId: string): void;
  disconnect(): void;
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

/**
 * Owns the installed-machine Computer lifecycle for Agent workspaces.
 *
 * The renderer may decide when the Computer surface is visible, but it must not
 * own registration, pairing, capability refresh, remote-session authorization
 * or RemoteComputerDesktopController lifetime. Every Agent resolves through the
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
  const controllerRef = useRef<RemoteComputerDesktopController | null>(null);
  const stateRef = useRef<RemoteComputerDesktopState | null>(null);
  stateRef.current = state;

  const reportError = useCallback((cause: unknown) => {
    optionsRef.current.onError(errorMessage(cause));
  }, []);

  const handleCapabilityStatus = useCallback((status: ComputerStatus) => {
    setCapabilityStatus(status);
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
    void controllerRef.current?.refreshPairingCode().catch(reportError);
  }, [reportError]);

  const approveSession = useCallback((sessionId: string) => {
    void controllerRef.current?.approvePendingSession(sessionId).catch(reportError);
  }, [reportError]);

  const denySession = useCallback((sessionId: string) => {
    void controllerRef.current?.denyPendingSession(sessionId).catch(reportError);
  }, [reportError]);

  const disconnect = useCallback(() => {
    void controllerRef.current?.disconnectActive().catch(reportError);
  }, [reportError]);

  const openControlPage = useCallback((agentId: string) => {
    void invokeNativeDesktop('openExternal', {
      url: `https://fabushi.ombhrum.com/remote-computer?agentId=${encodeURIComponent(agentId)}`,
    }).catch(reportError);
  }, [reportError]);

  useEffect(() => {
    if (!options.hostReady || !options.accountScope || !options.hydrated) return;

    let disposed = false;
    const controller = new RemoteComputerDesktopController({
      transport: options.transport,
      label: options.label,
      identityScope: options.accountScope,
      controlEnabled: options.remoteControlEnabled,
      resolveAgentId: (requestedAgentId) => optionsRef.current.resolveAgentId(requestedAgentId),
      onState: (nextState) => {
        if (!disposed) setState(nextState);
      },
    });

    controllerRef.current = controller;
    const snapshot = controller.snapshot();
    stateRef.current = snapshot;
    setState(snapshot);
    void controller.start().catch(reportError);

    return () => {
      disposed = true;
      if (controllerRef.current === controller) controllerRef.current = null;
      void controller.stop();
    };
  }, [
    options.hostReady,
    options.hydrated,
    options.accountScope,
    options.transport,
    options.label,
    reportError,
  ]);

  useEffect(() => {
    if (!options.hostReady) return;
    const controller = controllerRef.current;
    if (!controller) return;
    void controller.setControlEnabled(options.remoteControlEnabled).catch(reportError);
  }, [options.hostReady, options.remoteControlEnabled, reportError]);

  useEffect(() => {
    if (options.hostReady) refreshCapability();
  }, [options.hostReady, refreshCapability]);

  useEffect(() => {
    setOpen(false);
  }, [options.activePeerKey]);

  const online = Boolean(state?.running && state.registration);
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
    online,
    status,
    handleCapabilityStatus,
    refreshCapability,
    openForAgent,
    toggleForAgent,
    close,
    refreshPairingCode,
    approveSession,
    denySession,
    disconnect,
    openControlPage,
  };
}
