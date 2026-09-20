import { useCallback, useRef, useState } from 'react';
import type { RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentCoordinatorClient } from './coordinator-client';

export interface AgentMcpReference {
  readonly id: string;
  readonly name: string;
  readonly description?: string;
  readonly status?: string;
  readonly toolCount: number;
}

export interface AgentMcpControllerOptions {
  onError?(message: string): void;
}

export interface AgentMcpController {
  readonly references: readonly AgentMcpReference[];
  list(): Promise<void>;
  handle(event: RuntimeEvent): boolean;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function nonEmpty(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function normalizeServer(value: unknown): AgentMcpReference | null {
  if (!isRecord(value)) return null;
  const id = nonEmpty(value.id)
    ?? nonEmpty(value.serverIdentifier)
    ?? nonEmpty(value.name);
  const name = nonEmpty(value.displayName)
    ?? nonEmpty(value.name)
    ?? nonEmpty(value.serverIdentifier)
    ?? id;
  if (!id || !name) return null;

  const status = nonEmpty(value.status);
  const transport = nonEmpty(value.transport);
  const toolCount = Array.isArray(value.tools) ? value.tools.length : 0;
  const details = [
    status,
    toolCount > 0 ? `${toolCount} tool${toolCount === 1 ? '' : 's'}` : null,
    transport,
  ].filter((item): item is string => Boolean(item));

  return {
    id: id.startsWith('mcp:') ? id : `mcp:${id}`,
    name,
    ...(details.length ? { description: details.join(' · ') } : {}),
    ...(status ? { status } : {}),
    toolCount,
  };
}

function requestId(): string {
  return `agent-mcp:list:${crypto.randomUUID()}`;
}

/**
 * Agent-owned MCP reference catalog.
 *
 * Mahayana keeps the account-level MCP connection catalog; this controller
 * normalizes its deliberately-untyped wire payload into stable references that
 * can be attached to an Agent draft. Raw mcp.* events never become renderer
 * global state and the Composer only receives the normalized projection.
 */
export function useAgentMcpController(
  client: AgentCoordinatorClient,
  options: AgentMcpControllerOptions = {},
): AgentMcpController {
  const [references, setReferences] = useState<readonly AgentMcpReference[]>([]);
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const list = useCallback(async () => {
    try {
      await client.listMcpServers(requestId());
    } catch (cause) {
      optionsRef.current.onError?.(cause instanceof Error ? cause.message : String(cause));
      throw cause;
    }
  }, [client]);

  const handle = useCallback((event: RuntimeEvent): boolean => {
    if (event.type === 'mcp.listed') {
      const next = event.servers
        .map(normalizeServer)
        .filter((item): item is AgentMcpReference => item !== null)
        .filter((item, index, all) => all.findIndex((candidate) => candidate.id === item.id) === index);
      setReferences(next);
      return true;
    }
    if (event.type === 'mcp.refreshed' || event.type === 'mcp.oauth') {
      void list().catch(() => {});
      return true;
    }
    return false;
  }, [list]);

  return { references, list, handle };
}
