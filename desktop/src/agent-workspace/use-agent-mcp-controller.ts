import { useCallback, useRef, useState } from 'react';
import type { RuntimeEvent } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentCoordinatorClient } from './coordinator-client';
import { findPullRequestReadTool, type AgentPullRequestToolTarget } from './agent-composer-suggestion-provider';

export interface AgentMcpToolReference {
  readonly id: string;
  readonly name: string;
  readonly description?: string;
  readonly disabled: boolean;
}

export interface AgentMcpReference {
  readonly id: string;
  readonly server: string;
  readonly name: string;
  readonly description?: string;
  readonly status?: string;
  readonly transport?: string;
  readonly customInstructions?: string;
  readonly tools: readonly AgentMcpToolReference[];
  readonly toolCount: number;
}

export interface AgentMcpControllerOptions {
  onError?(message: string): void;
}

export interface AgentMcpController {
  readonly references: readonly AgentMcpReference[];
  readonly pullRequestTool: AgentPullRequestToolTarget | null;
  readonly authorizationUrls: Readonly<Record<string, string>>;
  list(): Promise<void>;
  refresh(): Promise<void>;
  oauthLogin(server: string): Promise<void>;
  oauthLogout(server: string): Promise<void>;
  remove(server: string): Promise<void>;
  setCustomInstructions(server: string, instructions: string): Promise<void>;
  setToolEnabled(server: string, tool: string, enabled: boolean): Promise<void>;
  handle(event: RuntimeEvent): boolean;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function nonEmpty(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function booleanValue(value: unknown, fallback = false): boolean {
  return typeof value === 'boolean' ? value : fallback;
}

function normalizeTool(value: unknown): AgentMcpToolReference | null {
  if (!isRecord(value)) return null;
  const id = nonEmpty(value.id) ?? nonEmpty(value.name);
  const name = nonEmpty(value.displayName) ?? nonEmpty(value.name) ?? id;
  if (!id || !name) return null;
  return {
    id,
    name,
    ...(nonEmpty(value.description) ? { description: nonEmpty(value.description)! } : {}),
    disabled: booleanValue(value.disabled, value.enabled === false),
  };
}

export function normalizeAgentMcpServer(value: unknown): AgentMcpReference | null {
  if (!isRecord(value)) return null;
  const server = nonEmpty(value.serverIdentifier)
    ?? nonEmpty(value.server)
    ?? nonEmpty(value.id)
    ?? nonEmpty(value.name);
  const name = nonEmpty(value.displayName)
    ?? nonEmpty(value.name)
    ?? server;
  if (!server || !name) return null;

  const status = nonEmpty(value.status);
  const transport = nonEmpty(value.transport);
  const tools = Array.isArray(value.tools)
    ? value.tools.map(normalizeTool).filter((item): item is AgentMcpToolReference => item !== null)
    : [];
  const details = [
    status,
    tools.length > 0 ? `${tools.length} tool${tools.length === 1 ? '' : 's'}` : null,
    transport,
  ].filter((item): item is string => Boolean(item));

  return {
    id: server.startsWith('mcp:') ? server : `mcp:${server}`,
    server,
    name,
    ...(details.length ? { description: details.join(' · ') } : {}),
    ...(status ? { status } : {}),
    ...(transport ? { transport } : {}),
    ...(nonEmpty(value.customInstructions)
      ? { customInstructions: nonEmpty(value.customInstructions)! }
      : {}),
    tools,
    toolCount: tools.length,
  };
}

export function projectAgentMcpReferences(servers: readonly unknown[]): AgentMcpReference[] {
  return servers
    .map(normalizeAgentMcpServer)
    .filter((item): item is AgentMcpReference => item !== null)
    .filter((item, index, all) => all.findIndex((candidate) => candidate.id === item.id) === index);
}

function requestId(action: string): string {
  return `agent-mcp:${action}:${crypto.randomUUID()}`;
}

export function useAgentMcpController(
  client: AgentCoordinatorClient,
  options: AgentMcpControllerOptions = {},
): AgentMcpController {
  const [references, setReferences] = useState<readonly AgentMcpReference[]>([]);
  const [pullRequestTool, setPullRequestTool] = useState<AgentPullRequestToolTarget | null>(null);
  const [authorizationUrls, setAuthorizationUrls] = useState<Readonly<Record<string, string>>>({});
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const report = useCallback((cause: unknown) => {
    const message = cause instanceof Error ? cause.message : String(cause);
    optionsRef.current.onError?.(message);
    return cause;
  }, []);

  const list = useCallback(async () => {
    try {
      await client.listMcpServers(requestId('list'));
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const refresh = useCallback(async () => {
    try {
      await client.refreshMcpServers(requestId('refresh'));
      await list();
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, list, report]);

  const oauthLogin = useCallback(async (server: string) => {
    try {
      await client.mcpOauthLogin(requestId('oauth-login'), server);
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, report]);

  const oauthLogout = useCallback(async (server: string) => {
    try {
      await client.mcpOauthLogout(requestId('oauth-logout'), server);
      setAuthorizationUrls((current) => {
        const next = { ...current };
        delete next[server];
        return next;
      });
      await list();
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, list, report]);

  const remove = useCallback(async (server: string) => {
    try {
      await client.removeMcpServer(requestId('remove'), server);
      setAuthorizationUrls((current) => {
        const next = { ...current };
        delete next[server];
        return next;
      });
      await list();
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, list, report]);

  const setCustomInstructions = useCallback(async (server: string, instructions: string) => {
    try {
      await client.setMcpCustomInstructions(requestId('instructions'), server, instructions);
      await list();
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, list, report]);

  const setToolEnabled = useCallback(async (server: string, tool: string, enabled: boolean) => {
    try {
      await client.setMcpToolDisabled(requestId('tool'), server, tool, !enabled);
      await list();
    } catch (cause) {
      report(cause);
      throw cause;
    }
  }, [client, list, report]);

  const handle = useCallback((event: RuntimeEvent): boolean => {
    if (event.type === 'mcp.listed') {
      setReferences(projectAgentMcpReferences(event.servers));
      setPullRequestTool(findPullRequestReadTool(event.servers));
      return true;
    }
    if (event.type === 'mcp.oauth') {
      setAuthorizationUrls((current) => {
        const next = { ...current };
        if (event.removed || !event.authorizationUrl) delete next[event.server];
        else next[event.server] = event.authorizationUrl;
        return next;
      });
      void list().catch(() => {});
      return true;
    }
    if (event.type === 'mcp.refreshed') {
      void list().catch(() => {});
      return true;
    }
    return false;
  }, [list]);

  return {
    references,
    pullRequestTool,
    authorizationUrls,
    list,
    refresh,
    oauthLogin,
    oauthLogout,
    remove,
    setCustomInstructions,
    setToolEnabled,
    handle,
  };
}
