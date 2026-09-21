import { useCallback, useEffect, useRef, useState } from 'react';
import type { AgentCoordinatorClient } from './coordinator-client';
import {
  parsePullRequestToolResult,
  type AgentPullRequestSuggestion,
  type AgentPullRequestToolTarget,
} from './agent-composer-suggestion-provider';

function requestId(): string {
  return `agent-composer:pr:${crypto.randomUUID()}`;
}

function queryArguments(target: AgentPullRequestToolTarget): Record<string, unknown> {
  const normalized = target.tool.toLocaleLowerCase().replace(/[-.]/g, '_');
  if (normalized.startsWith('search_')) {
    return {
      query: 'is:pr is:open',
      topn: 50,
      limit: 50,
      per_page: 50,
    };
  }
  return {
    state: 'open',
    limit: 50,
    per_page: 50,
  };
}

export interface AgentPrSuggestionController {
  readonly candidates: readonly AgentPullRequestSuggestion[];
  readonly loading: boolean;
  refresh(): Promise<void>;
}

/**
 * Read-only GitHub MCP projection for Composer # references.
 *
 * The renderer never speaks to GitHub directly. It asks the Mahayana Host to
 * execute an explicitly read-only MCP PR tool and correlates the result by
 * requestId, keeping credentials and connector state outside React.
 */
export function useAgentPrSuggestionController(
  client: AgentCoordinatorClient,
  target: AgentPullRequestToolTarget | null,
): AgentPrSuggestionController {
  const [candidates, setCandidates] = useState<readonly AgentPullRequestSuggestion[]>([]);
  const [loading, setLoading] = useState(false);
  const generationRef = useRef(0);

  const refresh = useCallback(async () => {
    const generation = ++generationRef.current;
    if (!target) {
      setCandidates([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    try {
      const result = await client.callMcpTool(
        requestId(),
        target.server,
        target.tool,
        queryArguments(target),
      );
      if (generation === generationRef.current) {
        setCandidates(parsePullRequestToolResult(result));
      }
    } catch {
      if (generation === generationRef.current) setCandidates([]);
    } finally {
      if (generation === generationRef.current) setLoading(false);
    }
  }, [client, target]);

  useEffect(() => {
    void refresh();
    return () => { generationRef.current += 1; };
  }, [refresh]);

  return { candidates, loading, refresh };
}
