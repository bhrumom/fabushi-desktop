import { useCallback, useRef } from 'react';
import type { RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  FABU_AGENT_MEMORY_INDEX_PATH,
  fabuAgentAutomationPath,
} from '../fabu-runtime/agent-store';
import type { AgentCoordinatorClient } from './coordinator-client';

export interface AgentStoreSyncControllerOptions {
  mirror(agentId: string, path: string, value: unknown): void | Promise<void>;
  remove(agentId: string, path: string): void | Promise<void>;
  onError?(agentId: string, message: string): void;
}

export interface AgentStoreSyncController {
  handle(event: RuntimeEvent): boolean;
}

/**
 * Keeps Agent-owned memory/automation objects mirrored into the existing
 * FabuAgentStore CAS graph without making the Messenger compatibility renderer
 * own mutation follow-up or object-path logic.
 */
export function useAgentStoreSyncController(
  client: AgentCoordinatorClient,
  options: AgentStoreSyncControllerOptions,
): AgentStoreSyncController {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const refreshMemory = useCallback(async (agentId: string) => {
    try {
      await client.listMemory(
        `agent-memory:list:${agentId}:${crypto.randomUUID()}`,
        agentId,
      );
    } catch (cause) {
      optionsRef.current.onError?.(agentId, cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }, [client]);

  const handle = useCallback((event: RuntimeEvent): boolean => {
    if (event.type === 'memory.changed') {
      // Re-list after every mutation so deletes/clears produce a complete
      // Agent-owned snapshot instead of an append-only cloud mirror.
      void refreshMemory(event.agentId).catch(() => {});
      return true;
    }

    if (event.type === 'memory.listed') {
      void optionsRef.current.mirror(event.agentId, FABU_AGENT_MEMORY_INDEX_PATH, {
        version: 1,
        count: event.count,
        memories: event.memories,
      });
      return true;
    }

    if (event.type === 'automation.changed') {
      const owner = event.automation.agentId;
      if (owner) {
        const path = fabuAgentAutomationPath(event.automation.id);
        if (event.action === 'deleted') void optionsRef.current.remove(owner, path);
        else void optionsRef.current.mirror(owner, path, event.automation);
      }
      return true;
    }

    if (event.type === 'automation.listed') {
      for (const automation of event.automations) {
        if (!automation.agentId) continue;
        void optionsRef.current.mirror(
          automation.agentId,
          fabuAgentAutomationPath(automation.id),
          automation,
        );
      }
      return true;
    }

    return false;
  }, [refreshMemory]);

  return { handle };
}
