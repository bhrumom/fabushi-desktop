import { useCallback, useRef, useState } from 'react';
import type { AgentCoordinatorClient } from './coordinator-client';

export interface AgentNetworkController {
  readonly open: boolean;
  readonly broadcastMode: boolean;
  openNetwork(): void;
  openBroadcast(): void;
  close(): void;
  refreshGroups(): Promise<void>;
  createGroup(name: string, memberAgentIds: readonly string[]): Promise<void>;
  updateGroup(id: string, patch: { name?: string; memberAgentIds?: readonly string[] }): Promise<void>;
  deleteGroup(id: string): Promise<void>;
  sendGroup(id: string, message: string): Promise<void>;
  broadcast(message: string, targetAgentIds?: readonly string[]): Promise<void>;
}

function requestId(scope: string): string {
  return `agent-network:${scope}:${crypto.randomUUID()}`;
}

/**
 * Agent-owned collaboration UI/controller boundary.
 *
 * React Messenger compatibility code may decide where to place the surface,
 * but it no longer owns Network open mode or Mahayana group/broadcast commands.
 */
export function useAgentNetworkController(
  client: AgentCoordinatorClient,
  onError?: (message: string) => void,
): AgentNetworkController {
  const [open, setOpen] = useState(false);
  const [broadcastMode, setBroadcastMode] = useState(false);
  const onErrorRef = useRef(onError);
  onErrorRef.current = onError;

  const openNetwork = useCallback(() => {
    setBroadcastMode(false);
    setOpen(true);
  }, []);

  const openBroadcast = useCallback(() => {
    setBroadcastMode(true);
    setOpen(true);
  }, []);

  const close = useCallback(() => {
    setOpen(false);
    setBroadcastMode(false);
  }, []);

  const refreshGroups = useCallback(
    () => client.listGroups(requestId('group-list')).then(() => undefined),
    [client],
  );

  const createGroup = useCallback(
    (name: string, memberAgentIds: readonly string[]) =>
      client.createGroup(requestId('group-create'), name, memberAgentIds).then(() => undefined),
    [client],
  );

  const updateGroup = useCallback(
    (id: string, patch: { name?: string; memberAgentIds?: readonly string[] }) =>
      client.updateGroup(requestId('group-update'), id, patch).then(() => undefined),
    [client],
  );

  const deleteGroup = useCallback(
    (id: string) => client.deleteGroup(requestId('group-delete'), id).then(() => undefined),
    [client],
  );

  const sendGroup = useCallback(
    (id: string, message: string) => client.sendGroup(requestId('group-send'), id, message).then(() => undefined),
    [client],
  );

  const broadcast = useCallback(async (message: string, targetAgentIds?: readonly string[]) => {
    try {
      await client.broadcast(requestId('broadcast'), message, targetAgentIds);
    } catch (cause) {
      onErrorRef.current?.(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }, [client]);

  return {
    open,
    broadcastMode,
    openNetwork,
    openBroadcast,
    close,
    refreshGroups,
    createGroup,
    updateGroup,
    deleteGroup,
    sendGroup,
    broadcast,
  };
}
