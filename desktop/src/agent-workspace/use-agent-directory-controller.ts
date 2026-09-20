import { useCallback, useRef, useState } from 'react';
import type { BotSummary, RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentCoordinatorClient } from './coordinator-client';

export interface AgentDirectoryControllerOptions {
  onListed?(agents: readonly BotSummary[]): void;
  onChanged?(action: string, agent: BotSummary): void;
  onError?(message: string): void;
}

export interface AgentDirectoryController {
  readonly agents: BotSummary[];
  list(): Promise<unknown>;
  create(input: { name: string; description?: string }): Promise<unknown>;
  update(
    id: string,
    patch: {
      name?: string;
      title?: string;
      description?: string;
      avatar?: string;
      avatarShape?: string;
      avatarColor?: string;
      notifyOnUpdates?: boolean;
      notificationsEnabled?: boolean;
    },
  ): Promise<unknown>;
  duplicate(id: string): Promise<unknown>;
  delete(id: string): Promise<unknown>;
  setHidden(id: string, hidden: boolean): Promise<unknown>;
  handle(event: RuntimeEvent): boolean;
}

function requestId(action: string): string {
  return `agent-directory:${action}:${crypto.randomUUID()}`;
}

function upsertAgent(current: readonly BotSummary[], agent: BotSummary): BotSummary[] {
  const index = current.findIndex((candidate) => candidate.id === agent.id);
  if (index < 0) return [...current, agent];
  return current.map((candidate, candidateIndex) => candidateIndex === index ? agent : candidate);
}

/**
 * Product-owned Agent directory boundary.
 *
 * Mahayana still speaks the existing bot.* wire protocol for compatibility,
 * but React surfaces consume Agent directory semantics and never own bot
 * command construction or bot.listed/bot.changed cache reduction.
 */
export function useAgentDirectoryController(
  client: AgentCoordinatorClient,
  initialAgents: readonly BotSummary[] = [],
  options: AgentDirectoryControllerOptions = {},
): AgentDirectoryController {
  const [agents, setAgents] = useState<BotSummary[]>(() => [...initialAgents]);
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const report = useCallback((cause: unknown) => {
    optionsRef.current.onError?.(cause instanceof Error ? cause.message : String(cause));
  }, []);

  const list = useCallback(async () => {
    try {
      return await client.listAgents(requestId('list'));
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const create = useCallback(async (input: { name: string; description?: string }) => {
    try {
      return await client.createAgent(requestId('create'), input);
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const update = useCallback(async (
    id: string,
    patch: {
      name?: string;
      title?: string;
      description?: string;
      avatar?: string;
      avatarShape?: string;
      avatarColor?: string;
      notifyOnUpdates?: boolean;
      notificationsEnabled?: boolean;
    },
  ) => {
    try {
      return await client.updateAgent(requestId('update'), id, patch);
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const duplicate = useCallback(async (id: string) => {
    try {
      return await client.duplicateAgent(requestId('duplicate'), id);
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const remove = useCallback(async (id: string) => {
    try {
      return await client.deleteAgent(requestId('delete'), id);
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const setHidden = useCallback(async (id: string, hidden: boolean) => {
    try {
      return await client.setAgentHidden(requestId('hidden'), id, hidden);
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const handle = useCallback((event: RuntimeEvent): boolean => {
    if (event.type === 'bot.listed') {
      setAgents([...event.bots]);
      optionsRef.current.onListed?.(event.bots);
      return true;
    }
    if (event.type === 'bot.changed') {
      setAgents((current) => event.action === 'deleted'
        ? current.filter((agent) => agent.id !== event.bot.id)
        : upsertAgent(current, event.bot));
      optionsRef.current.onChanged?.(event.action, event.bot);
      return true;
    }
    return false;
  }, []);

  return {
    agents,
    list,
    create,
    update,
    duplicate,
    delete: remove,
    setHidden,
    handle,
  };
}
