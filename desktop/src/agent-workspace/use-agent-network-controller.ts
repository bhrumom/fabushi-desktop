import { useCallback, useRef, useState } from 'react';
import type { AgentPeerMessage, GroupSummary, RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentCoordinatorClient } from './coordinator-client';

export interface AgentNetworkController {
  readonly open: boolean;
  readonly broadcastMode: boolean;
  readonly groups: readonly GroupSummary[];
  readonly peerMessages: readonly AgentPeerMessage[];
  openNetwork(): void;
  openBroadcast(): void;
  close(): void;
  refreshGroups(): Promise<void>;
  refreshPeerHistory(agentId: string): Promise<void>;
  createGroup(name: string, memberAgentIds: readonly string[]): Promise<void>;
  updateGroup(id: string, patch: { name?: string; memberAgentIds?: readonly string[] }): Promise<void>;
  deleteGroup(id: string): Promise<void>;
  sendGroup(id: string, message: string): Promise<void>;
  sendPeer(fromAgentId: string, targetAgentId: string, message: string, priority?: boolean): Promise<void>;
  broadcast(message: string, targetAgentIds?: readonly string[]): Promise<void>;
  handle(event: RuntimeEvent): boolean;
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
  const [groups, setGroups] = useState<readonly GroupSummary[]>([]);
  const [peerMessages, setPeerMessages] = useState<readonly AgentPeerMessage[]>([]);
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

  const refreshPeerHistory = useCallback(
    (agentId: string) => client.listAgentPeerHistory(requestId('peer-history'), agentId).then(() => undefined),
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

  const sendPeer = useCallback(async (
    fromAgentId: string,
    targetAgentId: string,
    message: string,
    priority = false,
  ) => {
    try {
      await client.sendAgentPeer(requestId('peer-send'), fromAgentId, targetAgentId, message, priority);
    } catch (cause) {
      onErrorRef.current?.(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }, [client]);

  const broadcast = useCallback(async (message: string, targetAgentIds?: readonly string[]) => {
    try {
      await client.broadcast(requestId('broadcast'), message, targetAgentIds);
    } catch (cause) {
      onErrorRef.current?.(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }, [client]);

  const handle = useCallback((event: RuntimeEvent): boolean => {
    if (event.type === 'group.listed') {
      setGroups(event.groups);
      // Compatibility Messenger still consumes group.listed.
      return false;
    }
    if (event.type === 'group.changed') {
      setGroups((current) => event.action === 'deleted'
        ? current.filter((group) => group.id !== event.group.id)
        : current.some((group) => group.id === event.group.id)
          ? current.map((group) => group.id === event.group.id ? event.group : group)
          : [...current, event.group]);
      // Keep the compatibility Messenger group projection alive during migration.
      return false;
    }
    if (event.type === 'agent.peerHistory') {
      setPeerMessages(event.messages);
      return true;
    }
    if (event.type === 'agent.peerMessage') {
      setPeerMessages((current) => {
        const next = current.filter((message) => message.id !== event.message.id);
        next.push(event.message);
        return next.sort((left, right) => left.createdAtMs - right.createdAtMs).slice(-500);
      });
      return true;
    }
    return false;
  }, []);

  return {
    open,
    broadcastMode,
    groups,
    peerMessages,
    openNetwork,
    openBroadcast,
    close,
    refreshGroups,
    refreshPeerHistory,
    createGroup,
    updateGroup,
    deleteGroup,
    sendGroup,
    sendPeer,
    broadcast,
    handle,
  };
}
