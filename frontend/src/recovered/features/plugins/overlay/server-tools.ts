export interface McpToolSummary {
  name: string;
  title?: string;
  description?: string;
  isDisabled: boolean;
}

export interface PluginServerToolsSnapshot {
  status: "loading" | "ready" | "failed";
  tools: readonly McpToolSummary[];
  failure: unknown | null;
  pendingTool: string | null;
}

export interface PluginServerToolsController {
  getSnapshot(): PluginServerToolsSnapshot;
  subscribe(listener: () => void): () => void;
  open(): void;
  retry(): Promise<void>;
  toggle(toolName: string): Promise<void>;
  dispose(): void;
}

function cloneTools(tools: readonly McpToolSummary[]): McpToolSummary[] {
  return tools.map((tool) => ({ ...tool }));
}

export function togglePluginServerToolOptimistically(
  tools: readonly McpToolSummary[],
  toolName: string,
): McpToolSummary[] {
  return tools.map((tool) =>
    tool.name === toolName
      ? { ...tool, isDisabled: !tool.isDisabled }
      : { ...tool }
  );
}

export function createPluginServerToolsController(
  load: () => Promise<readonly McpToolSummary[]>,
  toggleRemote: (toolName: string) => Promise<readonly McpToolSummary[]>,
): PluginServerToolsController {
  let snapshot: PluginServerToolsSnapshot = {
    status: "loading",
    tools: [],
    failure: null,
    pendingTool: null,
  };
  let generation = 0;
  let request = 0;
  let opened = false;
  let disposed = false;
  const listeners = new Set<() => void>();

  const publish = (next: PluginServerToolsSnapshot): void => {
    if (disposed) return;
    snapshot = next;
    for (const listener of Array.from(listeners)) listener();
  };

  const current = (g: number): boolean =>
    !disposed && opened && generation === g;

  const refresh = async (): Promise<void> => {
    if (disposed || !opened) return;
    const g = generation;
    const r = ++request;
    publish({ ...snapshot, status: "loading", failure: null });
    try {
      const tools = await load();
      if (!current(g) || r !== request) return;
      publish({
        status: "ready",
        tools: cloneTools(tools),
        failure: null,
        pendingTool: snapshot.pendingTool,
      });
    } catch (failure) {
      if (current(g) && r === request) {
        publish({ ...snapshot, status: "failed", failure });
      }
      throw failure;
    }
  };

  return {
    getSnapshot() {
      return snapshot;
    },
    subscribe(listener) {
      if (disposed) return () => {};
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    open() {
      if (disposed || opened) return;
      opened = true;
      generation += 1;
      void refresh().catch(() => {});
    },
    retry: refresh,
    async toggle(toolName) {
      if (disposed || !opened || snapshot.pendingTool != null) return;
      const g = generation;
      publish({
        ...snapshot,
        status: "ready",
        failure: null,
        tools: togglePluginServerToolOptimistically(snapshot.tools, toolName),
        pendingTool: toolName,
      });
      try {
        const tools = await toggleRemote(toolName);
        if (current(g)) {
          publish({
            status: "ready",
            tools: cloneTools(tools),
            failure: null,
            pendingTool: null,
          });
        }
      } catch (failure) {
        if (current(g)) {
          publish({
            ...snapshot,
            status: "ready",
            failure,
            pendingTool: null,
          });
        }
        throw failure;
      }
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      opened = false;
      generation += 1;
      request += 1;
      listeners.clear();
    },
  };
}
