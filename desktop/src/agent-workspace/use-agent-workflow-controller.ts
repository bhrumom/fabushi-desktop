import { useCallback, useRef, useState } from 'react';
import type { RuntimeEvent, WorkflowSummary } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentCoordinatorClient } from './coordinator-client';

export interface AgentWorkflowControllerOptions {
  onListed?(agentId: string, workflows: readonly WorkflowSummary[]): void;
  onError?(agentId: string, message: string): void;
}

export interface AgentWorkflowController {
  readonly workflowsByAgentId: Readonly<Record<string, readonly WorkflowSummary[]>>;
  list(agentId: string): Promise<void>;
  handle(event: RuntimeEvent): boolean;
}

function requestId(agentId: string): string {
  return `agent-workflow:list:${agentId}:${crypto.randomUUID()}`;
}

/**
 * Owns Agent-scoped workflow discovery for Composer and workspace surfaces.
 *
 * Workflow state is indexed by runtime Agent identity and refreshed from
 * Mahayana events; the compatibility Messenger renderer only consumes the
 * projection and never owns the cache or raw workflow.list command.
 */
export function useAgentWorkflowController(
  client: AgentCoordinatorClient,
  options: AgentWorkflowControllerOptions = {},
): AgentWorkflowController {
  const [workflowsByAgentId, setWorkflowsByAgentId] = useState<Record<string, readonly WorkflowSummary[]>>({});
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const list = useCallback(async (agentId: string) => {
    try {
      await client.listWorkflows(requestId(agentId), agentId);
    } catch (cause) {
      optionsRef.current.onError?.(agentId, cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }, [client]);

  const handle = useCallback((event: RuntimeEvent): boolean => {
    if (event.type === 'workflow.changed') {
      void list(event.agentId).catch(() => {});
      return true;
    }
    if (event.type === 'workflow.listed') {
      setWorkflowsByAgentId((current) => ({ ...current, [event.agentId]: event.workflows }));
      optionsRef.current.onListed?.(event.agentId, event.workflows);
      return true;
    }
    return false;
  }, [list]);

  return { workflowsByAgentId, list, handle };
}
